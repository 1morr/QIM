//! 按鍵標籤與動作（plan §2.6）：有寫 `label` 就固定顯示；省略時字母鍵用方案的字根，沒有字根用字母；
//! 組字中改用 `composing` 寫的標籤與動作。

use qim_layout::{Action, KeyState, Layout};

/// 沒在組字、Shift 關著。
const IDLE: KeyState = KeyState {
    composing: false,
    shift: qim_layout::Shift::Off,
};
const COMPOSING: KeyState = KeyState {
    composing: true,
    shift: qim_layout::Shift::Off,
};

fn keys(keys: &str) -> Layout {
    Layout::from_json(&format!(
        r#"{{ "format": "qim-layout/1", "variants": [{{
              "when": {{ "device": "phone", "orientation": "portrait" }},
              "canvas": {{ "width": 40, "height": 10 }},
              "pages": [{{ "id": "letters", "keys": {keys} }}] }}] }}"#
    ))
    .unwrap()
}

/// 只有 `a` 有字根的方案。
fn radical(letter: char) -> Option<&'static str> {
    (letter == 'a').then_some("日")
}

#[test]
fn letter_key_without_label_shows_the_scheme_radical_or_the_letter() {
    let layout = keys(
        r#"[
            { "id": "a", "frame": { "x": 0, "y": 0, "w": 10, "h": 10 }, "tap": { "letter": "a" }, "hint": "a" },
            { "id": "z", "frame": { "x": 10, "y": 0, "w": 10, "h": 10 }, "tap": { "letter": "z" }, "hint": "z" }
        ]"#,
    );
    let page = &layout.variants[0].pages[0];
    assert_eq!(page.keys[0].display_label(IDLE, radical), "日");
    assert_eq!(page.keys[0].hint.as_deref(), Some("a"));
    assert_eq!(page.keys[1].display_label(IDLE, radical), "z");
}

#[test]
fn written_label_wins_and_other_keys_default_to_blank() {
    let layout = keys(
        r#"[
            { "id": "a", "frame": { "x": 0, "y": 0, "w": 10, "h": 10 }, "tap": { "letter": "a" }, "label": "A" },
            { "id": "space", "frame": { "x": 10, "y": 0, "w": 20, "h": 10 }, "tap": "space" },
            { "id": "backspace", "frame": { "x": 30, "y": 0, "w": 10, "h": 10 }, "tap": "backspace", "label": "⌫" }
        ]"#,
    );
    let page = &layout.variants[0].pages[0];
    assert_eq!(page.keys[0].display_label(IDLE, radical), "A");
    assert_eq!(page.keys[0].hint, None);
    assert_eq!(page.keys[1].display_label(IDLE, radical), "");
    assert_eq!(page.keys[2].display_label(IDLE, radical), "⌫");
}

#[test]
fn composing_overrides_label_and_tap_only_while_composing() {
    let layout = keys(
        r#"[
            { "id": "a", "frame": { "x": 0, "y": 0, "w": 10, "h": 10 }, "tap": { "letter": "a" } },
            { "id": "enter", "frame": { "x": 10, "y": 0, "w": 10, "h": 10 }, "tap": "enter", "label": "↵",
              "composing": { "label": "確定", "tap": "space" } },
            { "id": "space", "frame": { "x": 20, "y": 0, "w": 10, "h": 10 }, "tap": "space",
              "composing": { "label": "選字" } },
            { "id": "x", "frame": { "x": 30, "y": 0, "w": 10, "h": 10 }, "tap": { "letter": "x" }, "composing": null }
        ]"#,
    );
    let page = &layout.variants[0].pages[0];
    let (enter, space) = (&page.keys[1], &page.keys[2]);
    assert_eq!(enter.display_label(IDLE, radical), "↵");
    assert_eq!(enter.display_label(COMPOSING, radical), "確定");
    assert_eq!(enter.tap_action(IDLE), Action::Enter);
    assert_eq!(enter.tap_action(COMPOSING), Action::Space);
    // 只寫標籤時動作不變。
    assert_eq!(space.display_label(COMPOSING, radical), "選字");
    assert_eq!(space.tap_action(COMPOSING), Action::Space);
    assert_eq!(page.keys[3].tap_action(COMPOSING), Action::Letter('x'));
    assert_eq!(page.keys[0].display_label(COMPOSING, radical), "日");
}

#[test]
fn accessibility_label_defaults_to_the_shown_label_and_hint() {
    let layout = keys(
        r#"[
            { "id": "a", "frame": { "x": 0, "y": 0, "w": 10, "h": 10 }, "tap": { "letter": "a" }, "hint": "a" },
            { "id": "space", "frame": { "x": 10, "y": 0, "w": 10, "h": 10 }, "tap": "space" },
            { "id": "b", "frame": { "x": 20, "y": 0, "w": 10, "h": 10 }, "tap": { "letter": "b" }, "hint": "b",
              "a11yLabel": "月，英文 b" }
        ]"#,
    );
    let page = &layout.variants[0].pages[0];
    assert_eq!(page.keys[0].accessibility_label("日"), "日，a");
    assert_eq!(page.keys[1].accessibility_label(""), "空白");
    assert_eq!(page.keys[2].accessibility_label("月"), "月，英文 b");
}

#[test]
fn long_press_options_are_labelled_by_what_they_type_or_do() {
    // 殼畫長按清單時，選項的標籤用 `Action::spoken`：文字與字母是本身，其他是動作的說法。
    assert_eq!(Action::Text("q".into()).spoken(), "q");
    assert_eq!(Action::Text("1".into()).spoken(), "1");
    assert_eq!(Action::Letter('a').spoken(), "a");
    assert_eq!(Action::Space.spoken(), "空白");
}

#[test]
fn keys_that_type_show_a_magnified_preview_and_function_keys_do_not() {
    // 按住時的放大預覽（brief §4.4）：會輸入字的鍵（字母、文字）才有；組字中照 `composing` 的動作。
    let layout = keys(
        r#"[
            { "id": "a", "frame": { "x": 0, "y": 0, "w": 10, "h": 10 }, "tap": { "letter": "a" } },
            { "id": "comma", "frame": { "x": 10, "y": 0, "w": 10, "h": 10 }, "tap": { "text": "，" } },
            { "id": "space", "frame": { "x": 20, "y": 0, "w": 10, "h": 10 }, "tap": "space" },
            { "id": "sym", "frame": { "x": 30, "y": 0, "w": 10, "h": 10 }, "tap": { "text": "@" },
              "composing": { "tap": "enter" } }
        ]"#,
    );
    let page = &layout.variants[0].pages[0];
    let previews: Vec<bool> = page.keys.iter().map(|k| k.shows_preview(IDLE)).collect();
    assert_eq!(previews, [true, true, false, true]);
    assert!(!page.keys[3].shows_preview(COMPOSING));
}
