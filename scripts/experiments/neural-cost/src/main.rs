//! Mac 上的命令列：`gen <目錄>` 寫出權重檔，`run <模型檔> [次數] [執行緒]` 量一個模型。

use std::path::Path;
use std::process::ExitCode;

fn parse(s: &str) -> candle_core::Result<usize> {
    s.parse()
        .map_err(|_| candle_core::Error::Msg(format!("not an integer: {s}")))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["gen", dir] => neural_cost::model::generate(Path::new(dir)).map(|files| {
            files
                .iter()
                .map(|f| format!("{}\n", f.display()))
                .collect::<String>()
        }),
        ["run", path] => neural_cost::bench::run(Path::new(path), 30, 2),
        ["run", path, n] => parse(n).and_then(|n| neural_cost::bench::run(Path::new(path), n, 2)),
        ["run", path, n, t] => parse(n).and_then(|n| {
            parse(t).and_then(|t| neural_cost::bench::run(Path::new(path), n, t.max(1)))
        }),
        _ => Err(candle_core::Error::Msg(
            "usage: neural-cost gen <dir> | run <model file> [iterations, default 30] [threads, default 2]".into(),
        )),
    };
    match result {
        Ok(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
