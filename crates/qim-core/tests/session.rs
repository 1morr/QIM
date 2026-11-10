//! `Session` 經公開介面的行為：資料包位元組 → `Engine` → `Session::handle` → `Snapshot`。

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, Engine, Key, Packs, Ranking, Scheme, SchemePack, Session, ShiftKey,
    Snapshot, UserModel,
};

fn entry(text: char) -> CodeEntry {
    CodeEntry {
        text,
        extension: false,
    }
}

/// 測試用的小資料包：`a` 有 10 個字（比一頁多一個），`b` 有一個字；
/// `aaa` 超過方案的 `max_code_length`，用來確認核心不把超長的碼當字的碼。
fn pack_bytes() -> Vec<u8> {
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        radicals: BTreeMap::from([('a', "日".into()), ('b', "月".into())]),
        packs: Packs::default(),
        ranking: Ranking::default(),
    };
    let a = "日曰昌晶昍旦早旭旨旬".chars().map(entry).collect();
    let code_table = CodeTable::from_entries([
        ("a".to_string(), a),
        ("b".to_string(), vec![entry('月')]),
        ("aaa".to_string(), vec![entry('品')]),
    ]);
    SchemePack { scheme, code_table }.to_bytes().unwrap()
}

struct Ime {
    engine: Engine,
    user: UserModel,
    session: Session,
}

impl Ime {
    fn new() -> Self {
        Self {
            engine: Engine::from_pack_bytes(&pack_bytes(), None, None).unwrap(),
            user: UserModel::default(),
            session: Session::default(),
        }
    }

    fn press(&mut self, key: Key) -> Snapshot {
        self.session.handle(&self.engine, &mut self.user, key)
    }
}

fn texts(snapshot: &Snapshot) -> Vec<String> {
    snapshot.candidates.iter().map(|c| c.text.clone()).collect()
}

#[test]
fn typing_a_composes_a_and_lists_its_first_character_as_top() {
    let mut ime = Ime::new();
    let snapshot = ime.press(Key::Letter('a'));
    assert!(snapshot.handled);
    assert_eq!(snapshot.composition.input, "a");
    assert_eq!(texts(&snapshot)[0], "日");
    assert_eq!(snapshot.commit, None);
}

#[test]
fn candidates_come_nine_per_page_in_code_table_order() {
    let mut ime = Ime::new();
    let snapshot = ime.press(Key::Letter('a'));
    assert_eq!(texts(&snapshot).concat(), "日曰昌晶昍旦早旭旨");
    assert_eq!(snapshot.page, 0);
    assert!(snapshot.has_more);
}

#[test]
fn space_commits_top_and_clears_the_composition() {
    let mut ime = Ime::new();
    ime.press(Key::Letter('a'));
    let snapshot = ime.press(Key::Space);
    assert!(snapshot.handled);
    assert_eq!(snapshot.commit.as_deref(), Some("日"));
    assert_eq!(snapshot.composition.input, "");
    assert!(snapshot.candidates.is_empty());
    assert!(!snapshot.has_more);

    let next = ime.press(Key::Letter('b'));
    assert_eq!(next.composition.input, "b");
    assert_eq!(texts(&next), ["月"]);
}

#[test]
fn space_without_composition_is_left_to_the_host() {
    let mut ime = Ime::new();
    let snapshot = ime.press(Key::Space);
    assert!(!snapshot.handled);
    assert_eq!(snapshot.commit, None);
}

#[test]
fn letters_outside_the_scheme_alphabet_are_left_to_the_host() {
    let mut ime = Ime::new();
    let snapshot = ime.press(Key::Letter('z'));
    assert!(!snapshot.handled);
    assert_eq!(snapshot.composition.input, "");
}

#[test]
fn a_key_with_no_radical_in_the_scheme_shows_as_its_letter() {
    let mut ime = Ime::new();
    ime.press(Key::Letter('a'));
    let snapshot = ime.press(Key::Letter('c'));
    assert_eq!(snapshot.composition.unselected_radicals(), "日c");
}

#[test]
fn space_with_no_candidates_is_swallowed_and_keeps_the_composition() {
    let mut ime = Ime::new();
    ime.press(Key::Letter('c'));
    let snapshot = ime.press(Key::Space);
    assert!(snapshot.handled);
    assert_eq!(snapshot.commit, None);
    assert_eq!(snapshot.composition.input, "c");
}

#[test]
fn escape_clears_the_composition() {
    let mut ime = Ime::new();
    ime.press(Key::Letter('a'));
    let snapshot = ime.press(Key::Escape);
    assert!(snapshot.handled);
    assert_eq!(snapshot.commit, None);
    assert_eq!(snapshot.composition.input, "");
    assert!(snapshot.candidates.is_empty());
}

#[test]
fn backspace_removes_the_last_letter_and_updates_candidates() {
    let mut ime = Ime::new();
    ime.press(Key::Letter('a'));
    assert_eq!(texts(&ime.press(Key::Letter('b')))[0], "日月");
    let snapshot = ime.press(Key::Backspace);
    assert!(snapshot.handled);
    assert_eq!(snapshot.composition.input, "a");
    assert_eq!(texts(&snapshot).concat(), "日曰昌晶昍旦早旭旨");

    let empty = ime.press(Key::Backspace);
    assert!(empty.handled);
    assert_eq!(empty.composition.input, "");
    assert!(empty.candidates.is_empty());
}

#[test]
fn digit_commits_the_candidate_at_that_position_on_the_page() {
    let mut ime = Ime::new();
    ime.press(Key::Letter('a'));
    let snapshot = ime.press(Key::Digit(2));
    assert!(snapshot.handled);
    assert_eq!(snapshot.commit.as_deref(), Some("曰"));
    assert_eq!(snapshot.composition.input, "");
    assert!(snapshot.candidates.is_empty());
}

#[test]
fn digit_without_a_candidate_at_that_position_does_nothing() {
    let mut ime = Ime::new();
    ime.press(Key::Letter('b'));
    for digit in [0, 2, 9] {
        let snapshot = ime.press(Key::Digit(digit));
        assert!(snapshot.handled, "digit {digit}");
        assert_eq!(snapshot.commit, None, "digit {digit}");
        assert_eq!(snapshot.composition.input, "b", "digit {digit}");
        assert_eq!(texts(&snapshot), ["月"], "digit {digit}");
    }
}

#[test]
fn page_down_and_up_move_between_pages_and_stop_at_both_ends() {
    let mut ime = Ime::new();
    ime.press(Key::Letter('a'));

    let first = ime.press(Key::PageUp);
    assert!(first.handled);
    assert_eq!(first.page, 0);
    assert_eq!(texts(&first).concat(), "日曰昌晶昍旦早旭旨");

    let last = ime.press(Key::PageDown);
    assert!(last.handled);
    assert_eq!(last.page, 1);
    assert_eq!(texts(&last), ["旬"]);
    assert!(!last.has_more);

    let still_last = ime.press(Key::PageDown);
    assert!(still_last.handled);
    assert_eq!(still_last.page, 1);
    assert_eq!(texts(&still_last), ["旬"]);

    let back = ime.press(Key::PageUp);
    assert_eq!(back.page, 0);
    assert_eq!(texts(&back).concat(), "日曰昌晶昍旦早旭旨");
    assert!(back.has_more);
}

#[test]
fn digit_selects_from_the_current_page() {
    let mut ime = Ime::new();
    ime.press(Key::Letter('a'));
    ime.press(Key::PageDown);
    assert_eq!(ime.press(Key::Digit(2)).commit, None);
    assert_eq!(ime.press(Key::Digit(1)).commit.as_deref(), Some("旬"));
}

#[test]
fn a_new_letter_or_backspace_goes_back_to_the_first_page() {
    let mut ime = Ime::new();
    ime.press(Key::Letter('a'));
    ime.press(Key::PageDown);
    ime.press(Key::Letter('b'));
    let snapshot = ime.press(Key::Backspace);
    assert_eq!(snapshot.page, 0);
    assert_eq!(texts(&snapshot)[0], "日");
}

#[test]
fn editing_and_paging_keys_without_composition_are_left_to_the_host() {
    let keys = [
        Key::Backspace,
        Key::Escape,
        Key::Digit(0),
        Key::Digit(1),
        Key::PageUp,
        Key::PageDown,
    ];
    for key in keys {
        let mut ime = Ime::new();
        let fresh = ime.press(key);
        assert!(!fresh.handled, "{key:?} before typing");

        ime.press(Key::Letter('a'));
        ime.press(Key::Space);
        let after_commit = ime.press(key);
        assert!(!after_commit.handled, "{key:?} after a commit");
        assert_eq!(after_commit.commit, None, "{key:?} after a commit");
    }
}

#[test]
fn a_code_longer_than_max_code_length_is_not_a_character_code() {
    let mut ime = Ime::new();
    for c in "aa".chars() {
        ime.press(Key::Letter(c));
    }
    let snapshot = ime.press(Key::Letter('a'));
    assert_eq!(snapshot.composition.input, "aaa");
    assert!(
        !ime.engine
            .candidates(&UserModel::default(), "aaa")
            .contains(&"品".to_string())
    );
    assert_eq!(texts(&snapshot)[0], "日日日");
    assert_eq!(ime.press(Key::Space).commit.as_deref(), Some("日日日"));
}

#[test]
fn space_after_paging_still_commits_the_top_candidate_of_the_first_page() {
    let mut ime = Ime::new();
    ime.press(Key::Letter('a'));
    assert_eq!(texts(&ime.press(Key::PageDown)), ["旬"]);
    let snapshot = ime.press(Key::Space);
    assert_eq!(snapshot.commit.as_deref(), Some("日"));
    assert_eq!(snapshot.composition.input, "");
}

#[test]
fn only_a_change_of_candidates_changes_the_generation() {
    let mut ime = Ime::new();
    let typed = ime.press(Key::Letter('a')).generation;
    // 翻頁不是候選變動（M1 票 23，重排的規則見 `tests/rerank.rs`）。
    assert_eq!(ime.press(Key::PageDown).generation, typed);
    assert_ne!(ime.press(Key::Backspace).generation, typed);
}

/// 單按 Shift（`ToggleAscii`）切英文直通（brief §4.3）：英文狀態下鍵都交還宿主，再按一次回中文。
#[test]
fn toggle_ascii_passes_keys_to_the_host_until_toggled_back() {
    let mut ime = Ime::new();
    let toggled = ime.press(Key::ToggleAscii(ShiftKey::Right));
    assert!(toggled.handled);
    assert!(toggled.ascii);
    for key in [
        Key::Letter('a'),
        Key::Punct(','),
        Key::Space,
        Key::Digit(1),
        Key::Enter,
        Key::Other,
    ] {
        let pressed = ime.press(key);
        assert!(!pressed.handled, "{key:?}");
        assert!(pressed.ascii, "{key:?}");
        assert_eq!(pressed.composition.input, "", "{key:?}");
    }
    let back = ime.press(Key::ToggleAscii(ShiftKey::Right));
    assert!(!back.ascii);
    assert_eq!(ime.press(Key::Letter('a')).composition.input, "a");
}

/// 組字中切英文：先上屏整個組字，同按空白（Rime `ascii_composer` 的 `commit_text`；
/// 要上屏輸入碼有 Enter）。
#[test]
fn toggle_ascii_while_composing_commits_the_composition_first() {
    let mut ime = Ime::new();
    ime.press(Key::Letter('a'));
    let toggled = ime.press(Key::ToggleAscii(ShiftKey::Right));
    assert!(toggled.handled);
    assert!(toggled.ascii);
    assert_eq!(toggled.commit.as_deref(), Some("日"));
    assert_eq!(toggled.composition.input, "");
    assert!(toggled.candidates.is_empty());
}

/// 候選展開面板（brief §4.4）一次列全部候選，不分頁；選字用全部候選的位置。
#[test]
fn all_candidates_lists_every_page_and_select_takes_its_position() {
    let mut ime = Ime::new();
    assert!(ime.session.all_candidates().is_empty());
    ime.press(Key::Letter('a'));
    let all: Vec<String> = ime
        .session
        .all_candidates()
        .into_iter()
        .map(|c| c.text)
        .collect();
    assert_eq!(
        all,
        "日曰昌晶昍旦早旭旨旬"
            .chars()
            .map(String::from)
            .collect::<Vec<_>>()
    );
    let chosen = ime.session.select(&ime.engine, &mut ime.user, 9);
    assert_eq!(chosen.commit.as_deref(), Some("旬"));
}
