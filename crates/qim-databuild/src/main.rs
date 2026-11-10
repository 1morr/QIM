//! `qim-databuild [--verify]`：在 repo 根目錄建資料包；平常由 `cargo xtask data` 呼叫。

use std::process::ExitCode;

use qim_databuild::personal;
use qim_databuild::sources::{CACHE_ENV, Http, default_cache_dir};

const USAGE: &str = "用法：qim-databuild [--verify]（在 repo 根目錄執行）";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let verify = match args.as_slice() {
        [] => false,
        [flag] if flag == "--verify" => true,
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
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
