//! 佈局檔的格式（plan §2.6）：tracer 用到的最小子集（M1 票 18）。

use qim_layout::{Action, Layout};

const KEY_A: &str =
    r#"{ "id": "a", "frame": { "x": 0, "y": 0, "w": 10, "h": 10 }, "tap": { "letter": "a" } }"#;

fn with(format: &str, variants: &str) -> String {
    format!(r#"{{ "format": "{format}", "variants": {variants} }}"#)
}

fn variant(pages: &str) -> String {
    format!(
        r#"[{{ "when": {{ "device": "phone", "orientation": "portrait" }},
              "canvas": {{ "width": 10, "height": 10 }}, "pages": {pages} }}]"#
    )
}

#[test]
fn minimal_layout_round_trips_through_json() {
    let text = with(
        "qim-layout/1",
        &variant(&format!(r#"[{{ "id": "letters", "keys": [{KEY_A}] }}]"#)),
    );
    let layout = Layout::from_json(&text).unwrap();
    assert_eq!(layout.variants[0].pages[0].keys[0].tap, Action::Letter('a'));
    assert_eq!(Layout::from_json(&layout.to_json()).unwrap(), layout);
}

#[test]
fn unknown_format_is_rejected() {
    let text = with(
        "qim-layout/2",
        &variant(&format!(r#"[{{ "id": "letters", "keys": [{KEY_A}] }}]"#)),
    );
    let error = Layout::from_json(&text).unwrap_err().to_string();
    assert!(error.contains("qim-layout/2"), "{error}");
}

#[test]
fn layout_without_variants_or_pages_is_rejected() {
    assert!(Layout::from_json(&with("qim-layout/1", "[]")).is_err());
    assert!(Layout::from_json(&with("qim-layout/1", &variant("[]"))).is_err());
}

#[test]
fn space_backspace_and_enter_are_bare_action_names() {
    let keys = r#"[
        { "id": "space", "frame": { "x": 0, "y": 0, "w": 4, "h": 10 }, "tap": "space" },
        { "id": "backspace", "frame": { "x": 4, "y": 0, "w": 3, "h": 10 }, "tap": "backspace" },
        { "id": "enter", "frame": { "x": 7, "y": 0, "w": 3, "h": 10 }, "tap": "enter" }
    ]"#;
    let text = with(
        "qim-layout/1",
        &variant(&format!(r#"[{{ "id": "letters", "keys": {keys} }}]"#)),
    );
    let taps: Vec<Action> = Layout::from_json(&text).unwrap().variants[0].pages[0]
        .keys
        .iter()
        .map(|key| key.tap.clone())
        .collect();
    assert_eq!(taps, [Action::Space, Action::Backspace, Action::Enter]);
}

#[test]
fn action_names_match_the_json_tags() {
    for action in [
        Action::Letter('a'),
        Action::Space,
        Action::Backspace,
        Action::Enter,
    ] {
        let json = serde_json::to_value(&action).unwrap();
        let tag = json
            .as_str()
            .map(str::to_owned)
            .or_else(|| json.as_object().and_then(|o| o.keys().next().cloned()))
            .unwrap();
        assert_eq!(action.name(), tag);
    }
}
