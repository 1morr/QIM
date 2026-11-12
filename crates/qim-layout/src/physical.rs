//! 實體鍵盤的鄰鍵先驗（plan §2.6，M4 票 07）：按一個實體鍵算成各字母的機率，殼交給核心當
//! `Key::Weighted`，讓容錯對觸控與實體鍵盤用同一套機制。
//!
//! 這是先驗，不是點擊位置：按下的鍵固定拿 [`PhysicalModel::hit_probability`]，剩下的依鍵中心的距離
//! 套 Gaussian 分給鄰鍵。鍵位照 `KeyboardEvent.code`（實體位置，名稱取自美式鍵盤上那個位置的字母，
//! W3C UI Events KeyboardEvent code Values），不照 `key`：換 Dvorak 等配列結果不變。各排錯位，所以
//! 上下排與斜角都算鄰鍵（libime 只取同排左右，`docs/research/ranking-techniques.md` §5.2）。

/// 實體鍵盤字母區的幾何：每排第一個鍵左緣的位置（鍵距為單位，1 鍵距約 19.05 mm）與該排的字母鍵，
/// 由左到右、排距 1 鍵距。ANSI 與 ISO 鍵盤的字母區一樣：前面是 Tab 1.5、CapsLock 1.75、左 Shift 2.25
/// （ISO 是 1.25 加 IntlBackslash 1）。只列字母鍵：數字、分號、逗號這類鍵不送字母，不算鄰鍵。
const ROWS: [(f64, &[&str]); 3] = [
    (
        1.5,
        &[
            "KeyQ", "KeyW", "KeyE", "KeyR", "KeyT", "KeyY", "KeyU", "KeyI", "KeyO", "KeyP",
        ],
    ),
    (
        1.75,
        &[
            "KeyA", "KeyS", "KeyD", "KeyF", "KeyG", "KeyH", "KeyJ", "KeyK", "KeyL",
        ],
    ),
    (
        2.25,
        &["KeyZ", "KeyX", "KeyC", "KeyV", "KeyB", "KeyN", "KeyM"],
    ),
];

/// 鄰鍵先驗的參數。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicalModel {
    /// 按下的鍵的字母機率；其餘分給鄰鍵（沒有鄰鍵時是 1）。
    pub hit_probability: f64,
    /// 鄰鍵之間怎麼分：依鍵中心的距離 d 取 `exp(-d² / 2σ²)` 的比例，σ 以鍵距為單位。
    pub sigma_keys: f64,
    /// 鍵中心在這個距離內（鍵距）的才算鄰鍵。
    pub radius_keys: f64,
    /// 最多回幾個字母。
    pub limit: usize,
}

impl Default for PhysicalModel {
    /// 初值都**沒有來源**，容錯的票在基準的 dev 切分上調：
    /// - `hit_probability` 0.9：按下的鍵是首選，打錯的先驗佔一成。
    /// - `sigma_keys` 0.5：F 的鄰鍵 D、G（1 鍵距）各約 0.023，R（1.03）0.020，C、V（1.12）0.014，T（1.25）0.007。
    /// - `radius_keys` 1.5：中間的鍵有 6 個鄰鍵（同排左右，上下排各二）都在 1.25 內，下一圈從 1.6 起。
    /// - `limit` 7：按下的鍵加 6 個鄰鍵。
    fn default() -> Self {
        Self {
            hit_probability: 0.9,
            sigma_keys: 0.5,
            radius_keys: 1.5,
            limit: 7,
        }
    }
}

impl PhysicalModel {
    /// 按下實體鍵 `code` 時各字母的機率，由高到低（按下的鍵在第一；一樣遠的鄰鍵照排序由上而下、由左而右），
    /// 最多 [`PhysicalModel::limit`] 個，和為 1。`code` 不是字母鍵（`KeyA`–`KeyZ`）時是 `None`。
    pub fn letter_weights(&self, code: &str) -> Option<Vec<(char, f32)>> {
        let keys = keys();
        let &(pressed, x, y) = keys.iter().find(|&&(c, _, _)| c == code)?;
        let mut neighbours: Vec<(char, f64)> = keys
            .iter()
            .filter(|&&(c, _, _)| c != pressed)
            .filter_map(|&(c, kx, ky)| {
                let distance = (kx - x).hypot(ky - y);
                (distance <= self.radius_keys).then(|| {
                    let weight = (-distance * distance / (2.0 * self.sigma_keys.powi(2))).exp();
                    (letter(c), weight)
                })
            })
            .collect();
        // 穩定排序：一樣遠時留著 `ROWS` 的順序。
        neighbours.sort_by(|a, b| b.1.total_cmp(&a.1));
        neighbours.truncate(self.limit.saturating_sub(1));
        let total: f64 = neighbours.iter().map(|&(_, w)| w).sum();
        let hit = if total > 0.0 {
            self.hit_probability
        } else {
            1.0
        };
        let mut weights = vec![(letter(pressed), hit as f32)];
        weights.extend(
            neighbours
                .into_iter()
                .map(|(c, w)| (c, ((1.0 - hit) * w / total) as f32))
                .filter(|&(_, p)| p > 0.0),
        );
        Some(weights)
    }
}

/// 字母鍵與鍵中心的座標（鍵距）。
fn keys() -> Vec<(&'static str, f64, f64)> {
    ROWS.iter()
        .zip(0u32..)
        .flat_map(|(&(offset, codes), row)| {
            codes
                .iter()
                .zip(0u32..)
                .map(move |(&code, i)| (code, offset + f64::from(i) + 0.5, f64::from(row)))
        })
        .collect()
}

/// `KeyF` → `f`：`ROWS` 只列 `Key` 加一個大寫字母的鍵。
fn letter(code: &str) -> char {
    let c = code.as_bytes()[3];
    char::from(c.to_ascii_lowercase())
}
