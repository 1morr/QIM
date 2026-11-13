//! 字根提示資料包的內容（plan〈速成碼表〉 步驟 5）：字 → 倉頡三代 / 五代碼，只給提示用（ADR 0003）。

use std::collections::BTreeMap;

use qim_core::{CangjieCodes, RadicalHints};

use crate::libcangjie::Row;

/// 每個漢字合併所有列的碼，三代、五代各自依列序去重；兩代都沒有碼的字不收。
pub fn hints(rows: &[Row]) -> RadicalHints {
    let mut chars: BTreeMap<char, CangjieCodes> = BTreeMap::new();
    for row in rows.iter().filter(|row| row.hanzi) {
        let codes = chars.entry(row.text).or_default();
        for (all, new) in [
            (&mut codes.cangjie3, &row.cangjie3),
            (&mut codes.cangjie5, &row.cangjie5),
        ] {
            for code in new {
                if !all.contains(code) {
                    all.push(code.clone());
                }
            }
        }
    }
    chars.retain(|_, codes| !(codes.cangjie3.is_empty() && codes.cangjie5.is_empty()));
    RadicalHints::new(chars)
}
