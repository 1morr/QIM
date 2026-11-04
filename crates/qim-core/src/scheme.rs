use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// 輸入方案：方案檔（`data/schemes/*.toml`）由 `qim-databuild` 編進資料包，欄位見 plan §2.5。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scheme {
    pub id: String,
    pub name: String,
    pub version: String,
    /// 方案用到的鍵，依鍵盤標籤的順序。
    pub alphabet: String,
    pub max_code_length: u8,
    /// 鍵 → 字根，組字與鍵盤標籤用。
    pub radicals: BTreeMap<char, String>,
    /// 方案要搭配的其他資料包。
    pub packs: Packs,
}

/// 方案要搭配的其他資料包的 id（方案檔的 `[packs]`）。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Packs {
    /// 詞庫資料包（[`crate::LexiconPack`]）。
    pub lexicon: Option<String>,
}
