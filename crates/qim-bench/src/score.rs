//! 用核心實際打每一題、算分數：只經 `Session::handle`，不重寫排序（plan §1.3）。

use std::fmt;

use qim_core::{Engine, Key, Session, UserModel};

use crate::items::Item;

/// 一個集合的結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetResult {
    pub name: String,
    pub total: usize,
    /// 整句首選正確的題數。
    pub correct: usize,
    /// 期望首選的總字數。
    pub chars: usize,
    /// 各題首選與期望的編輯距離（以字計，每題最多算到該題字數）的總和。
    pub char_errors: usize,
    /// 答錯的題：題目與實際的首選。
    pub failures: Vec<Failure>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub item: Item,
    /// 打完輸入碼後的首選；沒有候選時是空字串。
    pub top: String,
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Item {
            input,
            expected,
            label,
        } = &self.item;
        write!(
            f,
            "{input}（{label}）期望「{expected}」，首選「{}」",
            self.top
        )
    }
}

impl SetResult {
    pub fn wrong(&self) -> usize {
        self.total - self.correct
    }

    /// 整句首選正確率。
    pub fn sentence_accuracy(&self) -> f64 {
        ratio(self.correct, self.total)
    }

    /// 字正確率：1 − 編輯距離總和 ÷ 總字數。
    pub fn char_accuracy(&self) -> f64 {
        1.0 - ratio(self.char_errors, self.chars)
    }
}

fn ratio(part: usize, whole: usize) -> f64 {
    if whole == 0 {
        0.0
    } else {
        part as f64 / whole as f64
    }
}

/// 每題用新的 `Session` 與空白的 `UserModel` 打，題與題互不影響（plan §5.2）。
pub fn score(engine: &Engine, name: &str, items: &[Item]) -> SetResult {
    let mut result = SetResult {
        name: name.to_string(),
        total: items.len(),
        correct: 0,
        chars: 0,
        char_errors: 0,
        failures: Vec::new(),
    };
    for item in items {
        let top = top(engine, &item.input);
        let chars = item.expected.chars().count();
        result.chars += chars;
        result.char_errors += strsim::levenshtein(&top, &item.expected).min(chars);
        if top == item.expected {
            result.correct += 1;
        } else {
            result.failures.push(Failure {
                item: item.clone(),
                top,
            });
        }
    }
    result
}

/// 逐鍵打完 `input` 後的首選。
fn top(engine: &Engine, input: &str) -> String {
    let mut session = Session::default();
    let mut user = UserModel::default();
    let mut snapshot = None;
    for c in input.chars() {
        snapshot = Some(session.handle(engine, &mut user, Key::Letter(c)));
    }
    snapshot
        .and_then(|s| s.candidates.into_iter().next())
        .map(|c| c.text)
        .unwrap_or_default()
}
