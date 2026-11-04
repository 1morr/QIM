//! `bench/baseline.json` 與 `bench/personal/baseline.json`：各集合的答錯數、修正次數與答錯的題號，
//! `--check` 拿它擋退步，翻盤清單拿它比（plan §5.2）。

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::score::SetResult;

/// 集合名 → 那個集合的上限。固定題不記：它一律要全過。
pub type Baseline = BTreeMap<String, SetBaseline>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetBaseline {
    pub wrong: usize,
    /// 修正次數的總和（每句平均乘題數）。
    pub correction_count: usize,
    /// 答錯的題的行號，寫成 `1-3,7` 這樣的區段，翻盤清單用。
    pub wrong_lines: String,
}

pub fn parse(text: &str) -> Result<Baseline, String> {
    let baseline: Baseline = serde_json::from_str(text).map_err(|e| e.to_string())?;
    for (name, set) in &baseline {
        parse_lines(&set.wrong_lines).map_err(|e| format!("集合 {name} 的 wrong_lines：{e}"))?;
    }
    Ok(baseline)
}

/// 這次各集合的結果，寫成 baseline 檔的內容。
pub fn render(sets: &[&SetResult]) -> String {
    let baseline: Baseline = sets
        .iter()
        .map(|set| {
            (
                set.name.clone(),
                SetBaseline {
                    wrong: set.wrong(),
                    correction_count: set.correction_count(),
                    wrong_lines: render_lines(&set.wrong_lines()),
                },
            )
        })
        .collect();
    let mut text = serde_json::to_string_pretty(&baseline).expect("baseline 一定序列化得了");
    text.push('\n');
    text
}

/// 和 baseline 比，翻盤的題號。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Flips {
    /// 由錯變對。
    pub fixed: Vec<usize>,
    /// 由對變錯。
    pub broken: Vec<usize>,
}

impl Flips {
    pub fn is_empty(&self) -> bool {
        self.fixed.is_empty() && self.broken.is_empty()
    }
}

/// `set` 和它的 baseline 比；只比兩邊都有的題（行號不超過這次的題數）。
pub fn flips(set: &SetResult, baseline: &SetBaseline) -> Flips {
    let before = parse_lines(&baseline.wrong_lines).expect("parse 時檢查過");
    let now = set.wrong_lines();
    let lines: BTreeSet<usize> = set.items.iter().map(|i| i.line).collect();
    Flips {
        fixed: before
            .difference(&now)
            .filter(|l| lines.contains(l))
            .copied()
            .collect(),
        broken: now.difference(&before).copied().collect(),
    }
}

/// 每個集合都要有 baseline，答錯數與修正次數都不高於它，而且每題都打得出；
/// baseline 有的集合這次也要跑到。回傳每一條違規，`file` 是 baseline 檔，訊息裡指給人看。
pub fn check(sets: &[&SetResult], baseline: &Baseline, file: &str) -> Vec<String> {
    let mut violations = Vec::new();
    for set in sets {
        let name = &set.name;
        let Some(limit) = baseline.get(name) else {
            violations.push(format!(
                "集合 {name} 在 {file} 沒有 baseline：確認結果後跑 `cargo xtask bench --update-baseline`"
            ));
            continue;
        };
        if set.wrong() > limit.wrong {
            violations.push(format!(
                "集合 {name} 答錯 {} 題，高於 baseline 的 {} 題",
                set.wrong(),
                limit.wrong
            ));
        }
        if set.correction_count() > limit.correction_count {
            violations.push(format!(
                "集合 {name} 修正 {} 次，高於 baseline 的 {} 次",
                set.correction_count(),
                limit.correction_count
            ));
        }
        if set.unfinished() > 0 {
            violations.push(format!(
                "集合 {name} 有 {} 題一個字一個字打也打不出：核心查不到該字自己的碼，或選字後的行為變了",
                set.unfinished()
            ));
        }
    }
    for name in baseline.keys() {
        if !sets.iter().any(|set| &set.name == name) {
            violations.push(format!(
                "{file} 有集合 {name}，但這次沒有跑到：語料檔不見了，或要用 `--update-baseline` 拿掉它"
            ));
        }
    }
    violations
}

fn render_lines(lines: &BTreeSet<usize>) -> String {
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    for &line in lines {
        match ranges.last_mut() {
            Some((_, end)) if *end + 1 == line => *end = line,
            _ => ranges.push((line, line)),
        }
    }
    ranges
        .iter()
        .map(|&(start, end)| {
            if start == end {
                start.to_string()
            } else {
                format!("{start}-{end}")
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn parse_lines(text: &str) -> Result<BTreeSet<usize>, String> {
    let mut lines = BTreeSet::new();
    if text.is_empty() {
        return Ok(lines);
    }
    for part in text.split(',') {
        let number = |s: &str| s.parse::<usize>().map_err(|_| format!("看不懂「{part}」"));
        let (start, end) = match part.split_once('-') {
            Some((start, end)) => (number(start)?, number(end)?),
            None => (number(part)?, number(part)?),
        };
        if start > end {
            return Err(format!("區段「{part}」的開頭大於結尾"));
        }
        lines.extend(start..=end);
    }
    Ok(lines)
}
