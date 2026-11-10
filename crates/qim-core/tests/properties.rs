//! 性質測試（plan §5.1）：隨機的碼表、詞庫與按鍵序列。

use std::collections::{BTreeMap, BTreeSet};

use proptest::prelude::*;
use qim_core::{
    CodeEntry, CodeTable, Engine, Key, Lexicon, LexiconPack, Options, PAGE_SIZE, Packs, Ranking,
    SEPARATOR, Scheme, SchemePack, Session, ShiftKey, Snapshot, UserModel, Word,
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
        1 => prop::sample::select(vec![ShiftKey::Left, ShiftKey::Right]).prop_map(Key::ToggleAscii),
        1 => Just(Key::Other),
        1 => prop::sample::select(vec![',', '.', '@', '-', SEPARATOR]).prop_map(Key::Punct),
    ]
}

/// 殼對 `Session` 做的事：按鍵、點組字的某一段、點某個候選（全部候選的位置）。
#[derive(Debug, Clone, Copy)]
enum Action {
    Key(Key),
    Focus(usize),
    Select(usize),
}

fn action() -> impl Strategy<Value = Action> {
    prop_oneof![
        8 => key().prop_map(Action::Key),
        1 => (0usize..5).prop_map(Action::Focus),
        1 => (0usize..12).prop_map(Action::Select),
    ]
}

fn act(session: &mut Session, engine: &Engine, user: &mut UserModel, action: Action) -> Snapshot {
    match action {
        Action::Key(key) => session.handle(engine, user, key),
        Action::Focus(segment) => session.focus_segment(engine, user, segment),
        Action::Select(index) => session.select(engine, user, index),
    }
}

/// 候選列的是哪一串輸入碼：目前分段的碼，沒有目前分段時是剩下沒選定的整串；臨時英文
/// 沒有候選，是空字串。
fn listed_input(snapshot: &Snapshot) -> &str {
    let composition = &snapshot.composition;
    if inline_english(snapshot) {
        return "";
    }
    composition
        .segments
        .get(composition.focus)
        .map_or(composition.unselected_input(), |s| s.code.as_str())
}

/// 臨時英文（組字中單按留碼的 Shift）：最後一段是 `raw` 的分段。
fn inline_english(snapshot: &Snapshot) -> bool {
    snapshot.composition.segments.last().is_some_and(|s| s.raw)
}

fn engine(table: &BTreeMap<String, Vec<char>>, words: &BTreeMap<String, Vec<String>>) -> Engine {
    engine_for("quick", table, words)
}

/// 方案 `id` 的引擎；碼表與詞庫同 [`engine`]。
fn engine_for(
    id: &str,
    table: &BTreeMap<String, Vec<char>>,
    words: &BTreeMap<String, Vec<String>>,
) -> Engine {
    let scheme = Scheme {
        id: id.into(),
        name: id.into(),
        version: "1".into(),
        alphabet: "abc".into(),
        max_code_length: MAX_CODE_LENGTH,
        radicals: BTreeMap::new(),
        packs: Packs {
            lexicon: Some("lexicon".into()),
            language_model: None,
            ..Packs::default()
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
    Engine::new(SchemePack { scheme, code_table }, Some(lexicon), None).unwrap()
}

/// 同碼不同字的另一個方案：每個碼的字倒過來排、前面多一個「甲」，詞也倒過來；
/// 同一組（碼, 字詞）兩個方案都有，排的位置不同，學習計數混在一起就看得出來。
fn other_scheme(
    table: &BTreeMap<String, Vec<char>>,
    words: &BTreeMap<String, Vec<String>>,
) -> (BTreeMap<String, Vec<char>>, BTreeMap<String, Vec<String>>) {
    let table = table
        .iter()
        .map(|(code, chars)| {
            let chars = std::iter::once('甲')
                .chain(chars.iter().rev().copied())
                .collect();
            (code.clone(), chars)
        })
        .collect();
    let words = words
        .iter()
        .map(|(code, texts)| (code.clone(), texts.iter().rev().cloned().collect()))
        .collect();
    (table, words)
}

/// `text` 能拆成一串字詞（`pieces` 的「文字, 碼」），碼接起來是 `input` 開頭的一段（至少一碼）；
/// 分隔記號只能在字詞之間，不在碼裡。
fn composes(text: &str, input: &str, pieces: &[(String, String)]) -> bool {
    let mut seen = BTreeSet::new();
    let mut todo = vec![(0, 0)];
    while let Some((t, c)) = todo.pop() {
        if t == text.len() && c > 0 {
            return true;
        }
        if input[c..].starts_with(SEPARATOR) && c > 0 && seen.insert((t, c + 1)) {
            todo.push((t, c + 1));
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
        actions in prop::collection::vec(action(), 0..40),
    ) {
        let engine = engine(&table, &words);
        let mut user = UserModel::default();
        let mut session = Session::default();
        for action in actions {
            let snapshot = act(&mut session, &engine, &mut user, action);
            if !snapshot.handled {
                prop_assert_eq!(&snapshot.composition.input, "", "{:?} 交還宿主，組字還在", action);
                prop_assert_eq!(snapshot.commit, None, "{:?} 交還宿主，卻有要上屏的字", action);
            }
        }
    }

    #[test]
    fn every_candidate_can_be_typed_with_the_beginning_of_the_listed_input(
        table in random_table(),
        words in random_words(),
        actions in prop::collection::vec(action(), 0..40),
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
        for action in actions {
            let snapshot = act(&mut session, &engine, &mut user, action);
            let input = listed_input(&snapshot);
            for candidate in &snapshot.candidates {
                prop_assert!(
                    composes(&candidate.text, input, &pieces),
                    "{action:?} 後候選 {} 打不出 {input:?} 的開頭",
                    candidate.text
                );
            }
        }
    }

    #[test]
    fn candidates_computed_key_by_key_equal_those_computed_from_scratch(
        table in random_table(),
        words in random_words(),
        actions in prop::collection::vec(action(), 0..40),
    ) {
        let engine = engine(&table, &words);
        let mut user = UserModel::default();
        let mut session = Session::default();
        for action in actions {
            let snapshot = act(&mut session, &engine, &mut user, action);
            let from_scratch = engine.candidates(&user, listed_input(&snapshot));
            let page: Vec<String> = from_scratch
                .into_iter()
                .skip(snapshot.page * PAGE_SIZE)
                .take(PAGE_SIZE)
                .collect();
            let texts: Vec<String> = snapshot.candidates.iter().map(|c| c.text.clone()).collect();
            prop_assert_eq!(texts, page, "{:?} 後", action);
        }
    }

    /// 改選某一段後，沒選定的分段等於把剩下的輸入碼從頭打一次的分段（plan §2.1 的重算）；
    /// 各分段的碼接起來是整串輸入碼，選定的都在沒選定的前面。
    #[test]
    fn unselected_segments_equal_those_of_typing_the_unselected_input_afresh(
        table in random_table(),
        words in random_words(),
        actions in prop::collection::vec(action(), 0..40),
    ) {
        let engine = engine(&table, &words);
        let mut user = UserModel::default();
        let mut session = Session::default();
        for action in actions {
            let snapshot = act(&mut session, &engine, &mut user, action);
            let composition = &snapshot.composition;
            let codes: String = composition.segments.iter().map(|s| s.code.as_str()).collect();
            prop_assert_eq!(&codes, &composition.input, "{:?} 後", action);
            let selected = composition.segments.iter().take_while(|s| s.selected).count();
            prop_assert!(composition.segments[selected..].iter().all(|s| !s.selected), "{:?} 後", action);
            // 臨時英文：沒選定的輸入碼整串是一段英文。
            if inline_english(&snapshot) {
                prop_assert_eq!(composition.segments.len(), selected + 1, "{:?} 後", action);
                prop_assert_eq!(&composition.segments[selected].text, composition.unselected_input(), "{:?} 後", action);
                continue;
            }

            let mut fresh = Session::default();
            let mut fresh_user = user.clone();
            let mut typed = Snapshot::default();
            for c in composition.unselected_input().chars() {
                typed = fresh.handle(&engine, &mut fresh_user, Key::for_input(c));
            }
            prop_assert_eq!(&composition.segments[selected..], &typed.composition.segments[..], "{:?} 後", action);
        }
    }

    /// 無痕模式下怎麼打、選、上屏都不寫學習資料（plan §5.1），之前學過的也不動。
    #[test]
    fn incognito_leaves_the_user_model_unchanged(
        table in random_table(),
        words in random_words(),
        learning in prop::collection::vec(action(), 0..40),
        actions in prop::collection::vec(action(), 0..40),
    ) {
        let engine = engine(&table, &words);
        let mut user = UserModel::default();
        let mut session = Session::default();
        for action in learning {
            act(&mut session, &engine, &mut user, action);
        }
        let before = user.clone();
        session.set_options(Options { incognito: true, ..Options::default() });
        for action in actions {
            act(&mut session, &engine, &mut user, action);
            prop_assert_eq!(&user, &before, "{:?} 後", action);
        }
    }

    /// 同一個 `UserModel` 交替用在兩個同碼不同字的方案，各方案的候選和只用那個方案時相同
    /// （plan §2.4）。
    #[test]
    fn two_schemes_sharing_a_user_model_do_not_affect_each_other(
        table in random_table(),
        words in random_words(),
        actions in prop::collection::vec((any::<bool>(), action()), 0..100),
    ) {
        let (other_table, other_words) = other_scheme(&table, &words);
        let engines = [
            engine_for("quick", &table, &words),
            engine_for("cangjie", &other_table, &other_words),
        ];
        let mut shared = UserModel::default();
        let mut alone = [UserModel::default(), UserModel::default()];
        let mut sessions = [Session::default(), Session::default()];
        let mut alone_sessions = [Session::default(), Session::default()];
        for (second, action) in actions {
            let i = usize::from(second);
            let together: Snapshot = act(&mut sessions[i], &engines[i], &mut shared, action);
            let by_itself = act(&mut alone_sessions[i], &engines[i], &mut alone[i], action);
            prop_assert_eq!(together, by_itself, "方案 {} 做 {:?} 後", engines[i].scheme().id, action);
        }
    }

    /// 匯出後匯入等價（plan §5.1）：學過的計數、自動造詞、手動加的詞與觸發詞都還在。
    #[test]
    fn exporting_then_importing_gives_an_equal_user_model(
        table in random_table(),
        words in random_words(),
        keys in prop::collection::vec((any::<bool>(), key()), 0..100),
        added in prop::collection::vec(("[一-十]{1,3}", prop::option::of("[abc]{1,3}")), 0..5),
    ) {
        let (other_table, other_words) = other_scheme(&table, &words);
        let engines = [
            engine_for("quick", &table, &words),
            engine_for("cangjie", &other_table, &other_words),
        ];
        let mut user = UserModel::default();
        let mut sessions = [Session::default(), Session::default()];
        for (second, key) in keys {
            let i = usize::from(second);
            sessions[i].handle(&engines[i], &mut user, key);
        }
        for (text, trigger) in &added {
            // 字沒有碼又沒給觸發詞時加不進去，那就不加。
            let _ = user.add_word(&engines[0], text, trigger.as_deref());
        }

        let exported = user.export();
        prop_assert_eq!(exported.lines().next(), Some("# qim-userdict v1"));
        let imported = UserModel::import(&exported);
        prop_assert_eq!(imported, Ok(user));
    }
}
