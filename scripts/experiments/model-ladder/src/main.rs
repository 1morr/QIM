//! 模型階梯實驗的命令列（README〈怎麼重跑〉），在任何目錄執行都以 repo 根目錄為準。
//!
//! ```text
//! model-ladder eval [--split dev|test] [--n <N>] [--orders <結果.jsonl> --name <名>]
//! model-ladder export [--split dev|test] [--n <N>] [--include-personal]
//! model-ladder compare <逐題結果目錄 A> <逐題結果目錄 B>
//! model-ladder neural --model <權重> --vocab <字表> [--split dev|test] [--n <N>] [--lambdas <λ,…>] [--name <名>]
//! model-ladder quantize <f32 的 .safetensors> <q8 的 .gguf>
//! ```

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use model_ladder::{
    DEFAULT_N, Identity, Orders, Reranker, Split, compare, export_set, language_model_cost,
    load_sets, report, run_set, write_rows,
};

const USAGE: &str = "用法：\n  model-ladder eval [--split dev|test] [--n <N>] [--orders <檔> --name <名>]\n  \
                     model-ladder export [--split dev|test] [--n <N>] [--include-personal]\n  \
                     model-ladder compare <目錄 A> <目錄 B>\n  \
                     model-ladder neural --model <權重> --vocab <字表> [--split dev|test] [--n <N>] [--lambdas <λ,…>] [--name <名>]\n  \
                     model-ladder quantize <f32 的 .safetensors> <q8 的 .gguf>";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("eval") => eval(&args[1..]),
        Some("export") => export(&args[1..]),
        Some("compare") if args.len() == 3 => compare_dirs(&args[1], &args[2]),
        Some("neural") => neural(&args[1..]),
        Some("quantize") if args.len() == 3 => {
            model_ladder::neural::quantize(Path::new(&args[1]), Path::new(&args[2]))
                .map_err(|e| e.to_string())
        }
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

/// 神經重排（票 09）：每個 λ 打一次全部集合，印各組的前 2 名命中率；第一輪記神經分與計時，
/// 之後的 λ 重用神經分。只給一個 λ 時另印完整報告。逐題結果寫到 `<切分>/<名>-λ<λ>/`，給 `compare`。
fn neural(args: &[String]) -> Result<(), String> {
    use model_ladder::neural::{Neural, Vocab, lambda_label, load, parse_lambda};

    let mut model = None;
    let mut vocab = None;
    let mut lambdas = vec![Some(1.0)];
    let mut rest = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut value = || it.next().cloned().ok_or_else(|| USAGE.to_string());
        match a.as_str() {
            "--model" => model = Some(value()?),
            "--vocab" => vocab = Some(value()?),
            "--lambdas" => {
                lambdas = value()?
                    .split(',')
                    .map(parse_lambda)
                    .collect::<Result<_, _>>()?
            }
            _ => rest.push(a.clone()),
        }
    }
    let (Some(model), Some(vocab)) = (model, vocab) else {
        return Err(USAGE.into());
    };
    let f = flags(&rest)?;
    if f.orders.is_some() || f.include_personal {
        return Err("--orders、--include-personal 不給 neural".into());
    }
    let split = split_of(&f);
    let n = f.n.unwrap_or(DEFAULT_N);
    let model_path = Path::new(&model);
    let prefix = f.name.clone().unwrap_or_else(|| {
        model_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("neural")
            .to_string()
    });

    // 推論只用 2 條執行緒，同 neural-cost（README〈量法〉：gemm 每條執行緒留一塊 L2 大小的暫存）。
    // SAFETY: 還沒有開別的執行緒。
    unsafe { std::env::set_var("RAYON_NUM_THREADS", "2") };
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .map_err(|e| e.to_string())?;
    pool.install(|| {
        let root = root();
        let (engine, _, _) = qim_bench::load_engine(&root, &[])?;
        let sets = load_sets(&root, split)?;
        let start = std::time::Instant::now();
        let loaded = load(model_path).map_err(|e| e.to_string())?;
        let load_ms = start.elapsed().as_secs_f64() * 1000.0;
        let (kind, format, params) = (loaded.kind, loaded.format, loaded.params);
        let size = std::fs::metadata(model_path).map_err(|e| e.to_string())?.len();
        let vocab_size = loaded.scorer.vocab();
        check_reference(model_path, loaded.format, &*loaded.scorer)?;
        let mut reranker = Neural::new(prefix.clone(), loaded, Vocab::load(Path::new(&vocab), vocab_size)?);
        println!(
            "模型 {kind} {}：參數 {params}、檔案 {size} 位元組、字表 {vocab_size}、載入 {load_ms:.1} ms；{} 切分、N = {n}\n",
            format.label(),
            split.name()
        );
        let mut sweep: Vec<(String, Vec<(String, model_ladder::Summary)>)> = Vec::new();
        for (i, &lambda) in lambdas.iter().enumerate() {
            reranker.lambda = lambda;
            let name = format!("{prefix}-λ{}", lambda_label(lambda));
            reranker.set_name(name.clone());
            let runs: Vec<_> = sets
                .iter()
                .map(|set| {
                    let run = run_set(&engine, set, n, &mut reranker);
                    eprintln!("λ {}：{} 完成（{:.0} s）", lambda_label(lambda), set.name, start.elapsed().as_secs_f64());
                    run
                })
                .collect();
            if i == 0 {
                print!("{}", latency_table(&sets, &reranker));
            }
            if lambdas.len() == 1 {
                print!("{}", report(&runs, &name, n));
            }
            sweep.push((lambda_label(lambda), model_ladder::group_totals(&runs)));
            let dir = root.join("build/model-ladder").join(split.name()).join(&name);
            write_rows(&dir, &runs)?;
            eprintln!("逐題結果：{}", dir.display());
        }
        if lambdas.len() > 1 {
            print!("{}", sweep_table(&sweep));
        }
        Ok(())
    })
}

/// 各 λ 的前 2 名命中率（「可動首選」在括號裡），一組一欄。
fn sweep_table(sweep: &[(String, Vec<(String, model_ladder::Summary)>)]) -> String {
    let pct = |a: usize, b: usize| {
        if b == 0 {
            "—".to_string()
        } else {
            format!("{:.1}%", 100.0 * a as f64 / b as f64)
        }
    };
    let groups: Vec<&str> = sweep[0].1.iter().map(|(g, _)| g.as_str()).collect();
    let mut out = format!(
        "\n前 2 名命中率（括號是可動首選）\n\n| λ | {} |\n|{}\n",
        groups.join(" | "),
        " ---: |".repeat(groups.len() + 1)
    );
    for (label, totals) in sweep {
        let cells: Vec<String> = totals
            .iter()
            .map(|(_, s)| format!("{} ({})", pct(s.top2, s.total), pct(s.model_first, s.total)))
            .collect();
        out.push_str(&format!("| {label} | {} |\n", cells.join(" | ")));
    }
    out
}

/// 神經打分的延遲：「有前文」各前文長度（只算神經模型那段，不含打字與排序）與一般集合。
fn latency_table(sets: &[model_ladder::Set], reranker: &model_ladder::neural::Neural) -> String {
    use model_ladder::percentile;
    let mut rows = String::from(
        "神經打分延遲（ms，2 條執行緒）\n\n| 集合 | 次數 | 前文 token p50 | p50 | p95 | 最大 |\n| --- | ---: | ---: | ---: | ---: | ---: |\n",
    );
    let mut groups: Vec<(String, Vec<&model_ladder::Set>)> = Vec::new();
    for set in sets {
        let label = if set.group.is_context() {
            format!("有前文·前文{}", set.context_chars)
        } else {
            set.group.label().to_string()
        };
        match groups.iter_mut().find(|(l, _)| *l == label) {
            Some((_, v)) => v.push(set),
            None => groups.push((label, vec![set])),
        }
    }
    for (label, members) in groups {
        let scored: Vec<_> = reranker
            .scored
            .iter()
            .filter(|((name, _), _)| members.iter().any(|s| &s.name == name))
            .map(|(_, s)| s)
            .collect();
        let ms: Vec<f64> = scored.iter().map(|s| s.ms).collect();
        let ctx: Vec<usize> = scored.iter().map(|s| s.context_tokens).collect();
        let f = |v: Option<f64>| v.map_or("—".into(), |x| format!("{x:.1}"));
        rows.push_str(&format!(
            "| {label} | {} | {} | {} | {} | {} |\n",
            ms.len(),
            percentile(&ctx, 50).map_or("—".into(), |x| x.to_string()),
            f(percentile(&ms, 50)),
            f(percentile(&ms, 95)),
            f(ms.iter().copied().reduce(f64::max)),
        ));
    }
    rows.push('\n');
    rows
}

/// `train.py` 在權重旁寫的 `reference.json`（PyTorch 算的分數）：candle 算的要對得上，否則兩邊的
/// 結構不一致（GELU 近似、閘的順序……）。q8 只印差多少。
fn check_reference(
    model: &Path,
    format: model_ladder::neural::Format,
    scorer: &dyn model_ladder::neural::Scorer,
) -> Result<(), String> {
    #[derive(serde::Deserialize)]
    struct Reference {
        context: Vec<u32>,
        path: Vec<u32>,
        score: f32,
    }
    let path = model.with_file_name("reference.json");
    let Ok(text) = std::fs::read_to_string(&path) else {
        eprintln!("沒有 {}，不核對 PyTorch 的分數", path.display());
        return Ok(());
    };
    let refs: Vec<Reference> =
        serde_json::from_str(&text).map_err(|e| format!("{}：{e}", path.display()))?;
    let mut worst = 0f32;
    for r in &refs {
        let got = scorer
            .score_one(&r.context, &r.path)
            .map_err(|e| e.to_string())?;
        worst = worst.max((got - r.score).abs());
    }
    eprintln!(
        "和 PyTorch 的分數最多差 {worst:.4} nat（{} 組）",
        refs.len()
    );
    if format == model_ladder::neural::Format::F32 && worst > 1e-2 {
        return Err(format!(
            "candle 與 PyTorch 的分數差 {worst}：模型結構對不上"
        ));
    }
    Ok(())
}
