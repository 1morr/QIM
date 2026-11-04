//! `bench/baseline.json`：各語料集合的答錯數，`--check` 拿它擋退步（plan §5.2）。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::score::SetResult;

/// 集合名 → 那個集合的上限。固定題不記：它一律要全過。
pub type Baseline = BTreeMap<String, SetBaseline>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetBaseline {
    pub wrong: usize,
}

pub fn parse(text: &str) -> Result<Baseline, String> {
    serde_json::from_str(text).map_err(|e| e.to_string())
}

/// 這次各集合的答錯數，寫成 `baseline.json` 的內容。
pub fn render(sets: &[SetResult]) -> String {
    let baseline: Baseline = sets
        .iter()
        .map(|set| (set.name.clone(), SetBaseline { wrong: set.wrong() }))
        .collect();
    let mut text = serde_json::to_string_pretty(&baseline).expect("baseline 一定序列化得了");
    text.push('\n');
    text
}

/// 固定題全過，且每個集合都有 baseline、答錯數不高於它；回傳每一條違規。
pub fn check(fixed: &SetResult, sets: &[SetResult], baseline: &Baseline) -> Vec<String> {
    let mut violations: Vec<String> = fixed
        .failures
        .iter()
        .map(|f| {
            format!(
                "固定題沒過：{}（{}）期望「{}」，首選「{}」",
                f.item.input, f.item.label, f.item.expected, f.actual
            )
        })
        .collect();
    for set in sets {
        match baseline.get(&set.name) {
            None => violations.push(format!(
                "集合 {} 沒有 baseline：確認結果後跑 `cargo xtask bench --update-baseline`",
                set.name
            )),
            Some(limit) if set.wrong() > limit.wrong => violations.push(format!(
                "集合 {} 答錯 {} 題，高於 baseline 的 {} 題",
                set.name,
                set.wrong(),
                limit.wrong
            )),
            Some(_) => {}
        }
    }
    for name in baseline.keys() {
        if !sets.iter().any(|set| &set.name == name) {
            violations.push(format!(
                "baseline 有集合 {name}，但這次沒有跑到：語料檔不見了，或要用 `--update-baseline` 拿掉它"
            ));
        }
    }
    violations
}
