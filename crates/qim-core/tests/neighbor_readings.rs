//! 觸控鄰鍵讀法（M4 票 09c，照 Gboard 速成的做法，`docs/research/tolerance.md`〈Gboard 式的鄰鍵讀法〉）：
//! 觸控一下的按鍵機率裡的鄰鍵字母換進輸入碼，換出來的讀法從候選列第 2 格起插、不搶首選；實體鍵盤與
//! 開關關掉時候選不變；選了鄰鍵讀法學的是換過的碼。

mod common;

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, Engine, Key, Lexicon, LexiconPack, Options, Packs, Ranking, Scheme,
    SchemePack, Session, Snapshot, Swap, UserModel, Weights, Word,
};

/// 碼表：`a` 日、`s` 尸、`n` 弓、`r` 口、`m` 一，`an` 門、`sn` 刷、`am` 星、`nr` 同、`x` 難；詞庫：`anr` 門口。
fn engine() -> Engine {
    engine_with(None)
}

/// 同 [`engine`]，`z` 是萬用鍵。
fn wildcard_engine() -> Engine {
    engine_with(Some('z'))
}

fn engine_with(wildcard: Option<char>) -> Engine {
    let radicals = BTreeMap::from([
        ('a', "日".to_string()),
        ('s', "尸".to_string()),
        ('n', "弓".to_string()),
        ('r', "口".to_string()),
        ('m', "一".to_string()),
    ]);
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: if wildcard.is_some() {
            "abcdefghijklmnopqrstuvwxyz".into()
        } else {
            "abcdefghijklmnopqrstuvwxy".into()
        },
        max_code_length: 2,
        wildcard,
        radicals,
        packs: Packs {
            lexicon: Some("lexicon".into()),
            language_model: Some("unigram".into()),
            ..Packs::default()
        },
        ranking: Ranking::default(),
    };
    let table = [
        ("a", '日'),
        ("s", '尸'),
        ("n", '弓'),
        ("r", '口'),
        ("m", '一'),
        ("an", '門'),
        ("sn", '刷'),
        ("am", '星'),
        ("nr", '同'),
        ("x", '難'),
    ];
    let code_table = CodeTable::from_entries(table.iter().map(|&(code, text)| {
        let entry = CodeEntry {
            text,
            extension: false,
            simplified: false,
        };
        (code.to_string(), vec![entry])
    }));
    let chars = [
        ('日', 100),
        ('尸', 20),
        ('弓', 5),
        ('口', 300),
        ('一', 50),
        ('門', 200),
        ('刷', 30),
        ('星', 80),
        ('同', 400),
        ('難', 10),
    ];
    let words = BTreeMap::from([(
        "anr".to_string(),
        vec![Word {
            text: "門口".into(),
            frequency: 1000,
        }],
    )]);
    let lexicon = LexiconPack {
        id: "lexicon".into(),
        lexicon: Lexicon::new(words, chars),
    };
    let language_model = common::unigram("unigram", &chars, chars.len());
    Engine::new(
        SchemePack { scheme, code_table },
        Some(lexicon),
        Some(language_model),
    )
    .unwrap()
}

/// 點在 `letter` 鍵上、`neighbour` 鍵分到 `p` 的一下的按鍵機率。
fn weights(letter: char, neighbour: char, p: f32) -> Weights {
    Weights::new(vec![(letter, 1.0 - p - 0.01), (neighbour, p), ('q', 0.01)]).unwrap()
}

/// 觸控點一下：點在 `letter` 鍵上、`neighbour` 分到 `p`。
fn tap(letter: char, neighbour: char, p: f32) -> Key {
    Key::Weighted(weights(letter, neighbour, p))
}

/// 實體鍵盤按 `letter`，`neighbour` 的鄰鍵先驗是 `p`（同一組機率，只是來源不同）。
fn press(letter: char, neighbour: char, p: f32) -> Key {
    Key::Physical(weights(letter, neighbour, p))
}

fn type_keys(engine: &Engine, options: Options, keys: &[Key]) -> (Session, UserModel, Snapshot) {
    let mut session = Session::default();
    session.set_options(options);
    let mut user = UserModel::default();
    let mut snapshot = Snapshot::default();
    for key in keys {
        snapshot = session.handle(engine, &mut user, key.clone());
    }
    (session, user, snapshot)
}

fn texts(snapshot: &Snapshot) -> Vec<String> {
    snapshot.candidates.iter().map(|c| c.text.clone()).collect()
}

/// 想打 `anr`（門口），第一下點在 s、a 分到 0.3：觸控打錯一碼。
fn snr_tapped() -> Vec<Key> {
    vec![tap('s', 'a', 0.3), tap('n', 'm', 0.1), tap('r', 'e', 0.1)]
}

fn letters(code: &str) -> Vec<Key> {
    code.chars().map(Key::Letter).collect()
}

#[test]
fn the_intended_word_appears_from_the_second_slot_and_the_literal_reading_stays_first() {
    let engine = engine();
    let (_, _, literal) = type_keys(&engine, Options::default(), &letters("snr"));
    let (_, _, tapped) = type_keys(&engine, Options::default(), &snr_tapped());
    let list = texts(&tapped);
    assert_eq!(list[0], texts(&literal)[0], "原樣讀法照舊當首選：{list:?}");
    let at = list.iter().position(|t| t == "門口");
    assert!(at.is_some_and(|i| i >= 1), "門口在第 2 格起：{list:?}");
    assert!(!texts(&literal).contains(&"門口".to_string()));
}

#[test]
fn at_most_two_neighbor_readings_are_listed() {
    let engine = engine();
    let (_, _, literal) = type_keys(&engine, Options::default(), &letters("snr"));
    let (_, _, tapped) = type_keys(&engine, Options::default(), &snr_tapped());
    let literal = texts(&literal);
    let added = texts(&tapped)
        .iter()
        .filter(|t| !literal.contains(t))
        .count();
    assert!(
        (1..=2).contains(&added),
        "多了 {added} 個：{:?}",
        texts(&tapped)
    );
}

#[test]
fn a_single_code_also_lists_the_neighbor_character() {
    let engine = engine();
    let (_, _, snapshot) = type_keys(&engine, Options::default(), &[tap('s', 'a', 0.3)]);
    let list = texts(&snapshot);
    assert_eq!(list[0], "尸", "{list:?}");
    assert!(list[1..].contains(&"日".to_string()), "{list:?}");
}

#[test]
fn letters_below_the_floor_are_not_tried() {
    let engine = engine();
    // a 只有 s 的 0.1 倍（floor 0.2 以下）。
    let keys = [tap('s', 'a', 0.09), tap('n', 'm', 0.1), tap('r', 'e', 0.1)];
    let (_, _, tapped) = type_keys(&engine, Options::default(), &keys);
    assert!(!texts(&tapped).contains(&"門口".to_string()));
}

#[test]
fn a_physical_keyboard_with_the_same_weights_lists_the_literal_reading() {
    let engine = engine();
    let (_, _, literal) = type_keys(&engine, Options::default(), &letters("snr"));
    let keys = [
        press('s', 'a', 0.3),
        press('n', 'm', 0.1),
        press('r', 'e', 0.1),
    ];
    let (_, _, pressed) = type_keys(&engine, Options::default(), &keys);
    assert_eq!(texts(&pressed), texts(&literal));
}

#[test]
fn turning_the_option_off_lists_the_literal_reading() {
    let engine = engine();
    let (_, _, literal) = type_keys(&engine, Options::default(), &letters("snr"));
    let off = Options {
        neighbor_readings: false,
        ..Options::default()
    };
    let (_, _, tapped) = type_keys(&engine, off, &snr_tapped());
    assert_eq!(texts(&tapped), texts(&literal));
}

#[test]
fn the_option_is_on_by_default() {
    assert!(Options::default().neighbor_readings);
}

#[test]
fn choosing_a_neighbor_reading_learns_the_swapped_code() {
    let engine = engine();
    let (mut session, mut user, snapshot) = type_keys(&engine, Options::default(), &snr_tapped());
    let at = texts(&snapshot).iter().position(|t| t == "門口").unwrap();
    let snapshot = session.select(&engine, &mut user, at);
    assert_eq!(snapshot.commit.as_deref(), Some("門口"));
    let exported = user.export();
    assert!(exported.contains("\t門口\tanr\t"), "{exported}");
    assert!(!exported.contains("snr"), "不記打出的碼：{exported}");
}

#[test]
fn the_rest_after_choosing_a_prefix_keeps_its_touch_weights() {
    let engine = engine();
    // 先打 `m`（一），再照 `snr` 點：選了開頭的「一」之後，剩下的 `snr` 照樣有鄰鍵讀法。
    let mut keys = vec![tap('m', 'n', 0.05)];
    keys.extend(snr_tapped());
    let (mut session, mut user, snapshot) = type_keys(&engine, Options::default(), &keys);
    let at = texts(&snapshot).iter().position(|t| t == "一").unwrap();
    let snapshot = session.select(&engine, &mut user, at);
    assert!(snapshot.commit.is_none());
    assert!(
        texts(&snapshot).contains(&"門口".to_string()),
        "{:?}",
        texts(&snapshot)
    );
}

#[test]
fn backspace_drops_the_neighbor_readings_of_the_removed_code() {
    let engine = engine();
    let (mut session, mut user, _) = type_keys(&engine, Options::default(), &snr_tapped());
    let snapshot = session.handle(&engine, &mut user, Key::Backspace);
    let (_, _, typed) = type_keys(&engine, Options::default(), &snr_tapped()[..2]);
    assert_eq!(texts(&snapshot), texts(&typed));
}

#[test]
fn inspect_marks_the_neighbor_reading_and_the_swapped_code() {
    let engine = engine();
    let (session, user, snapshot) = type_keys(&engine, Options::default(), &snr_tapped());
    let at = texts(&snapshot).iter().position(|t| t == "門口").unwrap();
    let inspection = session.inspect(&engine, &user);
    let candidate = &inspection.candidates[at];
    let swaps = candidate.neighbor.as_ref().expect("標成鄰鍵讀法");
    assert_eq!(
        swaps.swaps,
        vec![Swap {
            position: 0,
            typed: 's',
            code: 'a'
        }]
    );
    assert!(swaps.penalty > 0.0);
    assert_eq!(candidate.segments[0].code, "anr");
    assert!(inspection.candidates[0].neighbor.is_none());
}

#[test]
fn a_neighbor_letter_is_never_swapped_into_the_wildcard() {
    let engine = wildcard_engine();
    // 點在 x、z 分到 0.4：換成 z 會展開成全部單碼的字，不換（協調者定）。
    let (_, _, literal) = type_keys(&engine, Options::default(), &letters("x"));
    let (_, _, tapped) = type_keys(&engine, Options::default(), &[tap('x', 'z', 0.4)]);
    assert_eq!(texts(&tapped), texts(&literal));
}

#[test]
fn a_typed_wildcard_is_never_swapped_for_a_neighbor_letter() {
    let engine = wildcard_engine();
    // 點在 z、x 分到 0.4：打出的萬用鍵不換成 x，排在後面的「難」不會被當成鄰鍵讀法移到第 2 格。
    let (_, _, literal) = type_keys(&engine, Options::default(), &letters("z"));
    let keys = [tap('z', 'x', 0.4)];
    let (_, _, tapped) = type_keys(&engine, Options::default(), &keys);
    assert_eq!(texts(&tapped), texts(&literal));
}

#[test]
fn a_neighbor_reading_never_becomes_the_first_candidate_even_when_the_literal_reading_has_none() {
    let engine = engine();
    // w 不在碼表：原樣讀法沒有候選，a 的「日」也不當首選（不搶首選）。
    let (_, _, literal) = type_keys(&engine, Options::default(), &letters("w"));
    assert!(texts(&literal).is_empty());
    let (_, _, tapped) = type_keys(&engine, Options::default(), &[tap('w', 'a', 0.4)]);
    assert_eq!(texts(&tapped), texts(&literal));
}
