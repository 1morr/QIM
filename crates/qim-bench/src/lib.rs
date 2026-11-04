//! 首選字基準（plan §5.2）：用核心實際打 `bench/` 的每一題，量首選正確率並擋退步。
//!
//! 只透過 `qim-core` 的公開介面打字；讀檔在這裡做，核心不碰 I/O（plan §1.3）。

mod baseline;
mod items;
mod report;
mod score;

use std::fs;
use std::path::Path;

use qim_core::Engine;

pub use baseline::{Baseline, SetBaseline, parse as parse_baseline};
pub use items::{Item, parse_corpus, parse_fixed};
pub use score::{Failure, SetResult};

/// 基準用的資料包，由 `cargo xtask data` 產生。
pub const PACK: &str = "build/packs/quick/quick.pack";
pub const FIXED: &str = "bench/fixed.tsv";
/// 語料目錄：每個 `<名>.tsv` 是一個集合。
pub const CORPUS: &str = "bench/corpus";
pub const BASELINE: &str = "bench/baseline.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// 只印結果。
    Report,
    /// 固定題全過且各集合答錯數不高於 baseline，否則有違規。
    Check,
    /// 用這次的答錯數改寫 baseline。
    UpdateBaseline,
}

#[derive(Debug)]
pub struct Outcome {
    pub fixed: SetResult,
    /// 各語料集合，依名稱排序。
    pub sets: Vec<SetResult>,
    /// 跑之前的 baseline，報告裡對照用；沒有檔案時是 `None`。
    pub baseline: Option<Baseline>,
    /// 擋退步的違規；空的就是通過。
    pub violations: Vec<String>,
}

impl Outcome {
    /// 給人看的結果，Markdown 表。
    pub fn report(&self) -> String {
        report::render(self)
    }
}

/// 在 repo 根目錄 `root` 跑基準。
pub fn run(root: &Path, mode: Mode) -> Result<Outcome, String> {
    let engine = load_engine(&root.join(PACK))?;
    let fixed = parse_fixed(&read(&root.join(FIXED))?).map_err(|e| format!("{FIXED}：{e}"))?;
    let fixed = score::score(&engine, "fixed", &fixed);
    let mut sets = Vec::new();
    for (name, path) in corpus_files(&root.join(CORPUS))? {
        let items = parse_corpus(&read(&path)?).map_err(|e| format!("{}：{e}", path.display()))?;
        sets.push(score::score(&engine, &name, &items));
    }
    let baseline = load_baseline(&root.join(BASELINE))?;
    let violations = match mode {
        Mode::Report => Vec::new(),
        Mode::Check => {
            let limits = baseline.as_ref().ok_or_else(|| {
                format!("沒有 {BASELINE}：先跑 `cargo xtask bench --update-baseline`")
            })?;
            baseline::check(&fixed, &sets, limits)
        }
        Mode::UpdateBaseline => {
            let path = root.join(BASELINE);
            fs::write(&path, baseline::render(&sets))
                .map_err(|e| format!("寫不了 {}：{e}", path.display()))?;
            Vec::new()
        }
    };
    Ok(Outcome {
        fixed,
        sets,
        baseline,
        violations,
    })
}

fn load_baseline(path: &Path) -> Result<Option<Baseline>, String> {
    if !path.exists() {
        return Ok(None);
    }
    baseline::parse(&read(path)?)
        .map(Some)
        .map_err(|e| format!("{}：{e}", path.display()))
}

fn load_engine(path: &Path) -> Result<Engine, String> {
    let bytes = fs::read(path).map_err(|e| {
        format!(
            "讀不到資料包 {}：{e}；先跑 `cargo xtask data`",
            path.display()
        )
    })?;
    Engine::from_pack_bytes(&bytes).map_err(|e| format!("{}：{e}", path.display()))
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("讀不到 {}：{e}", path.display()))
}

/// 語料目錄下的 `<名>.tsv`，依名稱排序。
fn corpus_files(dir: &Path) -> Result<Vec<(String, std::path::PathBuf)>, String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("讀不到 {}：{e}", dir.display()))?;
    let mut files = Vec::new();
    for entry in entries {
        let path = entry
            .map_err(|e| format!("讀不到 {}：{e}", dir.display()))?
            .path();
        if path.extension().is_some_and(|ext| ext == "tsv")
            && let Some(name) = path.file_stem().and_then(|s| s.to_str())
        {
            files.push((name.to_string(), path));
        }
    }
    files.sort();
    Ok(files)
}
