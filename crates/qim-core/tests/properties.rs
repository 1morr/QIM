//! 性質測試（plan §5.1）：隨機的碼表、詞庫與按鍵序列。

use std::collections::{BTreeMap, BTreeSet};

use proptest::prelude::*;
use qim_core::{
    CodeEntry, CodeTable, Engine, Key, Lexicon, LexiconPack, PAGE_SIZE, Pack, Packs, Ranking,
    Scheme, Session, UserModel, Word,
};

/// 測試方案的最長碼；碼表裡有更長的碼，那些不是字的碼。
const MAX_CODE_LENGTH: u8 = 2;

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
        4 => prop::sample::select(vec!['a', 'b', 'c', 'z', 'A']).prop_map(Key::Letter),
        1 => Just(Key::Space),
        1 => Just(Key::Enter),
        1 => Just(Key::Backspace),
        1 => Just(Key::Escape),
        1 => (0u8..=9).prop_map(Key::Digit),
        1 => Just(Key::PageUp),
        1 => Just(Key::PageDown),
        1 => Just(Key::Left),
        1 => Just(Key::Right),
        1 => Just(Key::ToggleAscii),
        1 => Just(Key::Other),
        1 => prop::sample::select(vec![',', '.', '@', '-']).prop_map(Key::Punct),
    ]
}

fn engine(table: &BTreeMap<String, Vec<char>>, words: &BTreeMap<String, Vec<String>>) -> Engine {
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abc".into(),
        max_code_length: MAX_CODE_LENGTH,
        radicals: BTreeMap::new(),
        packs: Packs {
            lexicon: Some("lexicon".into()),
            language_model: None,
        },
        ranking: Ranking::default(),
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
    Engine::new(Pack { scheme, code_table }, Some(lexicon), None).unwrap()
}

/// `text` 能拆成一串字詞（`pieces` 的「文字, 碼」），碼接起來是 `input` 開頭的一段（至少一碼）。
fn composes(text: &str, input: &str, pieces: &[(String, String)]) -> bool {
    let mut seen = BTreeSet::new();
    let mut todo = vec![(0, 0)];
    while let Some((t, c)) = todo.pop() {
        if t == text.len() && c > 0 {
            return true;
        }
        for (piece, code) in pieces {
            if text[t..].starts_with(piece.as_str())
                && input[c..].starts_with(code.as_str())
                && seen.insert((t + piece.len(), c + code.len()))
            {
                todo.push((t + piece.len(), c + code.len()));
            }
        }
    }
    false
}

proptest! {
    /// 交還宿主的鍵宿主會照打，組字還在就會漏字進輸入框、組字卻沒清。
    #[test]
    fn a_key_left_to_the_host_leaves_no_composition_behind(
        table in random_table(),
        words in random_words(),
        keys in prop::collection::vec(key(), 0..40),
    ) {
        let engine = engine(&table, &words);
        let mut user = UserModel::default();
        let mut session = Session::default();
        for key in keys {
            let snapshot = session.handle(&engine, &mut user, key);
            if !snapshot.handled {
                prop_assert_eq!(&snapshot.composition.input, "", "{:?} 交還宿主，組字還在", key);
                prop_assert_eq!(snapshot.commit, None, "{:?} 交還宿主，卻有要上屏的字", key);
            }
        }
    }

    #[test]
    fn every_candidate_can_be_typed_with_the_beginning_of_the_unselected_input(
        table in random_table(),
        words in random_words(),
        keys in prop::collection::vec(key(), 0..40),
    ) {
        // 字詞與它的碼，直接由生成的碼表與詞庫建，不經核心；碼超過方案最長碼的字打不出。
        let mut pieces: Vec<(String, String)> = Vec::new();
        let long = |code: &&String| code.len() <= usize::from(MAX_CODE_LENGTH);
        for (code, chars) in table.iter().filter(|(code, _)| long(code)) {
            pieces.extend(chars.iter().map(|c| (c.to_string(), code.clone())));
        }
        for (code, texts) in &words {
            pieces.extend(texts.iter().map(|text| (text.clone(), code.clone())));
        }

        let engine = engine(&table, &words);
        let mut user = UserModel::default();
        let mut session = Session::default();
        for key in keys {
            let snapshot = session.handle(&engine, &mut user, key);
            let input = snapshot.composition.unselected_input();
            for candidate in &snapshot.candidates {
                prop_assert!(
                    composes(&candidate.text, input, &pieces),
                    "{key:?} 後候選 {} 打不出 {input:?} 的開頭",
                    candidate.text
                );
            }
        }
    }

    #[test]
    fn candidates_computed_key_by_key_equal_those_computed_from_scratch(
        table in random_table(),
        words in random_words(),
        keys in prop::collection::vec(key(), 0..40),
    ) {
        let engine = engine(&table, &words);
        let mut user = UserModel::default();
        let mut session = Session::default();
        for key in keys {
            let snapshot = session.handle(&engine, &mut user, key);
            let from_scratch = engine.candidates(snapshot.composition.unselected_input());
            let page: Vec<String> = from_scratch
                .into_iter()
                .skip(snapshot.page * PAGE_SIZE)
                .take(PAGE_SIZE)
                .collect();
            let texts: Vec<String> = snapshot.candidates.iter().map(|c| c.text.clone()).collect();
            prop_assert_eq!(texts, page, "{:?} 後", key);
        }
    }
}
