//! 鄰鍵讀法（M4 票 09b 的實驗）：觸控的按鍵機率裡的鄰鍵字母換進輸入碼，換出來的讀法插在候選列的指定格，
//! 原樣讀法照舊當首選，點擊位置本身模稜兩可時才讓鄰鍵讀法當首選。只給實驗用
//! （`Engine::with_neighbor_readings`），預設關；量完刪掉（`docs/research/tolerance.md`）。

mod common;

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, Engine, Key, Lexicon, LexiconPack, NeighborReadings, Packs, Ranking,
    Scheme, SchemePack, Session, Snapshot, UserModel, Weights, Word,
};

/// 碼表：`a` 日、`s` 尸、`n` 弓、`r` 口、`m` 一，`an` 門、`sn` 刷、`am` 星、`nr` 同；詞庫：`anr` 門口。
fn engine() -> Engine {
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
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        wildcard: None,
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

/// 觸控的按鍵機率都收（實驗裡判斷是不是觸控的鍵由殼決定）。
fn touch(_: &Weights) -> bool {
    true
}

fn readings() -> NeighborReadings {
    NeighborReadings {
        floor: 0.01,
        letters: 2,
        weight: 1.0,
        slot: 2,
        limit: 3,
        ambiguous: None,
        accepts: touch,
    }
}

/// 點在 `letter` 鍵上、`neighbour` 鍵分到 `p` 的一下。
fn tap(letter: char, neighbour: char, p: f32) -> Key {
    Key::Weighted(
        Weights::new(vec![(letter, 1.0 - p - 0.01), (neighbour, p), ('q', 0.01)]).unwrap(),
    )
}

/// 沒有鄰鍵的一下。
fn exact(letter: char) -> Key {
    Key::Weighted(Weights::new(vec![(letter, 1.0)]).unwrap())
}

fn type_keys(engine: &Engine, keys: &[Key]) -> (Session, UserModel, Snapshot) {
    let mut session = Session::default();
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

/// `snr`：第一下點在 s、a 分到 `p`。
fn snr(p: f32) -> Vec<Key> {
    vec![tap('s', 'a', p), exact('n'), exact('r')]
}

#[test]
fn off_by_default_the_list_is_the_literal_reading() {
    let engine = engine();
    let (_, _, weighted) = type_keys(&engine, &snr(0.2));
    let (_, _, letters) = type_keys(
        &engine,
        &[Key::Letter('s'), Key::Letter('n'), Key::Letter('r')],
    );
    assert_eq!(texts(&weighted), texts(&letters));
    assert!(!texts(&weighted).contains(&"門口".to_string()));
}

#[test]
fn the_neighbour_reading_goes_into_the_slot_and_the_literal_reading_stays_first() {
    let engine = engine().with_neighbor_readings(readings());
    let (_, _, literal) = type_keys(&engine.clone(), &[exact('s'), exact('n'), exact('r')]);
    let (_, _, snapshot) = type_keys(&engine, &snr(0.2));
    let list = texts(&snapshot);
    assert_eq!(list[0], texts(&literal)[0], "{list:?}");
    assert_eq!(list[1], "門口", "{list:?}");
    // 鄰鍵讀法最多 3 個、接在第 2 格起；原樣的候選照原本的順序排在其餘的位置。
    let literal = texts(&literal);
    let added: Vec<&String> = list.iter().filter(|t| !literal.contains(t)).collect();
    assert_eq!(added.len(), 3, "{list:?}");
    assert!(list[1..4].iter().all(|t| !literal.contains(t)), "{list:?}");
    let kept: Vec<String> = list
        .iter()
        .filter(|t| literal.contains(t))
        .cloned()
        .collect();
    assert_eq!(kept, literal);
}

#[test]
fn a_single_key_lists_the_neighbour_keys_characters_after_the_slot() {
    let engine = engine().with_neighbor_readings(NeighborReadings {
        slot: 2,
        ..readings()
    });
    let (_, _, snapshot) = type_keys(&engine, &[tap('s', 'a', 0.2)]);
    assert_eq!(texts(&snapshot), ["尸", "日"]);
}

#[test]
fn at_most_limit_neighbour_readings_are_listed() {
    let engine = engine().with_neighbor_readings(NeighborReadings {
        limit: 1,
        ..readings()
    });
    // a 鍵上、s 與 q 都是鄰鍵：s 換出 `sn` 刷，q 換不出字。`m` 鍵上、n 是鄰鍵：`an` 門。
    let keys = [
        Key::Weighted(Weights::new(vec![('a', 0.6), ('s', 0.3), ('q', 0.1)]).unwrap()),
        tap('m', 'n', 0.2),
    ];
    let (_, _, snapshot) = type_keys(&engine, &keys);
    let list = texts(&snapshot);
    let neighbours = list.iter().filter(|t| ["門", "刷"].contains(&t.as_str()));
    assert_eq!(neighbours.count(), 1, "{list:?}");
    assert_eq!(list[0], "星");
}

#[test]
fn letters_below_the_floor_are_not_tried() {
    let engine = engine().with_neighbor_readings(NeighborReadings {
        floor: 0.5,
        ..readings()
    });
    let (_, _, snapshot) = type_keys(&engine, &snr(0.2));
    assert!(!texts(&snapshot).contains(&"門口".to_string()));
}

#[test]
fn an_ambiguous_tap_lets_the_neighbour_reading_take_the_first_place() {
    let ambiguous = NeighborReadings {
        ambiguous: Some(0.8),
        ..readings()
    };
    let engine = engine().with_neighbor_readings(ambiguous);
    // s 0.5、a 0.49：比 0.98 ≥ 0.8，門口的分數又高過原樣讀法。
    let (_, _, snapshot) = type_keys(&engine, &snr(0.49));
    assert_eq!(texts(&snapshot)[0], "門口");
    // 點在 s 的中心附近（a 只分到 0.2）：照舊不搶首選。
    let (_, _, snapshot) = type_keys(&engine, &snr(0.2));
    assert_ne!(texts(&snapshot)[0], "門口");
}

#[test]
fn keys_the_shell_does_not_mark_as_touch_get_no_neighbour_readings() {
    fn physical(_: &Weights) -> bool {
        false
    }
    let engine = engine().with_neighbor_readings(NeighborReadings {
        accepts: physical,
        ..readings()
    });
    let (_, _, snapshot) = type_keys(&engine, &snr(0.2));
    assert!(!texts(&snapshot).contains(&"門口".to_string()));
}

#[test]
fn choosing_the_neighbour_reading_commits_it() {
    let engine = engine().with_neighbor_readings(readings());
    let (mut session, mut user, snapshot) = type_keys(&engine, &snr(0.2));
    let index = texts(&snapshot).iter().position(|t| t == "門口").unwrap();
    let digit = u8::try_from(index + 1).unwrap();
    let snapshot = session.handle(&engine, &mut user, Key::Digit(digit));
    assert_eq!(snapshot.commit.as_deref(), Some("門口"));
}

#[test]
fn backspace_drops_the_neighbour_readings_of_the_deleted_key() {
    let engine = engine().with_neighbor_readings(readings());
    let mut keys = snr(0.2);
    let (mut session, mut user, _) = type_keys(&engine, &keys);
    let after_backspace = session.handle(&engine, &mut user, Key::Backspace);
    keys.pop();
    let (_, _, typed) = type_keys(&engine, &keys);
    assert_eq!(texts(&after_backspace), texts(&typed));
}

#[test]
fn the_rest_after_choosing_the_first_segment_keeps_its_neighbour_readings() {
    let engine = engine().with_neighbor_readings(readings());
    // `nr` + `sn`：第二段的 s 點得偏 a。選了第一段「同」以後，剩下的 `sn` 照樣有鄰鍵讀法「門」。
    let keys = [exact('n'), exact('r'), tap('s', 'a', 0.2), exact('n')];
    let (mut session, mut user, snapshot) = type_keys(&engine, &keys);
    let index = texts(&snapshot).iter().position(|t| t == "同").unwrap();
    let digit = u8::try_from(index + 1).unwrap();
    let snapshot = session.handle(&engine, &mut user, Key::Digit(digit));
    assert!(snapshot.commit.is_none());
    assert!(
        texts(&snapshot).contains(&"門".to_string()),
        "{:?}",
        texts(&snapshot)
    );
}

#[test]
fn a_separator_in_the_input_keeps_the_neighbour_paths_aligned() {
    let engine = engine().with_neighbor_readings(readings());
    // `sn'r`：分隔記號後面接 r。鄰鍵讀法的路徑的碼要和輸入碼一樣長，選了照樣上屏。
    let keys = [tap('s', 'a', 0.2), exact('n'), Key::Punct('\''), exact('r')];
    let (mut session, mut user, snapshot) = type_keys(&engine, &keys);
    let index = texts(&snapshot)
        .iter()
        .position(|t| t.starts_with('門'))
        .expect("有鄰鍵讀法");
    let digit = u8::try_from(index + 1).unwrap();
    let snapshot = session.handle(&engine, &mut user, Key::Digit(digit));
    assert!(snapshot.commit.is_some_and(|c| c.starts_with('門')));
}
