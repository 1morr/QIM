//! 線上更新的 release 目錄（plan〈線上更新〉）：`build/packs/<id>/` 的四個檔改名成 `<id>.pack`、
//! `<id>.manifest.json`、`<id>.LICENSE`、`<id>.NOTICE`（release asset 的名字不能重複，每份資料包的
//! `LICENSE` 會撞名），加一份列出全部的 `index.json`。入口是 `cargo xtask pack-release`。

use std::path::{Path, PathBuf};

use qim_core::{IndexEntry, MANIFEST_FILE, Manifest, PACK_INDEX_FILE, PackIndex};

/// release 目錄，相對於 repo 根目錄。
pub const RELEASE_DIR: &str = "build/pack-release";

/// 一份資料包要放上 release 的檔：(release 上的檔名, 內容)。
type Asset = (String, Vec<u8>);

/// 讀 `packs` 底下每個資料包目錄，核對 manifest 對得上資料包檔、有 `LICENSE` 與 `NOTICE`；給了上一版
/// 索引 `previous` 時再核對內容改了的都加了版本（[`PackIndex::check_successor`]）。全部過了才清空
/// `out` 寫出，任一項不過時回說明、`out` 不動。索引依 id 排序。
pub fn write(packs: &Path, out: &Path, previous: Option<&PackIndex>) -> Result<PackIndex, String> {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(packs)
        .map_err(|e| format!("讀不了 {}：{e}，先跑 cargo xtask data", packs.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.join(MANIFEST_FILE).is_file())
        .collect();
    dirs.sort();
    if dirs.is_empty() {
        return Err(format!("{} 底下沒有資料包", packs.display()));
    }

    let mut entries = Vec::new();
    let mut assets: Vec<Asset> = Vec::new();
    for dir in &dirs {
        let (entry, files) = collect(dir)?;
        entries.push(entry);
        assets.extend(files);
    }
    let index = PackIndex::new(entries);
    if let Some(previous) = previous {
        index
            .check_successor(previous)
            .map_err(|problems| problems.join("\n"))?;
    }

    if out.exists() {
        std::fs::remove_dir_all(out).map_err(|e| format!("清不掉 {}：{e}", out.display()))?;
    }
    std::fs::create_dir_all(out).map_err(|e| format!("建不了 {}：{e}", out.display()))?;
    assets.push((PACK_INDEX_FILE.into(), index.to_json().into_bytes()));
    for (name, bytes) in assets {
        let path = out.join(name);
        std::fs::write(&path, bytes).map_err(|e| format!("寫不了 {}：{e}", path.display()))?;
    }
    Ok(index)
}

/// 一個資料包目錄：索引的一項與它在 release 上的檔。
fn collect(dir: &Path) -> Result<(IndexEntry, Vec<Asset>), String> {
    let read = |name: &str| {
        let path = dir.join(name);
        std::fs::read(&path).map_err(|e| format!("讀不了 {}：{e}", path.display()))
    };
    let manifest_json = read(MANIFEST_FILE)?;
    let manifest = Manifest::from_json(&String::from_utf8_lossy(&manifest_json))
        .map_err(|e| format!("{}：{e}", dir.display()))?;
    let id = manifest.id.clone();
    let bytes = read(&manifest.file_name())?;
    manifest.verify(&bytes).map_err(|e| format!("{id}：{e}"))?;
    let url = manifest.file_name();
    let assets = vec![
        (url.clone(), bytes),
        (format!("{id}.manifest.json"), manifest_json),
        (format!("{id}.LICENSE"), read("LICENSE")?),
        (format!("{id}.NOTICE"), read("NOTICE")?),
    ];
    Ok((IndexEntry { manifest, url }, assets))
}
