//! 由倉頡碼推導速成碼表（plan §4.1 步驟 1–4、ADR 0003）。

use std::collections::{BTreeMap, BTreeSet};

use qim_core::{CodeEntry, CodeTable};

use crate::libcangjie::Row;

/// 速成碼：倉頡碼的首尾兩碼；1–2 碼的倉頡碼不變（brief §4.2、§12.1）。
pub fn quick_code(cangjie: &str) -> String {
    let mut letters = cangjie.chars();
    match (letters.next(), letters.next_back()) {
        (Some(first), Some(last)) => [first, last].iter().collect(),
        _ => cangjie.to_string(),
    }
}

/// 一個字合併所有列之後的資料。
struct Character {
    /// Big5 或 HKSCS 旗標為 1：連打字集。
    continuous: bool,
    /// 各列中非 0 的排序值取最小；沒有就是 0。
    order: u32,
    quick_codes: BTreeSet<String>,
}

/// 只收漢字；三代與五代的每個碼各自推導，取聯集。`grades` 是 Unihan `kGradeLevel`；
/// `simplified` 是簡體字（[`crate::simplified::characters`]），只標在連打字集的字上、不影響字序。
///
/// 每個速成碼的字序：連打字集在前、擴展區在後；同一區內有年級的字依年級排在前面，
/// 其餘與同年級的字依排序值，排序值為 0 的排在後面，最後依碼位，讓結果不受來源列序影響。
/// M1 的字頻接在年級之前（plan §4.1）。
pub fn code_table(
    rows: &[Row],
    grades: &BTreeMap<char, u8>,
    simplified: &BTreeSet<char>,
) -> CodeTable {
    let mut characters: BTreeMap<char, Character> = BTreeMap::new();
    for row in rows.iter().filter(|row| row.hanzi) {
        let c = characters.entry(row.text).or_insert(Character {
            continuous: false,
            order: 0,
            quick_codes: BTreeSet::new(),
        });
        c.continuous |= row.big5 || row.hkscs;
        if row.order != 0 && (c.order == 0 || row.order < c.order) {
            c.order = row.order;
        }
        c.quick_codes.extend(
            row.cangjie3
                .iter()
                .chain(&row.cangjie5)
                .map(|code| quick_code(code)),
        );
    }

    let mut by_code: BTreeMap<String, Vec<(char, &Character)>> = BTreeMap::new();
    for (&text, c) in &characters {
        for code in &c.quick_codes {
            by_code.entry(code.clone()).or_default().push((text, c));
        }
    }
    CodeTable::from_entries(by_code.into_iter().map(|(code, mut chars)| {
        chars.sort_by_key(|&(text, c)| {
            let grade = grades.get(&text);
            (
                !c.continuous,
                grade.is_none(),
                grade,
                c.order == 0,
                c.order,
                text,
            )
        });
        let entries = chars
            .into_iter()
            .map(|(text, c)| CodeEntry {
                text,
                extension: !c.continuous,
                simplified: c.continuous && simplified.contains(&text),
            })
            .collect();
        (code, entries)
    }))
}
