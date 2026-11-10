//! Unihan 的兩個用途：`kGradeLevel` 給速成碼表排字序（plan §4.1）；`kCangjie` 與
//! libcangjie 倉頡三代碼交叉比對（plan §4.5），只出報表，不擋建置。
//!
//! 研究顯示 `kCangjie` 對應三代（`docs/research/data-sources.md`），所以只和第 12 欄比。

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::io::Read;

use crate::Error;
use crate::libcangjie::Row;

/// `Unihan.zip` 裡放 `kCangjie` 與 `kGradeLevel` 的檔。
pub const ARCHIVE_ENTRY: &str = "Unihan_DictionaryLikeData.txt";

/// 從 `Unihan.zip` 的位元組取出 [`ARCHIVE_ENTRY`] 的內容。
pub fn read_archive(zip: &[u8]) -> Result<String, Error> {
    let invalid = |e: &dyn std::fmt::Display| Error::invalid("Unihan.zip", e.to_string());
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip)).map_err(|e| invalid(&e))?;
    let mut file = archive.by_name(ARCHIVE_ENTRY).map_err(|e| invalid(&e))?;
    let mut text = String::new();
    file.read_to_string(&mut text).map_err(|e| invalid(&e))?;
    Ok(text)
}

/// 取出 `kCangjie` 欄：字到小寫的倉頡碼。`kCangjie` 的值是一串大寫字母（UAX #38）。
pub fn cangjie_codes(text: &str) -> Result<BTreeMap<char, String>, Error> {
    field(text, "kCangjie", |v| {
        (!v.is_empty() && v.bytes().all(|b| b.is_ascii_uppercase()))
            .then(|| v.to_ascii_lowercase())
            .ok_or("應是一串大寫字母")
    })
}

/// 取出 `kGradeLevel` 欄：字到香港小學的年級 1–6（UAX #38，出自《朗文初級中文詞典》2001）。
pub fn grade_levels(text: &str) -> Result<BTreeMap<char, u8>, Error> {
    field(text, "kGradeLevel", |v| match v.as_bytes() {
        [d @ b'1'..=b'6'] => Ok(d - b'0'),
        _ => Err("應是 1–6"),
    })
}

/// 取出 `kHKGlyph` 欄：字在香港《常用字字形表》（2000 年修訂本）的序號（UAX #38）。同一條列了
/// 幾個寫法時它們共用一個序號。
pub fn hk_glyphs(text: &str) -> Result<BTreeMap<char, u16>, Error> {
    field(text, "kHKGlyph", |v| {
        (v.len() == 4 && v.bytes().all(|b| b.is_ascii_digit()))
            .then(|| v.parse().ok())
            .flatten()
            .ok_or("應是四位數字")
    })
}

/// 取出一個欄位。格式見 UAX #38：`U+XXXX<tab>欄位<tab>值`，`#` 開頭是註解。
fn field<T>(
    text: &str,
    name: &str,
    value_of: impl Fn(&str) -> Result<T, &'static str>,
) -> Result<BTreeMap<char, T>, Error> {
    let mut out = BTreeMap::new();
    for (i, line) in text.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.split('\t');
        let (codepoint, field, value) = (fields.next(), fields.next(), fields.next());
        if field != Some(name) {
            continue;
        }
        let invalid =
            |reason: String| Error::invalid(format!("{ARCHIVE_ENTRY} 第 {} 行", i + 1), reason);
        let text = codepoint
            .and_then(|c| c.strip_prefix("U+"))
            .and_then(|hex| u32::from_str_radix(hex, 16).ok())
            .and_then(char::from_u32)
            .ok_or_else(|| invalid(format!("碼位不合格式：{codepoint:?}")))?;
        let value = match (value, fields.next()) {
            (Some(v), None) => value_of(v).map_err(|e| invalid(format!("{name} {e}：{line:?}")))?,
            _ => return Err(invalid(format!("{name} 應是一個值：{line:?}"))),
        };
        out.insert(text, value);
    }
    Ok(out)
}

/// 兩邊都有碼、但 Unihan 的碼不在 libcangjie 的三代碼裡的字。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Difference {
    pub text: char,
    /// 小寫。
    pub unihan: String,
    /// libcangjie 這個字所有列的三代碼，依字母序。
    pub cangjie3: Vec<String>,
    /// libcangjie 這個字所有列的五代碼，依字母序；只當參考，不參與比對。
    pub cangjie5: Vec<String>,
}

impl Difference {
    /// Unihan 的碼等於某個五代碼：多半是三代、五代的版本差異，不是錯碼。
    pub fn matches_cangjie5(&self) -> bool {
        self.cangjie5.contains(&self.unihan)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossCheck {
    /// 兩邊都有碼的字數。
    pub compared: usize,
    /// 依碼位排序。
    pub differences: Vec<Difference>,
}

/// `kcangjie` 是字到小寫倉頡碼。Unihan 的碼是 libcangjie 該字任一個三代碼就算相同；
/// libcangjie 沒有三代碼的字不比。
pub fn compare(rows: &[Row], kcangjie: &BTreeMap<char, String>) -> CrossCheck {
    #[derive(Default)]
    struct Codes<'a> {
        cangjie3: BTreeSet<&'a str>,
        cangjie5: BTreeSet<&'a str>,
    }
    let mut by_char: BTreeMap<char, Codes> = BTreeMap::new();
    for row in rows {
        let codes = by_char.entry(row.text).or_default();
        codes
            .cangjie3
            .extend(row.cangjie3.iter().map(String::as_str));
        codes
            .cangjie5
            .extend(row.cangjie5.iter().map(String::as_str));
    }
    let owned = |set: &BTreeSet<&str>| set.iter().map(|c| c.to_string()).collect();
    let mut compared = 0;
    let mut differences = Vec::new();
    for (&text, unihan) in kcangjie {
        let Some(codes) = by_char.get(&text).filter(|c| !c.cangjie3.is_empty()) else {
            continue;
        };
        compared += 1;
        if !codes.cangjie3.contains(unihan.as_str()) {
            differences.push(Difference {
                text,
                unihan: unihan.clone(),
                cangjie3: owned(&codes.cangjie3),
                cangjie5: owned(&codes.cangjie5),
            });
        }
    }
    CrossCheck {
        compared,
        differences,
    }
}

impl CrossCheck {
    /// 差異中 Unihan 的碼等於 libcangjie 五代碼的字數。
    pub fn matching_cangjie5(&self) -> usize {
        self.differences
            .iter()
            .filter(|d| d.matches_cangjie5())
            .count()
    }

    /// 差異報表：第一行是欄名，之後每個差異一行，欄位以 tab 分隔，依碼位排序。
    /// 一個字有幾個碼時以逗號分隔（同 `table.txt`），沒有五代碼時該欄留空。
    /// 格式見 `docs/development.md`〈資料建置〉。
    pub fn to_tsv(&self) -> String {
        let mut out = String::from(
            "codepoint\tcharacter\tunihan\tlibcangjie_cangjie3\tlibcangjie_cangjie5\n",
        );
        for d in &self.differences {
            writeln!(
                out,
                "U+{:04X}\t{}\t{}\t{}\t{}",
                u32::from(d.text),
                d.text,
                d.unihan,
                d.cangjie3.join(","),
                d.cangjie5.join(",")
            )
            .expect("寫進 String 不會失敗");
        }
        out
    }
}
