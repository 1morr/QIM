use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// 碼表：輸入碼 → 字，每個輸入方案一份（ADR 0003）。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeTable {
    entries: BTreeMap<String, Vec<CodeEntry>>,
}

/// 碼表裡一個輸入碼對應的一個字。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeEntry {
    pub text: char,
    /// 擴展區的字：只在單字查詢時出現，排在連打字集之後（brief〈字集與字形〉）。
    pub extension: bool,
    /// 簡體字：HKSCS 收的「门」「农」這類，只在連打字集；繁體模式排在同碼的繁體字之後，
    /// 簡體模式不管它（brief〈字集與字形〉，判定在建置端，plan〈速成碼表〉）。
    pub simplified: bool,
}

impl CodeTable {
    /// 每個輸入碼的字依給定順序保存，排序由建置端決定。
    pub fn from_entries(entries: impl IntoIterator<Item = (String, Vec<CodeEntry>)>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
        }
    }

    /// 輸入碼對應的字，依建置時的字序；查不到時為空。
    pub fn lookup(&self, code: &str) -> &[CodeEntry] {
        self.entries.get(code).map_or(&[], Vec::as_slice)
    }

    /// 有沒有以 `prefix` 開頭的輸入碼（萬用鍵展開時剪枝用）。
    pub(crate) fn has_prefix(&self, prefix: &str) -> bool {
        has_key_with_prefix(&self.entries, prefix)
    }

    /// 依輸入碼的字典序列出全部輸入碼與它的字。
    pub fn iter(&self) -> impl Iterator<Item = (&str, &[CodeEntry])> {
        self.entries
            .iter()
            .map(|(code, e)| (code.as_str(), e.as_slice()))
    }
}

/// 依字串排序的表裡有沒有以 `prefix` 開頭的鍵：從 `prefix` 起的第一個鍵以它開頭就有。
pub(crate) fn has_key_with_prefix<V>(map: &BTreeMap<String, V>, prefix: &str) -> bool {
    map.range::<str, _>((
        std::ops::Bound::Included(prefix),
        std::ops::Bound::Unbounded,
    ))
    .next()
    .is_some_and(|(key, _)| key.starts_with(prefix))
}
