use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

use qim_core::Pack;
use qim_databuild::sources::{Fetch, sha256_hex};
use qim_databuild::{Error, run};

const TABLE: &str = "\
日 日 1 1 0 0 1 0 0 0 0 a a NA 23061
曰 曰 1 1 0 0 1 0 0 0 0 a,ax a,xa NA 23060
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

/// 日同碼、丟異碼、㐀只在 Unihan：比對 2 字、差異 1 字。日有年級，曰沒有（同 Unihan 18.0.0）。
const KCANGJIE: &str = "\
# Unihan_DictionaryLikeData.txt
U+3400\tkCangjie\tTM
U+4E1F\tkCangjie\tMGI
U+65E5\tkCangjie\tA
U+65E5\tkGradeLevel\t1
";

/// repo 裡存的 Unicode License V3 全文（這裡只取開頭）。
const UNICODE_LICENSE: &str = "\
UNICODE LICENSE V3

COPYRIGHT AND PERMISSION NOTICE

Copyright © 1991-2026 Unicode, Inc.
";

const UNICODE_LICENSE_PATH: &str = "data/licenses/Unicode-3.0.txt";

const DUMP_FILE: &str = "zh_yuewiki-20261101-pages-articles.xml.bz2";

/// 連打字集只有日曰問丟，所以拿它們組 3–5 字的句子：1,344 句，約 5% 落在 test。
/// 另有一句簡體（`s2hk` 轉成繁體才轉得了碼），與 [`DROPPED`] 句含字集外字元的句子。
fn wiki_articles() -> Vec<String> {
    let chars = ['日', '曰', '問', '丟'];
    let mut sentences: Vec<String> = vec![String::new()];
    let mut all = Vec::new();
    for _ in 0..5 {
        sentences = sentences
            .iter()
            .flat_map(|s| chars.iter().map(move |c| format!("{s}{c}")))
            .collect();
        if sentences[0].chars().count() >= 3 {
            all.extend(sentences.iter().cloned());
        }
    }
    let mut articles: Vec<String> = all
        .chunks(100)
        .map(|chunk| {
            format!(
                "'''{}'''。[[File:x.png|thumb|日日]]{{{{話|問}}}}",
                chunk.join("，")
            )
        })
        .collect();
    articles.push("问问问问问问。日𠀀日，ATM日，2004年".into());
    articles
}

const DROPPED: usize = 3;

fn dump() -> Vec<u8> {
    let pages: String = wiki_articles()
        .iter()
        .enumerate()
        .map(|(i, text)| {
            format!(
                "<page><title>{i}</title><ns>0</ns><id>{i}</id><revision>\
                 <text xml:space=\"preserve\">{text}</text></revision></page>\n"
            )
        })
        .collect();
    let xml = format!(
        "<mediawiki xmlns=\"http://www.mediawiki.org/xml/export-0.11/\">\n{pages}</mediawiki>\n"
    );
    let mut bz = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::fast());
    bz.write_all(xml.as_bytes()).unwrap();
    bz.finish().unwrap()
}

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
        (format!("https://example.invalid/{DUMP_FILE}"), dump()),
    ]))
}

/// 在 `root` 放 sources.toml、repo 裡的速成方案檔與 Unicode 授權全文。
fn repo(root: &Path, statement: &str) {
    std::fs::create_dir_all(root.join("data/licenses")).unwrap();
    std::fs::write(root.join(UNICODE_LICENSE_PATH), UNICODE_LICENSE).unwrap();
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
license_text = "{UNICODE_LICENSE_PATH}"

[[source.unihan.files]]
path = "Unihan.zip"
url = "https://example.invalid/Unihan.zip"
sha256 = "{unihan}"

[source.yue_wiki]
repository = "https://dumps.example.invalid/zh_yuewiki/"
version = "20261101"
license = "CC-BY-SA-4.0"

[[source.yue_wiki.files]]
path = "{DUMP_FILE}"
url = "https://example.invalid/{DUMP_FILE}"
sha256 = "{dump}"
"#,
        table = sha256_hex(TABLE.as_bytes()),
        readme = sha256_hex(README.as_bytes()),
        unihan = sha256_hex(&unihan_zip()),
        dump = sha256_hex(&dump()),
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
    assert!(license.contains("Unicode-3.0"), "{license}");
    assert!(license.contains(UNICODE_LICENSE), "{license}");
    let notice = std::fs::read_to_string(dir.join("NOTICE")).unwrap();
    assert!(
        notice.contains("346026b8ccf4ffdfbe424400edb6a588c0c5ba8e"),
        "{notice}"
    );
    assert!(notice.contains(&sha256_hex(TABLE.as_bytes())), "{notice}");
    assert!(notice.contains(&sha256_hex(&unihan_zip())), "{notice}");

    let report = outcome.report.unwrap();
    assert!(report.passed(), "{report}");
    assert_eq!((report.continuous, report.extension), (4, 1));
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
fn build_puts_characters_with_a_grade_level_before_big5_order() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let outcome = run(root.path(), &upstream(README), false).unwrap();
    let pack = Pack::from_bytes(&std::fs::read(&outcome.pack_path).unwrap()).unwrap();
    let a: String = pack.code_table.lookup("a").iter().map(|e| e.text).collect();
    // Big5 排序值曰 23060 < 日 23061，日有年級所以在前。
    assert_eq!(a, "日曰𠀀");
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

/// 改 sources.toml 的某些行。
fn edit_sources(root: &Path, edit: impl Fn(&str) -> String) {
    let path = root.join("data/sources.toml");
    let sources = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, edit(&sources)).unwrap();
}

#[test]
fn missing_license_text_file_fails_the_build() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    std::fs::remove_file(root.path().join(UNICODE_LICENSE_PATH)).unwrap();
    let err = run(root.path(), &upstream(README), false).unwrap_err();
    assert!(matches!(err, Error::Io { .. }), "{err}");
    assert!(err.to_string().contains("Unicode-3.0.txt"), "{err}");
}

#[test]
fn packed_source_without_any_license_fails_the_build() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    edit_sources(root.path(), |s| {
        s.lines()
            .filter(|l| !l.starts_with("license_text"))
            .map(|l| format!("{l}\n"))
            .collect()
    });
    let err = run(root.path(), &upstream(README), false).unwrap_err();
    assert!(err.to_string().contains("license_text"), "{err}");
}

#[test]
fn packed_source_with_both_license_forms_fails_the_build() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    edit_sources(root.path(), |s| {
        s.replace(
            "license_file = \"data/README.table.md\"",
            &format!(
                "license_file = \"data/README.table.md\"\nlicense_text = \"{UNICODE_LICENSE_PATH}\""
            ),
        )
    });
    let err = run(root.path(), &upstream(README), false).unwrap_err();
    assert!(err.to_string().contains("license_text"), "{err}");
}

fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap_or_else(|e| panic!("{rel}：{e}"))
}

#[test]
fn wiki_dump_becomes_bench_sentences_and_training_splits() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let outcome = run(root.path(), &upstream(README), true).unwrap();

    let bench = read(root.path(), "bench/corpus/yue-wiki.tsv");
    let train = read(root.path(), "build/corpus/yue-wiki/train.txt");
    let dev = read(root.path(), "build/corpus/yue-wiki/dev.txt");
    let code = |c| match c {
        '日' | '曰' => "a",
        '問' => "ar",
        '丟' => "mi",
        other => panic!("連打字集外的字：{other}"),
    };
    let mut bench_sentences = Vec::new();
    for line in bench.lines() {
        let (codes, sentence) = line.split_once('\t').unwrap();
        let expected: Vec<&str> = sentence.chars().map(code).collect();
        assert_eq!(codes, expected.join(" "), "{line}");
        bench_sentences.push(sentence);
    }
    for sentence in train.lines().chain(dev.lines()) {
        sentence.chars().for_each(|c| {
            code(c);
        });
        assert!(
            !bench_sentences.contains(&sentence),
            "test 句漏進訓練：{sentence}"
        );
    }

    // 1,344 句加簡體轉成的那句，每句只出現一次；「日日」只在檔案說明裡，被丟掉了。
    let [summary] = outcome.corpora.as_slice() else {
        panic!("{:?}", outcome.corpora)
    };
    assert_eq!(summary.name, "yue-wiki");
    let counts = summary.counts;
    assert_eq!(counts.train + counts.dev + counts.test, 1_345);
    assert_eq!(counts.dropped, DROPPED);
    assert_eq!(bench.lines().count(), counts.test);
    assert!((30..=110).contains(&counts.test), "{counts:?}");
    assert!(
        [&bench, &train, &dev]
            .iter()
            .any(|s| s.contains("問問問問問問")),
        "簡體句沒有轉成繁體"
    );

    let attribution = read(root.path(), "bench/corpus/ATTRIBUTION.md");
    for expected in [
        "CC BY-SA 4.0",
        "yue-wiki.tsv",
        "20261101",
        DUMP_FILE,
        &sha256_hex(&dump()),
    ] {
        assert!(
            attribution.contains(expected),
            "缺 {expected}：\n{attribution}"
        );
    }

    let report = outcome.report.unwrap();
    assert!(report.passed(), "{report}");
    let shown = report.to_string();
    for expected in [
        format!(
            "語料 yue-wiki：train {} 句、dev {} 句、test {} 句（不重複），其中 {} 句進 bench/corpus/yue-wiki.tsv；含字集外字元丟掉 {DROPPED} 句",
            counts.train, counts.dev, counts.test, counts.test
        ),
        "[通過] 語料轉碼：yue-wiki 失敗 0 句".into(),
        "[通過] 語料不洩漏：yue-wiki test 句原樣出現在 train 0 句".into(),
        "[通過] 語料決定性：yue-wiki 建兩次 sha256 相同".into(),
    ] {
        assert!(shown.contains(&expected), "缺 {expected}：\n{shown}");
    }
}

#[test]
fn corpus_build_is_deterministic_and_works_offline_from_the_cache() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    run(root.path(), &upstream(README), false).unwrap();
    let files = [
        "bench/corpus/yue-wiki.tsv",
        "bench/corpus/ATTRIBUTION.md",
        "build/corpus/yue-wiki/train.txt",
        "build/corpus/yue-wiki/dev.txt",
    ];
    let first: Vec<String> = files.iter().map(|f| read(root.path(), f)).collect();
    run(root.path(), &Offline, false).unwrap();
    let second: Vec<String> = files.iter().map(|f| read(root.path(), f)).collect();
    assert_eq!(first, second);
}
