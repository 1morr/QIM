//! Viterbi + beam 的獨立對照（plan §2.3）：小輸入上窮舉每一種切法，照 plan 的式子算分，
//! 核心的首選要是最高分的路徑之一。bigram 讓下一段的分數看前一段的最後一個字，beam 窄時
//! 可能剪掉之後會勝出的部分路徑，所以這裡給夠寬的 beam（大於可能的部分路徑數）。

use std::collections::{BTreeMap, BTreeSet};

use proptest::prelude::*;
use qim_core::{
    CharModel, CodeEntry, CodeTable, Engine, Key, LanguageModel, LanguageModelPack, Lexicon,
    LexiconPack, Packs, Ranking, Scheme, SchemePack, Session, UserModel, Word,
};

/// 夠寬的 beam：5 碼的輸入、每碼最多 3 字加 2 詞，部分路徑數遠小於它。
const WIDE_BEAM: u16 = 10_000;

/// 隨機資料：碼表（碼 → 字，可能有超過最長碼的碼）、詞庫（碼 → 詞）、語言模型、打分參數。
#[derive(Debug, Clone)]
struct Data {
    table: BTreeMap<String, Vec<char>>,
    words: BTreeMap<String, Vec<(String, u64)>>,
    chars: BTreeMap<char, (f64, f64, f64)>,
    bigrams: BTreeMap<(char, char), f64>,
    ranking: Ranking,
}

/// 只用十個字，bigram 才常常碰得到。
const CHARS: &str = "一二三四五六七八九十";

fn digit() -> impl Strategy<Value = char> {
    prop::sample::select(CHARS.chars().collect::<Vec<_>>())
}

fn data() -> impl Strategy<Value = Data> {
    let table = prop::collection::btree_map(
        "[abc]{1,3}",
        prop::collection::btree_set(digit(), 1..4)
            .prop_map(|set| set.into_iter().collect::<Vec<_>>()),
        1..8,
    );
    let words = prop::collection::btree_map(
        "[abc]{2,4}",
        prop::collection::vec(("[一二三四五六七八九十]{2,3}", 1u64..1000), 1..3),
        0..5,
    );
    let log_p = || -8.0..0.0f64;
    let chars = prop::collection::btree_map(digit(), (log_p(), log_p(), log_p()), 0..10);
    let bigrams = prop::collection::btree_map((digit(), digit()), log_p(), 0..20);
    let ranking = (-3.0..3.0f64, 0.0..3.0f64).prop_map(|(b, g)| Ranking {
        word_bonus: b,
        segment_penalty: g,
        beam: WIDE_BEAM,
        sentences: 3,
    });
    (table, words, chars, bigrams, ranking).prop_map(|(table, words, chars, bigrams, ranking)| {
        Data {
            table,
            words,
            chars,
            bigrams,
            ranking,
        }
    })
}

fn language_model(data: &Data) -> LanguageModel {
    let model = |&(start, continuation, backoff): &(f64, f64, f64)| CharModel {
        start,
        continuation,
        backoff,
    };
    LanguageModel::new(
        data.chars.iter().map(|(&c, m)| (c, model(m))),
        model(&UNKNOWN),
        data.bigrams.iter().map(|(&(a, b), &p)| (a, b, p)),
    )
}

fn engine(data: &Data) -> Engine {
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abc".into(),
        max_code_length: 2,
        radicals: BTreeMap::new(),
        packs: Packs {
            lexicon: Some("lexicon".into()),
            language_model: Some("bigram".into()),
            ..Packs::default()
        },
        ranking: data.ranking,
    };
    let code_table = CodeTable::from_entries(data.table.iter().map(|(code, chars)| {
        let entries = chars
            .iter()
            .map(|&text| CodeEntry {
                text,
                extension: false,
            })
            .collect();
        (code.clone(), entries)
    }));
    let words = data.words.iter().map(|(code, list)| {
        let list = list
            .iter()
            .map(|(text, frequency)| Word {
                text: text.clone(),
                frequency: *frequency,
            })
            .collect();
        (code.clone(), list)
    });
    let lexicon = LexiconPack {
        id: "lexicon".into(),
        lexicon: Lexicon::new(words, []),
    };
    let language_model = LanguageModelPack {
        id: "bigram".into(),
        model: language_model(data),
    };
    Engine::new(
        SchemePack { scheme, code_table },
        Some(lexicon),
        Some(language_model),
    )
    .unwrap()
}

/// 一段可用的字詞：碼、文字、不看前文的分數（plan §2.3）。字 −γ；詞 β − γ 加上
/// `ln(詞頻 ÷ 詞頻總和) − ln P_unigram(第一字)`，詞頻總和每個詞算一次。
fn pieces(data: &Data) -> Vec<(String, String, f64)> {
    let r = data.ranking;
    let mut pieces = Vec::new();
    for (code, chars) in data.table.iter().filter(|(code, _)| code.len() <= 2) {
        for c in chars {
            pieces.push((code.clone(), c.to_string(), -r.segment_penalty));
        }
    }
    let mut seen = BTreeSet::new();
    let total: f64 = data
        .words
        .values()
        .flatten()
        .filter(|(text, _)| seen.insert(text.clone()))
        .map(|(_, f)| *f as f64)
        .sum();
    for (code, list) in &data.words {
        for (text, f) in list {
            let first = text.chars().next().unwrap();
            let rest = (*f as f64 / total).ln() - stats(data, first).0;
            pieces.push((
                code.clone(),
                text.clone(),
                rest + r.word_bonus - r.segment_penalty,
            ));
        }
    }
    pieces
}

const UNKNOWN: (f64, f64, f64) = (-9.0, -9.0, 0.0);

/// 字的（unigram, 接續, 退回）；沒有統計的字用 [`UNKNOWN`]。
fn stats(data: &Data, c: char) -> (f64, f64, f64) {
    *data.chars.get(&c).unwrap_or(&UNKNOWN)
}

/// 照退回式查 `ln P(c | prev)`：留下的 bigram 用它的值（量化過的值只有核心知道，向它要），
/// 沒留下的用 α(前一字) + P_cont(字)，開頭用 unigram。
fn transition(data: &Data, model: &LanguageModel, prev: Option<char>, c: char) -> f64 {
    match prev {
        None => stats(data, c).0,
        Some(p)
            if data.chars.contains_key(&p)
                && data.chars.contains_key(&c)
                && data.bigrams.contains_key(&(p, c)) =>
        {
            model.log_prob(Some(p), c)
        }
        Some(p) => stats(data, p).2 + stats(data, c).1,
    }
}

/// 窮舉 `input` 的每一種切法：蓋滿整串的（文字, 分數）。每段的分數是前一段最後一個字接到
/// 這段第一個字的 bigram，加上這段不看前文的分數。
fn every_path(
    data: &Data,
    model: &LanguageModel,
    input: &str,
    prev: Option<char>,
    pieces: &[(String, String, f64)],
) -> Vec<(String, f64)> {
    if input.is_empty() {
        return vec![(String::new(), 0.0)];
    }
    let mut paths = Vec::new();
    for (code, text, score) in pieces {
        if let Some(rest) = input.strip_prefix(code.as_str()) {
            let first = text.chars().next().unwrap();
            let here = transition(data, model, prev, first) + score;
            let last = text.chars().next_back();
            for (tail, tail_score) in every_path(data, model, rest, last, pieces) {
                paths.push((format!("{text}{tail}"), here + tail_score));
            }
        }
    }
    paths
}

proptest! {
    /// 有前文時開頭的字接前文最後一字的 bigram；前文的最後一字沒有統計時照沒有前文（plan §2.3）。
    #[test]
    fn top_candidate_is_a_highest_scoring_path_over_every_segmentation(
        data in data(),
        input in "[abc]{1,5}",
        before in prop::option::of(digit()),
    ) {
        let model = language_model(&data);
        let prev = before.filter(|c| data.chars.contains_key(c));
        let paths = every_path(&data, &model, &input, prev, &pieces(&data));
        let engine = engine(&data);
        let mut user = UserModel::default();
        let mut session = Session::default();
        if let Some(before) = before {
            session.set_context(&format!("前文{before}"));
        }
        let mut snapshot = None;
        for c in input.chars() {
            snapshot = Some(session.handle(&engine, &mut user, Key::Letter(c)));
        }
        let top = snapshot.unwrap().candidates.first().map(|c| c.text.clone());
        let Some(best) = paths.iter().map(|(_, s)| *s).max_by(f64::total_cmp) else {
            // 沒有路徑蓋滿整串：首選只可能是開頭一段，不在這裡比。
            return Ok(());
        };
        let winners: BTreeSet<&str> = paths
            .iter()
            .filter(|(_, s)| (best - s).abs() < 1e-6)
            .map(|(t, _)| t.as_str())
            .collect();
        let top = top.unwrap_or_default();
        prop_assert!(winners.contains(top.as_str()), "首選 {top}，最高分的是 {winners:?}");
    }
}

proptest! {
    /// 選了只蓋住開頭的候選後，剩下輸入碼的首選是接在選定分段最後一字後面的最高分路徑
    /// （plan §2.3），不是從組字開頭用 unigram 算的。
    #[test]
    fn after_selecting_a_leading_segment_the_top_follows_its_last_character(
        data in data(),
        input in "[abc]{2,5}",
        pick in 0usize..20,
    ) {
        let engine = engine(&data);
        let mut user = UserModel::default();
        let mut session = Session::default();
        for c in input.chars() {
            session.handle(&engine, &mut user, Key::Letter(c));
        }
        let count = session.all_candidates().len();
        prop_assume!(count > 0);
        let snapshot = session.select(&engine, &mut user, pick % count);
        // 選的蓋滿整串就上屏了，沒有剩下的輸入碼。
        prop_assume!(snapshot.commit.is_none());
        let composition = &snapshot.composition;
        let last = composition.selected_text().chars().next_back();
        let model = language_model(&data);
        let paths = every_path(&data, &model, composition.unselected_input(), last, &pieces(&data));
        let Some(best) = paths.iter().map(|(_, s)| *s).max_by(f64::total_cmp) else {
            return Ok(());
        };
        let winners: BTreeSet<&str> = paths
            .iter()
            .filter(|(_, s)| (best - s).abs() < 1e-6)
            .map(|(t, _)| t.as_str())
            .collect();
        let top = snapshot.candidates.first().map(|c| c.text.clone()).unwrap_or_default();
        prop_assert!(winners.contains(top.as_str()), "首選 {top}，最高分的是 {winners:?}");
    }
}
