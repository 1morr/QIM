use std::fmt;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::{CodeTable, Lexicon, Scheme};

/// 資料包格式的版本，兩種資料包共用；格式改了就加一，舊核心遇到新版本回 [`PackError::UnsupportedVersion`]。
pub const FORMAT_VERSION: u32 = 3;

/// 輸入方案資料包檔頭的魔數。
const SCHEME_MAGIC: &[u8; 4] = b"QIMP";
/// 詞庫資料包檔頭的魔數。
const LEXICON_MAGIC: &[u8; 4] = b"QIML";

/// 一個輸入方案的資料包：方案與它的碼表。
///
/// 位元組格式：魔數 `QIMP`、postcard 的 [`FORMAT_VERSION`]、postcard 的本體。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pack {
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

impl Pack {
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
