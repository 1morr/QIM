//! 模型階梯實驗的框架（M3 票 07，`.scratch/m3/spec.md`〈模型階梯實驗〉）：用核心打基準的題，
//! 換上要比的重排模型（[`Reranker`]），依集合報首選正確率、「最高到第 2 位」規則下的前 2 名命中率、
//! 修正次數；另匯出每題的重排要求（[`Query`]）給 Rust 以外的模型（Swift、雲端），讀回它們的
//! 排序（[`Orders`]）再算一次。
//!
//! 首選正確率與修正次數照 `qim_bench::score_set` 的定義算（基準同一份程式），這裡只多算重排之後的
//! 前 2 名與召回。量法與限制見 README。

use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use qim_bench::{Item, Personal, parse_context, parse_corpus};
use qim_core::{
    Engine, Key, LanguageModelPack, Pack, RERANK_LIMIT, ScoredPath, Session, UserModel,
};
use serde::{Deserialize, Serialize};

/// 前文長度（字）：0 是不給前文（同基準的 `·前文·不給`），50、200 取長前文的最後幾個字。
pub const CONTEXT_LENGTHS: [usize; 3] = [0, 50, 200];

/// 「答案在前 k 個」報哪些 k：知意三檔 5、9、18，青簡知微與 M2 票 11 的 6，加 1、2、3 與速成方案的
/// beam 20（`data/schemes/quick.toml`；lattice 每個位置最多留 beam 條，路徑不會更多）。
pub const RECALL_AT: [usize; 8] = [1, 2, 3, 5, 6, 9, 18, 20];

/// 重排模型看的路徑數 N（本票定，README〈N 與路徑長度〉）：口語 dev 上答案在前 9 條的比例比前 6 條
/// 多 3.7 點（每條約 1.2 點），9 條之後每條不到 0.7 點；也等於候選列一頁。
pub const DEFAULT_N: usize = 9;

/// 題目取自哪個切分：調參只用 dev，test 只在定案後跑一次（README〈調參與報數〉）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Split {
    Dev,
    Test,
}

impl Split {
    pub fn name(self) -> &'static str {
        match self {
            Split::Dev => "dev",
            Split::Test => "test",
        }
    }
}

/// 報告分組（spec〈模型階梯實驗〉的測試集）。「有前文」依語料分維基、口語：口語的長前文很短，
/// 合在一起時前文長度的差別幾乎都來自維基（README〈限制〉）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Group {
    Wiki,
    Colloquial,
    Personal,
    ContextWiki,
    ContextColloquial,
}

impl Group {
    pub fn label(self) -> &'static str {
        match self {
            Group::Wiki => "維基",
            Group::Colloquial => "口語",
            Group::Personal => "個人",
            Group::ContextWiki => "有前文·維基",
            Group::ContextColloquial => "有前文·口語",
        }
    }

    fn is_context(self) -> bool {
        matches!(self, Group::ContextWiki | Group::ContextColloquial)
    }
}

/// 一個集合：同一批題、同一個前文長度。
#[derive(Debug, Clone)]
pub struct Set {
    pub name: String,
    /// 題目取自哪個切分；外部模型的結果靠它和集合名、行號對回題目。
    pub split: Split,
    pub group: Group,
    /// 前文長度（字），只有「有前文」的集合不是 0。
    pub context_chars: usize,
    /// 個人集：輸出只有數字與行號（plan §5.2）。
    pub private: bool,
    pub items: Vec<Item>,
}

/// 讀切分 `split` 的各集合：維基、口語各語料，「有前文」的各語料各三種前文長度，test 另加個人集
/// （個人集沒有 dev 切分，README〈調參與報數〉）。`root` 是 repo 根目錄。
pub fn load_sets(root: &Path, split: Split) -> Result<Vec<Set>, String> {
    let mut sets = Vec::new();
    let corpora = qim_bench::load_corpora(root)?;
    // 語料在 `CORPUS_DIRS` 的第二個目錄（口語）就是口語集。
    let colloquial = |name: &str| {
        root.join(qim_bench::CORPUS_DIRS[1])
            .join(format!("{name}.tsv"))
            .exists()
    };
    for (name, test_items) in &corpora {
        let items = match split {
            Split::Test => test_items.clone(),
            Split::Dev => parse_corpus(&read(&root.join(qim_bench::dev_items(name)))?)?,
        };
        sets.push(Set {
            name: name.clone(),
            split,
            group: if colloquial(name) {
                Group::Colloquial
            } else {
                Group::Wiki
            },
            context_chars: 0,
            private: false,
            items,
        });
    }
    for (name, _) in &corpora {
        let rel = match split {
            Split::Test => qim_bench::context_long_items(name),
            Split::Dev => qim_bench::dev_context_long_items(name),
        };
        let items = parse_context(&read(&root.join(&rel))?).map_err(|e| format!("{rel}：{e}"))?;
        for chars in CONTEXT_LENGTHS {
            sets.push(Set {
                name: format!("{name}·前文{chars}"),
                split,
                group: if colloquial(name) {
                    Group::ContextColloquial
                } else {
                    Group::ContextWiki
                },
                context_chars: chars,
                private: false,
                items: items.iter().map(|i| with_context_chars(i, chars)).collect(),
            });
        }
    }
    if split == Split::Test
        && let Personal::Ran(items) = qim_bench::load_personal(root)?
    {
        sets.push(Set {
            name: qim_bench::PERSONAL_SET.into(),
            split,
            group: Group::Personal,
            context_chars: 0,
            private: true,
            items,
        });
    }
    Ok(sets)
}

/// 題目的前文只留最後 `chars` 字；0 是拿掉前文（不呼叫 `set_context`，同基準的不給前文）。
pub fn with_context_chars(item: &Item, chars: usize) -> Item {
    let context = (chars > 0)
        .then(|| item.context.as_deref().map(|c| last_chars(c, chars)))
        .flatten();
    Item {
        context,
        ..item.clone()
    }
}

fn last_chars(text: &str, n: usize) -> String {
    let skip = text.chars().count().saturating_sub(n);
    text.chars().skip(skip).collect()
}

/// 給重排模型的一題（停鍵後的重排要求）：不含答案。匯出檔一行一個（JSONL）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Query {
    /// `dev` 或 `test`：dev 與 test 的集合名相同，結果檔要靠它分。
    pub split: String,
    pub set: String,
    pub line: usize,
    /// 輸入框前文（已截到集合的前文長度），沒給是空字串。
    pub before: String,
    pub input: String,
    /// 候選列第一頁（最多 `RERANK_LIMIT` 個），依核心的順序；第一個是首選，重排不動它。
    pub candidates: Vec<String>,
    /// lattice 蓋滿整串的前 N 條路徑與路徑分，依分數由高到低。
    pub paths: Vec<ScoredPath>,
}

impl Query {
    /// 模型看得到的文字：候選與路徑。
    fn shows(&self, text: &str) -> bool {
        self.candidates.iter().any(|c| c == text) || self.paths.iter().any(|p| p.text == text)
    }
}

/// 停鍵層、雲端層的模型（plan §2.7）：看一題的候選與路徑，回傳它排的文字，好的在前（可以是
/// 候選、路徑或兩者混合）。首選不動（ADR 0005）：顯示的第 2 名是它排的第一個不是首選的文字。
/// 候選與路徑以外的文字不算（框架丟掉並計數）：重排只能挑核心給得出的字。
pub trait Reranker {
    /// 報告與逐題結果的目錄名。
    fn name(&self) -> &str;
    fn rerank(&mut self, query: &Query) -> Vec<String>;
}

/// 基線：不重排。
pub struct Identity;

impl Reranker for Identity {
    fn name(&self) -> &str {
        "baseline"
    }

    fn rerank(&mut self, _: &Query) -> Vec<String> {
        Vec::new()
    }
}

/// 外部模型的結果：JSONL，一行 `{"split": 切分, "set": 集合名, "line": 行號, "ranked": [文字…]}`，
/// 對應匯出的 [`Query`]。沒有結果的題照原順序，算進 [`Orders::missing`]。
pub struct Orders {
    name: String,
    orders: HashMap<(String, usize), Vec<String>>,
    pub missing: usize,
}

#[derive(Debug, Deserialize)]
struct OrderLine {
    split: String,
    set: String,
    line: usize,
    ranked: Vec<String>,
}

impl Orders {
    /// 讀結果檔；每一行的切分都要是 `split`，免得拿 dev 的結果去算 test。
    pub fn parse(name: &str, text: &str, split: Split) -> Result<Self, String> {
        let mut orders = HashMap::new();
        for (i, line) in text
            .lines()
            .enumerate()
            .filter(|(_, l)| !l.trim().is_empty())
        {
            let o: OrderLine =
                serde_json::from_str(line).map_err(|e| format!("第 {} 行：{e}", i + 1))?;
            if o.split != split.name() {
                return Err(format!(
                    "第 {} 行是 {} 切分的結果，這次跑的是 {}",
                    i + 1,
                    o.split,
                    split.name()
                ));
            }
            orders.insert((o.set, o.line), o.ranked);
        }
        Ok(Self {
            name: name.into(),
            orders,
            missing: 0,
        })
    }
}

impl Reranker for Orders {
    fn name(&self) -> &str {
        &self.name
    }

    fn rerank(&mut self, query: &Query) -> Vec<String> {
        match self.orders.get(&(query.set.clone(), query.line)) {
            Some(ranked) => ranked.clone(),
            None => {
                self.missing += 1;
                Vec::new()
            }
        }
    }
}

/// 一題打完輸入碼（不選字）時的狀態與模型的排序。
#[derive(Debug, Clone)]
pub struct Typed {
    /// 全部候選，依核心的順序。
    pub candidates: Vec<String>,
    /// lattice 的全部路徑（最多 beam 條）。
    pub paths: Vec<ScoredPath>,
    /// 交給模型的一題（路徑只留前 N 條）；沒有候選時是 `None`，不要求重排。
    pub query: Option<Query>,
    /// 模型排的文字，只留看得到的（[`Reranker`]）。
    pub ranked: Vec<String>,
    /// 模型回了幾個看不到的文字（丟掉了）。
    pub unseen: usize,
    /// 模型花的時間（ms）。
    pub rerank_ms: Option<f64>,
}

impl Typed {
    /// 顯示的前兩名：首選不動，第 2 名是模型排的第一個不是首選的文字；模型沒排時照核心的第 2 個。
    pub fn shown_top2(&self) -> [Option<&str>; 2] {
        let top = self.candidates.first().map(String::as_str);
        let second = self
            .ranked
            .iter()
            .map(String::as_str)
            .find(|t| Some(*t) != top)
            .or_else(|| self.candidates.get(1).map(String::as_str));
        [top, second]
    }
}

/// 逐鍵打完 `item` 的輸入碼、不選字，再把候選與前 `n` 條路徑交給 `reranker`（只要求一次：殼在
/// 停鍵後要求，模型回來前使用者沒動）。有前文的題先 `set_context`，同基準。
pub fn type_item(
    engine: &Engine,
    set: &Set,
    item: &Item,
    n: usize,
    reranker: &mut dyn Reranker,
) -> Typed {
    let mut session = Session::default();
    let before = item.context.clone().unwrap_or_default();
    if item.context.is_some() {
        session.set_context(&before);
    }
    let mut user = UserModel::default();
    for c in item.input().chars() {
        session.handle(engine, &mut user, Key::for_input(c));
    }
    let candidates: Vec<String> = session
        .all_candidates()
        .into_iter()
        .map(|c| c.text)
        .collect();
    let paths = session.paths(usize::from(engine.scheme().ranking.beam));
    let query = (!candidates.is_empty()).then(|| Query {
        split: set.split.name().into(),
        set: set.name.clone(),
        line: item.line,
        before,
        input: item.input(),
        candidates: candidates.iter().take(RERANK_LIMIT).cloned().collect(),
        paths: paths.iter().take(n).cloned().collect(),
    });
    let (ranked, unseen, rerank_ms) = match &query {
        Some(q) => {
            let start = Instant::now();
            let all = reranker.rerank(q);
            let ms = start.elapsed().as_secs_f64() * 1000.0;
            let total = all.len();
            let ranked: Vec<String> = all.into_iter().filter(|t| q.shows(t)).collect();
            (ranked.clone(), total - ranked.len(), Some(ms))
        }
        None => (Vec::new(), 0, None),
    };
    Typed {
        candidates,
        paths,
        query,
        ranked,
        unseen,
        rerank_ms,
    }
}

/// 一題的結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub line: usize,
    /// 期望在候選列的位置（從 0 起）；不在是 `None`。
    pub rank: Option<usize>,
    /// 期望在 lattice 路徑的位置（從 0 起，最多看 beam 條）；不在是 `None`。
    pub path_rank: Option<usize>,
    /// 首選正確（重排不動首選，和基準的整句首選正確相同）。
    pub correct: bool,
    /// 重排後期望在顯示的前 2 名（[`Typed::shown_top2`]）。
    pub top2: bool,
    /// 模型自己排的第一名是期望（假如規則讓它動首選；ADR 0005 不讓，只當參考）。模型沒排時看首選。
    pub model_first: bool,
    /// 基準模擬的修正次數。
    pub correction_count: usize,
    pub finished: bool,
    /// 期望的字數。
    pub chars: usize,
}

impl Row {
    fn from(item: &Item, typed: &Typed, scored: &qim_bench::ItemResult) -> Self {
        let expected = item.expected();
        let model_top = typed.ranked.first().or(typed.candidates.first());
        Row {
            line: item.line,
            rank: typed.candidates.iter().position(|c| *c == expected),
            path_rank: typed.paths.iter().position(|p| p.text == expected),
            correct: scored.correct,
            top2: typed.shown_top2().contains(&Some(expected.as_str())),
            model_first: model_top.is_some_and(|t| *t == expected),
            correction_count: scored.correction_count,
            finished: scored.finished,
            chars: scored.chars,
        }
    }
}

/// 一個集合的結果。
#[derive(Debug, Clone)]
pub struct SetRun {
    pub set: Set,
    pub rows: Vec<Row>,
    /// 交給模型的路徑字數（每條一筆）。
    pub path_chars: Vec<usize>,
    /// 每題交給模型的路徑數。
    pub path_counts: Vec<usize>,
    /// 模型每次花的時間（ms）。
    pub rerank_ms: Vec<f64>,
    /// 模型回了幾個看不到的文字（候選與路徑以外，丟掉了）。
    pub unseen: usize,
}

/// 打一個集合：首選與修正次數照基準算（`qim_bench::score_set`），其餘照 [`type_item`] 算。
pub fn run_set(engine: &Engine, set: &Set, n: usize, reranker: &mut dyn Reranker) -> SetRun {
    let scored = qim_bench::score_set(engine, &set.name, &set.items, set.private);
    let mut run = SetRun {
        set: set.clone(),
        rows: Vec::with_capacity(set.items.len()),
        path_chars: Vec::new(),
        path_counts: Vec::new(),
        rerank_ms: Vec::new(),
        unseen: 0,
    };
    for (item, result) in set.items.iter().zip(&scored.items) {
        assert_eq!(item.line, result.line, "基準的結果和題目要逐題對齊");
        let typed = type_item(engine, set, item, n, reranker);
        if let Some(q) = &typed.query {
            run.path_counts.push(q.paths.len());
            run.path_chars
                .extend(q.paths.iter().map(|p| p.text.chars().count()));
        }
        run.rerank_ms.extend(typed.rerank_ms);
        run.unseen += typed.unseen;
        run.rows.push(Row::from(item, &typed, result));
    }
    run
}

/// 一組題的合計。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Summary {
    pub total: usize,
    pub correct: usize,
    pub top2: usize,
    pub model_first: usize,
    pub correction_count: usize,
    pub unfinished: usize,
    /// 答案在候選列前 k 個（[`RECALL_AT`]）。
    pub recall: [usize; RECALL_AT.len()],
    /// 答案在 lattice 前 k 條路徑（[`RECALL_AT`]）。
    pub path_recall: [usize; RECALL_AT.len()],
}

impl Summary {
    pub fn of(rows: &[Row]) -> Self {
        let mut s = Summary {
            total: rows.len(),
            ..Summary::default()
        };
        let count = |recall: &mut [usize; RECALL_AT.len()], rank: Option<usize>| {
            if let Some(rank) = rank {
                for (k, n) in RECALL_AT.iter().zip(recall) {
                    *n += usize::from(rank < *k);
                }
            }
        };
        for row in rows {
            s.correct += usize::from(row.correct);
            s.top2 += usize::from(row.top2);
            s.model_first += usize::from(row.model_first);
            s.correction_count += row.correction_count;
            s.unfinished += usize::from(!row.finished);
            count(&mut s.recall, row.rank);
            count(&mut s.path_recall, row.path_rank);
        }
        s
    }

    fn add(mut self, other: Summary) -> Self {
        self.total += other.total;
        self.correct += other.correct;
        self.top2 += other.top2;
        self.model_first += other.model_first;
        self.correction_count += other.correction_count;
        self.unfinished += other.unfinished;
        for (a, b) in self.recall.iter_mut().zip(other.recall) {
            *a += b;
        }
        for (a, b) in self.path_recall.iter_mut().zip(other.path_recall) {
            *a += b;
        }
        self
    }
}

fn pct(part: usize, whole: usize) -> String {
    if whole == 0 {
        "—".into()
    } else {
        format!("{:.1}%", 100.0 * part as f64 / whole as f64)
    }
}

fn per(part: usize, whole: usize) -> String {
    if whole == 0 {
        "—".into()
    } else {
        format!("{:.3}", part as f64 / whole as f64)
    }
}

/// 第 `p` 百分位（最近秩）；空的是 `None`。
pub fn percentile<T: Copy + PartialOrd>(values: &[T], p: usize) -> Option<T> {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).expect("沒有 NaN"));
    let n = sorted.len();
    (n > 0).then(|| sorted[(n * p).div_ceil(100).clamp(1, n) - 1])
}

/// 合計列：同一組、同一前文長度的集合加起來；「有前文」依前文長度各一列，最後再加維基與口語合起來的
/// 「有前文」各前文長度一列。
fn group_totals(runs: &[SetRun]) -> Vec<(String, Summary)> {
    let sum = |keep: &dyn Fn(&SetRun) -> bool| {
        runs.iter()
            .filter(|r| keep(r))
            .map(|r| Summary::of(&r.rows))
            .fold(Summary::default(), Summary::add)
    };
    let mut keys: Vec<(Group, usize)> = runs
        .iter()
        .map(|r| (r.set.group, r.set.context_chars))
        .collect();
    keys.sort();
    keys.dedup();
    let mut totals: Vec<(String, Summary)> = keys
        .iter()
        .map(|&(group, chars)| {
            let label = if group.is_context() {
                format!("{}·前文{chars}", group.label())
            } else {
                group.label().to_string()
            };
            let summary = sum(&|r| r.set.group == group && r.set.context_chars == chars);
            (label, summary)
        })
        .collect();
    for chars in CONTEXT_LENGTHS {
        let summary = sum(&|r| r.set.group.is_context() && r.set.context_chars == chars);
        if summary.total > 0 {
            totals.push((format!("有前文·前文{chars}"), summary));
        }
    }
    totals
}

/// 給人看的報告，Markdown 表。
pub fn report(runs: &[SetRun], reranker: &str, n: usize) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "重排：{reranker}，N = {n} 條路徑\n");
    out.push_str(
        "| 集合 | 題數 | 首選正確率 | 前 2 名命中率 | 可動首選時 | 修正/句 | 修正 | 打不出 |\n\
         | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n",
    );
    let line = |name: &str, s: &Summary| {
        format!(
            "| {name} | {} | {} | {} | {} | {} | {} | {} |\n",
            s.total,
            pct(s.correct, s.total),
            pct(s.top2, s.total),
            pct(s.model_first, s.total),
            per(s.correction_count, s.total),
            s.correction_count,
            s.unfinished
        )
    };
    for run in runs {
        out.push_str(&line(&run.set.name, &Summary::of(&run.rows)));
    }
    let totals = group_totals(runs);
    for (label, summary) in &totals {
        out.push_str(&line(&format!("**{label}**"), summary));
    }

    for (title, pick) in [
        (
            "答案在候選列的前 k 個（重排前）",
            (|s: &Summary| s.recall) as fn(&Summary) -> [usize; RECALL_AT.len()],
        ),
        (
            "答案在 lattice 的前 k 條路徑（重排模型能挑的上限）",
            |s: &Summary| s.path_recall,
        ),
    ] {
        let _ = write!(out, "\n{title}\n\n| 集合 |");
        for k in RECALL_AT {
            let _ = write!(out, " 前 {k} |");
        }
        out.push_str("\n| --- |");
        out.push_str(&" ---: |".repeat(RECALL_AT.len()));
        out.push('\n');
        for (label, summary) in &totals {
            let _ = write!(out, "| {label} |");
            for n in pick(summary) {
                let _ = write!(out, " {} |", pct(n, summary.total));
            }
            out.push('\n');
        }
    }

    out.push_str(
        "\n交給模型的路徑（路徑數與長度）\n\n\
         | 組 | 題數 | 路徑數 p50 | 路徑字數 p50 | p95 | 最長 |\n\
         | --- | ---: | ---: | ---: | ---: | ---: |\n",
    );
    let mut groups: Vec<Group> = runs.iter().map(|r| r.set.group).collect();
    groups.sort();
    groups.dedup();
    for group in groups {
        // 「有前文」只取前文 200 字那種：同一批題，前文長度只改開頭的字接什麼。
        let of_group: Vec<&SetRun> = runs
            .iter()
            .filter(|r| r.set.group == group)
            .filter(|r| !group.is_context() || r.set.context_chars == CONTEXT_LENGTHS[2])
            .collect();
        let counts: Vec<usize> = of_group
            .iter()
            .flat_map(|r| r.path_counts.clone())
            .collect();
        let chars: Vec<usize> = of_group.iter().flat_map(|r| r.path_chars.clone()).collect();
        let show = |v: Option<usize>| v.map_or("—".into(), |v| v.to_string());
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} |",
            group.label(),
            counts.len(),
            show(percentile(&counts, 50)),
            show(percentile(&chars, 50)),
            show(percentile(&chars, 95)),
            show(chars.iter().copied().max()),
        );
    }

    let ms: Vec<f64> = runs.iter().flat_map(|r| r.rerank_ms.clone()).collect();
    if reranker != Identity.name()
        && let (Some(p50), Some(p95)) = (percentile(&ms, 50), percentile(&ms, 95))
    {
        let unseen: usize = runs.iter().map(|r| r.unseen).sum();
        let _ = writeln!(
            out,
            "\n重排每次 p50 {p50:.2} ms、p95 {p95:.2} ms（{} 次，框架內計時）；回了候選與路徑以外的文字 {unseen} 個（不算）",
            ms.len()
        );
    }
    out
}

/// 逐題結果的檔：`<dir>/<集合>.tsv`，一行一題，只有行號與數字（個人集也只到這裡）。
pub fn write_rows(dir: &Path, runs: &[SetRun]) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| format!("建不了 {}：{e}", dir.display()))?;
    for run in runs {
        let mut text =
            String::from("line\trank\tpath_rank\tcorrect\ttop2\tmodel_first\tcorrection_count\n");
        let show = |r: Option<usize>| r.map_or("-".into(), |k| k.to_string());
        for r in &run.rows {
            let _ = writeln!(
                text,
                "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                r.line,
                show(r.rank),
                show(r.path_rank),
                u8::from(r.correct),
                u8::from(r.top2),
                u8::from(r.model_first),
                r.correction_count
            );
        }
        let path = dir.join(format!("{}.tsv", run.set.name));
        fs::write(&path, text).map_err(|e| format!("寫不了 {}：{e}", path.display()))?;
    }
    Ok(())
}

/// 匯出一個集合的重排要求（[`Query`]，不含答案），JSONL；路徑留前 `n` 條。
pub fn export_set(engine: &Engine, set: &Set, n: usize) -> String {
    let mut out = String::new();
    for item in &set.items {
        if let Some(query) = type_item(engine, set, item, n, &mut Identity).query {
            out.push_str(&serde_json::to_string(&query).expect("可序列化"));
            out.push('\n');
        }
    }
    out
}

/// 兩次結果在同一集合的配對比較。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Paired {
    pub total: usize,
    /// 由錯變對的題數。
    pub better: usize,
    /// 由對變錯的題數。
    pub worse: usize,
}

impl Paired {
    /// McNemar 精確檢定（雙尾）的 p 值：不一致的題裡翻好、翻壞各半的二項分佈。
    pub fn p_value(&self) -> f64 {
        mcnemar_exact(self.better, self.worse)
    }
}

/// McNemar 精確檢定的雙尾 p 值。
pub fn mcnemar_exact(b: usize, c: usize) -> f64 {
    let n = b + c;
    if n == 0 {
        return 1.0;
    }
    let k = b.min(c);
    // log C(n, i) 逐項累加，n 上千也不溢位。
    let ln_half_n = n as f64 * 0.5f64.ln();
    let mut ln_choose = 0.0f64;
    let mut tail = 0.0f64;
    for i in 0..=k {
        if i > 0 {
            ln_choose += ((n - i + 1) as f64).ln() - (i as f64).ln();
        }
        tail += (ln_choose + ln_half_n).exp();
    }
    (2.0 * tail).min(1.0)
}

/// 讀 [`write_rows`] 寫的檔：行號 → （首選正確, 前 2 名, 可動首選）。
pub fn read_rows(path: &Path) -> Result<HashMap<usize, [bool; 3]>, String> {
    let text = read(path)?;
    let mut rows = HashMap::new();
    for line in text.lines().skip(1) {
        let cols: Vec<&str> = line.split('\t').collect();
        let bad = || format!("{}：格式不對：{line}", path.display());
        if cols.len() < 6 {
            return Err(bad());
        }
        let flag = |s: &str| s == "1";
        let n: usize = cols[0].parse().map_err(|_| bad())?;
        rows.insert(n, [flag(cols[3]), flag(cols[4]), flag(cols[5])]);
    }
    Ok(rows)
}

/// 比較兩個逐題結果目錄裡同名的集合：首選、前 2 名、可動首選三種各一份 [`Paired`]。
pub fn compare(a: &Path, b: &Path) -> Result<Vec<(String, [Paired; 3])>, String> {
    let mut names: Vec<PathBuf> = fs::read_dir(a)
        .map_err(|e| format!("讀不到 {}：{e}", a.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension().is_some_and(|x| x == "tsv") && b.join(p.file_name().unwrap()).exists()
        })
        .collect();
    names.sort();
    let mut out = Vec::new();
    for path in names {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let (ra, rb) = (
            read_rows(&path)?,
            read_rows(&b.join(path.file_name().unwrap()))?,
        );
        let mut paired = [Paired::default(); 3];
        for (line, x) in &ra {
            let Some(y) = rb.get(line) else { continue };
            for (p, (x, y)) in paired.iter_mut().zip(x.iter().zip(y)) {
                p.total += 1;
                p.better += usize::from(!x && *y);
                p.worse += usize::from(*x && !y);
            }
        }
        out.push((name, paired));
    }
    Ok(out)
}

/// 同步層的成本：語言模型資料包的大小與解開的時間（讀進記憶體後、`from_bytes` 那一步，`runs` 次的
/// 中位數）。常駐記憶體沿用 M2 票 04 的量測（README〈基線結果〉）。
pub fn language_model_cost(root: &Path, engine: &Engine, runs: usize) -> Result<String, String> {
    let Some(id) = &engine.scheme().packs.language_model else {
        return Ok("方案沒有語言模型資料包".into());
    };
    let rel = format!("{}/{id}/{id}.pack", qim_bench::PACKS);
    let bytes = fs::read(root.join(&rel)).map_err(|e| format!("讀不到 {rel}：{e}"))?;
    let mut ms: Vec<f64> = (0..runs.max(1))
        .map(|_| {
            let start = Instant::now();
            let pack = LanguageModelPack::from_bytes(&bytes).expect("基準載入過的資料包");
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
            drop(pack);
            elapsed
        })
        .collect();
    ms.sort_by(f64::total_cmp);
    Ok(format!(
        "{rel}：{} 位元組，解開 {:.1} ms（{} 次的中位數，最快 {:.1}、最慢 {:.1}）",
        bytes.len(),
        ms[ms.len() / 2],
        ms.len(),
        ms[0],
        ms[ms.len() - 1]
    ))
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| {
        format!(
            "讀不到 {}：{e}；先在 repo 根目錄跑 `cargo xtask data`",
            path.display()
        )
    })
}
