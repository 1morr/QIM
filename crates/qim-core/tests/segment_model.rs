//! 實驗的語言模型（M3 票 08，`Engine::with_segment_model`）：lattice 的靜態機率改問它，
//! 前面幾段與組字前面的那一段都交給它；回現行分數的模型和沒掛上時完全相同。

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use qim_core::{
    CLAUSE_START, CharModel, CodeEntry, CodeTable, Engine, Key, LanguageModel, LanguageModelPack,
    Packs, Ranking, Scheme, SchemePack, SegmentModel, SegmentQuery, Session, Snapshot, UserModel,
};

/// `a` 是「甲」「乙」「丁」，`b` 是「天」；「天乙」是常見的接續。
fn engine() -> Engine {
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 1,
        radicals: BTreeMap::new(),
        packs: Packs {
            language_model: Some("bigram".into()),
            ..Packs::default()
        },
        ranking: Ranking::default(),
    };
    let entry = |text| CodeEntry {
        text,
        extension: false,
        simplified: false,
    };
    let code_table = CodeTable::from_entries([
        ("a".to_string(), vec![entry('甲'), entry('乙'), entry('丁')]),
        ("b".to_string(), vec![entry('天')]),
    ]);
    let model = |start: f64, continuation: f64| CharModel {
        start: start.ln(),
        continuation: continuation.ln(),
        backoff: 0.0,
    };
    let language_model = LanguageModel::new(
        [
            ('甲', model(0.5, 0.1)),
            ('乙', model(0.1, 0.5)),
            ('天', model(0.4, 0.4)),
            ('丁', model(0.05, 0.05)),
            (CLAUSE_START, model(1e-6, 1e-6)),
        ],
        model(1e-6, 1e-6),
        [
            ('天', '乙', 0.9f64.ln()),
            ('天', '甲', 0.01f64.ln()),
            (CLAUSE_START, '丁', 0.9f64.ln()),
        ],
    );
    Engine::new(
        SchemePack { scheme, code_table },
        None,
        Some(LanguageModelPack {
            id: "bigram".into(),
            model: language_model,
        }),
    )
    .unwrap()
}

/// 回現行分數，另記下每次查詢的前面幾段與字詞。
#[derive(Debug, Default)]
struct Recording {
    seen: Mutex<Vec<(Vec<String>, String)>>,
}

impl SegmentModel for Recording {
    fn log_prob(&self, query: &SegmentQuery<'_>) -> f64 {
        let before = query.before.iter().map(|s| s.to_string()).collect();
        self.seen
            .lock()
            .unwrap()
            .push((before, query.text.to_string()));
        query.current_log_prob
    }
}

/// 「天」「甲」之後的「丁」特別高，其餘照現行分數：只看前一字的 bigram 給不出這種偏好。
#[derive(Debug)]
struct TwoBack;

impl SegmentModel for TwoBack {
    fn log_prob(&self, query: &SegmentQuery<'_>) -> f64 {
        if query.before == ["天", "甲"] && query.text == "丁" {
            10.0
        } else {
            query.current_log_prob
        }
    }
}

fn type_input(engine: &Engine, before: Option<&str>, input: &str) -> (Snapshot, Session) {
    let mut user = UserModel::default();
    let mut session = Session::default();
    if let Some(before) = before {
        session.set_context(before);
    }
    let mut snapshot = Snapshot::default();
    for c in input.chars() {
        snapshot = session.handle(engine, &mut user, Key::Letter(c));
    }
    (snapshot, session)
}

fn texts(snapshot: &Snapshot) -> Vec<String> {
    snapshot.candidates.iter().map(|c| c.text.clone()).collect()
}

#[test]
fn a_model_returning_the_bigram_score_changes_nothing() {
    let plain = engine();
    let hooked = engine().with_segment_model(Arc::new(Recording::default()));
    for before in [None, Some("天"), Some("。")] {
        for input in ["a", "ab", "ba", "baa", "abab"] {
            let (a, sa) = type_input(&plain, before, input);
            let (b, sb) = type_input(&hooked, before, input);
            assert_eq!(texts(&a), texts(&b), "{before:?} {input}");
            assert_eq!(sa.paths(20), sb.paths(20), "{before:?} {input}");
        }
    }
}

#[test]
fn the_model_sees_the_two_previous_segments() {
    let plain = engine();
    let hooked = engine().with_segment_model(Arc::new(TwoBack));
    assert_ne!(texts(&type_input(&plain, None, "baa").0)[0], "天甲丁");
    assert_eq!(texts(&type_input(&hooked, None, "baa").0)[0], "天甲丁");
}

#[test]
fn at_the_start_the_model_sees_the_segment_before_the_composition() {
    let cases = [
        (None, vec![]),
        (Some("天"), vec!["天".to_string()]),
        (Some("。"), vec![CLAUSE_START.to_string()]),
    ];
    for (before, expected) in cases {
        let model = Arc::new(Recording::default());
        let hooked = engine().with_segment_model(model.clone());
        type_input(&hooked, before, "a");
        let seen = model.seen.lock().unwrap();
        assert!(!seen.is_empty());
        assert!(
            seen.iter().all(|(b, _)| *b == expected),
            "{before:?}: {seen:?}"
        );
    }
}
