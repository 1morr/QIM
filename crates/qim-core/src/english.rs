//! 中文狀態的英文候選用的詞表（plan §2.4 規則 4、§4.4）：輸入碼（小寫）→ 要顯示的拼法。

use serde::{Deserialize, Serialize};

/// 英文詞表：每個小寫的鍵一個拼法。同一個鍵有幾種大小寫時取全小寫的（`may` 不取 `May`），
/// 沒有全小寫的取字典序最前的（`english` → `English`、`nasa` → `NASA`）。
///
/// 存成依鍵排序的拼法清單；從位元組讀回時重新整理，不靠資料包的順序。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Vec<String>", into = "Vec<String>")]
pub struct EnglishWords {
    /// 依鍵（[`key`]）排序、鍵不重複。
    words: Vec<String>,
}

impl EnglishWords {
    /// 空字串不收。
    pub fn new<S: Into<String>>(words: impl IntoIterator<Item = S>) -> Self {
        let mut words: Vec<String> = words
            .into_iter()
            .map(Into::into)
            .filter(|w| !w.is_empty())
            .collect();
        // 同鍵的排在一起，全小寫的在前，其餘依字典序；每個鍵留第一個。
        words.sort_by(|a, b| {
            key(a)
                .cmp(key(b))
                .then_with(|| (!is_lower(a)).cmp(&!is_lower(b)))
                .then_with(|| a.cmp(b))
        });
        words.dedup_by(|later, kept| key(later).eq(key(kept)));
        Self { words }
    }

    /// 鍵剛好是 `code` 的拼法；`code` 要是小寫，不補全也不容錯。
    pub fn lookup(&self, code: &str) -> Option<&str> {
        self.words
            .binary_search_by(|w| key(w).cmp(code.chars()))
            .ok()
            .map(|i| self.words[i].as_str())
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.words.iter().map(String::as_str)
    }
}

/// 拼法的鍵：轉成小寫。
fn key(word: &str) -> impl Iterator<Item = char> + Clone + '_ {
    word.chars().flat_map(char::to_lowercase)
}

fn is_lower(word: &str) -> bool {
    key(word).eq(word.chars())
}

impl From<Vec<String>> for EnglishWords {
    fn from(words: Vec<String>) -> Self {
        Self::new(words)
    }
}

impl From<EnglishWords> for Vec<String> {
    fn from(words: EnglishWords) -> Self {
        words.words
    }
}
