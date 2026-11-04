//! `qim-databuild [--verify]`：在 repo 根目錄建資料包；平常由 `cargo xtask data` 呼叫。

use std::process::ExitCode;

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
            let shown = outcome
                .pack_path
                .strip_prefix(&root)
                .unwrap_or(&outcome.pack_path);
            println!("資料包：{}", shown.display());
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
                    println!("sha256：{}", outcome.sha256);
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
