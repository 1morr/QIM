//! 標點鍵 `{ "punct": "," }`（M1 票 27）：同實體鍵盤按那個 ASCII 標點鍵，全形、半形由核心決定；
//! 只收 ASCII 的可見標點（字母、數字、空白與全形字不行，全形字用 `text`）。

use qim_layout::{Action, KeyState, Target, validate};

fn layout_with(action: &str) -> String {
    format!(
        r#"{{ "format": "qim-layout/1", "variants": [{{
              "when": {{ "device": "phone", "orientation": "portrait" }},
              "canvas": {{ "width": 20, "height": 10 }},
              "pages": [{{ "id": "letters", "keys": [
                {{ "id": "a", "frame": {{ "x": 0, "y": 0, "w": 10, "h": 10 }}, "tap": {{ "letter": "a" }} }},
                {{ "id": "p", "frame": {{ "x": 10, "y": 0, "w": 10, "h": 10 }}, "label": "。", "tap": {action} }}
              ] }}] }}] }}"#
    )
}

#[test]
fn punct_takes_one_ascii_punctuation_character() {
    for c in [",", ".", "?", "(", "-", "@", "~", "\\\\"] {
        let report = validate(
            &layout_with(&format!(r#"{{ "punct": "{c}" }}"#)),
            Target::default(),
        );
        let (layout, _) = report.into_layout().unwrap_or_else(|e| panic!("{c}: {e}"));
        let key = &layout.variants[0].pages[0].keys[1];
        assert_eq!(key.tap.name(), "punct");
        assert!(matches!(key.tap, Action::Punct(_)), "{c}");
    }
    let (layout, _) = validate(&layout_with(r#"{ "punct": "." }"#), Target::default())
        .into_layout()
        .unwrap();
    let key = &layout.variants[0].pages[0].keys[1];
    assert_eq!(key.tap, Action::Punct('.'));
    assert_eq!(Action::Punct('.').spoken(), ".");
    // 會輸入字，按住時有放大預覽。
    assert!(key.shows_preview(KeyState::default()));
}

#[test]
fn punct_rejects_letters_digits_space_and_full_width_characters() {
    for c in ["a", "1", " ", "，", "..", ""] {
        let report = validate(
            &layout_with(&format!(r#"{{ "punct": "{c}" }}"#)),
            Target::default(),
        );
        assert!(report.into_layout().is_err(), "{c:?} 應該不符合 Schema");
    }
}
