//! `Session` 經公開介面的行為：資料包位元組 → `Engine` → `Session::handle` → `Snapshot`。

use std::collections::BTreeMap;

use qim_core::{CodeEntry, CodeTable, Engine, Key, Pack, Scheme, Session, Snapshot, UserModel};

fn entry(text: char) -> CodeEntry {
    CodeEntry {
        text,
        extension: false,
    }
}

/// 測試用的小資料包：`a` 有 10 個字（比一頁多一個），`b` 有一個字。
fn pack_bytes() -> Vec<u8> {
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        radicals: BTreeMap::from([('a', "日".into()), ('b', "月".into())]),
    };
    let a = "日曰昌晶昍旦早旭旨旬".chars().map(entry).collect();
    let code_table =
        CodeTable::from_entries([("a".to_string(), a), ("b".to_string(), vec![entry('月')])]);
    Pack { scheme, code_table }.to_bytes().unwrap()
}

struct Ime {
    engine: Engine,
    user: UserModel,
    session: Session,
}

impl Ime {
    fn new() -> Self {
        Self {
            engine: Engine::from_pack_bytes(&pack_bytes()).unwrap(),
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
