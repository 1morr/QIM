//! 數字欄開的頁（佈局版本的 `numberPage`，M4 票 26）：殼在數字欄聚焦時開它（照 Trime 在數字欄換 `number` 鍵盤、
//! fcitx5-android 換數字鍵盤），離開時回到一開始的頁。

use qim_layout::{Device, Keyboard, Layout, Orientation, When};

const PORTRAIT: When = When {
    device: Device::Phone,
    orientation: Orientation::Portrait,
};

/// 速成字母頁、英文頁與數字頁；`number_page` 是版本的 `numberPage`（`None` 是不寫）。
fn layout(number_page: Option<&str>) -> Layout {
    let number_page = number_page.map_or(String::new(), |id| format!(r#""numberPage": "{id}","#));
    Layout::from_json(&format!(
        r#"{{ "format": "qim-layout/1", "variants": [
          {{ "when": {{ "device": "phone", "orientation": "portrait" }},
            "canvas": {{ "width": 40, "height": 10 }}, {number_page}
            "pages": [
              {{ "id": "letters", "asciiPage": "english", "keys": [
                {{ "id": "a", "frame": {{ "x": 20, "y": 0, "w": 10, "h": 10 }}, "tap": {{ "letter": "a" }} }},
                {{ "id": "globe", "frame": {{ "x": 0, "y": 0, "w": 10, "h": 10 }}, "tap": "toggleAscii" }},
                {{ "id": "num", "frame": {{ "x": 10, "y": 0, "w": 10, "h": 10 }}, "tap": {{ "switchPage": "numbers" }} }}
              ] }},
              {{ "id": "english", "keys": [
                {{ "id": "a", "frame": {{ "x": 20, "y": 0, "w": 10, "h": 10 }}, "label": "a", "tap": {{ "letter": "a" }} }},
                {{ "id": "globe", "frame": {{ "x": 0, "y": 0, "w": 10, "h": 10 }}, "tap": "toggleAscii" }},
                {{ "id": "num", "frame": {{ "x": 10, "y": 0, "w": 10, "h": 10 }}, "tap": {{ "switchPage": "numbers" }} }}
              ] }},
              {{ "id": "numbers", "keys": [
                {{ "id": "back", "frame": {{ "x": 0, "y": 0, "w": 10, "h": 10 }}, "tap": {{ "switchPage": "letters" }} }}
              ] }}
            ] }}
        ] }}"#
    ))
    .unwrap()
}

fn keyboard(number_page: Option<&str>) -> Keyboard {
    Keyboard::new(layout(number_page), PORTRAIT).0
}

#[test]
fn a_number_field_opens_the_number_page_and_leaving_returns_to_the_first_page() {
    let mut keyboard = keyboard(Some("numbers"));
    assert!(keyboard.set_number_field(true));
    assert_eq!(keyboard.page().id, "numbers");
    assert!(!keyboard.set_number_field(true), "已在數字頁");
    assert!(keyboard.set_number_field(false));
    assert_eq!(keyboard.page().id, "letters");
}

/// 英文時離開數字欄回到英文的那頁（`asciiPage`）。
#[test]
fn leaving_a_number_field_in_english_returns_to_the_english_page() {
    let mut keyboard = keyboard(Some("numbers"));
    keyboard.set_ascii(true, false);
    keyboard.set_number_field(true);
    assert!(keyboard.set_number_field(false));
    assert_eq!(keyboard.page().id, "english");
}

/// 已經自己切離數字頁的不動；佈局沒寫 `numberPage` 時數字欄不換頁。
#[test]
fn only_the_number_page_itself_is_left_and_layouts_without_one_stay_put() {
    let mut left = keyboard(Some("numbers"));
    left.set_number_field(true);
    let back = left
        .page()
        .keys
        .iter()
        .position(|k| k.id == "back")
        .unwrap();
    left.tap(back, false, false, None).unwrap();
    assert!(!left.set_number_field(false));
    assert_eq!(left.page().id, "letters");

    let mut plain = keyboard(None);
    assert!(!plain.set_number_field(true));
    assert_eq!(plain.page().id, "letters");
}
