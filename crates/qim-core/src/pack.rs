use std::fmt;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::{
    CodeTable, EmojiTable, EnglishWords, LanguageModel, Lexicon, RadicalHints, Scheme, Simplifier,
};

/// 資料包格式的版本，各種資料包共用；格式改了就加一，舊核心遇到新版本回 [`PackError::UnsupportedVersion`]。
pub const FORMAT_VERSION: u32 = 6;

/// 各種資料包共用的位元組格式：魔數 [`Pack::MAGIC`]、postcard 的 [`FORMAT_VERSION`]、
/// postcard 的本體。
pub trait Pack: Serialize + DeserializeOwned {
    /// 檔頭的魔數，每種資料包不同。
    const MAGIC: [u8; 4];

    /// 資料包的 id；方案資料包是方案的 id。
    fn id(&self) -> &str;

    fn to_bytes(&self) -> Result<Vec<u8>, PackError> {
        let mut bytes = Self::MAGIC.to_vec();
        bytes.extend(postcard::to_allocvec(&FORMAT_VERSION).map_err(malformed)?);
        bytes.extend(postcard::to_allocvec(self).map_err(malformed)?);
        Ok(bytes)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, PackError> {
        let rest = bytes
            .strip_prefix(&Self::MAGIC)
            .ok_or(PackError::NotAPack)?;
        let (found, rest) = postcard::take_from_bytes::<u32>(rest).map_err(malformed)?;
        if found != FORMAT_VERSION {
            return Err(PackError::UnsupportedVersion {
                found,
                supported: FORMAT_VERSION,
            });
        }
        let (body, rest) = postcard::take_from_bytes::<Self>(rest).map_err(malformed)?;
        if !rest.is_empty() {
            return Err(PackError::Malformed(format!(
                "結尾多了 {} 個位元組",
                rest.len()
            )));
        }
        Ok(body)
    }
}

/// 一個輸入方案的資料包：方案與它的碼表。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemePack {
    pub scheme: Scheme,
    pub code_table: CodeTable,
}

/// 詞庫資料包：詞、詞頻與字頻（plan §4.2）。方案用 [`crate::Packs::lexicon`] 指名它的 `id`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LexiconPack {
    pub id: String,
    pub lexicon: Lexicon,
}

/// 語言模型資料包：字級 bigram（plan §4.3）。方案用 [`crate::Packs::language_model`] 指名它的 `id`。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguageModelPack {
    pub id: String,
    pub model: LanguageModel,
}

/// 簡繁資料包：OpenCC 的轉換表（plan §4.4），由 [`crate::Engine::with_simplifier`] 掛上，
/// 不由方案指名：各方案都輸出繁體，簡體模式共用同一份。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimplifierPack {
    pub id: String,
    pub simplifier: Simplifier,
}

/// 字根提示資料包：字 → 倉頡三代 / 五代碼（plan §4.1 步驟 5）。方案用
/// [`crate::Packs::radical_hint`] 指名它的 `id`，殼用 [`RadicalHints::for_scheme`] 取出。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RadicalHintPack {
    pub id: String,
    pub hints: RadicalHints,
}

/// 英文資料包：中文狀態的英文候選用的詞表（plan §4.4），由 [`crate::Engine::with_english`]
/// 掛上，不由方案指名，各方案共用。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnglishPack {
    pub id: String,
    pub words: EnglishWords,
}

/// emoji 資料包：字詞 → emoji（plan §4.4），由 [`crate::Engine::with_emoji`] 掛上，不由方案指名，
/// 各方案共用。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmojiPack {
    pub id: String,
    pub emoji: EmojiTable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackError {
    /// 檔頭不是這種資料包的魔數。
    NotAPack,
    UnsupportedVersion {
        found: u32,
        supported: u32,
    },
    /// 版本對，但內容解不開、被截斷或有多餘的位元組。
    Malformed(String),
    /// 給的詞庫資料包不是方案指名的那份；`None` 是方案沒指名或沒給。
    LexiconMismatch {
        expected: Option<String>,
        found: Option<String>,
    },
    /// 給的語言模型資料包不是方案指名的那份；`None` 是方案沒指名或沒給。
    LanguageModelMismatch {
        expected: Option<String>,
        found: Option<String>,
    },
    /// 給的字根提示資料包不是方案指名的那份；`None` 是方案沒指名或沒給。
    RadicalHintMismatch {
        expected: Option<String>,
        found: Option<String>,
    },
}

impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = |id: &Option<String>| id.clone().unwrap_or_else(|| "（沒有）".into());
        match self {
            Self::NotAPack => write!(f, "不是這種 QIM 資料包"),
            Self::UnsupportedVersion { found, supported } => {
                write!(f, "資料包格式版本 {found}，這個核心只支援 {supported}")
            }
            Self::Malformed(reason) => write!(f, "資料包內容損壞：{reason}"),
            Self::LexiconMismatch { expected, found } => write!(
                f,
                "方案指名的詞庫是 {}，給的是 {}",
                name(expected),
                name(found)
            ),
            Self::LanguageModelMismatch { expected, found } => write!(
                f,
                "方案指名的語言模型是 {}，給的是 {}",
                name(expected),
                name(found)
            ),
            Self::RadicalHintMismatch { expected, found } => write!(
                f,
                "方案指名的字根提示是 {}，給的是 {}",
                name(expected),
                name(found)
            ),
        }
    }
}

impl std::error::Error for PackError {}

fn malformed(e: postcard::Error) -> PackError {
    PackError::Malformed(e.to_string())
}

impl Pack for SchemePack {
    const MAGIC: [u8; 4] = *b"QIMP";

    fn id(&self) -> &str {
        &self.scheme.id
    }
}

impl Pack for LexiconPack {
    const MAGIC: [u8; 4] = *b"QIML";

    fn id(&self) -> &str {
        &self.id
    }
}

impl Pack for LanguageModelPack {
    const MAGIC: [u8; 4] = *b"QIMB";

    fn id(&self) -> &str {
        &self.id
    }
}

impl Pack for SimplifierPack {
    const MAGIC: [u8; 4] = *b"QIMS";

    fn id(&self) -> &str {
        &self.id
    }
}

impl Pack for RadicalHintPack {
    const MAGIC: [u8; 4] = *b"QIMH";

    fn id(&self) -> &str {
        &self.id
    }
}

impl Pack for EnglishPack {
    const MAGIC: [u8; 4] = *b"QIME";

    fn id(&self) -> &str {
        &self.id
    }
}

impl Pack for EmojiPack {
    const MAGIC: [u8; 4] = *b"QIMJ";

    fn id(&self) -> &str {
        &self.id
    }
}
