//! `qim-databuild [--verify]`：在 repo 根目錄建資料包；平常由 `cargo xtask data` 呼叫。
//! `qim-databuild --pack-release [--previous <index.json>]`：把建好的資料包整理成線上更新的 release
//! 目錄（plan §4.7）；平常由 `cargo xtask pack-release` 呼叫。

use std::path::Path;
use std::process::ExitCode;

use qim_core::PackIndex;
use qim_databuild::sources::{CACHE_ENV, Http, default_cache_dir};
use qim_databuild::{PACKS_DIR, personal, release};

const USAGE: &str = "用法：qim-databuild [--verify] | qim-databuild --pack-release [--previous <index.json>]（在 repo 根目錄執行）";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        [] => build(false),
        ["--verify"] => build(true),
        ["--pack-release"] => pack_release(None),
        ["--pack-release", "--previous", previous] => pack_release(Some(Path::new(previous))),
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

/// 建資料包；`verify` 時另跑 `--verify` 的檢查。
fn build(verify: bool) -> ExitCode {
    let root = match std::env::current_dir() {
        Ok(root) => root,
        Err(e) => {
            eprintln!("讀不到目前目錄：{e}");
            return ExitCode::FAILURE;
        }
    };
    let Some(cache) = default_cache_dir() else {
        eprintln!("找不到使用者快取目錄：設定環境變數 {CACHE_ENV} 指定下載快取的位置");
        return ExitCode::FAILURE;
    };
    match qim_databuild::run(&root, &cache, &Http, verify) {
        Ok(outcome) => {
            let shown = |path: &std::path::Path| {
                path.strip_prefix(&root)
                    .unwrap_or(path)
                    .display()
                    .to_string()
            };
            for pack in &outcome.packs {
                println!("{}資料包：{}", pack.name, shown(&pack.path));
                println!("{} sha256：{}", pack.name, pack.sha256);
            }
            println!("佈局：{}", shown(&outcome.layout_path));
            // 個人集只印數字（plan §5.2）。
            match outcome.personal {
                Some(c) => println!(
                    "個人集：{} 轉出 {} 題（略過 c < 1 的 {} 列、轉不了碼的 {} 列，合併重複 {} 列）",
                    personal::ITEMS,
                    c.words,
                    c.unused,
                    c.untranscodable,
                    c.merged
                ),
                None => println!("個人集：沒有 {}，跳過", personal::USERDB),
            }
            match outcome.report {
                Some(report) => {
                    println!("{report}");
                    if report.passed() {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::FAILURE
                    }
                }
                None => {
                    println!("{}", outcome.lexicon);
                    println!("{}", outcome.words_hk);
                    println!("{}", outcome.language_model);
                    println!(
                        "{}",
                        qim_databuild::verify::english_summary(&outcome.english)
                    );
                    println!("{}", qim_databuild::verify::emoji_summary(&outcome.emoji));
                    for corpus in &outcome.corpora {
                        println!("{corpus}");
                    }
                    ExitCode::SUCCESS
                }
            }
        }
        Err(e) => {
            eprintln!("建置失敗：{e}");
            ExitCode::FAILURE
        }
    }
}

/// 把 `build/packs/` 整理成 `build/pack-release/`；`previous` 是上一版 release 的 `index.json`。
fn pack_release(previous: Option<&Path>) -> ExitCode {
    let previous = match previous.map(|path| {
        std::fs::read_to_string(path)
            .map_err(|e| format!("讀不了 {}：{e}", path.display()))
            .and_then(|json| PackIndex::from_json(&json).map_err(|e| e.to_string()))
    }) {
        None => None,
        Some(Ok(index)) => Some(index),
        Some(Err(e)) => {
            eprintln!("上一版索引：{e}");
            return ExitCode::FAILURE;
        }
    };
    let out = Path::new(release::RELEASE_DIR);
    match release::write(Path::new(PACKS_DIR), out, previous.as_ref()) {
        Ok(index) => {
            for entry in &index.packs {
                let m = &entry.manifest;
                println!("{} {}（{}，{} 位元組）", m.id, m.version, m.kind, m.size);
            }
            match previous {
                Some(_) => println!("對上一版索引：內容改了的都加了版本"),
                None => println!("沒給 --previous，沒比對上一版索引"),
            }
            println!("release 目錄：{}", out.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
