//! 切中英時換頁（頁的 `asciiPage`，Trime 的 `ascii_keyboard`）與開切換鍵盤選單的動作 `keyboardMenu`
//! （M4 票 23，plan〈佈局：畫布模型〉）。

use qim_layout::{Device, Keyboard, Layout, Orientation, Output, When};

const PORTRAIT: When = When {
    device: Device::Phone,
    orientation: Orientation::Portrait,
};

/// 速成字母頁、英文頁、兩種符號頁與共用的數字頁；地球鍵點一下切中英、長按開選單。
fn layout() -> Layout {
    Layout::from_json(
        r#"{ "format": "qim-layout/1", "variants": [
          { "when": { "device": "phone", "orientation": "portrait" },
            "canvas": { "width": 40, "height": 10 },
            "pages": [
              { "id": "letters", "asciiPage": "english", "keys": [
                { "id": "a", "frame": { "x": 0, "y": 0, "w": 10, "h": 10 }, "tap": { "letter": "a" } },
                { "id": "globe", "frame": { "x": 10, "y": 0, "w": 10, "h": 10 }, "tap": "toggleAscii",
                  "longPress": ["keyboardMenu"] },
                { "id": "sym", "frame": { "x": 20, "y": 0, "w": 10, "h": 10 }, "tap": { "switchPage": "symbols" } }
              ] },
              { "id": "english", "keys": [
                { "id": "a", "frame": { "x": 0, "y": 0, "w": 10, "h": 10 }, "label": "a", "tap": { "letter": "a" } },
                { "id": "globe", "frame": { "x": 10, "y": 0, "w": 10, "h": 10 }, "tap": "toggleAscii",
                  "longPress": ["keyboardMenu"] },
                { "id": "sym", "frame": { "x": 20, "y": 0, "w": 10, "h": 10 }, "tap": { "switchPage": "english-symbols" } }
              ] },
              { "id": "symbols", "asciiPage": "english-symbols", "keys": [
                { "id": "back", "frame": { "x": 0, "y": 0, "w": 10, "h": 10 }, "tap": { "switchPage": "letters" } },
                { "id": "num", "frame": { "x": 10, "y": 0, "w": 10, "h": 10 }, "tap": { "switchPage": "numbers" } }
              ] },
              { "id": "english-symbols", "keys": [
                { "id": "abc", "frame": { "x": 0, "y": 0, "w": 10, "h": 10 }, "tap": { "switchPage": "english" } },
                { "id": "num", "frame": { "x": 10, "y": 0, "w": 10, "h": 10 }, "tap": { "switchPage": "numbers" } }
              ] },
              { "id": "numbers", "keys": [
                { "id": "back", "frame": { "x": 0, "y": 0, "w": 10, "h": 10 }, "tap": { "switchPage": "letters" } }
              ] }
            ] }
        ] }"#,
    )
    .unwrap()
}

fn keyboard() -> Keyboard {
    Keyboard::new(layout(), PORTRAIT).0
}

fn tap(keyboard: &mut Keyboard, id: &str) -> Output {
    let index = keyboard
        .page()
        .keys
        .iter()
        .position(|key| key.id == id)
        .unwrap();
    keyboard.tap(index, false, false, None).unwrap()
}

fn page(keyboard: &Keyboard) -> &str {
    &keyboard.page().id
}

#[test]
fn switching_to_english_shows_the_ascii_page_and_switching_back_returns() {
    let mut keyboard = keyboard();
    assert_eq!(tap(&mut keyboard, "globe"), Output::ToggleAscii);
    // 鍵盤照核心回來的中英狀態換頁，不自己猜（實體 Shift 切的也一樣）。
    assert_eq!(page(&keyboard), "letters");
    assert!(keyboard.set_ascii(true));
    assert_eq!(page(&keyboard), "english");
    assert!(keyboard.set_ascii(false));
    assert_eq!(page(&keyboard), "letters");
}

#[test]
fn telling_the_same_state_again_changes_nothing() {
    let mut keyboard = keyboard();
    assert!(!keyboard.set_ascii(false));
    keyboard.set_ascii(true);
    assert!(!keyboard.set_ascii(true));
    assert_eq!(page(&keyboard), "english");
}

#[test]
fn each_page_goes_to_its_own_ascii_page() {
    let mut keyboard = keyboard();
    tap(&mut keyboard, "sym");
    keyboard.set_ascii(true);
    assert_eq!(page(&keyboard), "english-symbols");
    keyboard.set_ascii(false);
    assert_eq!(page(&keyboard), "symbols");
}

#[test]
fn a_page_without_an_ascii_page_stays_when_the_mode_changes() {
    let mut keyboard = keyboard();
    tap(&mut keyboard, "sym");
    tap(&mut keyboard, "num");
    assert!(!keyboard.set_ascii(true));
    assert_eq!(page(&keyboard), "numbers");
}

#[test]
fn in_english_a_switch_to_a_page_with_an_ascii_page_lands_on_that_page() {
    let mut keyboard = keyboard();
    keyboard.set_ascii(true);
    tap(&mut keyboard, "sym");
    tap(&mut keyboard, "num");
    assert_eq!(page(&keyboard), "numbers");
    // 數字頁的「返回」寫的是速成字母頁，英文時落到英文頁。
    tap(&mut keyboard, "back");
    assert_eq!(page(&keyboard), "english");
}

#[test]
fn a_replacement_keyboard_takes_over_the_mode() {
    let mut old = keyboard();
    old.set_ascii(true);
    let mut new = keyboard();
    new.keep_state_of(&old);
    assert_eq!(page(&new), "english");
    new.set_ascii(false);
    assert_eq!(page(&new), "letters");
}

#[test]
fn a_long_press_on_the_globe_asks_the_shell_for_the_keyboard_menu() {
    let mut keyboard = keyboard();
    let globe = keyboard
        .page()
        .keys
        .iter()
        .position(|key| key.id == "globe")
        .unwrap();
    assert!(keyboard.page().keys[globe].opens_keyboard_menu());
    assert!(!keyboard.page().keys[0].opens_keyboard_menu());
    let output = keyboard.long_press(globe, 0).unwrap();
    assert_eq!(output, Output::KeyboardMenu);
    assert_eq!(output.kind(), "keyboardMenu");
    assert_eq!(page(&keyboard), "letters");
}
