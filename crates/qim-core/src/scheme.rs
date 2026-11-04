use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// 輸入方案：方案檔（`data/schemes/*.toml`）由 `qim-databuild` 編進資料包，欄位見 plan §2.5。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scheme {
    pub id: String,
    pub name: String,
    pub version: String,
    /// 方案用到的鍵，依鍵盤標籤的順序；都是 ASCII 字母，輸入碼的位置就是位元組位置。
    pub alphabet: String,
    pub max_code_length: u8,
    /// 鍵 → 字根，組字與鍵盤標籤用。
    pub radicals: BTreeMap<char, String>,
    /// 方案要搭配的其他資料包。
    pub packs: Packs,
    pub ranking: Ranking,
}

/// 方案要搭配的其他資料包的 id（方案檔的 `[packs]`）。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Packs {
    /// 詞庫資料包（[`crate::LexiconPack`]）。
    pub lexicon: Option<String>,
}

/// 連打的打分參數（方案檔的 `[ranking]`，plan §2.3）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ranking {
    /// β：路徑裡每個詞庫的段加的分。
    pub word_bonus: f64,
    /// γ：路徑裡每一段扣的分。
    pub segment_penalty: f64,
    /// 每個位置保留的部分路徑數。
    pub beam: u16,
    /// 候選列最多放幾條多段路徑（拼出來的句子）；單一字詞蓋滿整串的不算在內。
    pub sentences: u16,
}

impl Default for Ranking {
    /// 測試資料用的中性值：β、γ 為 0，beam 20（libime 預設，`docs/research/libime-lattice.md`），
    /// 句子 3 條。實際方案的值寫在方案檔，由 `qim-databuild` 讀進來，不用這裡。
    fn default() -> Self {
        Self {
            word_bonus: 0.0,
            segment_penalty: 0.0,
            beam: 20,
            sentences: 3,
        }
    }
}
