use std::cell::Cell;
use std::io::Write;

use qim_databuild::Error;
use qim_databuild::sources::{self, Fetch, SourceFile};

/// `printf hello | shasum -a 256`
const HELLO_SHA256: &str = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";

struct FakeFetch {
    body: &'static [u8],
    calls: Cell<usize>,
}

impl FakeFetch {
    fn new(body: &'static [u8]) -> Self {
        Self {
            body,
            calls: Cell::new(0),
        }
    }
}

impl Fetch for FakeFetch {
    fn fetch(&self, _url: &str, out: &mut dyn Write) -> Result<(), String> {
        self.calls.set(self.calls.get() + 1);
        out.write_all(self.body).map_err(|e| e.to_string())
    }
}

/// 寫了一半就斷線。
struct Interrupted;

impl Fetch for Interrupted {
    fn fetch(&self, _url: &str, out: &mut dyn Write) -> Result<(), String> {
        out.write_all(b"hel").map_err(|e| e.to_string())?;
        Err("連線中斷".into())
    }
}

struct Offline;

impl Fetch for Offline {
    fn fetch(&self, url: &str, _out: &mut dyn Write) -> Result<(), String> {
        Err(format!("離線，不該連 {url}"))
    }
}

fn file() -> SourceFile {
    SourceFile {
        path: "data/hello.txt".into(),
        url: "https://example.invalid/hello.txt".into(),
        sha256: HELLO_SHA256.into(),
    }
}

#[test]
fn missing_file_is_downloaded_once_then_read_from_cache() {
    let cache = tempfile::tempdir().unwrap();
    let fetch = FakeFetch::new(b"hello");
    assert_eq!(
        sources::load(cache.path(), "demo", &file(), &fetch).unwrap(),
        b"hello"
    );
    assert_eq!(
        sources::load(cache.path(), "demo", &file(), &Offline).unwrap(),
        b"hello"
    );
    assert_eq!(fetch.calls.get(), 1);
}

#[test]
fn tampered_cache_fails_without_downloading_again() {
    let cache = tempfile::tempdir().unwrap();
    sources::load(cache.path(), "demo", &file(), &FakeFetch::new(b"hello")).unwrap();
    std::fs::write(sources::cache_path(cache.path(), "demo", &file()), "hellO").unwrap();
    let fetch = FakeFetch::new(b"hello");
    let err = sources::load(cache.path(), "demo", &file(), &fetch).unwrap_err();
    assert!(matches!(err, Error::ChecksumMismatch { .. }), "{err}");
    assert_eq!(fetch.calls.get(), 0);
}

#[test]
fn download_with_wrong_checksum_fails_and_is_not_cached() {
    let cache = tempfile::tempdir().unwrap();
    let err = sources::load(cache.path(), "demo", &file(), &FakeFetch::new(b"hellO")).unwrap_err();
    assert!(matches!(err, Error::ChecksumMismatch { .. }), "{err}");
    assert!(!sources::cache_path(cache.path(), "demo", &file()).exists());
}

#[test]
fn fetch_failure_is_reported() {
    let cache = tempfile::tempdir().unwrap();
    let err = sources::load(cache.path(), "demo", &file(), &Offline).unwrap_err();
    assert!(matches!(err, Error::Fetch { .. }), "{err}");
}

#[test]
fn cached_returns_the_verified_file_in_the_cache() {
    let cache = tempfile::tempdir().unwrap();
    let path = sources::cached(cache.path(), "demo", &file(), &FakeFetch::new(b"hello")).unwrap();
    assert_eq!(path, sources::cache_path(cache.path(), "demo", &file()));
    assert_eq!(std::fs::read(&path).unwrap(), b"hello");
    assert_eq!(
        sources::cached(cache.path(), "demo", &file(), &Offline).unwrap(),
        path
    );
}

#[test]
fn interrupted_download_leaves_nothing_in_the_cache() {
    let cache = tempfile::tempdir().unwrap();
    let err = sources::cached(cache.path(), "demo", &file(), &Interrupted).unwrap_err();
    assert!(matches!(err, Error::Fetch { .. }), "{err}");
    let dir = sources::cache_path(cache.path(), "demo", &file());
    let dir = dir.parent().unwrap();
    let left: Vec<_> = std::fs::read_dir(dir)
        .map(|d| d.map(|e| e.unwrap().path()).collect())
        .unwrap_or_default();
    assert!(left.is_empty(), "{left:?}");
}

#[test]
fn cache_dir_prefers_the_environment_variable_then_the_platform_cache() {
    use std::path::{Path, PathBuf};
    let platform = Some(PathBuf::from("/home/u/.cache"));
    assert_eq!(
        sources::cache_dir(Some("/shared/qim".into()), platform.clone()),
        Some(PathBuf::from("/shared/qim"))
    );
    assert_eq!(
        sources::cache_dir(None, platform.clone()),
        Some(Path::new("/home/u/.cache").join("qim"))
    );
    // 空字串當沒設，同 XDG 慣例。
    assert_eq!(
        sources::cache_dir(Some("".into()), platform),
        Some(Path::new("/home/u/.cache").join("qim"))
    );
    assert_eq!(sources::cache_dir(None, None), None);
}

#[test]
fn changing_the_pinned_checksum_uses_a_new_cache_path() {
    let mut other = file();
    other.sha256 = "0".repeat(64);
    assert_ne!(
        sources::cache_path("c".as_ref(), "demo", &file()),
        sources::cache_path("c".as_ref(), "demo", &other)
    );
}

#[test]
fn sources_file_in_repo_pins_libcangjie_table() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/sources.toml");
    let all = sources::parse(&std::fs::read_to_string(path).unwrap()).unwrap();
    let lib = &all.source["libcangjie"];
    assert!(lib.version.starts_with("346026b"));
    let table = lib
        .files
        .iter()
        .find(|f| f.path == "data/table.txt")
        .unwrap();
    assert_eq!(table.sha256.len(), 64);
    assert!(table.url.contains(&lib.version));
}

#[test]
fn sources_file_in_repo_pins_unihan_zip_of_the_same_unicode_version() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/sources.toml");
    let all = sources::parse(&std::fs::read_to_string(path).unwrap()).unwrap();
    let unihan = &all.source["unihan"];
    assert_eq!(unihan.license, "Unicode-3.0");
    let zip = unihan.file("Unihan.zip").unwrap();
    assert_eq!(zip.sha256.len(), 64);
    assert!(
        zip.url
            .contains(&format!("/Public/{}/ucd/", unihan.version)),
        "{}",
        zip.url
    );
}
