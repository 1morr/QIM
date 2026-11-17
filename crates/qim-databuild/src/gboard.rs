//! 預設 Gboard 速成佈局：由按鍵排列算出畫布上每個鍵的位置（plan〈佈局：畫布模型〉、ADR 0004）。
//!
//! 排列以「鍵寬」為單位，一排十個鍵寬；每個鍵占一格，四周留縫，點在縫上由點擊判定取最近的鍵。
//! 手機直向與橫向兩個佈局版本用同一份排列，只有畫布、排高與縫不同；數字列打開時是另一份佈局
//! （字母頁多一排，畫布變高，其他頁的四排拉高填滿，同 Gboard）。
//! 頁面、鍵位、配色與尺寸照 Android 模擬器上的 Gboard 速成實測（M1 票 27，
//! `.scratch/m1/evidence/27-*.png`）：字母、符號（?123）、更多符號（=\<）、數字鍵盤（12/34），
//! 加上 Gboard 沒有的常用頁（自訂頁面的示範，長按逗號鍵進去，代替 Gboard 的表情入口）。
//! 英文頁與英文的兩個符號頁照 Gboard English (US)（事實 M49），由地球鍵切過去（M4 票 23）。

use qim_core::SEPARATOR;
use qim_layout::{
    Action, Composing, Device, FORMAT, Frame, Insets, Key, Layout, LayoutVariant, Orientation,
    Page, Shape, Size, Style, ThemeColor, When,
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
/// 英文頁與英文的符號頁（速成字母頁、符號頁的 `asciiPage`）。
pub const ENGLISH: &str = "english";
pub const ENGLISH_SYMBOLS: &str = "english-symbols";
pub const ENGLISH_MORE_SYMBOLS: &str = "english-more-symbols";

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

/// 直向：畫布寬是 iPhone 直向的點數寬（plan〈佈局：畫布模型〉 的例子）；其餘照 Gboard 在 411 dp 寬的實測
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

/// 字級，畫布座標（照 Gboard 實測縮成 390 寬）：字根約 15 sp，空白鍵與切頁鍵約 12 sp，
/// 符號頁的符號與數字約 20 sp，數字鍵盤的數字約 24 sp，⇧ ⌫ ↵ 圖示約 24 dp。
const LABEL_SIZE: f64 = 15.0;
const SMALL_LABEL_SIZE: f64 = 12.5;
const SYMBOL_LABEL_SIZE: f64 = 20.0;
const NUMPAD_LABEL_SIZE: f64 = 24.0;
const ICON_LABEL_SIZE: f64 = 24.0;
/// 英文頁的字母：Gboard 英文鍵盤的字母比字根大，大寫與小寫同字級（`Key::display_label_size`）。
const ENGLISH_LETTER_SIZE: f64 = 24.0;
/// 地球鍵的圖示比 ⇧ ⌫ 略小，同 Gboard。
const GLOBE_LABEL_SIZE: f64 = 22.0;

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
    /// 組字中改用的標籤與動作。
    composing: Option<Composing>,
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
            composing: None,
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

    fn plain(mut self) -> Self {
        self.tone = Tone::Plain;
        self
    }

    fn small(self) -> Self {
        self.size(SMALL_LABEL_SIZE)
    }

    fn size(mut self, label_size: f64) -> Self {
        self.label_size = label_size;
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

/// 字母鍵：角標是字母，長按只出英文字母（brief〈手機觸控鍵盤〉）。標籤省略時用方案的字根；
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

/// 符號頁的符號與數字：字比字根大（同 Gboard）。標點也照鍵上的字直接輸出，不送核心的標點鍵
/// （使用者 2026-11-05 決定，M1 票 27b）。
fn symbols(row: &str) -> Vec<Cell> {
    row.split(' ')
        .map(|t| text(t).size(SYMBOL_LABEL_SIZE))
        .collect()
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
        .size(ICON_LABEL_SIZE)
        .function()
        .units(1.5)
}

/// ⇧：組字中換成分隔鍵「'詞」，送核心的分隔記號（同 Gboard 速成，M1 票 28）。
fn shift() -> Cell {
    let mut cell = Cell::new("shift", Action::Shift, Some("⇧"))
        .spoken()
        .size(ICON_LABEL_SIZE)
        .function()
        .units(1.5);
    cell.composing = Some(Composing {
        label: Some(format!("{SEPARATOR}詞")),
        a11y_label: Some("分隔".into()),
        tap: Some(Action::Punct(SEPARATOR)),
    });
    cell
}

fn enter() -> Cell {
    Cell::new("enter", Action::Enter, Some("↵"))
        .spoken()
        .size(ICON_LABEL_SIZE)
        .function()
        .units(1.5)
        .pill()
}

/// 空白鍵顯示方案名「速成」，讀屏唸「空白」。
fn space(units: f64) -> Cell {
    labelled_space(units, "速成")
}

/// 英文的頁的空白鍵寫「English」，同 Gboard 英文鍵盤。
fn english_space(units: f64) -> Cell {
    labelled_space(units, "English")
}

fn labelled_space(units: f64, label: &str) -> Cell {
    Cell::new("space", Action::Space, Some(label))
        .spoken()
        .small()
        .units(units)
}

/// 字母頁與英文頁的逗號鍵：照鍵上的字輸出，長按進常用頁（代替 Gboard 逗號鍵的表情入口），角標 ☺。
fn comma(label: &str) -> Cell {
    let mut cell = text(label).function();
    cell.hint = Some("☺".into());
    cell.a11y_label = Some("逗號，長按切到常用".into());
    cell.long_press = vec![Action::SwitchPage(PHRASES.into())];
    cell
}

/// 地球鍵（M4 票 23）：點一下切中英，長按開切換鍵盤的選單；照 Gboard「Language switch key」是白鍵、
/// 放在逗號與空白之間（事實 M49）。標籤 🌐 由殼畫成單色圖示。
fn globe() -> Cell {
    let mut cell = Cell::new("globe", Action::ToggleAscii, Some("🌐"))
        .size(GLOBE_LABEL_SIZE)
        .long_press(vec![Action::KeyboardMenu]);
    cell.a11y_label = Some(Action::KeyboardMenu.spoken());
    cell
}

/// 「返回」回字母頁（Gboard 符號頁左下）；英文時落到英文頁（字母頁的 `asciiPage`）。
fn back() -> Cell {
    switch(LETTERS, "返回").units(1.5).pill()
}

/// 「ABC」回英文頁（Gboard 英文的符號頁左下）。
fn abc() -> Cell {
    switch(ENGLISH, "ABC").units(1.5).pill()
}

/// 「12/34」切到數字鍵盤（白鍵）。
fn to_numbers() -> Cell {
    let mut cell = switch(NUMBERS, "12\n34").plain();
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

/// 句號鍵長按（Gboard 實測的 2 × 8，預設「……」排第一個，其餘照 Gboard 由左上到右下）；
/// 打出的字元照 Gboard 實測：`；：！？` 全形，表情與其他符號半形。
const PERIOD_LONG_PRESS: &str = "…… ^_^ :) :( :D :P & $ ^ % _ ； ： 、 ！ ？";

/// 字母頁照 Gboard：QWERTY 三排，第二排錯開半鍵；第三排左 Shift、右刪除；
/// 底排 ?123、逗號（長按進常用頁，代替 Gboard 的表情入口）、地球鍵、空白、句號、Enter。
fn letters_page() -> Vec<Row> {
    let letters = |row: &str| row.chars().map(letter).collect::<Vec<_>>();
    let mut third = vec![shift()];
    third.extend(letters("zxcvbnm"));
    third.push(backspace());
    vec![
        row(0.0, letters("qwertyuiop")),
        row(0.5, letters("asdfghjkl")),
        row(0.0, third),
        row(
            0.0,
            vec![
                switch(SYMBOLS, "?123").units(1.5).pill(),
                comma("，"),
                globe(),
                space(4.0),
                text("。")
                    .function()
                    .long_press(texts_of(PERIOD_LONG_PRESS)),
                enter(),
            ],
        ),
    ]
}

/// 英文頁的句號鍵長按：Gboard English (US) 的 2 × 8（事實 M49），預設的 `,` 排第一個（同速成頁把預設排第一），
/// 其餘照 Gboard 由左上到右下。全是半形，英文頁本身就打得出 `? ! : ;`。
const ENGLISH_PERIOD_LONG_PRESS: &str = ", & % + \" - : ' @ ; / ( ) # ! ?";

/// 英文頁照 Gboard English (US)（事實 M49）：QWERTY 小寫字母直接送字母（英文直通或英文模式由核心決定）；
/// `digit_hints` 時第一排右上角標 1–0、長按打數字（使用者 2026-11-17 同意；數字列打開時不標，同 Gboard）。
/// ⇧ 沒有組字中的分隔鍵；底排 ?123、逗號、地球鍵、空白、句號、Enter，逗號長按照速成頁進常用頁。
fn english_page(digit_hints: bool) -> Vec<Row> {
    let english_letter = |c: char| {
        Cell::new(&c.to_string(), Action::Letter(c), Some(&c.to_string())).size(ENGLISH_LETTER_SIZE)
    };
    let top = "qwertyuiop"
        .chars()
        .zip("1234567890".chars())
        .map(|(c, digit)| {
            let mut cell = english_letter(c);
            if digit_hints {
                cell.hint = Some(digit.to_string());
                cell.long_press = vec![Action::Text(digit.to_string())];
            }
            cell
        })
        .collect();
    let letters = |row: &str| row.chars().map(english_letter).collect::<Vec<_>>();
    let mut shift = shift();
    shift.composing = None;
    let mut third = vec![shift];
    third.extend(letters("zxcvbnm"));
    third.push(backspace());
    vec![
        row(0.0, top),
        row(0.5, letters("asdfghjkl")),
        row(0.0, third),
        row(
            0.0,
            vec![
                switch(ENGLISH_SYMBOLS, "?123").units(1.5).pill(),
                comma(","),
                globe(),
                english_space(4.0),
                text(".")
                    .function()
                    .long_press(texts_of(ENGLISH_PERIOD_LONG_PRESS)),
                enter(),
            ],
        ),
    ]
}

/// 英文的符號頁（英文頁的 ?123）：照 Gboard English (US)，括號與 `: ; ! ?` 半形，「ABC」回英文頁。
fn english_symbols_page() -> Vec<Row> {
    symbol_page_with(
        ["1 2 3 4 5 6 7 8 9 0", "@ # $ _ & - + ( ) /"],
        switch(ENGLISH_MORE_SYMBOLS, "=\\<"),
        "* \" ' : ; ! ?",
        [",", "."],
        abc(),
        english_space(4.0),
    )
}

/// 英文的更多符號頁（=\<）：照 Gboard English (US)，左邊「?123」回英文的符號頁。
fn english_more_symbols_page() -> Vec<Row> {
    symbol_page_with(
        ["~ ` | • √ π ÷ × § Δ", "£ ¢ € ¥ ^ ° = { } \\"],
        switch(ENGLISH_SYMBOLS, "?123"),
        "% © ® ™ ✓ [ ]",
        ["<", ">"],
        abc(),
        english_space(4.0),
    )
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
            text(&digit.to_string())
                .size(SYMBOL_LABEL_SIZE)
                .long_press(long_press)
        })
        .collect();
    Row {
        indent: 0.0,
        weight: NUMBER_ROW_WEIGHT,
        cells,
    }
}

/// 兩個符號頁共用的排法：兩排符號；第三排左邊切到另一個符號頁、七個符號、刪除；
/// 底排「返回」、兩個功能鍵符號夾著「12/34」與空白、Enter。
fn symbol_page(rows: [&str; 2], other: Cell, third: &str, bottom: [&str; 2]) -> Vec<Row> {
    symbol_page_with(rows, other, third, bottom, back(), space(4.0))
}

/// [`symbol_page`]，左下回字母頁的鍵與空白鍵另給（英文的符號頁是「ABC」與「English」）。
fn symbol_page_with(
    rows: [&str; 2],
    other: Cell,
    third: &str,
    [left, right]: [&str; 2],
    back: Cell,
    space: Cell,
) -> Vec<Row> {
    let mut third_row = vec![other.units(1.5)];
    third_row.extend(symbols(third));
    third_row.push(backspace());
    vec![
        row(0.0, symbols(rows[0])),
        row(0.0, symbols(rows[1])),
        row(0.0, third_row),
        row(
            0.0,
            vec![
                back,
                text(left).function(),
                to_numbers(),
                space,
                text(right).function(),
                enter(),
            ],
        ),
    ]
}

/// 符號頁（?123）：照 Gboard；打完留在符號頁（brief〈手機觸控鍵盤〉），「返回」回字母頁，「=\<」切到更多符號。
/// 鍵上的字就是打出的字（M1 票 27b）：Gboard 實測打全形的 `：；！？（）` 標成全形，
/// 底排 `, .` 與其他 ASCII 標點半形，同 Gboard。
fn symbols_page() -> Vec<Row> {
    symbol_page(
        ["1 2 3 4 5 6 7 8 9 0", "@ # $ _ & - + （ ） /"],
        switch(MORE_SYMBOLS, "=\\<"),
        "、 “ ” ： ； ！ ？",
        [",", "."],
    )
}

/// 更多符號頁（=\<）：照 Gboard；左邊「?123」回符號頁；方括號照實測打全形 `［］`（直接輸出）。
fn more_symbols_page() -> Vec<Row> {
    symbol_page(
        ["~ ` | • √ π ÷ × * §", "£ ¢ € ¥ ^ ° = 「 」 \\"],
        switch(SYMBOLS, "?123"),
        "% ‘ ’ ™ ✓ ［ ］",
        ["<", ">"],
    )
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
            text(&d.to_string()).size(NUMPAD_LABEL_SIZE),
            1.5 + (i % 3) as f64 * digit,
            (i / 3) as f64,
            digit,
            1.0,
        ));
    }
    let mut space = space(1.5).function();
    space.label = Some("␣".into());
    space.label_size = LABEL_SIZE;
    for (i, cell) in [
        text("%").function().size(SYMBOL_LABEL_SIZE),
        space,
        backspace(),
    ]
    .into_iter()
    .enumerate()
    {
        placed.push(Placed::new(cell, 8.5, i as f64, 1.5, 1.0));
    }
    let bottom = [
        (back(), 1.5),
        (text(",").function(), 1.0),
        (switch(SYMBOLS, "!?#").plain(), digit - 1.0),
        (text("0").size(NUMPAD_LABEL_SIZE), digit),
        (text("=").size(SYMBOL_LABEL_SIZE), digit - 1.0),
        (text(".").function(), 1.0),
        (enter(), 1.5),
    ];
    let mut left = 0.0;
    for (cell, units) in bottom {
        placed.push(Placed::new(cell, left, 3.0, units, 1.0));
        left += units;
    }
    placed
}

/// 自訂頁面的示範（brief〈手機觸控鍵盤〉「一個自訂頁面」）：表情與粵語常用語，示範 `switchPage` 與多字的
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
                composing: cell.composing,
            }
        })
        .collect();
    Page {
        id: id.into(),
        ascii_page: None,
        keys,
    }
}

/// 字母頁上方要不要多一排數字（brief〈手機觸控鍵盤〉 的數字列，同 Gboard 的 Number row）。
#[derive(Clone, Copy, PartialEq)]
enum NumberRow {
    Off,
    On,
}

/// 沒有數字列時每頁四排。
const ROWS: f64 = 4.0;

fn variant(geometry: &Geometry, number_row: NumberRow) -> LayoutVariant {
    let mut letters = letters_page();
    let mut english = english_page(number_row == NumberRow::Off);
    let mut height = ROWS * geometry.row_height;
    if number_row == NumberRow::On {
        letters.insert(0, self::number_row());
        english.insert(0, self::number_row());
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
        number_page: Some(NUMBERS.into()),
        pages: vec![
            with_ascii_page(
                page(LETTERS, place_rows(letters), letter_rows, geometry, height),
                ENGLISH,
            ),
            with_ascii_page(rows(SYMBOLS, symbols_page()), ENGLISH_SYMBOLS),
            with_ascii_page(
                rows(MORE_SYMBOLS, more_symbols_page()),
                ENGLISH_MORE_SYMBOLS,
            ),
            page(NUMBERS, numbers_page(), ROWS, geometry, height),
            rows(PHRASES, phrases_page()),
            page(ENGLISH, place_rows(english), letter_rows, geometry, height),
            rows(ENGLISH_SYMBOLS, english_symbols_page()),
            rows(ENGLISH_MORE_SYMBOLS, english_more_symbols_page()),
        ],
    }
}

/// 切到英文時這頁換成 `english`（[`Page::ascii_page`]）。
fn with_ascii_page(mut page: Page, english: &str) -> Page {
    page.ascii_page = Some(english.into());
    page
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
