//! 更新索引（plan §4.7）：線上更新的來源（GitHub Releases）上的 `index.json`，列出每份資料包的
//! manifest 與下載位置。殼拿它和裝著的資料包比出要下載的（[`PackIndex::updates`]），下載後照樣經
//! [`crate::Pack::open`] 核對；建置端拿上一版比出內容改了卻沒加版本的（[`PackIndex::check_successor`]）。

use serde::{Deserialize, Serialize};

use crate::{Manifest, PackError};

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
    pub fn from_json(json: &str) -> Result<Self, PackError> {
        let invalid = |reason: String| PackError::InvalidIndex(reason);
        let index: Self = serde_json::from_str(json).map_err(|e| invalid(e.to_string()))?;
        if index.index_version != PACK_INDEX_VERSION {
            return Err(invalid(format!(
                "index_version {}，這個核心只讀 {PACK_INDEX_VERSION}",
                index.index_version
            )));
        }
        let mut seen = std::collections::HashSet::new();
        for entry in &index.packs {
            let id = &entry.manifest.id;
            entry
                .manifest
                .check_format()
                .map_err(|e| invalid(format!("{id}：{e}")))?;
            if !seen.insert(id) {
                return Err(invalid(format!("{id} 列了兩次")));
            }
        }
        Ok(index)
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
                            && have
                                .check_format()
                                .is_ok_and(|current| offered.version > current.version)
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
