//! `Keyboard`：殼的觸控鍵盤狀態（佈局版本、目前頁、觸控 Shift）與點按的結果，各平台共用（M2 票 03）。
//! 點一下或長按後，切頁與 Shift 已在這裡做完，殼只把 `Output` 交給核心。

use qim_layout::{Device, Keyboard, Layout, Orientation, Output, Shift, Touch, When};

const PORTRAIT: When = When {
    device: Device::Phone,
    orientation: Orientation::Portrait,
};
const LANDSCAPE: When = When {
    device: Device::Phone,
    orientation: Orientation::Landscape,
};
const TABLET_PORTRAIT: When = When {
    device: Device::Tablet,
    orientation: Orientation::Portrait,
};

/// 直向有字母頁與符號頁（一開始是字母頁）；橫向只有字母頁與另一頁 `emoji`。
fn layout() -> Layout {
    Layout::from_json(
        r#"{ "format": "qim-layout/1", "variants": [
          { "when": { "device": "phone", "orientation": "portrait" },
            "canvas": { "width": 90, "height": 10 },
            "initialPage": "letters",
            "pages": [
              { "id": "symbols", "keys": [
                { "id": "back", "frame": { "x": 0, "y": 0, "w": 10, "h": 10 }, "tap": { "switchPage": "letters" } },
                { "id": "a", "frame": { "x": 10, "y": 0, "w": 10, "h": 10 }, "tap": { "letter": "a" } }
              ] },
              { "id": "letters", "keys": [
                { "id": "a", "frame": { "x": 0, "y": 0, "w": 10, "h": 10 }, "tap": { "letter": "a" },
                  "longPress": [{ "text": "a" }, { "text": "á" }] },
                { "id": "shift", "frame": { "x": 10, "y": 0, "w": 10, "h": 10 }, "tap": "shift",
                  "composing": { "label": "'詞", "tap": { "punct": "'" } } },
                { "id": "symbols", "frame": { "x": 20, "y": 0, "w": 10, "h": 10 }, "tap": { "switchPage": "symbols" } },
                { "id": "space", "frame": { "x": 30, "y": 0, "w": 10, "h": 10 }, "tap": "space" },
                { "id": "enter", "frame": { "x": 40, "y": 0, "w": 10, "h": 10 }, "tap": "enter" },
                { "id": "backspace", "frame": { "x": 50, "y": 0, "w": 10, "h": 10 }, "tap": "backspace" },
                { "id": "lang", "frame": { "x": 60, "y": 0, "w": 10, "h": 10 }, "tap": "toggleAscii" },
                { "id": "globe", "frame": { "x": 70, "y": 0, "w": 10, "h": 10 }, "tap": "nextKeyboard" },
                { "id": "comma", "frame": { "x": 80, "y": 0, "w": 10, "h": 10 }, "tap": { "text": "，" } }
              ] }
            ] },
          { "when": { "device": "phone", "orientation": "landscape" },
            "canvas": { "width": 20, "height": 10 },
            "pages": [
              { "id": "letters", "keys": [
                { "id": "a", "frame": { "x": 0, "y": 0, "w": 10, "h": 10 }, "tap": { "letter": "a" } },
                { "id": "emoji", "frame": { "x": 10, "y": 0, "w": 10, "h": 10 }, "tap": { "switchPage": "emoji" } }
              ] },
              { "id": "emoji", "keys": [
                { "id": "back", "frame": { "x": 0, "y": 0, "w": 10, "h": 10 }, "tap": { "switchPage": "letters" } }
              ] }
            ] }
        ] }"#,
    )
    .unwrap()
}

/// 直向的鍵盤，在字母頁。
fn keyboard() -> Keyboard {
    let (keyboard, warning) = Keyboard::new(layout(), PORTRAIT);
    assert!(warning.is_none());
    keyboard
}

/// 目前頁 `id` 這個鍵的位置（殼由 `Page::hit` 得到）。
fn index(keyboard: &Keyboard, id: &str) -> usize {
    keyboard
        .page()
        .keys
        .iter()
        .position(|key| key.id == id)
        .unwrap()
}

fn tap(keyboard: &mut Keyboard, id: &str, composing: bool) -> Output {
    let index = index(keyboard, id);
    keyboard.tap(index, composing, false, None).unwrap()
}

#[test]
fn a_new_keyboard_picks_the_variant_for_the_device_and_shows_its_initial_page() {
    let keyboard = keyboard();
    assert_eq!(keyboard.variant().when, PORTRAIT);
    assert_eq!(keyboard.page().id, "letters");
    assert_eq!(keyboard.shift(), Shift::Off);

    let (landscape, _) = Keyboard::new(layout(), LANDSCAPE);
    assert_eq!(landscape.variant().when, LANDSCAPE);
    assert_eq!(landscape.page().id, "letters");
}

#[test]
fn a_device_with_no_variant_falls_back_with_a_warning() {
    let layout = Layout::from_json(
        r#"{ "format": "qim-layout/1", "variants": [
          { "when": { "device": "phone", "orientation": "landscape" },
            "canvas": { "width": 10, "height": 10 },
            "pages": [{ "id": "letters", "keys": [
              { "id": "a", "frame": { "x": 0, "y": 0, "w": 10, "h": 10 }, "tap": { "letter": "a" } }
            ] }] }
        ] }"#,
    )
    .unwrap();
    let (keyboard, warning) = Keyboard::new(layout, TABLET_PORTRAIT);
    assert!(warning.is_some());
    assert_eq!(keyboard.variant().when, LANDSCAPE);
}

#[test]
fn keys_that_send_input_become_the_matching_output() {
    let mut keyboard = keyboard();
    assert_eq!(tap(&mut keyboard, "a", false), Output::Letter('a'));
    assert_eq!(tap(&mut keyboard, "space", false), Output::Space);
    assert_eq!(tap(&mut keyboard, "enter", false), Output::Enter);
    assert_eq!(tap(&mut keyboard, "backspace", false), Output::Backspace);
    assert_eq!(
        tap(&mut keyboard, "comma", false),
        Output::Text("，".into())
    );
    // 觸控的中英鍵：交給核心後當右 Shift（先上屏組字）。
    assert_eq!(tap(&mut keyboard, "lang", false), Output::ToggleAscii);
    // 網頁沒有下一個鍵盤，核心收到的和切頁一樣；殼自己有系統的切換時另外接。
    assert_eq!(tap(&mut keyboard, "globe", false), Output::Modifier);
    assert_eq!(keyboard.page().id, "letters");
}

#[test]
fn a_switch_page_key_shows_that_page_and_sends_a_modifier() {
    let mut keyboard = keyboard();
    assert_eq!(tap(&mut keyboard, "symbols", true), Output::Modifier);
    assert_eq!(keyboard.page().id, "symbols");
    assert_eq!(tap(&mut keyboard, "back", false), Output::Modifier);
    assert_eq!(keyboard.page().id, "letters");
}

#[test]
fn shift_turns_on_once_types_one_capital_and_a_quick_second_tap_locks() {
    let mut keyboard = keyboard();
    assert_eq!(tap(&mut keyboard, "shift", false), Output::Modifier);
    assert_eq!(keyboard.shift(), Shift::Once);
    assert_eq!(tap(&mut keyboard, "a", false), Output::Text("A".into()));
    assert_eq!(keyboard.shift(), Shift::Off);

    let shift = index(&keyboard, "shift");
    keyboard.tap(shift, false, false, None).unwrap();
    keyboard.tap(shift, false, true, None).unwrap();
    assert_eq!(keyboard.shift(), Shift::Locked);
    assert_eq!(tap(&mut keyboard, "a", false), Output::Text("A".into()));
    assert_eq!(tap(&mut keyboard, "symbols", false), Output::Modifier);
    assert_eq!(keyboard.shift(), Shift::Locked, "鎖定的不被別的鍵解除");
}

#[test]
fn any_other_key_releases_a_one_shot_shift() {
    for other in ["space", "backspace", "symbols", "comma"] {
        let mut keyboard = keyboard();
        tap(&mut keyboard, "shift", false);
        tap(&mut keyboard, other, false);
        assert_eq!(keyboard.shift(), Shift::Off, "{other}");
    }
}

#[test]
fn while_composing_a_key_does_its_composing_action_instead() {
    let mut keyboard = keyboard();
    // 組字中 ⇧ 是分隔鍵，不動 Shift。
    assert_eq!(tap(&mut keyboard, "shift", true), Output::Punct('\''));
    assert_eq!(keyboard.shift(), Shift::Off);
    assert!(!keyboard.key_state(false).composing);
    assert!(keyboard.key_state(true).composing);
}

#[test]
fn a_long_press_does_the_chosen_entry_and_releases_a_one_shot_shift() {
    let mut keyboard = keyboard();
    tap(&mut keyboard, "shift", false);
    let a = index(&keyboard, "a");
    assert_eq!(keyboard.long_press(a, 1).unwrap(), Output::Text("á".into()));
    assert_eq!(keyboard.shift(), Shift::Off);
    assert_eq!(keyboard.long_press(a, 0).unwrap(), Output::Text("a".into()));
}

#[test]
fn a_key_or_choice_that_is_not_there_is_an_error_and_changes_nothing() {
    let mut keyboard = keyboard();
    tap(&mut keyboard, "shift", false);
    let error = keyboard
        .tap(99, false, false, None)
        .unwrap_err()
        .to_string();
    assert!(error.contains("99"), "{error}");
    let a = index(&keyboard, "a");
    let error = keyboard.long_press(a, 2).unwrap_err().to_string();
    assert!(error.contains('a') && error.contains('2'), "{error}");
    assert_eq!(keyboard.shift(), Shift::Once);
}

#[test]
fn turning_keeps_the_page_when_the_new_variant_has_it_and_otherwise_shows_the_initial_page() {
    let mut keyboard = keyboard();
    assert!(keyboard.select(LANDSCAPE).is_none());
    assert_eq!(keyboard.page().id, "letters");
    tap(&mut keyboard, "emoji", false);
    assert_eq!(keyboard.page().id, "emoji");
    keyboard.select(PORTRAIT);
    assert_eq!(keyboard.page().id, "letters", "直向沒有 emoji 頁");
    tap(&mut keyboard, "symbols", false);
    keyboard.select(PORTRAIT);
    assert_eq!(keyboard.page().id, "symbols");
}

#[test]
fn a_replacement_keyboard_takes_over_the_shift_and_page_of_the_old_one() {
    let mut old = keyboard();
    tap(&mut old, "symbols", false);
    let mut locked = keyboard();
    let shift = index(&locked, "shift");
    locked.tap(shift, false, false, None).unwrap();
    locked.tap(shift, false, true, None).unwrap();
    tap(&mut locked, "symbols", false);

    let mut next = keyboard();
    next.keep_state_of(&locked);
    assert_eq!(next.shift(), Shift::Locked);
    assert_eq!(next.page().id, "symbols");

    let (mut landscape, _) = Keyboard::new(layout(), LANDSCAPE);
    landscape.keep_state_of(&old);
    assert_eq!(
        landscape.page().id,
        "letters",
        "沒有同 id 的頁就回到一開始的頁"
    );
}

#[test]
fn an_output_names_its_kind_and_the_character_a_host_would_type() {
    assert_eq!(Output::Letter('a').kind(), "letter");
    assert_eq!(Output::Letter('a').character(), Some('a'));
    assert_eq!(Output::Punct(',').kind(), "punct");
    assert_eq!(Output::Punct(',').character(), Some(','));
    assert_eq!(Output::Text("，".into()).kind(), "text");
    assert_eq!(Output::Text("，".into()).character(), None);
    assert_eq!(Output::Space.kind(), "space");
    assert_eq!(Output::Enter.kind(), "enter");
    assert_eq!(Output::Backspace.kind(), "backspace");
    assert_eq!(Output::ToggleAscii.kind(), "toggleAscii");
    assert_eq!(Output::Modifier.kind(), "modifier");
    assert_eq!(Output::Modifier.character(), None);
}

/// 觸控點一下字母鍵時給了點擊位置，送出各字母的機率（`Output::Weighted`，M4 票 06）；其他鍵與 Shift
/// 開著的字母鍵照舊。
#[test]
fn a_letter_tap_with_a_touch_point_sends_letter_probabilities() {
    let touch = |x: f64| {
        Some(Touch {
            x,
            y: 5.0,
            mm_per_unit: 0.6,
        })
    };
    let (mut keyboard, _) = Keyboard::new(layout(), PORTRAIT);
    let output = keyboard.tap(0, false, false, touch(5.0)).unwrap();
    assert_eq!(output, Output::Weighted(vec![('a', 1.0)]));
    assert_eq!(output.kind(), "weighted");
    assert_eq!(output.character(), Some('a'));
    // 不是字母鍵：照舊。
    assert_eq!(
        keyboard.tap(3, false, false, touch(35.0)).unwrap(),
        Output::Space
    );
    // Shift 開著時字母鍵輸入大寫字母的文字，不算機率。
    keyboard.tap(1, false, false, touch(15.0)).unwrap();
    assert_eq!(
        keyboard.tap(0, false, false, touch(5.0)).unwrap(),
        Output::Text("A".into())
    );
}
