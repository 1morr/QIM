//! 同步層的語言模型（M3 票 08，README〈同步層〉）：字級 trigram、詞級 bigram／trigram，以及比照
//! 標準 Kneser-Ney 重訓的字級 bigram。都經 `Engine::with_segment_model` 換掉 lattice 的靜態機率，
//! 其餘（β、γ、`personal`、候選列）照核心。
//!
//! 高一階的模型都退回現行的打分（plan §2.3：字級 bigram 接前一字、詞內用詞頻）：
//!
//! - `P(w | h) = max(c(h,w) − d, 0) ÷ c(h) + (Σ d ÷ c(h)) × P_低(w | h')`，`d` 見 [`Discounting`]；
//! - 剪枝只留 `c(h,w) ≥` 門檻的，`α(h) = (1 − Σ 留下的 P) ÷ (1 − Σ 留下的 P_低)`，沒留下的用
//!   `α(h) × P_低`（α 夾在 1 以下，見 [`Table::build`]）；`h` 沒有留下任何一個時直接用 `P_低`。
//!
//! 機率存成 `u8`（同核心的 [`qim_core::LOG_PROB_STEP`]），退回權重 `f32`；資料包大小是 postcard
//! 序列化（gap 的 varint，同核心的語言模型資料包）的位元組數。

use std::fs::File;
use std::hash::Hash;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use qim_core::{CLAUSE_START, LOG_PROB_STEP, LanguageModel, Lexicon, SegmentModel, SegmentQuery};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

/// 一份訓練語料：`build/corpus/<名>/train.txt` 與它的權重（plan §4.3）。
#[derive(Debug, Clone)]
pub struct TrainCorpus {
    pub name: String,
    pub path: PathBuf,
    pub weight: u64,
    pub colloquial: bool,
}

/// `cargo xtask data` 寫的各語料 train 切分，權重照 `qim_databuild::corpora::CORPORA`。
pub fn train_corpora(root: &Path) -> Vec<TrainCorpus> {
    qim_databuild::corpora::CORPORA
        .iter()
        .map(|c| TrainCorpus {
            name: c.name.to_string(),
            path: qim_databuild::corpora::build_dir(root, c.name).join("train.txt"),
            weight: c.weight,
            colloquial: c.group == qim_databuild::corpora::Group::Colloquial,
        })
        .collect()
}

fn each_line(path: &Path, mut f: impl FnMut(&str)) -> Result<(), String> {
    let file = File::open(path).map_err(|e| format!("讀不到 {}：{e}", path.display()))?;
    for line in BufReader::new(file).lines() {
        f(&line.map_err(|e| format!("{}：{e}", path.display()))?);
    }
    Ok(())
}

/// 折扣 `d` 怎麼算。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Discounting {
    /// 現行字級 bigram 的做法（plan §4.3）：`D = n1 ÷ (n1 + 2 n2)` 在乘上語料權重後的次數上算，
    /// 每種 n-gram 扣同一個 `D`；口語的次數至少 64，口語幾乎不平滑。
    Weighted,
    /// 比照標準 KN：每份語料在自己的原始次數上算 `D_k`，`d = Σ D_k × w_k`（出現在語料 k 的才算），
    /// 等於各語料分開訓練、再依權重把次數加起來。
    PerCorpus,
}

/// 一種 n-gram 乘上權重的次數與它的折扣。
#[derive(Debug, Clone, Copy, Default)]
pub struct Count {
    pub weighted: u64,
    /// [`Discounting::PerCorpus`] 的 `Σ D_k × w_k`。
    pub per_corpus: f64,
}

/// 各語料的 n-gram 次數；`extract` 把一句的 n-gram 交給回呼。
pub fn count<K: Hash + Eq + Copy>(
    corpora: &[TrainCorpus],
    mut extract: impl FnMut(&str, &mut dyn FnMut(K)),
) -> Result<FxHashMap<K, Count>, String> {
    let mut total: FxHashMap<K, Count> = FxHashMap::default();
    for corpus in corpora {
        let mut local: FxHashMap<K, u32> = FxHashMap::default();
        each_line(&corpus.path, |line| {
            extract(line, &mut |k| *local.entry(k).or_default() += 1);
        })?;
        merge(&mut total, local, corpus.weight);
    }
    Ok(total)
}

/// `D = n1 ÷ (n1 + 2 n2)`；沒有出現一次的時是 0.5（同 `qim_databuild::bigram`）。
pub fn discount_of(counts: impl Iterator<Item = u64>) -> f64 {
    let (mut n1, mut n2) = (0.0, 0.0);
    for n in counts {
        match n {
            1 => n1 += 1.0,
            2 => n2 += 1.0,
            _ => {}
        }
    }
    if n1 > 0.0 { n1 / (n1 + 2.0 * n2) } else { 0.5 }
}

/// 每種 n-gram 扣多少。
fn discounts<K: Copy>(counts: &FxHashMap<K, Count>, how: Discounting) -> impl Fn(&Count) -> f64 {
    let weighted = discount_of(counts.values().map(|c| c.weighted));
    move |c: &Count| match how {
        Discounting::Weighted => weighted.min(c.weighted as f64),
        Discounting::PerCorpus => c.per_corpus.min(c.weighted as f64),
    }
}

/// 一階 n-gram 的表：依前文（`u64` 的鍵）分列，每列的後一個（`u32` 的鍵）遞增。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Table {
    contexts: Vec<u64>,
    backoff: Vec<f32>,
    rows: Vec<u32>,
    next: Vec<u32>,
    log_prob: Vec<u8>,
}

/// 查表的結果。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Lookup {
    /// 留下的 n-gram：`ln P`。
    Hit(f64),
    /// 前文有留下的 n-gram，這個沒有：`ln α`，加上低一階的。
    Backoff(f64),
    /// 前文沒有留下任何 n-gram：照低一階。
    Unseen,
}

impl Lookup {
    /// 這一階的 ln P：留下的直接用，否則 `ln α` 加上低一階的 `lower`（ln）。
    pub fn or_lower(self, lower: f64) -> f64 {
        match self {
            Lookup::Hit(lp) => lp,
            Lookup::Backoff(alpha) => alpha + lower,
            Lookup::Unseen => lower,
        }
    }
}

impl Table {
    /// 由次數建表：只留 `weighted ≥ min_count` 的；`lower(ctx, next)` 是低一階的機率（不取 ln）。
    pub fn build(
        counts: &FxHashMap<(u64, u32), Count>,
        how: Discounting,
        min_count: u64,
        lower: impl Fn(u64, u32) -> f64,
    ) -> (Self, TableStats) {
        let discount = discounts(counts, how);
        let mut keys: Vec<(u64, u32)> = counts.keys().copied().collect();
        keys.sort_unstable();
        let mut table = Table {
            rows: vec![0],
            ..Table::default()
        };
        let mut stats = TableStats::default();
        let mut start = 0;
        while start < keys.len() {
            let ctx = keys[start].0;
            let end = start + keys[start..].partition_point(|k| k.0 == ctx);
            let row = &keys[start..end];
            let total: f64 = row.iter().map(|k| counts[k].weighted as f64).sum();
            let mass: f64 = row.iter().map(|k| discount(&counts[k])).sum();
            let (mut kept_p, mut kept_lower) = (0.0, 0.0);
            let before = table.next.len();
            for k in row {
                let c = &counts[k];
                if c.weighted < min_count {
                    continue;
                }
                let low = lower(ctx, k.1);
                let p = (c.weighted as f64 - discount(c)).max(0.0) / total + mass / total * low;
                kept_p += p;
                kept_lower += low;
                table.next.push(k.1);
                table.log_prob.push(quantize(p.ln()));
            }
            if table.next.len() > before {
                // 低一階幾乎全給了留下的那幾個時（罕用字的 bigram 列常這樣），分母近 0、又受量化誤差
                // 左右，α 會爆大，沒留下的反而比低一階高；所以 α 夾在 1 以下。
                let (num, den) = (1.0 - kept_p, 1.0 - kept_lower);
                let alpha = num.max(1e-9) / den.max(1e-9);
                if alpha > 1.0 {
                    stats.clamped += 1;
                }
                #[allow(clippy::cast_possible_truncation)] // 存成 f32 是刻意的精度取捨
                table.backoff.push(alpha.min(1.0).ln() as f32);
                table.contexts.push(ctx);
                table
                    .rows
                    .push(u32::try_from(table.next.len()).expect("放得進 u32"));
            }
            stats.types += row.len();
            start = end;
        }
        stats.kept = table.next.len();
        stats.contexts = table.contexts.len();
        (table, stats)
    }

    pub fn lookup(&self, ctx: u64, next: u32) -> Lookup {
        let Ok(i) = self.contexts.binary_search(&ctx) else {
            return Lookup::Unseen;
        };
        let (from, to) = (self.rows[i] as usize, self.rows[i + 1] as usize);
        match self.next[from..to].binary_search(&next) {
            Ok(k) => Lookup::Hit(dequantize(self.log_prob[from + k])),
            Err(_) => Lookup::Backoff(f64::from(self.backoff[i])),
        }
    }

    /// 載入後常駐的陣列位元組數。
    pub fn memory(&self) -> usize {
        self.contexts.len() * 8
            + self.backoff.len() * 4
            + self.rows.len() * 4
            + self.next.len() * 4
            + self.log_prob.len()
    }

    fn to_wire(&self) -> TableWire {
        let mut last = 0;
        let context_gaps = self
            .contexts
            .iter()
            .map(|&c| {
                let gap = c - last;
                last = c;
                gap
            })
            .collect();
        let row_lengths = self.rows.windows(2).map(|w| w[1] - w[0]).collect();
        let mut next_gaps = Vec::with_capacity(self.next.len());
        for w in self.rows.windows(2) {
            let mut last = 0;
            for &j in &self.next[w[0] as usize..w[1] as usize] {
                next_gaps.push(j - last);
                last = j;
            }
        }
        TableWire {
            context_gaps,
            backoff: self.backoff.clone(),
            row_lengths,
            next_gaps,
            log_prob: self.log_prob.clone(),
        }
    }

    fn from_wire(w: TableWire) -> Self {
        let mut contexts = Vec::with_capacity(w.context_gaps.len());
        let mut last = 0;
        for gap in w.context_gaps {
            last += gap;
            contexts.push(last);
        }
        let mut rows = Vec::with_capacity(w.row_lengths.len() + 1);
        rows.push(0u32);
        let mut next = Vec::with_capacity(w.next_gaps.len());
        let mut gaps = w.next_gaps.into_iter();
        for len in w.row_lengths {
            let mut last = 0;
            for _ in 0..len {
                last += gaps.next().expect("列長和 gap 數對得上");
                next.push(last);
            }
            rows.push(u32::try_from(next.len()).expect("放得進 u32"));
        }
        Self {
            contexts,
            backoff: w.backoff,
            rows,
            next,
            log_prob: w.log_prob,
        }
    }
}

/// 建表的統計。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TableStats {
    /// 不重複的 n-gram 數。
    pub types: usize,
    pub kept: usize,
    pub contexts: usize,
    /// α 超過 1、夾成 1 的前文數。
    pub clamped: usize,
}

#[derive(Serialize, Deserialize)]
struct TableWire {
    context_gaps: Vec<u64>,
    backoff: Vec<f32>,
    row_lengths: Vec<u32>,
    next_gaps: Vec<u32>,
    log_prob: Vec<u8>,
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // 已夾在 0–255
fn quantize(log_prob: f64) -> u8 {
    (-log_prob / LOG_PROB_STEP).round().clamp(0.0, 255.0) as u8
}

fn dequantize(q: u8) -> f64 {
    -f64::from(q) * LOG_PROB_STEP
}

/// 一個模型的大小與載入成本。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cost {
    /// 這個模型自己的資料包位元組數（不含它退回的字級 bigram）。
    pub bytes: usize,
    /// 載入後常駐的陣列位元組數（估計：陣列長度乘元素大小，雜湊表另加估計的開銷）；沒估的是 `None`。
    pub memory: Option<usize>,
    /// 由位元組解開成查詢用的結構的毫秒數（5 次的中位數）。
    pub load_ms: f64,
}

/// 解開 `bytes` 量 5 次，回中位數（毫秒）。
fn time_load<T>(bytes: &[u8], decode: impl Fn(&[u8]) -> T) -> f64 {
    let mut ms: Vec<f64> = (0..5)
        .map(|_| {
            let start = std::time::Instant::now();
            let value = decode(bytes);
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
            drop(std::hint::black_box(value));
            elapsed
        })
        .collect();
    ms.sort_by(f64::total_cmp);
    ms[ms.len() / 2]
}

// ---- 字級 trigram ----

/// 一個字（Unicode 碼位）要幾個位元。
const CHAR_BITS: u32 = 21;

/// 字級 trigram 的鍵：前兩字各 [`CHAR_BITS`] 位元。
fn char_pair(a: char, b: char) -> u64 {
    (u64::from(u32::from(a)) << CHAR_BITS) | u64::from(u32::from(b))
}

/// [`char_pair`] 的後一字。
fn second_char(pair: u64) -> char {
    char::from_u32((pair & ((1 << CHAR_BITS) - 1)) as u32).expect("鍵是字")
}

/// 一句的 trigram：句首記號接在第一字前面，所以有 `(句首, 第一字, 第二字)`。
pub fn char_trigrams(line: &str, emit: &mut dyn FnMut((u64, u32))) {
    let mut a = None;
    let mut b = Some(CLAUSE_START);
    for c in line.chars() {
        if let (Some(a), Some(b)) = (a, b) {
            emit((char_pair(a, b), u32::from(c)));
        }
        (a, b) = (b, Some(c));
    }
}

/// 字級 trigram，退回字級 bigram（`base`）。
#[derive(Debug, Clone)]
pub struct CharTrigram {
    base: LanguageModel,
    table: Table,
}

impl CharTrigram {
    pub fn train(
        base: &LanguageModel,
        counts: &FxHashMap<(u64, u32), Count>,
        how: Discounting,
        min_count: u64,
    ) -> (Self, TableStats) {
        let lower = |ctx: u64, c: u32| {
            base.log_prob(Some(second_char(ctx)), char::from_u32(c).expect("鍵是字"))
                .exp()
        };
        let (table, stats) = Table::build(counts, how, min_count, lower);
        (
            Self {
                base: base.clone(),
                table,
            },
            stats,
        )
    }

    /// 查個別 trigram 用（`EXPLAIN` 除錯）：`(a, b, c)` 的查表結果與低一階的 ln P。
    pub fn explain(&self, a: char, b: char, c: char) -> (Lookup, f64) {
        (
            self.table.lookup(char_pair(a, b), u32::from(c)),
            self.base.log_prob(Some(b), c),
        )
    }

    pub fn cost(&self) -> Cost {
        let bytes = postcard::to_allocvec(&self.table.to_wire()).expect("序列化");
        let load_ms = time_load(&bytes, |b| {
            Table::from_wire(postcard::from_bytes(b).expect("解得開"))
        });
        Cost {
            bytes: bytes.len(),
            memory: Some(self.table.memory()),
            load_ms,
        }
    }
}

impl SegmentModel for CharTrigram {
    fn log_prob(&self, query: &SegmentQuery<'_>) -> f64 {
        let mut back = query.before.iter().rev().flat_map(|s| s.chars().rev());
        let (Some(b), Some(a)) = (back.next(), back.next()) else {
            return query.current_log_prob;
        };
        let first = query.text.chars().next().expect("段不是空的");
        match self.table.lookup(char_pair(a, b), u32::from(first)) {
            Lookup::Hit(lp) => query.current_log_prob - self.base.log_prob(Some(b), first) + lp,
            Lookup::Backoff(alpha) => query.current_log_prob + alpha,
            Lookup::Unseen => query.current_log_prob,
        }
    }
}

// ---- 詞級 n-gram ----

/// 現行打分要的東西：字級 bigram、詞庫的詞內機率、β、γ。切訓練語料與詞級模型的低一階都用它。
#[derive(Debug, Clone)]
pub struct CurrentScoring {
    pub base: LanguageModel,
    /// 詞庫的詞 → 第一字之後的 ln 機率（plan §2.3：`ln P_詞頻 − ln P_unigram(第一字)`）。
    rest: FxHashMap<Box<str>, f64>,
    longest: usize,
    word_bonus: f64,
    segment_penalty: f64,
}

impl CurrentScoring {
    /// `lexicon` 是方案的詞庫；分母同 `Engine`：每個不重複的詞算一次的詞頻總和。
    pub fn new(
        base: LanguageModel,
        lexicon: &Lexicon,
        word_bonus: f64,
        segment_penalty: f64,
    ) -> Self {
        let mut frequency: FxHashMap<Box<str>, u64> = FxHashMap::default();
        for (_, words) in lexicon.iter() {
            for w in words {
                frequency
                    .entry(w.text.as_str().into())
                    .or_insert(w.frequency);
            }
        }
        let total = (frequency.values().sum::<u64>().max(1) as f64).ln();
        let rest = frequency
            .into_iter()
            .map(|(text, f)| {
                let first = text.chars().next().expect("詞不是空的");
                let r = (f.max(1) as f64).ln() - total - base.log_prob(None, first);
                (text, r)
            })
            .collect::<FxHashMap<_, _>>();
        let longest = rest.keys().map(|t| t.chars().count()).max().unwrap_or(1);
        Self {
            base,
            rest,
            longest,
            word_bonus,
            segment_penalty,
        }
    }

    /// 段 `text` 接在前一字 `prev` 後面的靜態 ln 機率（同 `Engine` 的邊）；不是字也不是詞庫的詞時是 `None`。
    pub fn log_prob(&self, prev: Option<char>, text: &str) -> Option<f64> {
        let mut chars = text.chars();
        let first = chars.next()?;
        let rest = if chars.next().is_none() {
            0.0
        } else {
            *self.rest.get(text)?
        };
        Some(self.base.log_prob(prev, first) + rest)
    }

    /// 用現行打分把一句切成段（已知文字的 Viterbi）：前一字固定是句中的前一字（句首接
    /// [`CLAUSE_START`]），所以每段的分數不看前面怎麼切。回各段的位元組範圍。
    pub fn segment(&self, line: &str) -> Vec<(usize, usize)> {
        let bounds: Vec<usize> = line
            .char_indices()
            .map(|(i, _)| i)
            .chain([line.len()])
            .collect();
        let chars: Vec<char> = line.chars().collect();
        let n = chars.len();
        let mut best = vec![f64::NEG_INFINITY; n + 1];
        let mut from = vec![0usize; n + 1];
        best[0] = 0.0;
        for i in 0..n {
            if best[i] == f64::NEG_INFINITY {
                continue;
            }
            let prev = Some(if i == 0 { CLAUSE_START } else { chars[i - 1] });
            for j in i + 1..=(i + self.longest).min(n) {
                let text = &line[bounds[i]..bounds[j]];
                let Some(lp) = self.log_prob(prev, text) else {
                    continue;
                };
                let bonus = if j - i > 1 { self.word_bonus } else { 0.0 };
                let score = best[i] + lp + bonus - self.segment_penalty;
                if score > best[j] {
                    best[j] = score;
                    from[j] = i;
                }
            }
        }
        let mut segments = Vec::new();
        let mut j = n;
        while j > 0 {
            let i = from[j];
            segments.push((bounds[i], bounds[j]));
            j = i;
        }
        segments.reverse();
        segments
    }
}

/// 詞（段）的編號；0 是句首。
#[derive(Debug, Clone, Default)]
pub struct Vocabulary {
    ids: FxHashMap<Box<str>, u32>,
    texts: Vec<Box<str>>,
}

impl Vocabulary {
    pub fn new() -> Self {
        let mut v = Self::default();
        v.intern(&CLAUSE_START.to_string());
        v
    }

    pub fn intern(&mut self, text: &str) -> u32 {
        if let Some(&id) = self.ids.get(text) {
            return id;
        }
        let id = u32::try_from(self.texts.len()).expect("放得進 u32");
        self.ids.insert(text.into(), id);
        self.texts.push(text.into());
        id
    }

    pub fn get(&self, text: &str) -> Option<u32> {
        self.ids.get(text).copied()
    }

    pub fn text(&self, id: u32) -> &str {
        &self.texts[id as usize]
    }

    pub fn len(&self) -> usize {
        self.texts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.texts.is_empty()
    }
}

/// 切好的語料的詞級次數：bigram 的鍵是（前一詞, 詞），trigram 的前文是兩個 21 位元的詞編號。
#[derive(Debug, Default)]
pub struct WordCounts {
    pub vocabulary: Vocabulary,
    pub bigrams: FxHashMap<(u64, u32), Count>,
    pub trigrams: FxHashMap<(u64, u32), Count>,
}

const WORD_BITS: u32 = 21;

fn word_pair(u: u32, v: u32) -> u64 {
    (u64::from(u) << WORD_BITS) | u64::from(v)
}

/// 用 `scoring` 切各語料，數詞級 bigram（`order` ≥ 3 時也數 trigram）。
pub fn count_words(
    corpora: &[TrainCorpus],
    scoring: &CurrentScoring,
    order: usize,
) -> Result<WordCounts, String> {
    let mut vocabulary = Vocabulary::new();
    let mut ids = Vec::new();
    let mut bigrams = FxHashMap::default();
    let mut trigrams = FxHashMap::default();
    // 每份語料分開數，才能照 [`Discounting::PerCorpus`] 算各自的 D。
    for corpus in corpora {
        let mut local_bi: FxHashMap<(u64, u32), u32> = FxHashMap::default();
        let mut local_tri: FxHashMap<(u64, u32), u32> = FxHashMap::default();
        each_line(&corpus.path, |line| {
            ids.clear();
            ids.push(0);
            for (i, j) in scoring.segment(line) {
                ids.push(vocabulary.intern(&line[i..j]));
            }
            for w in ids.windows(2) {
                *local_bi.entry((u64::from(w[0]), w[1])).or_default() += 1;
            }
            if order >= 3 {
                for w in ids.windows(3) {
                    *local_tri.entry((word_pair(w[0], w[1]), w[2])).or_default() += 1;
                }
            }
        })?;
        merge(&mut bigrams, local_bi, corpus.weight);
        merge(&mut trigrams, local_tri, corpus.weight);
    }
    assert!(
        vocabulary.len() < 1 << WORD_BITS,
        "詞編號放不進 {WORD_BITS} 位元"
    );
    Ok(WordCounts {
        vocabulary,
        bigrams,
        trigrams,
    })
}

fn merge<K: Hash + Eq>(total: &mut FxHashMap<K, Count>, local: FxHashMap<K, u32>, weight: u64) {
    let d = discount_of(local.values().map(|&n| u64::from(n)));
    for (k, n) in local {
        let entry = total.entry(k).or_default();
        entry.weighted += u64::from(n) * weight;
        entry.per_corpus += d * weight as f64;
    }
}

/// 詞級 bigram（`trigram` 有表時是 trigram），退回現行打分。
#[derive(Debug, Clone)]
pub struct WordNgram {
    vocabulary: Vocabulary,
    bigram: Table,
    trigram: Option<Table>,
}

impl WordNgram {
    /// `min_trigram` 是 `None` 時只建 bigram。
    pub fn train(
        scoring: &CurrentScoring,
        counts: &WordCounts,
        how: Discounting,
        min_bigram: u64,
        min_trigram: Option<u64>,
    ) -> (Self, TableStats, Option<TableStats>) {
        let vocab = &counts.vocabulary;
        let last_char = |id: u32| vocab.text(id).chars().next_back();
        let low = |v: u32, w: u32| {
            scoring
                .log_prob(last_char(v), vocab.text(w))
                .expect("切出來的段都是字或詞")
                .exp()
        };
        let (bigram, bi_stats) = Table::build(&counts.bigrams, how, min_bigram, |ctx, w| {
            low(u32::try_from(ctx).expect("bigram 的前文是一個詞"), w)
        });
        let mut model = Self {
            vocabulary: Vocabulary::default(),
            bigram,
            trigram: None,
        };
        let tri_stats = min_trigram.map(|min| {
            let lower = |ctx: u64, w: u32| {
                let v = (ctx & ((1 << WORD_BITS) - 1)) as u32;
                let base = low(v, w).ln();
                model.bigram.lookup(u64::from(v), w).or_lower(base).exp()
            };
            let (table, stats) = Table::build(&counts.trigrams, how, min, lower);
            model.trigram = Some(table);
            stats
        });
        model.vocabulary = used_vocabulary(vocab, &model.bigram, model.trigram.as_ref());
        // 編號換成只含用到的詞的新編號：表要照新編號重建。
        let remap = |t: &Table, pair: bool| remap_table(t, vocab, &model.vocabulary, pair);
        model.bigram = remap(&model.bigram, false);
        model.trigram = model.trigram.as_ref().map(|t| remap(t, true));
        (model, bi_stats, tri_stats)
    }

    pub fn cost(&self) -> Cost {
        let wire = WordWire {
            vocabulary: self
                .vocabulary
                .texts
                .iter()
                .map(|t| t.to_string())
                .collect(),
            bigram: self.bigram.to_wire(),
            trigram: self.trigram.as_ref().map(Table::to_wire),
        };
        let bytes = postcard::to_allocvec(&wire).expect("序列化");
        let load_ms = time_load(&bytes, |b| {
            let w: WordWire = postcard::from_bytes(b).expect("解得開");
            let mut v = Vocabulary::default();
            for t in &w.vocabulary {
                v.intern(t);
            }
            (
                v,
                Table::from_wire(w.bigram),
                w.trigram.map(Table::from_wire),
            )
        });
        // 雜湊表：每格鍵（Box<str> 16 位元組加字串）與值 4 位元組，約 1.15 倍的空格；另存一份文字。
        let strings: usize = self.vocabulary.texts.iter().map(|t| t.len()).sum();
        let vocab_memory =
            self.vocabulary.len() * (16 + 4) * 115 / 100 + strings + self.vocabulary.len() * 16;
        Cost {
            bytes: bytes.len(),
            memory: Some(
                self.bigram.memory()
                    + self.trigram.as_ref().map_or(0, Table::memory)
                    + vocab_memory,
            ),
            load_ms,
        }
    }
}

/// 留下的表用到的詞：句首永遠在 0。
fn used_vocabulary(vocab: &Vocabulary, bigram: &Table, trigram: Option<&Table>) -> Vocabulary {
    let mask = (1u64 << WORD_BITS) - 1;
    let mut used = vec![false; vocab.len()];
    used[0] = true;
    for &c in &bigram.contexts {
        used[c as usize] = true;
    }
    for &n in &bigram.next {
        used[n as usize] = true;
    }
    if let Some(t) = trigram {
        for &c in &t.contexts {
            used[(c >> WORD_BITS) as usize] = true;
            used[(c & mask) as usize] = true;
        }
        for &n in &t.next {
            used[n as usize] = true;
        }
    }
    let mut out = Vocabulary::default();
    for (id, &u) in used.iter().enumerate() {
        if u {
            out.intern(vocab.text(u32::try_from(id).expect("放得進 u32")));
        }
    }
    out
}

fn remap_table(t: &Table, from: &Vocabulary, to: &Vocabulary, pair: bool) -> Table {
    let mask = (1u64 << WORD_BITS) - 1;
    let id = |old: u32| to.get(from.text(old)).expect("用到的詞都在新編號裡");
    let ctx = |c: u64| {
        if pair {
            word_pair(id((c >> WORD_BITS) as u32), id((c & mask) as u32))
        } else {
            u64::from(id(u32::try_from(c).expect("一個詞")))
        }
    };
    // 每列：（新的前文鍵, α, [(新的後一個, 量化的 ln P)]）。
    type Row = (u64, f32, Vec<(u32, u8)>);
    let mut rows: Vec<Row> = (0..t.contexts.len())
        .map(|i| {
            let (a, b) = (t.rows[i] as usize, t.rows[i + 1] as usize);
            let mut row: Vec<(u32, u8)> = (a..b).map(|k| (id(t.next[k]), t.log_prob[k])).collect();
            row.sort_unstable();
            (ctx(t.contexts[i]), t.backoff[i], row)
        })
        .collect();
    rows.sort_by_key(|r| r.0);
    let mut out = Table {
        rows: vec![0],
        ..Table::default()
    };
    for (c, alpha, row) in rows {
        out.contexts.push(c);
        out.backoff.push(alpha);
        for (n, q) in row {
            out.next.push(n);
            out.log_prob.push(q);
        }
        out.rows
            .push(u32::try_from(out.next.len()).expect("放得進 u32"));
    }
    out
}

#[derive(Serialize, Deserialize)]
struct WordWire {
    vocabulary: Vec<String>,
    bigram: TableWire,
    trigram: Option<TableWire>,
}

impl SegmentModel for WordNgram {
    fn log_prob(&self, query: &SegmentQuery<'_>) -> f64 {
        let Some(&v_text) = query.before.last() else {
            return query.current_log_prob;
        };
        let Some(v) = self.vocabulary.get(v_text) else {
            return query.current_log_prob;
        };
        let w = self.vocabulary.get(query.text);
        // 詞不在表裡：任何 n-gram 都沒有它，前文有留下的就乘 α。
        let w = w.unwrap_or(u32::MAX);
        let bigram = self
            .bigram
            .lookup(u64::from(v), w)
            .or_lower(query.current_log_prob);
        let (Some(trigram), Some(&u_text)) = (&self.trigram, query.before.iter().rev().nth(1))
        else {
            return bigram;
        };
        let Some(u) = self.vocabulary.get(u_text) else {
            return bigram;
        };
        trigram.lookup(word_pair(u, v), w).or_lower(bigram)
    }
}

// ---- 字級 bigram 重訓（同一套次數與折扣） ----

/// 一句的 bigram（前一字當前文的鍵）：句首記號接在第一字前面。
pub fn char_bigrams(line: &str, emit: &mut dyn FnMut((u64, u32))) {
    let mut prev = CLAUSE_START;
    for c in line.chars() {
        emit((u64::from(u32::from(prev)), u32::from(c)));
        prev = c;
    }
}

/// 字級 interpolated KN bigram 重訓，取代現行的字級 bigram：同 `qim_databuild::bigram` 的式子
/// （退回接續機率 `(N1+(•w) + 1) ÷ (N1+(••) + V)`，句首一列不算進接續次數），只是次數與折扣照
/// [`Table::build`]，可選 [`Discounting::PerCorpus`]。不知道前一字時照現行的 unigram；並存寫法的
/// 另一種寫法沒有照抄（評估的題目都寫代表寫法）。
#[derive(Debug, Clone)]
pub struct KnBigram {
    base: LanguageModel,
    table: Table,
    /// 字 → ln 接續機率；沒有的用 `unknown`。
    continuation: FxHashMap<u32, f64>,
    unknown: f64,
}

impl KnBigram {
    pub fn train(
        base: &LanguageModel,
        counts: &FxHashMap<(u64, u32), Count>,
        vocabulary: usize,
        how: Discounting,
        min_count: u64,
    ) -> (Self, TableStats) {
        let start = u64::from(u32::from(CLAUSE_START));
        let mut preceded: FxHashMap<u32, u64> = FxHashMap::default();
        let mut types = 0u64;
        for &(ctx, c) in counts.keys() {
            if ctx != start {
                *preceded.entry(c).or_default() += 1;
                types += 1;
            }
        }
        let denominator = types as f64 + vocabulary as f64;
        let continuation: FxHashMap<u32, f64> = preceded
            .iter()
            .map(|(&c, &n)| (c, ((n as f64 + 1.0) / denominator).ln()))
            .collect();
        let unknown = (1.0 / denominator).ln();
        let cont = |c: u32| continuation.get(&c).copied().unwrap_or(unknown).exp();
        let (table, stats) = Table::build(counts, how, min_count, |_, c| cont(c));
        (
            Self {
                base: base.clone(),
                table,
                continuation,
                unknown,
            },
            stats,
        )
    }

    fn log_prob_after(&self, prev: char, c: char) -> f64 {
        let cont = self
            .continuation
            .get(&u32::from(c))
            .copied()
            .unwrap_or(self.unknown);
        self.table
            .lookup(u64::from(u32::from(prev)), u32::from(c))
            .or_lower(cont)
    }

    /// 取代現行的資料包：表加上每個字的接續機率（`f32`），unigram 照現行另算一份 `f32`。
    pub fn cost(&self) -> Cost {
        let bytes = postcard::to_allocvec(&self.table.to_wire()).expect("序列化");
        let per_char = self.continuation.len() * 2 * 4;
        let load_ms = time_load(&bytes, |b| {
            Table::from_wire(postcard::from_bytes(b).expect("解得開"))
        });
        Cost {
            bytes: bytes.len() + per_char,
            memory: Some(self.table.memory() + per_char),
            load_ms,
        }
    }
}

impl SegmentModel for KnBigram {
    fn log_prob(&self, query: &SegmentQuery<'_>) -> f64 {
        let Some(prev) = query.before.last().and_then(|s| s.chars().next_back()) else {
            return query.current_log_prob;
        };
        let first = query.text.chars().next().expect("段不是空的");
        query.current_log_prob - self.base.log_prob(Some(prev), first)
            + self.log_prob_after(prev, first)
    }
}

// ---- 比照標準 KN 的字級 bigram ----

/// 維基與口語各自用 `qim_databuild::bigram::train`（權重 1，即標準的單一折扣 KN）訓練，查詢時線性內插：
/// `P = (1 − λ) P_維基 + λ P_口語`（SRILM 的 `-mix-lm` 做法）。詞內機率照現行。
#[derive(Debug, Clone)]
pub struct MixedBigram {
    base: LanguageModel,
    wiki: LanguageModel,
    colloquial: LanguageModel,
    lambda: f64,
}

impl MixedBigram {
    pub fn new(
        base: LanguageModel,
        wiki: LanguageModel,
        colloquial: LanguageModel,
        lambda: f64,
    ) -> Self {
        Self {
            base,
            wiki,
            colloquial,
            lambda,
        }
    }

    fn mixed(&self, prev: Option<char>, c: char) -> f64 {
        let w = self.wiki.log_prob(prev, c).exp();
        let k = self.colloquial.log_prob(prev, c).exp();
        ((1.0 - self.lambda) * w + self.lambda * k).ln()
    }

    /// 兩份資料包的位元組數與解開時間（取代現行的字級 bigram，不是加在它上面）。
    pub fn cost(&self) -> Cost {
        let bytes = |m: &LanguageModel| postcard::to_allocvec(m).expect("序列化");
        let (a, b) = (bytes(&self.wiki), bytes(&self.colloquial));
        let decode = |x: &[u8]| postcard::from_bytes::<LanguageModel>(x).expect("解得開");
        Cost {
            bytes: a.len() + b.len(),
            memory: None,
            load_ms: time_load(&a, decode) + time_load(&b, decode),
        }
    }
}

impl SegmentModel for MixedBigram {
    fn log_prob(&self, query: &SegmentQuery<'_>) -> f64 {
        let prev = query.before.last().and_then(|s| s.chars().next_back());
        let first = query.text.chars().next().expect("段不是空的");
        query.current_log_prob - self.base.log_prob(prev, first) + self.mixed(prev, first)
    }
}

/// [`MixedBigram`] 的兩份模型：維基、口語各一個 `Counter`（權重 1）；口語不剪枝（現行的口語權重 64
/// 讓口語的相鄰字全留下），維基照現行剪掉一次的。
pub fn train_mixed_parts(
    corpora: &[TrainCorpus],
    vocabulary: usize,
) -> Result<(LanguageModel, LanguageModel), String> {
    use qim_databuild::bigram::{Counter, MIN_COUNT, train};
    use qim_databuild::common_forms::{BothForms, ONE_WAY_FORMS, TWO_WAY_FORMS};
    let both = BothForms::new(&TWO_WAY_FORMS, &ONE_WAY_FORMS).map_err(|e| e.to_string())?;
    let part = |colloquial: bool| -> Result<LanguageModel, String> {
        let mut counter = Counter::default();
        for c in corpora.iter().filter(|c| c.colloquial == colloquial) {
            let file = File::open(&c.path).map_err(|e| format!("{}：{e}", c.path.display()))?;
            counter
                .add(BufReader::new(file), 1)
                .map_err(|e| format!("{}：{e}", c.path.display()))?;
        }
        let min = if colloquial { 1 } else { MIN_COUNT };
        Ok(train(&counter, vocabulary, min, &both).0)
    };
    Ok((part(false)?, part(true)?))
}
