//! 簡體模式（M1 票 16、plan §2.2 的後處理）：排序之後把候選、組字與上屏轉成簡體，
//! 轉換後重複的候選只留第一個；學習仍記繁體原文。

mod common;

use std::collections::{BTreeMap, BTreeSet};

use qim_core::{
    CodeEntry, CodeTable, ConversionTable, Engine, Key, Lexicon, LexiconPack, Options, Pack, Packs,
    Ranking, Scheme, Session, Simplifier, SimplifierPack, Snapshot, UserModel, Word,
};

/// 碼表的一列：碼、字（依字頻由高到低）。
const TABLE: [(&str, &str); 5] = [
    ("a", "日"),
    ("an", "門"),
    ("r", "口"),
    ("ef", "乾幹甲干"),
    ("cd", "們門"),
];

const CHARS: [(char, u64); 9] = [
    ('日', 100),
    ('門', 50),
    ('口', 50),
    ('乾', 100),
    ('幹', 90),
    ('甲', 80),
    ('干', 70),
    ('們', 100),
    ('丁', 1),
];

fn engine() -> Engine {
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
        },
        ranking: Ranking::default(),
    };
    let code_table = CodeTable::from_entries(TABLE.iter().map(|(code, chars)| {
        let entries = chars
            .chars()
            .map(|text| CodeEntry {
                text,
                extension: false,
            })
            .collect();
        (code.to_string(), entries)
    }));
    let words = [(
        "anr".to_string(),
        vec![Word {
            text: "門口".into(),
            frequency: 1000,
        }],
    )];
    let lexicon = LexiconPack {
        id: "lexicon".into(),
        lexicon: Lexicon::new(words, CHARS),
    };
    let continuous: BTreeSet<char> = TABLE.iter().flat_map(|(_, c)| c.chars()).collect();
    let model = common::unigram("model", &CHARS, continuous.len());
    Engine::new(Pack { scheme, code_table }, Some(lexicon), Some(model)).unwrap()
}

fn simplifier() -> SimplifierPack {
    let table = |entries: &[(&str, &str)]| {
        ConversionTable::new(entries.iter().map(|&(k, v)| (k.to_string(), v.to_string())))
    };
    SimplifierPack {
        id: "opencc-hk2s".into(),
        simplifier: Simplifier::new(
            vec![],
            table(&[]),
            vec![vec![table(&[
                ("門", "门"),
                ("們", "们"),
                ("乾", "干"),
                ("幹", "干"),
                ("謝", "谢"),
            ])]],
        ),
    }
}

struct Ime {
    engine: Engine,
    session: Session,
    user: UserModel,
}

impl Ime {
    fn new(engine: Engine, simplified: bool) -> Self {
        let mut session = Session::default();
        session.set_options(Options {
            simplified,
            ..Options::default()
        });
        Self {
            engine,
            session,
            user: UserModel::default(),
        }
    }

    fn simplified() -> Self {
        Self::new(engine().with_simplifier(simplifier()), true)
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

    fn set_simplified(&mut self, simplified: bool) -> Snapshot {
        self.session.set_options(Options {
            simplified,
            ..Options::default()
        });
        self.session.refresh(&self.engine, &self.user)
    }

    /// 打 `code`，按數字鍵選這頁文字是 `text` 的候選。
    fn choose(&mut self, code: &str, text: &str) -> Snapshot {
        let shown = self.type_code(code);
        let at = shown.candidates.iter().position(|c| c.text == text);
        let at = at.unwrap_or_else(|| panic!("`{code}` 這頁沒有「{text}」：{:?}", texts(&shown)));
        self.press(Key::Digit(u8::try_from(at + 1).unwrap()))
    }
}

fn texts(snapshot: &Snapshot) -> Vec<String> {
    snapshot.candidates.iter().map(|c| c.text.clone()).collect()
}

#[test]
fn the_top_for_anr_is_simplified_and_space_commits_it() {
    let mut ime = Ime::simplified();
    let shown = ime.type_code("anr");
    assert_eq!(texts(&shown)[0], "门口");
    assert_eq!(shown.composition.segments[0].text, "门口");
    assert_eq!(shown.composition.selected_text(), "");
    assert_eq!(ime.press(Key::Space).commit.as_deref(), Some("门口"));
}

#[test]
fn traditional_mode_is_unchanged() {
    let mut ime = Ime::new(engine().with_simplifier(simplifier()), false);
    assert_eq!(texts(&ime.type_code("ef")), ["乾", "幹", "甲", "干"]);
}

#[test]
fn candidates_that_become_the_same_keep_only_the_first() {
    let mut ime = Ime::simplified();
    assert_eq!(texts(&ime.type_code("ef")), ["干", "甲"]);
    // 數字鍵選的是轉換、去重後這頁的第 2 個。
    assert_eq!(ime.press(Key::Digit(2)).commit.as_deref(), Some("甲"));
}

#[test]
fn punctuation_commits_the_simplified_composition() {
    let mut ime = Ime::simplified();
    ime.type_code("anr");
    assert_eq!(ime.press(Key::Punct(',')).commit.as_deref(), Some("门口，"));
}

#[test]
fn a_selected_segment_is_shown_and_committed_simplified() {
    let mut ime = Ime::simplified();
    let shown = ime.type_code("anrcd");
    assert!(
        texts(&shown).contains(&"门".to_string()),
        "{:?}",
        texts(&shown)
    );
    let at = shown
        .candidates
        .iter()
        .position(|c| c.text == "门口")
        .unwrap();
    let shown = ime.press(Key::Digit(u8::try_from(at + 1).unwrap()));
    assert_eq!(shown.composition.selected_text(), "门口");
    assert_eq!(texts(&shown), ["们", "门"]);
    assert_eq!(ime.press(Key::Space).commit.as_deref(), Some("门口们"));
}

/// 簡體模式選了「門」兩次，學到的是繁體：切回繁體後「門」成了 `cd` 的首選。
#[test]
fn learning_records_the_traditional_text() {
    let mut ime = Ime::simplified();
    assert_eq!(ime.choose("cd", "门").commit.as_deref(), Some("门"));
    ime.choose("cd", "门");
    ime.set_simplified(false);
    assert_eq!(texts(&ime.type_code("cd")), ["門", "們"]);
}

/// 組字中切換：`refresh` 依新的選項重列候選，組字不變。
#[test]
fn switching_while_composing_relists_the_candidates() {
    let mut ime = Ime::new(engine().with_simplifier(simplifier()), false);
    ime.type_code("ef");
    let shown = ime.set_simplified(true);
    assert!(shown.handled);
    assert_eq!(shown.composition.input, "ef");
    assert_eq!(texts(&shown), ["干", "甲"]);
    assert_eq!(texts(&ime.set_simplified(false)), ["乾", "幹", "甲", "干"]);
}

#[test]
fn refresh_without_a_composition_shows_nothing() {
    let mut ime = Ime::simplified();
    let shown = ime.set_simplified(false);
    assert!(shown.candidates.is_empty());
    assert_eq!(shown.commit, None);
}

/// 沒掛上簡繁資料包時，簡體模式照原樣輸出。
#[test]
fn without_a_simplifier_pack_the_output_stays_traditional() {
    let mut ime = Ime::new(engine(), true);
    assert_eq!(texts(&ime.type_code("ef")), ["乾", "幹", "甲", "干"]);
}

/// 殼只換選項、沒呼叫 `refresh`：下一個鍵照新選項處理，上屏的和組字顯示的一致。
#[test]
fn a_key_after_switching_without_refresh_uses_the_new_option() {
    let mut ime = Ime::simplified();
    ime.type_code("anr");
    ime.session.set_options(Options::default());
    assert_eq!(ime.press(Key::Space).commit.as_deref(), Some("門口"));
}

/// 快照的候選、組字與上屏都另帶繁體原文（字根提示與回報用）。
#[test]
fn snapshots_carry_the_traditional_original() {
    let mut ime = Ime::simplified();
    let shown = ime.type_code("anr");
    assert_eq!(
        (
            shown.candidates[0].text.as_str(),
            shown.candidates[0].original.as_str()
        ),
        ("门口", "門口")
    );
    let segment = &shown.composition.segments[0];
    assert_eq!(
        (segment.text.as_str(), segment.original.as_str()),
        ("门口", "門口")
    );
    let committed = ime.press(Key::Space);
    assert_eq!(committed.commit.as_deref(), Some("门口"));
    assert_eq!(committed.commit_original.as_deref(), Some("門口"));
}

/// 繁體模式的原文就是顯示的字。
#[test]
fn in_traditional_mode_the_original_is_the_text() {
    let mut ime = Ime::new(engine().with_simplifier(simplifier()), false);
    let shown = ime.type_code("anr");
    assert_eq!(shown.candidates[0].original, shown.candidates[0].text);
    let committed = ime.press(Key::Punct(','));
    assert_eq!(committed.commit, committed.commit_original);
}

/// 觸控鍵盤的文字鍵（`Session::text`）：組字和文字鍵本身都照簡體上屏、原文是繁體
/// （使用者 2026-11-04 決定，M1 票 23；票 20 原本讓文字鍵照原樣）。
#[test]
fn a_touch_text_key_commits_the_composition_and_its_text_simplified() {
    let mut ime = Ime::simplified();
    ime.type_code("anr");
    let committed = ime.session.text(&ime.engine, &mut ime.user, "門");
    assert_eq!(committed.commit.as_deref(), Some("门口门"));
    assert_eq!(committed.commit_original.as_deref(), Some("門口門"));

    let alone = ime.session.text(&ime.engine, &mut ime.user, "多謝");
    assert_eq!(alone.commit.as_deref(), Some("多谢"));
    assert_eq!(alone.commit_original.as_deref(), Some("多謝"));
}

#[test]
fn in_traditional_mode_a_touch_text_key_is_committed_as_is() {
    let mut ime = Ime::new(engine().with_simplifier(simplifier()), false);
    let alone = ime.session.text(&ime.engine, &mut ime.user, "多謝");
    assert_eq!(alone.commit.as_deref(), Some("多謝"));
}

/// 殼只換選項、沒呼叫 `refresh` 就點候選：選的是新選項列出的第 2 個（繁體的「幹」），
/// 不是換選項前簡體列表的「甲」。
#[test]
fn selecting_after_switching_without_refresh_uses_the_new_list() {
    let mut ime = Ime::simplified();
    assert_eq!(texts(&ime.type_code("ef")), ["干", "甲"]);
    ime.session.set_options(Options::default());
    let shown = ime.session.select(&ime.engine, &mut ime.user, 1);
    assert_eq!(shown.commit.as_deref(), Some("幹"));
}

/// 殼只換選項就點組字（回到整串，目前分段沒變）：候選照新選項重列，不是換選項前的簡體列表。
#[test]
fn focusing_after_switching_without_refresh_lists_with_the_new_option() {
    let mut ime = Ime::simplified();
    let shown = ime.type_code("ef");
    let whole = shown.composition.segments.len();
    ime.session.set_options(Options::default());
    let shown = ime.session.focus_segment(&ime.engine, &mut ime.user, whole);
    assert_eq!(texts(&shown), ["乾", "幹", "甲", "干"]);
}

/// 殼只換選項就按觸控的文字鍵：先上屏的組字照新選項，接著上屏文字。
#[test]
fn text_after_switching_without_refresh_commits_with_the_new_option() {
    let mut ime = Ime::simplified();
    ime.type_code("anrcd");
    ime.session.set_options(Options::default());
    let shown = ime.session.text(&ime.engine, &mut ime.user, "！");
    assert_eq!(shown.commit.as_deref(), Some("門口們！"));
}
