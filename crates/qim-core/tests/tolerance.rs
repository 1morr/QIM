//! 容錯（brief §4.8，M4 票 09）：詞庫詞與使用者詞的邊可以改輸入碼的一處（漏碼、多碼、兩碼對調、
//! 鄰鍵替換）再查，帶懲罰進 lattice；單字的邊永遠不修。票 09 只給基準與實驗用
//! （`Engine::with_tolerance`），預設關。

mod common;

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, Engine, Key, Lexicon, LexiconPack, Packs, Ranking, Scheme, SchemePack,
    Session, Snapshot, Tolerance, UserModel, Weights, Word,
};

/// 碼表：`a` 日、`b` 月、`n` 弓、`r` 口，`ab` 明、`an` 門、`ar` 晶、`nr` 同、`br` 骨；
/// 詞庫：`anr` 門口、`abr` 明口、`abanar` 明門晶。
fn engine() -> Engine {
    let radicals = BTreeMap::from([
        ('a', "日".to_string()),
        ('b', "月".to_string()),
        ('n', "弓".to_string()),
        ('r', "口".to_string()),
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
        ("b", '月'),
        ("n", '弓'),
        ("r", '口'),
        ("ab", '明'),
        ("an", '門'),
        ("ar", '晶'),
        ("nr", '同'),
        ("br", '骨'),
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
        ('月', 80),
        ('弓', 5),
        ('口', 300),
        ('明', 500),
        ('門', 200),
        ('晶', 12),
        ('同', 400),
        ('骨', 10),
    ];
    let word = |text: &str| {
        vec![Word {
            text: text.into(),
            frequency: 1000,
        }]
    };
    let words = BTreeMap::from([
        ("anr".to_string(), word("門口")),
        ("abr".to_string(), word("明口")),
        ("abanar".to_string(), word("明門晶")),
    ]);
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

/// 四種修正都開、懲罰都是 `penalty`。
fn all(penalty: f64) -> Tolerance {
    Tolerance {
        omission: Some(penalty),
        insertion: Some(penalty),
        transposition: Some(penalty),
        substitution: Some(penalty),
    }
}

fn candidates(engine: &Engine, input: &str) -> Vec<String> {
    engine.candidates(&UserModel::default(), input)
}

#[test]
fn an_extra_code_inside_a_word_is_dropped() {
    let on = engine().with_tolerance(all(1.0));
    let list = candidates(&on, "anxr");
    assert_eq!(list[0], "門口", "{list:?}");
    assert!(!candidates(&engine(), "anxr").contains(&"門口".to_string()));
}

#[test]
fn two_swapped_codes_inside_a_word_are_swapped_back() {
    let on = engine().with_tolerance(all(1.0));
    let list = candidates(&on, "arn");
    assert!(list.contains(&"門口".to_string()), "{list:?}");
    assert!(!candidates(&engine(), "arn").contains(&"門口".to_string()));
}

/// 漏打 `abanar`（明門晶）的一碼：補回任一處的碼。
#[test]
fn a_missing_code_inside_a_word_is_put_back() {
    let on = engine().with_tolerance(all(1.0));
    for typed in ["banar", "abnar", "abaar", "abana"] {
        let list = candidates(&on, typed);
        assert!(list.contains(&"明門晶".to_string()), "{typed}：{list:?}");
        assert!(!candidates(&engine(), typed).contains(&"明門晶".to_string()));
    }
}

fn texts(snapshot: &Snapshot) -> Vec<String> {
    snapshot.candidates.iter().map(|c| c.text.clone()).collect()
}

/// 經 `Session` 逐鍵打 `keys`，回傳最後的候選。
fn typed(engine: &Engine, keys: Vec<Key>) -> Vec<String> {
    let mut session = Session::default();
    let mut user = UserModel::default();
    let mut snapshot = Snapshot::default();
    for key in keys {
        snapshot = session.handle(engine, &mut user, key);
    }
    texts(&snapshot)
}

fn weighted(entries: &[(char, f32)]) -> Key {
    Key::Weighted(Weights::new(entries.to_vec()).unwrap())
}

/// `n` 的鍵按偏了打成 `m`：按鍵機率裡有 `n` 才換得回來。
#[test]
fn a_neighbouring_key_is_replaced_by_a_letter_of_its_key_weights() {
    let on = engine().with_tolerance(all(1.0));
    let slipped = weighted(&[('m', 0.7), ('n', 0.3)]);
    let list = typed(&on, vec![Key::Letter('a'), slipped, Key::Letter('r')]);
    assert!(list.contains(&"門口".to_string()), "{list:?}");
    let plain = typed(
        &on,
        vec![Key::Letter('a'), Key::Letter('m'), Key::Letter('r')],
    );
    assert!(
        !plain.contains(&"門口".to_string()),
        "沒有按鍵機率不換：{plain:?}"
    );
    let far = weighted(&[('m', 0.7), ('j', 0.3)]);
    let list = typed(&on, vec![Key::Letter('a'), far, Key::Letter('r')]);
    assert!(
        !list.contains(&"門口".to_string()),
        "只換成按鍵機率裡的字母：{list:?}"
    );
}

/// 單字的邊永遠不修（brief §4.8）：`rn` 對調是「同」（`nr`），但它是單字。
#[test]
fn a_single_character_is_never_corrected() {
    let on = engine().with_tolerance(all(0.0));
    let list = candidates(&on, "rn");
    assert!(!list.contains(&"同".to_string()), "{list:?}");
}

/// 只開的那種修正才做。
#[test]
fn only_the_enabled_corrections_apply() {
    let only_insertion = engine().with_tolerance(Tolerance {
        insertion: Some(1.0),
        ..Tolerance::default()
    });
    assert!(candidates(&only_insertion, "anxr").contains(&"門口".to_string()));
    for typed in ["arn", "abnar"] {
        let list = candidates(&only_insertion, typed);
        assert!(
            !list.iter().any(|t| t == "門口" || t == "明門晶"),
            "{typed}：{list:?}"
        );
    }
}

/// 預設相信使用者（brief §4.8）：原樣讀得通的照原樣排，修正過的帶懲罰排在後面。`abnar` 原樣讀得通
/// （明弓日口）；補回 `a` 是「明門晶」。
#[test]
fn a_literal_reading_stays_first_when_the_penalty_is_large() {
    let on = engine().with_tolerance(all(30.0));
    let list = candidates(&on, "abnar");
    assert_eq!(list[0], candidates(&engine(), "abnar")[0], "{list:?}");
    assert!(list.contains(&"明門晶".to_string()), "{list:?}");
}

/// 選了開頭一段後剩下的輸入碼照樣帶著按鍵機率：`ab` 選「明」之後，`amr` 的 `m` 換得回 `n`。
#[test]
fn key_weights_survive_selecting_the_first_segment() {
    let on = engine().with_tolerance(all(1.0));
    let mut session = Session::default();
    let mut user = UserModel::default();
    let slipped = weighted(&[('m', 0.7), ('n', 0.3)]);
    let mut shown = Snapshot::default();
    for key in [
        Key::Letter('a'),
        Key::Letter('b'),
        Key::Letter('a'),
        slipped,
        Key::Letter('r'),
    ] {
        shown = session.handle(&on, &mut user, key);
    }
    assert_eq!(texts(&shown)[0], "明門口");
    // 同每鍵延遲的「選開頭一段」：按分段數次 ← 移到第一段。
    for _ in 0..shown.composition.segments.len() {
        shown = session.handle(&on, &mut user, Key::Left);
    }
    let first = texts(&shown);
    let at = first
        .iter()
        .position(|t| t == "明")
        .expect("第一段有「明」");
    let after = session.select(&on, &mut user, at);
    assert!(
        texts(&after).contains(&"門口".to_string()),
        "{:?}",
        texts(&after)
    );
}

/// 修正只進 lattice 的路徑與蓋滿整串的字詞，不進只蓋住開頭一段的候選：那一段依蓋住的長短排、不看
/// 分數，修正過的長詞會擠到原樣的字前面（dev 上修正次數因此多了一倍以上）。
#[test]
fn corrections_stay_out_of_the_candidates_covering_only_the_beginning() {
    let on = engine().with_tolerance(all(1.0));
    let list = candidates(&on, "anxrb");
    assert!(list.contains(&"門口月".to_string()), "{list:?}");
    assert!(!list.contains(&"門口".to_string()), "{list:?}");
}

/// 蓋滿整串的修正詞最多列一個（分數最高的）：蓋滿整串的候選都排在只蓋住開頭一段的前面，修正詞一多，
/// 使用者要選的開頭一段就被擠到下一頁（dev 上沒打錯的集合修正次數因此多一成）。`amr` 的 `m` 換成 `n`
/// 或 `b` 都是詞（門口、明口）。
#[test]
fn at_most_one_corrected_word_covers_the_whole_input() {
    let on = engine().with_tolerance(all(1.0));
    let slipped = weighted(&[('m', 0.6), ('n', 0.2), ('b', 0.2)]);
    let list = typed(&on, vec![Key::Letter('a'), slipped, Key::Letter('r')]);
    let corrected = list.iter().filter(|t| *t == "門口" || *t == "明口").count();
    assert_eq!(corrected, 1, "{list:?}");
}
