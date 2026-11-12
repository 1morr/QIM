//! 容錯實驗（M4 票 09）：四種修正（漏碼、多碼、兩碼對調、鄰鍵替換）各自與合起來，量打錯碼的題的首選、
//! 修正次數、「原樣正確卻被改錯」的比例與每鍵延遲，對照 `.scratch/m4/spec.md`〈容錯採用門檻〉
//! 〈每鍵延遲上限〉。容錯由 `Engine::with_tolerance` 打開（核心預設關）；題目與量法同首選字基準
//! （`qim_bench::score_set`）。
//!
//! 用法（repo 根目錄先跑 `cargo xtask data`），`cargo run --release -- <指令> [選項]`：
//! - `dev`：只看 dev 切分，每種修正掃一遍懲罰，照 README 的規則挑懲罰與加入的順序，印出 `test` 要的參數。
//! - `test --omission X --insertion X --transposition X --substitution X --order a,b,c,d`：設定定案後在
//!   test 切分照 README 的程序跑一次，逐條對照門檻。
//! - `combo [同 test 的懲罰]`：只在 dev 打一種設定（打錯碼的集合與沒打錯的集合），改機制時對照用。
//! - `ceiling`：dev 的打錯的題若錯碼全部改回來（照原本的碼打）有幾題答對，其中幾題的錯都落在原樣讀法的
//!   多字段、每段最多一處（容錯的規則救得到的上限）。
//! - `latency [同 test 的懲罰]`：原生每鍵延遲，容錯關與開交替量，空白與學過各一次；另量含萬用鍵 `z` 的輸入。
//!
//! `--root <repo 根目錄>` 預設是 `../../..`。

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Instant;

use qim_bench::top_char;
use qim_bench::{
    Item, SetResult, TouchSurface, learned_model, lefts_to_first_segment, load_corpora,
    load_engine, load_typo_sets, parse_context, parse_corpus, parse_fixed, parse_touch, parse_typo,
    percentile, physical_key, score_set,
};
use qim_core::{Engine, Key, Session, Snapshot, Tolerance, UserModel};

/// 口語的兩個語料：打錯碼的集合只有它們（票 08）。
const COLLOQUIAL: [&str; 2] = ["common-voice-yue", "hkcancor"];
/// 每種修正掃的懲罰（ln 機率）。鄰鍵替換另扣 `ln(按下的 ÷ 換成的)`（實體鍵盤的鄰鍵約 3.7）。
const GRID: [f64; 11] = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 10.0, 12.0];
/// 〈容錯採用門檻〉的數字。
const FALSE_CORRECTION_LIMIT: f64 = 0.010;
const TYPO_ACCURACY: f64 = 0.10;
const TOUCH_ACCURACY: f64 = 0.06;
const FEWER_CORRECTIONS: f64 = 0.10;
const KIND_GAIN: f64 = 0.010;
const P_LIMIT: f64 = 0.05;
const CLEAN_WRONG_LIMIT: f64 = 0.005;
const CLEAN_CORRECTION_LIMIT: f64 = 0.01;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Omission,
    Insertion,
    Transposition,
    Substitution,
}

const KINDS: [Kind; 4] = [
    Kind::Omission,
    Kind::Insertion,
    Kind::Transposition,
    Kind::Substitution,
];

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Kind::Omission => "omission",
            Kind::Insertion => "insertion",
            Kind::Transposition => "transposition",
            Kind::Substitution => "substitution",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Kind::Omission => "漏碼",
            Kind::Insertion => "多碼",
            Kind::Transposition => "兩碼對調",
            Kind::Substitution => "鄰鍵替換",
        }
    }

    fn parse(name: &str) -> Kind {
        KINDS
            .into_iter()
            .find(|k| k.name() == name)
            .unwrap_or_else(|| panic!("沒有這種修正：{name}"))
    }

    /// 〈容錯採用門檻〉：鄰鍵替換看兩種集合，其餘看含打錯碼。
    fn judged_on(self) -> &'static [Typo] {
        match self {
            Kind::Substitution => &[Typo::Physical, Typo::Touch],
            _ => &[Typo::Physical],
        }
    }
}

/// 打錯碼的兩種集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Typo {
    Physical,
    Touch,
}

impl Typo {
    fn label(self) -> &'static str {
        match self {
            Typo::Physical => "含打錯碼",
            Typo::Touch => "觸控模擬",
        }
    }

    fn accuracy_limit(self) -> f64 {
        match self {
            Typo::Physical => TYPO_ACCURACY,
            Typo::Touch => TOUCH_ACCURACY,
        }
    }
}

/// 各種修正的懲罰；沒有的是關。
type Config = BTreeMap<Kind, f64>;

fn tolerance(config: &Config) -> Tolerance {
    let get = |k| config.get(&k).copied();
    Tolerance {
        omission: get(Kind::Omission),
        insertion: get(Kind::Insertion),
        transposition: get(Kind::Transposition),
        substitution: get(Kind::Substitution),
    }
}

fn describe(config: &Config) -> String {
    if config.is_empty() {
        return "關".into();
    }
    config
        .iter()
        .map(|(k, p)| format!("{} {p}", k.label()))
        .collect::<Vec<_>>()
        .join("、")
}

/// 一批題的名字與題目。
struct Set {
    name: String,
    items: Vec<Item>,
}

/// 一個切分要打的題：兩種打錯碼的集合（兩個語料各一批）與沒打錯的集合（維基、口語、有前文）。
struct Split {
    typo: Vec<(Typo, Set)>,
    clean: Vec<Set>,
    fixed: Option<Set>,
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("讀不到 {}：{e}；先跑 cargo xtask data", path.display()))
}

fn surface(root: &Path) -> TouchSurface {
    TouchSurface::from_json(
        &read(&root.join(qim_bench::TOUCH_LAYOUT)),
        qim_bench::TOUCH_MM_PER_UNIT,
    )
    .unwrap()
}

/// 有前文的題打兩次：給前文、不給（同基準）。
fn with_and_without_context(name: &str, items: Vec<Item>) -> [Set; 2] {
    let without = items
        .iter()
        .map(|i| Item {
            context: None,
            ..i.clone()
        })
        .collect();
    let [given, not_given] = qim_bench::CONTEXT_SUFFIXES;
    [
        Set {
            name: format!("{name}{given}"),
            items,
        },
        Set {
            name: format!("{name}{not_given}"),
            items: without,
        },
    ]
}

fn dev_split(root: &Path) -> Split {
    let surface = surface(root);
    let mut typo = Vec::new();
    for name in COLLOQUIAL {
        let items = parse_typo(&read(&root.join(qim_bench::dev_typo_items(name)))).unwrap();
        typo.push((
            Typo::Physical,
            Set {
                name: format!("{name}{}", qim_bench::TYPO_SUFFIX),
                items,
            },
        ));
        let text = read(&root.join(qim_bench::dev_touch_items(name)));
        typo.push((
            Typo::Touch,
            Set {
                name: format!("{name}{}", qim_bench::TOUCH_SUFFIX),
                items: parse_touch(&text, &surface).unwrap(),
            },
        ));
    }
    let mut clean = Vec::new();
    let corpora: Vec<String> = load_corpora(root)
        .unwrap()
        .into_iter()
        .map(|(n, _)| n)
        .collect();
    for name in &corpora {
        let items = parse_corpus(&read(&root.join(qim_bench::dev_items(name)))).unwrap();
        clean.push(Set {
            name: name.clone(),
            items,
        });
    }
    for name in &corpora {
        let path = root.join(qim_bench::dev_context_items(name));
        if path.exists() {
            let items = parse_context(&read(&path)).unwrap();
            clean.extend(with_and_without_context(name, items));
        }
    }
    Split {
        typo,
        clean,
        fixed: None,
    }
}

fn test_split(root: &Path) -> Split {
    let typo = load_typo_sets(root)
        .unwrap()
        .into_iter()
        .map(|(name, items)| {
            let kind = if name.ends_with(qim_bench::TOUCH_SUFFIX) {
                Typo::Touch
            } else {
                Typo::Physical
            };
            (kind, Set { name, items })
        })
        .collect();
    let mut clean: Vec<Set> = load_corpora(root)
        .unwrap()
        .into_iter()
        .map(|(name, items)| Set { name, items })
        .collect();
    for dir in qim_bench::CONTEXT_DIRS {
        let Ok(entries) = std::fs::read_dir(root.join(dir)) else {
            continue;
        };
        let mut files: Vec<PathBuf> = entries
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|e| e == "tsv"))
            .collect();
        files.sort();
        for path in files {
            let name = path.file_stem().unwrap().to_str().unwrap().to_string();
            clean.extend(with_and_without_context(
                &name,
                parse_context(&read(&path)).unwrap(),
            ));
        }
    }
    let fixed = Set {
        name: "fixed".into(),
        items: parse_fixed(&read(&root.join(qim_bench::FIXED))).unwrap(),
    };
    Split {
        typo,
        clean,
        fixed: Some(fixed),
    }
}

/// 同 `score_set`，題目分給各核心並行打（每題本來就互不影響）。
fn score(engine: &Engine, set: &Set) -> SetResult {
    let threads = thread::available_parallelism().map_or(4, |n| n.get());
    let chunk = set.items.len().div_ceil(threads).max(1);
    let parts: Vec<SetResult> = thread::scope(|s| {
        let handles: Vec<_> = set
            .items
            .chunks(chunk)
            .map(|items| s.spawn(move || score_set(engine, &set.name, items, false)))
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut merged = SetResult {
        name: set.name.clone(),
        private: false,
        items: Vec::new(),
        failures: Vec::new(),
    };
    for part in parts {
        merged.items.extend(part.items);
        merged.failures.extend(part.failures);
    }
    merged
}

/// 打錯碼的集合在容錯關（`off`）與開（`on`）之下的比較，可以幾批相加（兩個語料合計）。
#[derive(Debug, Clone, Copy, Default)]
struct TypoCompare {
    mistyped: usize,
    mistyped_correct_off: usize,
    mistyped_correct_on: usize,
    /// 打錯的題翻好、翻壞（McNemar）。
    better: usize,
    worse: usize,
    corrections_off: usize,
    corrections_on: usize,
    /// 沒打錯、容錯關時首選正確的題（票 08 的分母），與其中容錯開時首選不正確的。
    literal_correct: usize,
    false_corrections: usize,
}

impl TypoCompare {
    fn of(off: &SetResult, on: &SetResult) -> Self {
        let mut c = TypoCompare::default();
        for (a, b) in off.items.iter().zip(&on.items) {
            assert_eq!(a.line, b.line);
            let clean = a.typed.expect("打錯碼的題").clean;
            c.corrections_off += a.correction_count;
            c.corrections_on += b.correction_count;
            if clean {
                if a.correct {
                    c.literal_correct += 1;
                    c.false_corrections += usize::from(!b.correct);
                }
            } else {
                c.mistyped += 1;
                c.mistyped_correct_off += usize::from(a.correct);
                c.mistyped_correct_on += usize::from(b.correct);
                c.better += usize::from(!a.correct && b.correct);
                c.worse += usize::from(a.correct && !b.correct);
            }
        }
        c
    }

    fn add(mut self, o: TypoCompare) -> Self {
        self.mistyped += o.mistyped;
        self.mistyped_correct_off += o.mistyped_correct_off;
        self.mistyped_correct_on += o.mistyped_correct_on;
        self.better += o.better;
        self.worse += o.worse;
        self.corrections_off += o.corrections_off;
        self.corrections_on += o.corrections_on;
        self.literal_correct += o.literal_correct;
        self.false_corrections += o.false_corrections;
        self
    }

    fn accuracy_on(&self) -> f64 {
        ratio(self.mistyped_correct_on, self.mistyped)
    }

    fn correction_change(&self) -> f64 {
        self.corrections_on as f64 / self.corrections_off as f64 - 1.0
    }

    fn false_correction_rate(&self) -> f64 {
        ratio(self.false_corrections, self.literal_correct)
    }

    fn p(&self) -> f64 {
        mcnemar(self.better, self.worse)
    }
}

fn ratio(a: usize, b: usize) -> f64 {
    if b == 0 { 0.0 } else { a as f64 / b as f64 }
}

/// McNemar 精確檢定（雙尾）：翻好 `b`、翻壞 `c`，在 b + c 次、p = 0.5 的二項分布下取較小一邊的機率 × 2。
fn mcnemar(b: usize, c: usize) -> f64 {
    let n = b + c;
    if n == 0 {
        return 1.0;
    }
    let k = b.min(c);
    // ln C(n, i) 逐項累加，免得 n 大時溢位。
    let ln_half_n = n as f64 * 0.5f64.ln();
    let mut ln_choose = 0.0;
    let mut tail = 0.0;
    for i in 0..=k {
        if i > 0 {
            ln_choose += ((n - i + 1) as f64).ln() - (i as f64).ln();
        }
        tail += (ln_choose + ln_half_n).exp();
    }
    (2.0 * tail).min(1.0)
}

/// 沒打錯的集合在容錯關與開之下：答錯數、修正次數。
struct CleanCompare {
    name: String,
    total: usize,
    wrong_off: usize,
    wrong_on: usize,
    corrections_off: usize,
    corrections_on: usize,
}

impl CleanCompare {
    fn of(off: &SetResult, on: &SetResult) -> Self {
        CleanCompare {
            name: off.name.clone(),
            total: off.total(),
            wrong_off: off.wrong(),
            wrong_on: on.wrong(),
            corrections_off: off.correction_count(),
            corrections_on: on.correction_count(),
        }
    }

    fn passes(&self) -> bool {
        let wrong = self.wrong_on as f64 - self.wrong_off as f64;
        let corrections = self.corrections_on as f64 - self.corrections_off as f64;
        wrong <= CLEAN_WRONG_LIMIT * self.total as f64
            && corrections <= CLEAN_CORRECTION_LIMIT * self.corrections_off as f64
    }
}

/// 容錯關的結果先打好，各設定都和它比。
struct Runner<'a> {
    engine: Engine,
    split: &'a Split,
    off_typo: Vec<SetResult>,
    off_clean: Option<Vec<SetResult>>,
}

impl<'a> Runner<'a> {
    fn new(engine: Engine, split: &'a Split) -> Self {
        let off_typo = split.typo.iter().map(|(_, s)| score(&engine, s)).collect();
        Runner {
            engine,
            split,
            off_typo,
            off_clean: None,
        }
    }

    /// 打錯碼的集合依種類合計（兩個語料合計）的比較。
    fn typo(&self, config: &Config) -> BTreeMap<&'static str, TypoCompare> {
        let engine = self.engine.clone().with_tolerance(tolerance(config));
        let mut sums: BTreeMap<&'static str, TypoCompare> = BTreeMap::new();
        for ((kind, set), off) in self.split.typo.iter().zip(&self.off_typo) {
            let on = score(&engine, set);
            let entry = sums.entry(kind.label()).or_default();
            *entry = entry.add(TypoCompare::of(off, &on));
        }
        sums
    }

    fn clean(&mut self, config: &Config) -> Vec<CleanCompare> {
        let off_clean = self.off_clean.get_or_insert_with(|| {
            self.split
                .clean
                .iter()
                .map(|s| score(&self.engine, s))
                .collect()
        });
        let engine = self.engine.clone().with_tolerance(tolerance(config));
        self.split
            .clean
            .iter()
            .zip(off_clean.iter())
            .map(|(set, off)| CleanCompare::of(off, &score(&engine, set)))
            .collect()
    }
}

fn typo_row(label: &str, sums: &BTreeMap<&'static str, TypoCompare>) -> String {
    let mut row = format!("| {label} ");
    for typo in [Typo::Physical, Typo::Touch] {
        let c = sums[typo.label()];
        let _ = write!(
            row,
            "| {}/{} ({:.1}%) | {:+.1}% | {}/{} ({:.2}%) | {}/{} p={:.3} ",
            c.mistyped_correct_on,
            c.mistyped,
            100.0 * c.accuracy_on(),
            100.0 * c.correction_change(),
            c.false_corrections,
            c.literal_correct,
            100.0 * c.false_correction_rate(),
            c.better,
            c.worse,
            c.p()
        );
    }
    row + "|"
}

const TYPO_HEADER: &str = "| 設定 | 含打錯碼：打錯的題首選 | 修正次數 | 改錯 | 翻好/翻壞 | 觸控：打錯的題首選 | 修正次數 | 改錯 | 翻好/翻壞 |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- |";

fn clean_table(rows: &[CleanCompare]) -> String {
    let mut out = String::from(
        "| 集合 | 題數 | 答錯（關 → 開） | 修正次數（關 → 開） | 不退步 |\n| --- | ---: | --- | --- | --- |\n",
    );
    for r in rows {
        let _ = writeln!(
            out,
            "| {} | {} | {} → {} | {} → {} | {} |",
            r.name,
            r.total,
            r.wrong_off,
            r.wrong_on,
            r.corrections_off,
            r.corrections_on,
            if r.passes() { "過" } else { "**沒過**" }
        );
    }
    out
}

/// 一種修正的挑法（README〈dev 怎麼挑〉）：兩種打錯碼的集合的改錯都在上限內的懲罰裡，判斷用的集合
/// 打錯的題首選答對最多的；一樣多取懲罰大的（偏向相信使用者）。
fn pick(kind: Kind, rows: &[(f64, BTreeMap<&'static str, TypoCompare>)]) -> Option<f64> {
    rows.iter()
        .filter(|(_, sums)| {
            sums.values()
                .all(|c| c.false_correction_rate() <= FALSE_CORRECTION_LIMIT)
        })
        .max_by(|a, b| {
            let gain = |sums: &BTreeMap<&'static str, TypoCompare>| -> usize {
                kind.judged_on()
                    .iter()
                    .map(|t| sums[t.label()].mistyped_correct_on)
                    .sum()
            };
            gain(&a.1).cmp(&gain(&b.1)).then(a.0.total_cmp(&b.0))
        })
        .map(|(p, _)| *p)
}

fn dev(root: &Path) {
    let split = dev_split(root);
    let mut runner = Runner::new(load(root), &split);
    println!("## dev：各種修正單獨開\n\n{TYPO_HEADER}");
    println!("{}", typo_row("關", &runner.typo(&Config::new())));
    let mut chosen = Config::new();
    let mut gains: Vec<(Kind, usize)> = Vec::new();
    for kind in KINDS {
        let mut rows = Vec::new();
        for penalty in GRID {
            let sums = runner.typo(&Config::from([(kind, penalty)]));
            println!(
                "{}",
                typo_row(&format!("{} {penalty}", kind.label()), &sums)
            );
            rows.push((penalty, sums));
        }
        if let Some(p) = pick(kind, &rows) {
            let sums = &rows.iter().find(|(q, _)| *q == p).unwrap().1;
            let gain = kind
                .judged_on()
                .iter()
                .map(|t| sums[t.label()].mistyped_correct_on - sums[t.label()].mistyped_correct_off)
                .sum();
            chosen.insert(kind, p);
            gains.push((kind, gain));
        }
    }
    gains.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    println!("\n挑到的懲罰：{}", describe(&chosen));
    let order: Vec<Kind> = gains.iter().map(|(k, _)| *k).collect();
    println!(
        "加入順序（dev 上單獨開時判斷用的集合多答對的題數）：{}",
        gains
            .iter()
            .map(|(k, g)| format!("{} +{g}", k.label()))
            .collect::<Vec<_>>()
            .join("、")
    );
    println!("\n## dev：依序加入\n\n{TYPO_HEADER}");
    let mut config = Config::new();
    for kind in &order {
        config.insert(*kind, chosen[kind]);
        println!("{}", typo_row(&describe(&config), &runner.typo(&config)));
    }
    println!("\n## dev：全部加入時沒打錯的集合\n");
    println!("{}", clean_table(&runner.clean(&config)));
    let args: Vec<String> = chosen
        .iter()
        .map(|(k, p)| format!("--{} {p}", k.name()))
        .collect();
    let order: Vec<&str> = order.iter().map(|k| k.name()).collect();
    println!(
        "test 用：`cargo run --release -- test {} --order {}`",
        args.join(" "),
        order.join(",")
    );
}

fn combo(root: &Path, config: &Config) {
    let split = dev_split(root);
    let mut runner = Runner::new(load(root), &split);
    println!("## dev：{}\n\n{TYPO_HEADER}", describe(config));
    println!("{}", typo_row(&describe(config), &runner.typo(config)));
    println!("\n{}", clean_table(&runner.clean(config)));
}

/// 一種修正加進已採用的組合的判斷（〈容錯採用門檻〉）：判斷用的每種集合打錯的題首選多 1.0 點以上、
/// 翻好／翻壞（和已採用的組合比）p < 0.05。
fn kind_passes(
    kind: Kind,
    before: &BTreeMap<&'static str, TypoCompare>,
    after: &BTreeMap<&'static str, TypoCompare>,
    split: &Split,
    before_on: &[SetResult],
    after_on: &[SetResult],
) -> (bool, String) {
    let mut ok = true;
    let mut notes = Vec::new();
    for typo in kind.judged_on() {
        let gain = after[typo.label()].accuracy_on() - before[typo.label()].accuracy_on();
        // 翻好／翻壞是和已採用的組合比（不是和容錯關比）。
        let (mut better, mut worse) = (0, 0);
        for (((t, _), a), b) in split.typo.iter().zip(before_on).zip(after_on) {
            if t != typo {
                continue;
            }
            for (x, y) in a.items.iter().zip(&b.items) {
                if !x.typed.unwrap().clean {
                    better += usize::from(!x.correct && y.correct);
                    worse += usize::from(x.correct && !y.correct);
                }
            }
        }
        let p = mcnemar(better, worse);
        let pass = gain >= KIND_GAIN && p < P_LIMIT;
        ok &= pass;
        notes.push(format!(
            "{}：{:+.1} 點、翻好/翻壞 {better}/{worse}、p={p:.4}（{}）",
            typo.label(),
            100.0 * gain,
            if pass { "過" } else { "沒過" }
        ));
    }
    (ok, notes.join("；"))
}

fn test(root: &Path, chosen: &Config, order: &[Kind]) {
    let split = test_split(root);
    let mut runner = Runner::new(load(root), &split);
    let fixed = split.fixed.as_ref().unwrap();
    println!("## test（設定：{}；順序：{order:?}）\n", describe(chosen));
    println!("{TYPO_HEADER}");
    let off = runner.typo(&Config::new());
    println!("{}", typo_row("關", &off));
    let mut singles = Vec::new();
    for kind in order {
        let config = Config::from([(*kind, chosen[kind])]);
        let sums = runner.typo(&config);
        println!("{}", typo_row(&format!("只開{}", kind.label()), &sums));
        singles.push(sums);
    }
    // 逐種加入：每批題容錯開時的結果另留一份，比翻好／翻壞用。
    let on_results = |config: &Config| -> Vec<SetResult> {
        let engine = runner.engine.clone().with_tolerance(tolerance(config));
        split.typo.iter().map(|(_, s)| score(&engine, s)).collect()
    };
    let mut adopted = Config::new();
    let mut adopted_sums = off.clone();
    let mut adopted_on = runner.off_typo.clone();
    let mut decisions = Vec::new();
    for kind in order {
        let mut config = adopted.clone();
        config.insert(*kind, chosen[kind]);
        let on = on_results(&config);
        let mut sums: BTreeMap<&'static str, TypoCompare> = BTreeMap::new();
        for (((t, _), off), on) in split.typo.iter().zip(&runner.off_typo).zip(&on) {
            let entry = sums.entry(t.label()).or_default();
            *entry = entry.add(TypoCompare::of(off, on));
        }
        println!(
            "{}",
            typo_row(&format!("{}（逐種加入）", describe(&config)), &sums)
        );
        let (pass, note) = kind_passes(*kind, &adopted_sums, &sums, &split, &adopted_on, &on);
        decisions.push(format!(
            "- {}：{}。{note}",
            kind.label(),
            if pass { "採用" } else { "不採用" }
        ));
        if pass {
            adopted = config;
            adopted_sums = sums;
            adopted_on = on;
        }
    }
    println!("\n### 逐種判斷\n\n{}", decisions.join("\n"));
    println!("\n### 採用的組合：{}\n", describe(&adopted));
    for typo in [Typo::Physical, Typo::Touch] {
        let c = adopted_sums[typo.label()];
        println!(
            "- {}：打錯的題首選 {:.1}%（門檻 ≥ {:.0}%）、修正次數 {:+.1}%（門檻 ≤ −{:.0}%）、McNemar p={:.4}、改錯 {:.2}%（上限 {:.1}%）",
            typo.label(),
            100.0 * c.accuracy_on(),
            100.0 * typo.accuracy_limit(),
            100.0 * c.correction_change(),
            100.0 * FEWER_CORRECTIONS,
            c.p(),
            100.0 * c.false_correction_rate(),
            100.0 * FALSE_CORRECTION_LIMIT
        );
    }
    if adopted.is_empty() {
        return;
    }
    println!("\n### 沒打錯的集合（採用的組合）\n");
    println!("{}", clean_table(&runner.clean(&adopted)));
    let engine = runner.engine.clone().with_tolerance(tolerance(&adopted));
    let result = score(&engine, fixed);
    println!(
        "固定題：{}/{} 過{}",
        result.correct(),
        result.total(),
        result
            .failures
            .iter()
            .map(|f| format!("\n  - {f}"))
            .collect::<String>()
    );
}

fn load(root: &Path) -> Engine {
    load_engine(root, &[], None).unwrap().engine
}

/// 照殼送實體鍵的按鍵機率打 `input`，量每一鍵與「選開頭一段」（同 `qim_bench::native_timing_with`，
/// 只是按鍵換成 `physical_key`）。
fn time_input(engine: &Engine, user: &mut UserModel, input: &str) -> (Vec<f64>, Option<f64>) {
    let mut session = Session::default();
    let mut last = Snapshot::default();
    let keys = input
        .chars()
        .map(|c| {
            let key = physical_key(c);
            let start = Instant::now();
            last = std::hint::black_box(session.handle(engine, user, key));
            start.elapsed().as_secs_f64() * 1000.0
        })
        .collect();
    let select = lefts_to_first_segment(&last).map(|lefts| {
        for _ in 0..lefts {
            session.handle(engine, user, Key::Left);
        }
        let start = Instant::now();
        std::hint::black_box(session.select(engine, user, 0));
        start.elapsed().as_secs_f64() * 1000.0
    });
    (keys, select)
}

/// 一組輸入的 p95：全部、第 20 碼起、選開頭一段（毫秒）。
fn p95s(per_key: &[Vec<f64>], selects: &[f64]) -> [f64; 3] {
    let all: Vec<f64> = per_key.concat();
    let long: Vec<f64> = per_key
        .iter()
        .filter_map(|k| k.get(qim_bench::LONG_INPUT_KEYS..))
        .flatten()
        .copied()
        .collect();
    [
        percentile(&all, 95).unwrap_or(0.0),
        percentile(&long, 95).unwrap_or(0.0),
        percentile(selects, 95).unwrap_or(0.0),
    ]
}

/// 輸入碼的第一個碼換成萬用鍵 `z`（萬用鍵實驗的「首碼換成 z」，這裡每個輸入換一處）。
fn with_wildcard(input: &str) -> String {
    let mut out = String::new();
    let mut done = false;
    for c in input.chars() {
        if !done && c.is_ascii_lowercase() {
            out.push('z');
            done = true;
        } else {
            out.push(c);
        }
    }
    out
}

fn load_average() -> String {
    std::process::Command::new("uptime")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn latency(root: &Path, chosen: &Config) {
    let engine = load(root);
    let on = engine.clone().with_tolerance(tolerance(chosen));
    let fixed = parse_fixed(&read(&root.join(qim_bench::FIXED))).unwrap();
    let corpora = load_corpora(root).unwrap();
    let inputs: Vec<String> = fixed
        .iter()
        .chain(corpora.iter().flat_map(|(_, items)| items))
        .map(Item::input)
        .collect();
    let wildcard: Vec<String> = inputs.iter().map(|i| with_wildcard(i)).collect();
    let mut context_items = Vec::new();
    for dir in qim_bench::CONTEXT_DIRS {
        if let Ok(entries) = std::fs::read_dir(root.join(dir)) {
            let mut files: Vec<PathBuf> = entries.map(|e| e.unwrap().path()).collect();
            files.sort();
            for path in files
                .iter()
                .filter(|p| p.extension().is_some_and(|e| e == "tsv"))
            {
                context_items.extend(parse_context(&read(path)).unwrap());
            }
        }
    }
    let learned = learned_model(
        &engine,
        corpora
            .iter()
            .flat_map(|(_, items)| items)
            .chain(&context_items),
    );
    println!("## 每鍵延遲（原生 release；容錯：{}）\n", describe(chosen));
    println!("開始時：{}\n", load_average());
    println!(
        "| 學習資料 | 輸入 | 容錯 | 全部 p95 | 第 20 碼起 p95 | 選開頭一段 p95 |\n| --- | --- | --- | ---: | ---: | ---: |"
    );
    for (user_label, user) in [("空白", UserModel::default()), ("學過", learned)] {
        for (input_label, inputs) in [("基準的輸入", &inputs), ("首碼換成 z", &wildcard)] {
            // 關與開逐題交替，機器負載的起伏兩邊一起吃。
            let mut runs: [(Vec<Vec<f64>>, Vec<f64>); 2] = Default::default();
            for input in inputs.iter() {
                for (i, e) in [&engine, &on].into_iter().enumerate() {
                    let mut user = user.clone();
                    let (keys, select) = time_input(e, &mut user, input);
                    runs[i].0.push(keys);
                    runs[i].1.extend(select);
                }
            }
            for (i, label) in ["關", "開"].into_iter().enumerate() {
                let [all, long, select] = p95s(&runs[i].0, &runs[i].1);
                println!(
                    "| {user_label} | {input_label} | {label} | {:.0} µs | {:.0} µs | {:.2} ms |",
                    all * 1000.0,
                    long * 1000.0,
                    select
                );
            }
        }
    }
    println!("\n結束時：{}", load_average());
}

/// 打錯的一題：哪幾個字（`units` 的位置）的鍵打錯了。
fn wrong_units(item: &Item) -> Vec<usize> {
    let typed = item.typed.as_ref().expect("打錯碼的題");
    item.units
        .iter()
        .zip(typed)
        .enumerate()
        .filter(|(_, (unit, keys))| !keys.iter().map(top_char).eq(unit.code.chars().map(Some)))
        .map(|(i, _)| i)
        .collect()
}

/// 照原本的碼逐鍵打 `item`（不給前文、空白學習資料）：首選與首選的各段文字。
fn literal_reading(engine: &Engine, item: &Item) -> (String, Vec<String>) {
    let mut session = Session::default();
    let mut user = UserModel::default();
    let mut snapshot = Snapshot::default();
    for c in item.input().chars() {
        snapshot = session.handle(engine, &mut user, physical_key(c));
    }
    let top = snapshot
        .candidates
        .first()
        .map(|c| c.text.clone())
        .unwrap_or_default();
    let segments = snapshot
        .composition
        .segments
        .iter()
        .map(|s| s.text.clone())
        .collect();
    (top, segments)
}

fn ceiling(root: &Path) {
    let engine = load(root);
    let split = dev_split(root);
    println!("## 容錯救得到的上限（dev，容錯關）\n");
    println!(
        "| 集合 | 打錯的題 | 照原碼打首選對 | 其中錯都在多字段、每段一處 | 其中只錯一字 |\n| --- | ---: | ---: | ---: | ---: |"
    );
    for (kind, set) in &split.typo {
        let mistyped: Vec<&Item> = set
            .items
            .iter()
            .filter(|i| !wrong_units(i).is_empty())
            .collect();
        let (mut right, mut in_words, mut single) = (0, 0, 0);
        for item in &mistyped {
            let (top, segments) = literal_reading(&engine, item);
            if top != item.expected() {
                continue;
            }
            right += 1;
            // 每個字落在首選的第幾段、那段幾個字。
            let mut segment_of = Vec::new();
            for (s, text) in segments.iter().enumerate() {
                segment_of.extend(std::iter::repeat_n(
                    (s, text.chars().count()),
                    text.chars().count(),
                ));
            }
            let mut char_at = 0;
            let wrong = wrong_units(item);
            let mut hit: Vec<usize> = Vec::new();
            let mut ok = segment_of.len() == item.expected().chars().count();
            for (u, unit) in item.units.iter().enumerate() {
                let len = unit.text.chars().count();
                if wrong.contains(&u) {
                    match segment_of.get(char_at) {
                        Some(&(s, n)) if n >= 2 && !hit.contains(&s) => hit.push(s),
                        _ => ok = false,
                    }
                }
                char_at += len;
            }
            in_words += usize::from(ok);
            single += usize::from(ok && wrong.len() == 1);
        }
        println!(
            "| {}（{}） | {} | {right} ({:.1}%) | {in_words} ({:.1}%) | {single} |",
            set.name,
            kind.label(),
            mistyped.len(),
            100.0 * ratio(right, mistyped.len()),
            100.0 * ratio(in_words, mistyped.len()),
        );
    }
}

/// `--<修正> 懲罰` 與 `--order`、`--root`。
fn parse_args(args: &[String]) -> (PathBuf, Config, Vec<Kind>) {
    let mut root = PathBuf::from("../../..");
    let mut config = Config::new();
    let mut order = Vec::new();
    let mut it = args.iter();
    while let Some(flag) = it.next() {
        let value = it.next().unwrap_or_else(|| panic!("{flag} 要給值"));
        match flag.as_str() {
            "--root" => root = PathBuf::from(value),
            "--order" => order = value.split(',').map(Kind::parse).collect(),
            _ => {
                let kind = Kind::parse(flag.trim_start_matches("--"));
                config.insert(kind, value.parse().expect("懲罰要是數字"));
            }
        }
    }
    if order.is_empty() {
        order = config.keys().copied().collect();
    }
    (root, config, order)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((command, rest)) = args.split_first() else {
        eprintln!("用法：cargo run --release -- dev|test|latency [選項]（見 src/main.rs 開頭）");
        std::process::exit(2);
    };
    let (root, config, order) = parse_args(rest);
    let started = Instant::now();
    match command.as_str() {
        "dev" => dev(&root),
        "test" => test(&root, &config, &order),
        "latency" => latency(&root, &config),
        "ceiling" => ceiling(&root),
        "combo" => combo(&root, &config),
        other => panic!("沒有這個指令：{other}"),
    }
    eprintln!("花了 {:.0} 秒", started.elapsed().as_secs_f64());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcnemar_matches_known_values() {
        assert_eq!(mcnemar(0, 0), 1.0);
        // 10 比 0：2 × 0.5^10。
        assert!((mcnemar(10, 0) - 2.0 * 0.5f64.powi(10)).abs() < 1e-12);
        // 5 比 5：雙尾取到 1。
        assert_eq!(mcnemar(5, 5), 1.0);
        // 對稱。
        assert!((mcnemar(3, 12) - mcnemar(12, 3)).abs() < 1e-12);
        // 3 比 12：2 × P(X ≤ 3)，n = 15 → 2 × 576/32768。
        assert!((mcnemar(3, 12) - 2.0 * 576.0 / 32768.0).abs() < 1e-12);
    }

    #[test]
    fn the_first_code_becomes_the_wildcard() {
        assert_eq!(with_wildcard("ab'cd"), "zb'cd");
    }
}
