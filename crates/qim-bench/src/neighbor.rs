//! 鄰鍵讀法的兩條閘門（M4 票 09c，`--check`）：
//! - 觸控模擬的集合：開著的修正次數不能比關著的多（[`touch_violation`]）。
//! - 照實體鍵送的集合（固定題、語料、有前文的題、含打錯碼）：開關前後逐題相同，因為實體鍵盤不換讀法
//!   （[`physical_violation`]）。

use crate::score::SetResult;

/// 不同的題最多列幾個題號。
const LINES_SHOWN: usize = 5;

/// 觸控集合 `set` 開著鄰鍵讀法的修正次數比關著的多時的違規；沒有關著的結果（不是觸控集合）時沒有違規。
pub fn touch_violation(set: &SetResult) -> Option<String> {
    let off = set.neighbor_off?;
    (set.correction_count() > off.correction_count).then(|| {
        format!(
            "集合 {} 鄰鍵讀法開著修正 {} 次，比關著的 {} 次多",
            set.name,
            set.correction_count(),
            off.correction_count
        )
    })
}

/// 照實體鍵送的集合 `on`（預設選項）與 `off`（鄰鍵讀法關掉）逐題比整個 [`crate::ItemResult`]；有不同時的違規，
/// 列出前幾個題號。
pub fn physical_violation(name: &str, on: &SetResult, off: &SetResult) -> Option<String> {
    let differing: Vec<String> = if on.items.len() == off.items.len() {
        on.items
            .iter()
            .zip(&off.items)
            .filter(|(a, b)| a != b)
            .map(|(a, _)| a.line.to_string())
            .collect()
    } else {
        // 題數不同不可能發生（同一批題），發生了就整個集合算不同。
        vec!["（題數不同）".into()]
    };
    if differing.is_empty() {
        return None;
    }
    let shown = differing
        .iter()
        .take(LINES_SHOWN)
        .cloned()
        .collect::<Vec<_>>()
        .join("、");
    Some(format!(
        "集合 {name} 照實體鍵送，鄰鍵讀法開關前後有 {} 題不同（實體鍵盤不該換讀法）：第 {shown} 題{}",
        differing.len(),
        if differing.len() > LINES_SHOWN {
            "…"
        } else {
            ""
        }
    ))
}
