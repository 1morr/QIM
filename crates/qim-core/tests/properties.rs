//! 性質測試（plan §5.1）：隨機的碼表、詞庫與按鍵序列。

use std::collections::{BTreeMap, BTreeSet};

use proptest::prelude::*;
use qim_core::{
    CodeEntry, CodeTable, Engine, Key, Lexicon, LexiconPack, Pack, Packs, Scheme, Session,
    UserModel, Word,
};

/// 輸入碼 → 字；碼含超過 `max_code_length` 的，字有重複的。
fn random_table() -> impl Strategy<Value = BTreeMap<String, Vec<char>>> {
    prop::collection::btree_map(
        "[abc]{1,3}",
        prop::collection::vec(prop::char::range('一', '十'), 1..20),
        0..12,
    )
}

/// 詞庫：輸入碼 → 詞；碼比方案的最長碼長，也有和碼表同碼的。
fn random_words() -> impl Strategy<Value = BTreeMap<String, Vec<String>>> {
    prop::collection::btree_map(
        "[abc]{2,5}",
        prop::collection::vec("[一-十]{2,3}", 1..5),
        0..8,
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

fn engine(table: &BTreeMap<String, Vec<char>>, words: &BTreeMap<String, Vec<String>>) -> Engine {
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abc".into(),
        max_code_length: 2,
        radicals: BTreeMap::new(),
        packs: Packs {
            lexicon: Some("lexicon".into()),
        },
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
    let lexicon = Lexicon::new(
        words.iter().map(|(code, texts)| {
            let words = texts
                .iter()
                .map(|text| Word {
                    text: text.clone(),
                    frequency: 1,
                })
                .collect();
            (code.clone(), words)
        }),
        // 字頻只改字序，不該讓候選多出別的碼的字。
        [('一', 9), ('五', 3)],
    );
    let lexicon = LexiconPack {
        id: "lexicon".into(),
        lexicon,
    };
    Engine::new(Pack { scheme, code_table }, Some(lexicon)).unwrap()
}

proptest! {
    #[test]
    fn every_candidate_has_a_code_equal_to_the_current_input(
        table in random_table(),
        words in random_words(),
        keys in prop::collection::vec(key(), 0..40),
    ) {
        // 字詞 → 它的所有碼，直接由生成的碼表與詞庫建，不經核心。
        let mut codes: BTreeMap<String, BTreeSet<&str>> = BTreeMap::new();
        for (code, chars) in &table {
            for c in chars {
                codes.entry(c.to_string()).or_default().insert(code);
            }
        }
        for (code, texts) in &words {
            for text in texts {
                codes.entry(text.clone()).or_default().insert(code);
            }
        }

        let engine = engine(&table, &words);
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
