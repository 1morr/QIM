//! 點擊判定的性質測試（plan §5.3）：上層優先、旋轉後內外判定正確、縮放後結果不變。
//! 多邊形的內外判定在 M6。

use proptest::prelude::*;
use qim_layout::{Action, Frame, Insets, Key, Page, Shape};

fn key(id: usize, frame: Frame, rotate: f64, z: i32, radius: f64, inset: Insets) -> Key {
    Key {
        id: id.to_string(),
        frame,
        rotate,
        z,
        shape: Shape::Rect { radius },
        extended_edges: inset,
        label: None,
        hint: None,
        a11y_label: None,
        style: None,
        tap: Action::Space,
        long_press: vec![],
        swipe: None,
        composing: None,
    }
}

prop_compose! {
    fn frame()(x in -50.0..350.0, y in -50.0..200.0, w in 1.0..120.0, h in 1.0..120.0) -> Frame {
        Frame { x, y, w, h }
    }
}

prop_compose! {
    fn insets()(top in 0.0..8.0, bottom in 0.0..8.0, left in 0.0..8.0, right in 0.0..8.0) -> Insets {
        Insets { top, bottom, left, right }
    }
}

prop_compose! {
    fn any_key(id: usize)(
        frame in frame(),
        rotate in -180.0..180.0,
        z in -3..3,
        radius in 0.0..30.0,
        inset in insets(),
    ) -> Key {
        key(id, frame, rotate, z, radius, inset)
    }
}

fn page(keys: Vec<Key>) -> Page {
    Page {
        id: "letters".into(),
        keys,
    }
}

fn keys(count: usize) -> impl Strategy<Value = Vec<Key>> {
    (0..count)
        .map(any_key)
        .collect::<Vec<_>>()
        .prop_map(|keys| keys)
}

/// 鍵 `key` 自己座標（原點在 frame 中心、旋轉前）的點換成畫布座標。
fn world(key: &Key, lx: f64, ly: f64) -> (f64, f64) {
    let (sin, cos) = key.rotate.to_radians().sin_cos();
    let (cx, cy) = (
        key.frame.x + key.frame.w / 2.0,
        key.frame.y + key.frame.h / 2.0,
    );
    (cx + lx * cos - ly * sin, cy + lx * sin + ly * cos)
}

/// 整頁等比例縮放 `s` 倍：位置、大小、圓角、`extendedEdges` 都乘上 `s`，角度不變。
fn scaled(page: &Page, s: f64) -> Page {
    let keys = page
        .keys
        .iter()
        .map(|k| {
            let f = k.frame;
            let i = k.extended_edges;
            let Shape::Rect { radius } = k.shape else {
                unreachable!("性質測試只產生矩形")
            };
            key(
                k.id.parse().unwrap(),
                Frame {
                    x: f.x * s,
                    y: f.y * s,
                    w: f.w * s,
                    h: f.h * s,
                },
                k.rotate,
                k.z,
                radius * s,
                Insets {
                    top: i.top * s,
                    bottom: i.bottom * s,
                    left: i.left * s,
                    right: i.right * s,
                },
            )
        })
        .collect();
    Page {
        id: page.id.clone(),
        keys,
    }
}

proptest! {
    /// 疊一個 z 最大的鍵上去，點在它裡面就是它，不管下面有什麼、排在陣列哪裡。
    #[test]
    fn the_key_on_top_wins(
        others in keys(6),
        top in any_key(6),
        position in 0usize..7,
        fx in -0.99f64..0.99,
        fy in -0.99f64..0.99,
    ) {
        let mut top = top;
        top.z = 10;
        let Shape::Rect { radius } = top.shape else { unreachable!() };
        let r = radius.min(top.frame.w / 2.0).min(top.frame.h / 2.0);
        // 圓角避開的十字範圍內一定在形狀裡。
        let (x, y) = world(&top, fx * (top.frame.w / 2.0 - r), fy * (top.frame.h / 2.0));
        let mut keys = others;
        keys.insert(position, top);
        let page = page(keys);
        prop_assert_eq!(page.hit(x, y), Some(position));
    }

    /// 同一層時，陣列裡後面的在上。
    #[test]
    fn on_the_same_z_the_later_key_wins(
        bottom in any_key(0),
        fx in -0.99f64..0.99,
        fy in -0.99f64..0.99,
    ) {
        let mut top = bottom.clone();
        top.id = "1".into();
        let Shape::Rect { radius } = top.shape else { unreachable!() };
        let r = radius.min(top.frame.w / 2.0).min(top.frame.h / 2.0);
        let (x, y) = world(&top, fx * (top.frame.w / 2.0 - r), fy * (top.frame.h / 2.0));
        prop_assert_eq!(page(vec![bottom, top]).hit(x, y), Some(1));
    }

    /// 旋轉後，點在不在鍵裡與旋轉前在鍵自己的座標裡一樣：|x| < w/2 且 |y| < h/2。
    #[test]
    fn rotated_key_contains_exactly_its_rotated_rectangle(
        frame in frame(),
        rotate in -720.0..720.0,
        fx in -2.0f64..2.0,
        fy in -2.0f64..2.0,
    ) {
        // 避開邊上：浮點誤差會讓正好在邊上的點落在任一邊。
        prop_assume!((fx.abs() - 1.0).abs() > 1e-6 && (fy.abs() - 1.0).abs() > 1e-6);
        let key = key(0, frame, rotate, 0, 0.0, Insets::default());
        let (x, y) = world(&key, fx * frame.w / 2.0, fy * frame.h / 2.0);
        prop_assert_eq!(key.contains(x, y), fx.abs() < 1.0 && fy.abs() < 1.0);
        // 外面的點距離大於 0，裡面的是 0。
        prop_assert_eq!(key.distance(x, y) == 0.0, fx.abs() < 1.0 && fy.abs() < 1.0);
    }

    /// 整頁放大縮小，點同一個相對位置，按到的鍵不變。
    #[test]
    fn scaling_the_page_keeps_the_hit(
        keys in keys(8),
        x in -80.0..450.0,
        y in -80.0..280.0,
        s in 0.3f64..3.0,
    ) {
        let page = page(keys);
        let hit = page.hit(x, y);
        // 離邊界或等距處太近時，縮放的浮點誤差可以合理地改變結果；只比穩定的點。
        let delta = 1e-6;
        prop_assume!([(delta, 0.0), (-delta, 0.0), (0.0, delta), (0.0, -delta)]
            .iter()
            .all(|(dx, dy)| page.hit(x + dx, y + dy) == hit));
        prop_assert_eq!(scaled(&page, s).hit(x * s, y * s), hit);
    }
}

#[test]
fn extended_edges_extend_or_shrink_the_touch_area() {
    let frame = Frame {
        x: 10.0,
        y: 10.0,
        w: 20.0,
        h: 20.0,
    };
    let grown = key(
        0,
        frame,
        0.0,
        0,
        0.0,
        Insets {
            top: 0.0,
            bottom: 6.0,
            left: 0.0,
            right: 0.0,
        },
    );
    assert!(grown.contains(20.0, 35.0));
    assert!(!grown.contains(20.0, 37.0));
    let shrunk = key(
        0,
        frame,
        0.0,
        0,
        0.0,
        Insets {
            top: -4.0,
            bottom: 0.0,
            left: 0.0,
            right: 0.0,
        },
    );
    assert!(!shrunk.contains(20.0, 12.0));
    assert!(shrunk.contains(20.0, 15.0));
}

#[test]
fn rotated_key_is_hit_where_it_is_drawn() {
    // 寬 100、高 10 的長條，以中心 (50, 50) 轉 90 度後變成直的。
    let bar = key(
        0,
        Frame {
            x: 0.0,
            y: 45.0,
            w: 100.0,
            h: 10.0,
        },
        90.0,
        0,
        0.0,
        Insets::default(),
    );
    assert!(bar.contains(50.0, 5.0));
    assert!(bar.contains(50.0, 95.0));
    assert!(!bar.contains(5.0, 50.0));
    assert!(!bar.contains(95.0, 50.0));
}
