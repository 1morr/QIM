//! 讀 libcangjie 的 `data/table.txt`；欄位定義見同目錄的 `README.table.md`。

use crate::Error;

/// `table.txt` 的一列。同一個字可能分在幾列，各列的碼不同。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub text: char,
    pub hanzi: bool,
    pub big5: bool,
    pub hkscs: bool,
    /// 倉頡三代碼，可能有幾個。
    pub cangjie3: Vec<String>,
    /// 倉頡五代碼，可能有幾個。
    pub cangjie5: Vec<String>,
    /// Big5 字依 Big5 順序的排序值；其他字為 0。
    pub order: u32,
}

const FIELDS: usize = 15;

/// 解析整份檔；空行與 `#` 開頭的註解行略過，其他列不合格式就回錯誤。
pub fn parse(text: &str) -> Result<Vec<Row>, Error> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.is_empty() && !line.starts_with('#'))
        .map(|(i, line)| {
            parse_row(line)
                .map_err(|reason| Error::invalid(format!("table.txt 第 {} 行", i + 1), reason))
        })
        .collect()
}

fn parse_row(line: &str) -> Result<Row, String> {
    let fields: Vec<&str> = line.split(' ').collect();
    if fields.len() != FIELDS {
        return Err(format!("應有 {FIELDS} 欄，實有 {} 欄", fields.len()));
    }
    let mut chars = fields[0].chars();
    let (Some(text), None) = (chars.next(), chars.next()) else {
        return Err(format!("第 1 欄應是一個字：{:?}", fields[0]));
    };
    let flag = |i: usize| match fields[i] {
        "0" => Ok(false),
        "1" => Ok(true),
        other => Err(format!("第 {} 欄應是 0 或 1：{other:?}", i + 1)),
    };
    let codes = |i: usize| -> Result<Vec<String>, String> {
        if fields[i] == "NA" {
            return Ok(Vec::new());
        }
        fields[i]
            .split(',')
            .map(|code| {
                if !code.is_empty() && code.bytes().all(|b| b.is_ascii_lowercase()) {
                    Ok(code.to_string())
                } else {
                    Err(format!("第 {} 欄的倉頡碼應是小寫字母：{code:?}", i + 1))
                }
            })
            .collect()
    };
    Ok(Row {
        text,
        hanzi: flag(2)?,
        big5: flag(3)?,
        hkscs: flag(4)?,
        cangjie3: codes(11)?,
        cangjie5: codes(12)?,
        order: fields[14]
            .parse()
            .map_err(|_| format!("第 15 欄應是整數：{:?}", fields[14]))?,
    })
}
