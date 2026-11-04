//! 兩鍵重疊的測試佈局（M1 票 19）：上層的 n 排在陣列前面、z 較大，點重疊處判定為 n。
//! 同一份檔案也給網頁的 Playwright 用（`web/e2e/touch-overlap.spec.ts`）。

use qim_layout::{Target, Warning, validate};

const OVERLAP: &str = include_str!("fixtures/overlap.json");

fn hit(x: f64, y: f64) -> String {
    let layout = validate(OVERLAP, Target::default()).layout.unwrap();
    let page = &layout.variants[0].pages[0];
    page.keys[page.hit(x, y).unwrap()].id.clone()
}

#[test]
fn overlap_layout_saves_with_an_overlap_warning() {
    let report = validate(OVERLAP, Target::default());
    assert!(report.can_save(), "{:?}", report.errors);
    assert!(report.warnings.contains(&Warning::Overlap {
        variant: 0,
        page: "letters".into(),
        keys: ["n".into(), "a".into()],
    }));
}

#[test]
fn tap_on_the_overlap_hits_the_upper_key() {
    // 重疊處是 x 150–210。
    assert_eq!(hit(180.0, 75.0), "n");
    assert_eq!(hit(151.0, 31.0 + 8.0), "n");
    // 只有下層的地方、只有上層的地方。
    assert_eq!(hit(60.0, 75.0), "a");
    assert_eq!(hit(240.0, 75.0), "n");
}

#[test]
fn rounded_corner_of_the_upper_key_falls_through_to_the_lower_key() {
    // n 的左上角 (150, 30)，圓角半徑 8：(151, 31) 在圓角外，落到下層的 a。
    assert_eq!(hit(151.0, 31.0), "a");
}
