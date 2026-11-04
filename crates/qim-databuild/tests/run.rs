use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::Path;

use qim_core::{
    CangjieVersion, Engine, LanguageModelPack, LexiconPack, Pack, RadicalHintPack, SimplifierPack,
};
use qim_databuild::Error;
use qim_databuild::sources::{Fetch, sha256_hex};

const TABLE: &str = "\
日 日 1 1 0 0 1 0 0 0 0 a a NA 23061
曰 曰 1 1 0 0 1 0 0 0 0 a,ax a,xa NA 23060
問 问 1 1 0 0 1 0 0 0 0 anr anr NA 20001
丟 丢 1 1 0 0 1 0 0 0 0 hgi mgi,xxmgi NA 22813
門 门 1 1 0 0 1 0 0 0 0 an an NA 20000
口 口 1 1 0 0 1 0 0 0 0 r r NA 20100
晒 晒 1 1 0 0 1 0 0 0 0 amcw amcw NA 0
搵 搵 1 1 0 0 1 0 0 0 0 qawt qawt NA 0
吓 吓 1 1 0 0 1 0 0 0 0 rmy rmy NA 0
郁 郁 1 1 0 0 1 0 0 0 0 kbnl kbnl NA 0
喂 喂 1 1 0 0 1 0 0 0 0 rwmv rwmv NA 0
涌 涌 1 1 0 0 1 0 0 0 0 enib enib NA 0
咸 咸 1 1 0 0 1 0 0 0 0 ihmr ihmr NA 0
响 响 1 1 0 0 1 0 0 0 0 rhbr rhbr NA 0
仆 仆 1 1 0 0 1 0 0 0 0 oy oy NA 0
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

/// repo 裡存的 Creative Commons 全文（這裡只取開頭），語料的字頻進詞庫資料包時用。
const CC_LICENSES: [(&str, &str); 3] = [
    (
        "data/licenses/CC-BY-SA-4.0.txt",
        "Attribution-ShareAlike 4.0 International",
    ),
    (
        "data/licenses/CC-BY-4.0.txt",
        "Attribution 4.0 International",
    ),
    ("data/licenses/CC0-1.0.txt", "CC0 1.0 Universal"),
];

/// rime-essay 的 `essay.txt`：門口、問日、日日收進詞庫；單字、低於門檻、含擴展區的字的不收。
const ESSAY: &str = "日\t71052\n門口\t9435\n問日\t300\n日日\t100\n丟丟\t99\n日𠀀\t500\n";

/// repo 裡的粵語口語詞表：口門收進詞庫（rime-essay 沒有）。
const COLLOQUIAL: &str = "# 粵語口語詞表\n\n口門\t300\n";

/// rime-essay 的 `LICENSE`（這裡只取開頭）。
const LGPL: &str = "\
                   GNU LESSER GENERAL PUBLIC LICENSE
                       Version 3, 29 June 2007
";

/// OpenCC 的字典（檔名, 內容）：只有「門」「問」有簡體，香港異體「綫」先轉回「線」。
const OPENCC_DICTIONARIES: [(&str, &str); 5] = [
    (
        "data/dictionary/CJK_Compatibility_Ideographs.txt",
        "# File\n",
    ),
    ("data/dictionary/TSPhrases.txt", "乾隆\t乾隆\n"),
    (
        "data/dictionary/TSCharacters.txt",
        "門\t门\n問\t问\n線\t线\n",
    ),
    ("data/dictionary/HKVariants.txt", "線\t綫\n"),
    ("data/dictionary/HKVariantsRevPhrases.txt", ""),
];

/// OpenCC 的 `LICENSE`（這裡只取開頭）。
const APACHE: &str = "\
                                 Apache License
                           Version 2.0, January 2004
";

const DUMP_FILE: &str = "zh_yuewiki-20261101-pages-articles.xml.bz2";
const ZH_DUMP_FILE: &str = "zhwiki-20220501-pages-articles1.xml-p1p187712.bz2";
const HKCANCOR_FILE: &str = "data/hkcancor-utf8.zip";
const CV_YUE_FILE: &str = "server/data/yue/sentence-collector.txt";
const CV_ZH_HK_FILE: &str = "server/data/zh-HK/sentence-collector.txt";

/// 下載快取放在暫存 repo 裡的 `cache/`。
fn run(root: &Path, fetch: &dyn Fetch, verify: bool) -> Result<qim_databuild::Outcome, Error> {
    qim_databuild::run(root, &root.join("cache"), fetch, verify)
}

/// 連打字集只有日曰問丟：它們組成的長度 `len` 的所有句子，依固定順序。
fn sentences_of_length(len: usize) -> Vec<String> {
    let chars = ['日', '曰', '問', '丟'];
    let mut sentences = vec![String::new()];
    for _ in 0..len {
        sentences = sentences
            .iter()
            .flat_map(|s| chars.iter().map(move |c| format!("{s}{c}")))
            .collect();
    }
    sentences
}

/// 連打字集只有日曰問丟，所以拿它們組 3–5 字的句子：1,344 句，約 5% 落在 test。
/// 另有一句簡體：粵語維基只經 `t2hk`，不轉簡體，所以和其他含字集外字元的句子一樣整句丟掉，共 [`DROPPED`] 句。
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

const DROPPED: usize = 4;

/// 中文維基：長度 6 的句子寫成簡體（问），`s2hk` 轉回繁體才轉得了碼。
fn zh_wiki_articles() -> Vec<String> {
    sentences_of_length(6)
        .chunks(100)
        .map(|chunk| chunk.join("。").replace('問', "问"))
        .collect()
}

fn dump() -> Vec<u8> {
    dump_of(&wiki_articles())
}

fn zh_dump() -> Vec<u8> {
    dump_of(&zh_wiki_articles())
}

fn dump_of(articles: &[String]) -> Vec<u8> {
    let pages: String = articles
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

/// HKCanCor 的一個轉寫檔：一個話輪一個 `<sent>`，`<sent_tag>` 裡一行一詞（`詞/詞性/粵拼/`）。
fn hkcancor_transcript(utterances: &[String]) -> String {
    let sents: String = utterances
        .iter()
        .map(|u| {
            let words: String = u
                .chars()
                .map(|c| format!("\t\t\t{c}/v/aa1/\n"))
                .collect();
            format!(
                "\t<sent>\n\t\t<sent_head>\n\t\t\tA:\n\t\t</sent_head>\n\t\t<sent_tag>\n\
                 {words}\t\t\t。/w/VQ1/\n\t\t</sent_tag>\n\t\t<sent_tran>\n\n\t\t</sent_tran>\n\t</sent>\n"
            )
        })
        .collect();
    format!("<info>\n\t1-TN-107\n\tINFO-END\n</info>\n{sents}")
}

/// 長度 7 的前 2,000 句分成兩個轉寫檔；另有一個夾英文詞的話輪，整句丟掉。
fn hkcancor_zip() -> Vec<u8> {
    let mut utterances: Vec<String> = sentences_of_length(7).into_iter().take(2000).collect();
    let second = utterances.split_off(1000);
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default();
    zip.add_directory("utf8/", options).unwrap();
    for (name, text) in [
        ("utf8/FC-001_v2", hkcancor_transcript(&utterances)),
        ("utf8/FC-002_v", hkcancor_transcript(&second)),
        ("README", "They are released under a CC BY license.".into()),
    ] {
        zip.start_file(name, options).unwrap();
        zip.write_all(text.as_bytes()).unwrap();
    }
    zip.start_file("utf8/FC-003_v", options).unwrap();
    zip.write_all(
        "<sent>\n<sent_tag>\n日/v/aa1/\nOK/e/ou1kei1/\n</sent_tag>\n</sent>\n".as_bytes(),
    )
    .unwrap();
    zip.finish().unwrap().into_inner()
}

/// Common Voice：長度 8 的前 2,000 句，一半在 yue、一半在 zh-HK，一行一句。
/// yue 另有前處理保留的粵語用字開頭的 8 字句，每字 10 句，總會有落在 train 的。
fn common_voice() -> (String, String) {
    let all = sentences_of_length(8);
    let lines = |range: std::ops::Range<usize>| -> String {
        all[range].iter().map(|s| format!("{s}\n")).collect()
    };
    let kept: String = qim_databuild::corpora::CANTONESE_KEPT
        .iter()
        .flat_map(|k| {
            sentences_of_length(7)
                .into_iter()
                .take(10)
                .map(move |s| format!("{k}{s}\n"))
        })
        .collect();
    (lines(0..1000) + &kept, lines(1000..2000))
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
    fn fetch(&self, url: &str, out: &mut dyn Write) -> Result<(), String> {
        let body = self.0.get(url).ok_or_else(|| format!("沒有 {url}"))?;
        out.write_all(body).map_err(|e| e.to_string())
    }
}

struct Offline;

impl Fetch for Offline {
    fn fetch(&self, url: &str, _out: &mut dyn Write) -> Result<(), String> {
        Err(format!("離線，不該連 {url}"))
    }
}

fn upstream(readme: &str) -> Files {
    Files(
        BTreeMap::from([
            ("https://example.invalid/table.txt".into(), TABLE.into()),
            (
                "https://example.invalid/README.table.md".into(),
                readme.into(),
            ),
            ("https://example.invalid/Unihan.zip".into(), unihan_zip()),
            (format!("https://example.invalid/{DUMP_FILE}"), dump()),
            (format!("https://example.invalid/{ZH_DUMP_FILE}"), zh_dump()),
            (
                "https://example.invalid/hkcancor.zip".into(),
                hkcancor_zip(),
            ),
            (
                "https://example.invalid/cv-yue.txt".into(),
                common_voice().0.into(),
            ),
            (
                "https://example.invalid/cv-zh-HK.txt".into(),
                common_voice().1.into(),
            ),
            ("https://example.invalid/essay.txt".into(), ESSAY.into()),
            ("https://example.invalid/essay-LICENSE".into(), LGPL.into()),
            (
                "https://example.invalid/opencc-LICENSE".into(),
                APACHE.into(),
            ),
        ])
        .into_iter()
        .chain(OPENCC_DICTIONARIES.iter().map(|(path, text)| {
            (
                format!("https://example.invalid/opencc/{path}"),
                text.as_bytes().to_vec(),
            )
        }))
        .collect(),
    )
}

/// 在 `root` 放 sources.toml、repo 裡的速成方案檔與 Unicode 授權全文。
fn repo(root: &Path, statement: &str) {
    std::fs::create_dir_all(root.join("data/licenses")).unwrap();
    std::fs::write(root.join(UNICODE_LICENSE_PATH), UNICODE_LICENSE).unwrap();
    for (path, text) in CC_LICENSES {
        std::fs::write(root.join(path), text).unwrap();
    }
    std::fs::create_dir_all(root.join("data/lexicon")).unwrap();
    std::fs::write(root.join("data/lexicon/yue-colloquial.tsv"), COLLOQUIAL).unwrap();
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

[source.libcangjie.license_text.statement]
file = "data/README.table.md"
quote = "{statement}"

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
license_text = {{ repo_file = "{UNICODE_LICENSE_PATH}" }}

[[source.unihan.files]]
path = "Unihan.zip"
url = "https://example.invalid/Unihan.zip"
sha256 = "{unihan}"

[source.yue_wiki]
repository = "https://dumps.example.invalid/zh_yuewiki/"
version = "20261101"
license = "CC-BY-SA-4.0"
license_text = {{ repo_file = "data/licenses/CC-BY-SA-4.0.txt" }}

[[source.yue_wiki.files]]
path = "{DUMP_FILE}"
url = "https://example.invalid/{DUMP_FILE}"
sha256 = "{dump}"

[source.zh_wiki]
repository = "https://archive.example.invalid/zhwiki-20220501"
version = "20220501"
license = "CC-BY-SA-4.0"
license_text = {{ repo_file = "data/licenses/CC-BY-SA-4.0.txt" }}

[[source.zh_wiki.files]]
path = "{ZH_DUMP_FILE}"
url = "https://example.invalid/{ZH_DUMP_FILE}"
sha256 = "{zh_dump}"

[source.hkcancor]
repository = "https://example.invalid/hkcancor"
version = "39aeadf920e0b5ca93d0ad7792c59e740e7bdd65"
license = "CC-BY-4.0"
license_text = {{ repo_file = "data/licenses/CC-BY-4.0.txt" }}

[[source.hkcancor.files]]
path = "{HKCANCOR_FILE}"
url = "https://example.invalid/hkcancor.zip"
sha256 = "{hkcancor}"

[source.common_voice]
repository = "https://example.invalid/common-voice"
version = "ec845c798ead7144f2acc95fd8955e9d3995c7d0"
license = "CC0-1.0"
license_text = {{ repo_file = "data/licenses/CC0-1.0.txt" }}

[[source.common_voice.files]]
path = "{CV_YUE_FILE}"
url = "https://example.invalid/cv-yue.txt"
sha256 = "{cv_yue}"

[[source.common_voice.files]]
path = "{CV_ZH_HK_FILE}"
url = "https://example.invalid/cv-zh-HK.txt"
sha256 = "{cv_zh_hk}"

[source.rime_essay]
repository = "https://example.invalid/rime-essay"
version = "054920de4f54c9e5994276a96a4fc2a35cb51aa3"
license = "LGPL-3.0"
license_text = {{ upstream_file = "LICENSE" }}

[[source.rime_essay.files]]
path = "essay.txt"
url = "https://example.invalid/essay.txt"
sha256 = "{essay}"

[[source.rime_essay.files]]
path = "LICENSE"
url = "https://example.invalid/essay-LICENSE"
sha256 = "{lgpl}"

[source.opencc]
repository = "https://example.invalid/OpenCC"
version = "fae2a0a911c30764e669a7d8b0591cd8e852934a"
license = "Apache-2.0"
license_text = {{ upstream_file = "LICENSE" }}

[[source.opencc.files]]
path = "LICENSE"
url = "https://example.invalid/opencc-LICENSE"
sha256 = "{apache}"
{opencc}
"#,
        table = sha256_hex(TABLE.as_bytes()),
        readme = sha256_hex(README.as_bytes()),
        unihan = sha256_hex(&unihan_zip()),
        dump = sha256_hex(&dump()),
        zh_dump = sha256_hex(&zh_dump()),
        hkcancor = sha256_hex(&hkcancor_zip()),
        cv_yue = sha256_hex(common_voice().0.as_bytes()),
        cv_zh_hk = sha256_hex(common_voice().1.as_bytes()),
        essay = sha256_hex(ESSAY.as_bytes()),
        lgpl = sha256_hex(LGPL.as_bytes()),
        apache = sha256_hex(APACHE.as_bytes()),
        opencc = OPENCC_DICTIONARIES
            .iter()
            .map(|(path, text)| {
                format!(
                    "\n[[source.opencc.files]]\npath = \"{path}\"\n\
                     url = \"https://example.invalid/opencc/{path}\"\nsha256 = \"{}\"\n",
                    sha256_hex(text.as_bytes())
                )
            })
            .collect::<String>(),
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
    assert_eq!((report.continuous, report.extension), (15, 1));
}

#[test]
fn builds_lexicon_pack_named_by_the_scheme_with_its_own_license_and_notice() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let outcome = run(root.path(), &upstream(README), false).unwrap();

    let dir = root.path().join("build/packs/quick-lexicon");
    assert_eq!(outcome.lexicon_path, dir.join("quick-lexicon.pack"));
    let bytes = std::fs::read(&outcome.lexicon_path).unwrap();
    let lexicon = LexiconPack::from_bytes(&bytes).unwrap();
    assert_eq!(lexicon.id, "quick-lexicon");
    let words = |code: &str| -> Vec<String> {
        lexicon
            .lexicon
            .lookup(code)
            .iter()
            .map(|w| w.text.clone())
            .collect()
    };
    assert_eq!(words("ara"), ["問日"]);
    assert_eq!(words("aa"), ["日日"]);
    assert_eq!(words("ran"), ["口門"]);
    assert!(lexicon.lexicon.char_frequency('日') > 0);
    let c = outcome.lexicon.counts;
    assert_eq!(
        (
            c.entries,
            c.single_characters,
            c.below_threshold,
            c.outside_continuous,
            c.words,
            c.colloquial,
        ),
        (6, 1, 1, 1, 4, 1)
    );
    // 速成資料包指名它，和語言模型資料包合起來建得出 Engine。
    let quick = std::fs::read(&outcome.pack_path).unwrap();
    let language_model = std::fs::read(&outcome.language_model_path).unwrap();
    assert!(Engine::from_pack_bytes(&quick, Some(&bytes), Some(&language_model)).is_ok());

    let license = std::fs::read_to_string(dir.join("LICENSE")).unwrap();
    assert!(license.contains("LGPL-3.0"), "{license}");
    assert!(license.contains(LGPL.trim_end()), "{license}");
    for (_, text) in CC_LICENSES {
        assert!(license.contains(text), "{license}");
    }
    // 速成資料包的授權不混進來。
    assert!(!license.contains(STATEMENT), "{license}");
    let notice = std::fs::read_to_string(dir.join("NOTICE")).unwrap();
    assert!(notice.contains(&sha256_hex(ESSAY.as_bytes())), "{notice}");
    assert!(notice.contains(&sha256_hex(&zh_dump())), "{notice}");
    assert!(notice.contains("Common Voice"), "{notice}");
    // 口語詞表在 repo 裡：NOTICE 列出它的路徑、sha256 與 CC0，LICENSE 有它的一段。
    let section = notice
        .split("\n\n")
        .find(|s| s.contains("data/lexicon/yue-colloquial.tsv"))
        .unwrap_or_else(|| panic!("{notice}"));
    assert!(
        section.contains(&sha256_hex(COLLOQUIAL.as_bytes())),
        "{section}"
    );
    assert!(section.contains("CC0-1.0"), "{section}");
    assert!(
        license.contains("Cantonese colloquial word list") && license.contains("License: CC0-1.0"),
        "{license}"
    );
    let quick_license =
        std::fs::read_to_string(root.path().join("build/packs/quick/LICENSE")).unwrap();
    assert!(!quick_license.contains("LGPL"), "{quick_license}");
}

#[test]
fn builds_language_model_pack_named_by_the_scheme_with_its_own_license_and_notice() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let outcome = run(root.path(), &upstream(README), false).unwrap();

    let dir = root.path().join("build/packs/zh-hant-bigram");
    assert_eq!(outcome.language_model_path, dir.join("zh-hant-bigram.pack"));
    let bytes = std::fs::read(&outcome.language_model_path).unwrap();
    let pack = LanguageModelPack::from_bytes(&bytes).unwrap();
    assert_eq!(pack.id, "zh-hant-bigram");
    // 由各語料的 train 切分數出來：有字、有相鄰字。
    assert!(outcome.language_model.characters > 0);
    assert!(outcome.language_model.bigrams > 0);

    let license = std::fs::read_to_string(dir.join("LICENSE")).unwrap();
    for (_, text) in CC_LICENSES {
        assert!(license.contains(text), "{license}");
    }
    // 詞庫與速成資料包的授權不混進來。
    assert!(!license.contains("LGPL"), "{license}");
    assert!(!license.contains(STATEMENT), "{license}");
    let notice = std::fs::read_to_string(dir.join("NOTICE")).unwrap();
    assert!(notice.contains(&sha256_hex(&zh_dump())), "{notice}");
    assert!(notice.contains("Common Voice"), "{notice}");
    assert!(notice.contains("Kneser-Ney"), "{notice}");
}

#[test]
fn builds_radical_hint_pack_named_by_the_scheme_with_the_libcangjie_license_only() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let outcome = run(root.path(), &upstream(README), false).unwrap();

    let dir = root.path().join("build/packs/cangjie-hint");
    assert_eq!(outcome.radical_hint_path, dir.join("cangjie-hint.pack"));
    let bytes = std::fs::read(&outcome.radical_hint_path).unwrap();
    assert_eq!(outcome.radical_hint_sha256, sha256_hex(&bytes));
    let pack = RadicalHintPack::from_bytes(&bytes).unwrap();
    assert_eq!(pack.id, "cangjie-hint");
    assert_eq!(pack.hints.codes('丟', CangjieVersion::Cangjie3), ["hgi"]);
    assert_eq!(
        pack.hints.codes('丟', CangjieVersion::Cangjie5),
        ["mgi", "xxmgi"]
    );

    let license = std::fs::read_to_string(dir.join("LICENSE")).unwrap();
    assert!(license.contains(STATEMENT), "{license}");
    assert!(!license.contains("Unicode-3.0"), "{license}");
    assert!(!license.contains("LGPL"), "{license}");
    let notice = std::fs::read_to_string(dir.join("NOTICE")).unwrap();
    assert!(notice.contains(&sha256_hex(TABLE.as_bytes())), "{notice}");
    assert!(!notice.contains("Unihan"), "{notice}");
}

#[test]
fn builds_the_simplifier_pack_with_its_own_license_and_notice() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let outcome = run(root.path(), &upstream(README), false).unwrap();

    let dir = root.path().join("build/packs/opencc-hk2s");
    assert_eq!(outcome.simplifier_path, dir.join("opencc-hk2s.pack"));
    let bytes = std::fs::read(&outcome.simplifier_path).unwrap();
    assert_eq!(outcome.simplifier_sha256, sha256_hex(&bytes));
    let pack = SimplifierPack::from_bytes(&bytes).unwrap();
    assert_eq!(pack.id, "opencc-hk2s");
    assert_eq!(pack.simplifier.convert("門口綫乾隆"), "门口线乾隆");

    let license = std::fs::read_to_string(dir.join("LICENSE")).unwrap();
    assert!(license.contains("Apache-2.0"), "{license}");
    assert!(license.contains(APACHE.trim_end()), "{license}");
    assert!(!license.contains("LGPL"), "{license}");
    assert!(!license.contains(STATEMENT), "{license}");
    let notice = std::fs::read_to_string(dir.join("NOTICE")).unwrap();
    assert!(
        notice.contains("fae2a0a911c30764e669a7d8b0591cd8e852934a"),
        "{notice}"
    );
    for (_, text) in OPENCC_DICTIONARIES {
        assert!(notice.contains(&sha256_hex(text.as_bytes())), "{notice}");
    }
    assert!(notice.contains("hk2s"), "{notice}");
    assert!(notice.contains("@tofu-risk"), "{notice}");
}

#[test]
fn writes_the_default_gboard_layout_for_the_web() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let outcome = run(root.path(), &upstream(README), false).unwrap();

    assert_eq!(
        outcome.layout_path,
        root.path().join("build/layouts/gboard-quick.json")
    );
    let text = std::fs::read_to_string(&outcome.layout_path).unwrap();
    assert_eq!(
        qim_layout::Layout::from_json(&text).unwrap(),
        qim_databuild::gboard::layout()
    );
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
    let first_lexicon = std::fs::read(&first.lexicon_path).unwrap();
    let first_language_model = std::fs::read(&first.language_model_path).unwrap();
    let first_radical_hint = std::fs::read(&first.radical_hint_path).unwrap();
    let second = run(root.path(), &Offline, false).unwrap();
    assert_eq!(
        std::fs::read(&second.radical_hint_path).unwrap(),
        first_radical_hint
    );
    assert_eq!(std::fs::read(&second.pack_path).unwrap(), first_bytes);
    assert_eq!(std::fs::read(&second.lexicon_path).unwrap(), first_lexicon);
    assert_eq!(
        std::fs::read(&second.language_model_path).unwrap(),
        first_language_model
    );
    assert_eq!(first.sha256, second.sha256);
    assert!(second.report.is_none());
}

#[test]
fn tampered_cached_table_fails_the_build() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    run(root.path(), &upstream(README), false).unwrap();
    let cached = walk(&root.path().join("cache/sources"))
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
fn license_statement_without_its_quote_fails_the_build() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    edit_sources(root.path(), |s| {
        s.lines()
            .filter(|l| !l.starts_with("quote"))
            .map(|l| format!("{l}\n"))
            .collect()
    });
    let err = run(root.path(), &upstream(README), false).unwrap_err();
    assert!(err.to_string().contains("quote"), "{err}");
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
fn packed_source_with_two_license_forms_fails_the_build() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    edit_sources(root.path(), |s| {
        s.replace(
            "license_text = { upstream_file = \"LICENSE\" }",
            &format!("license_text = {{ upstream_file = \"LICENSE\", repo_file = \"{UNICODE_LICENSE_PATH}\" }}"),
        )
    });
    let err = run(root.path(), &upstream(README), false).unwrap_err();
    assert!(matches!(err, Error::Invalid { .. }), "{err}");
    assert!(err.to_string().contains("license_text"), "{err}");
}

#[test]
fn upstream_license_file_not_pinned_in_files_fails_the_build() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    edit_sources(root.path(), |s| {
        s.replace("upstream_file = \"LICENSE\"", "upstream_file = \"COPYING\"")
    });
    let err = run(root.path(), &upstream(README), false).unwrap_err();
    assert!(err.to_string().contains("COPYING"), "{err}");
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

    // 1,344 句每句只出現一次；簡體那句與「日日」（只在檔案說明裡）被丟掉了。
    let summary = outcome
        .corpora
        .iter()
        .find(|c| c.name == "yue-wiki")
        .unwrap();
    let counts = summary.counts;
    assert_eq!(counts.train + counts.dev + counts.test, 1_344);
    assert_eq!(counts.dropped, DROPPED);
    assert_eq!(bench.lines().count(), counts.test);
    assert!((30..=110).contains(&counts.test), "{counts:?}");
    assert!(
        [&bench, &train, &dev]
            .iter()
            .all(|s| !s.contains("問問問問問問")),
        "粵語維基不該轉簡體"
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
fn dev_split_becomes_tuning_items_in_the_bench_format() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    run(root.path(), &upstream(README), false).unwrap();

    for name in ["yue-wiki", "hkcancor"] {
        let dev = read(root.path(), &format!("build/corpus/{name}/dev.txt"));
        let items = read(root.path(), &format!("build/corpus/{name}/dev.tsv"));
        // 不重複的 dev 句（樣本小於上限，全收），格式和基準檔相同。
        let expected: BTreeSet<&str> = dev.lines().collect();
        let mut sentences = BTreeSet::new();
        for line in items.lines() {
            let (codes, sentence) = line.split_once('\t').unwrap();
            assert_eq!(codes.split(' ').count(), sentence.chars().count(), "{line}");
            assert!(sentences.insert(sentence), "重複：{sentence}");
        }
        assert!(!sentences.is_empty(), "{name} 沒有 dev 句");
        assert_eq!(sentences, expected, "{name}");
    }
}

#[test]
fn corpus_build_is_deterministic_and_works_offline_from_the_cache() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    run(root.path(), &upstream(README), false).unwrap();
    let mut files = vec![
        "bench/corpus/ATTRIBUTION.md".to_string(),
        "bench/colloquial/ATTRIBUTION.md".to_string(),
    ];
    for (dir, name) in CORPORA {
        files.push(format!("{dir}/{name}.tsv"));
        for split in ["train.txt", "dev.txt", "chars.tsv"] {
            files.push(format!("build/corpus/{name}/{split}"));
        }
    }
    let first: Vec<String> = files.iter().map(|f| read(root.path(), f)).collect();
    run(root.path(), &Offline, false).unwrap();
    let second: Vec<String> = files.iter().map(|f| read(root.path(), f)).collect();
    assert!(first.iter().all(|f| !f.is_empty()));
    assert_eq!(first, second);
}

/// 每個語料的基準目錄與名字。
const CORPORA: [(&str, &str); 4] = [
    ("bench/corpus", "yue-wiki"),
    ("bench/corpus", "zh-wiki"),
    ("bench/colloquial", "hkcancor"),
    ("bench/colloquial", "common-voice-yue"),
];

/// 一個語料的檔案：基準句、train、dev、字頻。
fn corpus_files(root: &Path, dir: &str, name: &str) -> (Vec<String>, String, String, String) {
    let bench = read(root, &format!("{dir}/{name}.tsv"));
    let bench = bench
        .lines()
        .map(|line| {
            let (codes, sentence) = line.split_once('\t').unwrap();
            let expected: Vec<&str> = sentence
                .chars()
                .map(|c| match c {
                    '日' | '曰' => "a",
                    '問' => "ar",
                    '丟' => "mi",
                    // Common Voice 夾具裡前處理保留的粵語用字。
                    '晒' => "aw",
                    '搵' => "qt",
                    '吓' => "ry",
                    '郁' => "kl",
                    '喂' => "rv",
                    '涌' => "eb",
                    '咸' => "ir",
                    '响' => "rr",
                    '仆' => "oy",
                    other => panic!("{name} 有連打字集外的字：{other}"),
                })
                .collect();
            assert_eq!(codes, expected.join(" "), "{name}：{line}");
            sentence.to_string()
        })
        .collect();
    let split = |file: &str| read(root, &format!("build/corpus/{name}/{file}"));
    (
        bench,
        split("train.txt"),
        split("dev.txt"),
        split("chars.tsv"),
    )
}

#[test]
fn every_corpus_gets_bench_sentences_training_splits_and_character_counts() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let outcome = run(root.path(), &upstream(README), true).unwrap();

    // 各語料句子的長度不同：粵語維基 3–5、中文維基 6、HKCanCor 7、Common Voice 8。
    for ((dir, name), lengths) in CORPORA.iter().zip([3..=5, 6..=6, 7..=7, 8..=8]) {
        let (bench, train, dev, chars) = corpus_files(root.path(), dir, name);
        assert!(!bench.is_empty(), "{name} 沒有基準句");
        for sentence in bench
            .iter()
            .map(String::as_str)
            .chain(train.lines())
            .chain(dev.lines())
        {
            assert!(
                lengths.contains(&sentence.chars().count()),
                "{name} 混進別的句子：{sentence}"
            );
        }
        // 字頻：train 每個字出現幾次，一行「字<TAB>次數」，次數多的在前。
        let mut expected: BTreeMap<char, usize> = BTreeMap::new();
        for c in train.lines().flat_map(str::chars) {
            *expected.entry(c).or_default() += 1;
        }
        let counted: Vec<(char, usize)> = chars
            .lines()
            .map(|line| {
                let (c, n) = line.split_once('\t').unwrap();
                (c.parse().unwrap(), n.parse().unwrap())
            })
            .collect();
        assert_eq!(
            counted.iter().copied().collect::<BTreeMap<_, _>>(),
            expected,
            "{name}"
        );
        assert!(
            counted.windows(2).all(|w| w[0].1 >= w[1].1),
            "{name}：{chars}"
        );
    }

    let summary = |name: &str| outcome.corpora.iter().find(|c| c.name == name).unwrap();
    assert_eq!(
        summary("hkcancor").counts.dropped,
        1,
        "夾英文詞的話輪整句丟掉"
    );
    assert_eq!(
        summary("hkcancor").bench,
        Path::new("bench/colloquial/hkcancor.tsv")
    );

    let wiki = read(root.path(), "bench/corpus/ATTRIBUTION.md");
    for expected in ["CC BY-SA 4.0", "zh-wiki.tsv", "20220501", ZH_DUMP_FILE] {
        assert!(wiki.contains(expected), "缺 {expected}：\n{wiki}");
    }
    let colloquial = read(root.path(), "bench/colloquial/ATTRIBUTION.md");
    for expected in [
        "hkcancor.tsv",
        "K. K. Luke and May L.Y. Wong (2015)",
        "https://creativecommons.org/licenses/by/4.0/",
        "39aeadf920e0b5ca93d0ad7792c59e740e7bdd65",
        "common-voice-yue.tsv",
        "ec845c798ead7144f2acc95fd8955e9d3995c7d0",
        CV_ZH_HK_FILE,
        "CC0-1.0",
    ] {
        assert!(
            colloquial.contains(expected),
            "缺 {expected}：\n{colloquial}"
        );
    }
    assert!(!colloquial.contains("Wikipedia"), "{colloquial}");

    let report = outcome.report.unwrap();
    assert!(report.passed(), "{report}");
    let shown = report.to_string();
    for (_, name) in CORPORA {
        for expected in [
            format!("[通過] 語料轉碼：{name} 失敗 0 句"),
            format!("[通過] 語料不洩漏：{name} test 句原樣出現在 train 0 句"),
            format!("[通過] 語料決定性：{name} 建兩次 sha256 相同"),
        ] {
            assert!(shown.contains(&expected), "缺 {expected}：\n{shown}");
        }
    }
    // 粵語用字只在 Common Voice 的夾具裡。
    assert!(shown.contains("[通過] 前處理保留粵語用字：晒 "), "{shown}");
    assert!(
        shown.contains("[通過] 字形用常見寫法：對照表 1 組（揾→搵）"),
        "{shown}"
    );
    for expected in ["Conversion: OpenCC s2hk", "Conversion: OpenCC t2hk"] {
        assert!(wiki.contains(expected), "缺 {expected}：\n{wiki}");
    }
    assert!(!colloquial.contains("OpenCC s2hk"), "{colloquial}");
}

#[test]
fn personal_userdb_is_transcoded_next_to_it_and_removed_with_it() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let fetch = upstream(README);
    let outcome = run(root.path(), &fetch, false).unwrap();
    assert_eq!(outcome.personal, None);

    std::fs::create_dir_all(root.path().join("bench/personal")).unwrap();
    std::fs::write(
        root.path().join("bench/personal/rime-userdb.txt"),
        "#@/db_name\tquick5.userdb\nanr a \t問日\tc=2 d=1 t=1\na \t日\tc=0 d=1 t=2\n",
    )
    .unwrap();
    let outcome = run(root.path(), &fetch, false).unwrap();
    assert_eq!(outcome.personal.map(|c| (c.words, c.unused)), Some((1, 1)));
    assert_eq!(
        read(root.path(), "bench/personal/rime-userdb.tsv"),
        format!(
            "# rime-userdb.txt sha256 {}\nar a\t問日\t2\n",
            sha256_hex(&std::fs::read(root.path().join("bench/personal/rime-userdb.txt")).unwrap())
        )
    );

    std::fs::remove_file(root.path().join("bench/personal/rime-userdb.txt")).unwrap();
    run(root.path(), &fetch, false).unwrap();
    assert!(!root.path().join("bench/personal/rime-userdb.tsv").exists());
}
