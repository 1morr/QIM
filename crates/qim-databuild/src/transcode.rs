//! 把句子機械轉成速成碼：首選字基準的語料與個人集共用（plan §5.2）。

use std::collections::HashMap;

use crate::libcangjie::Row;
use crate::quick::quick_code;

/// 連打字集每個字的一個速成碼。
pub struct Transcoder {
    codes: HashMap<char, String>,
}

impl Transcoder {
    /// 只收連打字集（Big5 或 HKSCS 旗標）的漢字，同 [`crate::quick::code_table`]。
    /// 每個字取第一個五代碼的速成碼；沒有五代碼才取第一個三代碼。「第一個」照 `table.txt` 的列序與逗號序。
    pub fn new(rows: &[Row]) -> Self {
        #[derive(Default)]
        struct Character<'a> {
            continuous: bool,
            cangjie5: Option<&'a String>,
            cangjie3: Option<&'a String>,
        }
        let mut characters: HashMap<char, Character> = HashMap::new();
        for row in rows.iter().filter(|r| r.hanzi) {
            let c = characters.entry(row.text).or_default();
            c.continuous |= row.big5 || row.hkscs;
            c.cangjie5 = c.cangjie5.or(row.cangjie5.first());
            c.cangjie3 = c.cangjie3.or(row.cangjie3.first());
        }
        let codes = characters
            .into_iter()
            .filter(|(_, c)| c.continuous)
            .filter_map(|(ch, c)| Some((ch, quick_code(c.cangjie5.or(c.cangjie3)?))))
            .collect();
        Self { codes }
    }

    /// 字的速成碼；不在連打字集就是 `None`。
    pub fn code(&self, ch: char) -> Option<&str> {
        self.codes.get(&ch).map(String::as_str)
    }

    /// 每個字一個碼；遇到第一個轉不了的字就回傳它。
    pub fn transcode(&self, text: &str) -> Result<Vec<&str>, char> {
        text.chars().map(|ch| self.code(ch).ok_or(ch)).collect()
    }
}
