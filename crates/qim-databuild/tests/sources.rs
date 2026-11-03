use std::cell::Cell;

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
    fn fetch(&self, _url: &str) -> Result<Vec<u8>, String> {
        self.calls.set(self.calls.get() + 1);
        Ok(self.body.to_vec())
    }
}

struct Offline;

impl Fetch for Offline {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, String> {
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
