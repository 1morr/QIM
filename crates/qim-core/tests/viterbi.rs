//! Viterbi + beam 的獨立對照（plan §2.3）：小輸入上窮舉每一種切法，照 plan 寫的 unigram 式子算分，
//! 核心的首選要是最高分的路徑之一；beam 再窄也一樣，因為首選只靠每一欄的第一名接出來。

use std::collections::{BTreeMap, BTreeSet, HashSet};

use proptest::prelude::*;
use qim_core::{
    CodeEntry, CodeTable, Engine, Key, Lexicon, LexiconPack, Pack, Packs, Ranking, Scheme, Session,
    UserModel, Word,
};

/// 隨機資料：碼表（碼 → 字，可能有超過最長碼的碼）、詞庫（碼 → 詞與詞頻）、字頻、打分參數。
#[derive(Debug, Clone)]
struct Data {
    table: BTreeMap<String, Vec<char>>,
    words: BTreeMap<String, Vec<(String, u64)>>,
    chars: BTreeMap<char, u64>,
    ranking: Ranking,
}

fn data() -> impl Strategy<Value = Data> {
    let table = prop::collection::btree_map(
        "[abc]{1,3}",
        prop::collection::btree_set(prop::char::range('一', '十'), 1..4)
            .prop_map(|set| set.into_iter().collect::<Vec<_>>()),
        1..8,
    );
    let words = prop::collection::btree_map(
        "[abc]{2,4}",
        prop::collection::vec(("[一-十]{2,3}", 1u64..1000), 1..3),
        0..5,
    );
    let chars = prop::collection::btree_map(prop::char::range('一', '十'), 0u64..1000, 0..10);
    let ranking = (-3.0..3.0f64, 0.0..3.0f64, 1u16..4).prop_map(|(b, g, beam)| Ranking {
        word_bonus: b,
        segment_penalty: g,
        beam,
        sentences: 3,
    });
    (table, words, chars, ranking).prop_map(|(table, words, chars, ranking)| Data {
        table,
        words,
        chars,
        ranking,
    })
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
        lexicon: Lexicon::new(words, data.chars.clone()),
    };
    Engine::new(Pack { scheme, code_table }, Some(lexicon)).unwrap()
}

/// 每一段可用的字詞與分數，照 plan §2.3（bigram 之前的 unigram）：
/// - 字：ln((字頻 + 1) ÷ (連打字集的字頻總和 + 字數)) − γ；
/// - 詞：ln(詞頻 ÷ 每個詞算一次的詞頻總和) + β − γ。
fn pieces(data: &Data) -> Vec<(String, String, f64)> {
    let r = data.ranking;
    let continuous: BTreeSet<char> = data.table.values().flatten().copied().collect();
    let frequency = |c: &char| data.chars.get(c).copied().unwrap_or(0) as f64;
    let char_total = continuous.iter().map(frequency).sum::<f64>() + continuous.len() as f64;
    let mut seen = HashSet::new();
    let word_total: f64 = data
        .words
        .values()
        .flatten()
        .filter(|(text, _)| seen.insert(text.clone()))
        .map(|(_, f)| *f as f64)
        .sum();
    let mut pieces = Vec::new();
    for (code, chars) in data.table.iter().filter(|(code, _)| code.len() <= 2) {
        for c in chars {
            let score = ((frequency(c) + 1.0) / char_total).ln() - r.segment_penalty;
            pieces.push((code.clone(), c.to_string(), score));
        }
    }
    for (code, list) in &data.words {
        for (text, f) in list {
            let score = (*f as f64 / word_total).ln() + r.word_bonus - r.segment_penalty;
            pieces.push((code.clone(), text.clone(), score));
        }
    }
    pieces
}

/// 窮舉 `input` 的每一種切法：蓋滿整串的（文字, 分數）。
fn every_path(input: &str, pieces: &[(String, String, f64)]) -> Vec<(String, f64)> {
    if input.is_empty() {
        return vec![(String::new(), 0.0)];
    }
    let mut paths = Vec::new();
    for (code, text, score) in pieces {
        if let Some(rest) = input.strip_prefix(code.as_str()) {
            for (tail, tail_score) in every_path(rest, pieces) {
                paths.push((format!("{text}{tail}"), score + tail_score));
            }
        }
    }
    paths
}

proptest! {
    #[test]
    fn top_candidate_is_a_highest_scoring_path_over_every_segmentation(
        data in data(),
        input in "[abc]{1,5}",
    ) {
        let paths = every_path(&input, &pieces(&data));
        let engine = engine(&data);
        let mut user = UserModel::default();
        let mut session = Session::default();
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
            .filter(|(_, s)| (best - s).abs() < 1e-9)
            .map(|(t, _)| t.as_str())
            .collect();
        let top = top.unwrap_or_default();
        prop_assert!(winners.contains(top.as_str()), "首選 {top}，最高分的是 {winners:?}");
    }
}
