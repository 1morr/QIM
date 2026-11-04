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
use sha2::{Digest, Sha256};

pub use baseline::{Baseline, Flips, SetBaseline, parse as parse_baseline};
pub use items::{Item, Unit, parse_corpus, parse_fixed, parse_personal};
pub use score::{Failure, GroupStats, ItemResult, LENGTH_GROUPS, Metrics, SetResult};

/// 基準用的資料包，由 `cargo xtask data` 產生。
pub const PACK: &str = "build/packs/quick/quick.pack";
pub const FIXED: &str = "bench/fixed.tsv";
/// 語料目錄，依序跑：每個 `<名>.tsv` 是一個集合，集合名在所有目錄間不能重複。
/// 維基防退步，口語是品質判斷的主要依據（brief §9）；目錄不存在就當沒有集合。
pub const CORPUS_DIRS: [&str; 2] = ["bench/corpus", "bench/colloquial"];
pub const BASELINE: &str = "bench/baseline.json";
/// 個人集的題目，由 `cargo xtask data` 從 [`PERSONAL_USERDB`] 轉出；只在本機（plan §5.2）。
pub const PERSONAL: &str = "bench/personal/rime-userdb.tsv";
/// 使用者放的 Rime 學習資料；基準不直接讀它，只在題目還沒轉出時提示。
pub const PERSONAL_USERDB: &str = "bench/personal/rime-userdb.txt";
pub const PERSONAL_BASELINE: &str = "bench/personal/baseline.json";
/// 個人集在結果與 baseline 裡的集合名。
pub const PERSONAL_SET: &str = "personal";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// 只印結果。
    Report,
    /// 固定題全過、各集合答錯數與修正次數不高於 baseline、每題都打得出，否則有違規。
    Check,
    /// 用這次的結果改寫 baseline；有個人集時也改寫 [`PERSONAL_BASELINE`]。
    UpdateBaseline,
}

#[derive(Debug)]
pub struct Outcome {
    /// 資料包 sha256 的十六進位；同一份資料包前後對照時一樣，沒重建就看得出來。
    pub pack_sha256: String,
    pub fixed: SetResult,
    /// 各語料集合，依 [`CORPUS_DIRS`] 的順序，同一目錄內依名稱排序。
    pub sets: Vec<SetResult>,
    pub personal: Personal,
    /// 跑之前的 baseline，報告裡對照用；沒有檔案時是 `None`。
    pub baseline: Option<Baseline>,
    /// 跑之前的個人集 baseline；沒有檔案時是 `None`。
    pub personal_baseline: Option<Baseline>,
    /// 擋退步的違規；空的就是通過。
    pub violations: Vec<String>,
}

impl Outcome {
    /// 給人看的結果，Markdown 表。
    pub fn report(&self) -> String {
        report::render(self)
    }

    /// 集合 `name` 跑之前的 baseline。
    pub fn baseline_of(&self, name: &str) -> Option<&SetBaseline> {
        let file = if name == PERSONAL_SET {
            &self.personal_baseline
        } else {
            &self.baseline
        };
        file.as_ref()?.get(name)
    }

    /// 集合 `name` 和 baseline 比的翻盤清單；沒有這個集合或沒有 baseline 時是 `None`。
    pub fn flips(&self, name: &str) -> Option<Flips> {
        let set = self.scored_sets().find(|s| s.name == name)?;
        Some(baseline::flips(set, self.baseline_of(name)?))
    }

    /// 語料集合，有個人集時接在最後。
    pub fn scored_sets(&self) -> impl Iterator<Item = &SetResult> {
        self.sets.iter().chain(match &self.personal {
            Personal::Ran(set) => Some(set),
            _ => None,
        })
    }
}

/// 個人集這次的狀況。
#[derive(Debug)]
pub enum Personal {
    Ran(SetResult),
    /// 沒有 [`PERSONAL`] 也沒有 [`PERSONAL_USERDB`]：跳過。
    Missing,
    /// 有 [`PERSONAL_USERDB`] 但還沒轉出 [`PERSONAL`]：跳過，提示先跑 `cargo xtask data`。
    NotTranscoded,
}

/// 在 repo 根目錄 `root` 跑基準。
pub fn run(root: &Path, mode: Mode) -> Result<Outcome, String> {
    let (engine, pack_sha256) = load_engine(&root.join(PACK))?;
    let fixed = parse_fixed(&read(&root.join(FIXED))?).map_err(|e| format!("{FIXED}：{e}"))?;
    let fixed = score::score(&engine, "fixed", &fixed);
    let mut sets: Vec<SetResult> = Vec::new();
    for dir in CORPUS_DIRS {
        for (name, path) in corpus_files(&root.join(dir))? {
            if name == PERSONAL_SET {
                return Err(format!(
                    "語料不能叫 {PERSONAL_SET}，那是個人集的名字：{}",
                    path.display()
                ));
            }
            if sets.iter().any(|s| s.name == name) {
                return Err(format!(
                    "集合 {name} 在兩個語料目錄都有：{}",
                    path.display()
                ));
            }
            let items =
                parse_corpus(&read(&path)?).map_err(|e| format!("{}：{e}", path.display()))?;
            sets.push(score::score(&engine, &name, &items));
        }
    }
    let personal = if root.join(PERSONAL).exists() {
        let items =
            parse_personal(&read(&root.join(PERSONAL))?).map_err(|e| format!("{PERSONAL}：{e}"))?;
        Personal::Ran(score::score(&engine, PERSONAL_SET, &items))
    } else if root.join(PERSONAL_USERDB).exists() {
        Personal::NotTranscoded
    } else {
        Personal::Missing
    };
    // 改寫 baseline 時舊檔讀不了（例如格式改了）不算錯，當作沒有。
    let load = |path: &str| match (mode, load_baseline(&root.join(path))) {
        (Mode::UpdateBaseline, Err(_)) => Ok(None),
        (_, loaded) => loaded,
    };
    let baseline = load(BASELINE)?;
    let personal_baseline = load(PERSONAL_BASELINE)?;
    let sets_only: Vec<&SetResult> = sets.iter().collect();
    let violations = match mode {
        Mode::Report => Vec::new(),
        Mode::Check => {
            let limits = baseline.as_ref().ok_or_else(|| {
                format!("沒有 {BASELINE}：先跑 `cargo xtask bench --update-baseline`")
            })?;
            let mut violations: Vec<String> = fixed
                .failures
                .iter()
                .map(|failure| format!("固定題沒過：{failure}"))
                .collect();
            violations.extend(baseline::check(&sets_only, limits, BASELINE));
            // 沒有個人集的檔時跳過整個個人集，連它的 baseline 也不比。
            if let Personal::Ran(set) = &personal {
                let empty = Baseline::new();
                let limits = personal_baseline.as_ref().unwrap_or(&empty);
                violations.extend(baseline::check(&[set], limits, PERSONAL_BASELINE));
            }
            violations
        }
        Mode::UpdateBaseline => {
            write(&root.join(BASELINE), &baseline::render(&sets_only))?;
            if let Personal::Ran(set) = &personal {
                write(&root.join(PERSONAL_BASELINE), &baseline::render(&[set]))?;
            }
            Vec::new()
        }
    };
    Ok(Outcome {
        pack_sha256,
        fixed,
        sets,
        personal,
        baseline,
        personal_baseline,
        violations,
    })
}

fn write(path: &Path, contents: &str) -> Result<(), String> {
    fs::write(path, contents).map_err(|e| format!("寫不了 {}：{e}", path.display()))
}

fn load_baseline(path: &Path) -> Result<Option<Baseline>, String> {
    if !path.exists() {
        return Ok(None);
    }
    baseline::parse(&read(path)?)
        .map(Some)
        .map_err(|e| format!("{}：{e}", path.display()))
}

/// 資料包與它的 sha256。
fn load_engine(path: &Path) -> Result<(Engine, String), String> {
    let bytes = fs::read(path).map_err(|e| {
        format!(
            "讀不到資料包 {}：{e}；先跑 `cargo xtask data`",
            path.display()
        )
    })?;
    let engine = Engine::from_pack_bytes(&bytes).map_err(|e| format!("{}：{e}", path.display()))?;
    let sha256 = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    Ok((engine, sha256))
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("讀不到 {}：{e}", path.display()))
}

/// 語料目錄下的 `<名>.tsv`，依名稱排序；目錄不存在時是空的。
fn corpus_files(dir: &Path) -> Result<Vec<(String, std::path::PathBuf)>, String> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("讀不到 {}：{e}", dir.display())),
    };
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
