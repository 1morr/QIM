//! 佈局：畫布模型的資料格式與點擊判定（plan §2.6、ADR 0004）。
//!
//! 純邏輯，不做 I/O；依賴規則見 `docs/plan.md` §1.3。
//! 欄位名照 plan §2.6；這裡只有觸控 tracer（M1 票 18）用到的子集，其餘在票 19 定稿。

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
#[serde(deny_unknown_fields)]
pub struct LayoutVariant {
    pub when: When,
    /// 設計尺寸；按鍵的 `frame` 以它為座標，殼等比例縮放到實際鍵盤寬度。
    pub canvas: Size,
    pub pages: Vec<Page>,
}

/// 佈局版本適用的裝置與方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct When {
    pub device: Device,
    pub orientation: Orientation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Device {
    Phone,
    Tablet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Orientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Size {
    pub width: f64,
    pub height: f64,
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
#[serde(deny_unknown_fields)]
pub struct Key {
    pub id: String,
    pub frame: Frame,
    /// 上下層順序，大的在上；同一層時陣列裡後面的在上（後畫的蓋住先畫的）。
    #[serde(default)]
    pub z: i32,
    /// 主標籤；省略時見 [`Key::display_label`]。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// 角標，速成的字母鍵寫英文字母。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    pub tap: Action,
}

impl Key {
    /// 鍵上顯示的主標籤：有寫 `label` 就用它；沒寫時字母鍵用方案的字根（`radical`），
    /// 方案沒有這個鍵的字根就用字母；其他鍵是空字串。
    pub fn display_label<'r>(&self, radical: impl FnOnce(char) -> Option<&'r str>) -> String {
        match (&self.label, &self.tap) {
            (Some(label), _) => label.clone(),
            (None, Action::Letter(letter)) => {
                radical(*letter).map_or_else(|| letter.to_string(), str::to_owned)
            }
            (None, _) => String::new(),
        }
    }
}

/// 按鍵的矩形，左上角 `(x, y)`，以畫布座標計。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// 按鍵的動作。帶資料的寫成 `{ "letter": "a" }`，不帶的寫成名字 `"space"`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    Letter(char),
    Space,
    Backspace,
    Enter,
}

/// 讀不了的佈局檔。
#[derive(Debug)]
pub struct Error(String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl Layout {
    /// 讀佈局檔；`format` 不是 [`FORMAT`]、沒有佈局版本或某個版本沒有頁面時失敗。
    pub fn from_json(text: &str) -> Result<Self, Error> {
        let layout: Self = serde_json::from_str(text).map_err(|e| Error(e.to_string()))?;
        if layout.format != FORMAT {
            return Err(Error(format!(
                "format 是 {:?}，只讀得懂 {FORMAT:?}",
                layout.format
            )));
        }
        if layout.variants.is_empty() {
            return Err(Error("沒有佈局版本".into()));
        }
        if layout
            .variants
            .iter()
            .any(|variant| variant.pages.is_empty())
        {
            return Err(Error("佈局版本沒有頁面".into()));
        }
        Ok(layout)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("佈局只有字串、數字與列舉，一定序列化得了")
    }
}

impl Frame {
    /// 左、上邊算在內，右、下邊不算：相鄰兩鍵的共用邊只屬於一個鍵。
    fn contains(&self, x: f64, y: f64) -> bool {
        (self.x..self.x + self.w).contains(&x) && (self.y..self.y + self.h).contains(&y)
    }

    /// 點到矩形的距離平方；在矩形內是 0。
    fn distance_squared(&self, x: f64, y: f64) -> f64 {
        let dx = (self.x - x).max(x - (self.x + self.w)).max(0.0);
        let dy = (self.y - y).max(y - (self.y + self.h)).max(0.0);
        dx * dx + dy * dy
    }
}

impl Page {
    /// 點在畫布座標 `(x, y)` 時按到的鍵：由上層往下，落在矩形內的第一個；
    /// 都沒有時取距離最近的鍵，一樣近時取上層的。沒有按鍵時是 `None`。
    pub fn hit(&self, x: f64, y: f64) -> Option<&Key> {
        self.top_down()
            .find(|key| key.frame.contains(x, y))
            .or_else(|| {
                // `min_by` 一樣小時取先出現的，也就是上層的。
                self.top_down().min_by(|a, b| {
                    a.frame
                        .distance_squared(x, y)
                        .total_cmp(&b.frame.distance_squared(x, y))
                })
            })
    }

    /// 由上層往下層：`z` 大的先，同一層時陣列裡後面的先。
    fn top_down(&self) -> impl Iterator<Item = &Key> {
        let mut keys: Vec<&Key> = self.keys.iter().rev().collect();
        // 穩定排序，同一層保留反轉後的順序。
        keys.sort_by_key(|key| std::cmp::Reverse(key.z));
        keys.into_iter()
    }
}
