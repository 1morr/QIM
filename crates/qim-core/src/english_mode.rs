//! 英文模式的建議列（ADR 0008，M4 票 24）：正在打的串的前綴補完，英文詞典加英文使用者詞。只補完、不出下一個
//! 詞；拼錯的修正（英文容錯）與自動修正在 M4 票 25。

use std::collections::HashSet;

use crate::english_user::ENGLISH_SHOWN_AFTER;
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
pub const ENGLISH_SUGGESTION_LIMIT: usize = 2 * crate::PAGE_SIZE;

/// 正在打的詞 `typed`（[`crate::english_user::TypedRun::word`]）的建議，照使用頻率排：
///
/// 1. 學過的（英文使用者詞，`user_words` 開著時）：英文詞典有的詞打過一次就算（詞典有的詞不受出現門檻管，
///    學習讓它排前），詞典沒有的要過出現門檻（[`ENGLISH_SHOWN_AFTER`]）；衰減值大的（最近常打的）在前。
/// 2. 英文詞典其餘的詞，頻率高的在前。
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
    let mut learned: Vec<(f64, &str, SpanKind)> = Vec::new();
    let mut learned_keys: HashSet<&str> = HashSet::new();
    if user_words {
        for (key, text, used, count) in user.english_starting_with(&prefix) {
            let shown = match dictionary.lookup(key) {
                Some(spelling) => Some((spelling, SpanKind::English)),
                None => (count >= ENGLISH_SHOWN_AFTER).then_some((text, SpanKind::EnglishUserWord)),
            };
            if let Some((text, kind)) = shown {
                learned.push((used, text, kind));
                learned_keys.insert(key);
            }
        }
    }
    learned.sort_by(|a, b| b.0.total_cmp(&a.0));

    // 詞典的詞只取頻率最高的一批再排：一個字母開頭的詞有上萬個。多留一個給和 `typed` 相同而不列的。
    let wanted = ENGLISH_SUGGESTION_LIMIT + 1;
    let mut rest: Vec<(u8, &str)> = dictionary
        .starting_with(&prefix)
        .filter(|(word, _)| !learned_keys.contains(word.to_lowercase().as_str()))
        .map(|(word, frequency)| (frequency, word))
        .collect();
    let by_frequency = |a: &(u8, &str), b: &(u8, &str)| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1));
    if rest.len() > wanted {
        rest.select_nth_unstable_by(wanted, by_frequency);
        rest.truncate(wanted);
    }
    rest.sort_by(by_frequency);

    let case = Case::of(typed);
    let mut seen: HashSet<String> = HashSet::new();
    learned
        .into_iter()
        .map(|(_, text, kind)| (text, kind))
        .chain(rest.into_iter().map(|(_, text)| (text, SpanKind::English)))
        .filter_map(|(text, kind)| {
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
