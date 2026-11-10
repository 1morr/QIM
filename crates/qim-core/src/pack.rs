use std::fmt;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::{CodeTable, EnglishWords, LanguageModel, Lexicon, RadicalHints, Scheme, Simplifier};

/// 資料包格式的版本，各種資料包共用；格式改了就加一，舊核心遇到新版本回 [`PackError::UnsupportedVersion`]。
pub const FORMAT_VERSION: u32 = 5;

/// 輸入方案資料包檔頭的魔數。
const SCHEME_MAGIC: &[u8; 4] = b"QIMP";
/// 詞庫資料包檔頭的魔數。
const LEXICON_MAGIC: &[u8; 4] = b"QIML";
/// 語言模型資料包檔頭的魔數。
const LANGUAGE_MODEL_MAGIC: &[u8; 4] = b"QIMB";
/// 簡繁資料包檔頭的魔數。
const SIMPLIFIER_MAGIC: &[u8; 4] = b"QIMS";
/// 字根提示資料包檔頭的魔數。
const RADICAL_HINT_MAGIC: &[u8; 4] = b"QIMH";
/// 英文資料包檔頭的魔數。
const ENGLISH_MAGIC: &[u8; 4] = b"QIME";

/// 一個輸入方案的資料包：方案與它的碼表。
///
/// 位元組格式：魔數 `QIMP`、postcard 的 [`FORMAT_VERSION`]、postcard 的本體。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemePack {
    pub scheme: Scheme,
    pub code_table: CodeTable,
}

/// 詞庫資料包：詞、詞頻與字頻（plan §4.2）。方案用 [`crate::Packs::lexicon`] 指名它的 `id`。
///
/// 位元組格式：魔數 `QIML`、postcard 的 [`FORMAT_VERSION`]、postcard 的本體。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LexiconPack {
    pub id: String,
    pub lexicon: Lexicon,
}

/// 語言模型資料包：字級 bigram（plan §4.3）。方案用 [`crate::Packs::language_model`] 指名它的 `id`。
///
/// 位元組格式：魔數 `QIMB`、postcard 的 [`FORMAT_VERSION`]、postcard 的本體。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguageModelPack {
    pub id: String,
    pub model: LanguageModel,
}

/// 簡繁資料包：OpenCC 的轉換表（plan §4.4），由 [`crate::Engine::with_simplifier`] 掛上，
/// 不由方案指名：各方案都輸出繁體，簡體模式共用同一份。
///
/// 位元組格式：魔數 `QIMS`、postcard 的 [`FORMAT_VERSION`]、postcard 的本體。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimplifierPack {
    pub id: String,
    pub simplifier: Simplifier,
}

/// 字根提示資料包：字 → 倉頡三代 / 五代碼（plan §4.1 步驟 5）。方案用
/// [`crate::Packs::radical_hint`] 指名它的 `id`，殼用 [`RadicalHints::for_scheme`] 取出。
///
/// 位元組格式：魔數 `QIMH`、postcard 的 [`FORMAT_VERSION`]、postcard 的本體。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RadicalHintPack {
    pub id: String,
    pub hints: RadicalHints,
}

/// 英文資料包：中文狀態的英文候選用的詞表（plan §4.4），由 [`crate::Engine::with_english`]
/// 掛上，不由方案指名，各方案共用。
///
/// 位元組格式：魔數 `QIME`、postcard 的 [`FORMAT_VERSION`]、postcard 的本體。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnglishPack {
    pub id: String,
    pub words: EnglishWords,
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

fn encode(magic: &[u8; 4], body: &impl Serialize) -> Result<Vec<u8>, PackError> {
    let mut bytes = magic.to_vec();
    bytes.extend(postcard::to_allocvec(&FORMAT_VERSION).map_err(malformed)?);
    bytes.extend(postcard::to_allocvec(body).map_err(malformed)?);
    Ok(bytes)
}

fn decode<T: DeserializeOwned>(magic: &[u8; 4], bytes: &[u8]) -> Result<T, PackError> {
    let rest = bytes.strip_prefix(magic).ok_or(PackError::NotAPack)?;
    let (found, rest) = postcard::take_from_bytes::<u32>(rest).map_err(malformed)?;
    if found != FORMAT_VERSION {
        return Err(PackError::UnsupportedVersion {
            found,
            supported: FORMAT_VERSION,
        });
    }
    let (body, rest) = postcard::take_from_bytes::<T>(rest).map_err(malformed)?;
    if !rest.is_empty() {
        return Err(PackError::Malformed(format!(
            "結尾多了 {} 個位元組",
            rest.len()
        )));
    }
    Ok(body)
}

impl SchemePack {
    pub fn to_bytes(&self) -> Result<Vec<u8>, PackError> {
        encode(SCHEME_MAGIC, self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PackError> {
        decode(SCHEME_MAGIC, bytes)
    }
}

impl LexiconPack {
    pub fn to_bytes(&self) -> Result<Vec<u8>, PackError> {
        encode(LEXICON_MAGIC, self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PackError> {
        decode(LEXICON_MAGIC, bytes)
    }
}

impl LanguageModelPack {
    pub fn to_bytes(&self) -> Result<Vec<u8>, PackError> {
        encode(LANGUAGE_MODEL_MAGIC, self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PackError> {
        decode(LANGUAGE_MODEL_MAGIC, bytes)
    }
}

impl SimplifierPack {
    pub fn to_bytes(&self) -> Result<Vec<u8>, PackError> {
        encode(SIMPLIFIER_MAGIC, self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PackError> {
        decode(SIMPLIFIER_MAGIC, bytes)
    }
}

impl RadicalHintPack {
    pub fn to_bytes(&self) -> Result<Vec<u8>, PackError> {
        encode(RADICAL_HINT_MAGIC, self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PackError> {
        decode(RADICAL_HINT_MAGIC, bytes)
    }
}

impl EnglishPack {
    pub fn to_bytes(&self) -> Result<Vec<u8>, PackError> {
        encode(ENGLISH_MAGIC, self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PackError> {
        decode(ENGLISH_MAGIC, bytes)
    }
}
