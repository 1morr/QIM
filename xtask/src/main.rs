//! 開發指令與閘門；用法見 `docs/development.md`。

mod check;
mod commit_msg;
mod deps;
mod docs;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "用法：cargo xtask check [--fast] | cargo xtask commit-msg <訊息檔> | cargo xtask data [--verify]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["check"] => check(false),
        ["check", "--fast"] => check(true),
        ["data"] => data(&[]),
        ["data", "--verify"] => data(&["--verify"]),
        ["commit-msg", file] => {
            if commit_msg::report(Path::new(file)) {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn check(fast: bool) -> ExitCode {
    let root = repo_root();
    match check::run(&check::steps(fast), |step| check::exec(step, &root)) {
        Ok(()) => {
            eprintln!("check 通過");
            ExitCode::SUCCESS
        }
        Err(step) => {
            eprintln!("check 失敗：{step}");
            ExitCode::FAILURE
        }
    }
}

/// 建資料包（plan §4）；邏輯在 `qim-databuild`，這裡只轉交。
fn data(args: &[&str]) -> ExitCode {
    let status = std::process::Command::new(env!("CARGO"))
        .args(["run", "--quiet", "--release", "-p", "qim-databuild", "--"])
        .args(args)
        .current_dir(repo_root())
        .status();
    match status {
        Ok(s) if s.success() => ExitCode::SUCCESS,
        Ok(_) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("跑不起 qim-databuild：{e}");
            ExitCode::FAILURE
        }
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask 在 repo 根目錄下")
        .to_path_buf()
}
