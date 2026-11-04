//! 預設 Gboard 速成佈局：由按鍵排列算出畫布上每個鍵的位置（plan §2.6、ADR 0004）。
//!
//! 排列以「鍵寬」為單位，一排十個鍵寬；每個鍵占一格，四周留縫，點在縫上由點擊判定取最近的鍵。
//! 手機直向與橫向兩個佈局版本用同一份排列，只有畫布、排高與縫不同；數字列打開時是另一份佈局
//! （字母頁多一排，畫布變高，其他頁的四排拉高填滿，同 Gboard）。
//! 頁面、鍵位、配色與尺寸照 Android 模擬器上的 Gboard 速成實測（M1 票 27，
//! `.scratch/m1/evidence/27-*.png`）：字母、符號（?123）、更多符號（=\<）、數字鍵盤（12/34），
//! 加上 Gboard 沒有的常用頁（自訂頁面的示範，長按逗號鍵進去，代替 Gboard 的表情入口）。

use qim_layout::{
    Action, Device, FORMAT, Frame, Insets, Key, Layout, LayoutVariant, Orientation, Page, Shape,
    Size, Style, ThemeColor, When,
};

/// 寫到 `build/layouts/` 的檔名。
pub const FILE: &str = "gboard-quick.json";
/// 數字列打開時的佈局（[`number_row_layout`]）。
pub const NUMBER_ROW_FILE: &str = "gboard-quick-number-row.json";

/// 頁的 id。
pub const LETTERS: &str = "letters";
pub const SYMBOLS: &str = "symbols";
pub const MORE_SYMBOLS: &str = "more-symbols";
pub const NUMBERS: &str = "numbers";
/// 自訂頁面的示範；id 也是長按清單上顯示的字（「切到 常用」）。
pub const PHRASES: &str = "常用";

/// 一排十個鍵寬。
const UNITS_PER_ROW: f64 = 10.0;

/// 一個佈局版本的尺寸：畫布寬、排高、鍵盤左右的留白、鍵與格子邊緣的距離（左右、上下）、圓角。
struct Geometry {
    orientation: Orientation,
    width: f64,
    row_height: f64,
    padding_x: f64,
    gap_x: f64,
    gap_y: f64,
    radius: f64,
}

/// 直向：畫布寬是 iPhone 直向的點數寬（plan §2.6 的例子）；其餘照 Gboard 在 411 dp 寬的實測
/// 等比例縮成 390（排距 58.9 dp、鍵距 40.8 dp、縫 5.3 × 11.8 dp、左右多留 2 dp、圓角 6 dp）。
const PORTRAIT: Geometry = Geometry {
    orientation: Orientation::Portrait,
    width: 390.0,
    row_height: 56.0,
    padding_x: 2.0,
    gap_x: 2.5,
    gap_y: 5.5,
    radius: 5.5,
};

/// 橫向：iPhone 橫向的點數寬；鍵變寬、排變矮，鍵盤不占掉太多高度（Gboard 橫向沒有實測）。
const LANDSCAPE: Geometry = Geometry {
    orientation: Orientation::Landscape,
    width: 844.0,
    row_height: 40.0,
    padding_x: 4.0,
    gap_x: 4.0,
    gap_y: 4.0,
    radius: 5.5,
};

/// 數字列打開時，Gboard 的鍵盤只高出 0.57 排（數字列與其他排都壓矮了一點）。
const NUMBER_ROW_EXTRA_ROWS: f64 = 0.57;
/// 數字列比其他排矮：Gboard 實測數字列鍵高 41 dp、字母排 44–47 dp。
const NUMBER_ROW_WEIGHT: f64 = 0.88;

fn color(light: &str, dark: &str) -> ThemeColor {
    ThemeColor {
        light: light.into(),
        dark: dark.into(),
    }
}

/// 按鍵的底色。Gboard 預設主題（Material You 跟著桌布變色，這是模擬器預設桌布的顏色）：
/// 字母、符號與空白是白鍵；Shift、刪除、切頁、Enter 與字母頁的逗號、句號是淡藍的功能鍵。
#[derive(Clone, Copy, PartialEq)]
enum Tone {
    Plain,
    Function,
}

/// 字級，畫布座標（Gboard 字根約 16 sp、空白鍵與切頁鍵約 12 sp，縮成 390 寬）。
const LABEL_SIZE: f64 = 16.0;
const SMALL_LABEL_SIZE: f64 = 12.5;

fn style(tone: Tone, label_size: f64) -> Style {
    let (fill, label) = match tone {
        Tone::Plain => (color("#ffffff", "#292c34"), color("#30323a", "#e4e5f0")),
        Tone::Function => (color("#dce2f9", "#353b4d"), color("#4b5164", "#b8bfd5")),
    };
    Style {
        fill: Some(fill),
        label_color: Some(label),
        label_size: Some(label_size),
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
    /// 標籤是符號（⌫、↵）時給讀屏的文字；其他鍵由標籤與角標組出。
    a11y_label: Option<String>,
    hint: Option<String>,
    long_press: Vec<Action>,
    tone: Tone,
    label_size: f64,
    /// 膠囊形（圓角是半高），Gboard 的 ?123、返回、Enter。
    pill: bool,
}

impl Cell {
    fn new(id: &str, tap: Action, label: Option<&str>) -> Self {
        Self {
            id: id.into(),
            tap,
            units: 1.0,
            label: label.map(str::to_owned),
            a11y_label: None,
            hint: None,
            long_press: Vec::new(),
            tone: Tone::Plain,
            label_size: LABEL_SIZE,
            pill: false,
        }
    }

    /// 符號標籤的鍵給讀屏唸動作的說法（「刪除」「換行」）。
    fn spoken(mut self) -> Self {
        self.a11y_label = Some(self.tap.spoken());
        self
    }

    fn units(mut self, units: f64) -> Self {
        self.units = units;
        self
    }

    fn function(mut self) -> Self {
        self.tone = Tone::Function;
        self
    }

    fn small(mut self) -> Self {
        self.label_size = SMALL_LABEL_SIZE;
        self
    }

    fn pill(mut self) -> Self {
        self.pill = true;
        self
    }

    fn long_press(mut self, actions: Vec<Action>) -> Self {
        self.long_press = actions;
        self
    }
}

/// 字母鍵：角標是字母，長按只出英文字母（brief §4.4）。標籤省略時用方案的字根；
/// 方案沒有字根的鍵（`z`）照 Gboard 寫死。
fn letter(c: char) -> Cell {
    let label = (c == 'z').then_some("重");
    let mut cell = Cell::new(&c.to_string(), Action::Letter(c), label);
    cell.hint = Some(c.to_string());
    cell.long_press = vec![Action::Text(c.to_string())];
    cell
}

/// 直接輸入 `text` 的鍵，id 與標籤都是文字本身。
fn text(text: &str) -> Cell {
    Cell::new(text, Action::Text(text.into()), Some(text))
}

fn texts(row: &str) -> Vec<Cell> {
    row.split(' ').map(text).collect()
}

fn texts_of(list: &str) -> Vec<Action> {
    list.split(' ').map(|t| Action::Text(t.into())).collect()
}

/// 切頁鍵，id 是 `to-<頁>`；同一頁有兩個鍵切到同一頁時用 `id` 分開。
fn switch(page: &str, label: &str) -> Cell {
    Cell::new(
        &format!("to-{page}"),
        Action::SwitchPage(page.into()),
        Some(label),
    )
    .function()
    .small()
}

fn backspace() -> Cell {
    Cell::new("backspace", Action::Backspace, Some("⌫"))
        .spoken()
        .function()
        .units(1.5)
}

fn shift() -> Cell {
    Cell::new("shift", Action::Shift, Some("⇧"))
        .spoken()
        .function()
        .units(1.5)
}

fn enter() -> Cell {
    Cell::new("enter", Action::Enter, Some("↵"))
        .spoken()
        .function()
        .units(1.5)
        .pill()
}

/// 空白鍵顯示方案名「速成」，讀屏唸「空白」。
fn space(units: f64) -> Cell {
    Cell::new("space", Action::Space, Some("速成"))
        .spoken()
        .small()
        .units(units)
}

/// 「返回」回字母頁（Gboard 符號頁左下）。
fn back() -> Cell {
    switch(LETTERS, "返回").units(1.5).pill()
}

/// 「12/34」切到數字鍵盤（白鍵）。
fn to_numbers() -> Cell {
    let mut cell = switch(NUMBERS, "12\n34");
    cell.tone = Tone::Plain;
    cell.a11y_label = Some("數字鍵盤".into());
    cell
}

/// 一排：從左邊空幾個鍵寬開始，接著依序排；`weight` 是排高的比例（數字列比較矮）。
struct Row {
    indent: f64,
    weight: f64,
    cells: Vec<Cell>,
}

fn row(indent: f64, cells: Vec<Cell>) -> Row {
    Row {
        indent,
        weight: 1.0,
        cells,
    }
}

/// 句號鍵長按（Gboard 實測的 2 × 8，預設「……」排第一個，其餘照 Gboard 由左上到右下）。
const PERIOD_LONG_PRESS: &str = "…… ^_^ :) :( :D :P & $ ^ % _ ; : 、 ! ?";

/// 字母頁照 Gboard：QWERTY 三排，第二排錯開半鍵；第三排左 Shift、右刪除；
/// 底排 ?123、逗號（長按進常用頁，代替 Gboard 的表情入口）、空白、句號、Enter。
fn letters_page() -> Vec<Row> {
    let letters = |row: &str| row.chars().map(letter).collect::<Vec<_>>();
    let mut third = vec![shift()];
    third.extend(letters("zxcvbnm"));
    third.push(backspace());
    let mut comma = text("，").function();
    comma.hint = Some("☺".into());
    comma.a11y_label = Some("，，長按切到常用".into());
    comma.long_press = vec![Action::SwitchPage(PHRASES.into())];
    vec![
        row(0.0, letters("qwertyuiop")),
        row(0.5, letters("asdfghjkl")),
        row(0.0, third),
        row(
            0.0,
            vec![
                switch(SYMBOLS, "?123").units(1.5).pill(),
                comma,
                space(5.0),
                text("。")
                    .function()
                    .long_press(texts_of(PERIOD_LONG_PRESS)),
                enter(),
            ],
        ),
    ]
}

/// 數字列（Gboard 的 Number row）：點一下輸入數字，長按出上標（`1` 另有分數，同 Gboard）。
fn number_row() -> Row {
    let cells = "1234567890"
        .chars()
        .zip("¹²³⁴⁵⁶⁷⁸⁹⁰".chars())
        .map(|(digit, superscript)| {
            let mut long_press = vec![Action::Text(superscript.to_string())];
            if digit == '1' {
                long_press.extend(texts_of("½ ⅓ ¼ ⅕ ⅙ ⅐ ⅛ ⅑ ⅒"));
            }
            text(&digit.to_string()).long_press(long_press)
        })
        .collect();
    Row {
        indent: 0.0,
        weight: NUMBER_ROW_WEIGHT,
        cells,
    }
}

/// 符號頁（?123）：照 Gboard；打完留在符號頁（brief §4.4），「返回」回字母頁。
fn symbols_page() -> Vec<Row> {
    let mut third = vec![switch(MORE_SYMBOLS, "=\\<").units(1.5)];
    third.extend(texts("、 “ ” : ; ! ?"));
    third.push(backspace());
    vec![
        row(0.0, texts("1 2 3 4 5 6 7 8 9 0")),
        row(0.0, texts("@ # $ _ & - + ( ) /")),
        row(0.0, third),
        row(
            0.0,
            vec![
                back(),
                text(",").function(),
                to_numbers(),
                space(4.0),
                text(".").function(),
                enter(),
            ],
        ),
    ]
}

/// 更多符號頁（=\<）：照 Gboard；左邊「?123」回符號頁。
fn more_symbols_page() -> Vec<Row> {
    let mut third = vec![switch(SYMBOLS, "?123").units(1.5)];
    third.extend(texts("% ‘ ’ ™ ✓ [ ]"));
    third.push(backspace());
    vec![
        row(0.0, texts("~ ` | • √ π ÷ × * §")),
        row(0.0, texts("£ ¢ € ¥ ^ ° = 「 」 \\")),
        row(0.0, third),
        row(
            0.0,
            vec![
                back(),
                text("<").function(),
                to_numbers(),
                space(4.0),
                text(">").function(),
                enter(),
            ],
        ),
    ]
}

/// 數字鍵盤（12/34）：照 Gboard，左欄運算符、中間 3 × 3、右欄 % 空白 刪除；
/// 底排「!?#」回符號頁。Gboard 左欄是一條可捲動的鍵，這裡拆成上下四個鍵。
fn numbers_page() -> Vec<Placed> {
    let digit = 7.0 / 3.0;
    let mut placed = Vec::new();
    for (i, op) in ["+", "-", "*", "/"].into_iter().enumerate() {
        placed.push(Placed::new(
            text(op).function(),
            0.0,
            i as f64 * 0.75,
            1.5,
            0.75,
        ));
    }
    for (i, d) in "123456789".chars().enumerate() {
        placed.push(Placed::new(
            text(&d.to_string()),
            1.5 + (i % 3) as f64 * digit,
            (i / 3) as f64,
            digit,
            1.0,
        ));
    }
    let mut space = space(1.5).function();
    space.label = Some("␣".into());
    space.label_size = LABEL_SIZE;
    for (i, cell) in [text("%").function(), space, backspace()]
        .into_iter()
        .enumerate()
    {
        placed.push(Placed::new(cell, 8.5, i as f64, 1.5, 1.0));
    }
    let bottom = [
        (back(), 1.5),
        (text(",").function(), 1.0),
        (switch(SYMBOLS, "!?#"), digit - 1.0),
        (text("0"), digit),
        (text("="), digit - 1.0),
        (text(".").function(), 1.0),
        (enter(), 1.5),
    ];
    let mut left = 0.0;
    for (mut cell, units) in bottom {
        if cell.id == format!("to-{SYMBOLS}") {
            cell.tone = Tone::Plain;
        }
        placed.push(Placed::new(cell, left, 3.0, units, 1.0));
        left += units;
    }
    placed
}

/// 自訂頁面的示範（brief §4.4「一個自訂頁面」）：表情與粵語常用語，示範 `switchPage` 與多字的
/// `text`；由字母頁長按逗號鍵進來，「ABC」回字母頁。
fn phrases_page() -> Vec<Row> {
    let mut third = vec![switch(SYMBOLS, "?123").units(1.5)];
    third.extend(["哈哈", "收到"].map(|t| text(t).units(3.5)));
    third.push(backspace());
    vec![
        row(0.0, texts("😀 😂 😍 🥲 👍 🙏 👏 ❤️ 🎉 🔥")),
        row(
            0.0,
            ["唔該", "多謝", "早晨", "好呀", "冇問題"]
                .map(|t| text(t).units(2.0))
                .into(),
        ),
        row(0.0, third),
        row(
            0.0,
            vec![
                switch(LETTERS, "ABC").units(1.5).pill(),
                text("，").function(),
                space(5.0),
                text("。").function(),
                enter(),
            ],
        ),
    ]
}

/// 擺好的一格：位置與大小以鍵寬（x、w）與排高（y、h）為單位。
struct Placed {
    cell: Cell,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Placed {
    fn new(cell: Cell, x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { cell, x, y, w, h }
    }
}

/// 把一排一排的排列攤成擺好的格子；排高照 `weight` 的比例，總高換算成整數排。
fn place_rows(rows: Vec<Row>) -> Vec<Placed> {
    let total: f64 = rows.iter().map(|row| row.weight).sum();
    let scale = rows.len() as f64 / total;
    let mut placed = Vec::new();
    let mut top = 0.0;
    for row in rows {
        let height = row.weight * scale;
        let mut left = row.indent;
        for cell in row.cells {
            let width = cell.units;
            placed.push(Placed::new(cell, left, top, width, height));
            left += width;
        }
        top += height;
    }
    placed
}

/// 依 `geometry` 把擺好的格子算成一頁按鍵；`rows` 排平分畫布高 `height`。
fn page(id: &str, placed: Vec<Placed>, rows: f64, geometry: &Geometry, height: f64) -> Page {
    let unit = (geometry.width - 2.0 * geometry.padding_x) / UNITS_PER_ROW;
    let row_height = height / rows;
    let keys = placed
        .into_iter()
        .map(|Placed { cell, x, y, w, h }| {
            let frame = Frame {
                x: geometry.padding_x + x * unit + geometry.gap_x,
                y: y * row_height + geometry.gap_y,
                w: w * unit - 2.0 * geometry.gap_x,
                h: h * row_height - 2.0 * geometry.gap_y,
            };
            let radius = if cell.pill {
                frame.h / 2.0
            } else {
                geometry.radius
            };
            Key {
                id: cell.id,
                frame,
                rotate: 0.0,
                z: 0,
                shape: Shape::Rect { radius },
                extended_edges: Insets::default(),
                label: cell.label,
                hint: cell.hint,
                a11y_label: cell.a11y_label,
                style: Some(style(cell.tone, cell.label_size)),
                tap: cell.tap,
                long_press: cell.long_press,
                swipe: None,
                composing: None,
            }
        })
        .collect();
    Page {
        id: id.into(),
        keys,
    }
}

/// 字母頁上方要不要多一排數字（brief §4.4 的數字列，同 Gboard 的 Number row）。
#[derive(Clone, Copy, PartialEq)]
enum NumberRow {
    Off,
    On,
}

/// 沒有數字列時每頁四排。
const ROWS: f64 = 4.0;

fn variant(geometry: &Geometry, number_row: NumberRow) -> LayoutVariant {
    let mut letters = letters_page();
    let mut height = ROWS * geometry.row_height;
    if number_row == NumberRow::On {
        letters.insert(0, self::number_row());
        height += NUMBER_ROW_EXTRA_ROWS * geometry.row_height;
    }
    let letter_rows = letters.len() as f64;
    let rows = |id: &str, rows: Vec<Row>| {
        let count = rows.len() as f64;
        page(id, place_rows(rows), count, geometry, height)
    };
    LayoutVariant {
        when: When {
            device: Device::Phone,
            orientation: geometry.orientation,
        },
        canvas: Size {
            width: geometry.width,
            height,
        },
        grid: None,
        initial_page: None,
        pages: vec![
            page(LETTERS, place_rows(letters), letter_rows, geometry, height),
            rows(SYMBOLS, symbols_page()),
            rows(MORE_SYMBOLS, more_symbols_page()),
            page(NUMBERS, numbers_page(), ROWS, geometry, height),
            rows(PHRASES, phrases_page()),
        ],
    }
}

fn build(number_row: NumberRow) -> Layout {
    Layout {
        format: FORMAT.into(),
        variants: vec![
            variant(&PORTRAIT, number_row),
            variant(&LANDSCAPE, number_row),
        ],
    }
}

/// 預設的 Gboard 速成佈局：手機直向與橫向，數字列關（Gboard 的預設）。
pub fn layout() -> Layout {
    build(NumberRow::Off)
}

/// 同 [`layout`]，字母頁上方多一排數字；設定打開數字列時殼改讀這份（[`NUMBER_ROW_FILE`]）。
pub fn number_row_layout() -> Layout {
    build(NumberRow::On)
}
