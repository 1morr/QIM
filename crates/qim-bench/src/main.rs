//! `cargo xtask bench [--check | --update-baseline]` 轉進來的 CLI，在 repo 根目錄執行。

use std::path::Path;
use std::process::ExitCode;

use qim_bench::{Mode, run};

const USAGE: &str = "用法：qim-bench [--check | --update-baseline]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] => Mode::Report,
        ["--check"] => Mode::Check,
        ["--update-baseline"] => Mode::UpdateBaseline,
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    let outcome = match run(Path::new("."), mode) {
        Ok(outcome) => outcome,
        Err(error) => {
            eprintln!("{error}");
            eprintln!("bench 失敗");
            return ExitCode::FAILURE;
        }
    };
    println!("首選字基準\n");
    print!("{}", outcome.report());
    if mode == Mode::UpdateBaseline {
        println!("\n已改寫 {}", qim_bench::BASELINE);
    }
    for violation in &outcome.violations {
        eprintln!("{violation}");
    }
    if outcome.violations.is_empty() {
        eprintln!("bench 通過");
        ExitCode::SUCCESS
    } else {
        eprintln!("bench 失敗");
        ExitCode::FAILURE
    }
}
