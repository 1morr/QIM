use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

/// 手動分隔記號（brief〈桌面鍵盤操作〉）：組字中按它指定這裡一定要斷字：單字的碼不跨過它，詞只在它落在
/// 兩字的碼之間時跨過它（plan〈排序：同一把尺〉）。
/// 同 Rime 拼音方案 `speller/delimiter` 的 `'`；速成、倉頡的碼不用這個字元，所以各方案共用一個，
/// 不放進方案檔。它留在輸入碼裡，算在前一段的碼尾；查碼表、學習時去掉。
pub const SEPARATOR: char = '\'';

/// 輸入碼去掉分隔記號：查碼表、學習計數的碼（plan〈學習與排位〉）。
pub(crate) fn without_separators(input: &str) -> String {
    input.chars().filter(|&c| c != SEPARATOR).collect()
}

/// 輸入方案：方案檔（`data/schemes/*.toml`）由 `qim-databuild` 編進資料包，欄位見 plan〈輸入方案〉。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scheme {
    pub id: String,
    pub name: String,
    pub version: String,
    /// 方案用到的鍵，依鍵盤標籤的順序；都是 ASCII 字母，輸入碼的位置就是位元組位置。
    pub alphabet: String,
    pub max_code_length: u8,
    /// 萬用鍵（brief〈功能分級〉、plan〈輸入方案〉）：輸入碼裡代替任一個方案的鍵，每個剛好代一碼，放在碼的任一
    /// 位置都行；是碼表與詞庫的比對模式，不是容錯（Apple 倉頡的 `Z`、fcitx 表格的 `MatchingKey`）。
    /// 不在 `alphabet` 裡；`None` 時沒有萬用鍵，那個鍵照方案以外的鍵處理。
    pub wildcard: Option<char>,
    /// 鍵 → 字根，組字與鍵盤標籤用；可以有萬用鍵（組字顯示它的記號）。
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

    /// `c` 是組字收的碼：方案的鍵或萬用鍵。
    pub fn is_code_key(&self, c: char) -> bool {
        self.alphabet.contains(c) || self.wildcard == Some(c)
    }

    /// 輸入碼 `code` 有萬用鍵。
    pub(crate) fn has_wildcard(&self, code: &str) -> bool {
        self.wildcard.is_some_and(|w| code.contains(w))
    }

    /// `input` 是中文組字打得出的輸入碼：方案的鍵與萬用鍵，中間夾著分隔記號，不連著兩個（組字中
    /// 連按會吃下不動作）。臨時英文回中文組字時用；英文由組字的輸入碼起頭、只從尾端增刪，
    /// 所以不會以分隔記號開頭。
    pub(crate) fn composes(&self, input: &str) -> bool {
        !input.contains(&format!("{SEPARATOR}{SEPARATOR}"))
            && without_separators(input)
                .chars()
                .all(|c| self.is_code_key(c))
    }
}

/// 方案要搭配的其他資料包的 id（方案檔的 `[packs]`）。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Packs {
    /// 詞庫資料包（[`crate::LexiconPack`]）。
    pub lexicon: Option<String>,
    /// 語言模型資料包（[`crate::LanguageModelPack`]）；沒指名時打分只剩 β、γ。
    pub language_model: Option<String>,
    /// 詞級 trigram 資料包（[`crate::WordTrigramPack`]）；沒指名時打分只用字級 bigram。
    pub word_trigram: Option<String>,
    /// 字根提示資料包（[`crate::RadicalHintPack`]），只給殼顯示，[`crate::Engine`] 不載入。
    pub radical_hint: Option<String>,
}

/// 連打的打分參數（方案檔的 `[ranking]`，plan〈排序：同一把尺〉）。方案檔照它讀，欄位都必填、不收別的欄位。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

impl Ranking {
    /// 參數合不合用：beam 至少 1（0 時一條部分路徑都不留），β、γ 是有限的數。方案檔
    /// （`qim-databuild`）、`bench --set` 與 [`crate::Engine::new`] 都經這裡。
    pub fn check(&self) -> Result<(), RankingError> {
        if self.beam == 0 {
            return Err(RankingError::ZeroBeam);
        }
        for (field, value) in [
            ("word_bonus", self.word_bonus),
            ("segment_penalty", self.segment_penalty),
        ] {
            if !value.is_finite() {
                return Err(RankingError::NotFinite { field, value });
            }
        }
        Ok(())
    }
}

/// [`Ranking::check`] 不收的參數；訊息用方案檔 `[ranking]` 的欄位名。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RankingError {
    ZeroBeam,
    NotFinite { field: &'static str, value: f64 },
}

impl fmt::Display for RankingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroBeam => write!(f, "ranking.beam 至少是 1，給的是 0"),
            Self::NotFinite { field, value } => {
                write!(f, "ranking.{field} 要是有限的數，給的是 {value}")
            }
        }
    }
}

impl std::error::Error for RankingError {}
