use std::fmt;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::{
    CodeTable, EmojiTable, EnglishWords, LanguageModel, Lexicon, Manifest, PackKind, RadicalHints,
    Scheme, Simplifier, WordTrigram,
};

/// 資料包格式的版本，各種資料包共用；格式改了就加一，舊核心遇到新版本回 [`PackError::UnsupportedVersion`]。
pub const FORMAT_VERSION: u32 = 8;

/// 各種資料包共用的位元組格式：魔數 [`Pack::MAGIC`]、postcard 的 [`FORMAT_VERSION`]、
/// postcard 的本體。
pub trait Pack: Serialize + DeserializeOwned {
    /// 檔頭的魔數，每種資料包不同。
    const MAGIC: [u8; 4];

    /// manifest 的 `kind`。
    const KIND: PackKind;

    /// 資料包的 id；方案資料包是方案的 id。
    fn id(&self) -> &str;

    /// 殼載入資料包的入口：照 `manifest` 核對最低核心版本、種類、大小與 sha256，解開後再核對 id
    /// （plan〈資料包 manifest〉）。[`Pack::from_bytes`] 不核對，給建置端與測試讀剛建好的資料包。
    fn open(manifest: &Manifest, bytes: &[u8]) -> Result<Self, PackError> {
        manifest.check::<Self>(bytes)?;
        let pack = Self::from_bytes(bytes)?;
        if pack.id() != manifest.id {
            return Err(PackError::IdMismatch {
                manifest: manifest.id.clone(),
                pack: pack.id().to_string(),
            });
        }
        Ok(pack)
    }

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

/// 詞庫資料包：詞、詞頻與字頻（plan〈詞庫〉）。方案用 [`crate::Packs::lexicon`] 指名它的 `id`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LexiconPack {
    pub id: String,
    pub lexicon: Lexicon,
}

/// 語言模型資料包：字級 bigram（plan〈字級 bigram 與詞級 trigram〉）。方案用 [`crate::Packs::language_model`] 指名它的 `id`。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguageModelPack {
    pub id: String,
    pub model: LanguageModel,
}

/// 詞級 trigram 資料包：同步層的語言模型（plan〈排序：同一把尺〉〈字級 bigram 與詞級 trigram〉）。方案用 [`crate::Packs::word_trigram`]
/// 指名它的 `id`，由 [`crate::Engine::with_word_trigram`] 掛上。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WordTrigramPack {
    pub id: String,
    pub model: WordTrigram,
}

/// 簡繁資料包：OpenCC 的轉換表（plan〈英文、簡繁與 emoji〉），由 [`crate::Engine::with_simplifier`] 掛上，
/// 不由方案指名：各方案都輸出繁體，簡體模式共用同一份。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimplifierPack {
    pub id: String,
    pub simplifier: Simplifier,
}

/// 字根提示資料包：字 → 倉頡三代 / 五代碼（plan〈速成碼表〉 步驟 5）。方案用
/// [`crate::Packs::radical_hint`] 指名它的 `id`，殼用 [`RadicalHints::for_scheme`] 取出。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RadicalHintPack {
    pub id: String,
    pub hints: RadicalHints,
}

/// 英文資料包：中文狀態的英文候選用的詞表（plan〈英文、簡繁與 emoji〉），由 [`crate::Engine::with_english`]
/// 掛上，不由方案指名，各方案共用。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnglishPack {
    pub id: String,
    pub words: EnglishWords,
}

/// emoji 資料包：字詞 → emoji（plan〈英文、簡繁與 emoji〉），由 [`crate::Engine::with_emoji`] 掛上，不由方案指名，
/// 各方案共用。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmojiPack {
    pub id: String,
    pub emoji: EmojiTable,
}

/// 停鍵層的模型資料包（plan〈模型分層〉〈資料包 manifest〉）：神經重排的權重與字表。核心只核對與解開，不跑模型：
/// 殼把欄位交給 `qim-neural`（`Reranker::new`），在背景執行緒或 Web Worker 推論。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelPack {
    pub id: String,
    /// 字表裡兩個記號之後的字，依序是 token 2、3…（`scripts/neural-rerank/train.py prepare`）。
    pub vocab: Vec<char>,
    /// 訓練語料裡字表外的字數：一個字表外的字分到 `<unk>` 機率的這麼多分之一。
    pub unknown_chars: u32,
    /// `路徑分 + λ·神經分` 的 λ，在 dev 切分上調的。
    pub lambda: f64,
    /// GGUF 權重（q8）。
    pub weights: Vec<u8>,
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
    /// 方案指名了詞庫卻沒給，或沒指名卻給了；`None` 是方案沒指名或沒給。
    LexiconMismatch {
        expected: Option<String>,
        found: Option<String>,
    },
    /// 方案指名了語言模型卻沒給，或沒指名卻給了；`None` 是方案沒指名或沒給。
    LanguageModelMismatch {
        expected: Option<String>,
        found: Option<String>,
    },
    /// 方案指名了詞級 trigram 卻沒給，或沒指名卻給了；`None` 是方案沒指名或沒給。
    WordTrigramMismatch {
        expected: Option<String>,
        found: Option<String>,
    },
    /// 方案指名了字根提示卻沒給，或沒指名卻給了；`None` 是方案沒指名或沒給。
    RadicalHintMismatch {
        expected: Option<String>,
        found: Option<String>,
    },
    /// manifest 不是合格的 JSON、少了欄位或欄位的值不合格式。
    InvalidManifest(String),
    /// 更新索引（`index.json`）不是合格的 JSON、格式版本不對、有一項的 manifest 欄位不合格式或
    /// id 重複。
    InvalidIndex(String),
    /// manifest 要求比這個核心新的版本。
    CoreTooOld {
        required: String,
        current: String,
    },
    /// manifest 說的種類不是要載入的那種。
    KindMismatch {
        expected: PackKind,
        found: PackKind,
    },
    /// 資料包檔的大小和 manifest 不同：多半是下載或複製不完整。
    SizeMismatch {
        expected: u64,
        found: u64,
    },
    /// 資料包檔的 sha256 和 manifest 不同：檔案損壞或被改過。
    ChecksumMismatch {
        expected: String,
        found: String,
    },
    /// manifest 的 id 和資料包裡的不同。
    IdMismatch {
        manifest: String,
        pack: String,
    },
    /// 交給 [`crate::Engine::open`] 的資料包裡同一種有兩份。
    DuplicateKind(PackKind),
    /// 這種資料包不交給 [`crate::Engine::open`]（字根提示、停鍵層的模型由殼另外載入）。
    NotForEngine(PackKind),
    /// 少了一定要有的那種資料包。
    Missing(PackKind),
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
            Self::WordTrigramMismatch { expected, found } => write!(
                f,
                "方案指名的詞級 trigram 是 {}，給的是 {}",
                name(expected),
                name(found)
            ),
            Self::RadicalHintMismatch { expected, found } => write!(
                f,
                "方案指名的字根提示是 {}，給的是 {}",
                name(expected),
                name(found)
            ),
            Self::InvalidManifest(reason) => write!(f, "manifest 不合格式：{reason}"),
            Self::InvalidIndex(reason) => write!(f, "更新索引不合格式：{reason}"),
            Self::CoreTooOld { required, current } => write!(
                f,
                "資料包要核心 {required} 以上，這個核心是 {current}：先更新 App"
            ),
            Self::KindMismatch { expected, found } => {
                write!(f, "要的是 {expected} 資料包，manifest 寫的是 {found}")
            }
            Self::SizeMismatch { expected, found } => write!(
                f,
                "資料包檔 {found} 位元組，manifest 寫 {expected}：檔案不完整"
            ),
            Self::ChecksumMismatch { expected, found } => write!(
                f,
                "資料包檔的 sha256 是 {found}，manifest 寫 {expected}：檔案損壞或被改過"
            ),
            Self::IdMismatch { manifest, pack } => {
                write!(f, "manifest 的 id 是 {manifest}，資料包裡是 {pack}")
            }
            Self::DuplicateKind(kind) => write!(f, "給了兩份 {kind} 資料包"),
            Self::NotForEngine(kind) => write!(f, "{kind} 資料包不交給 Engine"),
            Self::Missing(kind) => write!(f, "沒有 {kind} 資料包"),
        }
    }
}

impl std::error::Error for PackError {}

fn malformed(e: postcard::Error) -> PackError {
    PackError::Malformed(e.to_string())
}

impl Pack for SchemePack {
    const MAGIC: [u8; 4] = *b"QIMP";
    const KIND: PackKind = PackKind::Scheme;

    fn id(&self) -> &str {
        &self.scheme.id
    }
}

impl Pack for LexiconPack {
    const MAGIC: [u8; 4] = *b"QIML";
    const KIND: PackKind = PackKind::Lexicon;

    fn id(&self) -> &str {
        &self.id
    }
}

impl Pack for LanguageModelPack {
    const MAGIC: [u8; 4] = *b"QIMB";
    const KIND: PackKind = PackKind::LanguageModel;

    fn id(&self) -> &str {
        &self.id
    }
}

impl Pack for SimplifierPack {
    const MAGIC: [u8; 4] = *b"QIMS";
    const KIND: PackKind = PackKind::Simplifier;

    fn id(&self) -> &str {
        &self.id
    }
}

impl Pack for RadicalHintPack {
    const MAGIC: [u8; 4] = *b"QIMH";
    const KIND: PackKind = PackKind::RadicalHint;

    fn id(&self) -> &str {
        &self.id
    }
}

impl Pack for EnglishPack {
    const MAGIC: [u8; 4] = *b"QIME";
    const KIND: PackKind = PackKind::English;

    fn id(&self) -> &str {
        &self.id
    }
}

impl Pack for EmojiPack {
    const MAGIC: [u8; 4] = *b"QIMJ";
    const KIND: PackKind = PackKind::Emoji;

    fn id(&self) -> &str {
        &self.id
    }
}

impl Pack for ModelPack {
    const MAGIC: [u8; 4] = *b"QIMN";
    const KIND: PackKind = PackKind::Model;

    fn id(&self) -> &str {
        &self.id
    }
}

impl Pack for WordTrigramPack {
    const MAGIC: [u8; 4] = *b"QIMW";
    const KIND: PackKind = PackKind::WordTrigram;

    fn id(&self) -> &str {
        &self.id
    }
}
