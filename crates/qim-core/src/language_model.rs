//! 字級 bigram（plan §2.3、§4.3）：建置端訓練好 interpolated Kneser-Ney 並剪枝，核心只查表。
//!
//! 形式同 ARPA 的退回式：留下的 bigram 存 `ln P(c | prev)`；沒留下的用
//! `ln α(prev) + ln P_cont(c)`。沒有前一字（組字的開頭）時用字頻的 unigram `start`。
//! 句首（[`CLAUSE_START`]）也是一個前一字：前文以標點或空白結尾時，組字開頭的字接它。

use serde::{Deserialize, Serialize};

/// bigram 機率量化的級距（自然對數）：存成 `u8` 的 `round(−ln P ÷ 級距)`，255 級蓋到約 −21。
/// 誤差最多半級（約 0.04），比 β、γ 的量級小得多。
pub const LOG_PROB_STEP: f64 = 1.0 / 12.0;

/// 句首記號（plan §2.3、§4.3）：當前一字時，`P(c | 句首)` 是 c 當一句第一字的機率。
/// 訓練時每句的第一字接在它後面；它不是打得出的字，連打的路徑裡不會有它。
pub const CLAUSE_START: char = '\u{2}';

/// 一個字的 unigram 與退回權重，都是自然對數。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CharModel {
    /// 沒有前一字時的 `ln P(c)`：字頻的 unigram。
    pub start: f64,
    /// 退回用的 `ln P_cont(c)`：Kneser-Ney 的接續機率。
    pub continuation: f64,
    /// 這個字當前一字時的 `ln α`：它後面的 bigram 沒留下時乘的權重。
    pub backoff: f64,
}

/// 字級 bigram 語言模型。
///
/// [`Default`] 是沒有任何統計的模型：每個字的 `ln P` 都是 0，打分只剩 β、γ。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "Wire", into = "Wire")]
pub struct LanguageModel {
    /// 有統計的字，依碼位排序；下面各欄都用它的索引。
    chars: Vec<char>,
    models: Vec<Model32>,
    /// 不在 `chars` 的字：`start`、`continuation` 用它；當前一字時 `backoff` 用它。
    unknown: Model32,
    /// 前一字索引 `i` 的 bigram 在 `next[rows[i]..rows[i + 1]]`，後一字索引遞增。
    rows: Vec<u32>,
    next: Vec<u32>,
    /// 和 `next` 對齊的量化 `ln P`，見 [`LOG_PROB_STEP`]。
    log_prob: Vec<u8>,
}

/// 存在資料包裡的 [`CharModel`]：`f32` 就夠，省一半。
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
struct Model32 {
    start: f32,
    continuation: f32,
    backoff: f32,
}

impl From<CharModel> for Model32 {
    #[allow(clippy::cast_possible_truncation)] // 存成 f32 是刻意的精度取捨
    fn from(m: CharModel) -> Self {
        Self {
            start: m.start as f32,
            continuation: m.continuation as f32,
            backoff: m.backoff as f32,
        }
    }
}

impl Default for LanguageModel {
    fn default() -> Self {
        let zero = CharModel {
            start: 0.0,
            continuation: 0.0,
            backoff: 0.0,
        };
        Self::new([], zero, [])
    }
}

impl LanguageModel {
    /// `chars` 是有統計的字；`unknown` 給其他字；`bigrams` 是要留下的 `(前一字, 字, ln P)`，
    /// 兩個字都要在 `chars` 裡，否則略過。同一對出現多次時取最後一個。
    pub fn new(
        chars: impl IntoIterator<Item = (char, CharModel)>,
        unknown: CharModel,
        bigrams: impl IntoIterator<Item = (char, char, f64)>,
    ) -> Self {
        let mut chars: Vec<(char, CharModel)> = chars.into_iter().collect();
        chars.sort_by_key(|(c, _)| *c);
        chars.dedup_by_key(|(c, _)| *c);
        let index = |c: char| chars.binary_search_by_key(&c, |(c, _)| *c).ok();
        let mut pairs: Vec<(usize, usize, u8)> = bigrams
            .into_iter()
            .filter_map(|(prev, c, p)| Some((index(prev)?, index(c)?, quantize(p))))
            .collect();
        pairs.sort_by_key(|&(i, j, _)| (i, j));
        // 同一對留最後一個：倒過來去重再轉回來。
        pairs.reverse();
        pairs.dedup_by_key(|&mut (i, j, _)| (i, j));
        pairs.reverse();
        let mut rows = vec![0u32; chars.len() + 1];
        for &(i, _, _) in &pairs {
            rows[i + 1] += 1;
        }
        for i in 0..chars.len() {
            rows[i + 1] += rows[i];
        }
        Self {
            models: chars.iter().map(|&(_, m)| m.into()).collect(),
            chars: chars.into_iter().map(|(c, _)| c).collect(),
            unknown: unknown.into(),
            rows,
            next: pairs.iter().map(|&(_, j, _)| index_u32(j)).collect(),
            log_prob: pairs.iter().map(|&(_, _, q)| q).collect(),
        }
    }

    /// `ln P(c | prev)`；`prev` 是 `None` 表示沒有前一字。
    pub fn log_prob(&self, prev: Option<char>, c: char) -> f64 {
        let j = self.index(c);
        let model = |i: Option<usize>| i.map_or(self.unknown, |i| self.models[i]);
        let Some(prev) = prev else {
            return f64::from(model(j).start);
        };
        let i = self.index(prev);
        if let (Some(i), Some(j)) = (i, j) {
            let (from, to) = (self.rows[i] as usize, self.rows[i + 1] as usize);
            if let Ok(k) = self.next[from..to].binary_search(&index_u32(j)) {
                return dequantize(self.log_prob[from + k]);
            }
        }
        f64::from(model(i).backoff) + f64::from(model(j).continuation)
    }

    /// `c` 有統計：當前一字時接得上它自己的 bigram 與退回權重。
    pub fn has_statistics(&self, c: char) -> bool {
        self.index(c).is_some()
    }

    fn index(&self, c: char) -> Option<usize> {
        self.chars.binary_search(&c).ok()
    }
}

fn index_u32(i: usize) -> u32 {
    u32::try_from(i).expect("字數放得進 u32")
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // 已夾在 0–255
fn quantize(log_prob: f64) -> u8 {
    (-log_prob / LOG_PROB_STEP).round().clamp(0.0, 255.0) as u8
}

fn dequantize(q: u8) -> f64 {
    -f64::from(q) * LOG_PROB_STEP
}

/// 資料包裡的樣子：每一列的後一字索引存和前一個的差，postcard 的 varint 多半只要一個位元組。
#[derive(Serialize, Deserialize)]
struct Wire {
    chars: Vec<char>,
    models: Vec<Model32>,
    unknown: Model32,
    /// 每個前一字的 bigram 數。
    row_lengths: Vec<u32>,
    next_gaps: Vec<u32>,
    log_prob: Vec<u8>,
}

impl From<LanguageModel> for Wire {
    fn from(m: LanguageModel) -> Self {
        let row_lengths = m.rows.windows(2).map(|w| w[1] - w[0]).collect();
        let mut next_gaps = Vec::with_capacity(m.next.len());
        for w in m.rows.windows(2) {
            let row = &m.next[w[0] as usize..w[1] as usize];
            let mut last = 0;
            for &j in row {
                next_gaps.push(j - last);
                last = j;
            }
        }
        Self {
            chars: m.chars,
            models: m.models,
            unknown: m.unknown,
            row_lengths,
            next_gaps,
            log_prob: m.log_prob,
        }
    }
}

impl TryFrom<Wire> for LanguageModel {
    type Error = String;

    fn try_from(w: Wire) -> Result<Self, String> {
        let n = w.chars.len();
        if !w.chars.windows(2).all(|p| p[0] < p[1]) {
            return Err("字沒有依碼位遞增".into());
        }
        if w.models.len() != n || w.row_lengths.len() != n {
            return Err("字、unigram、bigram 列數不一致".into());
        }
        if w.next_gaps.len() != w.log_prob.len() {
            return Err("bigram 的字與機率個數不一致".into());
        }
        let mut rows = Vec::with_capacity(n + 1);
        rows.push(0u32);
        let mut next = Vec::with_capacity(w.next_gaps.len());
        let mut gaps = w.next_gaps.iter();
        for &len in &w.row_lengths {
            let mut last: Option<u32> = None;
            for _ in 0..len {
                let gap = *gaps.next().ok_or("bigram 比列長的總和少")?;
                let j = match last {
                    None => gap,
                    Some(_) if gap == 0 => return Err("同一列有重複的後一字".into()),
                    Some(last) => last.checked_add(gap).ok_or("後一字索引溢位")?,
                };
                if j as usize >= n {
                    return Err("後一字索引超出字數".into());
                }
                next.push(j);
                last = Some(j);
            }
            rows.push(index_u32(next.len()));
        }
        if gaps.next().is_some() {
            return Err("bigram 比列長的總和多".into());
        }
        Ok(Self {
            chars: w.chars,
            models: w.models,
            unknown: w.unknown,
            rows,
            next,
            log_prob: w.log_prob,
        })
    }
}
