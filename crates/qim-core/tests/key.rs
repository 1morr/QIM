//! 殼用「種類 + 值」送按鍵（`Key::parse`）：綁定只轉型別，加一種鍵只改核心與殼的按鍵對照（M2 票 03）。

use qim_core::{Key, ShiftKey, ShiftSwitch};

#[test]
fn keys_with_a_value_parse_from_kind_and_value() {
    assert_eq!(Key::parse("letter", Some("a")), Ok(Key::Letter('a')));
    assert_eq!(Key::parse("letter", Some("A")), Ok(Key::Letter('A')));
    assert_eq!(Key::parse("punct", Some("'")), Ok(Key::Punct('\'')));
    assert_eq!(Key::parse("digit", Some("0")), Ok(Key::Digit(0)));
    assert_eq!(Key::parse("digit", Some("9")), Ok(Key::Digit(9)));
    assert_eq!(
        Key::parse("toggleAscii", Some("left")),
        Ok(Key::ToggleAscii(ShiftKey::Left))
    );
    assert_eq!(
        Key::parse("toggleAscii", Some("right")),
        Ok(Key::ToggleAscii(ShiftKey::Right))
    );
}

#[test]
fn keys_without_a_value_parse_from_kind_alone() {
    let named = [
        ("space", Key::Space),
        ("enter", Key::Enter),
        ("backspace", Key::Backspace),
        ("escape", Key::Escape),
        ("pageUp", Key::PageUp),
        ("pageDown", Key::PageDown),
        ("left", Key::Left),
        ("right", Key::Right),
        ("other", Key::Other),
        ("modifier", Key::Modifier),
    ];
    for (kind, key) in named {
        assert_eq!(Key::parse(kind, None), Ok(key), "{kind}");
    }
}

#[test]
fn a_wrong_kind_or_value_is_an_error_that_names_it() {
    let wrong = [
        ("tab", None),
        ("letter", None),
        ("letter", Some("ab")),
        ("letter", Some("")),
        ("punct", Some(",,")),
        ("digit", Some("10")),
        ("digit", Some("a")),
        ("toggleAscii", Some("middle")),
        ("toggleAscii", None),
        ("space", Some(" ")),
    ];
    for (kind, value) in wrong {
        let error = Key::parse(kind, value).unwrap_err().to_string();
        assert!(error.contains(kind), "{kind} {value:?}: {error}");
    }
}

#[test]
fn shift_switch_parses_from_the_names_the_shell_menu_uses() {
    assert_eq!("rime".parse(), Ok(ShiftSwitch::Rime));
    assert_eq!("commit-text".parse(), Ok(ShiftSwitch::CommitText));
    assert_eq!("inline-ascii".parse(), Ok(ShiftSwitch::InlineAscii));
    assert!("commit".parse::<ShiftSwitch>().is_err());
}

/// 觸控一下的各字母機率（M4 票 06）：值是「字母:機率」以逗號分隔，核心依機率由高到低排、正規化成和為 1。
#[test]
fn weighted_parses_letter_probabilities_sorted_and_normalised() {
    let key = Key::parse("weighted", Some("s:0.3,a:0.6,d:0.1")).unwrap();
    let Key::Weighted(weights) = &key else {
        panic!("{key:?}");
    };
    assert_eq!(weights.top(), 'a');
    let letters: Vec<char> = weights.entries().iter().map(|&(c, _)| c).collect();
    assert_eq!(letters, ['a', 's', 'd']);
    let sum: f32 = weights.entries().iter().map(|&(_, p)| p).sum();
    assert!((sum - 1.0).abs() < 1e-6, "{sum}");

    // 和不是 1 時照比例正規化（殼給的數字可能四捨五入過）。
    let Key::Weighted(rounded) = Key::parse("weighted", Some("a:2,s:2")).unwrap() else {
        unreachable!()
    };
    assert_eq!(rounded.entries(), &[('a', 0.5), ('s', 0.5)]);
    // 一樣高時照給的順序。
    assert_eq!(rounded.top(), 'a');
}

/// 實體鍵盤的按鍵機率（M4 票 07、09c）：值同 `weighted`，種類分開，核心才分得出觸控（只有觸控列鄰鍵讀法）。
#[test]
fn physical_parses_like_weighted_but_stays_a_physical_key() {
    let key = Key::parse("physical", Some("s:0.3,a:0.6")).unwrap();
    let Key::Physical(weights) = &key else {
        panic!("{key:?}");
    };
    assert_eq!(weights.top(), 'a');
    assert!(Key::parse("physical", None).is_err());
}

#[test]
fn weighted_with_a_bad_format_is_an_error() {
    let wrong = [
        None,
        Some(""),
        Some("a"),
        Some("a:"),
        Some(":0.5"),
        Some("ab:0.5"),
        Some("a:x"),
        Some("a:0.5,a:0.5"),
        Some("a:0"),
        Some("a:-0.5,s:1"),
        Some("a:NaN"),
        Some("a:inf"),
        Some("1:0.5"),
        Some("a:0.5,"),
    ];
    for value in wrong {
        let error = Key::parse("weighted", value).unwrap_err().to_string();
        assert!(error.contains("weighted"), "{value:?}: {error}");
    }
}
