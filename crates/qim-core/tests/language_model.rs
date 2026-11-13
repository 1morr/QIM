//! 字級 bigram 資料包經公開介面的行為（plan〈排序：同一把尺〉〈字級 bigram 與詞級 trigram〉）：三份資料包位元組 → `Engine` →
//! `Session::handle` → `Snapshot`。

use std::collections::BTreeMap;

use qim_core::{
    CharModel, CodeEntry, CodeTable, Engine, Key, LanguageModel, LanguageModelPack, Lexicon,
    LexiconPack, Pack, PackError, Packs, Ranking, Scheme, SchemePack, Session, Snapshot, UserModel,
};

fn scheme(language_model: Option<&str>) -> Scheme {
    Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        wildcard: None,
        radicals: BTreeMap::new(),
        packs: Packs {
            lexicon: Some("lexicon".into()),
            language_model: language_model.map(Into::into),
            ..Packs::default()
        },
        ranking: Ranking::default(),
    }
}

/// `a` 有甲、乙兩字，甲的字頻高；`b` 是丙、`c` 是丁。
fn pack(language_model: Option<&str>) -> SchemePack {
    let entries = |chars: &str| {
        chars
            .chars()
            .map(|text| CodeEntry {
                text,
                extension: false,
                simplified: false,
            })
            .collect()
    };
    SchemePack {
        scheme: scheme(language_model),
        code_table: CodeTable::from_entries([
            ("a".to_string(), entries("甲乙")),
            ("b".to_string(), entries("丙")),
            ("c".to_string(), entries("丁")),
        ]),
    }
}

fn lexicon() -> LexiconPack {
    LexiconPack {
        id: "lexicon".into(),
        lexicon: Lexicon::new([], [('甲', 100), ('乙', 10), ('丙', 50), ('丁', 50)]),
    }
}

/// 沒有前一字時甲比乙常見；但乙後面常接丙、甲後面常接丁。
fn bigram(id: &str) -> LanguageModelPack {
    let char_model = |start: f64| CharModel {
        start: start.ln(),
        continuation: 0.25f64.ln(),
        backoff: 0.1f64.ln(),
    };
    LanguageModelPack {
        id: id.into(),
        model: LanguageModel::new(
            [
                ('甲', char_model(0.4)),
                ('乙', char_model(0.1)),
                ('丙', char_model(0.25)),
                ('丁', char_model(0.25)),
            ],
            CharModel {
                start: 0.01f64.ln(),
                continuation: 0.01f64.ln(),
                backoff: 0.0,
            },
            [
                ('乙', '丙', 0.9f64.ln()),
                ('甲', '丁', 0.9f64.ln()),
                ('甲', '丙', 0.01f64.ln()),
            ],
        ),
    }
}

fn engine() -> Engine {
    Engine::from_pack_bytes(
        &pack(Some("bigram")).to_bytes().unwrap(),
        Some(&lexicon().to_bytes().unwrap()),
        Some(&bigram("bigram").to_bytes().unwrap()),
    )
    .unwrap()
}

fn type_input(engine: &Engine, input: &str) -> Snapshot {
    let mut user = UserModel::default();
    let mut session = Session::default();
    let mut snapshot = Snapshot::default();
    for c in input.chars() {
        snapshot = session.handle(engine, &mut user, Key::Letter(c));
    }
    snapshot
}

fn top(snapshot: &Snapshot) -> &str {
    snapshot.candidates.first().map_or("", |c| c.text.as_str())
}

#[test]
fn the_next_character_decides_which_character_of_the_same_code_comes_first() {
    let engine = engine();
    assert_eq!(top(&type_input(&engine, "a")), "甲");
    assert_eq!(top(&type_input(&engine, "ab")), "乙丙");
    assert_eq!(top(&type_input(&engine, "ac")), "甲丁");
}

#[test]
fn language_model_pack_round_trips_through_bytes() {
    let pack = bigram("bigram");
    assert_eq!(
        LanguageModelPack::from_bytes(&pack.to_bytes().unwrap()).unwrap(),
        pack
    );
}

#[test]
fn other_kinds_of_pack_are_not_language_model_packs() {
    let lexicon = lexicon().to_bytes().unwrap();
    assert_eq!(
        LanguageModelPack::from_bytes(&lexicon),
        Err(PackError::NotAPack)
    );
}

/// 方案指名的是預設的語言模型，可以換成另一份（plan〈資料包 manifest〉）；方案要時不能不給、不要時不能給。
#[test]
fn engine_takes_any_language_model_but_only_when_the_scheme_wants_one() {
    let lexicon = lexicon().to_bytes().unwrap();
    let named = pack(Some("bigram")).to_bytes().unwrap();
    let other = bigram("other").to_bytes().unwrap();
    assert!(Engine::from_pack_bytes(&named, Some(&lexicon), Some(&other)).is_ok());
    assert_eq!(
        Engine::from_pack_bytes(&named, Some(&lexicon), None).err(),
        Some(PackError::LanguageModelMismatch {
            expected: Some("bigram".into()),
            found: None,
        })
    );
    let unnamed = pack(None).to_bytes().unwrap();
    let given = bigram("bigram").to_bytes().unwrap();
    assert_eq!(
        Engine::from_pack_bytes(&unnamed, Some(&lexicon), Some(&given)).err(),
        Some(PackError::LanguageModelMismatch {
            expected: None,
            found: Some("bigram".into()),
        })
    );
}

#[test]
fn a_pair_that_was_not_kept_backs_off_to_the_continuation_probability() {
    let model = bigram("bigram").model;
    let close = |a: f64, b: f64, tolerance: f64| (a - b).abs() <= tolerance;
    // 乙丁沒留下：α(乙) × P_cont(丁)。
    let expected = 0.1f64.ln() + 0.25f64.ln();
    assert!(close(model.log_prob(Some('乙'), '丁'), expected, 1e-6));
    // 前一字沒有統計：用未知字的 α（0）與丁的 P_cont。
    assert!(close(model.log_prob(Some('戊'), '丁'), 0.25f64.ln(), 1e-6));
    // 沒有前一字：unigram；字也沒有統計時用未知字的。
    assert!(close(model.log_prob(None, '乙'), 0.1f64.ln(), 1e-6));
    assert!(close(model.log_prob(None, '戊'), 0.01f64.ln(), 1e-6));
    // 留下的 bigram 量化過，誤差不超過半級。
    let half_step = qim_core::LOG_PROB_STEP / 2.0;
    assert!(close(
        model.log_prob(Some('乙'), '丙'),
        0.9f64.ln(),
        half_step
    ));
    assert!(close(
        model.log_prob(Some('甲'), '丙'),
        0.01f64.ln(),
        half_step
    ));
}

#[test]
fn without_a_language_model_every_character_scores_the_same() {
    let model = LanguageModel::default();
    assert_eq!(model.log_prob(None, '甲'), 0.0);
    assert_eq!(model.log_prob(Some('甲'), '乙'), 0.0);
    assert_eq!(
        LanguageModelPack::from_bytes(
            &LanguageModelPack {
                id: "empty".into(),
                model: model.clone(),
            }
            .to_bytes()
            .unwrap()
        )
        .unwrap()
        .model,
        model
    );
}
