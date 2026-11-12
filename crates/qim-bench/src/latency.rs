//! 每鍵延遲（plan §5.2）：逐鍵打基準的每個輸入碼，量每次按鍵到 `Snapshot` 回來的時間，
//! 原生與 WebAssembly 各量一次，報 p50 / p95；空白的學習資料與學過的（[`UserData`]）各一組。
//! 原生的 p95 由 `--check` 擋（[`p95_limits_ms`]），WebAssembly 只報；baseline 不記。
//!
//! 每個輸入都從空的組字打起；載入資料包不算在內，另報一次載入的時間（[`Timing::load_ms`]）。
//! 打完有兩段以上的輸入另量「選開頭一段」那一鍵（[`lefts_to_first_segment`]）：選定後剩下的輸入碼
//! 從頭重建 lattice，是每鍵最重的一種。

use std::hint::black_box;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

use qim_core::{Engine, Key, Session, Snapshot, UserModel};

/// 計時時的學習資料。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserData {
    /// 空白的 `UserModel`，同基準的其他部分。
    Blank,
    /// 一個 `UserModel` 依序學完各語料集合與「有前文」的題（給前文）之後（M3 票 12 的量法）。
    Learned,
}

impl UserData {
    pub fn label(self) -> &'static str {
        match self {
            UserData::Blank => "空白",
            UserData::Learned => "學過",
        }
    }
}

/// 原生 p95 的上限，毫秒，順序同 [`Timing::groups`]（全部、第 20 碼起、選開頭一段）。
/// 數字與改上限的條件在 `.scratch/m4/spec.md`〈每鍵延遲上限〉：比 M3 收尾時留約 7–13 倍的餘裕，
/// 擋倍數級的變慢，不擋機器負載的雜訊。
pub fn p95_limits_ms(user: UserData) -> [f64; 3] {
    match user {
        UserData::Blank => [1.0, 1.0, 5.0],
        UserData::Learned => [3.0, 3.0, 10.0],
    }
}

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
    /// 每個輸入打完後選開頭一段的毫秒數（[`lefts_to_first_segment`]），順序同輸入；打完不到兩段的是 `None`。
    pub select_ms: Vec<Option<f64>>,
    /// 載入的毫秒數，沒量是 `None`。原生是解開每份資料包加建 `Engine`（`qim_bench::load_engine`，
    /// 不含讀檔）；WebAssembly 是 `new Session(packs)`（`Engine::open`：核對 manifest 與 sha256、
    /// 解開、建 `Engine`，同網頁）。
    pub load_ms: Option<f64>,
}

/// 在 WebAssembly 上量時間的方式：給基準載入的每份資料包路徑（順序不拘，腳本依檔頭魔數分辨）、
/// 學習資料（`UserModel::export` 的文字，空白的學習資料是 `None`）與輸入碼，逐鍵打每個輸入。
/// 實際用 [`node_wasm_timer`]；測試可以換成假的。
pub type WasmTimer<'a> = &'a dyn Fn(&[PathBuf], Option<&str>, &[String]) -> Result<Timing, String>;

#[derive(Debug, Clone, PartialEq)]
pub struct Latency {
    /// CPU、作業系統與架構，報告註明在哪台機器量的。
    pub machine: String,
    pub blank: Runs,
    pub learned: Runs,
}

/// 同一份學習資料下原生與 WebAssembly 各量一次。
#[derive(Debug, Clone, PartialEq)]
pub struct Runs {
    pub native: Timing,
    pub wasm: Timing,
}

impl Latency {
    /// 空白與學過兩組，依序。
    pub fn runs(&self) -> [(UserData, &Runs); 2] {
        [
            (UserData::Blank, &self.blank),
            (UserData::Learned, &self.learned),
        ]
    }

    /// 原生的 p95 超過 [`p95_limits_ms`] 的每一格一條；WebAssembly 只報不擋。組裡沒有按鍵的不比。
    pub fn violations(&self) -> Vec<String> {
        let mut violations = Vec::new();
        for (user, runs) in self.runs() {
            for ((group, summary), limit) in
                runs.native.groups().into_iter().zip(p95_limits_ms(user))
            {
                if let Some(s) = summary
                    && s.p95 > limit
                {
                    violations.push(format!(
                        "每鍵延遲超過上限：原生、學習資料{}、{group} 的 p95 {:.0} µs > {:.0} µs\
                         （先在負載低時單獨重跑一次，仍超過才算，見 docs/development.md〈每鍵延遲〉）",
                        user.label(),
                        s.p95 * 1000.0,
                        limit * 1000.0
                    ));
                }
            }
        }
        violations
    }
}

/// 一組輸入的統計，毫秒。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Summary {
    pub inputs: usize,
    pub keys: usize,
    pub p50: f64,
    pub p95: f64,
}

impl Summary {
    /// p95 超過上限 `limit_ms`（[`p95_limits_ms`] 的一格）。
    pub fn over(&self, limit_ms: f64) -> bool {
        self.p95 > limit_ms
    }
}

impl Timing {
    /// 「全部」、「第 [`LONG_INPUT_KEYS`] 碼起」與「選開頭一段」三組；組裡沒有按鍵時是 `None`。
    pub fn groups(&self) -> [(String, Option<Summary>); 3] {
        [
            ("全部".into(), summary(self, 0)),
            (
                format!("第 {LONG_INPUT_KEYS} 碼起"),
                summary(self, LONG_INPUT_KEYS - 1),
            ),
            ("選開頭一段".into(), select_summary(self)),
        ]
    }
}

/// 選開頭一段那一鍵：每個量到的輸入一鍵。
fn select_summary(timing: &Timing) -> Option<Summary> {
    let values: Vec<f64> = timing.select_ms.iter().flatten().copied().collect();
    Some(Summary {
        inputs: values.len(),
        keys: values.len(),
        p50: percentile(&values, 50)?,
        p95: percentile(&values, 95)?,
    })
}

/// 打完輸入後要怎麼量「選開頭一段」：分段數不到兩段時不量；否則先按這麼多次 ←（不計時）移到第一段，
/// 再計 `select(0)`（選它的首選，剩下的輸入碼從頭重建）。原生、WebAssembly、Swift 都照這個做。
pub fn lefts_to_first_segment(snapshot: &Snapshot) -> Option<usize> {
    let segments = snapshot.composition.segments.len();
    (segments >= 2).then_some(segments)
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

/// 空白與學過的學習資料（`learned`）下，原生與 WebAssembly 各量一次 `inputs`：固定題與各語料集合的
/// 每一題（個人集不量，各機器才比得起來）。
pub fn measure(
    engine: &Engine,
    packs: &[PathBuf],
    inputs: &[String],
    learned: &UserModel,
    wasm: WasmTimer,
) -> Result<Latency, String> {
    Ok(Latency {
        machine: machine(),
        blank: measure_runs(engine, packs, inputs, None, wasm)?,
        learned: measure_runs(engine, packs, inputs, Some(learned), wasm)?,
    })
}

/// 同一份學習資料（`None` 是空白的；WebAssembly 匯入它的 `UserModel::export`）下各量一次，
/// 核對兩邊量的是同一批輸入。
fn measure_runs(
    engine: &Engine,
    packs: &[PathBuf],
    inputs: &[String],
    user: Option<&UserModel>,
    wasm: WasmTimer,
) -> Result<Runs, String> {
    let native = native_timing_with(engine, user.unwrap_or(&UserModel::default()), inputs);
    let user_dict = user.map(UserModel::export);
    let wasm = wasm(packs, user_dict.as_deref(), inputs)?;
    let shape = |timing: &Timing| timing.per_key_ms.iter().map(Vec::len).collect::<Vec<_>>();
    let keys: Vec<usize> = inputs.iter().map(|input| input.chars().count()).collect();
    let selects = |timing: &Timing| {
        timing
            .select_ms
            .iter()
            .map(Option::is_some)
            .collect::<Vec<_>>()
    };
    if shape(&wasm) != keys {
        return Err(format!(
            "WebAssembly 的計時結果和輸入對不上：{} 個輸入 {} 鍵，回來 {} 個輸入 {} 鍵",
            keys.len(),
            keys.iter().sum::<usize>(),
            wasm.per_key_ms.len(),
            wasm.per_key_ms.iter().map(Vec::len).sum::<usize>()
        ));
    }
    if selects(&wasm) != selects(&native) {
        return Err(format!(
            "WebAssembly 量選開頭一段的輸入和原生不同：原生 {} 個，WebAssembly {} 個",
            native.select_ms.iter().flatten().count(),
            wasm.select_ms.iter().flatten().count()
        ));
    }
    Ok(Runs { native, wasm })
}

/// 排序後第 ⌈p/100 × n⌉ 個（nearest-rank）；沒有值時是 `None`。
pub fn percentile(values: &[f64], p: u32) -> Option<f64> {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let rank = (sorted.len() * p as usize).div_ceil(100).max(1);
    sorted.get(rank - 1).copied()
}

/// 原生在空白的學習資料下量一次 `inputs`（模型階梯也用它量換了語言模型的引擎，M3 票 08），
/// 見 [`native_timing_with`]。
pub fn native_timing(engine: &Engine, inputs: &[String]) -> Timing {
    native_timing_with(engine, &UserModel::default(), inputs)
}

/// 原生在學習資料 `user` 下量一次 `inputs`（[`measure`] 的原生那一半）：每個輸入用新的 `Session`，
/// 只計 `Session::handle`（選開頭一段是 `Session::select`）那一次呼叫。各輸入共用 `user` 的一份複本，
/// 同 WebAssembly 一個 `Session` 打到底（打字與選開頭一段都不上屏，不學）。
pub fn native_timing_with(engine: &Engine, user: &UserModel, inputs: &[String]) -> Timing {
    let mut user = user.clone();
    let (per_key_ms, select_ms) = inputs
        .iter()
        .map(|input| {
            let mut session = Session::default();
            let mut last = Snapshot::default();
            let keys = input
                .chars()
                .map(|c| {
                    let start = Instant::now();
                    let snapshot = session.handle(engine, &mut user, Key::for_input(c));
                    let ms = start.elapsed().as_secs_f64() * 1000.0;
                    last = black_box(snapshot);
                    ms
                })
                .collect();
            let select = lefts_to_first_segment(&last).map(|lefts| {
                for _ in 0..lefts {
                    session.handle(engine, &mut user, Key::Left);
                }
                let start = Instant::now();
                let snapshot = session.select(engine, &mut user, 0);
                let ms = start.elapsed().as_secs_f64() * 1000.0;
                drop(black_box(snapshot));
                ms
            });
            (keys, select)
        })
        .unzip();
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    Timing {
        runtime: profile.into(),
        per_key_ms,
        select_ms,
        load_ms: None,
    }
}

/// 用 Node 跑 [`WASM_SCRIPT`]：資料包路徑當參數，有學習資料時先寫到暫存檔、以 `--user-dict <檔>`
/// 給它；輸入碼一行一個從 stdin 送進去；stdout 的格式見 [`parse_timing`]。`root` 是 repo 根目錄。
pub fn node_wasm_timer(
    root: &Path,
) -> impl Fn(&[PathBuf], Option<&str>, &[String]) -> Result<Timing, String> + '_ {
    move |packs, user_dict, inputs| {
        if !root.join(WASM_BINDINGS).exists() {
            return Err(format!(
                "沒有 {WASM_BINDINGS}：用 `cargo xtask bench` 跑（會先編綁定），或先跑 `cargo xtask web`"
            ));
        }
        // 離開時刪掉。
        let dict_file = user_dict
            .map(|text| {
                let mut file = tempfile::NamedTempFile::new()?;
                file.write_all(text.as_bytes())?;
                Ok(file)
            })
            .transpose()
            .map_err(|e: std::io::Error| format!("寫不了學習資料的暫存檔：{e}"))?;
        let mut command = Command::new("node");
        command.arg(root.join(WASM_SCRIPT));
        if let Some(file) = &dict_file {
            command.arg("--user-dict").arg(file.path());
        }
        let mut child = command
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
        parse_timing(&stdout).map_err(|e| format!("{WASM_SCRIPT}：{e}"))
    }
}

/// 綁定的計時腳本（Node 的 [`WASM_SCRIPT`]、`scripts/experiments/` 的 Swift 程式）的輸出：第一行是
/// runtime，有量載入時接 ` | ` 與載入的毫秒數；之後每個輸入一行、各鍵的毫秒數以空白分隔；有選開頭
/// 一段時行尾接 ` | ` 與它的毫秒數。
pub fn parse_timing(stdout: &str) -> Result<Timing, String> {
    let mut lines = stdout.lines();
    let first = lines
        .next()
        .filter(|line| !line.is_empty())
        .ok_or("沒有輸出")?;
    let (runtime, load_ms) = match first.split_once(" | ") {
        Some((runtime, load)) => (
            runtime,
            Some(
                load.parse::<f64>()
                    .map_err(|_| format!("第 1 行的載入不是毫秒數：{load}"))?,
            ),
        ),
        None => (first, None),
    };
    let ms = |i: usize, text: &str| {
        text.parse::<f64>()
            .map_err(|_| format!("第 {} 行不是毫秒數：{text}", i + 2))
    };
    let (per_key_ms, select_ms) = lines
        .enumerate()
        .map(|(i, line)| {
            let (keys, select) = match line.split_once(" | ") {
                Some((keys, select)) => (keys, Some(ms(i, select)?)),
                None => (line, None),
            };
            let keys = keys
                .split(' ')
                .map(|text| ms(i, text))
                .collect::<Result<Vec<_>, _>>()?;
            Ok((keys, select))
        })
        .collect::<Result<Vec<_>, String>>()?
        .into_iter()
        .unzip();
    Ok(Timing {
        runtime: runtime.into(),
        per_key_ms,
        select_ms,
        load_ms,
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
