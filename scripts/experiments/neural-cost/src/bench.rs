//! 一個模型檔的四項指標：檔案大小、載入時間、每次重排的延遲、常駐記憶體增量。

use std::fmt::Write as _;
use std::path::Path;
use std::time::Instant;

use candle_core::Result;

use crate::footprint::{footprint, heap_in_use};
use crate::model::{self, Scorer};

/// 重排前幾條路徑（M2 票 11 的近似）；環境變數 `NC_PATHS` 可蓋掉（M3 票 07 定的 N 重量用）。
pub const PATHS: usize = 6;
/// 每條路徑的字數：M3 一次重排的是整段輸入碼的候選句，取 8 字的短句；`NC_PATH_LEN` 可蓋掉。
pub const PATH_LEN: usize = 8;

/// `name` 的環境變數是正整數就用它，否則用 `default`（iOS 擴充沒有這些變數，照預設）。
fn env_or(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&n| n > 0)
        .unwrap_or(default)
}
/// 上下文字數（plan §7.8）。
pub const CONTEXTS: [usize; 3] = [0, 50, 200];
/// 每個上下文長度先跑幾次不計。
pub const WARMUP: usize = 3;

const MB: f64 = 1024.0 * 1024.0;

/// 上下文要不要讓 6 條路徑共用（README〈前綴快取〉）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prefix {
    /// 上下文只算一次（`Scorer::score`）。iPhone 量的是這個。
    Shared,
    /// 每條路徑各自從句首連上下文算起（`Scorer::score_unshared`），對照用。
    Unshared,
}

impl Prefix {
    fn score(
        self,
        scorer: &dyn Scorer,
        context: &[u32],
        candidates: &[Vec<u32>],
    ) -> Result<Vec<f32>> {
        match self {
            Prefix::Shared => scorer.score(context, candidates),
            Prefix::Unshared => scorer.score_unshared(context, candidates),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Prefix::Shared => "shared",
            Prefix::Unshared => "unshared",
        }
    }
}

/// 固定種子的假字 id，避開句首記號。
fn tokens(seed: u64, n: usize, vocab: usize) -> Vec<u32> {
    // Knuth 的 64 位元 LCG。
    let step = |x: u64| {
        x.wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407)
    };
    let mut x = step(seed);
    (0..n)
        .map(|_| {
            x = step(x);
            1 + ((x >> 33) % (vocab as u64 - 1)) as u32
        })
        .collect()
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    // 最近秩法：p95 是第 ceil(0.95 n) 小的值。
    let rank = ((p * sorted.len() as f64).ceil() as usize).clamp(1, sorted.len());
    sorted[rank - 1]
}

fn ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

/// 載入、先跑一次、再對每個上下文長度跑 `iterations` 次；回幾行可讀的結果。
/// 推論只用 `threads` 條執行緒：gemm 在每條跑過矩陣乘法的 rayon 執行緒各留一塊 L2 大小的暫存，
/// 用全域 pool（每個核心一條）時暫存就跟著核心數漲（README〈觀察〉）。
pub fn run(path: &Path, iterations: usize, threads: usize, prefix: Prefix) -> Result<String> {
    // candle 的矩陣乘法照這個環境變數決定切幾份，逐元素運算的 pool 第一次用時也照它開。
    // SAFETY: 在任何 candle 運算之前設；這個程式庫裡沒有別的執行緒同時讀寫環境變數。
    unsafe { std::env::set_var("RAYON_NUM_THREADS", threads.to_string()) };
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .map_err(|e| candle_core::Error::Msg(e.to_string()))?;
    pool.install(|| measure(path, iterations, prefix))
}

fn measure(path: &Path, iterations: usize, prefix: Prefix) -> Result<String> {
    let mut out = String::new();
    let file_bytes = std::fs::metadata(path)?.len();
    let start_fp = footprint();
    let before = start_fp.now;
    let heap_before = heap_in_use();

    let start = Instant::now();
    let loaded = model::load(path)?;
    let load_ms = ms(start);
    let after_load = footprint().now;

    let vocab = loaded.kind.vocab();
    let (paths, path_len) = (env_or("NC_PATHS", PATHS), env_or("NC_PATH_LEN", PATH_LEN));
    let candidates: Vec<Vec<u32>> = (0..paths)
        .map(|i| tokens(100 + i as u64, path_len, vocab))
        .collect();
    let start = Instant::now();
    prefix.score(loaded.scorer.as_ref(), &[], &candidates)?;
    let first_ms = ms(start);

    let _ = writeln!(
        out,
        "{} {} params={:.2}M file={:.2}MB threads={} prefix={} paths={paths}x{path_len}",
        loaded.kind.stem(),
        loaded.format.label(),
        loaded.params as f64 / 1e6,
        file_bytes as f64 / MB,
        candle_core::utils::get_num_threads(),
        prefix.label(),
    );
    let _ = writeln!(out, "load={load_ms:.1}ms first(ctx=0)={first_ms:.1}ms");

    for ctx_len in CONTEXTS {
        let context = tokens(ctx_len as u64, ctx_len, vocab);
        let mut times = Vec::with_capacity(iterations);
        for i in 0..WARMUP + iterations {
            let start = Instant::now();
            let scores = prefix.score(loaded.scorer.as_ref(), &context, &candidates)?;
            let t = ms(start);
            if scores.iter().any(|s| !s.is_finite()) {
                candle_core::bail!("non-finite scores: {scores:?}");
            }
            if i >= WARMUP {
                times.push(t);
            }
        }
        times.sort_by(f64::total_cmp);
        let _ = writeln!(
            out,
            "ctx={ctx_len} p50={:.1}ms p95={:.1}ms n={iterations} fp+{:.1}MB",
            percentile(&times, 0.5),
            percentile(&times, 0.95),
            (footprint().now as f64 - after_load as f64) / MB,
        );
    }
    let end_fp = footprint();
    let after = end_fp.now;
    let _ = writeln!(
        out,
        "footprint {:.2}MB load+{:.2}MB infer+{:.2}MB total+{:.2}MB heap+{:.2}MB",
        before as f64 / MB,
        (after_load as f64 - before as f64) / MB,
        (after as f64 - after_load as f64) / MB,
        (after as f64 - before as f64) / MB,
        (heap_in_use() as f64 - heap_before as f64) / MB,
    );
    // 峰值是整個程序的：peak+ 對開始時的 footprint 算，開始前的峰值比它高時會多算那段差（README〈指標〉）。
    let _ = writeln!(
        out,
        "peak before={:.2}MB after={:.2}MB peak+{:.2}MB",
        start_fp.peak as f64 / MB,
        end_fp.peak as f64 / MB,
        (end_fp.peak as f64 - before as f64) / MB,
    );
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_rank_percentiles() {
        let v: Vec<f64> = (1..=20).map(f64::from).collect();
        assert_eq!(percentile(&v, 0.5), 10.0);
        assert_eq!(percentile(&v, 0.95), 19.0);
        assert_eq!(percentile(&[3.0], 0.95), 3.0);
    }

    #[test]
    fn tokens_avoid_bos_and_stay_in_vocab() {
        let t = tokens(7, 1000, 50);
        assert!(t.iter().all(|&x| (1..50).contains(&x)));
        assert_eq!(t, tokens(7, 1000, 50));
    }
}
