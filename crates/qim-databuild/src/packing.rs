//! 資料包的 `LICENSE` 與 `NOTICE`（plan §4）：每個資料包只寫它用到的來源。

use std::path::Path;

use crate::Error;
use crate::corpora::CORPORA;
use crate::emoji::{CLDR, EMOJI_TEST};
use crate::lexicon;
use crate::pipeline::{LIBCANGJIE, OPENCC, RIME_ESSAY, SCOWL, UNIHAN};
use crate::sources::{LicenseText, SOURCES, Source, Sources, sha256_hex};
use crate::words_hk::WORDS_HK;

/// 進資料包的一個來源：經 `sources.toml` 下載的，或 QIM 自己整理、存在 repo 裡的。
pub(crate) enum PackSource {
    Upstream(UpstreamSource),
    Repo(RepoData),
}

/// 經 `sources.toml` 下載、進資料包的一個來源。
pub(crate) struct UpstreamSource {
    /// 寫進 `LICENSE`、`NOTICE` 的名字。
    pub name: &'static str,
    /// `sources.toml` 的 `[source.<id>]`。
    pub source_id: &'static str,
    /// 取用的內容，寫進 `NOTICE`。
    pub used: &'static str,
}

/// 速成資料包：libcangjie 給碼，Unihan 的 `kGradeLevel` 給字序，OpenCC 判定簡體字（plan §4.1）。
pub(crate) const QUICK: [PackSource; 3] = [
    PackSource::Upstream(UpstreamSource {
        name: "libcangjie",
        source_id: LIBCANGJIE,
        used: "data/table.txt: Cangjie 3 and Cangjie 5 codes, Big5 and HKSCS flags, Big5 order",
    }),
    PackSource::Upstream(UpstreamSource {
        name: "Unihan",
        source_id: UNIHAN,
        used: "fields kGradeLevel and kHKGlyph of Unihan_DictionaryLikeData.txt",
    }),
    PackSource::Upstream(UpstreamSource {
        name: "OpenCC",
        source_id: OPENCC,
        used: "data/dictionary/STCharacters.txt, TSCharacters.txt and HKVariants.txt: which \
               characters are marked as simplified",
    }),
];

/// 簡繁資料包：OpenCC 的字典（plan §4.4）。
pub(crate) const SIMPLIFIER: [PackSource; 1] = [PackSource::Upstream(UpstreamSource {
    name: "OpenCC",
    source_id: OPENCC,
    used: "data/dictionary/CJK_Compatibility_Ideographs.txt, TSPhrases.txt, TSCharacters.txt, \
           HKVariants.txt and HKVariantsRevPhrases.txt: the dictionaries of the hk2s configuration",
})];

/// 英文資料包：ESDB（SCOWL）的 Hunspell en_US 字典（plan §4.4）。
pub(crate) const ENGLISH: [PackSource; 1] = [PackSource::Upstream(UpstreamSource {
    name: "ESDB (SCOWL)",
    source_id: SCOWL,
    used: "en_US.dic and en_US.aff in the hunspell-en_US zip: the words of the size 60 \
           American English speller dictionary",
})];

/// emoji 資料包：CLDR 的標註給字詞與 emoji，emoji-test.txt 給 emoji 的完整寫法（plan §4.4）。
pub(crate) const EMOJI: [PackSource; 2] = [
    PackSource::Upstream(UpstreamSource {
        name: "Unicode CLDR",
        source_id: CLDR,
        used: "common/annotations/yue.xml and zh_Hant.xml: emoji keywords and short names",
    }),
    PackSource::Upstream(UpstreamSource {
        name: "Unicode emoji-test.txt",
        source_id: EMOJI_TEST,
        used: "emoji-test.txt: the code points of fully-qualified emoji",
    }),
];

/// 字根提示資料包：libcangjie 給倉頡三代 / 五代碼（plan §4.1 步驟 5）。
pub(crate) const RADICAL_HINT: [PackSource; 1] = [PackSource::Upstream(UpstreamSource {
    name: "libcangjie",
    source_id: LIBCANGJIE,
    used: "data/table.txt: Cangjie 3 and Cangjie 5 codes of Han characters",
})];

/// 詞庫資料包的 rime-essay；字頻的語料與口語詞表另由 [`lexicon_sources`] 加上。
const ESSAY: UpstreamSource = UpstreamSource {
    name: "rime-essay",
    source_id: RIME_ESSAY,
    used: "essay.txt: words and their frequencies",
};

/// 詞庫資料包的來源：rime-essay 給詞與詞頻，每份語料的 train 切分給字頻（已分詞的 HKCanCor
/// 另給詞），最後是粵語口語詞表（plan §4.2）。
pub(crate) fn lexicon_sources() -> Vec<PackSource> {
    std::iter::once(ESSAY)
        .chain(CORPORA.iter().map(|corpus| UpstreamSource {
            name: corpus.title,
            source_id: corpus.source_id,
            used: if corpus.segmented() {
                "character counts of the training split of the extracted text, and the words of \
                 two or more characters in that split with their counts"
            } else {
                "character counts of the training split of the extracted text"
            },
        }))
        .map(PackSource::Upstream)
        .chain([PackSource::Repo(COLLOQUIAL)])
        .collect()
}

/// 可選的粵典詞庫資料包：預設詞庫的來源，再加粵典的詞頭表與詞頻（plan §4.2，M3 票 24）。
pub(crate) fn words_hk_lexicon_sources() -> Vec<PackSource> {
    let mut sources = lexicon_sources();
    sources.push(PackSource::Upstream(UpstreamSource {
        name: "words.hk (粵典)",
        source_id: WORDS_HK,
        used: "wordslist.csv: the headwords of two or more characters; existingwordcount.csv: \
               how often each headword occurs in the words.hk Cantonese corpus",
    }));
    sources
}

/// QIM 自己整理、存在 repo 裡的資料：不經 `sources.toml` 下載，授權全文也在 repo。
pub(crate) struct RepoData {
    /// 寫進 `LICENSE`、`NOTICE` 的名字。
    pub name: &'static str,
    /// 相對 repo 根目錄。
    pub path: &'static str,
    /// SPDX 識別碼。
    pub license: &'static str,
    /// repo 裡的授權全文，相對 repo 根目錄。
    pub license_path: &'static str,
    /// 取用的內容，寫進 `NOTICE`。
    pub used: &'static str,
}

/// 詞庫資料包的粵語口語詞表（plan §4.2、brief §5.2）。
const COLLOQUIAL: RepoData = RepoData {
    name: "QIM Cantonese colloquial word list",
    path: lexicon::COLLOQUIAL_PATH,
    license: "CC0-1.0",
    license_path: "data/licenses/CC0-1.0.txt",
    used: "Cantonese colloquial words and their frequencies, compiled by QIM",
};

/// 語言模型資料包的來源：每份語料的 train 切分（plan §4.3）。
pub(crate) fn language_model_sources() -> Vec<PackSource> {
    CORPORA
        .iter()
        .map(|corpus| {
            PackSource::Upstream(UpstreamSource {
                name: corpus.title,
                source_id: corpus.source_id,
                used: "counts of characters and adjacent character pairs in the training split of the extracted text",
            })
        })
        .collect()
}

/// 停鍵層模型資料包的來源（plan §2.7）：每份語料 train 切分的文字，訓練神經模型用。
pub(crate) fn neural_sources() -> Vec<PackSource> {
    CORPORA
        .iter()
        .map(|corpus| {
            PackSource::Upstream(UpstreamSource {
                name: corpus.title,
                source_id: corpus.source_id,
                used: "the training split of the extracted text, used to train the neural model \
                       and to count the characters of its vocabulary",
            })
        })
        .collect()
}

/// 詞級 trigram 資料包的來源（plan §4.3）：每份語料的 train 切分給段的次數；切段用預設詞庫的詞與
/// 詞頻（rime-essay、HKCanCor 補的詞、粵語口語詞表）與字級 bigram，所以詞庫的來源也列進來。
pub(crate) fn word_trigram_sources() -> Vec<PackSource> {
    let words = UpstreamSource {
        name: "rime-essay",
        source_id: RIME_ESSAY,
        used: "essay.txt: words and their frequencies, used to split the training text into words",
    };
    std::iter::once(words)
        .chain(CORPORA.iter().map(|corpus| UpstreamSource {
            name: corpus.title,
            source_id: corpus.source_id,
            used: if corpus.segmented() {
                "counts of word bigrams and trigrams in the training split of the extracted text, \
                 and the words of two or more characters in that split, used to split the text"
            } else {
                "counts of word bigrams and trigrams in the training split of the extracted text"
            },
        }))
        .map(PackSource::Upstream)
        .chain([PackSource::Repo(RepoData {
            used: "Cantonese colloquial words and their frequencies, used to split the training \
                   text into words",
            ..COLLOQUIAL
        })])
        .collect()
}

/// 一個進資料包的來源，授權已讀出並核對過。
pub(crate) struct Packed {
    name: &'static str,
    repository: String,
    /// repo 裡的資料沒有另外的版本，跟著建置用的 QIM commit。
    version: Option<String>,
    /// SPDX 識別碼。
    license_id: String,
    /// 每個檔的路徑與 sha256。
    files: Vec<(String, String)>,
    used: &'static str,
    license: License,
}

/// 要寫進資料包 `LICENSE` 的一段授權。
struct License {
    /// 這段授權從哪裡來，寫在原文前一行。
    origin: String,
    text: String,
}

/// 讀出每個來源的授權。`upstream(id, 檔)` 讀快取裡的來源檔，`repo(路徑)` 讀 repo 裡的檔。
pub(crate) fn resolve(
    specs: &[PackSource],
    sources: &Sources,
    upstream: impl Fn(&str, &str) -> Result<String, Error>,
    repo: impl Fn(&str) -> Result<String, Error>,
) -> Result<Vec<Packed>, Error> {
    specs
        .iter()
        .map(|spec| match spec {
            PackSource::Upstream(spec) => resolve_upstream(spec, sources, &upstream, &repo),
            PackSource::Repo(spec) => repo_data(spec, &repo),
        })
        .collect()
}

fn resolve_upstream(
    spec: &UpstreamSource,
    sources: &Sources,
    upstream: impl Fn(&str, &str) -> Result<String, Error>,
    repo: impl Fn(&str) -> Result<String, Error>,
) -> Result<Packed, Error> {
    let source = sources
        .source
        .get(spec.source_id)
        .ok_or_else(|| Error::invalid(SOURCES, format!("沒有 [source.{}]", spec.source_id)))?;
    let license = license_of(source, |file| upstream(spec.source_id, file), &repo)?;
    Ok(Packed {
        name: spec.name,
        repository: source.repository.clone(),
        version: Some(source.version.clone()),
        license_id: source.license.clone(),
        files: source
            .files
            .iter()
            .map(|f| (f.path.clone(), f.sha256.clone()))
            .collect(),
        used: spec.used,
        license,
    })
}

/// repo 裡的資料：`repo(路徑)` 讀 repo 裡的檔，`NOTICE` 記它這次內容的 sha256。
fn repo_data(
    spec: &RepoData,
    repo: impl Fn(&str) -> Result<String, Error>,
) -> Result<Packed, Error> {
    Ok(Packed {
        name: spec.name,
        repository: "the QIM repository".into(),
        version: None,
        license_id: spec.license.into(),
        files: vec![(spec.path.into(), sha256_hex(repo(spec.path)?.as_bytes()))],
        used: spec.used,
        license: License {
            origin: repo_file_origin(spec.license_path),
            text: repo(spec.license_path)?,
        },
    })
}

fn repo_file_origin(path: &str) -> String {
    format!("Full text, kept in the QIM repository as {path}")
}

fn license_of(
    source: &Source,
    upstream: impl Fn(&str) -> Result<String, Error>,
    repo: impl Fn(&str) -> Result<String, Error>,
) -> Result<License, Error> {
    let in_files = |file: &str| source.file(file).map(drop);
    match &source.license_text {
        Some(LicenseText::Statement { file, quote }) => {
            in_files(file)?;
            check_statement(source, file, quote, &upstream(file)?)?;
            Ok(License {
                origin: format!("Statement in {file}"),
                text: quote.clone(),
            })
        }
        Some(LicenseText::UpstreamFile(file)) => {
            in_files(file)?;
            Ok(License {
                origin: format!("Full text of {file} in the source"),
                text: upstream(file)?,
            })
        }
        Some(LicenseText::RepoFile(path)) => Ok(License {
            origin: repo_file_origin(path),
            text: repo(path)?,
        }),
        None => Err(Error::invalid(
            SOURCES,
            format!(
                "{} 的資料進資料包，要寫 license_text（statement、upstream_file 或 repo_file）",
                source.repository
            ),
        )),
    }
}

/// sources.toml 照抄的聲明原文要真的出現在上游檔裡（換行與空白不計）。
fn check_statement(source: &Source, file: &str, quote: &str, upstream: &str) -> Result<(), Error> {
    let squash = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    if squash(upstream).contains(&squash(quote)) {
        Ok(())
    } else {
        Err(Error::invalid(
            SOURCES,
            format!(
                "license_text.statement 的 quote 不在 {} 的 {file} 裡",
                source.repository
            ),
        ))
    }
}

/// manifest 的 `license`（plan §4.6）：各來源的 SPDX 識別碼依來源順序以 ` AND ` 接起，重複的只寫一次。
pub(crate) fn license_expression(packed: &[Packed]) -> String {
    let mut ids: Vec<&str> = Vec::new();
    for p in packed {
        if !ids.contains(&p.license_id.as_str()) {
            ids.push(&p.license_id);
        }
    }
    ids.join(" AND ")
}

/// 在 `dir` 寫 `<id>.pack`、`LICENSE`、`NOTICE`；`about` 是 NOTICE 最後說明怎麼用來源的一段。
pub(crate) fn write(
    dir: &Path,
    id: &str,
    bytes: &[u8],
    packed: &[Packed],
    about: &str,
) -> Result<(), Error> {
    std::fs::create_dir_all(dir).map_err(Error::io(dir))?;
    let write = |name: String, contents: &[u8]| {
        let path = dir.join(name);
        std::fs::write(&path, contents).map_err(Error::io(path))
    };
    write(format!("{id}.pack"), bytes)?;
    write("LICENSE".into(), license(packed).as_bytes())?;
    write("NOTICE".into(), notice(id, packed, about).as_bytes())
}

/// 每個來源一段：來源、授權、照抄的聲明或全文。
fn license(packed: &[Packed]) -> String {
    packed
        .iter()
        .map(|p| {
            let version = p
                .version
                .as_ref()
                .map(|v| format!(" ({v})"))
                .unwrap_or_default();
            format!(
                "Source: {name}, {repo}{version}\n\
                 License: {license}\n\
                 \n\
                 {origin}:\n\
                 \n\
                 {text}\n",
                name = p.name,
                repo = p.repository,
                license = p.license_id,
                origin = p.license.origin,
                text = p.license.text.trim_end(),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn notice(id: &str, packed: &[Packed], about: &str) -> String {
    let sources: String = packed
        .iter()
        .map(|p| {
            let files: String = p
                .files
                .iter()
                .map(|(path, sha256)| format!("File: {path} (sha256 {sha256})\n"))
                .collect();
            let version = p
                .version
                .as_ref()
                .map(|v| format!("Version: {v}\n"))
                .unwrap_or_default();
            format!(
                "Source: {name}\n\
                 Repository: {repo}\n\
                 {version}\
                 {files}\
                 Used: {used}\n\
                 License: {license} (see LICENSE)\n\
                 \n",
                name = p.name,
                repo = p.repository,
                used = p.used,
                license = p.license_id,
            )
        })
        .collect();
    format!("{id}.pack: QIM data pack \"{id}\".\n\n{sources}{about}")
}
