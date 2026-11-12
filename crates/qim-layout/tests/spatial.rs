//! 觸控空間模型的性質測試（plan §2.6、§5.3，M4 票 06）：點一下算成各字母的機率。
//! 點在鍵中心時該鍵最高、離得越遠越低、整頁旋轉或縮放（實體尺寸不變）後結果一致、重疊時上層加成、
//! 機率和為 1 且只回前 `limit` 個；Gboard 樣式的格子上點中字母鍵時，機率最高的字母就是點擊判定按到的
//! 字母；離所有鍵都遠、點擊判定與 Gaussian 選的鍵不同時，點中的鍵提到平手、排第一（票 06b）。

use proptest::prelude::*;
use qim_layout::{Action, Frame, Insets, Key, KeyState, Page, Shape, SpatialModel, Touch};

fn letter_key(letter: char, frame: Frame, rotate: f64, z: i32, radius: f64) -> Key {
    key(
        &letter.to_string(),
        Action::Letter(letter),
        frame,
        rotate,
        z,
        radius,
    )
}

fn key(id: &str, tap: Action, frame: Frame, rotate: f64, z: i32, radius: f64) -> Key {
    Key {
        id: id.into(),
        frame,
        rotate,
        z,
        shape: Shape::Rect { radius },
        extended_edges: Insets::default(),
        label: None,
        hint: None,
        a11y_label: None,
        style: None,
        tap,
        long_press: vec![],
        swipe: None,
        composing: None,
    }
}

fn page(keys: Vec<Key>) -> Page {
    Page {
        id: "letters".into(),
        keys,
    }
}

fn letter(i: usize) -> char {
    char::from(b'a' + u8::try_from(i).unwrap())
}

const STATE: KeyState = KeyState {
    composing: false,
    shift: qim_layout::Shift::Off,
};

/// 手機上畫布一單位約 0.16 mm（CSS px、dp）。
const MM: f64 = 0.16;

fn weights(page: &Page, x: f64, y: f64, mm_per_unit: f64) -> Vec<(char, f32)> {
    SpatialModel::default().letter_weights(page, Touch { x, y, mm_per_unit }, STATE)
}

fn probability(weights: &[(char, f32)], c: char) -> f32 {
    weights
        .iter()
        .find(|&&(d, _)| d == c)
        .map_or(0.0, |&(_, p)| p)
}

fn centre(key: &Key) -> (f64, f64) {
    (
        key.frame.x + key.frame.w / 2.0,
        key.frame.y + key.frame.h / 2.0,
    )
}

prop_compose! {
    /// 不重疊的鍵：每個鍵在自己的 40 × 50 格子裡，大小、位置、圓角隨機。
    fn grid(count: usize)(
        cells in prop::collection::vec((10.0..38.0f64, 12.0..48.0f64, 0.0..1.0f64, 0.0..1.0f64, 0.0..6.0f64), count),
    ) -> Page {
        let keys = cells
            .into_iter()
            .enumerate()
            .map(|(i, (w, h, fx, fy, radius))| {
                let (col, row) = ((i % 10) as f64, (i / 10) as f64);
                let frame = Frame {
                    x: col * 40.0 + fx * (40.0 - w),
                    y: row * 50.0 + fy * (50.0 - h),
                    w,
                    h,
                };
                letter_key(letter(i), frame, 0.0, 0, radius)
            })
            .collect();
        page(keys)
    }
}

/// Gboard 速成直向字母頁的前三排（`qim-databuild` 的 `gboard.rs`）：一排 10 鍵、每鍵 39 寬、左右各留 2.5、
/// 上下各留 5.5；第二排內縮半鍵；第三排左 Shift、右刪除，各 1.5 鍵寬。
fn gboard_grid() -> Page {
    let rows: [&str; 3] = ["qwertyuiop", "asdfghjkl", "zxcvbnm"];
    let (unit, row_height, gap_x, gap_y) = (39.0, 54.0, 2.5, 5.5);
    let cell = |row: usize, from: f64, units: f64| Frame {
        x: from * unit + gap_x,
        y: row as f64 * row_height + gap_y,
        w: units * unit - 2.0 * gap_x,
        h: row_height - 2.0 * gap_y,
    };
    let mut keys = Vec::new();
    // 各排的縮排同 `letters_page`：第三排先放 1.5 鍵寬的 Shift，刪除接在 `m` 後面（1.5 + 7 = 8.5）。
    for (r, row) in rows.iter().enumerate() {
        let indent = [0.0, 0.5, 1.5][r];
        for (i, c) in row.chars().enumerate() {
            keys.push(letter_key(c, cell(r, indent + i as f64, 1.0), 0.0, 0, 6.0));
        }
    }
    for (id, tap, from) in [
        ("shift", Action::Shift, 0.0),
        ("backspace", Action::Backspace, 8.5),
    ] {
        keys.push(key(id, tap, cell(2, from, 1.5), 0.0, 0, 6.0));
    }
    page(keys)
}

/// 整頁以原點旋轉 `degrees` 度：每個鍵的中心轉過去、鍵本身也多轉同樣的角度。
fn rotated(page: &Page, degrees: f64) -> Page {
    let (sin, cos) = degrees.to_radians().sin_cos();
    let keys = page
        .keys
        .iter()
        .map(|k| {
            let (cx, cy) = centre(k);
            let (nx, ny) = (cx * cos - cy * sin, cx * sin + cy * cos);
            let mut key = k.clone();
            key.frame.x = nx - k.frame.w / 2.0;
            key.frame.y = ny - k.frame.h / 2.0;
            key.rotate = k.rotate + degrees;
            key
        })
        .collect();
    Page {
        id: page.id.clone(),
        keys,
    }
}

fn rotate_point((x, y): (f64, f64), degrees: f64) -> (f64, f64) {
    let (sin, cos) = degrees.to_radians().sin_cos();
    (x * cos - y * sin, x * sin + y * cos)
}

/// 整頁放大 `s` 倍：位置、大小、圓角都乘上 `s`。
fn scaled(page: &Page, s: f64) -> Page {
    let keys = page
        .keys
        .iter()
        .map(|k| {
            let mut key = k.clone();
            key.frame = Frame {
                x: k.frame.x * s,
                y: k.frame.y * s,
                w: k.frame.w * s,
                h: k.frame.h * s,
            };
            let Shape::Rect { radius } = k.shape else {
                unreachable!("性質測試只產生矩形")
            };
            key.shape = Shape::Rect { radius: radius * s };
            key
        })
        .collect();
    Page {
        id: page.id.clone(),
        keys,
    }
}

fn assert_same(a: &[(char, f32)], b: &[(char, f32)]) -> Result<(), TestCaseError> {
    prop_assert_eq!(a.len(), b.len(), "{:?} vs {:?}", a, b);
    for (&(c, p), &(d, q)) in a.iter().zip(b) {
        // 機率差不多一樣的兩個字母，浮點誤差可以合理地換順序；比各字母的機率。
        prop_assert!((p - probability(b, c)).abs() < 1e-4, "{:?} vs {:?}", a, b);
        prop_assert!((q - probability(a, d)).abs() < 1e-4, "{:?} vs {:?}", a, b);
    }
    Ok(())
}

/// 點擊判定在這點附近穩定（離邊界太近時旋轉、縮放的浮點誤差可以合理地換按到的鍵、換加成）。
fn hit_is_stable(page: &Page, x: f64, y: f64) -> bool {
    let hit = page.hit(x, y);
    let delta = 1e-6;
    [(delta, 0.0), (-delta, 0.0), (0.0, delta), (0.0, -delta)]
        .iter()
        .all(|(dx, dy)| page.hit(x + dx, y + dy) == hit)
}

proptest! {
    /// 點在哪個鍵的中心，那個鍵的字母機率最高。
    #[test]
    fn the_key_under_the_centre_is_the_most_likely(page in grid(26), i in 0usize..26) {
        let (x, y) = centre(&page.keys[i]);
        let w = weights(&page, x, y, MM);
        prop_assert_eq!(w[0].0, letter(i), "{:?}", w);
        // 嚴格大於：是 Gaussian 自己勝出，不是被「點中的鍵提到最高」拉成平手。
        prop_assert!(w.len() == 1 || w[0].1 > w[1].1, "{:?}", w);
    }

    /// 只有兩個鍵時，點從 A 的中心往 B 的中心移，A 的機率不升、B 的不降。
    #[test]
    fn moving_away_lowers_the_probability(
        page in grid(2),
        steps in 2usize..30,
    ) {
        let (a, b) = (centre(&page.keys[0]), centre(&page.keys[1]));
        let mut last = (f32::INFINITY, f32::NEG_INFINITY);
        for step in 0..=steps {
            let t = step as f64 / steps as f64;
            let (x, y) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            let w = weights(&page, x, y, MM);
            let (pa, pb) = (probability(&w, 'a'), probability(&w, 'b'));
            prop_assert!(pa <= last.0 + 1e-6 && pb >= last.1 - 1e-6, "t = {t}: {w:?}，前一點 {last:?}");
            last = (pa, pb);
        }
    }

    /// 整頁旋轉，點也跟著轉，機率不變（距離在鍵自己的座標算）。
    #[test]
    fn rotating_the_page_keeps_the_probabilities(
        page in grid(12),
        x in -20.0..420.0f64,
        y in -20.0..120.0f64,
        degrees in -180.0..180.0f64,
    ) {
        prop_assume!(hit_is_stable(&page, x, y));
        let turned = rotated(&page, degrees);
        let (rx, ry) = rotate_point((x, y), degrees);
        prop_assume!(hit_is_stable(&turned, rx, ry));
        assert_same(&weights(&page, x, y, MM), &weights(&turned, rx, ry, MM))?;
    }

    /// 整頁放大 s 倍、每單位的毫米數縮成 1/s（實體大小不變），機率不變；單位是實體尺寸。
    #[test]
    fn scaling_the_page_at_the_same_physical_size_keeps_the_probabilities(
        page in grid(12),
        x in -20.0..420.0f64,
        y in -20.0..120.0f64,
        s in 0.3..3.0f64,
    ) {
        prop_assume!(hit_is_stable(&page, x, y));
        let bigger = scaled(&page, s);
        prop_assume!(hit_is_stable(&bigger, x * s, y * s));
        assert_same(&weights(&page, x, y, MM), &weights(&bigger, x * s, y * s, MM / s))?;
    }

    /// 和為 1、由高到低、字母不重複、最多 `limit` 個。
    #[test]
    fn probabilities_sum_to_one_and_only_the_top_few_are_kept(
        page in grid(26),
        x in -20.0..420.0f64,
        y in -20.0..170.0f64,
        mm in 0.05..0.5f64,
    ) {
        let w = weights(&page, x, y, mm);
        prop_assert!(!w.is_empty() && w.len() <= SpatialModel::default().limit, "{:?}", w);
        let sum: f32 = w.iter().map(|&(_, p)| p).sum();
        prop_assert!((sum - 1.0).abs() < 1e-5, "{sum}: {w:?}");
        prop_assert!(w.windows(2).all(|pair| pair[0].1 >= pair[1].1), "{:?}", w);
        prop_assert!(w.iter().all(|&(_, p)| p > 0.0), "{:?}", w);
        let mut letters: Vec<char> = w.iter().map(|&(c, _)| c).collect();
        letters.sort_unstable();
        letters.dedup();
        prop_assert_eq!(letters.len(), w.len());
    }

    /// 鍵大小不一的頁上點哪裡都一樣：機率最高的字母就是點擊判定按到的鍵。
    #[test]
    fn the_most_likely_letter_is_always_the_hit_key(
        page in grid(26),
        x in -20.0..420.0f64,
        y in -20.0..170.0f64,
    ) {
        let hit = page.hit(x, y).unwrap();
        let Action::Letter(expected) = page.keys[hit].tap else { unreachable!() };
        let w = weights(&page, x, y, MM);
        prop_assert_eq!(w[0].0, expected, "{:?}", w);
    }

    /// Gboard 樣式的格子上點哪裡都一樣：點中字母鍵時，機率最高的字母就是點擊判定按到的鍵（容錯進核心之前，
    /// 核心只用機率最高的字母，打字行為不變）。點中 Shift、刪除時送的不是字母，不看。
    #[test]
    fn on_a_gboard_grid_the_most_likely_letter_is_the_hit_key(
        x in 0.0..390.0f64,
        y in 0.0..162.0f64,
    ) {
        let page = gboard_grid();
        let hit = page.hit(x, y).unwrap();
        let Action::Letter(expected) = page.keys[hit].tap else { return Ok(()) };
        let w = weights(&page, x, y, MM);
        prop_assert_eq!(w[0].0, expected, "{:?}", w);
        // 一樣大的鍵上不靠「提到最高」：Gaussian 加上加成本身就讓點中的鍵勝出。
        prop_assert!(w.len() == 1 || w[0].1 > w[1].1, "{:?}", w);
    }
}

/// 點在小鍵與大鍵之間的空隙、靠小鍵那側：大鍵的 σ 寬，算出來會贏過比較近的小鍵；點中的是小鍵，
/// 機率最高的就要是它（容錯進核心之前核心只用最高的，打字行為不變）。被拉上來的小鍵和大鍵一樣高。
#[test]
fn between_a_small_and_a_big_key_the_nearer_small_key_stays_on_top() {
    let small = letter_key(
        'a',
        Frame {
            x: 0.0,
            y: 0.0,
            w: 10.0,
            h: 40.0,
        },
        0.0,
        0,
        0.0,
    );
    let big = letter_key(
        'b',
        Frame {
            x: 60.0,
            y: 0.0,
            w: 80.0,
            h: 40.0,
        },
        0.0,
        0,
        0.0,
    );
    let page = page(vec![small, big]);
    // 離小鍵 24、離大鍵 26。
    let (x, y) = (34.0, 20.0);
    assert_eq!(page.hit(x, y), Some(0));
    let w = weights(&page, x, y, MM);
    assert_eq!(w[0].0, 'a', "{w:?}");
    assert!((w[0].1 - w[1].1).abs() < 1e-6, "{w:?}");
}

/// 離所有鍵都遠的空白處（票 06b 的兩個失效點：拿掉刪除鍵後 `m` 右邊、`l` 下面）：點擊判定照直線距離按到
/// 比較近的 `l`，Gaussian 的縱向 σ 比橫向窄，算出來 `m` 較高；照「點中的鍵一定最高」提到平手、排第一。
/// 一樣大的鍵上「嚴格最高」只在鍵內與鍵間的窄縫成立：真的 Gboard 直向頁這兩點是刪除鍵，但橫向頁第二排
/// 兩端內縮的空白一樣會這樣平手，是設計內的行為。
#[test]
fn far_off_the_keys_the_hit_key_is_lifted_to_a_tie_with_the_gaussian_winner() {
    // 票 06b 的兩個種子縮到的點。
    for (x, y) in [
        (376.8759717002673, 146.41283071299227),
        (366.0247247808735, 138.6298953419659),
    ] {
        let full = gboard_grid();
        assert_eq!(full.keys[full.hit(x, y).unwrap()].tap, Action::Backspace);

        let mut letters = full;
        letters.keys.retain(|k| matches!(k.tap, Action::Letter(_)));
        assert_eq!(letters.keys[letters.hit(x, y).unwrap()].id, "l");
        let w = weights(&letters, x, y, MM);
        let order: Vec<char> = w.iter().map(|&(c, _)| c).collect();
        assert_eq!(order, ['l', 'm'], "({x}, {y}): {w:?}");
        assert!((w[0].1 - w[1].1).abs() < 1e-6, "({x}, {y}): {w:?}");
    }
}

/// 重疊的鍵各自算機率，最上層（點擊判定按到的）加成；被蓋住的鍵仍有機率。
#[test]
fn overlapping_keys_both_count_and_the_top_one_is_boosted() {
    let frame = Frame {
        x: 0.0,
        y: 0.0,
        w: 40.0,
        h: 40.0,
    };
    let bottom = letter_key('a', frame, 0.0, 0, 0.0);
    let top = letter_key('b', frame, 0.0, 1, 0.0);
    let boost = SpatialModel::default().top_boost as f32;
    for keys in [
        vec![bottom.clone(), top.clone()],
        vec![top.clone(), bottom.clone()],
    ] {
        let w = weights(&page(keys), 20.0, 20.0, MM);
        assert_eq!(w[0].0, 'b', "{w:?}");
        assert!((w[0].1 - boost / (1.0 + boost)).abs() < 1e-6, "{w:?}");
        assert!((w[1].1 - 1.0 / (1.0 + boost)).abs() < 1e-6, "{w:?}");
    }
}

/// 只算字母鍵（`tap` 是 `letter`）；同一個字母有好幾個鍵時機率相加；Shift 開著時字母鍵輸入文字，不算。
#[test]
fn only_letter_keys_count_and_a_letter_on_two_keys_adds_up() {
    let at = |x: f64| Frame {
        x,
        y: 0.0,
        w: 40.0,
        h: 40.0,
    };
    let row = |third: char| {
        let mut space = letter_key('s', at(132.0), 0.0, 0, 0.0);
        space.tap = Action::Space;
        page(vec![
            letter_key('a', at(0.0), 0.0, 0, 0.0),
            letter_key('b', at(44.0), 0.0, 0, 0.0),
            letter_key(third, at(88.0), 0.0, 0, 0.0),
            space,
        ])
    };
    // 點在 `b` 的中心，兩邊的鍵一樣遠；右邊的鍵也是 `a` 時兩個鍵的機率加在一起。
    let two = weights(&row('a'), 64.0, 20.0, MM);
    let apart = weights(&row('c'), 64.0, 20.0, MM);
    assert_eq!(two.iter().map(|&(c, _)| c).collect::<Vec<_>>(), ['b', 'a']);
    let ratio = probability(&two, 'a') / probability(&two, 'b');
    let separate = (probability(&apart, 'a') + probability(&apart, 'c')) / probability(&apart, 'b');
    assert!((ratio - separate).abs() < 1e-5, "{two:?} {apart:?}");
    // 點在空白鍵上：它不是字母鍵，不算；照樣給附近字母的機率。
    let on_space = weights(&row('c'), 152.0, 20.0, MM);
    assert!(on_space.iter().all(|&(c, _)| c != 's'), "{on_space:?}");

    let shifted = KeyState {
        composing: false,
        shift: qim_layout::Shift::Once,
    };
    let touch = Touch {
        x: 20.0,
        y: 20.0,
        mm_per_unit: MM,
    };
    assert!(
        SpatialModel::default()
            .letter_weights(&row('c'), touch, shifted)
            .is_empty()
    );
}

/// 離點擊超過剪枝半徑的鍵不算；按到的鍵一定算。
#[test]
fn keys_beyond_the_pruning_radius_are_left_out() {
    let model = SpatialModel::default();
    let width = 40.0;
    let frame = |x: f64| Frame {
        x,
        y: 0.0,
        w: width,
        h: 40.0,
    };
    // `b` 的左邊離 `a` 的中心剛好超過剪枝半徑。
    let gap = model.radius_mm / MM - width / 2.0 + 1.0;
    let page = page(vec![
        letter_key('a', frame(0.0), 0.0, 0, 0.0),
        letter_key('b', frame(width + gap), 0.0, 0, 0.0),
    ]);
    assert_eq!(weights(&page, 20.0, 20.0, MM), [('a', 1.0)]);
    // 點在很遠的地方：按到最近的 `a`，照樣算它。
    assert_eq!(weights(&page, -1000.0, 20.0, MM), [('a', 1.0)]);
}
