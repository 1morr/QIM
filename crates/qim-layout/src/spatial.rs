//! 觸控空間模型（plan §2.6，M4 票 06）：點一下算成各字母的機率，殼交給核心當 `Key::Weighted`。
//!
//! 每個字母鍵依點擊到它觸控範圍的距離（在鍵自己的座標，旋轉、圓角都只是距離函式不同；橢圓與多邊形
//! 在 M6 換距離函式）套 2D Gaussian：σ 跟該鍵的大小成比例，並有手指精度的下限，單位是毫米。不搬 AOSP
//! 的 `ProximityInfo`：它假設矩形、共用尺度、以排為單位（`docs/research/spatial-correction-swipe.md` §1.2）。
//! 參數的初值與來源在 [`SpatialModel::default`]，還沒在基準上調過。

use crate::model::{Action, Key, KeyState, Page};

/// 一次點擊：畫布座標與實體尺寸。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Touch {
    pub x: f64,
    pub y: f64,
    /// 畫布一單位是幾毫米（殼依鍵盤實際寬度與螢幕密度算）。
    pub mm_per_unit: f64,
}

/// 空間模型的參數。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpatialModel {
    /// 手指的絕對精度 σa（毫米）：σ 不隨鍵變小一路縮到 0（FFitts）。
    pub finger_sigma_mm: f64,
    /// 橫軸與縱軸的 σ 裡跟鍵大小成比例的部分：`比例 × 該軸的鍵長`（VelociTap 全鍵共用兩軸各一個比例）。
    /// 兩部分照 FFitts 合起來：`σ = √(σa² + (比例 × 鍵長)²)`。
    pub size_ratio_x: f64,
    pub size_ratio_y: f64,
    /// 點擊判定按到的鍵（重疊時是最上層）的機率乘上它。
    pub top_boost: f64,
    /// 剪枝半徑（毫米）：離點擊更遠的鍵不算，按到的鍵除外。
    pub radius_mm: f64,
    /// 最多回幾個字母。
    pub limit: usize,
}

impl Default for SpatialModel {
    /// 初值（`docs/research/spatial-correction-swipe.md` §2.3）：
    /// - `finger_sigma_mm` 1.5：Bi、Li、Zhai 2013（FFitts law）圓形目標的 σa。
    /// - `size_ratio_x` 0.21、`size_ratio_y` 0.12：Fowler 2015 量到的 σx 1.97 mm、σy 1.88 mm（鍵寬 6.16 mm、
    ///   高 9.42 mm）照 FFitts 拆成 σa² 加上跟鍵大小有關的部分：√(1.97² − 1.5²) ≈ 1.28 mm 是鍵寬的 0.21，
    ///   √(1.88² − 1.5²) ≈ 1.13 mm 是鍵高的 0.12。Fowler 的 σ 量的是到鍵中心的距離，這裡套在到觸控範圍的距離
    ///   （鍵內是 0），分布因此比 Fowler 的寬；當初值用，還沒調過。
    /// - `radius_mm` 7.99：Fowler 2015 的剪枝半徑。
    /// - `limit` 5：Fowler 的剪枝半徑內通常只有 3–6 個鍵（研究筆記〈推論〉）。
    /// - `top_boost` 2：沒有來源，重疊而一樣近的兩個鍵上層得 2/3。
    fn default() -> Self {
        Self {
            finger_sigma_mm: 1.5,
            size_ratio_x: 0.21,
            size_ratio_y: 0.12,
            top_boost: 2.0,
            radius_mm: 7.99,
            limit: 5,
        }
    }
}

impl SpatialModel {
    /// 點一下 `page` 時各字母的機率，由高到低（一樣高時上層的鍵在前；點中的鍵的字母一定在第一），最多 [`SpatialModel::limit`] 個，
    /// 和為 1。只算 `state` 下點一下會送字母的鍵（Shift 開著時字母鍵輸入文字，不算）；同一個字母有好幾個
    /// 鍵時機率相加。頁上沒有字母鍵時是空的。
    pub fn letter_weights(&self, page: &Page, touch: Touch, state: KeyState) -> Vec<(char, f32)> {
        let hit = page.hit(touch.x, touch.y);
        // 對數的權重，最後減掉最大的再取指數，遠處的鍵不會全部下溢成 0。
        let mut letters: Vec<(char, f64)> = Vec::new();
        for index in page.top_down() {
            let key = &page.keys[index];
            let Some(letter) = letter_of(key, state) else {
                continue;
            };
            let (dx, dy) = key.offset(touch.x, touch.y);
            let (dx, dy) = (dx * touch.mm_per_unit, dy * touch.mm_per_unit);
            let is_hit = hit == Some(index);
            if !is_hit && dx.hypot(dy) > self.radius_mm {
                continue;
            }
            let sigma = |ratio: f64, size: f64| {
                let size_mm = size * touch.mm_per_unit;
                self.finger_sigma_mm.hypot(ratio * size_mm)
            };
            let sx = sigma(self.size_ratio_x, key.frame.w);
            let sy = sigma(self.size_ratio_y, key.frame.h);
            let mut log = -(dx * dx / (2.0 * sx * sx) + dy * dy / (2.0 * sy * sy));
            if is_hit {
                log += self.top_boost.ln();
            }
            match letters.iter_mut().find(|(c, _)| *c == letter) {
                Some((_, sum)) => *sum = log_add(*sum, log),
                None => letters.push((letter, log)),
            }
        }
        // 穩定排序：一樣高時留著由上層往下的順序。
        letters.sort_by(|a, b| b.1.total_cmp(&a.1));
        // 點中的鍵的字母一定最高（核心只用最高的，打字要同點擊判定）：大小不一的鍵之間，
        // 大鍵的 σ 寬，可能贏過比較近的小鍵；這時把點中的字母提到和最高的一樣、排第一，留著「兩個一樣
        // 可能」的資訊。
        let hit_letter = hit.and_then(|index| letter_of(&page.keys[index], state));
        if let Some(at) = hit_letter.and_then(|c| letters.iter().position(|&(d, _)| d == c))
            && at > 0
        {
            let (letter, _) = letters.remove(at);
            letters.insert(0, (letter, letters[0].1));
        }
        letters.truncate(self.limit);
        let Some(&(_, max)) = letters.first() else {
            return Vec::new();
        };
        let weights: Vec<(char, f64)> = letters
            .into_iter()
            .map(|(c, log)| (c, (log - max).exp()))
            .collect();
        let sum: f64 = weights.iter().map(|&(_, w)| w).sum();
        weights
            .into_iter()
            .map(|(c, w)| (c, (w / sum) as f32))
            .filter(|&(_, p)| p > 0.0)
            .collect()
    }
}

/// `state` 下點一下 `key` 會送的字母；不送字母的鍵是 `None`。
fn letter_of(key: &Key, state: KeyState) -> Option<char> {
    match key.tap_action(state) {
        Action::Letter(letter) => Some(letter),
        _ => None,
    }
}

/// ln(eᵃ + eᵇ)。
fn log_add(a: f64, b: f64) -> f64 {
    let (high, low) = if a >= b { (a, b) } else { (b, a) };
    high + (low - high).exp().ln_1p()
}
