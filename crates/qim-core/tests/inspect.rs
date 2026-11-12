//! 網頁 debug 面板的查詢（plan §3.1，M4 票 01）：`Session::inspect` 說明核心為什麼這樣排——殼給的
//! 前文、組字接在什麼後面、各候選的來源、路徑分與學習加分、排位規則與重排。只給網頁用，不進
//! `Snapshot`。

mod common;

use std::collections::{BTreeMap, BTreeSet};

use qim_core::{
    CandidateKind, CodeEntry, CodeTable, ContextState, EmojiPack, EmojiTable, Engine, EnglishPack,
    EnglishWords, InspectedCandidate, Inspection, Key, Lexicon, LexiconPack, Options, Packs,
    PrecedingChar, Ranking, Reranked, Scheme, SchemePack, SegmentSource, Session, Snapshot,
    UserModel, Word,
};

/// `ab` 是「甲」「乙」「丙」，甲的字頻遠高；`c`「丁」、`d`「天」，`cd` 是詞「丁天」（接 🙂）；
/// 英文只有 `cdc`。
fn engine() -> Engine {
    let table = [("ab", "甲乙丙"), ("c", "丁"), ("d", "天")];
    let chars = [('甲', 1000), ('乙', 10), ('丙', 9), ('丁', 50), ('天', 50)];
    let code_table = CodeTable::from_entries(table.iter().map(|(code, chars)| {
        let entries = chars
            .chars()
            .map(|text| CodeEntry {
                text,
                extension: false,
                simplified: false,
            })
            .collect();
        (code.to_string(), entries)
    }));
    let words = BTreeMap::from([(
        "cd".to_string(),
        vec![Word {
            text: "丁天".into(),
            frequency: 100,
        }],
    )]);
    let lexicon = LexiconPack {
        id: "lexicon".into(),
        lexicon: Lexicon::new(words, chars),
    };
    let continuous: BTreeSet<char> = table.iter().flat_map(|(_, c)| c.chars()).collect();
    let model = common::unigram("model", &chars, continuous.len());
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        radicals: BTreeMap::new(),
        packs: Packs {
            lexicon: Some("lexicon".into()),
            language_model: Some("model".into()),
            ..Packs::default()
        },
        ranking: Ranking::default(),
    };
    Engine::new(
        SchemePack { scheme, code_table },
        Some(lexicon),
        Some(model),
    )
    .unwrap()
    .with_english(EnglishPack {
        id: "english".into(),
        words: EnglishWords::new(["cdc"]),
    })
    .with_emoji(EmojiPack {
        id: "emoji".into(),
        emoji: EmojiTable::new([("丁天", vec!["🙂"])]),
    })
}

struct Ime {
    engine: Engine,
    user: UserModel,
    session: Session,
}

impl Ime {
    fn new() -> Self {
        Self {
            engine: engine(),
            user: UserModel::default(),
            session: Session::default(),
        }
    }

    fn type_code(&mut self, code: &str) -> Snapshot {
        let mut snapshot = Snapshot::default();
        for c in code.chars() {
            snapshot = self
                .session
                .handle(&self.engine, &mut self.user, Key::Letter(c));
        }
        snapshot
    }

    fn press(&mut self, key: Key) -> Snapshot {
        self.session.handle(&self.engine, &mut self.user, key)
    }

    /// 選這頁文字是 `text` 的候選。
    fn pick(&mut self, shown: &Snapshot, text: &str) -> Snapshot {
        let at = shown.candidates.iter().position(|c| c.text == text);
        let at = at.unwrap_or_else(|| panic!("這頁沒有「{text}」"));
        self.press(Key::Digit(u8::try_from(at + 1).unwrap()))
    }

    fn inspect(&self) -> Inspection {
        self.session.inspect(&self.engine, &self.user)
    }
}

#[test]
fn the_context_is_known_unknown_or_not_read_in_incognito() {
    let mut ime = Ime::new();
    assert_eq!(ime.inspect().context, ContextState::Unknown);

    ime.session.set_context("你好，");
    assert_eq!(ime.inspect().context, ContextState::Known("你好，".into()));

    ime.session.set_options(Options {
        incognito: true,
        ..Options::default()
    });
    ime.session.set_context("你好，");
    assert_eq!(ime.inspect().context, ContextState::Incognito);
}

/// 一個候選的文字、種類與各段的（碼, 來源）。
type Sourced = (String, CandidateKind, Vec<(String, SegmentSource)>);

/// 各候選的 [`Sourced`]，依候選列的順序。
fn sources(inspection: &Inspection) -> Vec<Sourced> {
    inspection
        .candidates
        .iter()
        .map(|c| {
            let segments = c
                .segments
                .iter()
                .map(|s| (s.code.clone(), s.source))
                .collect();
            (c.text.clone(), c.kind, segments)
        })
        .collect()
}

#[test]
fn each_candidate_says_where_it_comes_from() {
    use CandidateKind::{Emoji, English, Word};
    use SegmentSource::{CodeTable, Lexicon};
    let mut ime = Ime::new();
    ime.type_code("cd");
    assert_eq!(
        sources(&ime.inspect()),
        [
            ("丁天".into(), Word, vec![("cd".into(), Lexicon)]),
            ("🙂".into(), Emoji, vec![]),
            ("丁".into(), Word, vec![("c".into(), CodeTable)]),
        ]
    );

    ime.type_code("c");
    let listed = sources(&ime.inspect());
    assert!(listed.contains(&(
        "丁天丁".into(),
        Word,
        vec![("cd".into(), Lexicon), ("c".into(), CodeTable)]
    )));
    assert!(listed.contains(&("cdc".into(), English, vec![])));
}

#[test]
fn a_user_word_is_its_own_source() {
    let mut ime = Ime::new();
    ime.user.add_word(&ime.engine, "乙丙", None).unwrap();
    ime.type_code("abab");
    let inspection = ime.inspect();
    let word = inspection
        .candidates
        .iter()
        .find(|c| c.text == "乙丙")
        .expect("使用者詞在候選裡");
    assert_eq!(word.segments.len(), 1);
    assert_eq!(word.segments[0].source, SegmentSource::UserWord);
}

#[test]
fn a_candidate_scores_like_its_lattice_path_and_has_no_learning_bonus_when_nothing_is_learned() {
    let mut ime = Ime::new();
    ime.type_code("ababcd");
    let paths = ime.session.paths(20);
    let inspection = ime.inspect();
    let mut compared = 0;
    for path in &paths {
        let Some(candidate) = inspection.candidates.iter().find(|c| c.text == path.text) else {
            continue;
        };
        compared += 1;
        let score = candidate.score.expect("字詞有路徑分");
        assert!(
            (score - path.score).abs() < 1e-9,
            "{}：{score} 對 {}",
            path.text,
            path.score
        );
        assert_eq!(candidate.learning, Some(0.0));
    }
    assert!(compared >= 2, "候選列裡至少有兩條路徑");

    ime.press(Key::Escape);
    ime.type_code("cdc");
    let english = ime
        .inspect()
        .candidates
        .into_iter()
        .find(|c| c.text == "cdc")
        .unwrap();
    assert_eq!((english.score, english.learning), (None, None));
}

/// 給前文 `context`、打 `code` 時文字是 `text` 的候選。
fn candidate_after(ime: &mut Ime, context: &str, code: &str, text: &str) -> InspectedCandidate {
    ime.session.set_context(context);
    ime.type_code(code);
    let inspection = ime.inspect();
    ime.press(Key::Escape);
    inspection
        .candidates
        .into_iter()
        .find(|c| c.text == text)
        .unwrap_or_else(|| panic!("沒有「{text}」"))
}

#[test]
fn a_learned_segment_shows_its_learning_bonus_and_personal_bigram() {
    let mut blank = Ime::new();
    let unlearned = candidate_after(&mut blank, "丁", "ab", "丙");

    let mut ime = Ime::new();
    ime.session.set_context("丁");
    let shown = ime.type_code("ab");
    ime.pick(&shown, "丙");
    let learned = candidate_after(&mut ime, "丁", "ab", "丙");

    let segment = &learned.segments[0];
    assert!(segment.learned && segment.personal_bigram, "{segment:?}");
    let bonus = learned.learning.unwrap();
    assert!(bonus > 0.0);
    assert!((learned.score.unwrap() - bonus - unlearned.score.unwrap()).abs() < 1e-9);

    // 換了前一個字詞，個人 bigram 就接不上，學習計數照樣有。
    let elsewhere = candidate_after(&mut ime, "天", "ab", "丙");
    assert!(elsewhere.segments[0].learned && !elsewhere.segments[0].personal_bigram);
}

/// 學習的時限（M4 票 13）：一年沒用而停用的條目不顯示學習加分，和排序一致；日期改回來又顯示。
#[test]
fn an_entry_unused_for_a_year_shows_no_learning_bonus() {
    let mut blank = Ime::new();
    let unlearned = candidate_after(&mut blank, "丁", "ab", "丙");

    let mut ime = Ime::new();
    ime.user.set_today("2026-01-01".parse().unwrap());
    ime.session.set_context("丁");
    let shown = ime.type_code("ab");
    ime.pick(&shown, "丙");

    ime.user.set_today("2027-01-02".parse().unwrap());
    let expired = candidate_after(&mut ime, "丁", "ab", "丙");
    let segment = &expired.segments[0];
    assert!(!segment.learned && !segment.personal_bigram, "{segment:?}");
    assert_eq!(expired.learning, Some(0.0));
    assert_eq!(expired.score, unlearned.score);

    ime.user.set_today("2026-06-01".parse().unwrap());
    let back = candidate_after(&mut ime, "丁", "ab", "丙");
    assert!(back.learning.unwrap() > 0.0);
    assert!(back.segments[0].learned && back.segments[0].personal_bigram);
}

/// 停用的個人 bigram 也不算進條件機率的分母：「丁」後面的「丙」停用了，「乙」的加分就和「丁」後面
/// 從沒接過「丙」時一樣。
#[test]
fn an_expired_personal_bigram_leaves_the_others_as_if_it_never_was() {
    let learn = |first_context: &str| {
        let mut ime = Ime::new();
        ime.user.set_today("2026-01-01".parse().unwrap());
        ime.session.set_context(first_context);
        let shown = ime.type_code("ab");
        ime.pick(&shown, "丙");
        ime.user.set_today("2026-12-01".parse().unwrap());
        ime.session.set_context("丁");
        let shown = ime.type_code("ab");
        ime.pick(&shown, "乙");
        ime.user.set_today("2027-01-02".parse().unwrap());
        let second = candidate_after(&mut ime, "丁", "ab", "乙");
        let first = candidate_after(&mut ime, "丁", "ab", "丙");
        (second, first)
    };
    let (with_expired, expired) = learn("丁");
    let (without, _) = learn("天");
    assert!(with_expired.segments[0].personal_bigram);
    assert_eq!(with_expired.learning, without.learning);
    assert!(!expired.segments[0].personal_bigram, "{expired:?}");
}

#[test]
fn the_composition_start_says_what_it_follows() {
    let mut ime = Ime::new();
    ime.session.set_context("丁");
    ime.type_code("ab");
    let inspection = ime.inspect();
    assert_eq!(inspection.preceding_char, PrecedingChar::Char('丁'));
    assert_eq!(inspection.preceding_word.as_deref(), Some("丁"));

    // 不知道前文時用 unigram，也沒有前一個字詞。
    let mut fresh = Ime::new();
    fresh.type_code("ab");
    let inspection = fresh.inspect();
    assert_eq!(inspection.preceding_char, PrecedingChar::Unigram);
    assert_eq!(inspection.preceding_word, None);
}

#[test]
fn a_candidate_placed_by_a_ranking_rule_names_the_rule() {
    let mut ime = Ime::new();
    for _ in 0..2 {
        let shown = ime.type_code("ab");
        ime.pick(&shown, "丙");
    }
    ime.type_code("ab");
    let inspection = ime.inspect();
    assert_eq!(inspection.candidates[0].text, "丙");
    assert_eq!(inspection.candidates[0].rule, Some(2));
    assert_eq!(inspection.candidates[1].rule, None);

    ime.press(Key::Escape);
    ime.type_code("cdc");
    let english = ime
        .inspect()
        .candidates
        .into_iter()
        .find(|c| c.text == "cdc")
        .unwrap();
    assert_eq!(english.rule, Some(4));
}

#[test]
fn a_path_inserted_by_the_rerank_says_so() {
    let mut ime = Ime::new();
    ime.type_code("ababab");
    let listed: Vec<String> = ime
        .session
        .all_candidates()
        .into_iter()
        .map(|c| c.text)
        .collect();
    let request = ime.session.rerank_request(&ime.engine).unwrap();
    let (k, path) = request
        .paths
        .iter()
        .enumerate()
        .find(|(_, p)| !listed.contains(&p.text))
        .expect("有一條路徑不在候選列");
    let path = path.text.clone();
    ime.session
        .apply_rerank(&ime.engine, request.generation, &[k])
        .unwrap();
    let inspection = ime.inspect();
    assert_eq!(inspection.candidates[1].text, path);
    assert_eq!(
        inspection.candidates[1].rerank,
        Some(Reranked {
            path: k,
            inserted: true
        })
    );
    assert!(
        inspection
            .candidates
            .iter()
            .skip(2)
            .all(|c| c.rerank.is_none())
    );
}

#[test]
fn association_candidates_are_marked_and_have_no_path_score() {
    let mut ime = Ime::new();
    ime.type_code("c");
    let shown = ime.press(Key::Space);
    assert_eq!(shown.commit.as_deref(), Some("丁"));
    let inspection = ime.inspect();
    assert!(!inspection.candidates.is_empty());
    assert!(inspection.candidates.iter().all(|c| {
        c.kind == CandidateKind::Association && c.score.is_none() && c.segments.is_empty()
    }));
}

/// debug 面板的資料只由 `inspect` 給、不進 `Snapshot`（票 01）：平台殼每鍵都收 `Snapshot`，多帶的欄位
/// 每鍵都要算。解構不加 `..`，`Snapshot` 加了欄位這裡就編譯不過；真的要加對外的欄位時（如 M4 的
/// `delete_before`），確認不是 debug 用的再把它加進這個清單。
#[test]
fn the_snapshot_carries_no_debug_fields() {
    let Snapshot {
        handled: _,
        composition: _,
        candidates: _,
        page: _,
        has_more: _,
        commit: _,
        commit_original: _,
        ascii: _,
        generation: _,
    } = Snapshot::default();
}

#[test]
fn the_input_code_is_shown_cut_into_segments() {
    let mut ime = Ime::new();
    ime.type_code("cdc");
    let cut: Vec<(String, String, bool)> = ime
        .inspect()
        .segments
        .into_iter()
        .map(|s| (s.code, s.text, s.selected))
        .collect();
    assert_eq!(
        cut,
        [
            ("cd".into(), "丁天".into(), false),
            ("c".into(), "丁".into(), false)
        ]
    );
}
