//! 鄰鍵讀法（M4 票 09b 的實驗，照 Gboard 速成的做法）：觸控一下的按鍵機率裡的鄰鍵字母換進輸入碼，
//! 換出來的讀法插在候選列的指定格；原樣讀法照舊當首選，點擊位置本身模稜兩可時才讓鄰鍵讀法當首選。
//!
//! 只給基準與實驗用（[`crate::Engine::with_neighbor_readings`]），預設關、殼不接；量完刪掉
//! （`docs/research/tolerance.md`〈Gboard 式的鄰鍵讀法〉）。

use crate::Weights;

/// 鄰鍵讀法的設定。
#[derive(Debug, Clone, Copy)]
pub struct NeighborReadings {
    /// 換成的字母的機率至少要是打出的字母的幾倍（0–1）；低於它的不試。
    pub floor: f32,
    /// 每一碼除了打出的字母，最多試幾個鄰鍵字母（照機率由高到低）。
    pub letters: usize,
    /// 空間懲罰的倍數：換一碼扣 `weight × ln(打出的字母的機率 ÷ 換成的字母的機率)`。
    pub weight: f64,
    /// 第一個鄰鍵讀法放在候選列第幾格（從 1 起；原樣的首選在第 1 格，所以通常 ≥ 2）。
    pub slot: usize,
    /// 最多列幾個鄰鍵讀法。
    pub limit: usize,
    /// 模稜兩可的門檻：最好的鄰鍵讀法換的每一碼的機率比（換成的 ÷ 打出的）乘起來 ≥ 它、分數又高過原樣
    /// 的首選時，它當首選；`None` 是一律不搶首選。
    pub ambiguous: Option<f32>,
    /// 這一下是不是觸控（殼送的按鍵機率才算數，實體鍵盤照舊只取最高的字母）。核心分不出兩者，由實驗給。
    pub accepts: fn(&Weights) -> bool,
}

impl NeighborReadings {
    /// 打出的字母 `typed` 在這一下（`weights`）要試的鄰鍵字母與空間懲罰（未乘 [`Self::weight`]），照機率
    /// 由高到低；`allowed` 篩掉方案沒有的鍵。
    pub(crate) fn alternatives(
        &self,
        typed: char,
        weights: &Weights,
        allowed: impl Fn(char) -> bool,
    ) -> Vec<(char, f64)> {
        let entries = weights.entries();
        let pressed = entries
            .iter()
            .find(|&&(c, _)| c == typed)
            .map_or(entries[0].1, |&(_, p)| p);
        entries
            .iter()
            .filter(|&&(c, p)| c != typed && allowed(c) && p >= pressed * self.floor)
            .take(self.letters)
            .map(|&(c, p)| (c, f64::from(pressed / p).ln()))
            .collect()
    }
}
