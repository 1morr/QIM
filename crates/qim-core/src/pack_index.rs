//! 更新索引（architecture〈資料包〉）：線上更新的來源（GitHub Releases）上的 `index.json`，列出每份資料包的
//! manifest 與下載位置。殼拿它和裝著的資料包比出要下載的（[`PackIndex::updates`]），下載後照樣經
//! [`crate::Pack::open`] 核對；建置端拿上一版比出內容改了卻沒加版本的（[`PackIndex::check_successor`]）。

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{Manifest, PackError, PackKind};

/// 更新索引的檔名，和資料包一起放在 release 上。
pub const PACK_INDEX_FILE: &str = "index.json";

/// 索引格式的版本（同 [`crate::MANIFEST_VERSION`] 的規則）：欄位的意思改了才加一，加欄位不必。
pub const PACK_INDEX_VERSION: u32 = 1;

/// 索引的一項：一份資料包的 manifest 欄位（攤平在同一層）加上下載位置。殼可以把一項原樣當 manifest
/// 交給核心：manifest 不認得的欄位（`url`）略過。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexEntry {
    #[serde(flatten)]
    pub manifest: Manifest,
    /// 資料包檔的位置，相對於索引本身的網址（RFC 3986 的相對參照）；release 上就是 `<id>.pack`。
    pub url: String,
}

/// [`PackIndex::from_json`] 讀出來的：索引與略過的項。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedIndex {
    pub index: PackIndex,
    /// 略過的項（不認得的種類），照索引的順序；殼印成警告。
    pub warnings: Vec<IndexWarning>,
}

/// [`PackIndex::from_json`] 略過的一項：第幾項（從 1 起）、它的 id 與不認得的種類。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexWarning {
    pub position: usize,
    pub id: String,
    pub kind: String,
}

impl fmt::Display for IndexWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "更新索引第 {} 項 {}：不認得的種類「{}」，略過",
            self.position, self.id, self.kind
        )
    }
}

/// `index.json` 的內容。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackIndex {
    /// [`PACK_INDEX_VERSION`]。
    pub index_version: u32,
    pub packs: Vec<IndexEntry>,
}

impl PackIndex {
    pub fn new(packs: Vec<IndexEntry>) -> Self {
        Self {
            index_version: PACK_INDEX_VERSION,
            packs,
        }
    }

    /// 讀 `index.json`：格式版本要是 [`PACK_INDEX_VERSION`]、每項的 manifest 欄位合格式、id 不重複。
    /// 種類不認得的項（之後的核心加的）略過並回警告，其他照讀：舊核心才不會因為多一種資料包就整個停掉
    /// 線上更新。單一 manifest 的不認得種類則拒絕（[`Manifest::from_json`]）。
    pub fn from_json(json: &str) -> Result<ParsedIndex, PackError> {
        /// 各項先不解開，才能逐項認種類。
        #[derive(Deserialize)]
        struct Raw {
            index_version: u32,
            packs: Vec<serde_json::Value>,
        }
        let invalid = |reason: String| PackError::InvalidIndex(reason);
        let raw: Raw = serde_json::from_str(json).map_err(|e| invalid(e.to_string()))?;
        if raw.index_version != PACK_INDEX_VERSION {
            return Err(invalid(format!(
                "index_version {}，這個核心只讀 {PACK_INDEX_VERSION}",
                raw.index_version
            )));
        }
        let mut packs = Vec::new();
        let mut warnings = Vec::new();
        for (i, value) in raw.packs.into_iter().enumerate() {
            if let Some(kind) = unknown_kind(&value) {
                let id = value.get("id").and_then(|id| id.as_str()).unwrap_or("");
                warnings.push(IndexWarning {
                    position: i + 1,
                    id: id.into(),
                    kind: kind.into(),
                });
                continue;
            }
            let entry: IndexEntry = serde_json::from_value(value)
                .map_err(|e| invalid(format!("第 {} 項：{e}", i + 1)))?;
            packs.push(entry);
        }
        let mut seen = std::collections::HashSet::new();
        for entry in &packs {
            let id = &entry.manifest.id;
            entry
                .manifest
                .check_format()
                .map_err(|e| invalid(format!("{id}：{e}")))?;
            if !seen.insert(id) {
                return Err(invalid(format!("{id} 列了兩次")));
            }
        }
        Ok(ParsedIndex {
            index: Self::new(packs),
            warnings,
        })
    }

    /// 縮排兩格、結尾換行的 JSON。
    pub fn to_json(&self) -> String {
        let mut json = serde_json::to_string_pretty(self).expect("索引都是字串與整數");
        json.push('\n');
        json
    }

    /// 要下載的：`installed` 裡每份資料包在索引裡同 id、同種類、版本較大、這個核心讀得了的那項，
    /// 照索引的順序。沒裝的資料包不列（線上更新只換新版，不裝新的）。
    pub fn updates(&self, installed: &[Manifest]) -> Vec<&IndexEntry> {
        self.packs
            .iter()
            .filter(|entry| {
                let Ok(offered) = entry.manifest.check_format() else {
                    return false;
                };
                offered.core_supported()
                    && installed.iter().any(|have| {
                        have.id == entry.manifest.id
                            && have.kind == entry.manifest.kind
                            && entry.manifest.is_newer_than(have)
                    })
            })
            .collect()
    }

    /// 對上一版索引 `previous`：同一個 id 的資料包內容（sha256）改了，版本就要較大；版本不能變小。
    /// 新加、拿掉的資料包不管。不合的每份回一行說明。
    pub fn check_successor(&self, previous: &PackIndex) -> Result<(), Vec<String>> {
        let mut problems = Vec::new();
        for entry in &self.packs {
            let now = &entry.manifest;
            let Some(before) = previous.packs.iter().find(|p| p.manifest.id == now.id) else {
                continue;
            };
            let before = &before.manifest;
            let (Ok(new), Ok(old)) = (now.check_format(), before.check_format()) else {
                problems.push(format!("{}：manifest 不合格式", now.id));
                continue;
            };
            let (new, old) = (new.version, old.version);
            if new < old {
                problems.push(format!(
                    "{}：版本 {} 比上一版的 {} 小",
                    now.id, now.version, before.version
                ));
            } else if new == old && now.sha256 != before.sha256 {
                problems.push(format!(
                    "{}：內容改了（sha256 不同），版本卻還是 {}：在 build_packs 加它的版本",
                    now.id, now.version
                ));
            }
        }
        if problems.is_empty() {
            Ok(())
        } else {
            Err(problems)
        }
    }
}

/// 項的 `kind` 是字串、但不是這個核心認得的 [`PackKind`] 時是它。
fn unknown_kind(entry: &serde_json::Value) -> Option<&str> {
    let kind = entry.get("kind")?.as_str()?;
    serde_json::from_value::<PackKind>(kind.into())
        .is_err()
        .then_some(kind)
}
