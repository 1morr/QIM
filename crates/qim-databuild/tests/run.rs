use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

use qim_core::Pack;
use qim_databuild::sources::{Fetch, sha256_hex};
use qim_databuild::{Error, run};

const TABLE: &str = "\
日 日 1 1 0 0 1 0 0 0 0 a a NA 23061
問 问 1 1 0 0 1 0 0 0 0 anr anr NA 20001
丟 丢 1 1 0 0 1 0 0 0 0 hgi mgi,xxmgi NA 22813
𠀀 𠀀 1 0 0 0 0 0 0 0 0 a NA NA 0
";

/// 聲明原文在上游檔裡換了行，sources.toml 照抄成一行。
const README: &str = "\
Legal
=====

The data in the ``table.txt`` file is hereby placed under the Public
Domain, in the hope that is is useful.
";

const STATEMENT: &str = "The data in the ``table.txt`` file is hereby placed under the Public Domain, in the hope that is is useful.";

/// 日同碼、丟異碼、㐀只在 Unihan：比對 2 字、差異 1 字。
const KCANGJIE: &str = "\
# Unihan_DictionaryLikeData.txt
U+3400\tkCangjie\tTM
U+4E1F\tkCangjie\tMGI
U+65E5\tkCangjie\tA
";

fn unihan_zip() -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    zip.start_file(
        "Unihan_DictionaryLikeData.txt",
        zip::write::SimpleFileOptions::default(),
    )
    .unwrap();
    zip.write_all(KCANGJIE.as_bytes()).unwrap();
    zip.finish().unwrap().into_inner()
}

struct Files(BTreeMap<String, Vec<u8>>);

impl Fetch for Files {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, String> {
        self.0
            .get(url)
            .cloned()
            .ok_or_else(|| format!("沒有 {url}"))
    }
}

struct Offline;

impl Fetch for Offline {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, String> {
        Err(format!("離線，不該連 {url}"))
    }
}

fn upstream(readme: &str) -> Files {
    Files(BTreeMap::from([
        ("https://example.invalid/table.txt".into(), TABLE.into()),
        (
            "https://example.invalid/README.table.md".into(),
            readme.into(),
        ),
        ("https://example.invalid/Unihan.zip".into(), unihan_zip()),
    ]))
}

/// 在 `root` 放 sources.toml 與 repo 裡的速成方案檔。
fn repo(root: &Path, statement: &str) {
    std::fs::create_dir_all(root.join("data/schemes")).unwrap();
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/schemes/quick.toml"),
        root.join("data/schemes/quick.toml"),
    )
    .unwrap();
    let sources = format!(
        r#"
[source.libcangjie]
repository = "https://example.invalid/libcangjie"
version = "346026b8ccf4ffdfbe424400edb6a588c0c5ba8e"
license = "Public Domain"
license_file = "data/README.table.md"
license_statement = "{statement}"

[[source.libcangjie.files]]
path = "data/table.txt"
url = "https://example.invalid/table.txt"
sha256 = "{table}"

[[source.libcangjie.files]]
path = "data/README.table.md"
url = "https://example.invalid/README.table.md"
sha256 = "{readme}"

[source.unihan]
repository = "https://example.invalid/ucd/"
version = "18.0.0"
license = "Unicode-3.0"

[[source.unihan.files]]
path = "Unihan.zip"
url = "https://example.invalid/Unihan.zip"
sha256 = "{unihan}"
"#,
        table = sha256_hex(TABLE.as_bytes()),
        readme = sha256_hex(README.as_bytes()),
        unihan = sha256_hex(&unihan_zip()),
    );
    std::fs::write(root.join("data/sources.toml"), sources).unwrap();
}

#[test]
fn builds_pack_with_license_and_notice_and_verifies_it() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let outcome = run(root.path(), &upstream(README), true).unwrap();

    let dir = root.path().join("build/packs/quick");
    assert_eq!(outcome.pack_path, dir.join("quick.pack"));
    let pack = Pack::from_bytes(&std::fs::read(&outcome.pack_path).unwrap()).unwrap();
    assert_eq!(pack.scheme.id, "quick");
    assert_eq!(pack.code_table.lookup("ar")[0].text, '問');

    let license = std::fs::read_to_string(dir.join("LICENSE")).unwrap();
    assert!(license.contains("Public Domain"), "{license}");
    assert!(license.contains(STATEMENT), "{license}");
    let notice = std::fs::read_to_string(dir.join("NOTICE")).unwrap();
    assert!(
        notice.contains("346026b8ccf4ffdfbe424400edb6a588c0c5ba8e"),
        "{notice}"
    );
    assert!(notice.contains(&sha256_hex(TABLE.as_bytes())), "{notice}");

    let report = outcome.report.unwrap();
    assert!(report.passed(), "{report}");
    assert_eq!((report.continuous, report.extension), (3, 1));
}

#[test]
fn verify_writes_the_unihan_report_and_still_passes_with_differences() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let report = run(root.path(), &upstream(README), true)
        .unwrap()
        .report
        .unwrap();
    assert!(report.passed(), "{report}");

    let unihan = report.unihan.as_ref().unwrap();
    assert_eq!(
        (
            unihan.compared,
            unihan.differences,
            unihan.matching_cangjie5
        ),
        (2, 1, 1)
    );
    assert_eq!(
        unihan.report,
        Path::new("build/reports/unihan-kcangjie.tsv")
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join(&unihan.report)).unwrap(),
        "codepoint\tcharacter\tunihan\tlibcangjie_cangjie3\tlibcangjie_cangjie5\n\
         U+4E1F\t丟\tmgi\thgi\tmgi,xxmgi\n"
    );
    let shown = report.to_string();
    assert!(
        shown.contains(
            "Unihan kCangjie：比對 2 字，差異 1 字（其中 1 字等於五代碼），報表 build/reports/unihan-kcangjie.tsv"
        ),
        "{shown}"
    );
}

#[test]
fn build_without_verify_does_not_fetch_unihan() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let mut files = upstream(README);
    files.0.remove("https://example.invalid/Unihan.zip");
    run(root.path(), &files, false).unwrap();
}

#[test]
fn second_build_uses_the_cache_and_gives_the_same_bytes() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let first = run(root.path(), &upstream(README), false).unwrap();
    let first_bytes = std::fs::read(&first.pack_path).unwrap();
    let second = run(root.path(), &Offline, false).unwrap();
    assert_eq!(std::fs::read(&second.pack_path).unwrap(), first_bytes);
    assert_eq!(first.sha256, second.sha256);
    assert!(second.report.is_none());
}

#[test]
fn tampered_cached_table_fails_the_build() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    run(root.path(), &upstream(README), false).unwrap();
    let cached = walk(&root.path().join(".cache/sources"))
        .into_iter()
        .find(|p| p.ends_with("data/table.txt"))
        .unwrap();
    std::fs::write(&cached, TABLE.replace("anr", "abr")).unwrap();
    let err = run(root.path(), &Offline, false).unwrap_err();
    assert!(matches!(err, Error::ChecksumMismatch { .. }), "{err}");
}

#[test]
fn license_statement_missing_from_upstream_file_fails_the_build() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), "Licensed under the GPL.");
    let err = run(root.path(), &upstream(README), false).unwrap_err();
    assert!(matches!(err, Error::Invalid { .. }), "{err}");
}

fn walk(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            out.extend(walk(&path));
        } else {
            out.push(path);
        }
    }
    out
}

#[test]
fn packed_source_without_license_statement_fails_the_build() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let path = root.path().join("data/sources.toml");
    let sources = std::fs::read_to_string(&path).unwrap();
    let without: String = sources
        .lines()
        .filter(|l| !l.starts_with("license_statement"))
        .map(|l| format!("{l}\n"))
        .collect();
    std::fs::write(&path, without).unwrap();
    let err = run(root.path(), &upstream(README), false).unwrap_err();
    assert!(err.to_string().contains("license_statement"), "{err}");
}
