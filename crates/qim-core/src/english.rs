//! 英文詞表（plan〈英文、簡繁與 emoji〉）：輸入碼（小寫）→ 要顯示的拼法與頻率。中文狀態的英文候選（plan〈學習與排位〉
//! 規則 4）與英文模式（ADR 0008）共用。

use serde::{Deserialize, Serialize};

/// 英文詞表：每個小寫的鍵一個拼法與它的頻率。同一個鍵有幾種大小寫時取全小寫的（`may` 不取 `May`），
/// 沒有全小寫的取字典序最前的（`english` → `English`、`nasa` → `NASA`），頻率是留下的那種寫法的。
/// 頻率照 AOSP 詞表的 `f`（0–255，頻率的對數，大的常見；plan〈英文、簡繁與 emoji〉），給英文模式的建議列排序。
///
/// 存成依鍵排序的（拼法, 頻率）清單；從位元組讀回時重新整理，不靠資料包的順序。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Vec<(String, u8)>", into = "Vec<(String, u8)>")]
pub struct EnglishWords {
    /// 依鍵（[`key`]）排序、鍵不重複。
    words: Vec<(String, u8)>,
}

impl EnglishWords {
    /// 空字串不收。
    pub fn new<S: Into<String>>(words: impl IntoIterator<Item = (S, u8)>) -> Self {
        let mut words: Vec<(String, u8)> = words
            .into_iter()
            .map(|(w, f)| (w.into(), f))
            .filter(|(w, _)| !w.is_empty())
            .collect();
        // 同鍵的排在一起，全小寫的在前，其餘依字典序；每個鍵留第一個。
        words.sort_by(|(a, _), (b, _)| {
            key(a)
                .cmp(key(b))
                .then_with(|| (!is_lower(a)).cmp(&!is_lower(b)))
                .then_with(|| a.cmp(b))
        });
        words.dedup_by(|(later, _), (kept, _)| key(later).eq(key(kept)));
        Self { words }
    }

    fn find(&self, code: &str) -> Option<&(String, u8)> {
        self.words
            .binary_search_by(|(w, _)| key(w).cmp(code.chars()))
            .ok()
            .map(|i| &self.words[i])
    }

    /// 鍵剛好是 `code` 的拼法；`code` 要是小寫，不補全也不容錯。
    pub fn lookup(&self, code: &str) -> Option<&str> {
        self.find(code).map(|(w, _)| w.as_str())
    }

    /// 鍵剛好是 `code` 的詞的頻率（同 [`Self::lookup`] 的比對）。
    pub fn frequency(&self, code: &str) -> Option<u8> {
        self.find(code).map(|&(_, f)| f)
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.words.iter().map(|(w, _)| w.as_str())
    }
}

/// 拼法的鍵：轉成小寫。
fn key(word: &str) -> impl Iterator<Item = char> + Clone + '_ {
    word.chars().flat_map(char::to_lowercase)
}

fn is_lower(word: &str) -> bool {
    key(word).eq(word.chars())
}

impl From<Vec<(String, u8)>> for EnglishWords {
    fn from(words: Vec<(String, u8)>) -> Self {
        Self::new(words)
    }
}

impl From<EnglishWords> for Vec<(String, u8)> {
    fn from(words: EnglishWords) -> Self {
        words.words
    }
}
