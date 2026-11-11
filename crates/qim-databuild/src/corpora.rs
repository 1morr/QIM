//! 語料清單（plan §4.3）：每份語料用哪個來源的哪些檔、怎麼讀出文字、基準放哪個目錄、怎麼署名。
//! 來源的版本、網址與 sha256 在 `data/sources.toml`。

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use ferrous_opencc::OpenCC;
use ferrous_opencc::config::BuiltinConfig;

use crate::common_forms::CommonForms;
use crate::corpus::{BENCH_SENTENCES, CONTEXT_CLAUSES, CONTEXT_SENTENCES};
use crate::sources::Source;
use crate::transcode::Transcoder;
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

    /// 「有前文」的基準題（plan §5.2）：`<目錄>/context/<名>.tsv`，授權同 [`Group::dir`]。
    pub fn context_dir(self) -> &'static str {
        match self {
            Group::Wiki => "bench/corpus/context",
            Group::Colloquial => "bench/colloquial/context",
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

/// 前處理的字形轉換（plan §4.3）：都轉成香港繁體，轉完再照 [`CommonForms`] 換回常見寫法。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Conversion {
    /// 原文混有簡體（中文維基）：OpenCC `s2hk`。
    S2hk,
    /// 原文已是繁體（粵語維基、HKCanCor、Common Voice）：只換香港寫法的 OpenCC `t2hk`。
    /// `s2hk` 會把繁體裡也是簡體的字再轉一次，「晒」→「曬」、「吓」→「嚇」、「布」→「佈」，
    /// 粵語維基 3,400 萬字裡換錯約 6.8 萬字（M1 票 17b Comments）。
    T2hk,
}

impl Conversion {
    /// `ATTRIBUTION.md` 裡的說明。
    fn description(self) -> &'static str {
        match self {
            Conversion::S2hk => {
                "OpenCC s2hk (Simplified to Hong Kong Traditional Chinese), then Hong Kong variant forms written in their common forms, and characters with two kept forms (such as 裏/裡) written in one representative form; 着, 臺 and 菸 are kept as the source wrote them"
            }
            Conversion::T2hk => {
                "OpenCC t2hk (Traditional to Hong Kong variant forms; the source is already Traditional Chinese), then Hong Kong variant forms written in their common forms, and characters with two kept forms (such as 裏/裡) written in one representative form; 着, 臺 and 菸 are kept as the source wrote them"
            }
        }
    }
}

/// `s2hk` 會換掉、粵語語料改用 `t2hk` 後保留的粵語用字（M1 票 17b 的統計）；
/// `--verify` 檢查它們在 train 的字頻都大於 0。
pub const CANTONESE_KEPT: [char; 9] = ['晒', '搵', '吓', '郁', '喂', '涌', '咸', '响', '仆'];

/// 一種 [`Conversion`] 加上字形對照表的轉換器，建一次給整份語料用。
pub struct Converter {
    opencc: OpenCC,
    forms: CommonForms,
}

impl Converter {
    pub fn new(conversion: Conversion, forms: CommonForms) -> Result<Self, Error> {
        let (config, name) = match conversion {
            Conversion::S2hk => (BuiltinConfig::S2hk, "OpenCC s2hk"),
            Conversion::T2hk => (BuiltinConfig::T2hk, "OpenCC t2hk"),
        };
        let opencc =
            OpenCC::from_config(config).map_err(|e| Error::invalid(name, e.to_string()))?;
        Ok(Self { opencc, forms })
    }

    /// 轉換、單向的寫法照原文放回（[`BothForms::as_written`](crate::common_forms::BothForms::as_written)），
    /// 再換成常見寫法。
    pub fn convert(&self, raw: &str) -> String {
        let converted = self.opencc.convert(raw);
        self.forms
            .apply(&self.forms.both().as_written(raw, &converted))
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
    pub conversion: Conversion,
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
        conversion: Conversion::T2hk,
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
        conversion: Conversion::S2hk,
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
        conversion: Conversion::T2hk,
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
        conversion: Conversion::T2hk,
        format: Format::Lines,
        title: "Mozilla Common Voice sentence collection, Cantonese (yue and zh-HK)",
        site: "https://github.com/common-voice/common-voice",
        credit: "sentences contributed by Common Voice users",
        license_url: "https://creativecommons.org/publicdomain/zero/1.0/",
        extraction: "one sentence per line, used as is",
    },
];

impl Corpus {
    /// 逐段交出經 [`Self::conversion`] 轉成香港繁體、再照 `forms` 換回常見寫法的文字。`paths` 是來源在 `sources.toml` 列的
    /// 每個檔在快取裡的位置，依列出的順序讀；換 dump 日期只改 `sources.toml`。
    pub fn texts(
        &self,
        paths: &[PathBuf],
        forms: &CommonForms,
        mut text: impl FnMut(&str) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let converter = Converter::new(self.conversion, forms.clone())?;
        self.raw_texts(paths, |raw| text(&converter.convert(raw)))
    }

    /// 來源已分詞（HKCanCor），詞庫另取它的詞。
    pub fn segmented(&self) -> bool {
        matches!(self.format, Format::HkCanCor)
    }

    /// 已分詞的語料（HKCanCor）train 切分裡兩字以上的詞與次數（[`hkcancor::TrainWords`]）；
    /// 沒分詞的語料是空的。詞庫用它補詞（plan §4.2）。
    pub fn train_words(
        &self,
        paths: &[PathBuf],
        forms: &CommonForms,
        transcoder: &Transcoder,
    ) -> Result<hkcancor::WordCounts, Error> {
        if !self.segmented() {
            return Ok(hkcancor::WordCounts::default());
        }
        let converter = Converter::new(self.conversion, forms.clone())?;
        let convert = |raw: &str| converter.convert(raw);
        let mut words = hkcancor::TrainWords::new(&convert, transcoder);
        for path in paths {
            let zip = std::fs::read(path).map_err(Error::io(path))?;
            hkcancor::archive_tagged(&zip, |utterance| words.add(utterance))?;
        }
        Ok(words.counts)
    }

    /// 逐段交出來源抽出、還沒轉換的文字。
    pub fn raw_texts(
        &self,
        paths: &[PathBuf],
        mut text: impl FnMut(&str) -> Result<(), Error>,
    ) -> Result<(), Error> {
        for path in paths {
            let open = || File::open(path).map_err(Error::io(path));
            match self.format {
                Format::WikiDump => wiki::articles(BufReader::new(open()?), &mut text)?,
                Format::HkCanCor => {
                    let zip = std::fs::read(path).map_err(Error::io(path))?;
                    hkcancor::archive(&zip, &mut text)?;
                }
                Format::Lines => {
                    for line in BufReader::new(open()?).lines() {
                        text(&line.map_err(Error::io(path))?)?;
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
             - Extraction: {extraction}.\n\
             - Conversion: {conversion}.\n",
            name = self.name,
            title = self.title,
            site = self.site,
            credit = self.credit,
            version = source.version,
            license = source.license,
            license_url = self.license_url,
            extraction = self.extraction,
            conversion = self.conversion.description(),
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
         Changes made by QIM: text was taken from each source and converted with OpenCC as its\n\
         section describes; it was split into clauses at punctuation and whitespace,\n\
         and clauses containing characters outside the QIM continuous character set were\n\
         dropped. Each file keeps up to {BENCH_SENTENCES} distinct clauses from the held-out test split,\n\
         one per line, preceded by machine-generated Quick codes (one per character, separated\n\
         by spaces) and a tab. The files of the same name under context/ keep up to\n\
         {CONTEXT_SENTENCES} held-out test clauses that follow other text on the same line of the source,\n\
         one per line: the text before the clause as written (up to {CONTEXT_CLAUSES} clauses and the\n\
         punctuation between them), a tab, the Quick codes, a tab and the clause.\n\
         \n\
         {}",
        sections.join("\n"),
        license = group.license_paragraph(),
    )
}

/// 語料 `name` 在 `build/corpus/` 下的目錄。
pub fn build_dir(root: &Path, name: &str) -> PathBuf {
    root.join("build/corpus").join(name)
}

/// 語料 `name` 的 dev 調參題（`build/corpus/<名>/dev.tsv`）；`qim_bench::dev_items` 讀同一個路徑
/// （plan §1.3 不讓 `qim-bench` 依賴本 crate），`xtask` 的 `shared_paths` 測試比對兩邊。
pub fn dev_items_path(root: &Path, name: &str) -> PathBuf {
    build_dir(root, name).join("dev.tsv")
}

/// 語料 `name` 的 dev「有前文」調參題（`build/corpus/<名>/dev-context.tsv`）；`qim_bench::dev_context_items`
/// 讀同一個路徑，`xtask` 的 `shared_paths` 測試比對兩邊。
pub fn dev_context_items_path(root: &Path, name: &str) -> PathBuf {
    build_dir(root, name).join("dev-context.tsv")
}

/// 語料 `name` 的長前文題（test 切分，`build/corpus/<名>/test-context-long.tsv`）：「有前文」的題換成
/// 長前文，模型階梯讀（M3 票 07）；`qim_bench::context_long_items` 讀同一個路徑，`shared_paths` 比對。
pub fn context_long_items_path(root: &Path, name: &str) -> PathBuf {
    build_dir(root, name).join("test-context-long.tsv")
}

/// 語料 `name` 的 dev 長前文題（`build/corpus/<名>/dev-context-long.tsv`）；`qim_bench::dev_context_long_items`
/// 讀同一個路徑，`shared_paths` 比對。
pub fn dev_context_long_items_path(root: &Path, name: &str) -> PathBuf {
    build_dir(root, name).join("dev-context-long.tsv")
}
