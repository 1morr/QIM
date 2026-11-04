//! 預設 Gboard 速成佈局：由按鍵排列算出畫布上每個鍵的位置（plan §2.6、ADR 0004）。
//!
//! 排列以「鍵寬」為單位，一排十個鍵寬；每個鍵占一格，四周留縫，點在縫上由點擊判定取最近的鍵。
//! 現在只有手機直向、一頁字母（M1 票 18）；橫向、符號頁與長按在票 20 加。
//! 顏色照 Gboard 預設主題的淺色與深色。

use qim_layout::{
    Action, Device, FORMAT, Frame, Insets, Key, Layout, LayoutVariant, Orientation, Page, Shape,
    Size, Style, ThemeColor, When,
};

/// 寫到 `build/layouts/` 的檔名。
pub const FILE: &str = "gboard-quick.json";

/// 畫布寬度，照 plan §2.6 的例子（iPhone 直向的點數）。
const WIDTH: f64 = 390.0;
/// 一排的高度。
const ROW_HEIGHT: f64 = 54.0;
/// 一排十個鍵寬。
const UNITS_PER_ROW: f64 = 10.0;
/// 鍵與格子邊緣的距離：左右、上下。
const GAP_X: f64 = 3.0;
const GAP_Y: f64 = 5.0;
/// 鍵的圓角半徑。
const RADIUS: f64 = 6.0;

fn color(light: &str, dark: &str) -> ThemeColor {
    ThemeColor {
        light: light.into(),
        dark: dark.into(),
    }
}

/// 字母鍵與空白用淺底，刪除與 Enter 用深一階的底。
fn style(tap: &Action) -> Style {
    let fill = match tap {
        Action::Backspace | Action::Enter => color("#c4c7c5", "#5f6368"),
        _ => color("#ffffff", "#3c4043"),
    };
    Style {
        fill: Some(fill),
        label_color: Some(color("#1f1f1f", "#e3e3e3")),
        ..Style::default()
    }
}

/// 一排裡的一格：動作、鍵寬、固定標籤（字母鍵不寫，用方案的字根）。
struct Cell {
    tap: Action,
    units: f64,
    label: Option<&'static str>,
}

const fn letter(c: char) -> Cell {
    Cell {
        tap: Action::Letter(c),
        units: 1.0,
        label: None,
    }
}

/// 一排：從左邊空幾個鍵寬開始，接著依序排。
struct Row {
    indent: f64,
    cells: Vec<Cell>,
}

fn letters(indent: f64, row: &str) -> Row {
    Row {
        indent,
        cells: row.chars().map(letter).collect(),
    }
}

/// Gboard 的排列：QWERTY 三排，第二排錯開半鍵；第三排左邊留給之後的分隔鍵，右邊是刪除；
/// 最下面是空白與 Enter。
fn rows() -> Vec<Row> {
    let mut third = letters(1.5, "zxcvbnm");
    third.cells.push(Cell {
        tap: Action::Backspace,
        units: 1.5,
        label: Some("⌫"),
    });
    vec![
        letters(0.0, "qwertyuiop"),
        letters(0.5, "asdfghjkl"),
        third,
        Row {
            indent: 0.0,
            cells: vec![
                Cell {
                    tap: Action::Space,
                    units: 8.5,
                    label: None,
                },
                Cell {
                    tap: Action::Enter,
                    units: 1.5,
                    label: Some("↵"),
                },
            ],
        },
    ]
}

/// 字母鍵的 id 是字母，其他鍵是動作名。
fn id(tap: &Action) -> String {
    match tap {
        Action::Letter(c) => c.to_string(),
        other => other.name().into(),
    }
}

/// 預設的 Gboard 速成佈局。
pub fn layout() -> Layout {
    let unit = WIDTH / UNITS_PER_ROW;
    let rows = rows();
    let mut keys = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        let top = index as f64 * ROW_HEIGHT;
        let mut left = row.indent * unit;
        for cell in &row.cells {
            let width = cell.units * unit;
            let hint = match cell.tap {
                Action::Letter(c) => Some(c.to_string()),
                _ => None,
            };
            keys.push(Key {
                id: id(&cell.tap),
                frame: Frame {
                    x: left + GAP_X,
                    y: top + GAP_Y,
                    w: width - 2.0 * GAP_X,
                    h: ROW_HEIGHT - 2.0 * GAP_Y,
                },
                rotate: 0.0,
                z: 0,
                shape: Shape::Rect { radius: RADIUS },
                touch_inset: Insets::default(),
                label: cell.label.map(str::to_owned),
                hint,
                a11y_label: None,
                style: Some(style(&cell.tap)),
                tap: cell.tap.clone(),
                long_press: Vec::new(),
                swipe: None,
                composing: None,
            });
            left += width;
        }
    }
    Layout {
        format: FORMAT.into(),
        variants: vec![LayoutVariant {
            when: When {
                device: Device::Phone,
                orientation: Orientation::Portrait,
            },
            canvas: Size {
                width: WIDTH,
                height: rows.len() as f64 * ROW_HEIGHT,
            },
            grid: None,
            background: Some(color("#e8eaed", "#1f1f1f")),
            initial_page: None,
            pages: vec![Page {
                id: "letters".into(),
                keys,
            }],
        }],
    }
}
