//! 票 17b：四份語料的原文與 OpenCC 轉換後（預設 `s2hk`，`CONFIG=t2hk` 換設定）逐字比對，
//! 列出被換掉的字與次數。不經前處理的字形對照表（`common_forms`），看的是 OpenCC 本身。
//! 跑法見同目錄的 README.md。只讀快取，不下載。

use std::collections::HashMap;
use std::path::Path;

use ferrous_opencc::OpenCC;
use ferrous_opencc::config::BuiltinConfig;
use qim_databuild::corpora::CORPORA;
use qim_databuild::sources::{self, Fetch};
use qim_databuild::transcode::Transcoder;
use qim_databuild::{Error, libcangjie};

struct NoFetch;
impl Fetch for NoFetch {
    fn fetch(&self, url: &str, _: &mut dyn std::io::Write) -> Result<(), String> {
        Err(format!("快取沒有 {url}，先跑 cargo xtask data"))
    }
}

#[derive(Default)]
struct Pair {
    count: u64,
    samples: Vec<String>,
}

fn main() -> Result<(), Error> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let all = sources::parse(&std::fs::read_to_string(root.join(sources::SOURCES)).unwrap())?;
    let cache = sources::default_cache_dir().unwrap().join("sources");
    let table = all
        .source
        .get("libcangjie")
        .unwrap()
        .file("data/table.txt")?;
    let table = sources::load(&cache, "libcangjie", table, &NoFetch)?;
    let transcoder = Transcoder::new(&libcangjie::parse(&String::from_utf8(table).unwrap())?);
    let config = match std::env::var("CONFIG").as_deref() {
        Ok("t2hk") => BuiltinConfig::T2hk,
        _ => BuiltinConfig::S2hk,
    };
    let opencc = OpenCC::from_config(config).unwrap();
    let samples_wanted: usize = std::env::var("SAMPLES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);
    let only: Option<String> = std::env::var("ONLY").ok();

    for corpus in &CORPORA {
        let source = all.source.get(corpus.source_id).unwrap();
        let paths = source
            .files
            .iter()
            .map(|f| sources::cached(&cache, corpus.source_id, f, &NoFetch))
            .collect::<Result<Vec<_>, _>>()?;
        let mut pairs: HashMap<(char, char), Pair> = HashMap::new();
        let mut raw_count: HashMap<char, u64> = HashMap::new();
        let (mut chars, mut mismatched) = (0u64, 0u64);
        corpus.raw_texts(&paths, |raw| {
            let out = opencc.convert(raw);
            let a: Vec<char> = raw.chars().collect();
            let b: Vec<char> = out.chars().collect();
            for &c in &a {
                *raw_count.entry(c).or_default() += 1;
            }
            chars += a.len() as u64;
            if a.len() != b.len() {
                mismatched += 1;
                return Ok(());
            }
            for (i, (&x, &y)) in a.iter().zip(&b).enumerate() {
                if x == y {
                    continue;
                }
                let p = pairs.entry((x, y)).or_default();
                p.count += 1;
                if p.samples.len() < samples_wanted {
                    let lo = i.saturating_sub(6);
                    let hi = (i + 7).min(a.len());
                    p.samples.push(
                        a[lo..hi]
                            .iter()
                            .collect::<String>()
                            .replace(['\n', '\t'], " "),
                    );
                }
            }
            Ok(())
        })?;
        let mut sorted: Vec<_> = pairs.into_iter().collect();
        sorted.sort_by(|a, b| b.1.count.cmp(&a.1.count).then(a.0.cmp(&b.0)));
        let changed: u64 = sorted.iter().map(|p| p.1.count).sum();
        println!(
            "## {}：{} 字，換掉 {} 字（{} 種），長度不同略過 {} 段",
            corpus.name,
            chars,
            changed,
            sorted.len(),
            mismatched
        );
        println!("原字\t轉成\t次數\t原字在連打字集\t轉成的字在原文出現\t例");
        let limit: usize = std::env::var("TOP")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60);
        let mut shown = 0;
        for ((x, y), p) in &sorted {
            if let Some(o) = &only {
                if !o.contains(*x) {
                    continue;
                }
            } else if shown >= limit {
                break;
            }
            shown += 1;
            println!(
                "{x}\t{y}\t{}\t{}\t{}\t{}",
                p.count,
                if transcoder.code(*x).is_some() {
                    "是"
                } else {
                    "否"
                },
                raw_count.get(y).copied().unwrap_or(0),
                p.samples.join(" ｜ ")
            );
        }
        println!();
    }
    Ok(())
}
