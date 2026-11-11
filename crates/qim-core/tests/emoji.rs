//! emoji 候選（M3 票 16，plan §2.4）：整串的候選是 emoji 資料包裡的詞時，它的 emoji 接在它
//! 後面（rime-emoji 的做法），整列最多 [`EMOJI_LIMIT`] 個；選了原樣上屏、不學。

mod common;

use std::collections::{BTreeMap, BTreeSet};

use qim_core::{
    CodeEntry, CodeTable, ConversionTable, EMOJI_LIMIT, EmojiPack, EmojiTable, Engine, EnglishPack,
    EnglishWords, Key, Lexicon, LexiconPack, Options, Pack, Packs, Ranking, Scheme, SchemePack,
    Session, Simplifier, SimplifierPack, Snapshot, UserModel, Word,
};

/// 碼表的一列：碼、字（依字頻由高到低）。
const TABLE: [(&str, &str); 6] = [
    ("hk", "笑筍"),
    ("bw", "貓"),
    ("ni", "多"),
    ("yi", "謝"),
    ("a", "日"),
    ("h", "竹"),
];

const CHARS: [(char, u64); 7] = [
    ('笑', 100),
    ('筍', 10),
    ('貓', 50),
    ('多', 100),
    ('謝', 50),
    ('日', 100),
    ('竹', 50),
];

/// 沒掛 emoji 資料包的引擎。
fn base_engine() -> Engine {
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
        lexicon: Lexicon::new(
            [(
                "niyi".to_string(),
                vec![Word {
                    text: "多謝".into(),
                    frequency: 1000,
                }],
            )],
            CHARS,
        ),
    };
    let continuous: BTreeSet<char> = TABLE.iter().flat_map(|(_, c)| c.chars()).collect();
    let model = common::unigram("model", &CHARS, continuous.len());
    Engine::new(
        SchemePack { scheme, code_table },
        Some(lexicon),
        Some(model),
    )
    .unwrap()
}

fn emoji() -> EmojiPack {
    EmojiPack {
        id: "emoji".into(),
        emoji: EmojiTable::new([
            ("笑", vec!["😆", "🤣", "😂", "😹"]),
            ("筍", vec!["🎍"]),
            ("貓", vec!["🐈"]),
            ("多謝", vec!["🙏"]),
        ]),
    }
}

fn engine() -> Engine {
    base_engine().with_emoji(emoji())
}

struct Ime {
    engine: Engine,
    session: Session,
    user: UserModel,
}

impl Ime {
    fn new() -> Self {
        Self::with(engine())
    }

    fn with(engine: Engine) -> Self {
        Self {
            engine,
            session: Session::default(),
            user: UserModel::default(),
        }
    }

    fn press(&mut self, key: Key) -> Snapshot {
        self.session.handle(&self.engine, &mut self.user, key)
    }

    fn type_code(&mut self, code: &str) -> Snapshot {
        let mut snapshot = Snapshot::default();
        for c in code.chars() {
            snapshot = self.press(Key::Letter(c));
        }
        snapshot
    }

    /// 打 `code`，回傳第一頁的候選文字。
    fn candidates(&mut self, code: &str) -> Vec<String> {
        let shown = texts(&self.type_code(code));
        self.press(Key::Escape);
        shown
    }
}

fn texts(snapshot: &Snapshot) -> Vec<String> {
    snapshot.candidates.iter().map(|c| c.text.clone()).collect()
}

#[test]
fn the_emoji_of_a_word_follow_it() {
    let mut ime = Ime::new();
    let shown = ime.candidates("bw");
    assert_eq!(shown, ["貓", "🐈"]);
}

#[test]
fn a_list_shows_at_most_the_limit_of_emoji() {
    // 「笑」的四個只接前三個，「筍」的也不接：整列最多 EMOJI_LIMIT 個。只蓋住開頭一段的
    // 「竹」不接 emoji。
    let mut ime = Ime::new();
    let shown = ime.candidates("hk");
    assert_eq!(EMOJI_LIMIT, 3);
    assert_eq!(shown, ["笑", "😆", "🤣", "😂", "筍", "竹"]);
}

#[test]
fn emoji_candidates_are_on_by_default() {
    assert!(Options::default().emoji_candidates);
}

#[test]
fn turning_the_option_off_hides_emoji() {
    let mut ime = Ime::new();
    ime.session.set_options(Options {
        emoji_candidates: false,
        ..Options::default()
    });
    assert_eq!(ime.candidates("hk"), ["笑", "筍", "竹"]);
}

#[test]
fn turning_the_option_off_mid_composition_takes_effect_on_refresh() {
    let mut ime = Ime::new();
    assert_eq!(texts(&ime.type_code("bw")), ["貓", "🐈"]);
    ime.session.set_options(Options {
        emoji_candidates: false,
        ..Options::default()
    });
    assert_eq!(texts(&ime.session.refresh(&ime.engine, &ime.user)), ["貓"]);
}

#[test]
fn an_engine_without_the_emoji_pack_has_no_emoji() {
    let mut ime = Ime::with(base_engine());
    assert_eq!(ime.candidates("bw"), ["貓"]);
}

#[test]
fn space_commits_the_word_not_its_emoji() {
    let mut ime = Ime::new();
    ime.type_code("bw");
    assert_eq!(ime.press(Key::Space).commit.as_deref(), Some("貓"));
}

#[test]
fn choosing_an_emoji_commits_it_and_learns_nothing() {
    let mut ime = Ime::new();
    for _ in 0..3 {
        ime.type_code("hk");
        let snapshot = ime.press(Key::Digit(2));
        assert_eq!(snapshot.commit.as_deref(), Some("😆"));
        assert_eq!(snapshot.commit_original.as_deref(), Some("😆"));
        assert!(snapshot.composition.input.is_empty());
    }
    assert_eq!(ime.user.export(), UserModel::default().export());
    assert_eq!(ime.candidates("hk")[..2], ["笑", "😆"]);
}

#[test]
fn after_choosing_a_prefix_the_emoji_follow_the_rest() {
    let mut ime = Ime::new();
    let shown = texts(&ime.type_code("bwhk"));
    let cat = shown.iter().position(|t| t == "貓").unwrap();
    assert!(
        !shown.contains(&"🐈".to_string()),
        "只蓋住開頭一段的不接：{shown:?}"
    );
    let rest = texts(&ime.press(Key::Digit(u8::try_from(cat + 1).unwrap())));
    assert_eq!(rest[..2], ["笑", "😆"]);
    assert_eq!(ime.press(Key::Digit(2)).commit.as_deref(), Some("貓😆"));
}

#[test]
fn a_focused_segment_lists_no_emoji() {
    let mut ime = Ime::new();
    ime.type_code("bwhk");
    let shown = texts(&ime.press(Key::Left));
    assert!(shown.contains(&"笑".to_string()), "{shown:?}");
    assert!(
        shown.iter().all(|t| !EMOJI.contains(&t.as_str())),
        "{shown:?}"
    );
}

const EMOJI: [&str; 7] = ["😆", "🤣", "😂", "😹", "🎍", "🐈", "🙏"];

#[test]
fn simplified_mode_looks_up_the_traditional_original() {
    let engine = engine().with_simplifier(simplifier());
    let mut ime = Ime::with(engine);
    ime.session.set_options(Options {
        simplified: true,
        ..Options::default()
    });
    let snapshot = ime.type_code("niyi");
    assert_eq!(texts(&snapshot)[..2], ["多谢", "🙏"]);
    assert_eq!(snapshot.candidates[1].original, "🙏");
    assert_eq!(ime.press(Key::Digit(2)).commit.as_deref(), Some("🙏"));
}

fn simplifier() -> SimplifierPack {
    let table = |entries: &[(&str, &str)]| {
        ConversionTable::new(entries.iter().map(|&(k, v)| (k.to_string(), v.to_string())))
    };
    SimplifierPack {
        id: "opencc-hk2s".into(),
        simplifier: Simplifier::new(vec![], table(&[]), vec![vec![table(&[("謝", "谢")])]]),
    }
}

#[test]
fn the_english_candidate_keeps_second_and_the_emoji_follow() {
    let engine = engine().with_english(EnglishPack {
        id: "english".into(),
        words: EnglishWords::new(["niyi"]),
    });
    let mut ime = Ime::with(engine);
    assert_eq!(ime.candidates("niyi")[..3], ["多謝", "niyi", "🙏"]);
}

#[test]
fn the_emoji_pack_round_trips_through_bytes() {
    let pack = emoji();
    let bytes = pack.to_bytes().unwrap();
    assert_eq!(EmojiPack::from_bytes(&bytes).unwrap(), pack);
}

#[test]
fn the_table_merges_a_word_listed_twice_and_drops_repeated_emoji() {
    let table = EmojiTable::new([
        ("貓", vec!["🐈", "🐱"]),
        ("狗", vec![]),
        ("貓", vec!["🐱", "😺"]),
    ]);
    assert_eq!(table.lookup("貓"), ["🐈", "🐱", "😺"]);
    assert!(table.lookup("狗").is_empty());
    assert_eq!(table.len(), 1);
}

#[test]
fn a_word_chosen_twice_goes_first_and_the_emoji_stay_after_their_own_word() {
    // 規則 2 把選兩次的「筍」移到首選；「笑」的 emoji 仍接在「笑」後面，不跟著首選走。
    // 整列的名額已由「笑」用掉，所以沒有「筍」的 🎍。
    let mut ime = Ime::new();
    for _ in 0..2 {
        ime.type_code("hk");
        ime.press(Key::Digit(5));
    }
    assert_eq!(ime.candidates("hk"), ["筍", "笑", "😆", "🤣", "😂", "竹"]);
}

#[test]
fn a_trigger_word_goes_first_and_the_emoji_stay_after_their_own_word() {
    let mut ime = Ime::new();
    ime.user.add_word(&ime.engine, "喵", Some("bw")).unwrap();
    assert_eq!(ime.candidates("bw"), ["喵", "貓", "🐈"]);
}
