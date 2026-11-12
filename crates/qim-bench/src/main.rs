//! `cargo xtask bench [--lexicon <id>] [--check | --update-baseline | --learning | --dev [--set <鍵>=<值>]...]`
//! 轉進來的 CLI，在 repo 根目錄執行。

use std::path::Path;
use std::process::ExitCode;

use qim_bench::{Mode, Timers, native_timing_with, node_wasm_timer, run, run_dev};

const USAGE: &str = "用法：qim-bench [--lexicon <id>] [--check | --update-baseline | --learning | --dev [--set <鍵>=<值>]...]";

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    // `--lexicon <id>`：換掉方案指名的詞庫（可選的粵典詞庫，M3 票 24），放在最前面。
    let lexicon = match args.first().map(String::as_str) {
        Some("--lexicon") if args.len() >= 2 => {
            let id = args[1].clone();
            args.drain(..2);
            Some(id)
        }
        Some("--lexicon") => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
        _ => None,
    };
    let lexicon = lexicon.as_deref();
    if args.first().map(String::as_str) == Some("--dev") {
        return dev(&args[1..], lexicon);
    }
    let mode = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] => Mode::Report,
        ["--check"] => Mode::Check,
        ["--update-baseline"] => Mode::UpdateBaseline,
        ["--learning"] if lexicon.is_none() => return learning(),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    let root = Path::new(".");
    let timers = Timers {
        native: &native_timing_with,
        wasm: &node_wasm_timer(root),
    };
    let outcome = match run(root, mode, lexicon, timers) {
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
        if matches!(outcome.personal, qim_bench::Personal::Ran(_)) {
            println!("已改寫 {}", qim_bench::PERSONAL_BASELINE);
        }
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

/// 學習模式（plan §5.2）：只報不擋。
fn learning() -> ExitCode {
    match qim_bench::run_learning(Path::new(".")) {
        Ok(learning) => {
            println!("首選字基準：學習模式\n");
            print!("{}", learning.report());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            eprintln!("bench 失敗");
            ExitCode::FAILURE
        }
    }
}

/// dev 模式：`--set <鍵>=<值>` 可重複，暫時蓋掉方案檔 `[ranking]` 的值（調參用，plan §2.3）。
fn dev(args: &[String], lexicon: Option<&str>) -> ExitCode {
    let mut overrides = Vec::new();
    let mut rest = args.iter();
    while let Some(flag) = rest.next() {
        match (flag.as_str(), rest.next()) {
            ("--set", Some(value)) => overrides.push(value.clone()),
            _ => {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            }
        }
    }
    match run_dev(Path::new("."), &overrides, lexicon) {
        Ok(outcome) => {
            println!("首選字基準（dev）\n");
            print!("{}", outcome.report());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            eprintln!("bench 失敗");
            ExitCode::FAILURE
        }
    }
}
