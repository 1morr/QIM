//! 開發指令與閘門；用法見 `docs/development.md`。

mod check;
mod commit_msg;
mod deps;
mod docs;
mod web;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "用法：cargo xtask check [--fast] | cargo xtask commit-msg <訊息檔> | cargo xtask data [--verify] | cargo xtask web | cargo xtask bench [--check | --update-baseline | --learning]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["check"] => check(false),
        ["check", "--fast"] => check(true),
        ["data"] => release_run("qim-databuild", &[]),
        ["data", "--verify"] => release_run("qim-databuild", &["--verify"]),
        ["bench"] => bench(&[]),
        ["bench", flag @ ("--check" | "--update-baseline")] => bench(&[flag]),
        // 學習模式不量延遲，不必重編 WebAssembly 綁定。
        ["bench", "--learning"] => release_run("qim-bench", &["--learning"]),
        ["web"] => {
            let root = repo_root();
            if web::wasm_bindings(&root) && web::serve_data_outputs(&root) {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
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

/// 首選字基準（plan §5.2）：WebAssembly 的每鍵延遲用 `web/src/wasm/` 的綁定量，先照目前的核心重編。
fn bench(args: &[&str]) -> ExitCode {
    if !web::wasm_bindings(&repo_root()) {
        return ExitCode::FAILURE;
    }
    release_run("qim-bench", args)
}

/// 在 repo 根目錄用 release 跑 workspace 的 `package`：資料建置（plan §4）在 `qim-databuild`、
/// 首選字基準（plan §5.2）在 `qim-bench`，這裡只轉交。
fn release_run(package: &str, args: &[&str]) -> ExitCode {
    let status = std::process::Command::new(cargo())
        .args(["run", "--quiet", "--release", "-p", package, "--"])
        .args(args)
        .current_dir(repo_root())
        .status();
    match status {
        Ok(s) if s.success() => ExitCode::SUCCESS,
        Ok(_) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("跑不起 {package}：{e}");
            ExitCode::FAILURE
        }
    }
}

/// 跑 xtask 的那個 cargo；`cargo xtask` 經 alias 執行時 cargo 會設 `CARGO`。
fn cargo() -> std::ffi::OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into())
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask 在 repo 根目錄下")
        .to_path_buf()
}
