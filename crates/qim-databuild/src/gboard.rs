//! 預設 Gboard 速成佈局：由按鍵排列算出畫布上每個鍵的位置（plan §2.6、ADR 0004）。
//!
//! 排列以「鍵寬」為單位，一排十個鍵寬；每個鍵占一格，四周留縫，點在縫上由點擊判定取最近的鍵。
//! 手機直向與橫向兩個佈局版本用同一份排列，只有畫布、排高與縫不同。
//! 三頁：字母、符號（打完留在符號頁，brief §4.4）、常用（自訂頁面的示範：表情與常用語）。
//! 顏色照 Gboard 預設主題的淺色與深色。

use qim_layout::{
    Action, Device, FORMAT, Frame, Insets, Key, Layout, LayoutVariant, Orientation, Page, Shape,
    Size, Style, ThemeColor, When,
};

/// 寫到 `build/layouts/` 的檔名。
pub const FILE: &str = "gboard-quick.json";

/// 頁的 id。
pub const LETTERS: &str = "letters";
pub const SYMBOLS: &str = "symbols";
pub const PHRASES: &str = "phrases";

/// 一排十個鍵寬。
const UNITS_PER_ROW: f64 = 10.0;

/// 一個佈局版本的尺寸：畫布寬、排高、鍵與格子邊緣的距離（左右、上下）、圓角。
struct Geometry {
    orientation: Orientation,
    width: f64,
    row_height: f64,
    gap_x: f64,
    gap_y: f64,
    radius: f64,
}

/// 直向：iPhone 直向的點數寬（plan §2.6 的例子）。
const PORTRAIT: Geometry = Geometry {
    orientation: Orientation::Portrait,
    width: 390.0,
    row_height: 54.0,
    gap_x: 3.0,
    gap_y: 5.0,
    radius: 6.0,
};

/// 橫向：iPhone 橫向的點數寬；鍵變寬、排變矮，鍵盤不占掉太多高度。
const LANDSCAPE: Geometry = Geometry {
    orientation: Orientation::Landscape,
    width: 844.0,
    row_height: 40.0,
    gap_x: 4.0,
    gap_y: 4.0,
    radius: 6.0,
};

fn color(light: &str, dark: &str) -> ThemeColor {
    ThemeColor {
        light: light.into(),
        dark: dark.into(),
    }
}

/// 輸入字的鍵與空白用淺底，刪除、Enter、切頁用深一階的底。
fn style(tap: &Action) -> Style {
    let fill = match tap {
        Action::Backspace | Action::Enter | Action::SwitchPage(_) => color("#c4c7c5", "#5f6368"),
        _ => color("#ffffff", "#3c4043"),
    };
    Style {
        fill: Some(fill),
        label_color: Some(color("#1f1f1f", "#e3e3e3")),
        ..Style::default()
    }
}

/// 一排裡的一格。
struct Cell {
    id: String,
    tap: Action,
    units: f64,
    /// 固定標籤；字母鍵不寫，用方案的字根。
    label: Option<String>,
    hint: Option<String>,
    long_press: Vec<Action>,
}

impl Cell {
    fn new(id: &str, tap: Action, units: f64, label: Option<&str>) -> Self {
        Self {
            id: id.into(),
            tap,
            units,
            label: label.map(str::to_owned),
            hint: None,
            long_press: Vec::new(),
        }
    }

    fn units(mut self, units: f64) -> Self {
        self.units = units;
        self
    }
}

/// 字母鍵：角標是字母，長按出英文字母（brief §4.4）；`digit` 是第一排長按另含的數字。
fn letter(c: char, digit: Option<char>) -> Cell {
    let mut cell = Cell::new(&c.to_string(), Action::Letter(c), 1.0, None);
    cell.hint = Some(c.to_string());
    cell.long_press = std::iter::once(c)
        .chain(digit)
        .map(|c| Action::Text(c.to_string()))
        .collect();
    cell
}

/// 直接輸入 `text` 的鍵，id 與標籤都是文字本身。
fn text(text: &str) -> Cell {
    Cell::new(text, Action::Text(text.into()), 1.0, Some(text))
}

fn texts(row: &str) -> Vec<Cell> {
    row.split(' ').map(text).collect()
}

fn switch(page: &str, label: &str) -> Cell {
    Cell::new(
        &format!("to-{page}"),
        Action::SwitchPage(page.into()),
        1.5,
        Some(label),
    )
}

fn backspace() -> Cell {
    Cell::new("backspace", Action::Backspace, 1.5, Some("⌫"))
}

/// 一排：從左邊空幾個鍵寬開始，接著依序排。
struct Row {
    indent: f64,
    cells: Vec<Cell>,
}

/// 每頁最下面一排：切頁鍵、逗號、空白、句號、Enter。
fn bottom_row(switch_to: Cell) -> Row {
    Row {
        indent: 0.0,
        cells: vec![
            switch_to,
            text("，"),
            Cell::new("space", Action::Space, 5.0, None),
            text("。"),
            Cell::new("enter", Action::Enter, 1.5, Some("↵")),
        ],
    }
}

/// 字母頁照 Gboard：QWERTY 三排，第二排錯開半鍵；第三排左邊留給之後的分隔鍵，右邊是刪除。
fn letters_page() -> Vec<Row> {
    let first = "qwertyuiop"
        .chars()
        .zip("1234567890".chars())
        .map(|(c, digit)| letter(c, Some(digit)))
        .collect();
    let row = |row: &str| row.chars().map(|c| letter(c, None)).collect::<Vec<_>>();
    let mut third = row("zxcvbnm");
    third.push(backspace());
    vec![
        Row {
            indent: 0.0,
            cells: first,
        },
        Row {
            indent: 0.5,
            cells: row("asdfghjkl"),
        },
        Row {
            indent: 1.5,
            cells: third,
        },
        bottom_row(switch(SYMBOLS, "?123")),
    ]
}

/// 符號頁：數字、常用符號、中文標點；左下「ABC」回字母頁，左邊「常用」切到自訂頁面。
fn symbols_page() -> Vec<Row> {
    let mut third = vec![switch(PHRASES, "常用")];
    third.extend(texts("、 「 」 ： ； ？ ！"));
    third.push(backspace());
    vec![
        Row {
            indent: 0.0,
            cells: texts("1 2 3 4 5 6 7 8 9 0"),
        },
        Row {
            indent: 0.0,
            cells: texts("@ # $ % & - + （ ） /"),
        },
        Row {
            indent: 0.0,
            cells: third,
        },
        bottom_row(switch(LETTERS, "ABC")),
    ]
}

/// 自訂頁面的示範（brief §4.4「一個自訂頁面」）：表情與粵語常用語，示範 `switchPage` 與多字的 `text`。
fn phrases_page() -> Vec<Row> {
    let mut third = vec![switch(SYMBOLS, "?123")];
    third.extend(["哈哈", "收到"].map(|t| text(t).units(3.5)));
    third.push(backspace());
    vec![
        Row {
            indent: 0.0,
            cells: texts("😀 😂 😍 🥲 👍 🙏 👏 ❤️ 🎉 🔥"),
        },
        Row {
            indent: 0.0,
            cells: ["唔該", "多謝", "早晨", "好呀", "冇問題"]
                .map(|t| text(t).units(2.0))
                .into(),
        },
        Row {
            indent: 0.0,
            cells: third,
        },
        bottom_row(switch(LETTERS, "ABC")),
    ]
}

/// 依 `geometry` 把排列算成一頁按鍵。
fn page(id: &str, rows: Vec<Row>, geometry: &Geometry) -> Page {
    let unit = geometry.width / UNITS_PER_ROW;
    let mut keys = Vec::new();
    for (index, row) in rows.into_iter().enumerate() {
        let top = index as f64 * geometry.row_height;
        let mut left = row.indent * unit;
        for cell in row.cells {
            let width = cell.units * unit;
            keys.push(Key {
                id: cell.id,
                frame: Frame {
                    x: left + geometry.gap_x,
                    y: top + geometry.gap_y,
                    w: width - 2.0 * geometry.gap_x,
                    h: geometry.row_height - 2.0 * geometry.gap_y,
                },
                rotate: 0.0,
                z: 0,
                shape: Shape::Rect {
                    radius: geometry.radius,
                },
                extended_edges: Insets::default(),
                label: cell.label,
                hint: cell.hint,
                a11y_label: None,
                style: Some(style(&cell.tap)),
                tap: cell.tap,
                long_press: cell.long_press,
                swipe: None,
                composing: None,
            });
            left += width;
        }
    }
    Page {
        id: id.into(),
        keys,
    }
}

/// 每頁都是四排。
const ROWS: f64 = 4.0;

fn variant(geometry: &Geometry) -> LayoutVariant {
    LayoutVariant {
        when: When {
            device: Device::Phone,
            orientation: geometry.orientation,
        },
        canvas: Size {
            width: geometry.width,
            height: ROWS * geometry.row_height,
        },
        grid: None,
        initial_page: None,
        pages: vec![
            page(LETTERS, letters_page(), geometry),
            page(SYMBOLS, symbols_page(), geometry),
            page(PHRASES, phrases_page(), geometry),
        ],
    }
}

/// 預設的 Gboard 速成佈局：手機直向與橫向。
pub fn layout() -> Layout {
    Layout {
        format: FORMAT.into(),
        variants: vec![variant(&PORTRAIT), variant(&LANDSCAPE)],
    }
}
