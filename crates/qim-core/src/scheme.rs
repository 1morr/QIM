use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// 手動分隔記號（brief §4.3）：組字中按它在目前位置斷開，連打的段不跨過它（plan §2.3）。
/// 同 Rime 拼音方案 `speller/delimiter` 的 `'`；速成、倉頡的碼不用這個字元，所以各方案共用一個，
/// 不放進方案檔。它留在輸入碼裡，算在前一段的碼尾；查碼表、學習時去掉。
pub const SEPARATOR: char = '\'';

/// 輸入碼去掉分隔記號：查碼表、學習計數的碼（plan §2.4）。
pub(crate) fn without_separators(input: &str) -> String {
    input.chars().filter(|&c| c != SEPARATOR).collect()
}

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

impl Scheme {
    /// 輸入碼的字根，如 `anr` →「日弓口」；方案沒有字根的鍵顯示原字母。
    pub fn radicals_of(&self, code: &str) -> String {
        code.chars()
            .map(|c| {
                self.radicals
                    .get(&c)
                    .map_or_else(|| c.to_string(), Clone::clone)
            })
            .collect()
    }

    /// `code` 的每個字元都是方案的鍵，可以當輸入碼（觸發詞、臨時英文回中文組字）。
    pub fn spells(&self, code: &str) -> bool {
        code.chars().all(|c| self.alphabet.contains(c))
    }

    /// `input` 是中文組字打得出的輸入碼：方案的鍵，中間夾著分隔記號；分隔記號不在開頭、
    /// 不連著兩個（組字中這樣按會吃下不動作）。臨時英文回中文組字時用。
    pub(crate) fn composes(&self, input: &str) -> bool {
        !input.starts_with(SEPARATOR)
            && !input.contains(&format!("{SEPARATOR}{SEPARATOR}"))
            && self.spells(&without_separators(input))
    }
}

/// 方案要搭配的其他資料包的 id（方案檔的 `[packs]`）。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Packs {
    /// 詞庫資料包（[`crate::LexiconPack`]）。
    pub lexicon: Option<String>,
    /// 語言模型資料包（[`crate::LanguageModelPack`]）；沒指名時打分只剩 β、γ。
    pub language_model: Option<String>,
    /// 字根提示資料包（[`crate::RadicalHintPack`]），只給殼顯示，[`crate::Engine`] 不載入。
    pub radical_hint: Option<String>,
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
