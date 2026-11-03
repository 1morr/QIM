//! `cargo xtask data` 的整條流程：讀來源、建資料包、寫產物，選擇性驗證。

use std::path::{Path, PathBuf};

use crate::sources::{self, Fetch, SOURCES, Source, sha256_hex};
use crate::verify::{self, Report};
use crate::{Error, build_pack, libcangjie, scheme};

const QUICK_SCHEME: &str = "data/schemes/quick.toml";
const CACHE: &str = ".cache/sources";
const PACKS: &str = "build/packs";
const LIBCANGJIE: &str = "libcangjie";
const TABLE: &str = "data/table.txt";

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
    let source = all
        .source
        .get(LIBCANGJIE)
        .ok_or_else(|| Error::invalid(SOURCES, format!("沒有 [source.{LIBCANGJIE}]")))?;
    let cache = root.join(CACHE);
    let load = |path: &str| -> Result<String, Error> {
        let bytes = sources::load(&cache, LIBCANGJIE, source.file(path)?, fetch)?;
        String::from_utf8(bytes).map_err(|e| Error::invalid(path, e.to_string()))
    };
    check_license_statement(source, &load(&source.license_file)?)?;

    // 每次都從快取與方案檔重讀，`--verify` 的第二次建置才是真的重建。
    let build = || -> Result<(Vec<libcangjie::Row>, Vec<u8>, String), Error> {
        let rows = libcangjie::parse(&load(TABLE)?)?;
        let scheme = scheme::parse(&read(QUICK_SCHEME)?)?;
        let id = scheme.id.clone();
        let bytes = build_pack(&rows, scheme)
            .to_bytes()
            .map_err(|e| Error::invalid("資料包", e.to_string()))?;
        Ok((rows, bytes, id))
    };
    let (rows, bytes, id) = build()?;

    let dir = root.join(PACKS).join(&id);
    std::fs::create_dir_all(&dir).map_err(Error::io(&dir))?;
    let pack_path = dir.join(format!("{id}.pack"));
    let write =
        |path: PathBuf, contents: &[u8]| std::fs::write(&path, contents).map_err(Error::io(path));
    write(pack_path.clone(), &bytes)?;
    write(dir.join("LICENSE"), license(source).as_bytes())?;
    write(
        dir.join("NOTICE"),
        notice(&id, source, source.file(TABLE)?).as_bytes(),
    )?;

    let report = if verify {
        let (_, second, _) = build()?;
        Some(verify::run(&rows, &bytes, &second))
    } else {
        None
    };
    Ok(Outcome {
        pack_path,
        sha256: sha256_hex(&bytes),
        report,
    })
}

/// sources.toml 照抄的聲明原文要真的出現在上游檔裡（換行與空白不計）。
fn check_license_statement(source: &Source, upstream: &str) -> Result<(), Error> {
    let squash = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    if squash(upstream).contains(&squash(&source.license_statement)) {
        Ok(())
    } else {
        Err(Error::invalid(
            SOURCES,
            format!(
                "license_statement 不在 {} 的 {} 裡",
                source.repository, source.license_file
            ),
        ))
    }
}

fn license(source: &Source) -> String {
    format!(
        "The code table in this pack is derived from {table} of libcangjie.\n\
         License of that data: {license}\n\
         \n\
         Statement in {file} ({repo}, commit {commit}):\n\
         \n\
         {statement}\n",
        table = TABLE,
        license = source.license,
        file = source.license_file,
        repo = source.repository,
        commit = source.commit,
        statement = source.license_statement,
    )
}

fn notice(id: &str, source: &Source, table: &sources::SourceFile) -> String {
    format!(
        "{id}.pack: QIM input scheme pack \"{id}\".\n\
         \n\
         Source: libcangjie\n\
         Repository: {repo}\n\
         Commit: {commit}\n\
         File: {path} (sha256 {sha256})\n\
         License: {license} (see LICENSE)\n\
         \n\
         Quick codes are derived from the Cangjie 3 and Cangjie 5 codes in that file:\n\
         the first and last letter of each code, or the code itself when it has one or\n\
         two letters. Han characters flagged as Big5 or HKSCS form the continuous set;\n\
         other Han characters are marked as extension.\n",
        repo = source.repository,
        commit = source.commit,
        path = table.path,
        sha256 = table.sha256,
        license = source.license,
    )
}
