//! 佈局檔的驗證（plan §2.6、§5.3）：JSON Schema 與兩級驗證。
//! 錯誤（不能存檔）每一種都造出來會紅；只有警告時可以存檔。

use qim_layout::{Layout, Target, validate};

const FULL: &str = include_str!("fixtures/full.json");

#[test]
fn every_field_of_the_format_is_read_and_saved_back() {
    let report = validate(FULL, Target::default());
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert!(report.can_save());
    let layout = report.layout.unwrap();
    assert_eq!(Layout::from_json(&layout.to_json()).unwrap(), layout);
}

use qim_layout::{Device, LayoutError, Orientation, Warning};
use serde_json::{Value, json};

/// 一個鍵，`tap` 照寫。
fn key(id: &str, x: f64, tap: Value) -> Value {
    json!({ "id": id, "frame": { "x": x, "y": 0, "w": 10, "h": 10 }, "tap": tap })
}

/// 字母頁：`a`–`z` 一排，每鍵寬 10；後面接 `extra`。
fn letters(extra: Vec<Value>) -> Value {
    let mut keys: Vec<Value> = ('a'..='z')
        .enumerate()
        .map(|(i, c)| {
            key(
                &c.to_string(),
                i as f64 * 10.0,
                json!({ "letter": c.to_string() }),
            )
        })
        .collect();
    keys.extend(extra);
    json!({ "id": "letters", "keys": keys })
}

/// 直向與橫向各一個版本，頁面都是 `pages`；畫布 400 × 10，放得下 `letters` 加 14 個鍵。
fn layout(pages: Vec<Value>) -> Value {
    let variant = |orientation: &str| {
        json!({
            "when": { "device": "phone", "orientation": orientation },
            "canvas": { "width": 400, "height": 10 },
            "pages": pages,
        })
    };
    json!({ "format": "qim-layout/1", "variants": [variant("portrait"), variant("landscape")] })
}

fn check(value: &Value) -> qim_layout::Report {
    validate(&value.to_string(), Target::default())
}

fn assert_broken(value: &Value, expected: &str) {
    let report = check(value);
    assert!(!report.can_save(), "應該不能存檔：{value}");
    let messages: Vec<String> = report.errors.iter().map(ToString::to_string).collect();
    assert!(
        report
            .errors
            .iter()
            .all(|e| matches!(e, LayoutError::Broken(_))),
        "{messages:?}"
    );
    assert!(
        messages.iter().any(|m| m.contains(expected)),
        "錯誤裡沒有 {expected:?}：{messages:?}"
    );
}

#[test]
fn clean_layout_has_no_errors_or_warnings() {
    let report = check(&layout(vec![letters(vec![])]));
    assert_eq!(report.errors, []);
    assert_eq!(report.warnings, []);
}

#[test]
fn unrelated_formatting_and_field_order_change_nothing() {
    let value = layout(vec![letters(vec![])]);
    let pretty = serde_json::to_string_pretty(&value).unwrap();
    // serde_json 的物件依字母序；把每個 frame 的 h、w 對調。
    let compact = value.to_string();
    let reordered = compact.replace(r#""frame":{"h":10,"w":10,"#, r#""frame":{"w":10,"h":10,"#);
    assert_ne!(reordered, compact, "替換沒找到 frame");
    for text in [pretty, reordered] {
        let report = validate(&text, Target::default());
        assert_eq!((report.errors, report.warnings), (vec![], vec![]));
    }
}

#[test]
fn file_that_is_not_json_is_broken() {
    let report = validate("{ \"format\": ", Target::default());
    assert!(!report.can_save());
    assert!(report.errors[0].to_string().contains("不是 JSON"));
}

/// 改一份佈局的方法。
type Mutation = Box<dyn Fn(&mut Value)>;

#[test]
fn file_that_breaks_the_schema_is_broken() {
    let base = layout(vec![letters(vec![])]);
    let cases: Vec<(&str, Mutation)> = vec![
        ("/format", Box::new(|v| v["format"] = json!(2))),
        ("/variants", Box::new(|v| v["variants"] = json!([]))),
        (
            "/variants/0/pages",
            Box::new(|v| v["variants"][0]["pages"] = json!([])),
        ),
        (
            "/variants/0",
            Box::new(|v| v["variants"][0]["rows"] = json!(4)),
        ),
        (
            "/variants/0/when/device",
            Box::new(|v| v["variants"][0]["when"]["device"] = json!("watch")),
        ),
        (
            "/variants/0/pages/0/keys/0",
            Box::new(|v| {
                v["variants"][0]["pages"][0]["keys"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("tap");
            }),
        ),
        (
            "/variants/0/pages/0/keys/0/tap",
            Box::new(|v| v["variants"][0]["pages"][0]["keys"][0]["tap"] = json!({ "letter": "A" })),
        ),
        (
            "/variants/0/pages/0/keys/0/tap",
            Box::new(|v| v["variants"][0]["pages"][0]["keys"][0]["tap"] = json!("jump")),
        ),
        (
            "/variants/0/pages/0/keys/0/frame/w",
            Box::new(|v| v["variants"][0]["pages"][0]["keys"][0]["frame"]["w"] = json!(0)),
        ),
        (
            "/variants/0/pages/0/keys/0/shape",
            Box::new(|v| {
                v["variants"][0]["pages"][0]["keys"][0]["shape"] = json!({ "type": "star" })
            }),
        ),
        (
            "/variants/0/pages/0/keys/0/style/fill/dark",
            Box::new(|v| {
                v["variants"][0]["pages"][0]["keys"][0]["style"] =
                    json!({ "fill": { "light": "#ffffff", "dark": "black" } })
            }),
        ),
        (
            "/variants/0/pages/0/keys/0/composing",
            Box::new(|v| v["variants"][0]["pages"][0]["keys"][0]["composing"] = json!("enter")),
        ),
    ];
    for (path, mutate) in cases {
        let mut value = base.clone();
        mutate(&mut value);
        assert_broken(&value, &format!("{path} 不符合 Schema"));
    }
}

#[test]
fn duplicate_ids_and_dangling_page_references_are_broken() {
    let duplicate_key = layout(vec![letters(vec![key("a", 260.0, json!("space"))])]);
    assert_broken(&duplicate_key, "鍵 id a 重複");

    let duplicate_page = layout(vec![letters(vec![]), letters(vec![])]);
    assert_broken(&duplicate_page, "頁 id letters 重複");

    let dangling = layout(vec![letters(vec![key(
        "sym",
        260.0,
        json!({ "switchPage": "symbols" }),
    )])]);
    assert_broken(&dangling, "switchPage 指到不存在的頁 symbols");

    let mut dangling_long_press = layout(vec![letters(vec![])]);
    dangling_long_press["variants"][0]["pages"][0]["keys"][0]["longPress"] =
        json!([{ "switchPage": "emoji" }]);
    assert_broken(&dangling_long_press, "switchPage 指到不存在的頁 emoji");

    let mut initial = layout(vec![letters(vec![])]);
    initial["variants"][1]["initialPage"] = json!("symbols");
    assert_broken(&initial, "initialPage 指到不存在的頁 symbols");
}

#[test]
fn page_that_cannot_switch_back_to_letters_is_an_error() {
    let to_symbols = key("sym", 260.0, json!({ "switchPage": "symbols" }));
    let trap = json!({ "id": "symbols", "keys": [key("comma", 0.0, json!({ "text": "，" }))] });
    let report = check(&layout(vec![letters(vec![to_symbols.clone()]), trap]));
    assert!(!report.can_save());
    assert_eq!(
        report.errors,
        [0, 1].map(|variant| LayoutError::NoWayBackToLetters {
            variant,
            page: "symbols".into()
        })
    );

    // 經過另一頁才回得來也可以。
    let symbols =
        json!({ "id": "symbols", "keys": [key("more", 0.0, json!({ "switchPage": "more" }))] });
    let more =
        json!({ "id": "more", "keys": [key("abc", 0.0, json!({ "switchPage": "letters" }))] });
    let report = check(&layout(vec![letters(vec![to_symbols]), symbols, more]));
    assert_eq!(report.errors, []);

    // 一個字母鍵都沒有。
    let no_letters = json!({ "id": "letters", "keys": [key("space", 0.0, json!("space"))] });
    let report = check(&layout(vec![no_letters]));
    assert!(matches!(
        report.errors[..],
        [
            LayoutError::NoWayBackToLetters { .. },
            LayoutError::NoWayBackToLetters { .. }
        ]
    ));
}

#[test]
fn ios_needs_a_next_keyboard_key_reachable_from_every_page() {
    let ios = Target {
        needs_input_mode_switch_key: true,
    };
    let to_symbols = key("sym", 260.0, json!({ "switchPage": "symbols" }));
    let symbols =
        json!({ "id": "symbols", "keys": [key("abc", 0.0, json!({ "switchPage": "letters" }))] });
    let without = layout(vec![letters(vec![to_symbols.clone()]), symbols.clone()]);
    // 其他平台不需要。
    assert_eq!(check(&without).errors, []);
    let report = validate(&without.to_string(), ios);
    assert!(!report.can_save());
    assert!(report.errors.contains(&LayoutError::MissingNextKeyboard {
        variant: 0,
        page: "symbols".into()
    }));

    // 地球鍵只放在字母頁，符號頁切得回去就夠。
    let globe = key("globe", 270.0, json!("nextKeyboard"));
    let with = layout(vec![letters(vec![to_symbols, globe]), symbols]);
    assert_eq!(validate(&with.to_string(), ios).errors, []);
}

#[test]
fn warnings_alone_still_save() {
    let extra = vec![
        // 完全蓋住 a、蓋住半個 b。
        json!({ "id": "big", "frame": { "x": 0, "y": 0, "w": 15, "h": 10 }, "z": 1, "tap": { "text": "!" } }),
        // 超出畫布右邊。
        key("edge", 395.0, json!("space")),
        json!({ "id": "oval", "frame": { "x": 260, "y": 0, "w": 10, "h": 10 },
                "shape": { "type": "ellipse" }, "tap": "enter" }),
        json!({ "id": "flick", "frame": { "x": 270, "y": 0, "w": 10, "h": 10 },
                "swipe": { "up": { "text": "1" } }, "tap": "backspace" }),
    ];
    let mut value = layout(vec![letters(extra)]);
    // 拿掉 q，缺字母；拿掉橫向，缺方向。
    value["variants"][0]["pages"][0]["keys"]
        .as_array_mut()
        .unwrap()
        .remove(16);
    value["variants"].as_array_mut().unwrap().truncate(1);

    let report = check(&value);
    assert!(report.can_save(), "{:?}", report.errors);
    let has = |wanted: &Warning| report.warnings.contains(wanted);
    let page = "letters".to_owned();
    assert!(has(&Warning::Overlap {
        variant: 0,
        page: page.clone(),
        keys: ["a".into(), "big".into()]
    }));
    assert!(has(&Warning::Overlap {
        variant: 0,
        page: page.clone(),
        keys: ["b".into(), "big".into()]
    }));
    assert!(has(&Warning::Covered {
        variant: 0,
        page: page.clone(),
        key: "a".into()
    }));
    assert!(has(&Warning::OutOfCanvas {
        variant: 0,
        page: page.clone(),
        key: "edge".into()
    }));
    assert!(has(&Warning::UnsupportedShape {
        variant: 0,
        page: page.clone(),
        key: "oval".into(),
        shape: "ellipse"
    }));
    assert!(has(&Warning::UnsupportedSwipe {
        variant: 0,
        page: page.clone(),
        key: "flick".into()
    }));
    assert!(has(&Warning::MissingLetters {
        variant: 0,
        letters: "q".into()
    }));
    assert!(has(&Warning::MissingOrientation {
        device: Device::Phone,
        orientation: Orientation::Landscape
    }));
    // 只碰到邊的相鄰鍵不算重疊；被部分蓋住的 b 不算完全蓋住。
    assert!(!has(&Warning::Overlap {
        variant: 0,
        page: page.clone(),
        keys: ["b".into(), "c".into()]
    }));
    assert!(!has(&Warning::Covered {
        variant: 0,
        page,
        key: "b".into()
    }));
}

#[test]
fn strict_check_wants_colors_everywhere_and_no_warnings() {
    use qim_layout::{StrictIssue, check_builtin};
    let colored = |mut key: Value| {
        key["style"] = json!({
            "fill": { "light": "#ffffff", "dark": "#3c4043" },
            "labelColor": { "light": "#000000", "dark": "#ffffff" },
        });
        key
    };
    let mut value = layout(vec![letters(vec![])]);
    for variant in value["variants"].as_array_mut().unwrap() {
        variant["background"] = json!({ "light": "#e8eaed", "dark": "#202124" });
        for key in variant["pages"][0]["keys"].as_array_mut().unwrap() {
            *key = colored(key.take());
        }
    }
    assert_eq!(check_builtin(&value.to_string()), []);

    let mut no_background = value.clone();
    no_background["variants"][1]
        .as_object_mut()
        .unwrap()
        .remove("background");
    assert_eq!(
        check_builtin(&no_background.to_string()),
        [StrictIssue::MissingBackground { variant: 1 }]
    );

    let mut no_dark_label = value.clone();
    no_dark_label["variants"][0]["pages"][0]["keys"][3]["style"]
        .as_object_mut()
        .unwrap()
        .remove("labelColor");
    assert_eq!(
        check_builtin(&no_dark_label.to_string()),
        [StrictIssue::MissingKeyColors {
            variant: 0,
            page: "letters".into(),
            key: "d".into()
        }]
    );

    // 一般佈局只是警告的，內建佈局都不行。
    let mut overlapping = value.clone();
    overlapping["variants"][0]["pages"][0]["keys"][1]["frame"]["x"] = json!(5);
    let issues = check_builtin(&overlapping.to_string());
    assert!(
        issues.contains(&StrictIssue::Warning(Warning::Overlap {
            variant: 0,
            page: "letters".into(),
            keys: ["a".into(), "b".into()]
        })),
        "{issues:?}"
    );
}

#[test]
fn variant_selection_prefers_device_and_orientation_then_orientation_then_the_first() {
    let variant = |device: &str, orientation: &str| {
        json!({
            "when": { "device": device, "orientation": orientation },
            "canvas": { "width": 400, "height": 10 },
            "pages": [letters(vec![])],
        })
    };
    let value = json!({ "format": "qim-layout/1", "variants": [
        variant("tablet", "portrait"),
        variant("phone", "landscape"),
        variant("phone", "portrait"),
    ] });
    let layout = Layout::from_json(&value.to_string()).unwrap();
    assert_eq!(
        layout.select(Device::Phone, Orientation::Portrait),
        (2, None)
    );
    assert_eq!(
        layout.select(Device::Tablet, Orientation::Portrait),
        (0, None)
    );
    // 沒有平板橫向：方向符合的手機橫向。
    assert_eq!(
        layout.select(Device::Tablet, Orientation::Landscape),
        (1, None)
    );

    let only_tablet =
        json!({ "format": "qim-layout/1", "variants": [variant("tablet", "portrait")] });
    let layout = Layout::from_json(&only_tablet.to_string()).unwrap();
    assert_eq!(
        layout.select(Device::Phone, Orientation::Landscape),
        (
            0,
            Some(Warning::NoMatchingVariant {
                device: Device::Phone,
                orientation: Orientation::Landscape
            })
        )
    );
}
