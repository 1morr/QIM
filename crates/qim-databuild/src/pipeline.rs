//! `cargo xtask data` 的整條流程：讀來源、建資料包、寫產物，選擇性驗證。

use std::fs::File;
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::corpus::{BENCH_SENTENCES, Builder, Finished};
use crate::sources::{self, Fetch, SOURCES, Source, hex, sha256_hex};
use crate::transcode::Transcoder;
use crate::verify::{self, CorpusDigests, CorpusSummary, Report, UnihanSummary};
use crate::{Error, build_pack, libcangjie, scheme, unihan, wiki};

const QUICK_SCHEME: &str = "data/schemes/quick.toml";
const CACHE: &str = ".cache/sources";
const PACKS: &str = "build/packs";
const LIBCANGJIE: &str = "libcangjie";
const TABLE: &str = "data/table.txt";
const UNIHAN: &str = "unihan";
const UNIHAN_ZIP: &str = "Unihan.zip";
/// Unihan 交叉比對的差異報表，相對 repo 根目錄；格式見 `unihan::CrossCheck::to_tsv`。
const UNIHAN_REPORT: &str = "build/reports/unihan-kcangjie.tsv";

/// 進 git 的基準語料（plan §5.2），旁邊放 `ATTRIBUTION.md`。
const BENCH_CORPUS: &str = "bench/corpus";
/// 訓練切分：建置產物，不進 git。
const BUILD_CORPUS: &str = "build/corpus";

/// 一份維基語料：基準檔與切分的名字、`sources.toml` 的 id、署名用的名稱與網址。
struct Wiki {
    name: &'static str,
    source_id: &'static str,
    title: &'static str,
    site: &'static str,
}

const WIKIS: [Wiki; 1] = [Wiki {
    name: "yue-wiki",
    source_id: "yue_wiki",
    title: "Cantonese Wikipedia",
    site: "https://zh-yue.wikipedia.org/",
}];

/// 進資料包的來源：名字、`sources.toml` 的 id、取用的檔、取用的內容（寫進 NOTICE）。
/// libcangjie 給碼，Unihan 的 `kGradeLevel` 給字序（plan §4.1）。
const PACKED: [(&str, &str, &str, &str); 2] = [
    (
        "libcangjie",
        LIBCANGJIE,
        TABLE,
        "Cangjie 3 and Cangjie 5 codes, Big5 and HKSCS flags, Big5 order",
    ),
    (
        "Unihan",
        UNIHAN,
        UNIHAN_ZIP,
        "field kGradeLevel of Unihan_DictionaryLikeData.txt",
    ),
];

/// 一個進資料包的來源，授權已核對過。
struct Packed<'a> {
    name: &'a str,
    source: &'a Source,
    file: &'a sources::SourceFile,
    used: &'a str,
    license: License,
}

/// 要寫進資料包 `LICENSE` 的一段授權。
struct License {
    /// 這段授權從哪裡來，寫在原文前一行。
    origin: String,
    text: String,
}

/// 建一次的結果；`--verify` 會再建一次比 `bytes`。
struct Build {
    rows: Vec<libcangjie::Row>,
    /// `Unihan_DictionaryLikeData.txt` 的內容。
    dictionary_like: String,
    bytes: Vec<u8>,
    scheme_id: String,
}

#[derive(Debug)]
pub struct Outcome {
    pub pack_path: PathBuf,
    pub sha256: String,
    /// 各語料的句數。
    pub corpora: Vec<CorpusSummary>,
    /// 只有要求驗證時才有。
    pub report: Option<Report>,
}

/// 在 repo 根目錄 `root` 建速成資料包到 `build/packs/<方案 id>/`，旁邊放 `LICENSE` 與 `NOTICE`。
pub fn run(root: &Path, fetch: &dyn Fetch, verify: bool) -> Result<Outcome, Error> {
    let read = |rel: &str| {
        let path = root.join(rel);
        std::fs::read_to_string(&path).map_err(Error::io(path))
    };
    let all = sources::parse(&read(SOURCES)?)?;
    let source_of = |id: &str| {
        all.source
            .get(id)
            .ok_or_else(|| Error::invalid(SOURCES, format!("沒有 [source.{id}]")))
    };
    let cache = root.join(CACHE);
    let load = |id: &str, path: &str| -> Result<Vec<u8>, Error> {
        sources::load(&cache, id, source_of(id)?.file(path)?, fetch)
    };
    let load_text = |id: &str, path: &str| -> Result<String, Error> {
        String::from_utf8(load(id, path)?).map_err(|e| Error::invalid(path, e.to_string()))
    };
    let mut packed = Vec::new();
    for (name, id, path, used) in PACKED {
        let source = source_of(id)?;
        let license = license_of(source, |file| load_text(id, file), |path| read(path))?;
        packed.push(Packed {
            name,
            source,
            file: source.file(path)?,
            used,
            license,
        });
    }

    // 每次都從快取與方案檔重讀，`--verify` 的第二次建置才是真的重建。
    let build = || -> Result<Build, Error> {
        let rows = libcangjie::parse(&load_text(LIBCANGJIE, TABLE)?)?;
        let dictionary_like = unihan::read_archive(&load(UNIHAN, UNIHAN_ZIP)?)?;
        let grades = unihan::grade_levels(&dictionary_like)?;
        let scheme = scheme::parse(&read(QUICK_SCHEME)?)?;
        let scheme_id = scheme.id.clone();
        let bytes = build_pack(&rows, &grades, scheme)
            .to_bytes()
            .map_err(|e| Error::invalid("資料包", e.to_string()))?;
        Ok(Build {
            rows,
            dictionary_like,
            bytes,
            scheme_id,
        })
    };
    let Build {
        rows,
        dictionary_like,
        bytes,
        scheme_id: id,
    } = build()?;

    let dir = root.join(PACKS).join(&id);
    std::fs::create_dir_all(&dir).map_err(Error::io(&dir))?;
    let pack_path = dir.join(format!("{id}.pack"));
    let write =
        |path: PathBuf, contents: &[u8]| std::fs::write(&path, contents).map_err(Error::io(path));
    write(pack_path.clone(), &bytes)?;
    write(dir.join("LICENSE"), license(&packed).as_bytes())?;
    write(dir.join("NOTICE"), notice(&id, &packed).as_bytes())?;

    let transcoder = Transcoder::new(&rows);
    let mut corpora = Vec::new();
    let mut corpus_checks = Vec::new();
    let mut attributions = Vec::new();
    let bench_dir = root.join(BENCH_CORPUS);
    std::fs::create_dir_all(&bench_dir).map_err(Error::io(&bench_dir))?;
    for wiki in &WIKIS {
        let source = source_of(wiki.source_id)?;
        let [file] = source.files.as_slice() else {
            return Err(Error::invalid(
                SOURCES,
                format!("[source.{}] 要剛好列一個 dump 檔", wiki.source_id),
            ));
        };
        let dir = root.join(BUILD_CORPUS).join(wiki.name);
        std::fs::create_dir_all(&dir).map_err(Error::io(&dir))?;
        let (train_path, dev_path) = (dir.join("train.txt"), dir.join("dev.txt"));
        let create = |path: &Path| {
            File::create(path)
                .map(|f| Hashed::new(BufWriter::new(f)))
                .map_err(Error::io(path))
        };
        let dump = load(wiki.source_id, &file.path)?;
        let first = build_wiki(
            &dump,
            &transcoder,
            create(&train_path)?,
            create(&dev_path)?,
            &dir,
        )?;
        let bench_rel = format!("{BENCH_CORPUS}/{}.tsv", wiki.name);
        write(root.join(&bench_rel), first.bench.as_bytes())?;
        corpora.push(CorpusSummary {
            name: wiki.name.into(),
            counts: first.counts,
            bench_sentences: first.bench.lines().count(),
            bench: bench_rel.into(),
        });
        attributions.push(attribution(wiki, source, file));
        if verify {
            // 第二次從快取重讀 dump，只算 sha256、不寫檔。
            drop(dump);
            let dump = load(wiki.source_id, &file.path)?;
            let sink = || Hashed::new(io::sink());
            let second = build_wiki(&dump, &transcoder, sink(), sink(), &dir)?;
            let open = |path: &Path| {
                File::open(path)
                    .map(BufReader::new)
                    .map_err(Error::io(path))
            };
            corpus_checks.extend(
                verify::corpus(
                    wiki.name,
                    &transcoder,
                    &first.bench,
                    open(&train_path)?,
                    open(&dev_path)?,
                    &first.digests(),
                    &second.digests(),
                )
                .map_err(Error::io(&dir))?,
            );
        }
    }
    write(
        bench_dir.join("ATTRIBUTION.md"),
        attribution_file(&attributions).as_bytes(),
    )?;

    let report = if verify {
        let second = build()?.bytes;
        let mut report = verify::run(&rows, &bytes, &second);
        // Unihan 交叉比對的結果只寫報表，不影響通過與否。
        let check = unihan::compare(&rows, &unihan::cangjie_codes(&dictionary_like)?);
        let path = root.join(UNIHAN_REPORT);
        let reports = path.parent().expect("報表路徑有上層目錄");
        std::fs::create_dir_all(reports).map_err(Error::io(reports))?;
        write(path, check.to_tsv().as_bytes())?;
        report.unihan = Some(UnihanSummary {
            compared: check.compared,
            differences: check.differences.len(),
            matching_cangjie5: check.matching_cangjie5(),
            report: UNIHAN_REPORT.into(),
        });
        report.corpora = corpora.clone();
        report.checks.extend(corpus_checks);
        Some(report)
    } else {
        None
    };
    Ok(Outcome {
        pack_path,
        sha256: sha256_hex(&bytes),
        corpora,
        report,
    })
}

/// 一份維基 dump 建一次語料；train、dev 寫進兩個 writer，同時算 sha256。寫入失敗時報 `dir`。
fn build_wiki<W: Write>(
    dump: &[u8],
    transcoder: &Transcoder,
    train: Hashed<W>,
    dev: Hashed<W>,
    dir: &Path,
) -> Result<Built, Error> {
    let mut builder = Builder::new(transcoder, train, dev);
    wiki::articles(dump, |text| builder.add(text).map_err(Error::io(dir)))?;
    let Finished {
        counts,
        bench,
        train,
        dev,
    } = builder.finish().map_err(Error::io(dir))?;
    Ok(Built {
        counts,
        bench,
        train: train.sha256(),
        dev: dev.sha256(),
    })
}

/// 建一次語料的結果，train、dev 只留 sha256。
struct Built {
    counts: crate::corpus::Counts,
    bench: String,
    train: String,
    dev: String,
}

impl Built {
    /// 比較兩次建置用。
    fn digests(&self) -> CorpusDigests {
        CorpusDigests {
            bench: sha256_hex(self.bench.as_bytes()),
            train: self.train.clone(),
            dev: self.dev.clone(),
        }
    }
}

/// 寫入的同時算 sha256。
struct Hashed<W> {
    inner: W,
    hasher: Sha256,
}

impl<W: Write> Hashed<W> {
    fn new(inner: W) -> Self {
        Self {
            inner,
            hasher: Sha256::new(),
        }
    }

    fn sha256(self) -> String {
        hex(&self.hasher.finalize())
    }
}

impl<W: Write> Write for Hashed<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// `bench/corpus/ATTRIBUTION.md` 裡一份語料的段落。
fn attribution(wiki: &Wiki, source: &Source, file: &sources::SourceFile) -> String {
    format!(
        "## {name}.tsv\n\
         \n\
         - Source: {title}, {site}, by its contributors (each article's page history lists its authors).\n\
         - Dump: {version}, file {path} (sha256 {sha256}), {url}\n\
         - License: {license}\n",
        name = wiki.name,
        title = wiki.title,
        site = wiki.site,
        version = source.version,
        path = file.path,
        sha256 = file.sha256,
        url = file.url,
        license = source.license,
    )
}

fn attribution_file(sections: &[String]) -> String {
    format!(
        "# Attribution\n\
         \n\
         The sentences in this directory are derived from Wikipedia articles and are licensed\n\
         under the Creative Commons Attribution-ShareAlike 4.0 International License\n\
         (CC BY-SA 4.0), https://creativecommons.org/licenses/by-sa/4.0/. They are test data\n\
         for the QIM top-choice benchmark and are not shipped in any app.\n\
         \n\
         Changes made by QIM: article text was extracted from the wikitext (templates, tables,\n\
         references, file links, headings and lists removed) and converted with OpenCC s2hk;\n\
         the text was split into clauses at punctuation and whitespace, and clauses containing\n\
         characters outside the QIM continuous character set were dropped. Each file keeps up to\n\
         {BENCH_SENTENCES} distinct clauses from the held-out test split, one per line, preceded by\n\
         machine-generated Quick codes (one per character, separated by spaces) and a tab.\n\
         \n\
         {}",
        sections.join("\n")
    )
}

/// 進資料包的來源二選一：照抄上游檔裡的聲明（`license_file` 加 `license_statement`），
/// 或 repo 裡存一份授權全文（`license_text`）。`upstream` 讀來源檔，`repo` 讀 repo 裡的檔。
fn license_of(
    source: &Source,
    upstream: impl Fn(&str) -> Result<String, Error>,
    repo: impl Fn(&str) -> Result<String, Error>,
) -> Result<License, Error> {
    match (
        &source.license_file,
        &source.license_statement,
        &source.license_text,
    ) {
        (Some(file), Some(statement), None) => {
            check_license_statement(source, file, statement, &upstream(file)?)?;
            Ok(License {
                origin: format!("Statement in {file}"),
                text: statement.clone(),
            })
        }
        (None, None, Some(path)) => Ok(License {
            origin: format!("Full text, kept in the QIM repository as {path}"),
            text: repo(path)?,
        }),
        _ => Err(Error::invalid(
            SOURCES,
            format!(
                "{} 的資料進資料包，要寫 license_file 加 license_statement，或只寫 license_text",
                source.repository
            ),
        )),
    }
}

/// sources.toml 照抄的聲明原文要真的出現在上游檔裡（換行與空白不計）。
fn check_license_statement(
    source: &Source,
    file: &str,
    statement: &str,
    upstream: &str,
) -> Result<(), Error> {
    let squash = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    if squash(upstream).contains(&squash(statement)) {
        Ok(())
    } else {
        Err(Error::invalid(
            SOURCES,
            format!("license_statement 不在 {} 的 {file} 裡", source.repository),
        ))
    }
}

/// 每個進資料包的來源一段：來源、授權、照抄的聲明。
fn license(packed: &[Packed]) -> String {
    packed
        .iter()
        .map(|p| {
            format!(
                "Source: {name}, {repo} ({version})\n\
                 License: {license}\n\
                 \n\
                 {origin}:\n\
                 \n\
                 {text}\n",
                name = p.name,
                repo = p.source.repository,
                version = p.source.version,
                license = p.source.license,
                origin = p.license.origin,
                text = p.license.text.trim_end(),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn notice(id: &str, packed: &[Packed]) -> String {
    let sources: String = packed
        .iter()
        .map(|p| {
            format!(
                "Source: {name}\n\
                 Repository: {repo}\n\
                 Version: {version}\n\
                 File: {path} (sha256 {sha256})\n\
                 Used: {used}\n\
                 License: {license} (see LICENSE)\n\
                 \n",
                name = p.name,
                repo = p.source.repository,
                version = p.source.version,
                path = p.file.path,
                sha256 = p.file.sha256,
                used = p.used,
                license = p.source.license,
            )
        })
        .collect();
    format!(
        "{id}.pack: QIM input scheme pack \"{id}\".\n\
         \n\
         {sources}\
         Quick codes are derived from the Cangjie 3 and Cangjie 5 codes in the libcangjie\n\
         file: the first and last letter of each code, or the code itself when it has one\n\
         or two letters. Han characters flagged as Big5 or HKSCS form the continuous set;\n\
         other Han characters are marked as extension. Within each code, the continuous set\n\
         comes before extension; within each set, characters with a Unihan kGradeLevel\n\
         come first by grade, then libcangjie's Big5 order.\n"
    )
}
