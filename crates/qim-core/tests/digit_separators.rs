//! 數字後的句點、逗號、冒號用半形（brief §4.7，M1 票 26），照 librime `punctuator` 的
//! `digit_separators`：分隔符先待定，接數字或空白就半形上屏，再按一次同一個鍵改回全形。

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, Engine, Key, Options, Pack, Packs, Ranking, Scheme, Session, UserModel,
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
        }],
    )]);
    Pack { scheme, code_table }.to_bytes().unwrap()
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
fn a_period_between_digits_stays_half_width() {
    assert_eq!(Ime::typed("3.5"), "3.5");
}

#[test]
fn a_comma_between_digits_stays_half_width() {
    assert_eq!(Ime::typed("1,000"), "1,000");
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
fn the_separator_waits_in_the_composition_until_the_next_key() {
    let mut ime = Ime::new();
    ime.type_keys("3");
    let snapshot = ime.press(Key::Punct('.'));
    assert!(snapshot.handled);
    assert_eq!(snapshot.commit, None);
    assert_eq!(snapshot.composition.input, ".");
    assert_eq!(snapshot.composition.selected_text(), ".");
    assert!(snapshot.candidates.is_empty());
    assert_eq!(ime.text, "3");
}

#[test]
fn pressing_the_same_separator_again_makes_it_full_width() {
    assert_eq!(Ime::typed("1,,"), "1，");
    assert_eq!(Ime::typed("3.."), "3。");
}

#[test]
fn a_space_commits_the_separator_half_width_with_the_space() {
    assert_eq!(Ime::typed("1. "), "1. ");
}

#[test]
fn a_letter_commits_the_separator_half_width_and_starts_composing() {
    let mut ime = Ime::new();
    ime.type_keys("3.");
    let snapshot = ime.press(Key::Letter('a'));
    assert_eq!(snapshot.commit.as_deref(), Some("."));
    assert_eq!(snapshot.composition.input, "a");
    ime.type_keys(" ");
    assert_eq!(ime.text, "3.日");
}

#[test]
fn other_keys_commit_the_separator_half_width_then_act_as_usual() {
    // 另一個標點照常轉全形；大寫字母和其他沒有全形的符號照打。
    assert_eq!(Ime::typed("3.,"), "3.，");
    assert_eq!(Ime::typed("3.A"), "3.A");
    assert_eq!(Ime::typed("3.%"), "3.%");
    let mut ime = Ime::new();
    ime.type_keys("3.");
    ime.press(Key::Enter);
    assert_eq!(ime.text, "3.");
}

#[test]
fn backspace_or_escape_drops_the_separator() {
    for key in [Key::Backspace, Key::Escape] {
        let mut ime = Ime::new();
        ime.type_keys("3.");
        let snapshot = ime.press(key);
        assert!(snapshot.handled, "{key:?}");
        assert_eq!(snapshot.composition.input, "", "{key:?}");
        assert_eq!(ime.text, "3", "{key:?}");
        // 還是緊接在數字後。
        ime.type_keys(".5");
        assert_eq!(ime.text, "3.5", "{key:?}");
    }
}

#[test]
fn keys_that_only_move_within_the_composition_keep_the_separator_waiting() {
    let mut ime = Ime::new();
    ime.type_keys("3.");
    for key in [
        Key::Left,
        Key::Right,
        Key::PageUp,
        Key::PageDown,
        Key::Other,
        Key::Letter('z'),
    ] {
        let snapshot = ime.press(key);
        assert!(snapshot.handled, "{key:?}");
        assert_eq!(snapshot.composition.input, ".", "{key:?}");
    }
    ime.type_keys("5");
    assert_eq!(ime.text, "3.5");
}

#[test]
fn switching_to_english_commits_the_separator_half_width() {
    let mut ime = Ime::new();
    ime.type_keys("3.");
    let snapshot = ime.press(Key::ToggleAscii);
    assert!(snapshot.ascii);
    assert_eq!(ime.text, "3.");
}

#[test]
fn text_from_the_touch_keyboard_commits_the_separator_half_width_first() {
    let mut ime = Ime::new();
    ime.type_keys("3.");
    let snapshot = ime.session.text(&ime.engine, &mut ime.user, "5");
    assert_eq!(snapshot.commit.as_deref(), Some(".5"));
    assert_eq!(snapshot.composition.input, "");
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
    assert_eq!(Ime::typed("1,000.5"), "1,000.5");
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
fn tapping_the_waiting_separator_or_a_candidate_keeps_it_waiting() {
    let mut ime = Ime::new();
    ime.type_keys("3.");
    let snapshot = ime.session.focus_segment(&ime.engine, &mut ime.user, 0);
    assert!(snapshot.handled);
    assert_eq!(snapshot.composition.input, ".");
    let snapshot = ime.session.select(&ime.engine, &mut ime.user, 0);
    assert!(snapshot.handled);
    assert_eq!(snapshot.commit, None);
    assert_eq!(snapshot.composition.input, ".");
}
