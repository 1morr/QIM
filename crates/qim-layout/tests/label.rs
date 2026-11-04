//! 按鍵標籤（plan §2.6）：有寫 `label` 就固定顯示；省略時字母鍵用方案的字根，沒有字根用字母。

use qim_layout::Layout;

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
    assert_eq!(page.keys[0].display_label(radical), "日");
    assert_eq!(page.keys[0].hint.as_deref(), Some("a"));
    assert_eq!(page.keys[1].display_label(radical), "z");
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
    assert_eq!(page.keys[0].display_label(radical), "A");
    assert_eq!(page.keys[0].hint, None);
    assert_eq!(page.keys[1].display_label(radical), "");
    assert_eq!(page.keys[2].display_label(radical), "⌫");
}
