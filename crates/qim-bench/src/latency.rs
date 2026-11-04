//! 每鍵延遲（plan §5.2）：逐鍵打基準的每個輸入碼，量每次按鍵到 `Snapshot` 回來的時間，
//! 原生與 WebAssembly 各量一次，報 p50 / p95。只報不擋：機器負載會飄，`--check` 不比、baseline 不記。
//!
//! 每個輸入都從空的組字打起、不選字；載入資料包不算在內。

use std::hint::black_box;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

use qim_core::{Engine, Key, Session, UserModel};

/// 組字已有這麼多碼時的按鍵另成一組：連打變長時延遲會不會跟著長（票 12 的延遲預算要看它）。
/// 只算每個輸入的第 20 鍵起：前 19 鍵的組字和短輸入一樣短，算進來會稀釋。
pub const LONG_INPUT_KEYS: usize = 20;

/// WebAssembly 的計時腳本，用 Node 跑；它讀 [`WASM_BINDINGS`]，與網頁是同一份綁定。
pub const WASM_SCRIPT: &str = "web/bench/latency.ts";
/// `cargo xtask web`（`cargo xtask bench` 也會）產生的綁定。
pub const WASM_BINDINGS: &str = "web/src/wasm/qim_wasm.js";

/// 量一次的結果。
#[derive(Debug, Clone, PartialEq)]
pub struct Timing {
    /// 執行環境，印在報告的「環境」欄：原生是 build profile，WebAssembly 是執行它的 runtime 與版本。
    pub runtime: String,
    /// 每個輸入每一鍵的毫秒數，順序同輸入。
    pub per_key_ms: Vec<Vec<f64>>,
}

/// 在 WebAssembly 上量時間的方式：給資料包路徑（速成，有詞庫時接著詞庫）與輸入碼，逐鍵打每個輸入。
/// 實際用 [`node_wasm_timer`]；測試可以換成假的。
pub type WasmTimer<'a> = &'a dyn Fn(&[PathBuf], &[String]) -> Result<Timing, String>;

#[derive(Debug, Clone, PartialEq)]
pub struct Latency {
    /// CPU、作業系統與架構，報告註明在哪台機器量的。
    pub machine: String,
    pub native: Timing,
    pub wasm: Timing,
}

/// 一組輸入的統計，毫秒。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Summary {
    pub inputs: usize,
    pub keys: usize,
    pub p50: f64,
    pub p95: f64,
}

impl Timing {
    /// 「全部」與「第 [`LONG_INPUT_KEYS`] 碼起」兩組；組裡沒有按鍵時是 `None`。
    pub fn groups(&self) -> [(String, Option<Summary>); 2] {
        [
            ("全部".into(), summary(self, 0)),
            (
                format!("第 {LONG_INPUT_KEYS} 碼起"),
                summary(self, LONG_INPUT_KEYS - 1),
            ),
        ]
    }
}

/// 每個輸入跳過前 `skip` 鍵；輸入數只算還剩下按鍵的輸入。
fn summary(timing: &Timing, skip: usize) -> Option<Summary> {
    let picked: Vec<&[f64]> = timing
        .per_key_ms
        .iter()
        .filter_map(|ms| ms.get(skip..).filter(|rest| !rest.is_empty()))
        .collect();
    let values: Vec<f64> = picked.concat();
    Some(Summary {
        inputs: picked.len(),
        keys: values.len(),
        p50: percentile(&values, 50)?,
        p95: percentile(&values, 95)?,
    })
}

/// 原生與 WebAssembly 各量一次 `inputs`：固定題與各語料集合的每一題（個人集不量，各機器才比得起來）。
pub fn measure(
    engine: &Engine,
    packs: &[PathBuf],
    inputs: &[String],
    wasm: WasmTimer,
) -> Result<Latency, String> {
    let native = native(engine, inputs);
    let wasm = wasm(packs, inputs)?;
    let shape = |timing: &Timing| timing.per_key_ms.iter().map(Vec::len).collect::<Vec<_>>();
    let keys: Vec<usize> = inputs.iter().map(|input| input.chars().count()).collect();
    if shape(&wasm) != keys {
        return Err(format!(
            "WebAssembly 的計時結果和輸入對不上：{} 個輸入 {} 鍵，回來 {} 個輸入 {} 鍵",
            keys.len(),
            keys.iter().sum::<usize>(),
            wasm.per_key_ms.len(),
            wasm.per_key_ms.iter().map(Vec::len).sum::<usize>()
        ));
    }
    Ok(Latency {
        machine: machine(),
        native,
        wasm,
    })
}

/// 排序後第 ⌈p/100 × n⌉ 個（nearest-rank）；沒有值時是 `None`。
pub fn percentile(values: &[f64], p: u32) -> Option<f64> {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let rank = (sorted.len() * p as usize).div_ceil(100).max(1);
    sorted.get(rank - 1).copied()
}

/// 原生：每個輸入用新的 `Session` 與空白的 `UserModel`，只計 `Session::handle` 那一次呼叫。
fn native(engine: &Engine, inputs: &[String]) -> Timing {
    let per_key_ms = inputs
        .iter()
        .map(|input| {
            let mut session = Session::default();
            let mut user = UserModel::default();
            input
                .chars()
                .map(|c| {
                    let start = Instant::now();
                    let snapshot = session.handle(engine, &mut user, Key::Letter(c));
                    let ms = start.elapsed().as_secs_f64() * 1000.0;
                    drop(black_box(snapshot));
                    ms
                })
                .collect()
        })
        .collect();
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    Timing {
        runtime: profile.into(),
        per_key_ms,
    }
}

/// 用 Node 跑 [`WASM_SCRIPT`]：資料包路徑當參數，輸入碼一行一個從 stdin 送進去；stdout 第一行是 runtime，
/// 之後每個輸入一行、各鍵的毫秒數以空白分隔。`root` 是 repo 根目錄。
pub fn node_wasm_timer(
    root: &Path,
) -> impl Fn(&[PathBuf], &[String]) -> Result<Timing, String> + '_ {
    move |packs, inputs| {
        if !root.join(WASM_BINDINGS).exists() {
            return Err(format!(
                "沒有 {WASM_BINDINGS}：用 `cargo xtask bench` 跑（會先編綁定），或先跑 `cargo xtask web`"
            ));
        }
        let mut child = Command::new("node")
            .arg(root.join(WASM_SCRIPT))
            .args(packs)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| format!("跑不起 node：{e}"))?;
        let mut stdin = child.stdin.take().expect("stdin 是 piped");
        let text: String = inputs.iter().map(|input| format!("{input}\n")).collect();
        // 另一個執行緒寫 stdin，免得兩邊的管線都滿了互等。寫失敗多半是 node 先結束了，看它的 exit code。
        let output = std::thread::scope(|scope| {
            scope.spawn(move || stdin.write_all(text.as_bytes()));
            child.wait_with_output()
        })
        .map_err(|e| format!("讀不到 node 的輸出：{e}"))?;
        if !output.status.success() {
            return Err(format!("{WASM_SCRIPT} 失敗：{}", output.status));
        }
        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| format!("{WASM_SCRIPT} 的輸出不是 UTF-8：{e}"))?;
        parse_node_output(&stdout)
    }
}

fn parse_node_output(stdout: &str) -> Result<Timing, String> {
    let mut lines = stdout.lines();
    let runtime = lines
        .next()
        .filter(|line| !line.is_empty())
        .ok_or_else(|| format!("{WASM_SCRIPT} 沒有輸出"))?;
    let per_key_ms = lines
        .enumerate()
        .map(|(i, line)| {
            line.split(' ')
                .map(|ms| {
                    ms.parse::<f64>()
                        .map_err(|_| format!("{WASM_SCRIPT} 第 {} 行不是毫秒數：{ms}", i + 2))
                })
                .collect()
        })
        .collect::<Result<_, _>>()?;
    Ok(Timing {
        runtime: runtime.into(),
        per_key_ms,
    })
}

/// 例如 `Apple M6（macos aarch64）`；查不到 CPU 型號時只有作業系統與架構。
fn machine() -> String {
    let os = format!("{} {}", std::env::consts::OS, std::env::consts::ARCH);
    match cpu() {
        Some(cpu) => format!("{cpu}（{os}）"),
        None => os,
    }
}

fn cpu() -> Option<String> {
    let name = if cfg!(target_os = "macos") {
        let out = Command::new("sysctl")
            .args(["-n", "machdep.cpu.brand_string"])
            .output()
            .ok()?;
        String::from_utf8(out.stdout).ok()?
    } else {
        let info = std::fs::read_to_string("/proc/cpuinfo").ok()?;
        let line = info.lines().find(|l| l.starts_with("model name"))?;
        line.split_once(':')?.1.to_string()
    };
    let name = name.trim();
    (!name.is_empty()).then(|| name.to_string())
}
