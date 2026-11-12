//! 繁體模式的簡體字（M3 票 19、brief §4.1）：碼表標成簡體字的字（HKSCS 收的「门」這類）排在
//! 同碼的繁體字之後、擴展區之前，不過濾，選過照常學；簡體模式不管這個標記。

mod common;

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, ConversionTable, Engine, Key, Lexicon, LexiconPack, Options, Packs,
    Ranking, Scheme, SchemePack, Session, Simplifier, SimplifierPack, Snapshot, UserModel,
};

/// 碼表的一列：碼、字（依字頻由高到低）與它的種類。
const TABLE: [(&str, &[(char, Kind)]); 3] = [
    (
        "ls",
        &[
            ('门', Kind::Simplified),
            ('甲', Kind::Traditional),
            ('申', Kind::Traditional),
            ('𠀀', Kind::Extension),
        ],
    ),
    ("a", &[('日', Kind::Traditional)]),
    ("an", &[('門', Kind::Traditional)]),
];

#[derive(Clone, Copy)]
enum Kind {
    Traditional,
    Simplified,
    Extension,
}

/// 「门」的字頻最高：不管標記時它是 `ls` 的首選。
const CHARS: [(char, u64); 5] = [
    ('门', 500),
    ('甲', 100),
    ('申', 10),
    ('日', 100),
    ('門', 50),
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
            .iter()
            .map(|&(text, kind)| CodeEntry {
                text,
                extension: matches!(kind, Kind::Extension),
                simplified: matches!(kind, Kind::Simplified),
            })
            .collect();
        (code.to_string(), entries)
    }));
    let lexicon = LexiconPack {
        id: "lexicon".into(),
        lexicon: Lexicon::new([], CHARS),
    };
    let model = common::unigram("model", &CHARS, CHARS.len());
    let simplifier = SimplifierPack {
        id: "opencc-hk2s".into(),
        simplifier: Simplifier::new(
            vec![],
            ConversionTable::new([]),
            vec![vec![ConversionTable::new([("門".into(), "门".into())])]],
        ),
    };
    Engine::new(
        SchemePack { scheme, code_table },
        Some(lexicon),
        Some(model),
    )
    .unwrap()
    .with_simplifier(simplifier)
}

struct Ime {
    engine: Engine,
    session: Session,
    user: UserModel,
}

impl Ime {
    fn new(simplified: bool) -> Self {
        Self::with_options(Options {
            simplified,
            ..Options::default()
        })
    }

    fn with_options(options: Options) -> Self {
        let mut session = Session::default();
        session.set_options(options);
        Self {
            engine: engine(),
            session,
            user: UserModel::default(),
        }
    }

    fn press(&mut self, key: Key) -> Snapshot {
        self.session.handle(&self.engine, &mut self.user, key)
    }

    fn type_code(&mut self, code: &str) -> Vec<String> {
        let mut snapshot = Snapshot::default();
        for c in code.chars() {
            snapshot = self.press(Key::Letter(c));
        }
        snapshot.candidates.into_iter().map(|c| c.text).collect()
    }

    /// 打 `code`，按數字鍵選這頁文字是 `text` 的候選。
    fn choose(&mut self, code: &str, text: &str) {
        let shown = self.type_code(code);
        let at = shown.iter().position(|c| c == text);
        let at = at.unwrap_or_else(|| panic!("`{code}` 這頁沒有「{text}」：{shown:?}"));
        let snapshot = self.press(Key::Digit(u8::try_from(at + 1).unwrap()));
        assert_eq!(snapshot.commit.as_deref(), Some(text));
    }
}

#[test]
fn traditional_mode_puts_a_simplified_character_after_the_traditional_ones_of_its_code() {
    assert_eq!(Ime::new(false).type_code("ls"), ["甲", "申", "门", "𠀀"]);
}

#[test]
fn engine_candidates_are_the_traditional_mode_list() {
    let engine = engine();
    assert_eq!(
        engine.candidates(&UserModel::default(), "ls"),
        ["甲", "申", "门", "𠀀"]
    );
}

#[test]
fn simplified_mode_ignores_the_mark() {
    assert_eq!(Ime::new(true).type_code("ls"), ["门", "甲", "申", "𠀀"]);
}

/// 只蓋住開頭一段的字也一樣：同一段的簡體字排在繁體字之後。
#[test]
fn a_simplified_character_covering_the_first_segment_follows_the_traditional_ones() {
    let shown = Ime::new(false).type_code("lsa");
    let singles: Vec<&str> = shown
        .iter()
        .map(String::as_str)
        .filter(|t| ["甲", "申", "门"].contains(t))
        .collect();
    assert_eq!(singles, ["甲", "申", "门"], "{shown:?}");
}

/// 不過濾、照常學：選一次不成首選（brief §4.5），選兩次成首選（排位規則 2）。
#[test]
fn choosing_a_simplified_character_twice_makes_it_the_top() {
    let mut ime = Ime::new(false);
    ime.choose("ls", "门");
    assert_eq!(ime.type_code("ls")[0], "甲");
    ime.press(Key::Escape);
    ime.choose("ls", "门");
    assert_eq!(ime.type_code("ls")[0], "门");
}

/// 開關（spec〈共同規則〉：改變候選位置的新行為都有開關）：預設開，同 macOS 速成。
#[test]
fn the_switch_is_on_by_default() {
    assert!(Options::default().simplified_after_traditional);
}

/// 關掉時回到改前的排序：「门」照分數排第 1。
#[test]
fn switching_off_lists_simplified_characters_by_score() {
    let mut ime = Ime::with_options(Options {
        simplified_after_traditional: false,
        ..Options::default()
    });
    assert_eq!(ime.type_code("ls"), ["门", "甲", "申", "𠀀"]);
}

/// 組字中切換：`refresh` 依新的選項重列候選。
#[test]
fn switching_during_composition_relists_the_candidates() {
    let mut ime = Ime::new(false);
    assert_eq!(ime.type_code("ls"), ["甲", "申", "门", "𠀀"]);
    ime.session.set_options(Options {
        simplified_after_traditional: false,
        ..Options::default()
    });
    let shown = ime.session.refresh(&ime.engine, &ime.user);
    let texts: Vec<String> = shown.candidates.into_iter().map(|c| c.text).collect();
    assert_eq!(texts, ["门", "甲", "申", "𠀀"]);
}
