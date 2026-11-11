//! 同步層換語言模型的接口（M3 票 08，模型階梯）：lattice 每條邊的靜態機率改由外部的模型給，
//! 實驗用它比字級 trigram、詞級 n-gram 和現行的字級 bigram（plan §2.3）。沒掛上時打分照舊。
//! 產品要不要留它由階梯結論票（票 11）定；不留就連同測試一起刪。

use std::fmt;

/// 交給 [`SegmentModel`] 的前面幾段：最多這麼多段。
pub const SEGMENT_HISTORY: usize = 2;

/// 一段接在前面的段後面的靜態 ln 機率（plan §2.3 的 `P(段 | 前文)`），取代現行的
/// 「首字 bigram 加詞內機率」；β、γ 與 `personal` 照舊由核心加。
pub trait SegmentModel: fmt::Debug + Send + Sync {
    fn log_prob(&self, query: &SegmentQuery<'_>) -> f64;
}

/// 一次查詢。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegmentQuery<'a> {
    /// 前面的段，最近的在最後，最多 [`SEGMENT_HISTORY`] 段。到了組字開頭接組字前面的那一段：
    /// 選定的分段、前文剛上屏的那一段，或前文的最後一字、句首（[`crate::CLAUSE_START`]）；
    /// 不知道前文時沒有。
    pub before: &'a [&'a str],
    /// 這一段的字詞。
    pub text: &'a str,
    /// 現行打分的靜態 ln 機率：字級 bigram 接前一字乘詞內機率（plan §2.3）。
    pub current_log_prob: f64,
}
