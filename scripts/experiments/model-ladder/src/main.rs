//! 模型階梯實驗的命令列（README〈怎麼重跑〉），在任何目錄執行都以 repo 根目錄為準。
//!
//! ```text
//! model-ladder eval [--split dev|test] [--n <N>] [--orders <結果.jsonl> --name <名>]
//! model-ladder export [--split dev|test] [--n <N>] [--include-personal]
//! model-ladder compare <逐題結果目錄 A> <逐題結果目錄 B>
//! ```

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use model_ladder::{
    DEFAULT_N, Identity, Orders, Reranker, Split, compare, export_set, language_model_cost,
    load_sets, report, run_set, write_rows,
};

const USAGE: &str = "用法：\n  model-ladder eval [--split dev|test] [--n <N>] [--orders <檔> --name <名>]\n  \
                     model-ladder export [--split dev|test] [--n <N>] [--include-personal]\n  \
                     model-ladder compare <目錄 A> <目錄 B>";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("eval") => eval(&args[1..]),
        Some("export") => export(&args[1..]),
        Some("compare") if args.len() == 3 => compare_dirs(&args[1], &args[2]),
        _ => Err(USAGE.into()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

/// 共用的旗標：`--split`、`--n`、`--orders`、`--name`、`--include-personal`。
#[derive(Default)]
struct Flags {
    split: Option<Split>,
    n: Option<usize>,
    orders: Option<String>,
    name: Option<String>,
    include_personal: bool,
}

fn flags(args: &[String]) -> Result<Flags, String> {
    let mut f = Flags::default();
    let mut rest = args.iter();
    while let Some(flag) = rest.next() {
        let mut value = || rest.next().cloned().ok_or_else(|| USAGE.to_string());
        match flag.as_str() {
            "--split" => {
                f.split = Some(match value()?.as_str() {
                    "dev" => Split::Dev,
                    "test" => Split::Test,
                    _ => return Err(USAGE.into()),
                })
            }
            "--n" => f.n = Some(value()?.parse().map_err(|_| USAGE.to_string())?),
            "--orders" => f.orders = Some(value()?),
            "--name" => f.name = Some(value()?),
            "--include-personal" => f.include_personal = true,
            _ => return Err(USAGE.into()),
        }
    }
    Ok(f)
}

fn split_of(f: &Flags) -> Split {
    let split = f.split.unwrap_or(Split::Dev);
    if split == Split::Test {
        eprintln!("注意：test 切分只在設定定案後跑一次報數，調參只用 dev（README〈調參與報數〉）");
    }
    split
}

fn eval(args: &[String]) -> Result<(), String> {
    let f = flags(args)?;
    if f.include_personal {
        return Err("--include-personal 只給 export".into());
    }
    if f.name.as_deref() == Some(Identity.name()) {
        return Err(format!("--name 不能叫 {}：那是基線的目錄", Identity.name()));
    }
    let split = split_of(&f);
    let n = f.n.unwrap_or(DEFAULT_N);
    let root = root();
    let (engine, _, _) = qim_bench::load_engine(&root, &[])?;
    let sets = load_sets(&root, split)?;
    let mut orders = match (&f.orders, &f.name) {
        (Some(path), Some(name)) => {
            let text = std::fs::read_to_string(path).map_err(|e| format!("讀不到 {path}：{e}"))?;
            Some(Orders::parse(name, &text, split).map_err(|e| format!("{path}：{e}"))?)
        }
        (None, None) => None,
        _ => return Err("--orders 與 --name 要一起給".into()),
    };
    let mut identity = Identity;
    let reranker: &mut dyn Reranker = match &mut orders {
        Some(o) => o,
        None => &mut identity,
    };
    let runs: Vec<_> = sets
        .iter()
        .map(|set| run_set(&engine, set, n, reranker))
        .collect();
    let name = reranker.name().to_string();
    println!("模型階梯：{} 切分\n", split.name());
    print!("{}", report(&runs, &name, n));
    if let Some(o) = &orders {
        println!("\n沒有外部結果、照原順序的題：{}", o.missing);
    }
    println!("\n{}", language_model_cost(&root, &engine, 5)?);
    let dir = root
        .join("build/model-ladder")
        .join(split.name())
        .join(&name);
    write_rows(&dir, &runs)?;
    eprintln!("逐題結果：{}", dir.display());
    Ok(())
}

fn export(args: &[String]) -> Result<(), String> {
    let f = flags(args)?;
    if f.orders.is_some() || f.name.is_some() {
        return Err("--orders、--name 只給 eval".into());
    }
    let split = split_of(&f);
    let n = f.n.unwrap_or(DEFAULT_N);
    let root = root();
    let (engine, _, _) = qim_bench::load_engine(&root, &[])?;
    let dir = root
        .join("build/model-ladder")
        .join(split.name())
        .join("requests");
    std::fs::create_dir_all(&dir).map_err(|e| format!("建不了 {}：{e}", dir.display()))?;
    for set in load_sets(&root, split)? {
        // 個人集的內容只在本機，也不送雲端（plan §5.2）：要明說才匯出。
        if set.private && !f.include_personal {
            continue;
        }
        let path = dir.join(format!("{}.jsonl", set.name));
        std::fs::write(&path, export_set(&engine, &set, n))
            .map_err(|e| format!("寫不了 {}：{e}", path.display()))?;
    }
    eprintln!("重排要求：{}", dir.display());
    Ok(())
}

fn compare_dirs(a: &str, b: &str) -> Result<(), String> {
    println!("| 集合 | 題數 | 首選 好/壞 | p | 前 2 名 好/壞 | p | 可動首選 好/壞 | p |");
    println!("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |");
    for (name, paired) in compare(Path::new(a), Path::new(b))? {
        let mut line = format!("| {name} | {} |", paired[0].total);
        for p in paired {
            line.push_str(&format!(" {}/{} | {:.3} |", p.better, p.worse, p.p_value()));
        }
        println!("{line}");
    }
    Ok(())
}
