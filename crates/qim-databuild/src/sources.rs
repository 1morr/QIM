//! `data/sources.toml` 與下載快取（`.cache/sources/`）。

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufWriter, Read, Write};
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
    /// git 來源填 commit，其他填上游的版本號。
    pub version: String,
    pub license: String,
    /// 授權聲明所在的檔，是 [`Source::files`] 之一；進資料包的來源才要。
    pub license_file: Option<String>,
    /// 照抄 [`Source::license_file`] 裡的聲明原文；進資料包的來源才要。
    pub license_statement: Option<String>,
    /// repo 裡存的授權全文檔，相對 repo 根目錄；上游的授權檔不釘版本時用它，
    /// 不和 [`Source::license_file`] 並用。進資料包的來源才要。
    pub license_text: Option<String>,
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

/// 下載一個網址的內容，邊下載邊寫進 `out`；測試換成假的，不連網。
pub trait Fetch {
    fn fetch(&self, url: &str, out: &mut dyn Write) -> Result<(), String>;
}

/// 真的 HTTP 下載。
pub struct Http;

/// 單一來源檔的大小上限，擋住回應異常大的伺服器；中文維基的分片檔不到 1 GB。
const MAX_DOWNLOAD_BYTES: u64 = 8 * 1024 * 1024 * 1024;

impl Fetch for Http {
    fn fetch(&self, url: &str, out: &mut dyn Write) -> Result<(), String> {
        let response = ureq::get(url).call().map_err(|e| e.to_string())?;
        let mut body = response
            .into_body()
            .into_with_config()
            .limit(MAX_DOWNLOAD_BYTES)
            .reader();
        io::copy(&mut body, out)
            .map(drop)
            .map_err(|e| e.to_string())
    }
}

/// 指定下載快取位置的環境變數。
pub const CACHE_ENV: &str = "QIM_CACHE_DIR";

/// 下載快取的根目錄：`env`（[`CACHE_ENV`] 的值）有設且非空就用它，否則用平台的使用者快取目錄
/// （`platform`，例如 macOS 的 `~/Library/Caches`）下的 `qim`。放在 repo 外，各 worktree 共用；
/// 快取路徑含釘住的 sha256，不同 worktree 釘不同版本也不會互相蓋掉。
pub fn cache_dir(env: Option<OsString>, platform: Option<PathBuf>) -> Option<PathBuf> {
    match env {
        Some(dir) if !dir.is_empty() => Some(PathBuf::from(dir)),
        _ => platform.map(|p| p.join("qim")),
    }
}

/// 依目前環境的 [`cache_dir`]。
pub fn default_cache_dir() -> Option<PathBuf> {
    cache_dir(std::env::var_os(CACHE_ENV), dirs::cache_dir())
}

/// 快取位置含釘住的 sha256：換版本時自動用新路徑，不會誤用舊檔。
pub fn cache_path(cache_root: &Path, id: &str, file: &SourceFile) -> PathBuf {
    let pin = file.sha256.get(..16).unwrap_or(&file.sha256);
    cache_root.join(id).join(pin).join(&file.path)
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

/// 小寫十六進位，sha256 的寫法。
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// 讀進記憶體的 [`cached`]，給小檔用。
pub fn load(
    cache_root: &Path,
    id: &str,
    file: &SourceFile,
    fetch: &dyn Fetch,
) -> Result<Vec<u8>, Error> {
    let path = cached(cache_root, id, file, fetch)?;
    std::fs::read(&path).map_err(Error::io(path))
}

/// 確保來源檔在快取裡且 sha256 相符，回傳它的路徑：有快取就只核對、不連網；沒有才下載。
/// 下載邊收邊寫進暫存檔並算 sha256，不整份放進記憶體；不符或中斷時不留下任何檔。
pub fn cached(
    cache_root: &Path,
    id: &str,
    file: &SourceFile,
    fetch: &dyn Fetch,
) -> Result<PathBuf, Error> {
    let path = cache_path(cache_root, id, file);
    let mismatch = |actual: String| Error::ChecksumMismatch {
        path: path.clone(),
        expected: file.sha256.clone(),
        actual,
    };
    match File::open(&path) {
        Ok(f) => {
            let actual = sha256_of(f).map_err(Error::io(&path))?;
            return if actual == file.sha256 {
                Ok(path)
            } else {
                Err(mismatch(actual))
            };
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(Error::io(&path)(e)),
    }

    let dir = path.parent().expect("快取路徑至少有 id 一層");
    std::fs::create_dir_all(dir).map_err(Error::io(dir))?;
    // 暫存檔名含行程 id：共用快取時兩個 worktree 同時下載同一個檔也不會寫到同一個暫存檔。
    let partial = path.with_extension(format!("part{}", std::process::id()));
    let result = download(&partial, file, fetch);
    let outcome = match result {
        Ok(actual) if actual == file.sha256 => {
            std::fs::rename(&partial, &path).map_err(Error::io(&path))
        }
        Ok(actual) => Err(mismatch(actual)),
        Err(e) => Err(e),
    };
    if outcome.is_err() {
        let _ = std::fs::remove_file(&partial);
    }
    outcome.map(|()| path)
}

/// 下載到 `partial`，回傳內容的 sha256。
fn download(partial: &Path, file: &SourceFile, fetch: &dyn Fetch) -> Result<String, Error> {
    let out = File::create(partial).map_err(Error::io(partial))?;
    let mut out = HashingWriter {
        inner: BufWriter::new(out),
        hasher: Sha256::new(),
    };
    fetch
        .fetch(&file.url, &mut out)
        .map_err(|reason| Error::Fetch {
            url: file.url.clone(),
            reason,
        })?;
    out.inner.flush().map_err(Error::io(partial))?;
    Ok(hex(&out.hasher.finalize()))
}

fn sha256_of(mut reader: impl Read) -> io::Result<String> {
    let mut hasher = Sha256::new();
    let mut buf = vec![0; 1 << 16];
    loop {
        match reader.read(&mut buf)? {
            0 => return Ok(hex(&hasher.finalize())),
            n => hasher.update(&buf[..n]),
        }
    }
}

struct HashingWriter<W> {
    inner: W,
    hasher: Sha256,
}

impl<W: Write> Write for HashingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
