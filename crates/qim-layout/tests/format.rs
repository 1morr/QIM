//! 佈局檔的格式（plan〈佈局：畫布模型〉）：讀檔、存檔與動作的寫法。

use qim_layout::{Action, Device, Layout, Orientation, Shape};

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
        { "id": "a", "frame": { "x": 0, "y": 0, "w": 1, "h": 1 }, "tap": { "letter": "a" } },
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
        .skip(1)
        .map(|key| key.tap.clone())
        .collect();
    assert_eq!(taps, [Action::Space, Action::Backspace, Action::Enter]);
}

#[test]
fn action_names_match_the_json_tags() {
    for action in [
        Action::Letter('a'),
        Action::Punct(','),
        Action::Text("，".into()),
        Action::Backspace,
        Action::Space,
        Action::Enter,
        Action::SwitchPage("symbols".into()),
        Action::NextKeyboard,
        Action::ToggleAscii,
        Action::Shift,
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

#[test]
fn device_orientation_and_shape_names_match_the_json() {
    for device in [Device::Phone, Device::Tablet] {
        assert_eq!(serde_json::to_value(device).unwrap(), device.name());
        assert_eq!(device.name().parse::<Device>(), Ok(device));
    }
    for orientation in [Orientation::Portrait, Orientation::Landscape] {
        assert_eq!(
            serde_json::to_value(orientation).unwrap(),
            orientation.name()
        );
        assert_eq!(orientation.name().parse::<Orientation>(), Ok(orientation));
    }
    assert!("watch".parse::<Device>().is_err());
    assert!("upside-down".parse::<Orientation>().is_err());
    for shape in [
        Shape::Rect { radius: 6.0 },
        Shape::Ellipse,
        Shape::Polygon {
            points: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
        },
    ] {
        assert_eq!(serde_json::to_value(&shape).unwrap()["type"], shape.name());
    }
}
