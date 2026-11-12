//! 詞級 trigram（M3 票 11b，plan §2.3）：lattice 的靜態機率改用它，前一段不在表裡、或前文沒留下
//! 任何 n-gram 時照現行打分；沒有統計的模型和沒掛上時完全相同。

use std::collections::BTreeMap;

use qim_core::{
    CLAUSE_START, CharModel, CodeEntry, CodeTable, ConversionTable, Engine, Key, LanguageModel,
    LanguageModelPack, Manifest, NgramRow, Options, Pack, PackError, PackKind, Packs, Ranking,
    Scheme, SchemePack, Session, Simplifier, SimplifierPack, Snapshot, UserModel, WordTrigram,
    WordTrigramPack,
};

const START: &str = "\u{2}";

fn scheme_pack(word_trigram: Option<&str>) -> SchemePack {
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 1,
        wildcard: None,
        radicals: BTreeMap::new(),
        packs: Packs {
            language_model: Some("bigram".into()),
            word_trigram: word_trigram.map(Into::into),
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
        ("c".to_string(), vec![entry('後')]),
    ]);
    SchemePack { scheme, code_table }
}

/// `a` 是「甲」「乙」「丁」，`b` 是「天」，`c` 是「後」（简「后」）；字級 bigram 裡「天乙」是常見的
/// 接續，「後」沒有接續。
fn bigram() -> LanguageModelPack {
    let model = |start: f64, continuation: f64| CharModel {
        start: start.ln(),
        continuation: continuation.ln(),
        backoff: 0.0,
    };
    LanguageModelPack {
        id: "bigram".into(),
        model: LanguageModel::new(
            [
                ('甲', model(0.5, 0.1)),
                ('乙', model(0.1, 0.5)),
                ('天', model(0.4, 0.4)),
                ('丁', model(0.05, 0.05)),
                ('後', model(0.1, 0.1)),
                (CLAUSE_START, model(1e-6, 1e-6)),
            ],
            model(1e-6, 1e-6),
            [
                ('天', '乙', 0.9f64.ln()),
                ('天', '甲', 0.01f64.ln()),
                (CLAUSE_START, '丁', 0.9f64.ln()),
            ],
        ),
    }
}

fn engine() -> Engine {
    Engine::new(scheme_pack(None), None, Some(bigram())).unwrap()
}

fn vocabulary(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}

fn with_model(model: WordTrigram) -> Engine {
    engine().with_word_trigram(WordTrigramPack {
        id: "word-trigram".into(),
        model,
    })
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

fn top(engine: &Engine, before: Option<&str>, input: &str) -> String {
    texts(&type_input(engine, before, input).0)[0].clone()
}

#[test]
fn a_model_without_statistics_changes_nothing() {
    let plain = engine();
    let models = [
        WordTrigram::new(vocabulary(&[START]), vec![], vec![]),
        // 有詞但沒有留下任何 n-gram：每個前文都沒見過。
        WordTrigram::new(vocabulary(&[START, "天", "甲", "乙", "丁"]), vec![], vec![]),
    ];
    for model in models {
        let hooked = with_model(model);
        for before in [None, Some("天"), Some("。")] {
            for input in ["a", "ab", "ba", "baa", "abab"] {
                let (a, sa) = type_input(&plain, before, input);
                let (b, sb) = type_input(&hooked, before, input);
                assert_eq!(texts(&a), texts(&b), "{before:?} {input}");
                assert_eq!(sa.paths(20), sb.paths(20), "{before:?} {input}");
            }
        }
    }
}

#[test]
fn a_trigram_sees_the_two_previous_segments() {
    // 「天」「甲」之後是「丁」：只看前一字的 bigram 給不出這種偏好。
    let model = WordTrigram::new(
        vocabulary(&["天", "甲", "丁"]),
        vec![NgramRow {
            context: 0,
            backoff: 0.0,
            next: vec![(1, 0.5f64.ln())],
        }],
        vec![NgramRow {
            context: [0, 1],
            backoff: 0.0,
            next: vec![(2, 0.95f64.ln())],
        }],
    );
    assert_ne!(top(&engine(), None, "baa"), "天甲丁");
    assert_eq!(top(&with_model(model), None, "baa"), "天甲丁");
}

#[test]
fn at_the_start_the_word_bigram_follows_the_segment_before_the_composition() {
    // 句首之後「甲」；前文是「天」時「甲」。字級 bigram 在這兩處都偏好別的字。
    let model = WordTrigram::new(
        vocabulary(&[START, "天", "甲"]),
        vec![
            NgramRow {
                context: 0,
                backoff: 0.0,
                next: vec![(2, 0.95f64.ln())],
            },
            NgramRow {
                context: 1,
                backoff: 0.0,
                next: vec![(2, 0.95f64.ln())],
            },
        ],
        vec![],
    );
    let plain = engine();
    let hooked = with_model(model);
    assert_eq!(top(&plain, Some("。"), "a"), "丁");
    assert_eq!(top(&hooked, Some("。"), "a"), "甲");
    assert_eq!(top(&plain, Some("天"), "a"), "乙");
    assert_eq!(top(&hooked, Some("天"), "a"), "甲");
    // 不知道前文時沒有前一段，照字級 bigram。
    assert_eq!(top(&hooked, None, "a"), top(&plain, None, "a"));
}

/// 簡體模式下殼給的前文是簡體字，詞表是繁體：前一段是轉回繁體的字（M4 票 04），和繁體模式下
/// 對應的繁體前文首選相同。
#[test]
fn in_simplified_mode_the_word_bigram_follows_the_context_turned_back_into_traditional() {
    // 「後」之後「丁」；字級 bigram 退到 unigram 時偏好「乙」，不知道前文時偏好「甲」。
    let model = WordTrigram::new(
        vocabulary(&["後", "丁"]),
        vec![NgramRow {
            context: 0,
            backoff: 0.0,
            next: vec![(1, 0.95f64.ln())],
        }],
        vec![],
    );
    let simplifier = SimplifierPack {
        id: "opencc-hk2s".into(),
        simplifier: Simplifier::new(
            vec![],
            ConversionTable::new([]),
            vec![vec![ConversionTable::new([(
                "後".to_string(),
                "后".to_string(),
            )])]],
        ),
    };
    let engine = with_model(model).with_simplifier(simplifier);
    assert_eq!(top(&engine, Some("後"), "a"), "丁");
    let mut user = UserModel::default();
    let mut session = Session::default();
    session.set_options(Options {
        simplified: true,
        ..Options::default()
    });
    session.set_context("后");
    let shown = session.handle(&engine, &mut user, Key::Letter('a'));
    assert_eq!(shown.candidates[0].text, "丁");
}

#[test]
fn lookups_back_off_to_the_lower_order() {
    // 詞表不照文字排序：編號照給的順序，查文字另有索引。
    let model = WordTrigram::new(
        vocabulary(&["乙", "甲", "天", "丁"]),
        vec![NgramRow {
            context: 2,
            backoff: 0.5f64.ln(),
            next: vec![(1, 0.25f64.ln())],
        }],
        vec![NgramRow {
            context: [3, 2],
            backoff: 0.1f64.ln(),
            next: vec![(0, 0.5f64.ln())],
        }],
    );
    let close = |a: f64, b: f64| (a - b).abs() < 0.05;
    let lower = -7.0;
    // 留下的 bigram；沒留下、前文有列：α 加低一階；前文沒有列：低一階。
    assert!(close(model.log_prob(&["天"], "甲", lower), 0.25f64.ln()));
    assert!(close(
        model.log_prob(&["天"], "乙", lower),
        0.5f64.ln() + lower
    ));
    assert!(close(model.log_prob(&["甲"], "乙", lower), lower));
    // 不在詞表的段：前文有列就乘 α；前一段不在詞表就照低一階。
    assert!(close(
        model.log_prob(&["天"], "日", lower),
        0.5f64.ln() + lower
    ));
    assert!(close(model.log_prob(&["日"], "甲", lower), lower));
    // trigram 留下的直接用；沒留下的退到詞級 bigram 再乘 α。
    assert!(close(
        model.log_prob(&["丁", "天"], "乙", lower),
        0.5f64.ln()
    ));
    assert!(close(
        model.log_prob(&["丁", "天"], "甲", lower),
        0.1f64.ln() + 0.25f64.ln()
    ));
    // 前前一段沒有 trigram 的列：照詞級 bigram。
    assert!(close(
        model.log_prob(&["甲", "天"], "甲", lower),
        0.25f64.ln()
    ));
    assert_eq!(model.id("丁"), Some(3));
    assert_eq!(model.id("日"), None);
    assert_eq!(model.len(), 4);
}

fn pack(id: &str) -> WordTrigramPack {
    WordTrigramPack {
        id: id.into(),
        model: WordTrigram::new(
            vocabulary(&[START, "天", "甲"]),
            vec![NgramRow {
                context: 1,
                backoff: 0.3f64.ln(),
                next: vec![(2, 0.5f64.ln()), (0, 0.01f64.ln())],
            }],
            vec![NgramRow {
                context: [0, 1],
                backoff: 0.0,
                next: vec![(2, 0.9f64.ln())],
            }],
        ),
    }
}

#[test]
fn word_trigram_pack_round_trips_through_bytes() {
    let pack = pack("word-trigram");
    let bytes = pack.to_bytes().unwrap();
    assert_eq!(WordTrigramPack::from_bytes(&bytes).unwrap(), pack);
    assert_eq!(
        LanguageModelPack::from_bytes(&bytes),
        Err(PackError::NotAPack)
    );
}

#[test]
fn a_pack_with_a_repeated_word_is_malformed() {
    let bytes = pack("word-trigram").to_bytes().unwrap();
    // 詞表是第一個欄位：句首、「天」「甲」。把「甲」改成「天」。
    let (tian, jia) = ("天".as_bytes(), "甲".as_bytes());
    let at = bytes
        .windows(jia.len())
        .position(|w| w == jia)
        .expect("詞表裡有「甲」");
    let mut repeated = bytes.clone();
    repeated[at..at + 3].copy_from_slice(tian);
    assert!(matches!(
        WordTrigramPack::from_bytes(&repeated),
        Err(PackError::Malformed(_))
    ));
}

fn file<P: Pack>(pack: &P) -> (Manifest, Vec<u8>) {
    let bytes = pack.to_bytes().unwrap();
    (
        Manifest::new(P::KIND, pack.id(), &bytes, "1.0.0", "CC0-1.0"),
        bytes,
    )
}

/// 方案指名的是預設的那份，可以換成同種的另一份（plan §4.6）；方案要時不能不給、不要時不能給。
#[test]
fn engine_open_takes_a_word_trigram_only_when_the_scheme_names_one() {
    let bigram = file(&bigram());
    let other = file(&pack("other-word-trigram"));
    let open = |scheme: Option<&str>, word_trigram: Option<&(Manifest, Vec<u8>)>| {
        let scheme = file(&scheme_pack(scheme));
        let mut files = vec![(&scheme.0, &scheme.1[..]), (&bigram.0, &bigram.1[..])];
        files.extend(word_trigram.map(|(m, b)| (m, &b[..])));
        Engine::open(&files).map(|_| ())
    };
    assert_eq!(open(Some("word-trigram"), Some(&other)), Ok(()));
    assert_eq!(open(None, None), Ok(()));
    assert_eq!(
        open(Some("word-trigram"), None),
        Err(PackError::WordTrigramMismatch {
            expected: Some("word-trigram".into()),
            found: None
        })
    );
    assert_eq!(
        open(None, Some(&other)),
        Err(PackError::WordTrigramMismatch {
            expected: None,
            found: Some("other-word-trigram".into())
        })
    );
    assert_eq!(PackKind::WordTrigram.as_str(), "word-trigram");
}

#[test]
fn the_opened_engine_scores_with_the_word_trigram() {
    let bigram = file(&bigram());
    let scheme = file(&scheme_pack(Some("word-trigram")));
    let word_trigram = file(&pack("word-trigram"));
    let engine = Engine::open(&[
        (&scheme.0, &scheme.1),
        (&bigram.0, &bigram.1),
        (&word_trigram.0, &word_trigram.1),
    ])
    .unwrap();
    // 「天」之後的詞級 bigram 偏好「甲」，字級 bigram 偏好「乙」。
    assert_eq!(top(&engine, Some("天"), "a"), "甲");
}
