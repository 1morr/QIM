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
