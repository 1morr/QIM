//! 資料包 manifest（plan §4.6）：資料包檔旁的 `manifest.json`，說明它是哪一份、要多新的核心、
//! 檔案的大小與 sha256、授權。殼匯入或載入資料包時經 [`crate::Pack::open`] 核對。

use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    EmojiPack, EnglishPack, LanguageModelPack, LexiconPack, Pack, PackError, RadicalHintPack,
    SchemePack, SimplifierPack, WordTrigramPack,
};

/// 這個核心的版本（`qim-core` 的 crate 版本）；manifest 的 `min_core_version` 和它比。
pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// manifest 的檔名，放在資料包檔 `<id>.pack` 旁邊。
pub const MANIFEST_FILE: &str = "manifest.json";

/// manifest 格式的版本（同 WebExtension 的 `manifest_version`）；欄位的意思改了才加一，加欄位不必。
pub const MANIFEST_VERSION: u32 = 1;

/// 資料包的種類，一種對應一個 [`Pack`] 型別（魔數不同）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackKind {
    Scheme,
    Lexicon,
    LanguageModel,
    WordTrigram,
    RadicalHint,
    Simplifier,
    English,
    Emoji,
}

impl PackKind {
    /// manifest 裡的寫法（`language-model`）。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Scheme => "scheme",
            Self::Lexicon => "lexicon",
            Self::LanguageModel => "language-model",
            Self::WordTrigram => "word-trigram",
            Self::RadicalHint => "radical-hint",
            Self::Simplifier => "simplifier",
            Self::English => "english",
            Self::Emoji => "emoji",
        }
    }
}

impl fmt::Display for PackKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 一份資料包的 manifest。欄位見 plan §4.6；不認得的欄位略過，給之後加欄位的舊核心讀。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// [`MANIFEST_VERSION`]。
    pub manifest_version: u32,
    /// 資料包的 id，等於資料包裡的 id（[`Pack::id`]）；小寫英數與 `-`，資料包檔名是 `<id>.pack`。
    pub id: String,
    pub kind: PackKind,
    /// 資料包的版本，semver；同一個 id 的新版較大。
    pub version: String,
    /// 讀得了它的最舊核心，`MAJOR.MINOR.PATCH`；比 [`CORE_VERSION`] 新時拒絕載入。
    pub min_core_version: String,
    /// 資料包檔的位元組數。
    pub size: u64,
    /// 資料包檔的 sha256，小寫十六進位。
    pub sha256: String,
    /// 授權，SPDX 運算式；全文在同目錄的 `LICENSE`。
    pub license: String,
}

impl Manifest {
    /// 描述 `kind` 種、id 是 `id` 的資料包檔 `bytes`；最低核心版本是建它的這個核心。
    pub fn new(
        kind: PackKind,
        id: impl Into<String>,
        bytes: &[u8],
        version: impl Into<String>,
        license: impl Into<String>,
    ) -> Self {
        Self {
            manifest_version: MANIFEST_VERSION,
            id: id.into(),
            kind,
            version: version.into(),
            min_core_version: CORE_VERSION.into(),
            size: bytes.len() as u64,
            sha256: sha256_hex(bytes),
            license: license.into(),
        }
    }

    pub fn from_json(json: &str) -> Result<Self, PackError> {
        serde_json::from_str(json).map_err(|e| PackError::InvalidManifest(e.to_string()))
    }

    /// 縮排兩格、結尾換行的 JSON。
    pub fn to_json(&self) -> String {
        let mut json = serde_json::to_string_pretty(self).expect("manifest 都是字串與整數");
        json.push('\n');
        json
    }

    /// 資料包檔的檔名 `<id>.pack`，和 manifest 在同一個目錄。
    pub fn file_name(&self) -> String {
        format!("{}.pack", self.id)
    }

    /// 版本（semver）比 `other` 的大；同版或任一邊讀不懂時不算。更新索引挑新版（[`crate::PackIndex`]）、
    /// 殼在同一個 id 的內建版與下載版之間挑一份（plan §4.7）都照它。
    pub fn is_newer_than(&self, other: &Manifest) -> bool {
        match (
            semver::Version::parse(&self.version),
            semver::Version::parse(&other.version),
        ) {
            (Ok(mine), Ok(theirs)) => mine > theirs,
            _ => false,
        }
    }

    /// 照 manifest 的種類核對並解開 `bytes` 一次，不留結果：不知道種類時用（匯入前檢查、建置端讀回）。
    pub fn verify(&self, bytes: &[u8]) -> Result<(), PackError> {
        fn open<P: Pack>(manifest: &Manifest, bytes: &[u8]) -> Result<(), PackError> {
            P::open(manifest, bytes).map(drop)
        }
        match self.kind {
            PackKind::Scheme => open::<SchemePack>(self, bytes),
            PackKind::Lexicon => open::<LexiconPack>(self, bytes),
            PackKind::LanguageModel => open::<LanguageModelPack>(self, bytes),
            PackKind::WordTrigram => open::<WordTrigramPack>(self, bytes),
            PackKind::RadicalHint => open::<RadicalHintPack>(self, bytes),
            PackKind::Simplifier => open::<SimplifierPack>(self, bytes),
            PackKind::English => open::<EnglishPack>(self, bytes),
            PackKind::Emoji => open::<EmojiPack>(self, bytes),
        }
    }

    /// 欄位的格式：`manifest_version`、`id`、`sha256`、`version` 與 `min_core_version`。回傳解析好的
    /// 兩個版本。
    pub(crate) fn check_format(&self) -> Result<Versions, PackError> {
        let invalid = |reason: String| Err(PackError::InvalidManifest(reason));
        if self.manifest_version != MANIFEST_VERSION {
            return invalid(format!(
                "manifest_version {}，這個核心只讀 {MANIFEST_VERSION}",
                self.manifest_version
            ));
        }
        let id_char = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-';
        if self.id.is_empty() || !self.id.chars().all(id_char) {
            return invalid(format!("id {:?} 只能有小寫英數與 -", self.id));
        }
        let hex = |c: char| c.is_ascii_digit() || ('a'..='f').contains(&c);
        if self.sha256.len() != 64 || !self.sha256.chars().all(hex) {
            return invalid(format!("sha256 {:?} 要是 64 位小寫十六進位", self.sha256));
        }
        let version = semver::Version::parse(&self.version).map_err(|e| {
            PackError::InvalidManifest(format!("version {:?} 不是 semver：{e}", self.version))
        })?;
        let required = semver::Version::parse(&self.min_core_version).map_err(|e| {
            PackError::InvalidManifest(format!(
                "min_core_version {:?} 不是 MAJOR.MINOR.PATCH：{e}",
                self.min_core_version
            ))
        })?;
        Ok(Versions {
            version,
            min_core: required,
        })
    }

    /// 核對 `bytes` 是這份 manifest 說的 `P` 資料包，而且這個核心讀得了：先看欄位的格式、核心
    /// 版本與種類，再看大小，最後算 sha256。
    pub(crate) fn check<P: Pack>(&self, bytes: &[u8]) -> Result<(), PackError> {
        if !self.check_format()?.core_supported() {
            return Err(PackError::CoreTooOld {
                required: self.min_core_version.clone(),
                current: CORE_VERSION.into(),
            });
        }
        if self.kind != P::KIND {
            return Err(PackError::KindMismatch {
                expected: P::KIND,
                found: self.kind,
            });
        }
        let size = bytes.len() as u64;
        if size != self.size {
            return Err(PackError::SizeMismatch {
                expected: self.size,
                found: size,
            });
        }
        let found = sha256_hex(bytes);
        if found != self.sha256 {
            return Err(PackError::ChecksumMismatch {
                expected: self.sha256.clone(),
                found,
            });
        }
        Ok(())
    }
}

/// manifest 解析好的兩個版本（[`Manifest::check_format`]）。
pub(crate) struct Versions {
    /// `version`。
    pub(crate) version: semver::Version,
    /// `min_core_version`。
    pub(crate) min_core: semver::Version,
}

impl Versions {
    /// 這個核心讀得了它：`min_core_version ≤ CORE_VERSION`。
    pub(crate) fn core_supported(&self) -> bool {
        self.min_core <= semver::Version::parse(CORE_VERSION).expect("crate 版本是 semver")
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
