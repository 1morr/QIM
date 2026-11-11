use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::Path;

use qim_core::{
    CangjieVersion, EmojiPack, Engine, EnglishPack, LanguageModelPack, LexiconPack, MANIFEST_FILE,
    Manifest, Pack, PackKind, RadicalHintPack, SchemePack, SimplifierPack, WordTrigramPack,
};
use qim_databuild::sources::{Fetch, sha256_hex};
use qim_databuild::verify::{self, CheckId};
use qim_databuild::{Error, Outcome, WrittenPack, lexicon};

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
廿 廿 1 1 0 0 1 0 0 0 0 t t NA 23000
竹 竹 1 1 0 0 1 0 0 0 0 h h NA 23001
水 水 1 1 0 0 1 0 0 0 0 e e NA 23002
群 群 1 1 0 0 1 0 0 0 0 srtq srtq NA 0
羣 羣 1 1 0 0 1 0 0 0 0 srtq srtq NA 0
組 组 1 1 0 0 1 0 0 0 0 vmbm vmbm NA 0
吃 吃 1 1 0 0 1 0 0 0 0 ron ron NA 0
喫 喫 1 1 0 0 1 0 0 0 0 rdhk rdhk NA 0
飯 饭 1 1 0 0 1 0 0 0 0 oihne oihne NA 0
裏 裏 1 1 0 0 1 0 0 0 0 ywgv ywgv NA 0
裡 裡 1 1 0 0 1 0 0 0 0 lwg lwg NA 0
這 这 1 1 0 0 1 0 0 0 0 yymr yymr NA 0
着 着 1 1 0 0 1 0 0 0 0 tqbu tqbu NA 0
著 著 1 1 0 0 1 0 0 0 0 tjka tjka NA 0
顯 显 1 1 0 0 1 0 0 0 0 afmbc afmbc NA 0
臺 台 1 1 0 0 1 0 0 0 0 grmbg grmbg NA 0
台 台 1 1 0 0 1 0 0 0 0 ir ir NA 0
门 门 1 0 1 0 1 0 0 0 0 ls lis NA 0
笑 笑 1 1 0 0 1 0 0 0 0 hhhk hhhk NA 0
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
/// 後面幾個詞驗建詞庫接上了寫法（M1 票 17d）：羣組（對照表）、喫飯（`t2hk`）、這裡（雙向並存）、
/// 顯着（反向的錯字）、晒日（`t2hk` 不換、`s2hk` 會換成曬）、臺日與台口（單向並存）。
const ESSAY: &str = "日\t71052\n門口\t9435\n問日\t300\n日日\t100\n丟丟\t99\n日𠀀\t500\n\
                     羣組\t500\n喫飯\t400\n這裡\t300\n顯着\t300\n顯著\t200\n晒日\t500\n\
                     臺日\t500\n台口\t300\n";

/// 粵典的詞頭表與詞頻（CRLF、首行 `Source,…`）：門日見過 7 次、曰曰沒見過，兩個都是別的來源沒收的；
/// 門口 essay 已收、OK 打不出、單字不收。
const WORDSLIST: &str = "Source,https://example.invalid/wordslist/\r\n門日,mun4 jat6\r\n曰曰*,jyut6 jyut6\r\n\
                         門口,mun4 hau2\r\nOK,ou1 kei1\r\n日,jat6\r\n";
const WORD_COUNT: &str =
    "Source,https://example.invalid/existingwordcount/\r\n門日,7\r\n門口,99\r\n";
/// repo 裡存的粵典授權全文（這裡只取開頭）。
const WORDS_HK_LICENSE: &str =
    "Copyright (c) 2015-2025, Hong Kong Lexicography Limited. All rights reserved.\n";
const WORDS_HK_LICENSE_PATH: &str = "data/licenses/words-hk-NCODL-1.0.txt";

/// repo 裡的粵語口語詞表：口門收進詞庫（rime-essay 沒有）。
const COLLOQUIAL: &str = "# 粵語口語詞表\n\n口門\t300\n";

/// rime-essay 的 `LICENSE`（這裡只取開頭）。
const LGPL: &str = "\
                   GNU LESSER GENERAL PUBLIC LICENSE
                       Version 3, 29 June 2007
";

/// OpenCC 的字典（檔名, 內容）：只有「門」「問」有簡體，香港異體「綫」先轉回「線」。
/// STCharacters 只給速成碼表判定簡體字，不進簡繁資料包。
const OPENCC_DICTIONARIES: [(&str, &str); 6] = [
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
    ("data/dictionary/STCharacters.txt", "门\t門\n问\t問\n"),
];

/// OpenCC 的 `LICENSE`（這裡只取開頭）。
const APACHE: &str = "\
                                 Apache License
                           Version 2.0, January 2004
";

/// ESDB release 的 Hunspell 字典（這裡只取幾個詞）：`the`、`walk` 展開出 `walked`，
/// `damn` 是禁忌詞，`zoo` 打不出（`z`）。
const EN_US_AFF: &str = "SET UTF-8\nNOSUGGEST !\n\nSFX D Y 1\nSFX D   0     ed         [^ey]\n";
const EN_US_DIC: &str = "4\nthe\nwalk/D\ndamn/D!\nzoo\n";

/// ESDB 的 `Copyright`（這裡只取開頭）。
const SCOWL_COPYRIGHT: &str = "Copyright 2000-2026 by Kevin Atkinson\n";

const SCOWL_ZIP: &str = "hunspell-en_US-2026.02.25.zip";

/// CLDR 的粵語標註：「笑」是 😂 的關鍵字，「門口」是 🚪 的短名，「臉」打不出（碼表沒有）。
const CLDR_YUE: &str = r#"<?xml version="1.0" encoding="UTF-8" ?>
<ldml><annotations>
<annotation cp="😂">笑 | 臉</annotation>
<annotation cp="😂" type="tts">有開心眼淚嘅臉</annotation>
<annotation cp="🚪">門 | 門口</annotation>
<annotation cp="🚪" type="tts">門口</annotation>
</annotations></ldml>
"#;
/// CLDR 的繁中標註：「笑」另有 🤣，排在粵語的 😂 後面；❤ 的 cp 沒有 U+FE0F。
const CLDR_ZH_HANT: &str = r#"<?xml version="1.0" encoding="UTF-8" ?>
<ldml><annotations>
<annotation cp="🤣">笑 | 大笑</annotation>
<annotation cp="🤣" type="tts">笑到在地上打滾</annotation>
<annotation cp="❤">日</annotation>
<annotation cp="❤" type="tts">紅心</annotation>
</annotations></ldml>
"#;
const CLDR_LICENSE: &str = "COPYRIGHT AND PERMISSION NOTICE\n\nAll other content in this repo is released under UNICODE LICENSE V3\n";
const EMOJI_TEST: &str =
    "2764 FE0F ; fully-qualified # ❤️ E0.6 red heart\n2764 ; unqualified # ❤ E0.6 red heart\n";

fn scowl_zip() -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, text) in [("en_US.aff", EN_US_AFF), ("en_US.dic", EN_US_DIC)] {
        zip.start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(text.as_bytes()).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

const DUMP_FILE: &str = "zh_yuewiki-20261101-pages-articles.xml.bz2";
const ZH_DUMP_FILE: &str = "zhwiki-20220501-pages-articles1.xml-p1p187712.bz2";
const HKCANCOR_FILE: &str = "data/hkcancor-utf8.zip";
const CV_YUE_FILE: &str = "server/data/yue/sentence-collector.txt";
const CV_ZH_HK_FILE: &str = "server/data/zh-HK/sentence-collector.txt";

/// 下載快取放在暫存 repo 裡的 `cache/`。
/// `outcome` 裡種類名是 `name` 的那份資料包。
fn written<'a>(outcome: &'a Outcome, name: &str) -> &'a WrittenPack {
    outcome
        .packs
        .iter()
        .find(|p| p.name == name)
        .unwrap_or_else(|| panic!("沒有{name}資料包"))
}

fn path<'a>(outcome: &'a Outcome, name: &str) -> &'a Path {
    &written(outcome, name).path
}

fn run(root: &Path, fetch: &dyn Fetch, verify: bool) -> Result<Outcome, Error> {
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

/// HKCanCor 分好詞的固件裡的兩字詞：rime-essay 與口語詞表都沒有，由 HKCanCor 補進詞庫。
const HKCANCOR_WORD: &str = "丟曰";

/// 長度 7 的前 2,000 句分成兩個轉寫檔；另有一個夾英文詞的話輪，整句丟掉。
/// 第四個檔是分好詞的話輪（[`HKCANCOR_WORD`] 接一個標成人名的 5 字詞，湊成和其他話輪一樣的 7 字，各兩次），給詞庫補詞。
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
    zip.start_file("utf8/FC-004_v", options).unwrap();
    let segmented: String = sentences_of_length(5)
        .iter()
        .take(64)
        .flat_map(|name| [name, name])
        .map(|name| {
            format!("<sent>\n<sent_tag>\n{HKCANCOR_WORD}/d/aa1/\n{name}/nr/aa1/\n。/w/VQ1/\n</sent_tag>\n</sent>\n")
        })
        .collect();
    zip.write_all(segmented.as_bytes()).unwrap();
    zip.start_file("utf8/FC-003_v", options).unwrap();
    zip.write_all(
        "<sent>\n<sent_tag>\n日/v/aa1/\nOK/e/ou1kei1/\n</sent_tag>\n</sent>\n".as_bytes(),
    )
    .unwrap();
    zip.finish().unwrap().into_inner()
}

/// Common Voice：長度 8 的前 2,000 句，一半在 yue、一半在 zh-HK，一行一句。
/// yue 另有前處理保留的粵語用字開頭的 8 字句，每字 10 句，總會有落在 train 的；並存寫法的
/// 「裡」（換成代表寫法裏）、「臺」（照原文）開頭與結尾的 8 字句也各 10 句。
fn common_voice() -> (String, String) {
    let all = sentences_of_length(8);
    let lines = |range: std::ops::Range<usize>| -> String {
        all[range].iter().map(|s| format!("{s}\n")).collect()
    };
    let kept: String = qim_databuild::corpora::CANTONESE_KEPT
        .iter()
        .chain(&['裡', '臺'])
        .flat_map(|k| {
            sentences_of_length(7)
                .into_iter()
                .take(10)
                .map(move |s| format!("{k}{s}\n"))
        })
        .collect();
    let ending: String = ['裡', '臺']
        .iter()
        .flat_map(|k| {
            sentences_of_length(7)
                .into_iter()
                .skip(10)
                .take(10)
                .map(move |s| format!("{s}{k}\n"))
        })
        .collect();
    let kept = kept + &ending;
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
            (format!("https://example.invalid/{SCOWL_ZIP}"), scowl_zip()),
            (
                "https://example.invalid/scowl-Copyright".into(),
                SCOWL_COPYRIGHT.into(),
            ),
            (
                "https://example.invalid/cldr-yue.xml".into(),
                CLDR_YUE.into(),
            ),
            (
                "https://example.invalid/cldr-zh_Hant.xml".into(),
                CLDR_ZH_HANT.into(),
            ),
            (
                "https://example.invalid/cldr-LICENSE".into(),
                CLDR_LICENSE.into(),
            ),
            (
                "https://example.invalid/emoji-test.txt".into(),
                EMOJI_TEST.into(),
            ),
            (
                "https://example.invalid/wordslist.csv".into(),
                WORDSLIST.into(),
            ),
            (
                "https://example.invalid/existingwordcount.csv".into(),
                WORD_COUNT.into(),
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
    std::fs::write(root.join(WORDS_HK_LICENSE_PATH), WORDS_HK_LICENSE).unwrap();
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
license = "LicenseRef-Public-Domain"

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
[source.scowl]
repository = "https://example.invalid/wordlist"
version = "rel-2026.02.25"
license = "LicenseRef-SCOWL"
license_text = {{ upstream_file = "Copyright" }}

[[source.scowl.files]]
path = "{SCOWL_ZIP}"
url = "https://example.invalid/{SCOWL_ZIP}"
sha256 = "{scowl}"

[[source.scowl.files]]
path = "Copyright"
url = "https://example.invalid/scowl-Copyright"
sha256 = "{scowl_copyright}"

[source.cldr]
repository = "https://example.invalid/cldr"
version = "96e54b273d2da4ec6fbad350ba19ff23fa24a588"
license = "Unicode-3.0"
license_text = {{ upstream_file = "LICENSE" }}

[[source.cldr.files]]
path = "common/annotations/yue.xml"
url = "https://example.invalid/cldr-yue.xml"
sha256 = "{cldr_yue}"

[[source.cldr.files]]
path = "common/annotations/zh_Hant.xml"
url = "https://example.invalid/cldr-zh_Hant.xml"
sha256 = "{cldr_zh_hant}"

[[source.cldr.files]]
path = "LICENSE"
url = "https://example.invalid/cldr-LICENSE"
sha256 = "{cldr_license}"

[source.emoji_test]
repository = "https://example.invalid/emoji/"
version = "18.0"
license = "Unicode-3.0"
license_text = {{ repo_file = "{UNICODE_LICENSE_PATH}" }}

[[source.emoji_test.files]]
path = "emoji-test.txt"
url = "https://example.invalid/emoji-test.txt"
sha256 = "{emoji_test}"

[source.words_hk]
repository = "https://example.invalid/analysis/"
version = "2026-11-11"
license = "LicenseRef-words-hk-NCODL-1.0"
license_text = {{ repo_file = "{WORDS_HK_LICENSE_PATH}" }}

[[source.words_hk.files]]
path = "wordslist.csv"
url = "https://example.invalid/wordslist.csv"
sha256 = "{wordslist}"

[[source.words_hk.files]]
path = "existingwordcount.csv"
url = "https://example.invalid/existingwordcount.csv"
sha256 = "{word_count}"
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
        scowl = sha256_hex(&scowl_zip()),
        scowl_copyright = sha256_hex(SCOWL_COPYRIGHT.as_bytes()),
        cldr_yue = sha256_hex(CLDR_YUE.as_bytes()),
        cldr_zh_hant = sha256_hex(CLDR_ZH_HANT.as_bytes()),
        cldr_license = sha256_hex(CLDR_LICENSE.as_bytes()),
        emoji_test = sha256_hex(EMOJI_TEST.as_bytes()),
        wordslist = sha256_hex(WORDSLIST.as_bytes()),
        word_count = sha256_hex(WORD_COUNT.as_bytes()),
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
    assert_eq!(path(&outcome, "速成"), dir.join("quick.pack"));
    let pack = SchemePack::from_bytes(&std::fs::read(path(&outcome, "速成")).unwrap()).unwrap();
    assert_eq!(pack.scheme.id, "quick");
    assert_eq!(pack.code_table.lookup("ar")[0].text, '問');

    let license = std::fs::read_to_string(dir.join("LICENSE")).unwrap();
    assert!(license.contains("Public Domain"), "{license}");
    assert!(license.contains(STATEMENT), "{license}");
    assert!(license.contains("Unicode-3.0"), "{license}");
    assert!(license.contains(UNICODE_LICENSE), "{license}");
    // OpenCC 判定簡體字（plan §4.1）。
    assert!(license.contains(APACHE.trim_end()), "{license}");
    let notice = std::fs::read_to_string(dir.join("NOTICE")).unwrap();
    assert!(
        notice.contains("346026b8ccf4ffdfbe424400edb6a588c0c5ba8e"),
        "{notice}"
    );
    assert!(notice.contains(&sha256_hex(TABLE.as_bytes())), "{notice}");
    assert!(notice.contains(&sha256_hex(&unihan_zip())), "{notice}");
    assert!(notice.contains("STCharacters.txt"), "{notice}");

    let report = outcome.report.unwrap();
    assert!(report.passed(), "{report}");
    assert_eq!((report.continuous, report.extension), (34, 1));
}

/// 每份資料包旁有 manifest（plan §4.6）：描述旁邊的資料包檔，授權是各來源的 SPDX 識別碼以 AND 接起、
/// 不重複；`--verify` 讀回來核對。
#[test]
fn every_pack_has_a_manifest_next_to_it() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let outcome = run(root.path(), &upstream(README), true).unwrap();
    assert_eq!(outcome.packs.len(), 10);
    let manifest_of = |name: &str| {
        let pack = written(&outcome, name);
        let dir = pack.path.parent().unwrap();
        let manifest =
            Manifest::from_json(&std::fs::read_to_string(dir.join(MANIFEST_FILE)).unwrap())
                .unwrap();
        let bytes = std::fs::read(&pack.path).unwrap();
        assert_eq!(manifest.id, pack.id, "{name}");
        assert_eq!(manifest.file_name(), format!("{}.pack", pack.id), "{name}");
        assert_eq!(manifest.sha256, pack.sha256, "{name}");
        assert_eq!(manifest.size, bytes.len() as u64, "{name}");
        assert_eq!(manifest.version, "1.0.0", "{name}");
        (manifest, bytes)
    };
    for name in [
        "速成",
        "詞庫",
        "詞庫（粵典）",
        "語言模型",
        "詞級 trigram",
        "詞級 trigram（放寬）",
        "字根提示",
        "簡繁",
        "英文",
        "emoji",
    ] {
        manifest_of(name);
    }
    let (words, _) = manifest_of("詞級 trigram");
    assert_eq!(words.kind, PackKind::WordTrigram);
    assert_eq!(
        words.license,
        "LGPL-3.0 AND CC-BY-SA-4.0 AND CC-BY-4.0 AND CC0-1.0"
    );
    let (quick, _) = manifest_of("速成");
    assert_eq!(quick.kind, PackKind::Scheme);
    assert_eq!(
        quick.license,
        "LicenseRef-Public-Domain AND Unicode-3.0 AND Apache-2.0"
    );
    let (lexicon, bytes) = manifest_of("詞庫");
    assert_eq!(lexicon.kind, PackKind::Lexicon);
    // 兩份維基都是 CC-BY-SA-4.0，口語詞表與 Common Voice 都是 CC0-1.0：各只寫一次。
    assert_eq!(
        lexicon.license,
        "LGPL-3.0 AND CC-BY-SA-4.0 AND CC-BY-4.0 AND CC0-1.0"
    );
    assert_eq!(
        LexiconPack::open(&lexicon, &bytes).unwrap().id,
        "quick-lexicon"
    );
    let (words_hk, bytes) = manifest_of("詞庫（粵典）");
    assert_eq!(words_hk.kind, PackKind::Lexicon);
    assert_eq!(
        words_hk.license,
        "LGPL-3.0 AND CC-BY-SA-4.0 AND CC-BY-4.0 AND CC0-1.0 AND LicenseRef-words-hk-NCODL-1.0"
    );
    assert_eq!(
        LexiconPack::open(&words_hk, &bytes).unwrap().id,
        "quick-lexicon-words-hk"
    );
    assert_eq!(manifest_of("emoji").0.kind, PackKind::Emoji);

    let report = outcome.report.unwrap();
    let check = report
        .checks
        .iter()
        .find(|c| c.id == CheckId::Manifests)
        .expect("verify 核對 manifest");
    assert!(check.passed, "{report}");
}

/// manifest 和旁邊的資料包對不上時 `--verify` 失敗。
#[test]
fn manifest_not_matching_its_pack_fails_the_check() {
    let pack = LexiconPack {
        id: "l".into(),
        lexicon: qim_core::Lexicon::default(),
    };
    let bytes = pack.to_bytes().unwrap();
    let good = Manifest::new(PackKind::Lexicon, "l", &bytes, "1.0.0", "CC0-1.0");
    assert!(verify::manifests(&[(good.to_json(), bytes.clone())]).passed);
    let mut bad = good.clone();
    bad.sha256 = "0".repeat(64);
    let check = verify::manifests(&[(good.to_json(), bytes.clone()), (bad.to_json(), bytes)]);
    assert!(!check.passed);
    assert!(check.detail.contains("sha256"), "{}", check.detail);
}

#[test]
fn builds_lexicon_pack_named_by_the_scheme_with_its_own_license_and_notice() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let outcome = run(root.path(), &upstream(README), false).unwrap();

    let dir = root.path().join("build/packs/quick-lexicon");
    assert_eq!(path(&outcome, "詞庫"), dir.join("quick-lexicon.pack"));
    let bytes = std::fs::read(path(&outcome, "詞庫")).unwrap();
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
    // HKCanCor train 切分的兩字詞補進來，詞頻是次數乘上換算值；標成人名的不收。
    let corpus_word = lexicon
        .lexicon
        .iter()
        .flat_map(|(_, words)| words.iter())
        .find(|w| w.text == HKCANCOR_WORD)
        .unwrap_or_else(|| panic!("詞庫沒有 {HKCANCOR_WORD}"));
    assert_eq!(corpus_word.frequency % lexicon::CORPUS_WORD_FREQUENCY, 0);
    assert!(corpus_word.frequency >= lexicon::CORPUS_MIN_COUNT * lexicon::CORPUS_WORD_FREQUENCY);
    let c = outcome.lexicon.counts;
    assert_eq!(
        (
            c.entries,
            c.single_characters,
            c.below_threshold,
            c.outside_continuous,
            c.words,
            c.colloquial,
            c.corpus,
        ),
        (14, 1, 1, 1, 12, 1, 1)
    );
    assert_eq!(outcome.lexicon.unaligned, 0);
    // 速成資料包指名它，和語言模型資料包合起來建得出 Engine。
    let quick = std::fs::read(path(&outcome, "速成")).unwrap();
    let language_model = std::fs::read(path(&outcome, "語言模型")).unwrap();
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
    assert!(
        notice.contains("words of two or more characters in that split"),
        "{notice}"
    );
    assert!(notice.contains("personal\nnames left out"), "{notice}");
    // LGPL 要寫明對來源做了什麼：錯字「顯着」改寫併進「顯著」、同頻只算一次、單向的例外（M1 票 17d）。
    for expected in ["顯着 went into 顯著", "equal frequencies", "except 合着"] {
        assert!(notice.contains(expected), "缺 {expected}：{notice}");
    }
    // 字頻乘上語料權重：NOTICE 寫出各語料的權重（M1 票 24）。
    assert!(notice.contains("hkcancor 64"), "{notice}");
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
fn lexicon_and_language_model_are_built_in_the_forms_of_the_corpus() {
    // 建詞庫與 bigram 都要接上寫法的轉換（M1 票 17d：拿掉任一處，其他測試照樣綠）。
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let outcome = run(root.path(), &upstream(README), false).unwrap();
    let lexicon = LexiconPack::from_bytes(&std::fs::read(path(&outcome, "詞庫")).unwrap())
        .unwrap()
        .lexicon;
    let mut words: BTreeMap<String, u64> = BTreeMap::new();
    for (_, list) in lexicon.iter() {
        for w in list {
            words.insert(w.text.clone(), w.frequency);
        }
    }
    let frequency = |word: &str| words.get(word).copied();
    // 對照表、`t2hk`（不是 `s2hk`：晒日不換成曬日）、反向的錯字改回。
    assert_eq!(frequency("群組"), Some(500));
    assert_eq!(frequency("吃飯"), Some(400));
    assert_eq!(frequency("晒日"), Some(500));
    assert_eq!(frequency("顯著"), Some(500));
    for gone in ["羣組", "喫飯", "顯着"] {
        assert_eq!(frequency(gone), None, "{gone}");
    }
    // 雙向：兩種寫法同頻；單向：臺日另收台日，台口不生出臺口。
    assert_eq!(
        (frequency("這裏"), frequency("這裡")),
        (Some(300), Some(300))
    );
    assert_eq!(
        (frequency("臺日"), frequency("台日")),
        (Some(500), Some(500))
    );
    assert_eq!((frequency("台口"), frequency("臺口")), (Some(300), None));
    // 語料：「裡」歸到「裏」、兩字共用字頻；「臺」照原文，有自己的字頻。
    assert!(lexicon.char_frequency('裏') > 0);
    assert_eq!(lexicon.char_frequency('裡'), lexicon.char_frequency('裏'));
    assert!(lexicon.char_frequency('臺') > 0);

    let model = LanguageModelPack::from_bytes(&std::fs::read(path(&outcome, "語言模型")).unwrap())
        .unwrap()
        .model;
    // 雙向的另一種寫法照抄代表寫法的 unigram 與前後字的 bigram。
    for (prev, c) in [(None, '裏'), (Some('裏'), '日'), (Some('日'), '裏')] {
        let other = |ch: char| if ch == '裏' { '裡' } else { ch };
        assert_eq!(
            model.log_prob(prev.map(other), other(c)),
            model.log_prob(prev, c),
            "{prev:?} {c}"
        );
    }
}

#[test]
fn builds_language_model_pack_named_by_the_scheme_with_its_own_license_and_notice() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let outcome = run(root.path(), &upstream(README), false).unwrap();

    let dir = root.path().join("build/packs/zh-hant-bigram");
    assert_eq!(path(&outcome, "語言模型"), dir.join("zh-hant-bigram.pack"));
    let bytes = std::fs::read(path(&outcome, "語言模型")).unwrap();
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
    assert!(notice.contains("zh-wiki 1, hkcancor 64"), "{notice}");
}

/// 詞級 trigram（plan §4.3）：預算版與放寬版都建，方案指名的那份在前；授權含切詞用的詞庫來源。
#[test]
fn builds_both_word_trigram_packs_with_the_named_one_first() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let outcome = run(root.path(), &upstream(README), false).unwrap();

    let ids: Vec<&str> = outcome
        .packs
        .iter()
        .filter(|p| p.name.starts_with("詞級 trigram"))
        .map(|p| p.id.as_str())
        .collect();
    assert_eq!(ids, ["zh-hant-word-trigram", "zh-hant-word-trigram-large"]);
    for id in ids {
        let dir = root.path().join("build/packs").join(id);
        let pack =
            WordTrigramPack::from_bytes(&std::fs::read(dir.join(format!("{id}.pack"))).unwrap())
                .unwrap();
        assert_eq!(pack.id, id);
        // 句首一定在詞表裡；語料切出的段也有一些。
        assert!(pack.model.id("\u{2}").is_some());
        let license = std::fs::read_to_string(dir.join("LICENSE")).unwrap();
        assert!(license.contains("LGPL"), "{license}");
        for (_, text) in CC_LICENSES {
            assert!(license.contains(text), "{license}");
        }
        assert!(!license.contains(STATEMENT), "{license}");
        let notice = std::fs::read_to_string(dir.join("NOTICE")).unwrap();
        assert!(notice.contains("rime-essay"), "{notice}");
        assert!(notice.contains("Common Voice"), "{notice}");
        assert!(notice.contains("word trigram"), "{notice}");
        assert!(notice.contains("zh-wiki 1, hkcancor 64"), "{notice}");
    }
    let summaries: Vec<&str> = outcome
        .word_trigrams
        .iter()
        .map(|s| s.id.as_str())
        .collect();
    assert_eq!(
        summaries,
        ["zh-hant-word-trigram", "zh-hant-word-trigram-large"]
    );
    assert!(outcome.word_trigrams.iter().all(|s| s.segments > 1));
}

#[test]
fn builds_radical_hint_pack_named_by_the_scheme_with_the_libcangjie_license_only() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let outcome = run(root.path(), &upstream(README), false).unwrap();

    let dir = root.path().join("build/packs/cangjie-hint");
    assert_eq!(path(&outcome, "字根提示"), dir.join("cangjie-hint.pack"));
    let bytes = std::fs::read(path(&outcome, "字根提示")).unwrap();
    assert_eq!(written(&outcome, "字根提示").sha256, sha256_hex(&bytes));
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
    assert_eq!(path(&outcome, "簡繁"), dir.join("opencc-hk2s.pack"));
    let bytes = std::fs::read(path(&outcome, "簡繁")).unwrap();
    assert_eq!(written(&outcome, "簡繁").sha256, sha256_hex(&bytes));
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
fn builds_the_english_pack_with_its_own_license_and_notice() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let outcome = run(root.path(), &upstream(README), false).unwrap();

    let dir = root.path().join("build/packs/scowl-en-us");
    assert_eq!(path(&outcome, "英文"), dir.join("scowl-en-us.pack"));
    let bytes = std::fs::read(path(&outcome, "英文")).unwrap();
    assert_eq!(written(&outcome, "英文").sha256, sha256_hex(&bytes));
    let pack = EnglishPack::from_bytes(&bytes).unwrap();
    assert_eq!(pack.id, "scowl-en-us");
    assert_eq!(
        pack.words.iter().collect::<Vec<_>>(),
        ["the", "walk", "walked"]
    );
    assert_eq!(
        (
            outcome.english.stems,
            outcome.english.taboo,
            outcome.english.untypable
        ),
        (4, 1, 1)
    );

    let license = std::fs::read_to_string(dir.join("LICENSE")).unwrap();
    assert!(license.contains("LicenseRef-SCOWL"), "{license}");
    assert!(license.contains(SCOWL_COPYRIGHT.trim_end()), "{license}");
    assert!(!license.contains("Apache"), "{license}");
    let notice = std::fs::read_to_string(dir.join("NOTICE")).unwrap();
    assert!(notice.contains("rel-2026.02.25"), "{notice}");
    assert!(notice.contains(&sha256_hex(&scowl_zip())), "{notice}");
    assert!(notice.contains("NOSUGGEST"), "{notice}");
}

#[test]
fn builds_the_emoji_pack_with_its_own_license_and_notice() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let outcome = run(root.path(), &upstream(README), false).unwrap();

    let dir = root.path().join("build/packs/cldr-emoji-yue-hant");
    assert_eq!(
        path(&outcome, "emoji"),
        dir.join("cldr-emoji-yue-hant.pack")
    );
    let bytes = std::fs::read(path(&outcome, "emoji")).unwrap();
    assert_eq!(written(&outcome, "emoji").sha256, sha256_hex(&bytes));
    let pack = EmojiPack::from_bytes(&bytes).unwrap();
    assert_eq!(pack.id, "cldr-emoji-yue-hant");
    // 粵語在前；短名在關鍵字前；❤ 換回完整寫法；「臉」打不出不收。
    assert_eq!(pack.emoji.lookup("笑"), ["😂", "🤣"]);
    assert_eq!(pack.emoji.lookup("門口"), ["🚪"]);
    assert_eq!(pack.emoji.lookup("日"), ["\u{2764}\u{FE0F}"]);
    assert!(pack.emoji.lookup("臉").is_empty());

    let license = std::fs::read_to_string(dir.join("LICENSE")).unwrap();
    assert!(license.contains(CLDR_LICENSE.trim_end()), "{license}");
    assert!(license.contains(UNICODE_LICENSE.trim_end()), "{license}");
    assert!(!license.contains("LicenseRef-SCOWL"), "{license}");
    let notice = std::fs::read_to_string(dir.join("NOTICE")).unwrap();
    assert!(
        notice.contains("96e54b273d2da4ec6fbad350ba19ff23fa24a588"),
        "{notice}"
    );
    assert!(
        notice.contains(&sha256_hex(EMOJI_TEST.as_bytes())),
        "{notice}"
    );
    assert!(notice.contains("than 6 emoji"), "{notice}");
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
    // 數字列打開時讀的那份放在旁邊（M1 票 27）。
    let number_row = std::fs::read_to_string(
        root.path()
            .join("build/layouts/gboard-quick-number-row.json"),
    )
    .unwrap();
    assert_eq!(
        qim_layout::Layout::from_json(&number_row).unwrap(),
        qim_databuild::gboard::number_row_layout()
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
    let pack = SchemePack::from_bytes(&std::fs::read(path(&outcome, "速成")).unwrap()).unwrap();
    let a: String = pack.code_table.lookup("a").iter().map(|e| e.text).collect();
    // Big5 排序值曰 23060 < 日 23061，日有年級所以在前。
    assert_eq!(a, "日曰𠀀");
}

#[test]
fn second_build_uses_the_cache_and_gives_the_same_bytes() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let first = run(root.path(), &upstream(README), false).unwrap();
    let first_bytes = std::fs::read(path(&first, "速成")).unwrap();
    let first_lexicon = std::fs::read(path(&first, "詞庫")).unwrap();
    let first_language_model = std::fs::read(path(&first, "語言模型")).unwrap();
    let first_radical_hint = std::fs::read(path(&first, "字根提示")).unwrap();
    let second = run(root.path(), &Offline, false).unwrap();
    assert_eq!(
        std::fs::read(path(&second, "字根提示")).unwrap(),
        first_radical_hint
    );
    assert_eq!(std::fs::read(path(&second, "速成")).unwrap(), first_bytes);
    assert_eq!(std::fs::read(path(&second, "詞庫")).unwrap(), first_lexicon);
    assert_eq!(
        std::fs::read(path(&second, "語言模型")).unwrap(),
        first_language_model
    );
    assert_eq!(first.packs, second.packs);
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
    // 「有前文」的題：前文、碼、句子三欄；句子是 test 句（樣本小於上限，都在基準檔裡）。
    let context = read(root.path(), "bench/corpus/context/yue-wiki.tsv");
    let context_items = context.lines().count();
    assert!(context_items > 0);
    for line in context.lines() {
        let [before, codes, sentence] = line.split('\t').collect::<Vec<_>>()[..] else {
            panic!("要三欄：{line}");
        };
        assert!(before.chars().any(char::is_alphanumeric), "{line}");
        assert!(bench.contains(&format!("{codes}\t{sentence}\n")), "{line}");
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
            "語料 yue-wiki：train {} 句、dev {} 句、test {} 句（不重複），其中 {} 句進 bench/corpus/yue-wiki.tsv、有前文的 {context_items} 句進 bench/corpus/context/yue-wiki.tsv；含字集外字元丟掉 {DROPPED} 句",
            counts.train, counts.dev, counts.test, counts.test
        ),
        "[通過] 語料不洩漏：yue-wiki 有前文 test 句原樣出現在 train 0 句".into(),
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
        // 有前文的調參題：三欄，句子是 dev 句。
        let context = read(root.path(), &format!("build/corpus/{name}/dev-context.tsv"));
        for line in context.lines() {
            let [before, _, sentence] = line.split('\t').collect::<Vec<_>>()[..] else {
                panic!("要三欄：{line}");
            };
            assert!(!before.is_empty() && expected.contains(sentence), "{line}");
        }
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
                    // 並存寫法：「裡」換成代表寫法「裏」，「臺」照原文。
                    '裏' => "yv",
                    '臺' => "gg",
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
        shown.contains("[通過] 字形用常見寫法：對照表 2 組（揾→搵、羣→群）"),
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

/// 粵典的詞另建一份可選的詞庫資料包（M3 票 24，plan §4.2）：預設詞庫的詞全在、再補粵典的詞頭；
/// 預設詞庫沒有粵典的詞。授權照非商業條款，`LICENSE`、`NOTICE` 附版權告示、鳴謝與官網，寫明不得再授權、
/// 不得商業使用。
#[test]
fn builds_an_optional_lexicon_pack_with_the_words_hk_headwords_under_their_own_terms() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let outcome = run(root.path(), &upstream(README), true).unwrap();

    let dir = root.path().join("build/packs/quick-lexicon-words-hk");
    assert_eq!(
        path(&outcome, "詞庫（粵典）"),
        dir.join("quick-lexicon-words-hk.pack")
    );
    let read = |name: &str| {
        LexiconPack::from_bytes(&std::fs::read(path(&outcome, name)).unwrap())
            .unwrap()
            .lexicon
    };
    let (default, words_hk) = (read("詞庫"), read("詞庫（粵典）"));
    let frequency = |lexicon: &qim_core::Lexicon, code: &str, word: &str| {
        lexicon
            .lookup(code)
            .iter()
            .find(|w| w.text == word)
            .map(|w| w.frequency)
    };
    assert_eq!(
        frequency(&words_hk, "ana", "門日"),
        Some(7 * lexicon::HEADWORD_FREQUENCY)
    );
    assert_eq!(
        frequency(&words_hk, "aa", "曰曰"),
        Some(lexicon::MIN_FREQUENCY)
    );
    assert_eq!(frequency(&words_hk, "anr", "門口"), Some(9435));
    assert_eq!(frequency(&default, "ana", "門日"), None);
    for (code, words) in default.iter() {
        for word in words {
            assert_eq!(
                frequency(&words_hk, code, &word.text),
                Some(word.frequency),
                "{code} {}",
                word.text
            );
        }
    }
    assert_eq!(outcome.words_hk.unwrap().counts.headwords, 2);
    assert_eq!(outcome.words_hk.unwrap().headwords, 4);
    assert_eq!(default.char_frequency('日'), words_hk.char_frequency('日'));

    let license = std::fs::read_to_string(dir.join("LICENSE")).unwrap();
    assert!(license.contains(WORDS_HK_LICENSE), "{license}");
    assert!(license.contains(LGPL.trim_end()), "{license}");
    let notice = std::fs::read_to_string(dir.join("NOTICE")).unwrap();
    for expected in [
        &sha256_hex(WORDSLIST.as_bytes()),
        &sha256_hex(WORD_COUNT.as_bytes()),
        "https://words.hk/",
        "may not be sub-licensed",
        "may not be used for commercial purposes",
        "use quick-lexicon instead",
        "Non-Commercial Open Data License",
        // §1(b)(iii) 的告示要用實際取用的網址，兩個檔各一個。
        "( https://words.hk/faiman/analysis/wordslist/ )",
        "( https://words.hk/faiman/analysis/existingwordcount/ )",
    ] {
        assert!(notice.contains(expected), "缺 {expected}：{notice}");
    }
    // 預設詞庫的授權不混進粵典。
    let default_license =
        std::fs::read_to_string(root.path().join("build/packs/quick-lexicon/LICENSE")).unwrap();
    assert!(
        !default_license.contains(WORDS_HK_LICENSE),
        "{default_license}"
    );

    let report = outcome.report.unwrap();
    assert!(report.passed(), "{report}");
    let check = report
        .checks
        .iter()
        .find(|c| c.id == CheckId::WordsHkLexicon)
        .expect("verify 比對兩份詞庫");
    assert!(check.passed, "{report}");
}

/// 粵典詞庫少了預設詞庫的詞條、詞頻變了或沒多任何詞時，`--verify` 的比對失敗。
#[test]
fn a_words_hk_lexicon_missing_a_default_entry_fails_the_check() {
    let word = |text: &str, frequency| qim_core::Word {
        text: text.into(),
        frequency,
    };
    let lexicon = |entries: Vec<(&str, Vec<qim_core::Word>)>| {
        qim_core::Lexicon::new(
            entries
                .into_iter()
                .map(|(code, words)| (code.to_string(), words))
                .collect::<BTreeMap<_, _>>(),
            BTreeMap::new(),
        )
    };
    let default = lexicon(vec![("anr", vec![word("門口", 9435)])]);
    let more = lexicon(vec![
        ("anr", vec![word("門口", 9435)]),
        ("ana", vec![word("門日", 700)]),
    ]);
    assert!(verify::words_hk_lexicon(&default, &more).passed);
    let lower = lexicon(vec![
        ("anr", vec![word("門口", 100)]),
        ("ana", vec![word("門日", 700)]),
    ]);
    assert!(!verify::words_hk_lexicon(&default, &lower).passed);
    assert!(!verify::words_hk_lexicon(&more, &default).passed);
    // 一個詞都沒多（粵典沒接上）也算失敗。
    assert!(!verify::words_hk_lexicon(&default, &default).passed);
}

/// 粵典的來源和釘的 sha256 對不上（上游更新了）時，只略過粵典詞庫：其他資料包照建、`--verify` 照過、
/// 報告列出略過的那份；上次建出的粵典詞庫刪掉，免得殼讀到和預設詞庫對不上的舊版。
#[test]
fn a_words_hk_source_that_changed_upstream_only_skips_the_words_hk_pack() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    let sources = root.path().join("data/sources.toml");
    let pinned = std::fs::read_to_string(&sources).unwrap();
    let changed = pinned.replace(&sha256_hex(WORDSLIST.as_bytes()), &"0".repeat(64));
    assert_ne!(pinned, changed);
    std::fs::write(&sources, changed).unwrap();
    let stale = root.path().join("build/packs/quick-lexicon-words-hk");
    std::fs::create_dir_all(&stale).unwrap();
    std::fs::write(stale.join("quick-lexicon-words-hk.pack"), b"old").unwrap();

    let outcome = run(root.path(), &upstream(README), true).unwrap();
    assert_eq!(outcome.packs.len(), 9);
    assert!(
        outcome
            .packs
            .iter()
            .all(|p| p.id != "quick-lexicon-words-hk")
    );
    assert!(path(&outcome, "詞庫").exists());
    assert!(!stale.exists(), "上次建出的粵典詞庫要刪掉");
    assert_eq!(outcome.words_hk, None);
    let [skipped] = outcome.skipped.as_slice() else {
        panic!("應該只略過粵典詞庫：{:?}", outcome.skipped);
    };
    assert_eq!(skipped.id, "quick-lexicon-words-hk");
    assert!(skipped.reason.contains("sha256"), "{}", skipped.reason);
    let report = outcome.report.unwrap();
    assert!(report.passed(), "{report}");
    let shown = report.to_string();
    assert!(shown.contains("略過 quick-lexicon-words-hk"), "{shown}");
    assert!(shown.contains("[source.words_hk]"), "{shown}");
}

/// 下載不到粵典（離線、網址掛了）時同樣只略過粵典詞庫；其他來源已在快取。
#[test]
fn a_words_hk_source_that_cannot_be_downloaded_only_skips_the_words_hk_pack() {
    let root = tempfile::tempdir().unwrap();
    repo(root.path(), STATEMENT);
    run(root.path(), &upstream(README), false).unwrap();
    std::fs::remove_dir_all(root.path().join("cache/sources/words_hk")).unwrap();

    let outcome = run(root.path(), &Offline, false).unwrap();
    assert_eq!(outcome.packs.len(), 9);
    assert_eq!(outcome.skipped.len(), 1);
    assert!(
        outcome.skipped[0].reason.contains("離線"),
        "{}",
        outcome.skipped[0].reason
    );
    // 別的來源出錯照樣失敗：只有可選的粵典能略過。
    std::fs::remove_dir_all(root.path().join("cache/sources/rime_essay")).unwrap();
    assert!(run(root.path(), &Offline, false).is_err());
}
