//! 基準照殼的做法把按鍵交給核心（M4 票 08）：實體鍵盤的字母鍵經 `qim_layout::PhysicalModel` 轉成
//! `Key::Weighted`（同網頁的 `pressCode`，票 07），觸控點擊經 `qim_layout::SpatialModel` 轉成
//! `Key::Weighted`（同網頁的 `tap`，票 06）。容錯進核心後，基準量到的就是殼實際送的機率。

use qim_core::{Key, Weights};
use qim_layout::{
    Device, KeyState, Keyboard, Layout, Orientation, Page, PhysicalModel, Shift, SpatialModel,
    Touch, When,
};

/// 打出輸入碼裡的字元 `c` 要按的鍵（實體鍵盤）：小寫字母是同位置實體鍵的按鍵機率（中文模式整個當美式
/// 鍵盤，字母照實體鍵位，brief §4.8），分隔記號是 `Punct`，其餘同 `Key::for_input`。
pub fn physical_key(c: char) -> Key {
    if !c.is_ascii_lowercase() {
        return Key::for_input(c);
    }
    let code = format!("Key{}", c.to_ascii_uppercase());
    PhysicalModel::default()
        .letter_weights(&code)
        .and_then(|weights| Weights::new(weights).ok())
        .map_or(Key::Letter(c), Key::Weighted)
}

/// 觸控模擬的鍵盤：佈局檔的手機直向鍵盤一開始顯示的頁（字母頁，同 `qim_databuild::typo::touch_page`）與畫布尺寸。
#[derive(Debug, Clone)]
pub struct TouchSurface {
    page: Page,
    mm_per_unit: f64,
}

impl TouchSurface {
    /// 讀佈局檔 `json`；`mm_per_unit` 是畫布一單位幾毫米（[`crate::TOUCH_MM_PER_UNIT`]）。
    pub fn from_json(json: &str, mm_per_unit: f64) -> Result<Self, String> {
        let layout = Layout::from_json(json).map_err(|e| e.to_string())?;
        let when = When {
            device: Device::Phone,
            orientation: Orientation::Portrait,
        };
        let page = Keyboard::new(layout, when).0.page().clone();
        Ok(Self { page, mm_per_unit })
    }

    /// 在畫布座標 `(x, y)` 點一下：各字母的機率（組字中、Shift 沒開）；點的不是字母鍵附近時是 `None`。
    pub fn tap(&self, x: f64, y: f64) -> Option<Key> {
        let state = KeyState {
            composing: true,
            shift: Shift::Off,
        };
        let touch = Touch {
            x,
            y,
            mm_per_unit: self.mm_per_unit,
        };
        let weights = SpatialModel::default().letter_weights(&self.page, touch, state);
        Weights::new(weights).ok().map(Key::Weighted)
    }
}

/// 按鍵機率最高的字母；`Weighted` 以外的鍵照它本身（`Letter` 的字母、`Punct` 的符號）。判斷一題有沒有
/// 打錯用：每一鍵最高的字母都等於原本的碼，就是「沒打錯的題」。
pub fn top_char(key: &Key) -> Option<char> {
    match key {
        Key::Weighted(weights) => Some(weights.top()),
        Key::Letter(c) | Key::Punct(c) => Some(*c),
        _ => None,
    }
}
