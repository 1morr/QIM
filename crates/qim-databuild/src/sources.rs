//! `data/sources.toml` 與下載快取（`.cache/sources/`）。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::Error;

/// 來源清單在 repo 裡的位置。
pub const SOURCES: &str = "data/sources.toml";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sources {
    pub source: BTreeMap<String, Source>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub repository: String,
    pub commit: String,
    pub license: String,
    /// 授權聲明所在的檔，是 [`Source::files`] 之一。
    pub license_file: String,
    /// 照抄 [`Source::license_file`] 裡的聲明原文。
    pub license_statement: String,
    pub files: Vec<SourceFile>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceFile {
    /// 檔案在來源 repo 裡的路徑。
    pub path: String,
    pub url: String,
    pub sha256: String,
}

impl Source {
    pub fn file(&self, path: &str) -> Result<&SourceFile, Error> {
        self.files
            .iter()
            .find(|f| f.path == path)
            .ok_or_else(|| Error::invalid(SOURCES, format!("沒有列出 {path}")))
    }
}

pub fn parse(text: &str) -> Result<Sources, Error> {
    toml::from_str(text).map_err(|e| Error::invalid(SOURCES, e.to_string()))
}

/// 下載一個網址的內容；測試換成假的，不連網。
pub trait Fetch {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, String>;
}

/// 真的 HTTP 下載。
pub struct Http;

/// 單一來源檔的大小上限；ureq 預設只收 10 MB。
const MAX_DOWNLOAD_BYTES: u64 = 256 * 1024 * 1024;

impl Fetch for Http {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, String> {
        let mut response = ureq::get(url).call().map_err(|e| e.to_string())?;
        response
            .body_mut()
            .with_config()
            .limit(MAX_DOWNLOAD_BYTES)
            .read_to_vec()
            .map_err(|e| e.to_string())
    }
}

/// 快取位置含釘住的 sha256：換版本時自動用新路徑，不會誤用舊檔。
pub fn cache_path(cache_root: &Path, id: &str, file: &SourceFile) -> PathBuf {
    let pin = file.sha256.get(..16).unwrap_or(&file.sha256);
    cache_root.join(id).join(pin).join(&file.path)
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// 讀來源檔：有快取就只讀快取、不連網；沒有才下載。兩種情況 sha256 不符都失敗，
/// 下載的內容不符時不寫進快取。
pub fn load(
    cache_root: &Path,
    id: &str,
    file: &SourceFile,
    fetch: &dyn Fetch,
) -> Result<Vec<u8>, Error> {
    let path = cache_path(cache_root, id, file);
    let checked = |bytes: Vec<u8>| {
        let actual = sha256_hex(&bytes);
        if actual == file.sha256 {
            Ok(bytes)
        } else {
            Err(Error::ChecksumMismatch {
                path: path.clone(),
                expected: file.sha256.clone(),
                actual,
            })
        }
    };
    match std::fs::read(&path) {
        Ok(bytes) => return checked(bytes),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(Error::io(&path)(e)),
    }

    let bytes = fetch.fetch(&file.url).map_err(|reason| Error::Fetch {
        url: file.url.clone(),
        reason,
    })?;
    let bytes = checked(bytes)?;
    let dir = path.parent().expect("快取路徑至少有 id 一層");
    std::fs::create_dir_all(dir).map_err(Error::io(dir))?;
    // 先寫暫存檔再改名，中斷的下載不會留下半個檔。
    let partial = path.with_extension("part");
    std::fs::write(&partial, &bytes).map_err(Error::io(&partial))?;
    std::fs::rename(&partial, &path).map_err(Error::io(&path))?;
    Ok(bytes)
}
