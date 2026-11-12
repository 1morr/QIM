//! 性質測試（plan §5.1）：隨機的碼表、詞庫與按鍵序列。

use std::collections::{BTreeMap, BTreeSet};

use proptest::prelude::*;
use qim_core::{
    CodeEntry, CodeTable, Date, Engine, Key, Lexicon, LexiconPack, Options, PAGE_SIZE, Packs,
    Ranking, SEPARATOR, Scheme, SchemePack, Session, ShiftKey, Snapshot, UserModel, Word,
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
#[derive(Debug, Clone)]
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
        wildcard: None,
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
                simplified: false,
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

/// 匯出檔的這一行是方案 `scheme` 的列或 tick 行。
fn is_row_of(line: &str, scheme: &str) -> bool {
    line.starts_with(&format!("{scheme}\t")) || line.starts_with(&format!("#@/tick/{scheme}\t"))
}

/// 匯出檔裡方案 `scheme` 的列與 tick 行。
fn rows_of(exported: &str, scheme: &str) -> Vec<String> {
    exported
        .lines()
        .filter(|line| is_row_of(line, scheme))
        .map(String::from)
        .collect()
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
            let snapshot = act(&mut session, &engine, &mut user, action.clone());
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
            let snapshot = act(&mut session, &engine, &mut user, action.clone());
            // 沒有組字時的候選是聯想（M3 票 13），不由輸入碼打出。
            if snapshot.composition.input.is_empty() {
                continue;
            }
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
            let snapshot = act(&mut session, &engine, &mut user, action.clone());
            // 沒有組字時的候選是聯想（M3 票 13），不是輸入碼的候選。
            if snapshot.composition.input.is_empty() {
                continue;
            }
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

    /// 改選某一段後，沒選定的分段等於以選定的分段為前文、把剩下的輸入碼從頭打一次的分段
    /// （plan §2.1 的重算、§2.3 接選定分段的最後一字）；各分段的碼接起來是整串輸入碼，
    /// 選定的都在沒選定的前面。
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
            let snapshot = act(&mut session, &engine, &mut user, action.clone());
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
            fresh.set_context(&composition.selected_text());
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
            act(&mut session, &engine, &mut user, action.clone());
        }
        let before = user.clone();
        session.set_options(Options { incognito: true, ..Options::default() });
        for action in actions {
            act(&mut session, &engine, &mut user, action.clone());
            prop_assert_eq!(&user, &before, "{:?} 後", action);
        }
    }

    /// 學習關掉時怎麼打、選、上屏都不寫學習資料（brief §4.5），之前學過的也不動。
    #[test]
    fn learning_off_leaves_the_user_model_unchanged(
        table in random_table(),
        words in random_words(),
        learning in prop::collection::vec(action(), 0..40),
        actions in prop::collection::vec(action(), 0..40),
    ) {
        let engine = engine(&table, &words);
        let mut user = UserModel::default();
        let mut session = Session::default();
        for action in learning {
            act(&mut session, &engine, &mut user, action.clone());
        }
        let before = user.clone();
        session.set_options(Options { learning: false, ..Options::default() });
        for action in actions {
            act(&mut session, &engine, &mut user, action.clone());
            prop_assert_eq!(&user, &before, "{:?} 後", action);
        }
    }

    /// 同一個 `UserModel` 交替用在兩個同碼不同字的方案：一個方案的上屏不寫另一個方案的學習計數與
    /// 使用者詞，候選也不讀它們（plan §2.4）。個人 bigram 只看文字、各方案共用，不在此列。
    #[test]
    fn one_scheme_neither_writes_nor_reads_the_other_schemes_learning(
        table in random_table(),
        words in random_words(),
        actions in prop::collection::vec((any::<bool>(), action()), 0..100),
        inputs in prop::collection::vec("[abc]{1,4}", 1..5),
    ) {
        let (other_table, other_words) = other_scheme(&table, &words);
        let engines = [
            engine_for("quick", &table, &words),
            engine_for("cangjie", &other_table, &other_words),
        ];
        let mut shared = UserModel::default();
        let mut sessions = [Session::default(), Session::default()];
        for (second, action) in actions {
            let i = usize::from(second);
            let other = &engines[1 - i].scheme().id;
            let before = rows_of(&shared.export(), other);
            act(&mut sessions[i], &engines[i], &mut shared, action.clone());
            prop_assert_eq!(rows_of(&shared.export(), other), before, "方案 {} 做 {:?} 後", engines[i].scheme().id, action);
        }
        for i in 0..2 {
            let other = &engines[1 - i].scheme().id;
            let exported = shared.export();
            let without: String = exported
                .lines()
                .filter(|line| !is_row_of(line, other))
                .map(|line| format!("{line}\n"))
                .collect();
            let without = UserModel::import(&without, None).map(|i| i.user).unwrap();
            for input in &inputs {
                prop_assert_eq!(
                    engines[i].candidates(&shared, input),
                    engines[i].candidates(&without, input),
                    "方案 {} 的 `{}`", engines[i].scheme().id, input
                );
            }
        }
    }

    /// 匯出後匯入等價（plan §5.1）：學過的計數、個人 bigram、自動造詞、手動加的詞與觸發詞、常用語都還在，
    /// 每個條目最後使用的日期（打字中途換日）與裝置 id 也是。
    #[test]
    fn exporting_then_importing_gives_an_equal_user_model(
        table in random_table(),
        words in random_words(),
        keys in prop::collection::vec((any::<bool>(), key(), prop::option::weighted(0.1, r"[一a\\\t，]{0,3}"), prop::option::weighted(0.1, 0..400u32)), 0..100),
        device in prop::option::of("[0-9a-f-]{1,64}"),
        added in prop::collection::vec(("[一-十]{1,3}", prop::option::of("[abc]{1,3}")), 0..5),
        phrases in prop::collection::vec((r"[一a \\\n\t]{1,6}", prop::option::of("[abc]{1,3}"), any::<bool>()), 0..5),
        removed in prop::collection::vec(r"[一a \\\n\t]{1,6}", 0..5),
    ) {
        let (other_table, other_words) = other_scheme(&table, &words);
        let engines = [
            engine_for("quick", &table, &words),
            engine_for("cangjie", &other_table, &other_words),
        ];
        let mut user = UserModel::default();
        if let Some(device) = &device {
            user.set_device(device).unwrap();
        }
        let mut sessions = [Session::default(), Session::default()];
        for (second, key, context, day) in keys {
            let i = usize::from(second);
            // 換日：2026 年的第幾天（同一天裡不換）。
            if let Some(day) = day {
                user.set_today(Date::from_ymd(2026, 1 + day / 31 % 12, 1 + day % 28).unwrap());
            }
            // 前文給個人 bigram 的前一個字詞（含要跳脫的字）。
            if let Some(context) = context {
                sessions[i].set_context(&context);
            }
            sessions[i].handle(&engines[i], &mut user, key);
        }
        for (text, trigger) in &added {
            // 字沒有碼又沒給觸發詞時加不進去，那就不加。
            let _ = user.add_word(&engines[0], text, trigger.as_deref());
        }
        for (text, trigger, _) in &phrases {
            user.add_phrase(&engines[1], text, trigger.as_deref()).unwrap();
        }
        // 刪掉一部分常用語（留下的照樣匯出）：刪常用語會連它學到的計數一起刪，方案可能只剩 tick 行；
        // 刪不到的也不該留下痕跡。
        let dropped = phrases.iter().filter(|(_, _, drop)| *drop).map(|(text, _, _)| text);
        for text in dropped.chain(&removed) {
            user.remove_phrase(&engines[1], text);
        }

        let exported = user.export();
        prop_assert_eq!(exported.lines().next(), Some("# qim-userdict v3"));
        let imported = UserModel::import(&exported, user.today());
        prop_assert_eq!(imported.map(|i| (i.user, i.warnings)), Ok((user, vec![])));
    }
}

/// 三台裝置（各一份個人模型與 Session）上做的事，跨裝置合併的性質測試用（M4 票 17）。
#[derive(Debug, Clone)]
enum Step {
    /// 第幾台做一個 Session 的動作。
    Act(usize, Action),
    /// 第幾台清掉全部候選的第幾個學到的（撤銷學習）。
    Forget(usize, usize),
    /// 第幾台存一條常用語（文字、觸發詞），或刪掉它。
    Phrase(usize, String, Option<String>, bool),
    /// 第幾台刪掉一個英文使用者詞（照它學到的清單的第幾個）。
    ForgetEnglish(usize, usize),
    /// 第一台匯入第二台匯出的檔（合併）：讓三台有共同的歷史、墓碑與各台的次數。
    Sync(usize, usize),
    /// 換日（三台同一天；2026 年內，墓碑不會過保留期）。
    Day(u32),
    /// 第幾台清空學習資料（清空記號同步到別台，使用者 2026-11-13 決定）。
    Clear(usize),
    /// 第一台從第二台匯出的檔還原（新的清空記號，協調者 2026-11-13 決定）。
    Restore(usize, usize),
}

fn step() -> impl Strategy<Value = Step> {
    let device = 0usize..3;
    prop_oneof![
        12 => (device.clone(), action()).prop_map(|(i, a)| Step::Act(i, a)),
        2 => (device.clone(), 0usize..12).prop_map(|(i, n)| Step::Forget(i, n)),
        1 => (device.clone(), "[一二]{1,2}", prop::option::of("[abc]{1,2}"), any::<bool>())
            .prop_map(|(i, t, k, r)| Step::Phrase(i, t, k, r)),
        1 => (device.clone(), 0usize..3).prop_map(|(i, n)| Step::ForgetEnglish(i, n)),
        2 => (device.clone(), device).prop_map(|(i, j)| Step::Sync(i, j)),
        1 => (0u32..300).prop_map(Step::Day),
        1 => (0usize..3).prop_map(Step::Clear),
        1 => (0usize..3, 0usize..3).prop_map(|(i, j)| Step::Restore(i, j)),
    ]
}

/// `into` 匯入 `from` 匯出的檔、合併進來（殼的同步：各自匯出快照、匯入時合併）。
fn merged(into: &UserModel, from: &UserModel) -> UserModel {
    let mut user = into.clone();
    let snapshot = UserModel::import(&from.export(), into.today()).unwrap();
    assert_eq!(snapshot.warnings, []);
    user.merge(snapshot.user);
    user
}

/// 學習資料：匯出檔去掉這台的裝置 id（合併的結果留著自己的 id，比的是學到的東西）。
fn learning(user: &UserModel) -> String {
    user.export()
        .lines()
        .filter(|l| !l.starts_with("#@/device\t"))
        .map(|l| format!("{l}\n"))
        .collect()
}

/// 照 `steps` 跑三台裝置，回傳三份個人模型。
fn run_devices(engine: &Engine, steps: Vec<Step>) -> [UserModel; 3] {
    let mut users: [UserModel; 3] = Default::default();
    let mut sessions: [Session; 3] = Default::default();
    for (i, user) in users.iter_mut().enumerate() {
        user.set_device(&format!("device-{i}")).unwrap();
        user.set_today(Date::from_ymd(2026, 1, 1).unwrap());
    }
    for step in steps {
        match step {
            Step::Act(i, action) => {
                act(&mut sessions[i], engine, &mut users[i], action);
            }
            Step::Forget(i, n) => {
                sessions[i].forget(engine, &mut users[i], n);
            }
            Step::Phrase(i, text, trigger, remove) => {
                if remove {
                    users[i].remove_phrase(engine, &text);
                } else {
                    users[i]
                        .add_phrase(engine, &text, trigger.as_deref())
                        .unwrap();
                }
            }
            Step::ForgetEnglish(i, n) => {
                if let Some(word) = users[i].english_user_words().get(n).cloned() {
                    users[i].forget_english_user_word(&word);
                }
            }
            Step::Sync(i, j) => {
                users[i] = merged(&users[i], &users[j]);
                sessions[i].expire_undo(&mut users[i]);
            }
            Step::Clear(i) => users[i].clear(),
            Step::Restore(i, j) => {
                let backup = UserModel::import(&users[j].export(), None).unwrap().user;
                users[i].restore(backup);
                sessions[i].expire_undo(&mut users[i]);
            }
            Step::Day(day) => {
                let date = Date::from_ymd(2026, 1 + day / 31 % 12, 1 + day % 28).unwrap();
                for user in &mut users {
                    user.set_today(date);
                }
            }
        }
    }
    users
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// 跨裝置合併（plan §2.4〈跨裝置合併〉，ADR 0007）和順序無關、重複合併不變：交換律、結合律、冪等。
    /// 三台各自學習、撤銷學習、存刪常用語與英文使用者詞、清空，中間互相同步過，最後兩兩、三台合併比學習資料。
    #[test]
    fn merging_is_commutative_associative_and_idempotent(
        table in random_table(),
        words in random_words(),
        steps in prop::collection::vec(step(), 0..300),
    ) {
        let engine = engine(&table, &words);
        let [a, b, c] = run_devices(&engine, steps);
        prop_assert_eq!(learning(&merged(&a, &b)), learning(&merged(&b, &a)), "交換律");
        prop_assert_eq!(
            learning(&merged(&merged(&a, &b), &c)),
            learning(&merged(&a, &merged(&b, &c))),
            "結合律"
        );
        prop_assert_eq!(learning(&merged(&a, &a)), learning(&a), "合併自己");
        let ab = merged(&a, &b);
        prop_assert_eq!(learning(&merged(&ab, &b)), learning(&ab), "再合併一次");
        // 合併後的匯出檔照樣讀得回來、等價。
        let again = UserModel::import(&ab.export(), ab.today()).unwrap().user;
        prop_assert_eq!(again, ab);
    }

    /// 刪除後再用會復活（plan §2.4〈跨裝置合併〉）：三台有各自的歷史，第一台選一個候選再撤銷學習、同步給
    /// 第二台，第二台再選一次；之後不論怎麼合併，那一列都在、有學到的值。
    #[test]
    fn a_deleted_entry_used_again_is_alive_in_every_merge(
        table in random_table(),
        words in random_words(),
        steps in prop::collection::vec(step(), 0..120),
        input in "[abc]{1,3}",
    ) {
        let engine = engine(&table, &words);
        // 清空會讓代數大的那邊整份算數（清空記號），刪除與復活只在同一代比；清空另由三條律與 `sync_merge.rs` 測。
        let steps = steps
            .into_iter()
            .filter(|s| !matches!(s, Step::Clear(_) | Step::Restore(..)))
            .collect();
        let [mut a, mut b, c] = run_devices(&engine, steps);
        let mut session = Session::default();
        for letter in input.chars() {
            session.handle(&engine, &mut a, Key::Letter(letter));
        }
        let Some(text) = session.all_candidates().first().map(|c| c.text.clone()) else {
            return Ok(());
        };
        // 整串是一個候選才算一列（方案碼表外的打不出時跳過）。
        if !engine.candidates(&UserModel::default(), &input).contains(&text) {
            return Ok(());
        }
        session.select(&engine, &mut a, 0);
        session.expire_undo(&mut a);
        let row = format!("entry\tquick\t{text}\t{input}\t");
        prop_assume!(a.export().contains(&row));
        for letter in input.chars() {
            session.handle(&engine, &mut a, Key::Letter(letter));
        }
        let at = session.all_candidates().iter().position(|c| c.text == text);
        session.forget(&engine, &mut a, at.unwrap());
        session.handle(&engine, &mut a, Key::Escape);
        prop_assert!(!a.export().contains(&row), "刪掉後不留原文");

        b = merged(&b, &a);
        let mut other = Session::default();
        for letter in input.chars() {
            other.handle(&engine, &mut b, Key::Letter(letter));
        }
        let at = other.all_candidates().iter().position(|c| c.text == text);
        prop_assume!(at.is_some());
        other.select(&engine, &mut b, at.unwrap());
        for user in [merged(&b, &a), merged(&a, &b), merged(&merged(&c, &a), &b), merged(&b, &merged(&c, &a))] {
            let exported = user.export();
            let line = exported.lines().find(|l| l.starts_with(&row));
            prop_assert!(line.is_some_and(|l| !l.split('\t').nth(7).unwrap().is_empty()), "{}", exported);
        }
    }
}
