//! 性質測試（plan §5.1）：隨機的碼表與按鍵序列。

use std::collections::{BTreeMap, BTreeSet};

use proptest::prelude::*;
use qim_core::{CodeEntry, CodeTable, Engine, Key, Pack, Scheme, Session, UserModel};

/// 輸入碼 → 字；碼含超過 `max_code_length` 的，字有重複的。
fn random_table() -> impl Strategy<Value = BTreeMap<String, Vec<char>>> {
    prop::collection::btree_map(
        "[abc]{1,3}",
        prop::collection::vec(prop::char::range('一', '十'), 1..20),
        0..12,
    )
}

fn key() -> impl Strategy<Value = Key> {
    prop_oneof![
        4 => prop::sample::select(vec!['a', 'b', 'c', 'z']).prop_map(Key::Letter),
        1 => Just(Key::Space),
        1 => Just(Key::Backspace),
        1 => Just(Key::Escape),
        1 => (0u8..=9).prop_map(Key::Digit),
        1 => Just(Key::PageUp),
        1 => Just(Key::PageDown),
    ]
}

fn engine(table: &BTreeMap<String, Vec<char>>) -> Engine {
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abc".into(),
        max_code_length: 2,
        radicals: BTreeMap::new(),
    };
    let code_table = CodeTable::from_entries(table.iter().map(|(code, chars)| {
        let entries = chars
            .iter()
            .map(|&text| CodeEntry {
                text,
                extension: false,
            })
            .collect();
        (code.clone(), entries)
    }));
    let bytes = Pack { scheme, code_table }.to_bytes().unwrap();
    Engine::from_pack_bytes(&bytes).unwrap()
}

proptest! {
    #[test]
    fn every_candidate_has_a_code_equal_to_the_current_input(
        table in random_table(),
        keys in prop::collection::vec(key(), 0..40),
    ) {
        // 字 → 它的所有碼，直接由生成的碼表建，不經核心。
        let mut codes: BTreeMap<String, BTreeSet<&str>> = BTreeMap::new();
        for (code, chars) in &table {
            for c in chars {
                codes.entry(c.to_string()).or_default().insert(code);
            }
        }

        let engine = engine(&table);
        let mut user = UserModel::default();
        let mut session = Session::default();
        for key in keys {
            let snapshot = session.handle(&engine, &mut user, key);
            let input = snapshot.composition.input.as_str();
            for candidate in &snapshot.candidates {
                let has_code = codes.get(&candidate.text).is_some_and(|c| c.contains(input));
                prop_assert!(has_code, "{key:?} 後候選 {} 沒有碼 {input:?}", candidate.text);
            }
        }
    }
}
