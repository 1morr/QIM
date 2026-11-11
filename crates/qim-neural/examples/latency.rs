//! 重排一次的延遲（M3 票 11d 的驗收：這台 Mac 上 9 條路徑 × 18 字、前文 50 字，p95 ≤ 40 ms）。
//!
//! `cargo run --release -p qim-neural --example latency [-- <次數>]`：讀 `data/neural-rerank/` 的
//! 模型，2 條執行緒（同模型階梯的量法），先跑 20 次熱身，再量 `Reranker::rank` 的 p50、p95、最大值。
//! 路徑與前文的字從字表前 3,000 字依固定的序列取，每次不同。

use std::path::Path;
use std::time::Instant;

use qim_neural::{CONTEXT_CHARS, Reranker};

const PATHS: usize = 9;
const PATH_CHARS: usize = 18;

fn main() {
    // 同 scripts/experiments/neural-cost 的量法：gemm 照這個環境變數切幾份。
    // SAFETY: 還沒有開別的執行緒，也還沒有跑任何 candle 運算。
    unsafe { std::env::set_var("RAYON_NUM_THREADS", "2") };
    let iterations: usize = std::env::args()
        .nth(1)
        .map(|s| s.parse().expect("次數要是正整數"))
        .unwrap_or(500);
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/neural-rerank");
    let read = |name: &str| std::fs::read(dir.join(name)).expect("先跑 cargo xtask neural-import");
    let weights = read("model.gguf");
    let vocab_text = String::from_utf8(read("vocab.txt")).unwrap();
    let vocab: Vec<char> = vocab_text
        .lines()
        .skip(2)
        .filter_map(|l| l.chars().next())
        .collect();
    let config = String::from_utf8(read("model.toml")).unwrap();
    let value = |key: &str| -> f64 {
        config
            .lines()
            .find_map(|l| {
                l.strip_prefix(key)?
                    .trim()
                    .strip_prefix('=')?
                    .trim()
                    .parse()
                    .ok()
            })
            .unwrap_or_else(|| panic!("model.toml 沒有 {key}"))
    };

    let start = Instant::now();
    let reranker = Reranker::new(
        &weights,
        &vocab,
        value("unknown_chars") as u32,
        value("lambda"),
    )
    .unwrap();
    let load_ms = start.elapsed().as_secs_f64() * 1000.0;

    // 線性同餘序列：固定、每次不同的字。
    let mut seed: u64 = 11;
    let mut next_char = || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        vocab[(seed >> 33) as usize % 3000.min(vocab.len())]
    };
    let mut cases = Vec::new();
    for _ in 0..iterations + 20 {
        let before: String = (0..CONTEXT_CHARS).map(|_| next_char()).collect();
        let paths: Vec<String> = (0..PATHS)
            .map(|_| (0..PATH_CHARS).map(|_| next_char()).collect())
            .collect();
        cases.push((before, paths));
    }

    let mut times = Vec::with_capacity(iterations);
    for (i, (before, paths)) in cases.iter().enumerate() {
        let scored: Vec<(&str, f64)> = paths
            .iter()
            .enumerate()
            .map(|(k, p)| (p.as_str(), -(k as f64)))
            .collect();
        let start = Instant::now();
        let order = reranker.rank(before, &scored).unwrap();
        let ms = start.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(order.len(), PATHS);
        if i >= 20 {
            times.push(ms);
        }
    }
    times.sort_by(f64::total_cmp);
    let at = |q: f64| times[((times.len() as f64 * q).ceil() as usize).saturating_sub(1)];
    println!(
        "載入 {load_ms:.1} ms；重排 {PATHS} 條 × {PATH_CHARS} 字、前文 {CONTEXT_CHARS} 字，{} 次：\
         p50 {:.2} ms、p95 {:.2} ms、最大 {:.2} ms（2 條執行緒）",
        times.len(),
        at(0.5),
        at(0.95),
        times[times.len() - 1]
    );
}
