//! 實體鍵盤的鄰鍵先驗（plan〈佈局：畫布模型〉，M4 票 07）：按一個實體鍵算成各字母的機率，殼交給核心當
//! `Key::Physical`（核心照機率最高的字母打，不列鄰鍵讀法，M4 票 09c）。
//!
//! 這是先驗，不是點擊位置：按下的鍵固定拿 [`PhysicalModel::hit_probability`]，剩下的依鍵中心的距離
//! 套 Gaussian 分給鄰鍵。鍵位照 `KeyboardEvent.code`（實體位置，名稱取自美式鍵盤上那個位置的字母，
//! W3C UI Events KeyboardEvent code Values），不照 `key`：換 Dvorak 等配列結果不變。各排錯位，所以
//! 上下排與斜角都算鄰鍵（libime 只取同排左右，`docs/research/ranking-techniques.md` §5.2）。

/// 實體鍵盤字母區的幾何：每排第一個鍵左緣的位置（鍵距為單位，1 鍵距約 19.05 mm）與該排各鍵的 `code`
/// 字母（`KeyQ` 的 `q`…），由左到右、排距 1 鍵距。ANSI 與 ISO 鍵盤的字母區一樣：前面是 Tab 1.5、
/// CapsLock 1.75、左 Shift 2.25（ISO 是 1.25 加 IntlBackslash 1）。只列字母鍵：數字、分號、逗號這類鍵
/// 不送字母，不算鄰鍵。
const ROWS: [(f64, &str); 3] = [(1.5, "qwertyuiop"), (1.75, "asdfghjkl"), (2.25, "zxcvbnm")];

/// 鄰鍵先驗的參數。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicalModel {
    /// 按下的鍵的字母機率；其餘分給鄰鍵（沒有鄰鍵時是 1）。
    pub hit_probability: f64,
    /// 鄰鍵之間怎麼分：依鍵中心的距離 d 取 `exp(-d² / 2σ²)` 的比例，σ 以鍵距為單位。
    pub sigma_keys: f64,
    /// 鍵中心在這個距離內（鍵距）的才算鄰鍵。
    pub radius_keys: f64,
}

impl Default for PhysicalModel {
    /// 初值都**沒有來源**，還沒在基準上調過：
    /// - `hit_probability` 0.9：按下的鍵是首選，打錯的先驗佔一成。
    /// - `sigma_keys` 0.5：F 的鄰鍵 D、G（1 鍵距）各約 0.023，R（1.03）0.020，C、V（1.12）0.014，T（1.25）0.007。
    /// - `radius_keys` 1.5：中間的鍵有 6 個鄰鍵（同排左右，上下排各二）都在 1.25 內，下一圈從 1.6 起；
    ///   所以最多回 7 個字母。
    fn default() -> Self {
        Self {
            hit_probability: 0.9,
            sigma_keys: 0.5,
            radius_keys: 1.5,
        }
    }
}

impl PhysicalModel {
    /// 按下實體鍵 `code` 時各字母的機率，由高到低（按下的鍵在第一；一樣遠的鄰鍵照排序由上而下、由左而右），
    /// 和為 1。`code` 不是字母鍵（`KeyA`–`KeyZ`）時是 `None`。
    pub fn letter_weights(&self, code: &str) -> Option<Vec<(char, f32)>> {
        let pressed = code_letter(code)?;
        let keys = letter_keys();
        let &(_, x, y) = keys.iter().find(|&&(c, _, _)| c == pressed)?;
        let mut neighbours: Vec<(char, f64)> = keys
            .iter()
            .filter(|&&(c, _, _)| c != pressed)
            .filter_map(|&(c, kx, ky)| {
                let distance = (kx - x).hypot(ky - y);
                (distance <= self.radius_keys).then(|| {
                    let weight = (-distance * distance / (2.0 * self.sigma_keys.powi(2))).exp();
                    (c, weight)
                })
            })
            .collect();
        // 穩定排序：一樣遠時留著 `ROWS` 的順序。
        neighbours.sort_by(|a, b| b.1.total_cmp(&a.1));
        let total: f64 = neighbours.iter().map(|&(_, w)| w).sum();
        let hit = if total > 0.0 {
            self.hit_probability
        } else {
            1.0
        };
        let mut weights = vec![(pressed, hit as f32)];
        weights.extend(
            neighbours
                .into_iter()
                .map(|(c, w)| (c, ((1.0 - hit) * w / total) as f32))
                .filter(|&(_, p)| p > 0.0),
        );
        Some(weights)
    }
}

/// 實體鍵盤的字母鍵（`code` 的字母）與鍵中心的座標：鍵距為單位，x 從 Tab 鍵左緣往右、y 從字母區第一排
/// 往下。鄰鍵先驗用它；打錯碼的基準（M4 票 08）也照它抽鄰鍵，不另寫一份 QWERTY。
pub fn letter_keys() -> Vec<(char, f64, f64)> {
    ROWS.iter()
        .zip(0u32..)
        .flat_map(|(&(offset, letters), row)| {
            letters
                .chars()
                .zip(0u32..)
                .map(move |(c, i)| (c, offset + f64::from(i) + 0.5, f64::from(row)))
        })
        .collect()
}

/// `KeyF` → `f`；不是 `Key` 加一個大寫英文字母時是 `None`。
fn code_letter(code: &str) -> Option<char> {
    let mut rest = code.strip_prefix("Key")?.chars();
    match (rest.next(), rest.next()) {
        (Some(c), None) if c.is_ascii_uppercase() => Some(c.to_ascii_lowercase()),
        _ => None,
    }
}
