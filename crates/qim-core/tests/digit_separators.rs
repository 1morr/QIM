//! 數字後的句點、冒號用半形（brief §4.7，M1 票 26）：游標前的字是半形數字時直接上屏半形，
//! 不待定、不出候選；逗號照舊全形（使用者決定，同 librime 預設 `.:` 加 `digit_separator_action:
//! commit`）。

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, Engine, Key, Options, Pack, Packs, Ranking, Scheme, SchemePack, Session,
    ShiftKey, UserModel,
};

fn pack_bytes() -> Vec<u8> {
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        radicals: BTreeMap::from([('a', "日".into())]),
        packs: Packs::default(),
        ranking: Ranking::default(),
    };
    let code_table = CodeTable::from_entries([(
        "a".to_string(),
        vec![CodeEntry {
            text: '日',
            extension: false,
            simplified: false,
        }],
    )]);
    SchemePack { scheme, code_table }.to_bytes().unwrap()
}

/// 核心加一個假宿主：上屏的接到輸入框，核心沒吃下的鍵照宿主的預設動作打出來。
struct Ime {
    engine: Engine,
    user: UserModel,
    session: Session,
    /// 輸入框的內容。
    text: String,
}

impl Ime {
    fn new() -> Self {
        Self {
            engine: Engine::from_pack_bytes(&pack_bytes(), None, None).unwrap(),
            user: UserModel::default(),
            session: Session::default(),
            text: String::new(),
        }
    }

    fn press(&mut self, key: Key) -> qim_core::Snapshot {
        let snapshot = self.session.handle(&self.engine, &mut self.user, key);
        if let Some(commit) = &snapshot.commit {
            self.text.push_str(commit);
        }
        if !snapshot.handled {
            match key {
                Key::Digit(d) => self.text.push(char::from(b'0' + d)),
                Key::Letter(c) | Key::Punct(c) => self.text.push(c),
                Key::Space => self.text.push(' '),
                Key::Backspace => {
                    self.text.pop();
                }
                _ => {}
            }
        }
        snapshot
    }

    /// 照實體鍵盤打一串字元：數字、字母、空白、其餘是標點。
    fn type_keys(&mut self, keys: &str) {
        for c in keys.chars() {
            let key = match c {
                '0'..='9' => Key::Digit(c as u8 - b'0'),
                ' ' => Key::Space,
                c if c.is_ascii_alphabetic() => Key::Letter(c),
                c => Key::Punct(c),
            };
            self.press(key);
        }
    }

    fn typed(keys: &str) -> String {
        let mut ime = Self::new();
        ime.type_keys(keys);
        ime.text
    }
}

#[test]
fn the_separator_commits_at_once_with_nothing_left_composing() {
    let mut ime = Ime::new();
    ime.type_keys("3");
    let snapshot = ime.press(Key::Punct('.'));
    assert!(snapshot.handled);
    assert_eq!(snapshot.commit.as_deref(), Some("."));
    assert_eq!(snapshot.composition.input, "");
    assert!(snapshot.candidates.is_empty());
}

#[test]
fn a_second_separator_is_full_width_because_it_no_longer_follows_a_digit() {
    assert_eq!(Ime::typed("3.."), "3.。");
    assert_eq!(Ime::typed("12::"), "12:：");
}

#[test]
fn a_period_between_digits_stays_half_width() {
    assert_eq!(Ime::typed("3.5"), "3.5");
}

#[test]
fn a_comma_after_a_digit_stays_full_width() {
    // 「共3，」「第1，第2」：中文裡數字後的逗號多半是句子的逗號。
    assert_eq!(Ime::typed("1,"), "1，");
    assert_eq!(Ime::typed("1,2,"), "1，2，");
}

#[test]
fn a_colon_between_digits_stays_half_width() {
    assert_eq!(Ime::typed("12:30"), "12:30");
}

#[test]
fn a_period_not_after_a_digit_is_full_width() {
    assert_eq!(Ime::typed("."), "。");
    assert_eq!(Ime::typed("a."), "日。");
    // 數字鍵在組字中是選字，上屏的是字不是數字。
    assert_eq!(Ime::typed("a1."), "日。");
}

#[test]
fn with_the_switch_off_a_period_after_a_digit_is_full_width() {
    let mut ime = Ime::new();
    ime.session.set_options(Options {
        digit_separators: false,
        ..Options::default()
    });
    ime.type_keys("3.5");
    assert_eq!(ime.text, "3。5");
}

#[test]
fn digits_and_punctuation_in_inline_english_stay_as_typed() {
    // 留碼接英文（左 Shift 的預設，brief §4.3）時字元照原樣接在英文後，不套數字後半形。
    let mut ime = Ime::new();
    ime.type_keys("a");
    ime.press(Key::ToggleAscii(ShiftKey::Left));
    // 空白也接進英文（票 25b），空白後的數字與標點同樣照原樣。
    ime.type_keys("3.5,0 1:2.");
    let snapshot = ime.press(Key::Enter);
    assert_eq!(snapshot.commit.as_deref(), Some("a3.5,0 1:2."));
    assert_eq!(ime.text, "a3.5,0 1:2.");
}

#[test]
fn digits_from_the_touch_keyboard_count_as_digits() {
    let mut ime = Ime::new();
    let snapshot = ime.session.text(&ime.engine, &mut ime.user, "3");
    ime.text.push_str(snapshot.commit.as_deref().unwrap());
    ime.type_keys(".5");
    assert_eq!(ime.text, "3.5");
}

#[test]
fn a_number_can_have_several_separators() {
    assert_eq!(Ime::typed("12:30:15.5"), "12:30:15.5");
    assert_eq!(Ime::typed("1.2.3"), "1.2.3");
}

#[test]
fn keys_passed_to_the_host_end_the_number() {
    // 宿主打了別的字元或刪了字，前一個字元不一定是數字了。
    assert_eq!(Ime::typed("3%."), "3%。");
    let mut ime = Ime::new();
    ime.type_keys("3");
    ime.press(Key::Backspace);
    ime.type_keys(".");
    assert_eq!(ime.text, "。");
}

#[test]
fn tapping_a_candidate_with_nothing_composed_does_not_end_the_number() {
    let mut ime = Ime::new();
    ime.type_keys("3");
    let snapshot = ime.session.select(&ime.engine, &mut ime.user, 0);
    assert!(!snapshot.handled);
    ime.type_keys(".5");
    assert_eq!(ime.text, "3.5");
}

#[test]
fn an_arrow_key_passed_to_the_host_ends_the_number() {
    // 不學 Rime（它不記方向鍵）：游標已離開數字（brief §4.7）。
    let mut ime = Ime::new();
    ime.type_keys("3");
    assert!(!ime.press(Key::Left).handled);
    ime.type_keys(".");
    assert_eq!(ime.text, "3。");
}

#[test]
fn a_digit_typed_in_english_counts_after_switching_back() {
    let mut ime = Ime::new();
    ime.press(Key::ToggleAscii(ShiftKey::Right));
    ime.type_keys("3");
    ime.press(Key::ToggleAscii(ShiftKey::Right));
    ime.type_keys(".");
    assert_eq!(ime.text, "3.");
}

#[test]
fn a_modifier_key_passed_to_the_host_does_not_end_the_number() {
    // 單按 Ctrl、觸控切頁、⇧ 沒有送字元，輸入框沒變（M1 票 26b）。
    let mut ime = Ime::new();
    ime.type_keys("3");
    assert!(!ime.press(Key::Modifier).handled);
    ime.type_keys(".");
    assert_eq!(ime.text, "3.");
}

#[test]
fn a_modifier_key_in_english_does_not_end_the_number() {
    let mut ime = Ime::new();
    ime.press(Key::ToggleAscii(ShiftKey::Right));
    ime.type_keys("3");
    ime.press(Key::Modifier);
    ime.press(Key::ToggleAscii(ShiftKey::Right));
    ime.type_keys(".");
    assert_eq!(ime.text, "3.");
}

#[test]
fn a_modifier_key_while_composing_is_eaten_and_keeps_the_composition() {
    let mut ime = Ime::new();
    ime.type_keys("a");
    let snapshot = ime.press(Key::Modifier);
    assert!(snapshot.handled);
    assert_eq!(snapshot.commit, None);
    assert_eq!(snapshot.composition.input, "a");
    assert_eq!(snapshot.candidates.len(), 1);
}

/// 殼給的前文決定游標前是不是數字（M3 票 05）：游標移到別處、宿主自己改了字，核心都從前文知道。
#[test]
fn the_context_from_the_host_decides_whether_a_period_follows_a_digit() {
    let mut ime = Ime::new();
    ime.session.set_context("共3");
    ime.type_keys(".");
    assert_eq!(ime.text, ".");

    let mut ime = Ime::new();
    ime.type_keys("3");
    // 游標移到輸入框開頭。
    ime.session.set_context("");
    ime.type_keys(".");
    assert_eq!(ime.text, "3。");
}

/// 無痕時殼不給前文（brief §8），核心丟掉之前給的，只看自己送出去的字。
#[test]
fn turning_incognito_on_forgets_a_digit_in_the_context() {
    let mut ime = Ime::new();
    ime.session.set_context("3");
    ime.session.set_options(Options {
        incognito: true,
        ..Options::default()
    });
    ime.type_keys(".3.");
    assert_eq!(ime.text, "。3.");
}

/// 無痕時前文不記，數字後半形照樣看核心自己送出去的字（只記游標前是不是數字，不記文字）。
#[test]
fn in_incognito_digits_typed_still_make_a_period_half_width() {
    let mut ime = Ime::new();
    ime.session.set_options(Options {
        incognito: true,
        ..Options::default()
    });
    let snapshot = ime.session.text(&ime.engine, &mut ime.user, "3");
    ime.text.push_str(snapshot.commit.as_deref().unwrap());
    ime.type_keys(".5:0");
    assert_eq!(ime.text, "3.5:0");
}

/// 換欄位（`clear_context`）後不再是數字後。
#[test]
fn clearing_the_context_ends_the_number() {
    let mut ime = Ime::new();
    ime.type_keys("3");
    ime.session.clear_context(&mut ime.user);
    ime.type_keys(".");
    assert_eq!(ime.text, "3。");
}
