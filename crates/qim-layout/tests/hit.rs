//! 點擊判定（plan〈佈局：畫布模型〉〈佈局〉）：從 JSON 讀進佈局，在設計尺寸的座標上點。

use qim_layout::{Layout, Page};

/// 一個手機直向的佈局版本、一頁；`keys` 是按鍵陣列的 JSON。
fn layout(keys: &str) -> Layout {
    Layout::from_json(&format!(
        r#"{{
            "format": "qim-layout/1",
            "variants": [{{
                "when": {{ "device": "phone", "orientation": "portrait" }},
                "canvas": {{ "width": 100, "height": 50 }},
                "pages": [{{ "id": "letters", "keys": {keys} }}]
            }}]
        }}"#
    ))
    .unwrap()
}

fn hit(layout: &Layout, x: f64, y: f64) -> Option<String> {
    let page = &layout.variants[0].pages[0];
    page.hit(x, y).map(|index| page.keys[index].id.clone())
}

#[test]
fn tap_inside_a_key_hits_it() {
    let layout = layout(
        r#"[
            { "id": "a", "frame": { "x": 0, "y": 0, "w": 50, "h": 50 }, "tap": { "letter": "a" } },
            { "id": "b", "frame": { "x": 50, "y": 0, "w": 50, "h": 50 }, "tap": { "letter": "b" } }
        ]"#,
    );
    assert_eq!(hit(&layout, 10.0, 10.0).as_deref(), Some("a"));
    assert_eq!(hit(&layout, 60.0, 10.0).as_deref(), Some("b"));
}

#[test]
fn overlapping_keys_hit_the_one_with_the_higher_z() {
    // 上層的 b 排在陣列前面，下層的 a 蓋住整張畫布。
    let upper_first = layout(
        r#"[
            { "id": "b", "frame": { "x": 40, "y": 10, "w": 20, "h": 20 }, "z": 1, "tap": { "letter": "b" } },
            { "id": "a", "frame": { "x": 0, "y": 0, "w": 100, "h": 50 }, "z": 0, "tap": { "letter": "a" } }
        ]"#,
    );
    assert_eq!(hit(&upper_first, 50.0, 20.0).as_deref(), Some("b"));
    assert_eq!(hit(&upper_first, 10.0, 20.0).as_deref(), Some("a"));

    // 陣列順序反過來，結果一樣由 z 決定。
    let reversed = layout(
        r#"[
            { "id": "a", "frame": { "x": 0, "y": 0, "w": 100, "h": 50 }, "z": 0, "tap": { "letter": "a" } },
            { "id": "b", "frame": { "x": 40, "y": 10, "w": 20, "h": 20 }, "z": 1, "tap": { "letter": "b" } }
        ]"#,
    );
    assert_eq!(hit(&reversed, 50.0, 20.0).as_deref(), Some("b"));
}

#[test]
fn same_z_hits_the_key_listed_later() {
    // 同一層時後面的畫在上面，點擊也跟著畫面。
    let layout = layout(
        r#"[
            { "id": "a", "frame": { "x": 0, "y": 0, "w": 60, "h": 50 }, "tap": { "letter": "a" } },
            { "id": "b", "frame": { "x": 40, "y": 0, "w": 60, "h": 50 }, "tap": { "letter": "b" } }
        ]"#,
    );
    assert_eq!(hit(&layout, 50.0, 20.0).as_deref(), Some("b"));
}

#[test]
fn tap_in_a_gap_hits_the_nearest_key() {
    // a 在左上、b 在右下，中間與四周都是空白。
    let layout = layout(
        r#"[
            { "id": "a", "frame": { "x": 0, "y": 0, "w": 20, "h": 20 }, "tap": { "letter": "a" } },
            { "id": "b", "frame": { "x": 70, "y": 30, "w": 20, "h": 20 }, "tap": { "letter": "b" } }
        ]"#,
    );
    // 離 a 的右邊 10、離 b 的左邊 40。
    assert_eq!(hit(&layout, 30.0, 10.0).as_deref(), Some("a"));
    // 離 b 的左邊 5（同一高度）、離 a 約 56。
    assert_eq!(hit(&layout, 65.0, 40.0).as_deref(), Some("b"));
    // 畫布外也算：右邊外面最近的是 b。
    assert_eq!(hit(&layout, 120.0, 0.0).as_deref(), Some("b"));
}

#[test]
fn page_without_keys_hits_nothing() {
    let page = Page {
        id: "empty".into(),
        keys: vec![],
    };
    assert_eq!(page.hit(10.0, 10.0), None);
}
