//! `cargo xtask data` 的整條流程：讀來源、建資料包、寫產物，選擇性驗證。

use std::fs::File;
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use qim_core::{LexiconPack, Pack};

use crate::corpora::{self, CORPORA, Corpus, Group};
use crate::corpus::{Builder, Finished};
use crate::sources::{self, Fetch, Hashed, SOURCES, sha256_hex};
use crate::transcode::Transcoder;
use crate::verify::{
    self, CorpusDigests, CorpusSummary, LexiconSummary, PackBytes, Report, UnihanSummary,
};
use crate::{Error, build_pack, lexicon, libcangjie, packing, personal, scheme, unihan};

const QUICK_SCHEME: &str = "data/schemes/quick.toml";
const PACKS: &str = "build/packs";
pub(crate) const LIBCANGJIE: &str = "libcangjie";
const TABLE: &str = "data/table.txt";
pub(crate) const UNIHAN: &str = "unihan";
const UNIHAN_ZIP: &str = "Unihan.zip";
pub(crate) const RIME_ESSAY: &str = "rime_essay";
const ESSAY: &str = "essay.txt";
/// Unihan 交叉比對的差異報表，相對 repo 根目錄；格式見 `unihan::CrossCheck::to_tsv`。
const UNIHAN_REPORT: &str = "build/reports/unihan-kcangjie.tsv";

/// 速成資料包 NOTICE 最後一段：怎麼用來源。
const QUICK_ABOUT: &str = "\
Quick codes are derived from the Cangjie 3 and Cangjie 5 codes in the libcangjie
file: the first and last letter of each code, or the code itself when it has one
or two letters. Han characters flagged as Big5 or HKSCS form the continuous set;
other Han characters are marked as extension. Within each code, the continuous set
comes before extension; within each set, characters with a Unihan kGradeLevel
come first by grade, then libcangjie's Big5 order. At run time QIM puts characters
with a higher frequency in the lexicon pack first.
";

/// 建一次速成資料包的結果；`--verify` 會再建一次比 `bytes`。
struct QuickBuild {
    rows: Vec<libcangjie::Row>,
    /// `Unihan_DictionaryLikeData.txt` 的內容。
    dictionary_like: String,
    pack: Pack,
    bytes: Vec<u8>,
}

#[derive(Debug)]
pub struct Outcome {
    pub pack_path: PathBuf,
    pub sha256: String,
    pub lexicon_path: PathBuf,
    pub lexicon_sha256: String,
    pub lexicon: LexiconSummary,
    /// 各語料的句數。
    pub corpora: Vec<CorpusSummary>,
    /// 個人集轉出的統計；沒有 [`personal::USERDB`] 時是 `None`。
    pub personal: Option<personal::Counts>,
    /// 只有要求驗證時才有。
    pub report: Option<Report>,
}

/// 在 repo 根目錄 `root` 建速成資料包到 `build/packs/<方案 id>/`、建各語料，再用語料的字頻
/// 建方案指名的詞庫資料包到 `build/packs/<詞庫 id>/`；每個資料包旁邊放 `LICENSE` 與 `NOTICE`。
/// 來源下載到 `cache` 下的 `sources/`（[`sources::default_cache_dir`]）。
pub fn run(root: &Path, cache: &Path, fetch: &dyn Fetch, verify: bool) -> Result<Outcome, Error> {
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
    let cache = cache.join("sources");
    let load = |id: &str, path: &str| -> Result<Vec<u8>, Error> {
        sources::load(&cache, id, source_of(id)?.file(path)?, fetch)
    };
    let load_text = |id: &str, path: &str| -> Result<String, Error> {
        String::from_utf8(load(id, path)?).map_err(|e| Error::invalid(path, e.to_string()))
    };
    let lexicon_sources = packing::lexicon_sources();
    let quick_packed = packing::resolve(&packing::QUICK, &all, load_text, read)?;
    let lexicon_packed = packing::resolve(&lexicon_sources, &all, load_text, read)?;

    // 每次都從快取與方案檔重讀，`--verify` 的第二次建置才是真的重建。
    let build_quick = || -> Result<QuickBuild, Error> {
        let rows = libcangjie::parse(&load_text(LIBCANGJIE, TABLE)?)?;
        let dictionary_like = unihan::read_archive(&load(UNIHAN, UNIHAN_ZIP)?)?;
        let grades = unihan::grade_levels(&dictionary_like)?;
        let pack = build_pack(&rows, &grades, scheme::parse(&read(QUICK_SCHEME)?)?);
        let bytes = pack
            .to_bytes()
            .map_err(|e| Error::invalid("資料包", e.to_string()))?;
        Ok(QuickBuild {
            rows,
            dictionary_like,
            pack,
            bytes,
        })
    };
    let quick = build_quick()?;
    let id = quick.pack.scheme.id.clone();
    let lexicon_id =
        quick.pack.scheme.packs.lexicon.clone().ok_or_else(|| {
            Error::invalid(QUICK_SCHEME, "要在 [packs] 寫 lexicon：詞庫資料包的 id")
        })?;
    let dir = root.join(PACKS).join(&id);
    packing::write(&dir, &id, &quick.bytes, &quick_packed, QUICK_ABOUT)?;
    let pack_path = dir.join(format!("{id}.pack"));
    let write =
        |path: PathBuf, contents: &[u8]| std::fs::write(&path, contents).map_err(Error::io(path));

    let transcoder = Transcoder::new(&quick.rows);
    let mut corpora = Vec::new();
    let mut corpus_checks = Vec::new();
    let mut attributions: Vec<(Group, String)> = Vec::new();
    // 各語料兩次建置的字頻，給詞庫資料包。
    let mut chars: Vec<String> = Vec::new();
    let mut second_chars: Vec<String> = Vec::new();
    // 驗證用：各語料的基準檔內容與 train 檔，洩漏要拿每個基準集比對所有 train。
    let mut benches: Vec<(&str, String)> = Vec::new();
    let mut trains = Vec::new();
    for corpus in &CORPORA {
        let source = source_of(corpus.source_id)?;
        let paths = source
            .files
            .iter()
            .map(|file| sources::cached(&cache, corpus.source_id, file, fetch))
            .collect::<Result<Vec<_>, _>>()?;
        let dir = corpora::build_dir(root, corpus);
        std::fs::create_dir_all(&dir).map_err(Error::io(&dir))?;
        let (train_path, dev_path) = (dir.join("train.txt"), dir.join("dev.txt"));
        let create = |path: &Path| {
            File::create(path)
                .map(|f| Hashed::new(BufWriter::new(f)))
                .map_err(Error::io(path))
        };
        let first = build_corpus(
            corpus,
            &paths,
            &transcoder,
            create(&train_path)?,
            create(&dev_path)?,
            &dir,
        )?;
        let bench_dir = root.join(corpus.group.dir());
        std::fs::create_dir_all(&bench_dir).map_err(Error::io(&bench_dir))?;
        let bench_rel = format!("{}/{}.tsv", corpus.group.dir(), corpus.name);
        write(root.join(&bench_rel), first.bench.as_bytes())?;
        write(dir.join("chars.tsv"), first.chars.as_bytes())?;
        corpora.push(CorpusSummary {
            name: corpus.name.into(),
            counts: first.counts,
            bench_sentences: first.bench.lines().count(),
            bench: bench_rel.into(),
        });
        attributions.push((corpus.group, corpus.attribution(source)));
        if verify {
            // 第二次從快取重讀來源，只算 sha256、不寫檔。
            let sink = || Hashed::new(io::sink());
            let second = build_corpus(corpus, &paths, &transcoder, sink(), sink(), &dir)?;
            corpus_checks.extend(
                verify::corpus(
                    corpus.name,
                    &transcoder,
                    &first.bench,
                    open(&train_path)?,
                    open(&dev_path)?,
                    &first.digests(),
                    &second.digests(),
                )
                .map_err(Error::io(&dir))?,
            );
            second_chars.push(second.chars);
            trains.push(train_path);
        }
        chars.push(first.chars);
        if verify {
            benches.push((corpus.name, first.bench));
        }
    }
    for group in Group::ALL {
        let sections: Vec<String> = attributions
            .iter()
            .filter(|(g, _)| *g == group)
            .map(|(_, section)| section.clone())
            .collect();
        write(
            root.join(group.dir()).join("ATTRIBUTION.md"),
            corpora::attribution_file(group, &sections).as_bytes(),
        )?;
    }

    // 詞庫的碼由這次建出的速成碼表展開，字頻由各語料的 train 相加；每次從快取重讀 essay.txt。
    let build_lexicon =
        |quick: &QuickBuild, chars: &[String]| -> Result<(Vec<u8>, LexiconSummary), Error> {
            let tables: Vec<&str> = chars.iter().map(String::as_str).collect();
            let frequency = lexicon::char_frequency(&tables)?;
            let characters = frequency.len();
            let (lexicon, counts) = lexicon::build(
                &load_text(RIME_ESSAY, ESSAY)?,
                &quick.pack.code_table,
                &Transcoder::new(&quick.rows),
                frequency,
            )?;
            let bytes = LexiconPack {
                id: lexicon_id.clone(),
                lexicon,
            }
            .to_bytes()
            .map_err(|e| Error::invalid("詞庫資料包", e.to_string()))?;
            Ok((bytes, LexiconSummary { counts, characters }))
        };
    let (lexicon_bytes, lexicon_summary) = build_lexicon(&quick, &chars)?;
    let lexicon_dir = root.join(PACKS).join(&lexicon_id);
    packing::write(
        &lexicon_dir,
        &lexicon_id,
        &lexicon_bytes,
        &lexicon_packed,
        &lexicon_about(&lexicon_id, &id),
    )?;
    let lexicon_path = lexicon_dir.join(format!("{lexicon_id}.pack"));

    let personal = build_personal(root, &transcoder)?;
    if verify {
        let benches: Vec<(&str, &str)> = benches.iter().map(|(n, b)| (*n, b.as_str())).collect();
        let readers = trains
            .iter()
            .map(|p| open(p))
            .collect::<Result<Vec<_>, _>>()?;
        corpus_checks.extend(
            verify::leaks(&benches, readers).map_err(Error::io(root.join("build/corpus")))?,
        );
    }

    let report = if verify {
        // 第二次從來源重建速成碼表，詞庫也由它重建，整條鏈都比到。
        let second_quick = build_quick()?;
        let (second_lexicon, _) = build_lexicon(&second_quick, &second_chars)?;
        let mut report = verify::run(
            &quick.rows,
            PackBytes {
                quick: &quick.bytes,
                lexicon: &lexicon_bytes,
            },
            PackBytes {
                quick: &second_quick.bytes,
                lexicon: &second_lexicon,
            },
        );
        // Unihan 交叉比對的結果只寫報表，不影響通過與否。
        let check = unihan::compare(&quick.rows, &unihan::cangjie_codes(&quick.dictionary_like)?);
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
        report.lexicon = Some(lexicon_summary);
        report.corpora = corpora.clone();
        report.checks.extend(corpus_checks);
        Some(report)
    } else {
        None
    };
    Ok(Outcome {
        pack_path,
        sha256: sha256_hex(&quick.bytes),
        lexicon_path,
        lexicon_sha256: sha256_hex(&lexicon_bytes),
        lexicon: lexicon_summary,
        corpora,
        personal,
        report,
    })
}

/// 詞庫資料包 NOTICE 最後一段：怎麼用來源。
fn lexicon_about(id: &str, scheme: &str) -> String {
    format!(
        "{id} is the lexicon of the QIM input scheme \"{scheme}\". QIM changed the rime-essay\n\
         data as follows: single characters, words with a frequency below {min}, and words\n\
         containing characters outside the continuous set of the scheme were dropped; a word\n\
         listed more than once keeps its highest frequency. Each word is indexed by the\n\
         Quick codes of its characters joined, at most {max} combinations per word.\n\
         \n\
         Character frequencies are the counts of each character in the training split of\n\
         each corpus, added together. Each corpus is extracted, converted with OpenCC s2hk\n\
         and split into clauses as described in bench/corpus/ATTRIBUTION.md and\n\
         bench/colloquial/ATTRIBUTION.md of the QIM repository.\n",
        min = lexicon::MIN_FREQUENCY,
        max = lexicon::MAX_COMBINATIONS,
    )
}

/// 有 [`personal::USERDB`] 就轉出 [`personal::ITEMS`]；沒有就刪掉上次轉出的，基準才不會跑舊題目。
fn build_personal(root: &Path, transcoder: &Transcoder) -> Result<Option<personal::Counts>, Error> {
    let items = root.join(personal::ITEMS);
    let userdb = root.join(personal::USERDB);
    let text = match std::fs::read_to_string(&userdb) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => match std::fs::remove_file(&items) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => {
                return Err(Error::io(&items)(e));
            }
            _ => return Ok(None),
        },
        Err(e) => return Err(Error::io(&userdb)(e)),
    };
    let (tsv, counts) = personal::from_userdb(&text, transcoder)
        .map_err(|e| Error::invalid(personal::USERDB, e))?;
    let header = format!(
        "{}{}\n",
        personal::SOURCE_HEADER,
        sha256_hex(text.as_bytes())
    );
    std::fs::write(&items, header + &tsv).map_err(Error::io(&items))?;
    Ok(Some(counts))
}

/// 一份語料建一次；train、dev 寫進兩個 writer，同時算 sha256。寫入失敗時報 `dir`。
fn build_corpus<W: Write>(
    corpus: &Corpus,
    paths: &[PathBuf],
    transcoder: &Transcoder,
    train: Hashed<W>,
    dev: Hashed<W>,
    dir: &Path,
) -> Result<Built, Error> {
    let mut builder = Builder::new(transcoder, train, dev);
    corpus.texts(paths, |text| builder.add(text).map_err(Error::io(dir)))?;
    let Finished {
        counts,
        bench,
        chars,
        train,
        dev,
    } = builder.finish().map_err(Error::io(dir))?;
    Ok(Built {
        counts,
        bench,
        chars,
        train: train.sha256(),
        dev: dev.sha256(),
    })
}

fn open(path: &Path) -> Result<BufReader<File>, Error> {
    File::open(path)
        .map(BufReader::new)
        .map_err(Error::io(path))
}

/// 建一次語料的結果，train、dev 只留 sha256。
struct Built {
    counts: crate::corpus::Counts,
    bench: String,
    chars: String,
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
            chars: sha256_hex(self.chars.as_bytes()),
        }
    }
}
