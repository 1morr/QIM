//! 容錯（brief §4.8，M4 票 09）：詞庫詞與使用者詞的邊可以改輸入碼的一處再查，帶懲罰進 lattice；
//! 單字的邊永遠不修（速成幾乎每個兩碼都是合法的字，單字層分不出打錯）。
//!
//! 票 09 只給基準與實驗用（[`crate::Engine::with_tolerance`]），預設關、殼不接；開關、預設值與懲罰
//! 放哪裡（方案檔或 `Options`）由票 10 定。

use std::collections::HashMap;

use crate::Weights;

/// 各種修正的懲罰（ln 機率，正數；路徑分扣掉它）；`None` 是不做這種修正。預設全關。
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Tolerance {
    /// 漏碼：輸入碼少打了一碼，修正是在任一處補一個方案的鍵。
    pub omission: Option<f64>,
    /// 多碼：輸入碼多打了一碼，修正是刪掉一碼。
    pub insertion: Option<f64>,
    /// 兩碼對調：相鄰兩碼打反了。
    pub transposition: Option<f64>,
    /// 鄰鍵替換：一碼換成同一鍵的按鍵機率裡的另一個字母，另扣 `ln(機率最高的 ÷ 那個字母的)`；
    /// 沒有按鍵機率的碼不換。
    pub substitution: Option<f64>,
}

impl Tolerance {
    /// 四種修正都關。
    pub(crate) fn is_off(&self) -> bool {
        self.omission.is_none()
            && self.insertion.is_none()
            && self.transposition.is_none()
            && self.substitution.is_none()
    }
}

/// 打出的一段輸入碼 `typed`（只有方案的鍵）改一處後的碼與懲罰，不含 `typed` 本身；同一個碼有幾種
/// 改法時取懲罰最小的。`has_prefix` 回答有沒有以它開頭的詞的碼，用來剪枝：改出的碼一定是某個
/// 詞的碼的開頭。依懲罰、再依碼排序，結果固定。
pub(crate) fn corrected_codes(
    typed: &str,
    weights: &[Option<Weights>],
    alphabet: &str,
    tolerance: &Tolerance,
    has_prefix: impl Fn(&str) -> bool,
) -> Vec<(String, f64)> {
    let typed: Vec<char> = typed.chars().collect();
    let mut best: HashMap<String, f64> = HashMap::new();
    // 待走的（已對齊的打出的碼數, 改出的碼的開頭, 用掉的懲罰；還沒改是 `None`）。
    let mut todo: Vec<(usize, String, Option<f64>)> = vec![(0, String::new(), None)];
    while let Some((at, prefix, cost)) = todo.pop() {
        if cost.is_none()
            && let Some(penalty) = tolerance.omission
        {
            // 補一碼可以補在任一處，含最後。
            for key in alphabet.chars() {
                let mut longer = prefix.clone();
                longer.push(key);
                if has_prefix(&longer) {
                    todo.push((at, longer, Some(penalty)));
                }
            }
        }
        let Some(&next) = typed.get(at) else {
            if let Some(cost) = cost {
                let entry = best.entry(prefix).or_insert(f64::INFINITY);
                *entry = entry.min(cost);
            }
            continue;
        };
        let mut matched = prefix.clone();
        matched.push(next);
        if has_prefix(&matched) {
            todo.push((at + 1, matched, cost));
        }
        if cost.is_some() {
            continue;
        }
        if let Some(penalty) = tolerance.insertion {
            todo.push((at + 1, prefix.clone(), Some(penalty)));
        }
        if let Some(penalty) = tolerance.substitution
            && let Some(Some(key)) = weights.get(at)
        {
            let entries = key.entries();
            let pressed = entries
                .iter()
                .find(|&&(c, _)| c == next)
                .map_or(entries[0].1, |&(_, p)| p);
            for &(letter, p) in entries.iter().filter(|&&(c, _)| c != next) {
                let mut replaced = prefix.clone();
                replaced.push(letter);
                if has_prefix(&replaced) {
                    let cost = penalty + f64::from(pressed / p).ln();
                    todo.push((at + 1, replaced, Some(cost)));
                }
            }
        }
        if let Some(penalty) = tolerance.transposition
            && let Some(&after) = typed.get(at + 1)
            && after != next
        {
            let mut swapped = prefix.clone();
            swapped.push(after);
            swapped.push(next);
            if has_prefix(&swapped) {
                todo.push((at + 2, swapped, Some(penalty)));
            }
        }
    }
    let original: String = typed.iter().collect();
    let mut codes: Vec<(String, f64)> = best.into_iter().filter(|(c, _)| *c != original).collect();
    codes.sort_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
    codes
}
