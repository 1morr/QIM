//! emoji 候選用的對照表（plan〈學習與排位〉〈英文、簡繁與 emoji〉）：字詞（繁體原文）→ 它的 emoji，好的在前。

use serde::{Deserialize, Serialize};

/// 一列候選最多接幾個 emoji：第一頁（9 個）大半仍是字詞。
pub const EMOJI_LIMIT: usize = 3;

/// 字詞 → emoji。存成依字詞排序的清單；從位元組讀回時重新整理，不靠資料包的順序。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    from = "Vec<(String, Vec<String>)>",
    into = "Vec<(String, Vec<String>)>"
)]
pub struct EmojiTable {
    /// 依字詞排序、字詞不重複、每列至少一個 emoji。
    entries: Vec<(String, Vec<String>)>,
}

impl EmojiTable {
    /// 同一個字詞列了幾次時依序接起來；重複的 emoji 只留第一個，沒有 emoji 的字詞不收。
    pub fn new<K, E, V>(entries: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<String>,
        E: Into<String>,
        V: IntoIterator<Item = E>,
    {
        let mut merged: std::collections::BTreeMap<String, Vec<String>> = Default::default();
        for (word, emoji) in entries {
            let list = merged.entry(word.into()).or_default();
            for e in emoji {
                let e = e.into();
                if !e.is_empty() && !list.contains(&e) {
                    list.push(e);
                }
            }
        }
        Self {
            entries: merged
                .into_iter()
                .filter(|(word, emoji)| !word.is_empty() && !emoji.is_empty())
                .collect(),
        }
    }

    /// `word` 的 emoji，好的在前；沒有時是空的。
    pub fn lookup(&self, word: &str) -> &[String] {
        self.entries
            .binary_search_by(|(w, _)| w.as_str().cmp(word))
            .map(|i| self.entries[i].1.as_slice())
            .unwrap_or_default()
    }

    /// 有 emoji 的字詞數。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &[String])> {
        self.entries.iter().map(|(w, e)| (w.as_str(), e.as_slice()))
    }
}

impl From<Vec<(String, Vec<String>)>> for EmojiTable {
    fn from(entries: Vec<(String, Vec<String>)>) -> Self {
        Self::new(entries)
    }
}

impl From<EmojiTable> for Vec<(String, Vec<String>)> {
    fn from(table: EmojiTable) -> Self {
        table.entries
    }
}
