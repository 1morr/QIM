use std::cmp::Reverse;

use crate::{CodeTable, Lexicon, LexiconPack, Pack, PackError, Scheme};

/// 不可變的方案與資料包，多個 [`crate::Session`] 共用。
#[derive(Debug, Clone)]
pub struct Engine {
    scheme: Scheme,
    /// 每個碼的字已依字頻重排（[`by_frequency`]）。
    code_table: CodeTable,
    /// 方案沒指名詞庫時是空的。
    lexicon: Lexicon,
}

impl Engine {
    /// `lexicon` 要剛好是方案 [`crate::Packs::lexicon`] 指名的那份；方案沒指名時要是 `None`。
    pub fn new(pack: Pack, lexicon: Option<LexiconPack>) -> Result<Self, PackError> {
        let found = lexicon.as_ref().map(|l| l.id.clone());
        if pack.scheme.packs.lexicon != found {
            return Err(PackError::LexiconMismatch {
                expected: pack.scheme.packs.lexicon,
                found,
            });
        }
        let lexicon = lexicon.map(|l| l.lexicon).unwrap_or_default();
        Ok(Self {
            code_table: by_frequency(&pack.code_table, &lexicon),
            scheme: pack.scheme,
            lexicon,
        })
    }

    /// 由資料包的位元組建立；讀檔是呼叫端的事（plan §1.3）。
    pub fn from_pack_bytes(pack: &[u8], lexicon: Option<&[u8]>) -> Result<Self, PackError> {
        let lexicon = lexicon.map(LexiconPack::from_bytes).transpose()?;
        Self::new(Pack::from_bytes(pack)?, lexicon)
    }

    pub fn scheme(&self) -> &Scheme {
        &self.scheme
    }

    /// 整串輸入碼的全部候選。暫時的合排（票 07 換成 plan §2.3 的打分）：碼表裡連打字集的字、
    /// 詞庫的詞（依建置時的詞頻順序）、擴展區的字。輸入碼超過方案的最長碼時不查碼表。
    pub fn candidates(&self, input: &str) -> Vec<String> {
        let chars = if input.chars().count() <= usize::from(self.scheme.max_code_length) {
            self.code_table.lookup(input)
        } else {
            &[]
        };
        let chars_where = |extension: bool| {
            chars
                .iter()
                .filter(move |e| e.extension == extension)
                .map(|e| e.text.to_string())
        };
        chars_where(false)
            .chain(self.lexicon.lookup(input).iter().map(|w| w.text.clone()))
            .chain(chars_where(true))
            .collect()
    }
}

/// 字序：字頻高的在前；字頻相同（含都沒有字頻）時照碼表原本的順序，也就是建置端排好的
/// 年級 → Big5（plan §4.1）。擴展區的字仍在連打字集之後。
fn by_frequency(code_table: &CodeTable, lexicon: &Lexicon) -> CodeTable {
    CodeTable::from_entries(code_table.iter().map(|(code, entries)| {
        let mut entries = entries.to_vec();
        entries.sort_by_key(|e| (e.extension, Reverse(lexicon.char_frequency(e.text))));
        (code.to_string(), entries)
    }))
}
