//! 語料清單（plan §4.3）：每份語料用哪個來源的哪些檔、怎麼讀出文字、基準放哪個目錄、怎麼署名。
//! 來源的版本、網址與 sha256 在 `data/sources.toml`。

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use ferrous_opencc::OpenCC;
use ferrous_opencc::config::BuiltinConfig;

use crate::corpus::BENCH_SENTENCES;
use crate::sources::Source;
use crate::{Error, hkcancor, wiki};

/// 基準檔放哪個目錄（plan §5.2）：維基防退步，口語是品質判斷的主要依據（brief §9）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Wiki,
    Colloquial,
}

impl Group {
    pub const ALL: [Group; 2] = [Group::Wiki, Group::Colloquial];

    /// 相對 repo 根目錄，進 git，旁邊放 `ATTRIBUTION.md`。
    pub fn dir(self) -> &'static str {
        match self {
            Group::Wiki => "bench/corpus",
            Group::Colloquial => "bench/colloquial",
        }
    }

    fn license_paragraph(self) -> &'static str {
        match self {
            Group::Wiki => {
                "The sentences in this directory are derived from Wikipedia articles and are licensed\n\
                 under the Creative Commons Attribution-ShareAlike 4.0 International License\n\
                 (CC BY-SA 4.0), https://creativecommons.org/licenses/by-sa/4.0/."
            }
            Group::Colloquial => {
                "The sentences in this directory come from several sources; each file keeps the\n\
                 license of its source, given in its section below."
            }
        }
    }
}

/// 來源檔的格式。
#[derive(Debug, Clone, Copy)]
enum Format {
    /// Wikimedia 的 `pages-articles*.xml.bz2`，見 `wiki`。
    WikiDump,
    /// fcbond/hkcancor 的 `hkcancor-utf8.zip`，見 `hkcancor`。
    HkCanCor,
    /// 一行一句的純文字。
    Lines,
}

pub struct Corpus {
    /// 基準檔與訓練切分的名字，也是基準的集合名。
    pub name: &'static str,
    /// `sources.toml` 的 `[source.<id>]`。
    pub source_id: &'static str,
    pub group: Group,
    /// bigram 與詞庫字頻的次數乘上的權重（plan §4.3）。口語語料只有維基的約 1/127 句，
    /// M1 票 12 在 dev 切分上掃 1–256 定為 64：口語集修正次數少約三成，維基集答錯多約 2%；
    /// 再往上口語只再好一點、維基退得更多（票 12 Comments）。
    pub weight: u64,
    format: Format,
    /// 署名：來源名稱、網址、作者與引用、授權網址、QIM 怎麼抽出文字。
    pub(crate) title: &'static str,
    site: &'static str,
    credit: &'static str,
    license_url: &'static str,
    extraction: &'static str,
}

const WIKI_CREDIT: &str = "by its contributors (each article's page history lists its authors)";
const WIKI_EXTRACTION: &str = "article text extracted from the wikitext (templates, tables, references, file links, headings and lists removed)";
const CC_BY_SA_4: &str = "https://creativecommons.org/licenses/by-sa/4.0/";

pub const CORPORA: [Corpus; 4] = [
    Corpus {
        name: "yue-wiki",
        source_id: "yue_wiki",
        group: Group::Wiki,
        weight: 1,
        format: Format::WikiDump,
        title: "Cantonese Wikipedia",
        site: "https://zh-yue.wikipedia.org/",
        credit: WIKI_CREDIT,
        license_url: CC_BY_SA_4,
        extraction: WIKI_EXTRACTION,
    },
    Corpus {
        name: "zh-wiki",
        source_id: "zh_wiki",
        group: Group::Wiki,
        weight: 1,
        format: Format::WikiDump,
        title: "Chinese Wikipedia",
        site: "https://zh.wikipedia.org/",
        credit: WIKI_CREDIT,
        license_url: CC_BY_SA_4,
        extraction: WIKI_EXTRACTION,
    },
    Corpus {
        name: "hkcancor",
        source_id: "hkcancor",
        group: Group::Colloquial,
        weight: 64,
        format: Format::HkCanCor,
        title: "Hong Kong Cantonese Corpus (HKCanCor), transcriptions",
        site: "https://github.com/fcbond/hkcancor",
        credit: "created by Luke Kang Kwong; please cite K. K. Luke and May L.Y. Wong (2015) The Hong Kong Cantonese Corpus: Design and Uses, Journal of Chinese Linguistics Monograph Series 25, 312-333",
        license_url: "https://creativecommons.org/licenses/by/4.0/",
        extraction: "the words of each utterance joined in order, part-of-speech tags and Jyutping removed",
    },
    Corpus {
        name: "common-voice-yue",
        source_id: "common_voice",
        group: Group::Colloquial,
        weight: 64,
        format: Format::Lines,
        title: "Mozilla Common Voice sentence collection, Cantonese (yue and zh-HK)",
        site: "https://github.com/common-voice/common-voice",
        credit: "sentences contributed by Common Voice users",
        license_url: "https://creativecommons.org/publicdomain/zero/1.0/",
        extraction: "one sentence per line, used as is",
    },
];

impl Corpus {
    /// 逐段交出轉成香港繁體（OpenCC `s2hk`）的文字。`paths` 是來源在 `sources.toml` 列的每個檔
    /// 在快取裡的位置，依列出的順序讀；換 dump 日期只改 `sources.toml`。
    pub fn texts(
        &self,
        paths: &[PathBuf],
        mut text: impl FnMut(&str) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let s2hk = OpenCC::from_config(BuiltinConfig::S2hk)
            .map_err(|e| Error::invalid("OpenCC s2hk", e.to_string()))?;
        let mut converted = |raw: &str| text(&s2hk.convert(raw));
        for path in paths {
            let open = || File::open(path).map_err(Error::io(path));
            match self.format {
                Format::WikiDump => wiki::articles(BufReader::new(open()?), &mut converted)?,
                Format::HkCanCor => {
                    let zip = std::fs::read(path).map_err(Error::io(path))?;
                    hkcancor::archive(&zip, &mut converted)?;
                }
                Format::Lines => {
                    for line in BufReader::new(open()?).lines() {
                        converted(&line.map_err(Error::io(path))?)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// `ATTRIBUTION.md` 裡這份語料的段落。
    pub fn attribution(&self, source: &Source) -> String {
        let files: String = source
            .files
            .iter()
            .map(|file| {
                format!(
                    "- File: {} (sha256 {}), {}\n",
                    file.path, file.sha256, file.url
                )
            })
            .collect();
        format!(
            "## {name}.tsv\n\
             \n\
             - Source: {title}, {site}, {credit}.\n\
             - Version: {version}\n\
             {files}\
             - License: {license}, {license_url}\n\
             - Extraction: {extraction}.\n",
            name = self.name,
            title = self.title,
            site = self.site,
            credit = self.credit,
            version = source.version,
            license = source.license,
            license_url = self.license_url,
            extraction = self.extraction,
        )
    }
}

/// 一個基準目錄的 `ATTRIBUTION.md`：授權說明、QIM 的改動、各語料的段落。
pub fn attribution_file(group: Group, sections: &[String]) -> String {
    format!(
        "# Attribution\n\
         \n\
         {license} They are test data for the QIM\n\
         top-choice benchmark and are not shipped in any app.\n\
         \n\
         Changes made by QIM: text was taken from each source as its section describes and\n\
         converted with OpenCC s2hk; it was split into clauses at punctuation and whitespace,\n\
         and clauses containing characters outside the QIM continuous character set were\n\
         dropped. Each file keeps up to {BENCH_SENTENCES} distinct clauses from the held-out test split,\n\
         one per line, preceded by machine-generated Quick codes (one per character, separated\n\
         by spaces) and a tab.\n\
         \n\
         {}",
        sections.join("\n"),
        license = group.license_paragraph(),
    )
}

/// 語料在 `build/corpus/` 下的目錄。
pub fn build_dir(root: &Path, corpus: &Corpus) -> PathBuf {
    root.join("build/corpus").join(corpus.name)
}
