//! 英文模式的建議列（ADR 0008，M4 票 24）：正在打的串的前綴補完，英文詞典加英文使用者詞。只補完、不出下一個
//! 詞；拼錯的修正（英文容錯）與自動修正在 M4 票 25。

use std::collections::{HashMap, HashSet};

use crate::english_user::{ENGLISH_SHOWN_AFTER, UsedWord};
use crate::lattice::{CandidateSpan, SpanKind};
use crate::{Engine, UserModel};

/// 游標前的字在句子的哪裡（句首自動大寫，`Snapshot::auto_shift`）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Sentence {
    /// 不知道游標前是什麼（前文作廢、無痕時剛換欄位）：不自動大寫。
    #[default]
    Unknown,
    /// 句子開頭：空的欄位、句尾標點後接空白、換行之後。
    Start,
    /// 剛打了句尾標點（`.` `?` `!`），再接空白才是句子開頭（`e.g` 的 `.` 後面接字母就不是）。
    Ended,
    /// 句子中間。
    Middle,
}

impl Sentence {
    /// 從這裡接著打出 `text` 之後在哪裡。照 Gboard 實測（事實 M48）：空的欄位、`.` `?` 後接空白、換行之後
    /// 自動大寫；`!` 同 `?`。`e.g. this` 這類縮寫後 Gboard 不大寫，這裡照樣大寫。
    pub(crate) fn after(self, text: &str) -> Self {
        text.chars().fold(self, |state, c| match c {
            '\n' => Self::Start,
            '.' | '?' | '!' => Self::Ended,
            c if c.is_whitespace() => match state {
                Self::Ended | Self::Start => Self::Start,
                other => other,
            },
            _ => Self::Middle,
        })
    }
}

/// 點了建議、補了空白之後緊接著打 `c` 時，先刪掉補的空白：句讀與右括號（`hello,`、`hello.`、`(hello)`）。
/// Gboard 實測打逗號會刪（事實 M48），其餘照同一個道理：這些字元前面不留空白。
pub(crate) fn takes_picked_space(c: char) -> bool {
    matches!(c, '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']' | '}')
}

/// 建議列最多幾個：兩頁（[`crate::PAGE_SIZE`]），候選展開面板看得到第二頁。
pub(crate) const ENGLISH_SUGGESTION_LIMIT: usize = 2 * crate::PAGE_SIZE;

/// LatinIME 的機率刻度：機率每多一倍，0–255 的編碼值多這麼多（`PROBABILITY_ENCODING_SCALER`，事實 M50）。詞典的
/// 頻率 `f` 就是這個刻度。
const PROBABILITY_SCALE: f64 = 8.589_237;

/// 學過一次的加分相當於機率乘幾倍（[`learned_bonus`]）。LatinIME 沒有加分（同一個詞取主詞典與使用者歷史較大的，
/// 事實 M50），打一次的詞典詞照它不會往前；這裡要它照 Gboard 打過一次就出在建議列（事實 M47），又不壓過常用詞。
/// AOSP 詞表裡 `hel` 開頭第 3 名比 helmet 高 31，`t` 開頭第 3 名比 tumors 高 97：64 倍（+51.5）落在中間。
const FIRST_USE_FACTOR: f64 = 64.0;

/// 學過 `count` 次（一年內）的加分：機率乘上 `FIRST_USE_FACTOR × count` 倍，打得越多越前，每多一倍次數多
/// [`PROBABILITY_SCALE`]（1 次 +51.5、2 次 +60.1、5 次 +71.5、20 次 +88.7）。
fn learned_bonus(count: u64) -> f64 {
    PROBABILITY_SCALE * (FIRST_USE_FACTOR * count as f64).log2()
}

/// 詞典沒有的英文使用者詞的底分：照 LatinIME 把使用者歷史的次數換成同一刻度的機率（事實 M50）：
/// `255 + 刻度 × log2(次數 ÷ 8192) − 32`。分母是 LatinIME 總輸入次數的下限 8192（QIM 不記總輸入次數，取下限），
/// −32 是它沒有前一個詞時的退回權重。打過 2 次是 120。
fn history_probability(count: u64) -> f64 {
    255.0 + PROBABILITY_SCALE * (count as f64 / 8192.0).log2() - 32.0
}

/// 正在打的詞 `typed`（[`crate::english_user::TypedRun::word`]）的建議，照使用頻率排（使用者 2026-11-17 定照 LatinIME
/// 合比、不分兩層）：每個詞一個分數，在 LatinIME 的 0–255 對數刻度上比，大的在前，同分依寫法。
///
/// - 英文詞典的詞：詞典的頻率；學過的（英文使用者詞，`user_words` 開著時，一年內打過，詞典有的詞不受出現門檻管）
///   再加 [`learned_bonus`]。
/// - 詞典沒有的英文使用者詞：過了出現門檻（[`ENGLISH_SHOWN_AFTER`]）才列，[`history_probability`] 加
///   [`learned_bonus`]。
///
/// 比對不分大小寫；詞典的詞用詞典的寫法、詞典沒有的用最後打的寫法，再照 `typed` 的大小寫改（[`Case`]）。
/// 改完和 `typed` 一字不差的不列（選了也不變）。最多 [`ENGLISH_SUGGESTION_LIMIT`] 個。
pub(crate) fn suggestions(
    engine: &Engine,
    user: &UserModel,
    typed: &str,
    user_words: bool,
) -> Vec<CandidateSpan> {
    let prefix = typed.to_lowercase();
    let dictionary = engine.english_words();
    let learned: HashMap<&str, UsedWord> = if user_words {
        user.english_starting_with(&prefix)
            .map(|w| (w.key, w))
            .collect()
    } else {
        HashMap::new()
    };
    let mut scored: Vec<(f64, &str, SpanKind)> = dictionary
        .starting_with(&prefix)
        .map(|(word, frequency)| {
            let bonus = learned
                .get(word.to_lowercase().as_str())
                .map_or(0.0, |w| learned_bonus(w.count));
            (f64::from(frequency) + bonus, word, SpanKind::English)
        })
        .collect();
    scored.extend(
        learned
            .values()
            .filter(|w| w.count >= ENGLISH_SHOWN_AFTER && dictionary.lookup(w.key).is_none())
            .map(|w| {
                let score = history_probability(w.count) + learned_bonus(w.count);
                (score, w.text, SpanKind::EnglishUserWord)
            }),
    );
    // 只取分數最高的一批再排：一個字母開頭的詞有上萬個。多留一個給和 `typed` 相同而不列的。
    let wanted = ENGLISH_SUGGESTION_LIMIT + 1;
    let by_score = |a: &(f64, &str, SpanKind), b: &(f64, &str, SpanKind)| {
        b.0.total_cmp(&a.0).then_with(|| a.1.cmp(b.1))
    };
    if scored.len() > wanted {
        scored.select_nth_unstable_by(wanted, by_score);
        scored.truncate(wanted);
    }
    scored.sort_by(by_score);

    let case = Case::of(typed);
    let mut seen: HashSet<String> = HashSet::new();
    scored
        .into_iter()
        .filter_map(|(_, text, kind)| {
            let shown = case.apply(text);
            (shown != typed && seen.insert(shown.clone())).then(|| CandidateSpan {
                text: shown,
                len: typed.len(),
                segments: Vec::new(),
                kind,
                rule: None,
                neighbor: None,
            })
        })
        .take(ENGLISH_SUGGESTION_LIMIT)
        .collect()
}

/// 打的詞的大小寫，建議跟著它（Gboard 句中首字大寫的詞，修正也保留大寫，事實 M47）：第一個字母大寫時建議的
/// 開頭也大寫；兩個字母以上而全是大寫時整個建議大寫；其他照建議原本的寫法（`nasa` 給 NASA）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Case {
    AsIs,
    Capitalized,
    Upper,
}

impl Case {
    fn of(typed: &str) -> Self {
        let letters: Vec<char> = typed.chars().filter(|c| c.is_alphabetic()).collect();
        match letters.first() {
            Some(c) if c.is_uppercase() => {
                if letters.len() >= 2 && letters.iter().all(|c| c.is_uppercase()) {
                    Self::Upper
                } else {
                    Self::Capitalized
                }
            }
            _ => Self::AsIs,
        }
    }

    fn apply(self, text: &str) -> String {
        match self {
            Self::AsIs => text.to_string(),
            Self::Upper => text.to_uppercase(),
            Self::Capitalized => {
                let mut chars = text.chars();
                chars
                    .next()
                    .map(|first| first.to_uppercase().chain(chars).collect())
                    .unwrap_or_default()
            }
        }
    }
}
