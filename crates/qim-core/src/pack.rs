use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{CodeTable, Scheme};

/// 資料包格式的版本；格式改了就加一，舊核心遇到新版本回 [`PackError::UnsupportedVersion`]。
pub const FORMAT_VERSION: u32 = 1;

/// 檔頭的魔數，用來分辨資料包與其他檔案。
const MAGIC: &[u8; 4] = b"QIMP";

/// 一個輸入方案的資料包：方案與它的碼表。
///
/// 位元組格式：[`MAGIC`]、postcard 的 [`FORMAT_VERSION`]、postcard 的本體。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pack {
    pub scheme: Scheme,
    pub code_table: CodeTable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackError {
    /// 檔頭不是資料包的魔數。
    NotAPack,
    UnsupportedVersion {
        found: u32,
        supported: u32,
    },
    /// 版本對，但內容解不開、被截斷或有多餘的位元組。
    Malformed(String),
}

impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAPack => write!(f, "不是 QIM 資料包"),
            Self::UnsupportedVersion { found, supported } => {
                write!(f, "資料包格式版本 {found}，這個核心只支援 {supported}")
            }
            Self::Malformed(reason) => write!(f, "資料包內容損壞：{reason}"),
        }
    }
}

impl std::error::Error for PackError {}

fn malformed(e: postcard::Error) -> PackError {
    PackError::Malformed(e.to_string())
}

impl Pack {
    pub fn to_bytes(&self) -> Result<Vec<u8>, PackError> {
        let mut bytes = MAGIC.to_vec();
        bytes.extend(postcard::to_allocvec(&FORMAT_VERSION).map_err(malformed)?);
        bytes.extend(postcard::to_allocvec(self).map_err(malformed)?);
        Ok(bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PackError> {
        let rest = bytes.strip_prefix(MAGIC).ok_or(PackError::NotAPack)?;
        let (found, rest) = postcard::take_from_bytes::<u32>(rest).map_err(malformed)?;
        if found != FORMAT_VERSION {
            return Err(PackError::UnsupportedVersion {
                found,
                supported: FORMAT_VERSION,
            });
        }
        let (pack, rest) = postcard::take_from_bytes::<Self>(rest).map_err(malformed)?;
        if !rest.is_empty() {
            return Err(PackError::Malformed(format!(
                "結尾多了 {} 個位元組",
                rest.len()
            )));
        }
        Ok(pack)
    }
}
