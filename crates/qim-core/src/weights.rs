//! 觸控一下的各字母機率（`Key::Weighted`，plan §2.1，M4 票 06）。
//!
//! 機率由殼經 `qim-layout` 依點擊位置與按鍵幾何算出，核心不知道佈局（plan §1.3）。

/// 一次點擊的各字母機率：至少一個、字母不重複、機率由高到低（一樣高時照殼給的順序），和為 1。
#[derive(Debug, Clone, PartialEq)]
pub struct Weights(Vec<(char, f32)>);

// 建構時擋掉 NaN 與無限大，`f32` 的 `==` 在這裡是等價關係。
impl Eq for Weights {}

/// [`Weights::new`] 收不下的機率。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeightsError {
    Empty,
    /// 不是字母（`char::is_alphabetic`）。
    NotLetter(char),
    Duplicate(char),
    /// 機率要是大於 0 的有限數。
    BadProbability(char),
}

impl std::fmt::Display for WeightsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeightsError::Empty => f.write_str("沒有任何字母"),
            WeightsError::NotLetter(c) => write!(f, "{c:?} 不是字母"),
            WeightsError::Duplicate(c) => write!(f, "{c:?} 重複"),
            WeightsError::BadProbability(c) => write!(f, "{c:?} 的機率不是大於 0 的有限數"),
        }
    }
}

impl std::error::Error for WeightsError {}

impl Weights {
    /// 照比例正規化成和為 1、依機率由高到低排（穩定排序）。殼給的數字可能四捨五入過，所以不要求和剛好是 1。
    pub fn new(entries: Vec<(char, f32)>) -> Result<Self, WeightsError> {
        if entries.is_empty() {
            return Err(WeightsError::Empty);
        }
        for (i, &(c, p)) in entries.iter().enumerate() {
            if !c.is_alphabetic() {
                return Err(WeightsError::NotLetter(c));
            }
            if !(p.is_finite() && p > 0.0) {
                return Err(WeightsError::BadProbability(c));
            }
            if entries[..i].iter().any(|&(d, _)| d == c) {
                return Err(WeightsError::Duplicate(c));
            }
        }
        let sum: f32 = entries.iter().map(|&(_, p)| p).sum();
        if !sum.is_finite() {
            return Err(WeightsError::BadProbability(entries[0].0));
        }
        let mut entries: Vec<(char, f32)> =
            entries.into_iter().map(|(c, p)| (c, p / sum)).collect();
        entries.sort_by(|a, b| b.1.total_cmp(&a.1));
        Ok(Weights(entries))
    }

    /// 機率最高的字母；核心只用它（同 `Key::Letter`）。
    pub fn top(&self) -> char {
        self.0[0].0
    }

    /// 各字母與機率，由高到低。
    pub fn entries(&self) -> &[(char, f32)] {
        &self.0
    }

    /// 讀 `Key::parse` 的值：`字母:機率` 以逗號分隔，例如 `a:0.62,s:0.38`。
    pub(crate) fn parse(value: &str) -> Option<Weights> {
        let entries = value
            .split(',')
            .map(|item| {
                let (letter, probability) = item.split_once(':')?;
                let mut chars = letter.chars();
                let c = chars.next()?;
                if chars.next().is_some() {
                    return None;
                }
                Some((c, probability.parse::<f32>().ok()?))
            })
            .collect::<Option<Vec<_>>>()?;
        Weights::new(entries).ok()
    }
}
