//! 字級 bigram 的訓練（plan §4.3）：各語料 train 切分的句子 → interpolated Kneser-Ney →
//! 依次數剪枝 → [`LanguageModel`]。
//!
//! - 一行一句，bigram 只數句內相鄰的兩字，不跨句、不加句尾記號。句首另算一列：每句的第一字
//!   接在 [`CLAUSE_START`] 後面，前文以標點結尾時組字開頭的字用它（plan §2.3）；它的次數不算進
//!   字次數、`N1+(•w)` 與 `D`，所以其他字的機率和沒有它時相同。不知道前文時組字開頭的字用
//!   unigram（[`CharModel::start`]）：使用者的組字多半不在句首。
//! - `P(w | v) = max(c(v,w) − D, 0) ÷ c(v) + D × N1+(v•) ÷ c(v) × P_cont(w)`，
//!   `D = n1 ÷ (n1 + 2 n2)`（Chen & Goodman 1998 的單一折扣）。
//! - `P_cont(w) = (N1+(•w) + 1) ÷ (N1+(••) + V)`，`P_start(w) = (c(w) + 1) ÷ (N + V)`，
//!   `V` 是連打字集的字數；加一讓沒出現過的字也有機率。
//! - 剪枝：只留 `c(v,w) ≥` [`MIN_COUNT`] 的 bigram，`α(v)` 重算成讓 `Σ_w P(w | v) = 1`。
//! - 並存寫法（`common_forms::BothForms`）：次數歸到代表寫法再訓練，訓練完把代表寫法的 unigram 與
//!   含它的 bigram 照抄給另一種寫法，兩種寫法的機率相同（brief §4.1）。
//!
//! 浮點運算都依字的順序做，同樣的語料算出位元組相同的資料包。

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::io::{self, BufRead};

use qim_core::{CLAUSE_START, CharModel, LanguageModel};

use crate::common_forms::BothForms;

/// 剪枝門檻：bigram 出現至少這麼多次才留下（plan §4.3：資料包 ≤ [`MAX_BYTES`]）。
pub const MIN_COUNT: u64 = 2;

/// 資料包大小上限（plan §4.3，Gboard 文獻的 5–10 MB 預算）；`--verify` 檢查。
pub const MAX_BYTES: usize = 8_000_000;

/// 各語料的字與相鄰字的次數，已乘上語料權重。
#[derive(Debug, Default)]
pub struct Counter {
    unigrams: HashMap<char, u64>,
    bigrams: HashMap<(char, char), u64>,
    /// 每句的第一字。
    starts: HashMap<char, u64>,
}

impl Counter {
    /// 讀一份一行一句的 train 切分，每個次數乘上 `weight`。
    pub fn add(&mut self, sentences: impl BufRead, weight: u64) -> io::Result<()> {
        for line in sentences.lines() {
            let line = line?;
            if let Some(first) = line.chars().next() {
                *self.starts.entry(first).or_default() += weight;
            }
            let mut prev = None;
            for c in line.chars() {
                *self.unigrams.entry(c).or_default() += weight;
                if let Some(p) = prev {
                    *self.bigrams.entry((p, c)).or_default() += weight;
                }
                prev = Some(c);
            }
        }
        Ok(())
    }
}

/// 訓練的統計，印在 `cargo xtask data` 的輸出。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Summary {
    /// 有統計的字數。
    pub characters: usize,
    /// 不重複的 bigram 數。
    pub bigrams: usize,
    /// 剪枝門檻：留下次數 ≥ 它的 bigram。
    pub min_count: u64,
    /// 剪枝後留下的 bigram 數，不含句首與照抄給並存寫法的。
    pub kept: usize,
    /// 剪枝後留下的句首 bigram 數，不含照抄給並存寫法的。
    pub clause_starts: usize,
    /// 照抄給並存寫法另一種寫法的 bigram 數。
    pub copied: usize,
    /// Kneser-Ney 的折扣 `D`。
    pub discount: f64,
}

impl fmt::Display for Summary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "bigram：{} 字、不重複的相鄰字 {} 對，留下次數 ≥ {} 的 {} 對（D = {:.3}）、句首 {} 字，\
             並存寫法另抄 {} 對",
            self.characters,
            self.bigrams,
            self.min_count,
            self.kept,
            self.discount,
            self.clause_starts,
            self.copied
        )
    }
}

/// 由次數訓練出語言模型；`vocabulary` 是連打字集的字數（`V`），只留次數 ≥ `min_count` 的
/// bigram（建置用 [`MIN_COUNT`]）；`both` 的另一種寫法歸到代表寫法計數，訓練完照抄。
pub fn train(
    counter: &Counter,
    vocabulary: usize,
    min_count: u64,
    both: &BothForms,
) -> (LanguageModel, Summary) {
    let representative = |c: char| both.representative(c);
    let mut unigrams: BTreeMap<char, u64> = BTreeMap::new();
    for (&c, &n) in &counter.unigrams {
        *unigrams.entry(representative(c)).or_default() += n;
    }
    let unigrams: Vec<(char, u64)> = unigrams.into_iter().collect();
    let mut merged: BTreeMap<(char, char), u64> = BTreeMap::new();
    for (&(p, c), &n) in &counter.bigrams {
        *merged
            .entry((representative(p), representative(c)))
            .or_default() += n;
    }
    let bigrams: Vec<((char, char), u64)> = merged.into_iter().collect();
    let mut starts: BTreeMap<char, u64> = BTreeMap::new();
    for (&c, &n) in &counter.starts {
        *starts.entry(representative(c)).or_default() += n;
    }

    let v = vocabulary as f64;
    let total: u64 = unigrams.iter().map(|(_, n)| n).sum();
    let n1 = bigrams.iter().filter(|(_, n)| *n == 1).count() as f64;
    let n2 = bigrams.iter().filter(|(_, n)| *n == 2).count() as f64;
    let discount = if n1 > 0.0 { n1 / (n1 + 2.0 * n2) } else { 0.5 };

    // 前一字 v：c(v)、N1+(v•)；後一字 w：N1+(•w)。
    let mut context: BTreeMap<char, (u64, u64)> = BTreeMap::new();
    let mut preceded: BTreeMap<char, u64> = BTreeMap::new();
    for &((p, c), n) in &bigrams {
        let entry = context.entry(p).or_default();
        entry.0 += n;
        entry.1 += 1;
        *preceded.entry(c).or_default() += 1;
    }
    let types = bigrams.len() as f64;
    let continuation =
        |c: char| (preceded.get(&c).copied().unwrap_or(0) as f64 + 1.0) / (types + v);

    // 留下的 bigram 的機率，以及每個前一字留下的 P 與 P_cont 總和（重算 α 用）。
    let mut kept = Vec::new();
    let mut kept_mass: BTreeMap<char, (f64, f64)> = BTreeMap::new();
    for &((p, c), n) in &bigrams {
        if n < min_count {
            continue;
        }
        let (count, followers) = context[&p];
        let count = count as f64;
        let lambda = discount * followers as f64 / count;
        let prob = (n as f64 - discount).max(0.0) / count + lambda * continuation(c);
        let mass = kept_mass.entry(p).or_default();
        mass.0 += prob;
        mass.1 += continuation(c);
        kept.push((p, c, prob.ln()));
    }
    let kept_bigrams = kept.len();
    // 句首一列：同樣的式子，前一字是句首，D 與 P_cont 用句內的。
    let start_count: u64 = starts.values().sum();
    if start_count > 0 {
        let count = start_count as f64;
        let lambda = discount * starts.len() as f64 / count;
        for (&c, &n) in &starts {
            if n < min_count {
                continue;
            }
            let prob = (n as f64 - discount).max(0.0) / count + lambda * continuation(c);
            let mass = kept_mass.entry(CLAUSE_START).or_default();
            mass.0 += prob;
            mass.1 += continuation(c);
            kept.push((CLAUSE_START, c, prob.ln()));
        }
    }
    // 照抄給並存寫法：前一字、後一字各自換或不換成另一種寫法。
    let forms = |c: char| std::iter::once(c).chain(both.other(c));
    let copies: Vec<(char, char, f64)> = kept
        .iter()
        .flat_map(|&(p, c, lp)| {
            forms(p)
                .flat_map(move |p2| forms(c).map(move |c2| (p2, c2, lp)))
                .filter(move |&(p2, c2, _)| (p2, c2) != (p, c))
        })
        .collect();

    let model = |c: char, n: u64| {
        let (p, q) = kept_mass.get(&c).copied().unwrap_or((0.0, 0.0));
        CharModel {
            start: ((n as f64 + 1.0) / (total as f64 + v)).ln(),
            continuation: continuation(c).ln(),
            backoff: ((1.0 - p) / (1.0 - q)).ln(),
        }
    };
    let unknown = CharModel {
        start: (1.0 / (total as f64 + v)).ln(),
        continuation: (1.0 / (types + v)).ln(),
        backoff: 0.0,
    };
    let summary = Summary {
        characters: unigrams.len(),
        bigrams: bigrams.len(),
        min_count,
        kept: kept_bigrams,
        clause_starts: kept.len() - kept_bigrams,
        copied: copies.len(),
        discount,
    };
    let chars = unigrams.iter().flat_map(|&(c, n)| {
        let m = model(c, n);
        forms(c).map(move |c| (c, m))
    });
    // 句首打不出，它的 unigram 與接續機率用不到，照未知字的；退回權重照它那一列算。
    let clause_start = (start_count > 0).then(|| {
        let backoff = model(CLAUSE_START, 0).backoff;
        (CLAUSE_START, CharModel { backoff, ..unknown })
    });
    let chars = chars.chain(clause_start);
    (
        LanguageModel::new(chars, unknown, kept.into_iter().chain(copies)),
        summary,
    )
}
