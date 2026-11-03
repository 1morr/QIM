//! `cargo xtask data` 的整條流程：讀來源、建資料包、寫產物，選擇性驗證。

use std::path::{Path, PathBuf};

use crate::sources::{self, Fetch, SOURCES, Source, sha256_hex};
use crate::verify::{self, Report, UnihanSummary};
use crate::{Error, build_pack, libcangjie, scheme, unihan};

const QUICK_SCHEME: &str = "data/schemes/quick.toml";
const CACHE: &str = ".cache/sources";
const PACKS: &str = "build/packs";
const LIBCANGJIE: &str = "libcangjie";
const TABLE: &str = "data/table.txt";
const UNIHAN: &str = "unihan";
const UNIHAN_ZIP: &str = "Unihan.zip";
/// Unihan 交叉比對的差異報表，相對 repo 根目錄；格式見 `unihan::CrossCheck::to_tsv`。
const UNIHAN_REPORT: &str = "build/reports/unihan-kcangjie.tsv";

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

/// 一個進資料包的來源，授權聲明已核對過。
struct Packed<'a> {
    name: &'a str,
    source: &'a Source,
    file: &'a sources::SourceFile,
    used: &'a str,
    license_file: &'a str,
    statement: &'a str,
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
        let (license_file, statement) = license_notice(source)?;
        check_license_statement(
            source,
            license_file,
            statement,
            &load_text(id, license_file)?,
        )?;
        packed.push(Packed {
            name,
            source,
            file: source.file(path)?,
            used,
            license_file,
            statement,
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

    let report = if verify {
        let second = build()?.bytes;
        let mut report = verify::run(&rows, &bytes, &second);
        // Unihan 交叉比對的結果只寫報表，不影響通過與否。
        let check = unihan::compare(&rows, &unihan::parse(&dictionary_like)?);
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
        Some(report)
    } else {
        None
    };
    Ok(Outcome {
        pack_path,
        sha256: sha256_hex(&bytes),
        report,
    })
}

/// 進資料包的來源要寫授權聲明所在的檔與聲明原文。
fn license_notice(source: &Source) -> Result<(&str, &str), Error> {
    match (&source.license_file, &source.license_statement) {
        (Some(file), Some(statement)) => Ok((file, statement)),
        _ => Err(Error::invalid(
            SOURCES,
            format!(
                "{} 的資料進資料包，要寫 license_file 與 license_statement",
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
                 Statement in {file}:\n\
                 \n\
                 {statement}\n",
                name = p.name,
                repo = p.source.repository,
                version = p.source.version,
                license = p.source.license,
                file = p.license_file,
                statement = p.statement,
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
         other Han characters are marked as extension. Within each code, characters with a\n\
         Unihan kGradeLevel come first by grade, then libcangjie's Big5 order.\n"
    )
}
