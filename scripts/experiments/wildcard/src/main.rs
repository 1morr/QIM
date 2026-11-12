//! 萬用鍵實驗（M4 票 11）：dev 切分每句挑一個字，把它的一碼換成萬用鍵 `z`，量整句首選、修正次數
//! 與每鍵延遲；和原樣的碼、現在的行為（組字中 `z` 吃下不動作，等於少打那一碼）並列。
//!
//! 用法（repo 根目錄先跑 `cargo xtask data`）：`cargo run --release -- [repo 根目錄] [--latency-only]`；
//! `--latency-only` 只量每鍵延遲（延遲超過上限時在負載低時單獨重量用）。

use std::path::{Path, PathBuf};
use std::thread;

use qim_bench::{Item, Unit, learned_model, native_timing_with, parse_corpus, score_set};
use qim_core::Engine;
use qim_core::UserModel;

const WILDCARD: char = 'z';

/// 一題改成哪種：原樣、換成萬用鍵、刪掉那一碼（現在的行為）。
#[derive(Clone, Copy, PartialEq, Eq)]
enum Change {
    Original,
    Wildcard,
    Dropped,
}

/// 換哪一碼：兩碼字的首碼、尾碼，或一碼字的那一碼。
#[derive(Clone, Copy, PartialEq, Eq)]
enum Position {
    First,
    Last,
    Single,
}

impl Position {
    fn label(self) -> &'static str {
        match self {
            Self::First => "首碼",
            Self::Last => "尾碼",
            Self::Single => "一碼字",
        }
    }
}

/// FNV-1a：同一份資料每次挑同一個字，不靠亂數。
fn hash(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// 句子 `item` 依 `position` 挑一個字（碼長合的字裡由雜湊挑），回傳它在 `units` 的位置。
fn pick(item: &Item, position: Position) -> Option<usize> {
    let wanted = if position == Position::Single { 1 } else { 2 };
    let fits: Vec<usize> = (0..item.units.len())
        .filter(|&i| item.units[i].code.len() == wanted)
        .collect();
    let seed = hash(&format!("{}\t{}", item.line, item.expected()));
    (!fits.is_empty()).then(|| fits[(seed % fits.len() as u64) as usize])
}

/// 把第 `at` 個字的那一碼照 `change` 改掉。
fn changed(item: &Item, at: usize, position: Position, change: Change) -> Item {
    let mut item = item.clone();
    let code = &item.units[at].code;
    let index = if position == Position::Last { 1 } else { 0 };
    let replaced: String = code
        .chars()
        .enumerate()
        .filter_map(|(i, c)| match (i == index, change) {
            (true, Change::Wildcard) => Some(WILDCARD),
            (true, Change::Dropped) => None,
            _ => Some(c),
        })
        .collect();
    item.units[at] = Unit {
        code: replaced,
        text: item.units[at].text.clone(),
    };
    item
}

/// 一組題的結果列。
struct Row {
    set: String,
    variant: String,
    total: usize,
    correct: usize,
    corrections: f64,
    unfinished: usize,
}

fn score(engine: &Engine, set: &str, variant: &str, items: &[Item]) -> Row {
    let result = score_set(engine, set, items, false);
    let metrics = result.metrics(false);
    Row {
        set: set.to_string(),
        variant: variant.to_string(),
        total: result.total(),
        correct: result.correct(),
        corrections: metrics.correction_count,
        unfinished: result.items.iter().filter(|i| !i.finished).count(),
    }
}

/// dev 切分的集合 `name` 的句子與由它取出的單字（兩碼的字，不重複）。
fn load(root: &Path, name: &str) -> (Vec<Item>, Vec<Item>) {
    let path = root.join(qim_bench::dev_items(name));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("讀不到 {}：{e}；先跑 cargo xtask data", path.display()));
    let sentences = parse_corpus(&text).unwrap();
    let mut seen = std::collections::BTreeSet::new();
    let singles = sentences
        .iter()
        .flat_map(|s| s.units.iter())
        .filter(|u| u.code.len() == 2 && seen.insert(u.text.clone()))
        .enumerate()
        .map(|(i, u)| Item {
            units: vec![u.clone()],
            line: i + 1,
            label: (i + 1).to_string(),
            weight: 1,
            context: None,
        })
        .collect();
    (sentences, singles)
}

/// 一個集合的各組題：（組名, 題目）。
fn variants(items: &[Item]) -> Vec<(String, Vec<Item>)> {
    let mut out = Vec::new();
    for position in [Position::First, Position::Last, Position::Single] {
        let picked: Vec<(&Item, usize)> = items
            .iter()
            .filter_map(|item| Some((item, pick(item, position)?)))
            .collect();
        for (change, label) in [
            (Change::Original, "原樣"),
            (Change::Wildcard, "換成 z"),
            (Change::Dropped, "z 吃掉（現在）"),
        ] {
            let set = picked
                .iter()
                .map(|&(item, at)| match change {
                    Change::Original => item.clone(),
                    _ => changed(item, at, position, change),
                })
                .collect();
            out.push((format!("{} {label}", position.label()), set));
        }
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let latency_only = args.iter().any(|a| a == "--latency-only");
    let root = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .map_or_else(|| PathBuf::from("../../.."), PathBuf::from);
    let loaded = qim_bench::load_engine(&root, &[], None).unwrap();
    let engine = &loaded.engine;
    assert_eq!(
        engine.scheme().wildcard,
        Some(WILDCARD),
        "速成資料包沒有萬用鍵：先跑 cargo xtask data"
    );
    let names = ["yue-wiki", "zh-wiki", "hkcancor", "common-voice-yue"];
    let mut jobs: Vec<(String, String, Vec<Item>)> = Vec::new();
    let mut wildcard_inputs = Vec::new();
    for name in names {
        let (sentences, singles) = load(&root, name);
        for (set, items) in [
            (name.to_string(), sentences),
            (format!("{name}·單字"), singles),
        ] {
            for (variant, changed) in variants(&items) {
                if variant.ends_with("換成 z") && !set.ends_with("·單字") {
                    wildcard_inputs.extend(changed.iter().map(Item::input));
                }
                jobs.push((set.clone(), variant, changed));
            }
        }
    }
    if latency_only {
        jobs.clear();
    }
    let rows: Vec<Row> = thread::scope(|scope| {
        let handles: Vec<_> = jobs
            .iter()
            .map(|(set, variant, items)| scope.spawn(move || score(engine, set, variant, items)))
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    if !latency_only {
        print_rows(&rows);
    }
    print_latency(engine, &root, &wildcard_inputs);
}

fn print_rows(rows: &[Row]) {
    println!("| 集合 | 組 | 題數 | 整句首選正確 | 修正/句 | 打不出 |");
    println!("| --- | --- | ---: | ---: | ---: | ---: |");
    for row in rows {
        println!(
            "| {} | {} | {} | {} ({:.1}%) | {:.3} | {} |",
            row.set,
            row.variant,
            row.total,
            row.correct,
            100.0 * row.correct as f64 / row.total.max(1) as f64,
            row.corrections,
            row.unfinished
        );
    }
}

/// 每鍵延遲：句子的「換成 z」各組（首碼、尾碼、一碼字）合起來，空白與學過的學習資料各一次；
/// 學過的照基準學完全部公開集合的句子（不含「有前文」的題，比基準少約 3 千句）。
fn print_latency(engine: &Engine, root: &Path, wildcard_inputs: &[String]) {
    let corpora = qim_bench::load_corpora(root).unwrap();
    let learned = learned_model(engine, corpora.iter().flat_map(|(_, items)| items));
    println!();
    println!(
        "每鍵延遲（原生 release，µs，p50 / p95），輸入 {} 個",
        wildcard_inputs.len()
    );
    println!();
    println!("| 學習資料 | 組 | 按鍵數 | p50 | p95 |");
    println!("| --- | --- | ---: | ---: | ---: |");
    for (label, user) in [("空白", UserModel::default()), ("學過", learned)] {
        let timing = native_timing_with(engine, &user, wildcard_inputs);
        for (group, summary) in timing.groups() {
            if let Some(s) = summary {
                println!(
                    "| {label} | {group} | {} | {:.1} | {:.1} |",
                    s.keys,
                    s.p50 * 1000.0,
                    s.p95 * 1000.0
                );
            }
        }
    }
}
