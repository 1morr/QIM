//! 句中英文混輸的實驗（M4 票 28，`Engine::with_inline_english`）：英文詞典裡拼對的常見詞帶懲罰進 lattice，
//! 一串輸入碼裡夾英文詞也接得成一條路徑；預設關。

mod common;

use std::collections::{BTreeMap, BTreeSet};

use qim_core::{
    CodeEntry, CodeTable, Engine, EnglishPack, EnglishWords, InlineEnglish, Key, Lexicon,
    LexiconPack, Packs, Ranking, Scheme, SchemePack, Session, Snapshot, UserModel,
};

/// 碼表的一列：碼、字。`the`、`yes` 拆成單字都打得出，但沒有一條中文路徑像英文那麼短。
const TABLE: [(&str, &str); 7] = [
    ("a", "日"),
    ("an", "門"),
    ("t", "廿"),
    ("h", "竹"),
    ("e", "水"),
    ("y", "卜"),
    ("s", "尸"),
];

const CHARS: [(char, u64); 7] = [
    ('日', 100),
    ('門', 50),
    ('廿', 10),
    ('竹', 50),
    ('水', 100),
    ('卜', 10),
    ('尸', 10),
];

fn engine() -> Engine {
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        wildcard: None,
        radicals: BTreeMap::new(),
        packs: Packs {
            lexicon: Some("lexicon".into()),
            language_model: Some("model".into()),
            radical_hint: None,
            word_trigram: None,
        },
        ranking: Ranking::default(),
    };
    let code_table = CodeTable::from_entries(TABLE.iter().map(|(code, chars)| {
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
    let lexicon = LexiconPack {
        id: "lexicon".into(),
        lexicon: Lexicon::new(Vec::new(), CHARS),
    };
    let continuous: BTreeSet<char> = TABLE.iter().flat_map(|(_, c)| c.chars()).collect();
    let model = common::unigram("model", &CHARS, continuous.len());
    Engine::new(
        SchemePack { scheme, code_table },
        Some(lexicon),
        Some(model),
    )
    .unwrap()
    .with_english(EnglishPack {
        id: "english".into(),
        words: EnglishWords::new([("the", 222), ("yes", 40)]),
    })
}

/// 懲罰小到英文一定勝過拆成單字的路徑。
fn inline(min_len: usize, min_frequency: u8) -> InlineEnglish {
    InlineEnglish {
        penalty: 1.0,
        min_len,
        min_frequency,
    }
}

fn first_candidate(engine: &Engine, code: &str) -> String {
    let (mut session, mut user) = (Session::default(), UserModel::default());
    let mut snapshot = Snapshot::default();
    for c in code.chars() {
        snapshot = session.handle(engine, &mut user, Key::Letter(c));
    }
    snapshot.candidates[0].text.clone()
}

#[test]
fn off_by_default_the_english_word_in_the_middle_is_not_read() {
    assert_eq!(first_candidate(&engine(), "anthe"), "門廿竹水");
}

#[test]
fn a_common_english_word_joins_the_chinese_path() {
    let engine = engine().with_inline_english(inline(3, 100));
    assert_eq!(first_candidate(&engine, "anthe"), "門the");
}

#[test]
fn a_word_below_the_frequency_floor_is_not_read() {
    let engine = engine().with_inline_english(inline(3, 100));
    assert_eq!(first_candidate(&engine, "anyes"), "門卜水尸");
}

#[test]
fn a_word_shorter_than_the_minimum_length_is_not_read() {
    let engine = engine().with_inline_english(inline(4, 100));
    assert_eq!(first_candidate(&engine, "anthe"), "門廿竹水");
}
