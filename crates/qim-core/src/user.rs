//! 個人模型（plan §2.4）：學習計數。只在記憶體；存檔與匯出匯入在 M1 票 14。

use std::collections::BTreeMap;

/// 衰減的時間常數，以上屏次數計（librime `formula_d`，brief §12.2）。
const DECAY_STEP: f64 = 200.0;

/// 個人機率在 `personal` 裡的權重 w（plan §2.3 的初值，libime `UserLanguageModel`）。
const PERSONAL_WEIGHT: f64 = 0.2;

/// 排位規則 2 的期限：距最後上屏不超過這麼多次上屏（plan §2.4 的初值）。
pub(crate) const RECENT_COMMITS: u64 = 10_000;

/// 使用者詞與學習計數；殼負責保存（plan §2.1）。各輸入方案分開記，互不影響（plan §2.4）。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UserModel {
    schemes: BTreeMap<String, SchemeModel>,
}

/// 一個輸入方案的學習計數。
#[derive(Debug, Clone, Default, PartialEq)]
struct SchemeModel {
    /// 這個方案學習過的上屏次數；學習計數的時間（plan §2.4），不讀系統時間。
    tick: u64,
    /// 輸入碼 → 候選文字 → 計數。
    entries: BTreeMap<String, BTreeMap<String, Count>>,
}

/// 一個（輸入方案, 輸入碼, 候選文字）的計數。兩種上屏分開記（plan §2.4）：
/// - 整串上屏：整串輸入碼就是這個碼、上屏的就是這個候選；給排位規則 2 用。
/// - 句中各段的上屏也算進衰減值，給 `personal` 用。
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Count {
    /// 整串上屏的次數，不衰減，排位規則 2 的門檻用；只在句中出現過時是 0。
    commits: u64,
    /// 最後一次整串上屏的 tick；`commits` 是 0 時沒有意義。
    last_commit_tick: u64,
    /// 衰減值：整串上屏與句中各段的上屏每次加一，舊的隨上屏次數指數衰減（librime
    /// `formula_d`）；記的是 `last_tick` 當時的值。
    decayed: f64,
    /// 最後一次（整串或句中）上屏時的 tick。
    last_tick: u64,
}

impl UserModel {
    /// 一次上屏：整串輸入碼和每一段（輸入碼, 字詞）的衰減值各記一次，同一個鍵只記一次；
    /// 整串另記一次整串上屏（plan §2.4）。`segments` 依輸入碼的順序、接起來是整串；空的不記。
    pub(crate) fn learn(&mut self, scheme: &str, segments: &[(&str, &str)]) {
        if segments.is_empty() {
            return;
        }
        let model = self.schemes.entry(scheme.to_string()).or_default();
        model.tick += 1;
        let tick = model.tick;
        let whole: (String, String) = (
            segments.iter().map(|s| s.0).collect(),
            segments.iter().map(|s| s.1).collect(),
        );
        let mut keys: Vec<(&str, &str)> = vec![(&whole.0, &whole.1)];
        for &segment in segments {
            if !keys.contains(&segment) {
                keys.push(segment);
            }
        }
        for (i, (code, text)) in keys.into_iter().enumerate() {
            let count = model
                .entries
                .entry(code.to_string())
                .or_default()
                .entry(text.to_string())
                .or_default();
            count.decayed = 1.0 + count.decayed * decay(tick - count.last_tick);
            count.last_tick = tick;
            // `keys` 的第一個是整串。
            if i == 0 {
                count.commits += 1;
                count.last_commit_tick = tick;
            }
        }
    }

    /// 這組（方案, 碼, 字詞）學過沒有。
    pub(crate) fn has_learned(&self, scheme: &str, code: &str, text: &str) -> bool {
        self.get(scheme, code, text).is_some()
    }

    /// 排位規則 2（plan §2.4）：整串上屏 ≥ 2 次且距最後一次整串上屏不超過 [`RECENT_COMMITS`]
    /// 次上屏時，回傳那次的 tick，越大越近；否則是 `None`。句中各段的上屏不算。
    pub(crate) fn last_tick_if_chosen_twice(
        &self,
        scheme: &str,
        code: &str,
        text: &str,
    ) -> Option<u64> {
        let (model, count) = self.get(scheme, code, text)?;
        (count.commits >= 2 && model.tick - count.last_commit_tick <= RECENT_COMMITS)
            .then_some(count.last_commit_tick)
    }

    /// `personal`（plan §2.3）：學過的（輸入碼, 字詞）在任何長度的連打裡加的分，只加不減。
    /// `log_static` 是這一段原本的 ln 機率；沒學過時是 0。
    pub(crate) fn personal(&self, scheme: &str, code: &str, text: &str, log_static: f64) -> f64 {
        let Some((model, count)) = self.get(scheme, code, text) else {
            return 0.0;
        };
        let p = count.decayed * decay(model.tick - count.last_tick) / max_decayed();
        let mixed = ((1.0 - PERSONAL_WEIGHT) * log_static.exp() + PERSONAL_WEIGHT * p).ln();
        (mixed - log_static).max(0.0)
    }

    fn get(&self, scheme: &str, code: &str, text: &str) -> Option<(&SchemeModel, &Count)> {
        let model = self.schemes.get(scheme)?;
        Some((model, model.entries.get(code)?.get(text)?))
    }
}

/// 衰減值的上限：每次上屏都是同一個（輸入碼, 字詞）時趨近的值，`1 / (1 − e^(−1/200))` ≈ 200.5。
/// 個人機率是衰減值除以它（libime 的歷史機率也除以滿載的總量，`docs/research/libime-user-model.md` §2）。
fn max_decayed() -> f64 {
    1.0 / (1.0 - decay(1))
}

/// 經過 `steps` 次上屏後剩下的比例。
fn decay(steps: u64) -> f64 {
    (-(steps as f64) / DECAY_STEP).exp()
}
