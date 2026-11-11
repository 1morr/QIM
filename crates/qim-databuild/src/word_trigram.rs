//! 詞級 trigram 的訓練（plan §4.3，M3 票 11b；做法與數字在 `scripts/experiments/model-ladder/`
//! README〈同步層〉，票 08 量過）：各語料 train 切分的句子 → 用現行打分切成段 → 段的 bigram 與
//! trigram，各語料各算折扣的 interpolated Kneser-Ney → 依次數剪枝 → [`WordTrigram`]。
//!
//! - 段：文字已知的 Viterbi，分數同 lattice 的邊（字級 bigram 接前一字、詞庫的詞乘詞內機率、
//!   β、γ 照方案），和輸入法解碼時的切法一致；每句前面接句首（[`CLAUSE_START`] 當一段）。
//! - 機率：`P(w | h) = max(c(h,w) − d, 0) ÷ c(h) + (Σ d ÷ c(h)) × P_低(w | h')`。`d` 比照標準 KN：
//!   每份語料在自己的原始次數上算 `D_k = n1 ÷ (n1 + 2 n2)`，一種 n-gram 扣 `Σ D_k × w_k`（出現在
//!   語料 k 的才算），等於各語料分開訓練再依權重把次數加起來。字級 bigram 的「乘權重後算 D」用在
//!   高一階時口語首選低 1.6–3.5 點（票 08）。
//! - 低一階：trigram 退回詞級 bigram，詞級 bigram 退回現行打分（[`CurrentScoring`]）。
//! - 剪枝：只留乘上權重後次數 ≥ 門檻的，`α(h) = (1 − Σ 留下的 P) ÷ (1 − Σ 留下的 P_低)`，夾在 1 以下：
//!   罕用的前文常把低一階幾乎全給了留下的那一兩個，分母近 0、又受量化誤差左右，α 會爆大，沒留下的
//!   反而比低一階高（票 08 的除錯）。
//!
//! 詞表依序編號、表依鍵排序後才算浮點，同樣的語料算出位元組相同的資料包。

use std::fmt;
use std::hash::Hash;
use std::io::{self, BufRead};

use qim_core::{
    CLAUSE_START, LOG_PROB_STEP, LanguageModel, Lexicon, NgramRow, WORD_TRIGRAM_MAX_WORDS,
    WordTrigram,
};
use rustc_hash::FxHashMap;

/// 剪枝門檻：乘上權重後次數至少這麼多才留下（口語出現一次就是 64）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pruning {
    pub min_bigram: u64,
    pub min_trigram: u64,
}

/// 預算版（票 08 的 `word3:8:8`）：加上字級 bigram ≤ 8 MB（spec〈採用門檻〉）。
pub const BUDGET: Pruning = Pruning {
    min_bigram: 8,
    min_trigram: 8,
};

/// 放寬版（票 08 的 `word3:2:4`）：常駐估計 35 MB，iOS 鍵盤擴充的峰值增量要 ≤ 60 MB。
pub const LARGE: Pruning = Pruning {
    min_bigram: 2,
    min_trigram: 4,
};

/// 建出的一份詞級 trigram：id、種類名與剪枝門檻。方案 `[packs]` 指名其中一份當預設，兩份都建，
/// 設定頁可以切換（plan §4.6）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Variant {
    pub id: &'static str,
    /// 給人看的種類名（`cargo xtask data` 印的、`--verify` 比 sha256 的）。
    pub name: &'static str,
    pub pruning: Pruning,
}

/// 預算版與放寬版。
pub const VARIANTS: [Variant; 2] = [
    Variant {
        id: "zh-hant-word-trigram",
        name: "詞級 trigram",
        pruning: BUDGET,
    },
    Variant {
        id: "zh-hant-word-trigram-large",
        name: "詞級 trigram（放寬）",
        pruning: LARGE,
    },
];

/// 預算版的 id：`--verify` 檢查它加上字級 bigram 不超過 [`crate::bigram::MAX_BYTES`]。
pub const BUDGET_ID: &str = VARIANTS[0].id;

/// 現行打分要的東西：字級 bigram、詞庫的詞內機率、β、γ。切訓練語料與詞級 bigram 的低一階都用它。
#[derive(Debug, Clone)]
pub struct CurrentScoring<'a> {
    base: &'a LanguageModel,
    /// 詞庫的詞 → 第一字之後的 ln 機率（plan §2.3：`ln P_詞頻 − ln P_unigram(第一字)`）。
    rest: FxHashMap<Box<str>, f64>,
    /// 最長的詞幾個字。
    longest: usize,
    word_bonus: f64,
    segment_penalty: f64,
}

impl<'a> CurrentScoring<'a> {
    /// `lexicon` 是方案的詞庫；分母同 `Engine`：每個不重複的詞算一次的詞頻總和。
    pub fn new(
        base: &'a LanguageModel,
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
        // 依詞排序後加總，浮點的結果不看雜湊表的順序。
        let mut sorted: Vec<(Box<str>, u64)> = frequency.into_iter().collect();
        sorted.sort_unstable();
        let total = (sorted.iter().map(|(_, f)| f).sum::<u64>().max(1) as f64).ln();
        let longest = sorted
            .iter()
            .map(|(t, _)| t.chars().count())
            .max()
            .unwrap_or(1)
            .max(1);
        let rest = sorted
            .into_iter()
            .map(|(text, f)| {
                let first = text.chars().next().expect("詞不是空的");
                let r = (f.max(1) as f64).ln() - total - base.log_prob(None, first);
                (text, r)
            })
            .collect();
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

/// 一種 n-gram 乘上權重的次數與它的折扣。
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Count {
    pub weighted: u64,
    /// 各語料的 `Σ D_k × w_k`。
    pub discount: f64,
}

/// `D = n1 ÷ (n1 + 2 n2)`；沒有出現一次的時是 0.5（同 [`crate::bigram`]）。
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

/// 一份語料的原始次數併進合計：次數乘權重，折扣加上這份語料的 `D × 權重`。
pub fn merge<K: Hash + Eq>(total: &mut FxHashMap<K, Count>, local: FxHashMap<K, u32>, weight: u64) {
    let d = discount_of(local.values().map(|&n| u64::from(n)));
    for (k, n) in local {
        let entry = total.entry(k).or_default();
        entry.weighted += u64::from(n) * weight;
        entry.discount += d * weight as f64;
    }
}

/// 詞的編號要幾個位元：trigram 的前文鍵是兩個編號接起來（同核心，[`WORD_TRIGRAM_MAX_WORDS`] 是 2 的這麼多次方）。
const WORD_BITS: u32 = WORD_TRIGRAM_MAX_WORDS.trailing_zeros();

fn word_pair(u: u32, v: u32) -> u64 {
    (u64::from(u) << WORD_BITS) | u64::from(v)
}

/// 切好的語料的詞級次數：詞依第一次出現的順序編號（0 是句首），bigram 的鍵是（前一詞, 詞），
/// trigram 的前文是兩個詞編號（[`word_pair`]）。
#[derive(Debug, Default)]
pub struct WordCounts {
    pub vocabulary: Vec<Box<str>>,
    pub bigrams: FxHashMap<(u64, u32), Count>,
    pub trigrams: FxHashMap<(u64, u32), Count>,
}

/// 用 `scoring` 切各語料（一行一句，與權重），數詞級 bigram 與 trigram。
pub fn count(
    corpora: Vec<(Box<dyn BufRead + '_>, u64)>,
    scoring: &CurrentScoring,
) -> io::Result<WordCounts> {
    let mut ids: FxHashMap<Box<str>, u32> = FxHashMap::default();
    let mut vocabulary: Vec<Box<str>> = Vec::new();
    let mut intern = |text: &str| -> u32 {
        if let Some(&id) = ids.get(text) {
            return id;
        }
        let id = u32::try_from(vocabulary.len()).expect("放得進 u32");
        ids.insert(text.into(), id);
        vocabulary.push(text.into());
        id
    };
    intern(&CLAUSE_START.to_string());
    let mut bigrams = FxHashMap::default();
    let mut trigrams = FxHashMap::default();
    let mut line_ids = Vec::new();
    // 每份語料分開數，才能算各自的 D。
    for (reader, weight) in corpora {
        let mut local_bi: FxHashMap<(u64, u32), u32> = FxHashMap::default();
        let mut local_tri: FxHashMap<(u64, u32), u32> = FxHashMap::default();
        for line in reader.lines() {
            let line = line?;
            line_ids.clear();
            line_ids.push(0);
            for (i, j) in scoring.segment(&line) {
                line_ids.push(intern(&line[i..j]));
            }
            for w in line_ids.windows(2) {
                *local_bi.entry((u64::from(w[0]), w[1])).or_default() += 1;
            }
            for w in line_ids.windows(3) {
                *local_tri.entry((word_pair(w[0], w[1]), w[2])).or_default() += 1;
            }
        }
        merge(&mut bigrams, local_bi, weight);
        merge(&mut trigrams, local_tri, weight);
    }
    assert!(
        vocabulary.len() <= WORD_TRIGRAM_MAX_WORDS,
        "詞表超過核心收得下的 {WORD_TRIGRAM_MAX_WORDS} 個詞"
    );
    Ok(WordCounts {
        vocabulary,
        bigrams,
        trigrams,
    })
}

/// 一階 n-gram 剪枝後留下的表：依前文鍵分列，每列的後一段遞增。機率量化成 `u8`（同核心），
/// trigram 的低一階要查剪枝、量化後的詞級 bigram，和核心查到的一樣。
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
            Self::Hit(lp) => lp,
            Self::Backoff(alpha) => alpha + lower,
            Self::Unseen => lower,
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

impl Table {
    /// 由次數建表：只留 `weighted ≥ min_count` 的；`lower(前文, 後一段)` 是低一階的機率（不取 ln）。
    pub fn build(
        counts: &FxHashMap<(u64, u32), Count>,
        min_count: u64,
        lower: impl Fn(u64, u32) -> f64,
    ) -> (Self, TableStats) {
        let mut keys: Vec<(u64, u32)> = counts.keys().copied().collect();
        keys.sort_unstable();
        let discount = |c: &Count| c.discount.min(c.weighted as f64);
        let mut table = Self {
            rows: vec![0],
            ..Self::default()
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
                let (num, den) = (1.0 - kept_p, 1.0 - kept_lower);
                let alpha = num.max(1e-9) / den.max(1e-9);
                if alpha > 1.0 {
                    stats.clamped += 1;
                }
                #[allow(clippy::cast_possible_truncation)] // 存成 f32 是刻意的精度取捨，同核心
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

    /// 每列：前文鍵、`ln α`、留下的後一段與 `ln P`（量化過的值）。
    fn rows(&self) -> impl Iterator<Item = (u64, f64, Vec<(u32, f64)>)> + '_ {
        (0..self.contexts.len()).map(|i| {
            let (from, to) = (self.rows[i] as usize, self.rows[i + 1] as usize);
            let next = (from..to)
                .map(|k| (self.next[k], dequantize(self.log_prob[k])))
                .collect();
            (self.contexts[i], f64::from(self.backoff[i]), next)
        })
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // 已夾在 0–255
fn quantize(log_prob: f64) -> u8 {
    (-log_prob / LOG_PROB_STEP).round().clamp(0.0, 255.0) as u8
}

fn dequantize(q: u8) -> f64 {
    -f64::from(q) * LOG_PROB_STEP
}

/// 建置統計，`cargo xtask data` 印出。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    /// 資料包的 id。
    pub id: String,
    pub pruning: Pruning,
    /// 切出的不重複段數。
    pub segments: usize,
    /// 留下的表用到的段數（資料包的詞表）。
    pub words: usize,
    pub bigram: TableStats,
    pub trigram: TableStats,
}

/// 由次數訓練、剪枝成 id 是 `id` 的資料包內容：只留下的表用到的段進詞表。
pub fn train(
    scoring: &CurrentScoring,
    counts: &WordCounts,
    id: &str,
    pruning: Pruning,
) -> (WordTrigram, Summary) {
    let vocab = &counts.vocabulary;
    let last_char = |id: u32| vocab[id as usize].chars().next_back();
    let low = |v: u32, w: u32| {
        scoring
            .log_prob(last_char(v), &vocab[w as usize])
            .expect("切出來的段都是字或詞")
    };
    let (bigram, bigram_stats) = Table::build(&counts.bigrams, pruning.min_bigram, |ctx, w| {
        low(u32::try_from(ctx).expect("bigram 的前文是一個詞"), w).exp()
    });
    let mask = (1u64 << WORD_BITS) - 1;
    let (trigram, trigram_stats) = Table::build(&counts.trigrams, pruning.min_trigram, |ctx, w| {
        let v = (ctx & mask) as u32;
        bigram.lookup(u64::from(v), w).or_lower(low(v, w)).exp()
    });
    // 只留表用到的段，依原編號的順序重編。
    let mut used = vec![false; vocab.len()];
    used[0] = true;
    for (ctx, _, next) in bigram.rows() {
        used[ctx as usize] = true;
        for (n, _) in next {
            used[n as usize] = true;
        }
    }
    for (ctx, _, next) in trigram.rows() {
        used[(ctx >> WORD_BITS) as usize] = true;
        used[(ctx & mask) as usize] = true;
        for (n, _) in next {
            used[n as usize] = true;
        }
    }
    let mut new_id = vec![u32::MAX; vocab.len()];
    let mut words = Vec::new();
    for (old, _) in used.iter().enumerate().filter(|(_, u)| **u) {
        new_id[old] = u32::try_from(words.len()).expect("放得進 u32");
        words.push(vocab[old].to_string());
    }
    let renumber = |old: u32| new_id[old as usize];
    let row = |next: Vec<(u32, f64)>| next.into_iter().map(|(n, lp)| (renumber(n), lp)).collect();
    let bigrams = bigram
        .rows()
        .map(|(ctx, backoff, next)| NgramRow {
            context: renumber(u32::try_from(ctx).expect("一個詞")),
            backoff,
            next: row(next),
        })
        .collect();
    let trigrams = trigram
        .rows()
        .map(|(ctx, backoff, next)| NgramRow {
            context: [
                renumber((ctx >> WORD_BITS) as u32),
                renumber((ctx & mask) as u32),
            ],
            backoff,
            next: row(next),
        })
        .collect();
    let summary = Summary {
        id: id.into(),
        pruning,
        segments: vocab.len(),
        words: words.len(),
        bigram: bigram_stats,
        trigram: trigram_stats,
    };
    (WordTrigram::new(words, bigrams, trigrams), summary)
}

impl fmt::Display for Summary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (bi, tri) = (&self.bigram, &self.trigram);
        let pruning = format!(
            "（門檻 {}:{}）",
            self.pruning.min_bigram, self.pruning.min_trigram
        );
        write!(
            f,
            "詞級 trigram {}{pruning}：切出 {} 種段，詞表 {} 個；bigram 不重複 {}、留下 {}（前文 {}，α 夾成 1 的 {}）；\
             trigram 不重複 {}、留下 {}（前文 {}，α 夾成 1 的 {}）",
            self.id,
            self.segments,
            self.words,
            bi.types,
            bi.kept,
            bi.contexts,
            bi.clamped,
            tri.types,
            tri.kept,
            tri.contexts,
            tri.clamped
        )
    }
}
