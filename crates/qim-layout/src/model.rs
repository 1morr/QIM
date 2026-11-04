//! `qim-layout/1` 的型別，欄位名與 JSON 相同（camelCase）；JSON Schema 在
//! `crates/qim-layout/schema/qim-layout-1.schema.json`，兩邊同時改。

use serde::{Deserialize, Serialize};

/// 一份佈局檔。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    /// 格式與版本，現在只有 [`FORMAT`]。
    pub format: String,
    pub variants: Vec<LayoutVariant>,
}

/// 讀得懂的 `format`。
pub const FORMAT: &str = "qim-layout/1";

/// 佈局版本：一種裝置與方向的畫布與頁面。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LayoutVariant {
    pub when: When,
    /// 設計尺寸；按鍵的 `frame` 以它為座標，殼等比例縮放到實際鍵盤寬度。
    pub canvas: Size,
    /// 編輯器的格線與吸附；不影響鍵盤本身。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grid: Option<Grid>,
    /// 鍵盤底色。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<ThemeColor>,
    /// 一開始顯示的頁面；省略時是第一頁。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_page: Option<String>,
    pub pages: Vec<Page>,
}

impl LayoutVariant {
    /// 一開始顯示的頁面在 `pages` 的位置；`initialPage` 指到不存在的頁時是 `None`。
    pub fn initial_page_index(&self) -> Option<usize> {
        match &self.initial_page {
            None => (!self.pages.is_empty()).then_some(0),
            Some(id) => self.page_index(id),
        }
    }

    /// `id` 這一頁在 `pages` 的位置。
    pub fn page_index(&self, id: &str) -> Option<usize> {
        self.pages.iter().position(|page| page.id == id)
    }
}

/// 佈局版本適用的裝置與方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct When {
    pub device: Device,
    pub orientation: Orientation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Device {
    Phone,
    Tablet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Orientation {
    Portrait,
    Landscape,
}

impl Device {
    pub fn name(self) -> &'static str {
        match self {
            Device::Phone => "phone",
            Device::Tablet => "tablet",
        }
    }
}

impl Orientation {
    pub fn name(self) -> &'static str {
        match self {
            Orientation::Portrait => "portrait",
            Orientation::Landscape => "landscape",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

/// 編輯器的格線：畫布分成幾欄幾列，`snap` 為真時拖動吸附到格線（brief §4.6）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Grid {
    pub columns: u32,
    pub rows: u32,
    pub snap: bool,
}

/// 淺色與深色各一個顏色，`#rrggbb` 或 `#rrggbbaa`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThemeColor {
    pub light: String,
    pub dark: String,
}

/// 一頁按鍵。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Page {
    pub id: String,
    pub keys: Vec<Key>,
}

/// 畫布上的一個按鍵。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Key {
    pub id: String,
    pub frame: Frame,
    /// 以 `frame` 中心順時針旋轉的角度（度）。
    #[serde(default, skip_serializing_if = "is_zero")]
    pub rotate: f64,
    /// 上下層順序，大的在上；同一層時陣列裡後面的在上（後畫的蓋住先畫的）。
    #[serde(default)]
    pub z: i32,
    #[serde(default, skip_serializing_if = "Shape::is_default")]
    pub shape: Shape,
    /// 觸控範圍比 `frame` 每邊往外擴的距離，負數往內縮（同 Delta skin 的 `extendedEdges`）。
    #[serde(default, skip_serializing_if = "Insets::is_zero")]
    pub touch_inset: Insets,
    /// 主標籤；省略時見 [`Key::display_label`]。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// 角標，速成的字母鍵寫英文字母。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    /// 無障礙標籤；省略時見 [`Key::accessibility_label`]。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub a11y_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<Style>,
    pub tap: Action,
    /// 長按出現的選項，依序排；第一個是預設。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub long_press: Vec<Action>,
    /// 往四個方向滑時的動作；M5 才有作用，現在只存。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swipe: Option<Swipe>,
    /// 組字中改用的標籤與動作（同 Trime 的 `composing`）；`null` 與省略相同。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composing: Option<Composing>,
}

fn is_zero(value: &f64) -> bool {
    *value == 0.0
}

impl Key {
    /// 鍵上顯示的主標籤：組字中且 `composing` 有寫 `label` 就用它；其次是 `label`；
    /// 都沒寫時字母鍵用方案的字根（`radical`），方案沒有這個鍵的字根就用字母；其他鍵是空字串。
    pub fn display_label<'r>(
        &self,
        composing: bool,
        radical: impl FnOnce(char) -> Option<&'r str>,
    ) -> String {
        let composing_label = self
            .composing
            .as_ref()
            .filter(|_| composing)
            .and_then(|c| c.label.as_ref());
        match (composing_label.or(self.label.as_ref()), &self.tap) {
            (Some(label), _) => label.clone(),
            (None, Action::Letter(letter)) => {
                radical(*letter).map_or_else(|| letter.to_string(), str::to_owned)
            }
            (None, _) => String::new(),
        }
    }

    /// 點一下的動作：組字中且 `composing` 有寫 `tap` 就用它，否則是 `tap`。
    pub fn tap_action(&self, composing: bool) -> &Action {
        self.composing
            .as_ref()
            .filter(|_| composing)
            .and_then(|c| c.tap.as_ref())
            .unwrap_or(&self.tap)
    }

    /// 無障礙標籤：有寫 `a11yLabel` 就用它；否則是顯示的標籤加角標（「日，a」），
    /// 沒有標籤的鍵用動作的說法（「空白」）。
    pub fn accessibility_label(&self, displayed: &str) -> String {
        if let Some(label) = &self.a11y_label {
            return label.clone();
        }
        let base = if displayed.is_empty() {
            self.tap.spoken()
        } else {
            displayed.to_owned()
        };
        match &self.hint {
            Some(hint) if *hint != base => format!("{base}，{hint}"),
            _ => base,
        }
    }

    /// 這個鍵所有的動作：`tap`、長按、滑動、組字中的 `tap`。
    pub fn actions(&self) -> impl Iterator<Item = &Action> {
        let swipe = self.swipe.iter().flat_map(|s| {
            [&s.up, &s.down, &s.left, &s.right]
                .into_iter()
                .filter_map(Option::as_ref)
        });
        let composing = self.composing.iter().filter_map(|c| c.tap.as_ref());
        std::iter::once(&self.tap)
            .chain(&self.long_press)
            .chain(swipe)
            .chain(composing)
    }
}

/// 按鍵的矩形，左上角 `(x, y)`，以畫布座標計；旋轉前的位置。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// 按鍵的形狀。`ellipse`、`polygon` 的欄位已定，繪製與點擊判定在 M5；現在當矩形並給警告。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Shape {
    /// 矩形，`radius` 是圓角半徑（畫布座標），超過短邊一半時取一半。
    Rect {
        #[serde(default, skip_serializing_if = "is_zero")]
        radius: f64,
    },
    Ellipse,
    /// 多邊形，頂點以 `frame` 為 0–1 的座標。
    Polygon {
        points: Vec<[f64; 2]>,
    },
}

impl Default for Shape {
    fn default() -> Self {
        Shape::Rect { radius: 0.0 }
    }
}

impl Shape {
    fn is_default(&self) -> bool {
        *self == Shape::default()
    }

    /// 形狀在佈局檔裡的名字。
    pub fn name(&self) -> &'static str {
        match self {
            Shape::Rect { .. } => "rect",
            Shape::Ellipse => "ellipse",
            Shape::Polygon { .. } => "polygon",
        }
    }
}

/// 四邊的距離。
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Insets {
    #[serde(default)]
    pub top: f64,
    #[serde(default)]
    pub bottom: f64,
    #[serde(default)]
    pub left: f64,
    #[serde(default)]
    pub right: f64,
}

impl Insets {
    fn is_zero(&self) -> bool {
        *self == Insets::default()
    }
}

/// 按鍵外觀；每項都可省略，殼用自己的預設。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Style {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<ThemeColor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_color: Option<ThemeColor>,
    /// 0–1。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    /// 主標籤的字級，畫布座標。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_size: Option<f64>,
    /// 主標籤離鍵中心的位移 `[x, y]`，畫布座標。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_offset: Option<[f64; 2]>,
}

/// 四個方向的滑動動作。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Swipe {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub up: Option<Action>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub down: Option<Action>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left: Option<Action>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right: Option<Action>,
}

/// 組字中改用的標籤與動作，各自可省略。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Composing {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tap: Option<Action>,
}

/// 按鍵的動作。帶資料的寫成 `{ "letter": "a" }`，不帶的寫成名字 `"space"`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    /// 送一個字母給輸入法（`a`–`z`）。
    Letter(char),
    /// 直接輸入一段文字，不經組字。
    Text(String),
    Backspace,
    Space,
    Enter,
    /// 切到同一個佈局版本的另一頁（頁的 `id`）。
    SwitchPage(String),
    /// 切到系統的下一個鍵盤（iOS 的地球鍵）。
    NextKeyboard,
    /// 中英切換。
    ToggleAscii,
}

impl Action {
    /// 動作在佈局檔裡的名字，同 `serde` 的標記。
    pub fn name(&self) -> &'static str {
        match self {
            Action::Letter(_) => "letter",
            Action::Text(_) => "text",
            Action::Backspace => "backspace",
            Action::Space => "space",
            Action::Enter => "enter",
            Action::SwitchPage(_) => "switchPage",
            Action::NextKeyboard => "nextKeyboard",
            Action::ToggleAscii => "toggleAscii",
        }
    }

    /// 沒有標籤時給螢幕閱讀器念的說法。
    fn spoken(&self) -> String {
        match self {
            Action::Letter(c) => c.to_string(),
            Action::Text(text) => text.clone(),
            Action::Backspace => "刪除".into(),
            Action::Space => "空白".into(),
            Action::Enter => "換行".into(),
            Action::SwitchPage(page) => format!("切到 {page}"),
            Action::NextKeyboard => "下一個鍵盤".into(),
            Action::ToggleAscii => "中英切換".into(),
        }
    }
}
