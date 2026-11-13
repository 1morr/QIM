//! 詞級 trigram（plan〈排序：同一把尺〉〈字級 bigram 與詞級 trigram〉，M3 票 11b）：同步層的語言模型。建置端用現行打分把訓練句切成段、
//! 數段的 bigram 與 trigram、照標準 Kneser-Ney 算機率並剪枝，核心只查表。
//!
//! 退回式，最低一階是現行打分（字級 bigram 接前一字、詞內用詞頻，呼叫端算好交進來的 `lower`）：
//!
//! - 留下的 n-gram 直接用它的 `ln P`；
//! - 前文有留下的 n-gram、這個沒有：`ln α(前文)` 加上低一階；
//! - 前文沒有留下任何一個：照低一階。
//!
//! trigram 退回詞級 bigram，詞級 bigram 退回 `lower`。前一段不在表裡時整個照 `lower`，所以沒有統計的
//! 地方和只有字級 bigram 時一樣。

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::LOG_PROB_STEP;

/// 詞的編號要幾個位元：trigram 的前文鍵是兩個編號接起來。
const WORD_BITS: u32 = 21;

/// 詞表最多幾個詞。
pub const WORD_TRIGRAM_MAX_WORDS: usize = 1 << WORD_BITS;

/// 一個前文留下的 n-gram：建置端交給 [`WordTrigram::new`]。編號是傳進去的詞表的索引。
#[derive(Debug, Clone, PartialEq)]
pub struct NgramRow<C> {
    /// 前文：bigram 是前一段，trigram 是前兩段（`[前前一段, 前一段]`）。
    pub context: C,
    /// 這個前文沒留下的後一段乘的 `ln α`（≤ 0）。
    pub backoff: f64,
    /// 留下的後一段與它的 `ln P`。
    pub next: Vec<(u32, f64)>,
}

/// 詞級 trigram 語言模型。詞的編號是它在詞表的位置，照建置端給的順序（常見的段在前，資料包裡
/// 編號的差值小）；查文字用載入時建的雜湊索引（lattice 每條邊都查一次，二分搜尋讓每鍵慢一倍）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "Wire", into = "Wire")]
pub struct WordTrigram {
    /// 詞表的文字接成一串；第 `i` 個詞是 `text[offsets[i]..offsets[i + 1]]`。
    text: String,
    offsets: Vec<u32>,
    /// 開放定址的雜湊表（[`WordTrigram::id`] 用）：格子放詞的編號，[`EMPTY`] 是空格；長度是 2 的冪、
    /// 至少詞數的兩倍，線性探查。
    slots: Vec<u32>,
    bigram: Table,
    trigram: Table,
}

/// 一張表查到什麼。
#[derive(Debug, Clone, Copy, PartialEq)]
enum Lookup {
    Hit(f64),
    Backoff(f64),
    Unseen,
}

impl Lookup {
    fn or_lower(self, lower: f64) -> f64 {
        match self {
            Self::Hit(lp) => lp,
            Self::Backoff(alpha) => alpha + lower,
            Self::Unseen => lower,
        }
    }
}

/// 一階 n-gram：依前文鍵分列，每列的後一段編號遞增。
#[derive(Debug, Clone, Default, PartialEq)]
struct Table {
    contexts: Vec<u64>,
    /// 每列的 `ln α`。
    backoff: Vec<f32>,
    /// 第 `i` 列在 `next[rows[i]..rows[i + 1]]`。
    rows: Vec<u32>,
    next: Vec<u32>,
    /// 和 `next` 對齊的量化 `ln P`（[`LOG_PROB_STEP`]）。
    log_prob: Vec<u8>,
}

/// 建表時的一列：前文鍵、`ln α`、留下的後一段與 `ln P`。
type RawRow = (u64, f64, Vec<(u32, f64)>);

fn pair(u: u32, v: u32) -> u64 {
    (u64::from(u) << WORD_BITS) | u64::from(v)
}

impl WordTrigram {
    /// `vocabulary` 是詞表（文字不重複，最多 [`WORD_TRIGRAM_MAX_WORDS`] 個），詞的編號是它的索引；
    /// 兩階的列用這個編號，同一個前文只出現一列。
    ///
    /// # Panics
    ///
    /// 詞表有重複的文字、太多詞，或列裡的編號超出詞表時（建置端的錯）。
    pub fn new(
        vocabulary: Vec<String>,
        bigrams: Vec<NgramRow<u32>>,
        trigrams: Vec<NgramRow<[u32; 2]>>,
    ) -> Self {
        let (text, offsets, slots) = index(&vocabulary).expect("詞表合用");
        let words = vocabulary.len();
        let check = |id: u32| {
            assert!((id as usize) < words, "編號 {id} 超出詞表");
            id
        };
        let bigram = Table::new(
            bigrams
                .into_iter()
                .map(|r| (u64::from(check(r.context)), r.backoff, r.next)),
            check,
        );
        let trigram = Table::new(
            trigrams.into_iter().map(|r| {
                let [u, v] = r.context;
                (pair(check(u), check(v)), r.backoff, r.next)
            }),
            check,
        );
        Self {
            text,
            offsets,
            slots,
            bigram,
            trigram,
        }
    }

    /// 詞表的詞數。
    pub fn len(&self) -> usize {
        self.offsets.len() - 1
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn word(&self, id: usize) -> &str {
        &self.text[self.offsets[id] as usize..self.offsets[id + 1] as usize]
    }

    /// 詞表裡 `text` 的編號；不在詞表時是 `None`。
    pub fn id(&self, text: &str) -> Option<u32> {
        let mask = self.slots.len() - 1;
        let mut i = hash(text) as usize & mask;
        loop {
            match self.slots[i] {
                EMPTY => return None,
                id if self.word(id as usize) == text => return Some(id),
                _ => i = (i + 1) & mask,
            }
        }
    }

    /// 段 `text` 接在 `before`（前面的段，最近的在最後，只看最後兩段）後面的 ln 機率；`lower` 是
    /// 現行打分的靜態 ln 機率（首字 bigram 乘詞內機率）。
    pub fn log_prob(&self, before: &[&str], text: &str, lower: f64) -> f64 {
        let mut back = before.iter().rev();
        let v = back.next().and_then(|t| self.id(t));
        let u = back.next().and_then(|t| self.id(t));
        self.score(u, v, self.id(text), lower)
    }

    /// [`WordTrigram::log_prob`] 用編號查：`u`、`v` 是前兩段、前一段，`w` 是這一段；不在詞表是 `None`。
    pub(crate) fn score(&self, u: Option<u32>, v: Option<u32>, w: Option<u32>, lower: f64) -> f64 {
        let Some(v) = v else {
            return lower;
        };
        // 不在詞表的段任何 n-gram 都沒有它：前文有留下的就乘 α。
        let w = w.unwrap_or(u32::MAX);
        let bigram = self.bigram.lookup(u64::from(v), w).or_lower(lower);
        match u {
            Some(u) => self.trigram.lookup(pair(u, v), w).or_lower(bigram),
            None => bigram,
        }
    }
}

impl Table {
    /// 每列（前文鍵、`ln α`、留下的後一段與 `ln P`）；`id` 把後一段換成新編號。
    fn new(rows: impl Iterator<Item = RawRow>, id: impl Fn(u32) -> u32) -> Self {
        let mut rows: Vec<RawRow> = rows.collect();
        rows.sort_by_key(|r| r.0);
        let mut table = Self {
            rows: vec![0],
            ..Self::default()
        };
        for (context, backoff, next) in rows {
            assert!(
                table.contexts.last().is_none_or(|&last| last < context),
                "同一個前文有兩列"
            );
            let mut next: Vec<(u32, u8)> = next
                .into_iter()
                .map(|(n, lp)| (id(n), quantize(lp)))
                .collect();
            next.sort_by_key(|&(n, _)| n);
            table.contexts.push(context);
            #[allow(clippy::cast_possible_truncation)] // 存成 f32 是刻意的精度取捨
            table.backoff.push(backoff as f32);
            for (n, q) in next {
                table.next.push(n);
                table.log_prob.push(q);
            }
            table.rows.push(index_u32(table.next.len()));
        }
        table
    }

    fn lookup(&self, context: u64, next: u32) -> Lookup {
        let Ok(i) = self.contexts.binary_search(&context) else {
            return Lookup::Unseen;
        };
        let (from, to) = (self.rows[i] as usize, self.rows[i + 1] as usize);
        match self.next[from..to].binary_search(&next) {
            Ok(k) => Lookup::Hit(dequantize(self.log_prob[from + k])),
            Err(_) => Lookup::Backoff(f64::from(self.backoff[i])),
        }
    }

    /// 存成前文鍵與後一段編號的差（postcard 的 varint 讓它小）。
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
            for &n in &self.next[w[0] as usize..w[1] as usize] {
                next_gaps.push(n - last);
                last = n;
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

    fn from_wire(w: TableWire, words: usize) -> Result<Self, WireError> {
        if w.backoff.len() != w.context_gaps.len()
            || w.row_lengths.len() != w.context_gaps.len()
            || w.log_prob.len() != w.next_gaps.len()
        {
            return Err(WireError("表的欄長度對不上".into()));
        }
        let mut contexts = Vec::with_capacity(w.context_gaps.len());
        let mut last = 0u64;
        for (i, gap) in w.context_gaps.into_iter().enumerate() {
            if i > 0 && gap == 0 {
                return Err(WireError("前文沒有遞增".into()));
            }
            last = last
                .checked_add(gap)
                .ok_or_else(|| WireError("前文鍵溢位".into()))?;
            contexts.push(last);
        }
        let mut rows = Vec::with_capacity(w.row_lengths.len() + 1);
        rows.push(0u32);
        let mut next = Vec::with_capacity(w.next_gaps.len());
        let mut gaps = w.next_gaps.into_iter();
        for len in w.row_lengths {
            let mut last = 0u32;
            for k in 0..len {
                let gap = gaps
                    .next()
                    .ok_or_else(|| WireError("列長加起來比後一段多".into()))?;
                if k > 0 && gap == 0 {
                    return Err(WireError("後一段沒有遞增".into()));
                }
                last = last
                    .checked_add(gap)
                    .filter(|&n| (n as usize) < words)
                    .ok_or_else(|| WireError("後一段的編號超出詞表".into()))?;
                next.push(last);
            }
            rows.push(index_u32(next.len()));
        }
        if gaps.next().is_some() {
            return Err(WireError("列長加起來比後一段少".into()));
        }
        Ok(Self {
            contexts,
            backoff: w.backoff,
            rows,
            next,
            log_prob: w.log_prob,
        })
    }
}

/// [`WordTrigram`] 雜湊表的空格（詞的編號到不了這麼大，[`WORD_TRIGRAM_MAX_WORDS`]）。
const EMPTY: u32 = u32::MAX;

/// FNV-1a（64 位元）：短字串快、不必加依賴；詞表是建置端給的，不必防碰撞攻擊。
fn hash(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// 詞表接成一串文字、各詞的位移與雜湊表；太多詞、文字太長或有重複時回錯。
fn index(vocabulary: &[String]) -> Result<(String, Vec<u32>, Vec<u32>), WireError> {
    if vocabulary.len() > WORD_TRIGRAM_MAX_WORDS {
        return Err(WireError(format!("詞表超過 {WORD_TRIGRAM_MAX_WORDS} 個詞")));
    }
    let mut text = String::with_capacity(vocabulary.iter().map(String::len).sum());
    let mut offsets = Vec::with_capacity(vocabulary.len() + 1);
    offsets.push(0);
    for word in vocabulary {
        text.push_str(word);
        offsets.push(u32::try_from(text.len()).map_err(|_| WireError("詞表的文字太長".into()))?);
    }
    let mut slots = vec![EMPTY; (vocabulary.len() * 2).next_power_of_two().max(2)];
    let mask = slots.len() - 1;
    for (id, word) in vocabulary.iter().enumerate() {
        let mut i = hash(word) as usize & mask;
        while slots[i] != EMPTY {
            if vocabulary[slots[i] as usize] == *word {
                return Err(WireError("詞表有重複的詞".into()));
            }
            i = (i + 1) & mask;
        }
        slots[i] = index_u32(id);
    }
    Ok((text, offsets, slots))
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // 已夾在 0–255
fn quantize(log_prob: f64) -> u8 {
    (-log_prob / LOG_PROB_STEP).round().clamp(0.0, 255.0) as u8
}

fn dequantize(q: u8) -> f64 {
    -f64::from(q) * LOG_PROB_STEP
}

fn index_u32(i: usize) -> u32 {
    u32::try_from(i).expect("放得進 u32")
}

/// 資料包裡的形式：詞表照編號的順序，表存差值。
#[derive(Serialize, Deserialize)]
struct Wire {
    vocabulary: Vec<String>,
    bigram: TableWire,
    trigram: TableWire,
}

#[derive(Serialize, Deserialize)]
struct TableWire {
    context_gaps: Vec<u64>,
    backoff: Vec<f32>,
    row_lengths: Vec<u32>,
    next_gaps: Vec<u32>,
    log_prob: Vec<u8>,
}

/// 資料包內容不合（詞表有重複、表的欄對不上）；[`crate::Pack::from_bytes`] 報成損壞。
#[derive(Debug)]
struct WireError(String);

impl fmt::Display for WireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "詞級 trigram：{}", self.0)
    }
}

impl From<WordTrigram> for Wire {
    fn from(m: WordTrigram) -> Self {
        Self {
            vocabulary: (0..m.len()).map(|i| m.word(i).to_string()).collect(),
            bigram: m.bigram.to_wire(),
            trigram: m.trigram.to_wire(),
        }
    }
}

impl TryFrom<Wire> for WordTrigram {
    type Error = WireError;

    fn try_from(w: Wire) -> Result<Self, WireError> {
        let words = w.vocabulary.len();
        let (text, offsets, slots) = index(&w.vocabulary)?;
        Ok(Self {
            text,
            offsets,
            slots,
            bigram: Table::from_wire(w.bigram, words)?,
            trigram: Table::from_wire(w.trigram, words)?,
        })
    }
}
