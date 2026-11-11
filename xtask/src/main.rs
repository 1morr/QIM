//! 開發指令與閘門；用法見 `docs/development.md`。

mod check;
mod commit_msg;
mod deps;
mod docs;
mod ffi;
mod web;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "用法：cargo xtask check [--fast] | cargo xtask commit-msg <訊息檔> | cargo xtask data [--verify] | cargo xtask pack-release [--previous <index.json>] | cargo xtask web | cargo xtask ffi [--ios] | cargo xtask bench [--lexicon <id>] [--check | --update-baseline | --learning | --dev [--set <鍵>=<值>]...]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["check"] => check(false),
        ["check", "--fast"] => check(true),
        ["data"] => release_run("qim-databuild", &[]),
        ["data", "--verify"] => release_run("qim-databuild", &["--verify"]),
        ["pack-release"] => release_run("qim-databuild", &["--pack-release"]),
        ["pack-release", "--previous", previous] => {
            // qim-databuild 在 repo 根目錄跑：相對路徑先照目前目錄換成絕對路徑。
            let previous = std::path::absolute(previous).unwrap_or_else(|_| previous.into());
            release_run(
                "qim-databuild",
                &["--pack-release", "--previous", &previous.to_string_lossy()],
            )
        }
        ["bench"] => bench(&[]),
        ["bench", flag @ ("--check" | "--update-baseline")] => bench(&[flag]),
        // 學習模式不量延遲，不必重編 WebAssembly 綁定。
        ["bench", "--learning"] => release_run("qim-bench", &["--learning"]),
        // dev 模式只量正確率，不量每鍵延遲，不必重編 WebAssembly 綁定。
        ["bench", "--dev", ..] => release_run("qim-bench", &args[1..]),
        // 換掉方案指名的詞庫（M3 票 24）；qim-bench 自己擋不合的組合。
        ["bench", "--lexicon", _, "--dev", ..] => release_run("qim-bench", &args[1..]),
        ["bench", "--lexicon", _, ..] => bench(&args[1..]),
        ["web"] => {
            let root = repo_root();
            if web::wasm_bindings(&root) && web::serve_data_outputs(&root) {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        ["ffi"] => ffi(false),
        ["ffi", "--ios"] => ffi(true),
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

fn ffi(ios: bool) -> ExitCode {
    if ffi::swift_bindings(&repo_root(), ios) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
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

/// 跑子行程並等它結束；失敗時回傳說明（程式名與結束狀態）。
fn run_command(command: &mut std::process::Command) -> Result<(), String> {
    let program = command.get_program().to_string_lossy().into_owned();
    match command.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(format!("{program} 失敗：{status}")),
        Err(e) => Err(format!("無法執行 {program}：{e}")),
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
