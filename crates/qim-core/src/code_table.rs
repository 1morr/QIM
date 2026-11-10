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
    /// 擴展區的字：只在單字查詢時出現，排在連打字集之後（brief §4.1）。
    pub extension: bool,
    /// 簡體字：HKSCS 收的「门」「农」這類，只在連打字集；繁體模式排在同碼的繁體字之後，
    /// 簡體模式不管它（brief §4.1，判定在建置端，plan §4.1）。
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

    /// 依輸入碼的字典序列出全部輸入碼與它的字。
    pub fn iter(&self) -> impl Iterator<Item = (&str, &[CodeEntry])> {
        self.entries
            .iter()
            .map(|(code, e)| (code.as_str(), e.as_slice()))
    }
}
