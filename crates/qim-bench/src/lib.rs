//! 首選字基準（plan §5.2）：用核心實際打 `bench/` 的每一題，量首選正確率並擋退步。
//!
//! 只透過 `qim-core` 的公開介面打字；讀檔在這裡做，核心不碰 I/O（plan §1.3）。
//!
//! `qim-databuild` 寫、這裡讀的路徑與檔頭（[`CORPUS_DIRS`]、[`dev_items`]、個人集的 [`PERSONAL`]、
//! [`PERSONAL_USERDB`]、[`PERSONAL_SOURCE_HEADER`]）在兩個 crate 各有一份（plan §1.3 不讓本 crate
//! 依賴 `qim-databuild`），`xtask/tests/shared_paths.rs` 比對兩邊。

mod baseline;
mod items;
mod latency;
mod report;
mod score;

use std::fs;
use std::path::{Path, PathBuf};

use std::time::Instant;

use qim_core::{
    EmojiPack, Engine, EnglishPack, LanguageModelPack, Lexicon, LexiconPack, Pack, Ranking,
    SchemePack, WordTrigramPack,
};
use sha2::{Digest, Sha256};

pub use baseline::{Baseline, Flips, SetBaseline, parse as parse_baseline};
pub use items::{Item, Unit, parse_context, parse_corpus, parse_fixed, parse_personal};
pub use latency::{
    LONG_INPUT_KEYS, Latency, Summary, Timing, WASM_BINDINGS, WasmTimer, lefts_to_first_segment,
    native_timing, node_wasm_timer, parse_timing, percentile,
};
/// 打一個集合，指標同基準；模型階梯實驗用它照基準的定義打自己挑的題（M3 票 07）。
pub use score::score as score_set;
pub use score::{Failure, GroupStats, ItemResult, LENGTH_GROUPS, LearningSet, Metrics, SetResult};

/// 基準用的資料包，由 `cargo xtask data` 產生；方案指名的詞庫在 [`PACKS`] 下的 `<id>/<id>.pack`。
pub const QUICK_PACK: &str = "build/packs/quick/quick.pack";
pub const PACKS: &str = "build/packs";
/// 英文資料包：不由方案指名、各方案共用（plan §4.4）；預設選項開著英文候選，基準照網頁掛上。
/// WebAssembly 的延遲腳本依檔頭魔數分辨各資料包，不靠參數的位置。
pub const ENGLISH_PACK: &str = "build/packs/scowl-en-us/scowl-en-us.pack";
/// emoji 資料包：不由方案指名、各方案共用（plan §4.4）；預設選項開著 emoji 候選，基準照網頁掛上，
/// 量得到 emoji 把正解擠到下一頁的修正次數。
pub const EMOJI_PACK: &str = "build/packs/cldr-emoji-yue-hant/cldr-emoji-yue-hant.pack";
pub const FIXED: &str = "bench/fixed.tsv";
/// 語料目錄，依序跑：每個 `<名>.tsv` 是一個集合，集合名在所有目錄間不能重複。
/// 維基防退步，口語是品質判斷的主要依據（brief §9）；目錄不存在就當沒有集合。
pub const CORPUS_DIRS: [&str; 2] = ["bench/corpus", "bench/colloquial"];
pub const BASELINE: &str = "bench/baseline.json";
/// 「有前文」的題目目錄（plan §5.2），依序跑：每個 `<名>.tsv` 是一批題，集合名在這些目錄間不能重複。
pub const CONTEXT_DIRS: [&str; 2] = ["bench/corpus/context", "bench/colloquial/context"];
/// 「有前文」的一批題 `<名>` 打兩次，集合名是 `<名>` 接這些字尾：給前文、不給前文。
/// 學習模式只打給前文的。
pub const CONTEXT_SUFFIXES: [&str; 2] = ["·前文", "·前文·不給"];
/// dev 模式的題目：[`CORPUS_DIRS`] 每個集合 `name` 的 dev 切分，相對 repo 根目錄；
/// `cargo xtask data` 寫的，不進 git。
pub fn dev_items(name: &str) -> String {
    format!("build/corpus/{name}/dev.tsv")
}
/// dev 模式的「有前文」題：集合 `name` 的 dev 切分，取法同 [`CONTEXT_DIRS`] 的題；
/// `cargo xtask data` 寫的，不進 git，沒有就不打。
pub fn dev_context_items(name: &str) -> String {
    format!("build/corpus/{name}/dev-context.tsv")
}
/// 集合 `name` 的長前文題（test 切分）：[`CONTEXT_DIRS`] 的同一批題，前文換成同一段裡前面最多
/// 200 字；`cargo xtask data` 寫的，不進 git。基準不讀，模型階梯實驗讀（M3 票 07，
/// `scripts/experiments/model-ladder/`）。
pub fn context_long_items(name: &str) -> String {
    format!("build/corpus/{name}/test-context-long.tsv")
}
/// 集合 `name` 的 dev 長前文題，同 [`context_long_items`]，取自 [`dev_context_items`] 的同一批題。
pub fn dev_context_long_items(name: &str) -> String {
    format!("build/corpus/{name}/dev-context-long.tsv")
}
/// dev 模式裡由集合 `<名>` 的句子取出的短輸入，集合名是 `<名>` 接這些字尾：
/// 每個字（單字），與句子裡出現的詞庫的詞（2 到 [`SHORT_WORD_CHARS`] 字）。
pub const SHORT_INPUTS: [&str; 2] = ["·單字", "·詞"];
/// [`SHORT_INPUTS`] 的詞最長幾個字：個人集幾乎都在 4 字以內。
pub const SHORT_WORD_CHARS: usize = 4;
/// 個人集的題目，由 `cargo xtask data` 從 [`PERSONAL_USERDB`] 轉出；只在本機（plan §5.2）。
pub const PERSONAL: &str = "bench/personal/rime-userdb.tsv";
/// 使用者放的 Rime 學習資料；基準不直接讀它，只在題目還沒轉出時提示。
pub const PERSONAL_USERDB: &str = "bench/personal/rime-userdb.txt";
pub const PERSONAL_BASELINE: &str = "bench/personal/baseline.json";
/// 個人集在結果與 baseline 裡的集合名。
pub const PERSONAL_SET: &str = "personal";
/// [`PERSONAL`] 的第一行：這個前綴接 [`PERSONAL_USERDB`] 的 sha256（`qim_databuild::personal` 寫）。
pub const PERSONAL_SOURCE_HEADER: &str = "# rime-userdb.txt sha256 ";

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
    /// 讀進來的資料包：速成、方案指名的詞庫、語言模型與詞級 trigram、英文、emoji。前後對照時 sha256
    /// 一樣就是沒重建。
    pub packs: Vec<LoadedPack>,
    pub fixed: SetResult,
    /// 各語料集合，依 [`CORPUS_DIRS`] 的順序，同一目錄內依名稱排序；接著「有前文」的各批題，
    /// 依 [`CONTEXT_DIRS`] 的順序，每批給前文、不給前文各一個集合（[`CONTEXT_SUFFIXES`]）。
    pub sets: Vec<SetResult>,
    pub personal: Personal,
    /// 跑之前的 baseline，報告裡對照用；沒有檔案時是 `None`。
    pub baseline: Option<Baseline>,
    /// 跑之前的個人集 baseline；沒有檔案時是 `None`。
    pub personal_baseline: Option<Baseline>,
    /// 每鍵延遲：只報，不進違規也不進 baseline。
    pub latency: Latency,
    /// 擋退步的違規；空的就是通過。
    pub violations: Vec<String>,
}

impl Outcome {
    /// 給人看的結果，Markdown 表。
    pub fn report(&self) -> String {
        report::render(self)
    }

    /// 集合 `set` 跑之前的 baseline。
    pub fn baseline_of(&self, set: &SetResult) -> Option<&SetBaseline> {
        let file = if set.private {
            &self.personal_baseline
        } else {
            &self.baseline
        };
        file.as_ref()?.get(&set.name)
    }

    /// 集合 `name` 和 baseline 比的翻盤清單；沒有這個集合或沒有 baseline 時是 `None`。
    pub fn flips(&self, name: &str) -> Option<Flips> {
        let set = self.scored_sets().find(|s| s.name == name)?;
        Some(baseline::flips(set, self.baseline_of(set)?))
    }

    /// 語料集合，有個人集時接在最後。
    pub fn scored_sets(&self) -> impl Iterator<Item = &SetResult> {
        self.sets.iter().chain(match &self.personal {
            Personal::Ran(set) => Some(set),
            _ => None,
        })
    }
}

/// 一份讀進來的資料包。
#[derive(Debug, Clone)]
pub struct LoadedPack {
    /// 相對 repo 根目錄。
    pub path: String,
    /// 十六進位。
    pub sha256: String,
}

/// 個人集這次的狀況；`Ran` 平常是結果，讀檔時是題目。
#[derive(Debug)]
pub enum Personal<T = SetResult> {
    Ran(T),
    /// 沒有 [`PERSONAL_USERDB`]：跳過。
    Missing,
    /// 有 [`PERSONAL_USERDB`]，但 [`PERSONAL`] 不是由它轉出（還沒轉，或資料換過）：跳過，提示先跑 `cargo xtask data`。
    NotTranscoded,
}

impl<T> Personal<T> {
    fn map<U>(self, f: impl FnOnce(T) -> U) -> Personal<U> {
        match self {
            Personal::Ran(t) => Personal::Ran(f(t)),
            Personal::Missing => Personal::Missing,
            Personal::NotTranscoded => Personal::NotTranscoded,
        }
    }
}

/// 量每鍵延遲的輸入碼：固定題與各語料集合的每一題，依序（個人集與「有前文」的題不量）。
pub fn latency_inputs(root: &Path) -> Result<Vec<String>, String> {
    let fixed = parse_fixed(&read(&root.join(FIXED))?).map_err(|e| format!("{FIXED}：{e}"))?;
    Ok(inputs_of(&fixed, &load_corpora(root)?))
}

fn inputs_of(fixed: &[Item], corpora: &[(String, Vec<Item>)]) -> Vec<String> {
    fixed
        .iter()
        .chain(corpora.iter().flat_map(|(_, items)| items))
        .map(Item::input)
        .collect()
}

/// 在 repo 根目錄 `root` 跑基準；WebAssembly 的每鍵延遲用 `wasm` 量（實際用 [`node_wasm_timer`]）。
/// `lexicon` 是另一份詞庫資料包的 id（`--lexicon`），換掉方案指名的那份；baseline 是預設詞庫的數字，
/// 所以換了詞庫不能 [`Mode::UpdateBaseline`]，`--check` 照樣和它比（「不變差」）。
pub fn run(
    root: &Path,
    mode: Mode,
    lexicon: Option<&str>,
    wasm: WasmTimer,
) -> Result<Outcome, String> {
    if mode == Mode::UpdateBaseline
        && let Some(id) = lexicon
    {
        return Err(format!(
            "--lexicon {id} 不能 --update-baseline：baseline 是方案指名的詞庫的數字"
        ));
    }
    let LoadedEngine {
        engine,
        packs,
        load_ms,
        ..
    } = load_engine(root, &[], lexicon)?;
    let fixed_items =
        parse_fixed(&read(&root.join(FIXED))?).map_err(|e| format!("{FIXED}：{e}"))?;
    let fixed = score::score(&engine, "fixed", &fixed_items, false);
    let corpora = load_corpora(root)?;
    let mut sets: Vec<SetResult> = corpora
        .iter()
        .map(|(name, items)| score::score(&engine, name, items, false))
        .collect();
    let latency_inputs = inputs_of(&fixed_items, &corpora);
    // 「有前文」的題不量延遲：輸入碼和一般的題同一種，多量只是重複。
    for (name, items) in load_context_sets(root)? {
        sets.extend(score_with_and_without_context(&engine, &name, &items));
    }
    let personal =
        load_personal(root)?.map(|items| score::score(&engine, PERSONAL_SET, &items, true));
    let pack_paths: Vec<PathBuf> = packs.iter().map(|p| root.join(&p.path)).collect();
    let mut latency = latency::measure(&engine, &pack_paths, &latency_inputs, wasm)?;
    latency.native.load_ms = Some(load_ms);
    // 改寫 baseline 時舊檔的格式不對（例如格式改了）不算錯，當作沒有；讀不到檔照樣是錯。
    let tolerate_bad_format = mode == Mode::UpdateBaseline;
    let baseline = load_baseline(&root.join(BASELINE), tolerate_bad_format)?;
    let personal_baseline = load_baseline(&root.join(PERSONAL_BASELINE), tolerate_bad_format)?;
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
        packs,
        fixed,
        sets,
        personal,
        baseline,
        personal_baseline,
        latency,
        violations,
    })
}

/// dev 模式的結果：調參用（plan §2.3、§5.2），只報、不比 baseline，不跑個人集與每鍵延遲。
#[derive(Debug)]
pub struct DevOutcome {
    pub packs: Vec<LoadedPack>,
    /// 這次用的打分參數：方案檔的值套上覆寫之後。
    pub ranking: Ranking,
    pub fixed: SetResult,
    /// 各語料集合的 dev 切分，順序同 [`Outcome::sets`]；每個集合後面接它的短輸入
    /// （[`SHORT_INPUTS`]、[`items::substrings`]）：dev 都是長句，幾乎沒有 1、2 字的題，
    /// 看不出參數讓兩碼的字被兩個一碼字壓過、或單打一個詞時選錯（票 07、12；個人集七成是兩字詞）。
    pub sets: Vec<SetResult>,
}

impl DevOutcome {
    pub fn report(&self) -> String {
        report::render_dev(self)
    }
}

/// dev 模式：照 [`run`] 的方式打各集合的 dev 切分（[`dev_items`]）與固定題。`overrides` 是
/// `<鍵>=<值>`，暫時蓋掉方案檔 `[ranking]` 的同名欄位，資料包不變；`lexicon` 同 [`run`]。
pub fn run_dev(
    root: &Path,
    overrides: &[String],
    lexicon: Option<&str>,
) -> Result<DevOutcome, String> {
    let overrides = overrides
        .iter()
        .map(|o| {
            o.split_once('=')
                .ok_or_else(|| format!("覆寫要寫成 key=value：{o}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let LoadedEngine {
        engine,
        packs,
        lexicon,
        ..
    } = load_engine(root, &overrides, lexicon)?;
    let lexicon = lexicon.unwrap_or_default();
    let is_word = |item: &Item| {
        let text = item.expected();
        lexicon.lookup(&item.input()).iter().any(|w| w.text == text)
    };
    let fixed = parse_fixed(&read(&root.join(FIXED))?).map_err(|e| format!("{FIXED}：{e}"))?;
    let fixed = score::score(&engine, "fixed", &fixed, false);
    let mut sets = Vec::new();
    let corpora = load_corpora(root)?;
    for (name, _) in &corpora {
        let rel = dev_items(name);
        let path = root.join(&rel);
        let text = fs::read_to_string(&path)
            .map_err(|e| format!("讀不到 {rel}：{e}；先跑 `cargo xtask data`"))?;
        let items = parse_corpus(&text).map_err(|e| format!("{rel}：{e}"))?;
        sets.push(score::score(&engine, name, &items, false));
        let [chars, words] = SHORT_INPUTS;
        let short = items::substrings(&items, 1);
        sets.push(score::score(
            &engine,
            &format!("{name}{chars}"),
            &short,
            false,
        ));
        let short: Vec<Item> = (2..=SHORT_WORD_CHARS)
            .flat_map(|len| items::substrings(&items, len))
            .filter(|item| is_word(item))
            .collect();
        sets.push(score::score(
            &engine,
            &format!("{name}{words}"),
            &short,
            false,
        ));
    }
    for (name, _) in &corpora {
        let rel = dev_context_items(name);
        let text = match fs::read_to_string(root.join(&rel)) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(format!("讀不到 {rel}：{e}")),
        };
        let items = parse_context(&text).map_err(|e| format!("{rel}：{e}"))?;
        sets.extend(score_with_and_without_context(&engine, name, &items));
    }
    Ok(DevOutcome {
        packs,
        ranking: engine.scheme().ranking,
        fixed,
        sets,
    })
}

/// 「有前文」的一批題打兩次（[`CONTEXT_SUFFIXES`]）：給前文，與拿掉前文的同一批題。
fn score_with_and_without_context(engine: &Engine, name: &str, items: &[Item]) -> [SetResult; 2] {
    let [given, not_given] = CONTEXT_SUFFIXES;
    let without: Vec<Item> = items
        .iter()
        .map(|item| Item {
            context: None,
            ..item.clone()
        })
        .collect();
    [
        score::score(engine, &format!("{name}{given}"), items, false),
        score::score(engine, &format!("{name}{not_given}"), &without, false),
    ]
}

/// `--set` 能蓋的鍵：方案檔 `[ranking]` 的欄位名。
const RANKING_KEYS: [&str; 4] = ["word_bonus", "segment_penalty", "beam", "sentences"];

/// 把 `key=value` 套到 `ranking`；鍵見 [`RANKING_KEYS`]。
fn override_ranking(ranking: &mut Ranking, key: &str, value: &str) -> Result<(), String> {
    fn parse<T: std::str::FromStr>(key: &str, value: &str) -> Result<T, String> {
        value
            .parse()
            .map_err(|_| format!("--set {key} 的值不對：{value}"))
    }
    match key {
        "word_bonus" => ranking.word_bonus = parse(key, value)?,
        "segment_penalty" => ranking.segment_penalty = parse(key, value)?,
        "beam" => ranking.beam = parse(key, value)?,
        "sentences" => ranking.sentences = parse(key, value)?,
        _ => {
            return Err(format!(
                "沒有這個打分參數：{key}（可用 {}）",
                RANKING_KEYS.join("、")
            ));
        }
    }
    Ok(())
}

fn write(path: &Path, contents: &str) -> Result<(), String> {
    fs::write(path, contents).map_err(|e| format!("寫不了 {}：{e}", path.display()))
}

fn load_baseline(path: &Path, tolerate_bad_format: bool) -> Result<Option<Baseline>, String> {
    if !path.exists() {
        return Ok(None);
    }
    match baseline::parse(&read(path)?) {
        Ok(baseline) => Ok(Some(baseline)),
        Err(_) if tolerate_bad_format => Ok(None),
        Err(e) => Err(format!("{}：{e}", path.display())),
    }
}

/// 個人集：沒有 [`PERSONAL_USERDB`] 就跳過；題目檔第一行記的來源 sha256 要等於現在的
/// [`PERSONAL_USERDB`]，否則是還沒轉出或換過資料沒重轉，也跳過，免得安靜地跑舊題目。
pub fn load_personal(root: &Path) -> Result<Personal<Vec<Item>>, String> {
    let userdb = root.join(PERSONAL_USERDB);
    if !userdb.exists() {
        return Ok(Personal::Missing);
    }
    let items = root.join(PERSONAL);
    if !items.exists() {
        return Ok(Personal::NotTranscoded);
    }
    let source = fs::read(&userdb).map_err(|e| format!("讀不到 {}：{e}", userdb.display()))?;
    let text = read(&items)?;
    let header = format!("{PERSONAL_SOURCE_HEADER}{}", sha256_hex(&source));
    if text.lines().next() != Some(header.as_str()) {
        return Ok(Personal::NotTranscoded);
    }
    let items = parse_personal(&text).map_err(|e| format!("{PERSONAL}：{e}"))?;
    Ok(Personal::Ran(items))
}

/// 各語料集合的名稱與題目，依 [`CORPUS_DIRS`] 的順序，同一目錄內依名稱排序。
pub fn load_corpora(root: &Path) -> Result<Vec<(String, Vec<Item>)>, String> {
    load_sets(root, &CORPUS_DIRS, parse_corpus)
}

/// 「有前文」的各批題的名稱與題目，依 [`CONTEXT_DIRS`] 的順序，同一目錄內依名稱排序。
fn load_context_sets(root: &Path) -> Result<Vec<(String, Vec<Item>)>, String> {
    load_sets(root, &CONTEXT_DIRS, parse_context)
}

/// `dirs` 下每個 `<名>.tsv` 照 `parse` 讀成一個集合，依目錄的順序、同一目錄內依名稱排序；
/// 集合名在這些目錄間不能重複，也不能是個人集的名字。
fn load_sets(
    root: &Path,
    dirs: &[&str],
    parse: fn(&str) -> Result<Vec<Item>, String>,
) -> Result<Vec<(String, Vec<Item>)>, String> {
    let mut sets: Vec<(String, Vec<Item>)> = Vec::new();
    for dir in dirs {
        for (name, path) in corpus_files(&root.join(dir))? {
            if name == PERSONAL_SET {
                return Err(format!(
                    "語料不能叫 {PERSONAL_SET}，那是個人集的名字：{}",
                    path.display()
                ));
            }
            if sets.iter().any(|(n, _)| *n == name) {
                return Err(format!("集合 {name} 在兩個目錄都有：{}", path.display()));
            }
            let items = parse(&read(&path)?).map_err(|e| format!("{}：{e}", path.display()))?;
            sets.push((name, items));
        }
    }
    Ok(sets)
}

/// 學習模式（`--learning`，plan §5.2）的結果。
#[derive(Debug)]
pub struct Learning {
    pub packs: Vec<LoadedPack>,
    /// 各語料集合，有個人集時接在最後。
    pub sets: Vec<LearningSet>,
}

impl Learning {
    /// 給人看的結果，Markdown 表；個人集只有數字。
    pub fn report(&self) -> String {
        report::render_learning(self)
    }
}

/// 學習模式（`--learning`）：每個集合照檔案的順序模擬打每一題，比學習關（每題空白的
/// `UserModel`）與學習開（整個集合共用一個 `UserModel`，打完一題學一題）的修正次數。
/// 只報不擋，不讀也不寫 baseline。
pub fn run_learning(root: &Path) -> Result<Learning, String> {
    let LoadedEngine { engine, packs, .. } = load_engine(root, &[], None)?;
    let mut sets: Vec<LearningSet> = load_corpora(root)?
        .into_iter()
        .map(|(name, items)| score::learning(&engine, &name, &items, false))
        .collect();
    for (name, items) in load_context_sets(root)? {
        let name = format!("{name}{}", CONTEXT_SUFFIXES[0]);
        sets.push(score::learning(&engine, &name, &items, false));
    }
    if let Personal::Ran(items) = load_personal(root)? {
        sets.push(score::learning(&engine, PERSONAL_SET, &items, true));
    }
    Ok(Learning { packs, sets })
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// [`load_engine`] 建好的 `Engine` 與讀了哪些資料包。
#[derive(Debug)]
pub struct LoadedEngine {
    pub engine: Engine,
    pub packs: Vec<LoadedPack>,
    /// 詞庫的一份複本（方案沒指名時是 `None`），dev 模式挑詞用。
    pub lexicon: Option<Lexicon>,
    /// 解開每份資料包加建 `Engine` 的毫秒數，不含讀檔與算 sha256（報告的「載入」）。
    pub load_ms: f64,
}

/// 讀 [`QUICK_PACK`] 與方案指名的詞庫、語言模型、詞級 trigram 資料包，掛上 [`ENGLISH_PACK`]、
/// [`EMOJI_PACK`]，建 `Engine`；`overrides` 蓋掉方案的打分參數，`lexicon` 是換掉方案指名的那份詞庫的
/// 另一份 id（`--lexicon`）。
pub fn load_engine(
    root: &Path,
    overrides: &[(&str, &str)],
    lexicon: Option<&str>,
) -> Result<LoadedEngine, String> {
    // 解開、建 `Engine` 的毫秒數加總（[`timed`]）。
    let mut load_ms = 0.0;
    let load = |rel: String| -> Result<(Vec<u8>, LoadedPack), String> {
        let path = root.join(&rel);
        let bytes = fs::read(&path).map_err(|e| {
            format!(
                "讀不到資料包 {}：{e}；先跑 `cargo xtask data`",
                path.display()
            )
        })?;
        let sha256 = sha256_hex(&bytes);
        Ok((bytes, LoadedPack { path: rel, sha256 }))
    };
    let (bytes, loaded) = load(QUICK_PACK.into())?;
    let mut pack = timed(&mut load_ms, || SchemePack::from_bytes(&bytes))
        .map_err(|e| format!("{QUICK_PACK}：{e}"))?;
    for (key, value) in overrides {
        override_ranking(&mut pack.scheme.ranking, key, value)?;
    }
    // 蓋過的值照方案檔的規則驗（`Ranking::check`）；不先驗的話 `Engine::new` 會說成資料包壞了。
    if !overrides.is_empty() {
        pack.scheme
            .ranking
            .check()
            .map_err(|e| format!("--set 之後的打分參數不合用：{e}"))?;
    }
    let mut packs = vec![loaded];
    // 方案指名的另一份資料包：`build/packs/<id>/<id>.pack`，解不開時報它的路徑。
    let mut named = |id: &Option<String>| -> Result<Option<(Vec<u8>, String)>, String> {
        let Some(id) = id else { return Ok(None) };
        let (bytes, loaded) = load(format!("{PACKS}/{id}/{id}.pack"))?;
        let path = loaded.path.clone();
        packs.push(loaded);
        Ok(Some((bytes, path)))
    };
    let lexicon_id = lexicon
        .map(String::from)
        .or(pack.scheme.packs.lexicon.clone());
    fn decode<P: Pack>(load_ms: &mut f64, bytes: &[u8], path: &str) -> Result<P, String> {
        timed(load_ms, || P::from_bytes(bytes)).map_err(|e| format!("{path}：{e}"))
    }
    let lexicon: Option<LexiconPack> = named(&lexicon_id)?
        .map(|(bytes, path)| decode(&mut load_ms, &bytes, &path))
        .transpose()?;
    let language_model: Option<LanguageModelPack> = named(&pack.scheme.packs.language_model)?
        .map(|(bytes, path)| decode(&mut load_ms, &bytes, &path))
        .transpose()?;
    let word_trigram: Option<WordTrigramPack> = named(&pack.scheme.packs.word_trigram)?
        .map(|(bytes, path)| decode(&mut load_ms, &bytes, &path))
        .transpose()?;
    let copy = lexicon.as_ref().map(|l| l.lexicon.clone());
    let (bytes, loaded) = load(ENGLISH_PACK.into())?;
    let english: EnglishPack = decode(&mut load_ms, &bytes, ENGLISH_PACK)?;
    packs.push(loaded);
    let (bytes, loaded) = load(EMOJI_PACK.into())?;
    let emoji: EmojiPack = decode(&mut load_ms, &bytes, EMOJI_PACK)?;
    packs.push(loaded);
    let engine = timed(&mut load_ms, || {
        let engine = Engine::new(pack, lexicon, language_model)?
            .with_english(english)
            .with_emoji(emoji);
        Ok::<_, qim_core::PackError>(match word_trigram {
            Some(pack) => engine.with_word_trigram(pack),
            None => engine,
        })
    })
    .map_err(|e| format!("{QUICK_PACK}：{e}"))?;
    Ok(LoadedEngine {
        engine,
        packs,
        lexicon: copy,
        load_ms,
    })
}

/// 跑 `f`，把花的毫秒數加進 `total`。
fn timed<T>(total: &mut f64, f: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let value = f();
    *total += start.elapsed().as_secs_f64() * 1000.0;
    value
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
