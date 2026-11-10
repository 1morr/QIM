//! Mac 上的命令列：`gen <目錄>` 寫出權重檔，`run <模型檔> [次數] [執行緒] [unshared]` 量一個模型。

use std::path::Path;

use neural_cost::bench::Prefix;
use std::process::ExitCode;

fn parse(s: &str) -> candle_core::Result<usize> {
    s.parse()
        .map_err(|_| candle_core::Error::Msg(format!("not an integer: {s}")))
}

fn run(path: &str, n: &str, t: &str, prefix: Prefix) -> candle_core::Result<String> {
    neural_cost::bench::run(Path::new(path), parse(n)?, parse(t)?.max(1), prefix)
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
        ["run", path] => run(path, "30", "2", Prefix::Shared),
        ["run", path, n] => run(path, n, "2", Prefix::Shared),
        ["run", path, n, t] => run(path, n, t, Prefix::Shared),
        ["run", path, n, t, "unshared"] => run(path, n, t, Prefix::Unshared),
        _ => Err(candle_core::Error::Msg(
            "usage: neural-cost gen <dir> | run <model file> [iterations, default 30] [threads, default 2] [unshared]".into(),
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
