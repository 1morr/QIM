use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// 詞庫：輸入碼 → 多字詞，加上給單字排序的字頻（兩者放同一個資料包）；每個輸入方案各建一份碼索引（plan §2.5、§4.2）。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lexicon {
    words: BTreeMap<String, Vec<Word>>,
    char_frequency: BTreeMap<char, u64>,
}

/// 詞庫裡一個輸入碼對應的一個詞。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Word {
    pub text: String,
    /// 詞頻，來源的原值。
    pub frequency: u64,
}

impl Lexicon {
    /// 每個輸入碼的詞依給定順序保存，排序由建置端決定；`char_frequency` 是字 → 出現次數。
    pub fn new(
        words: impl IntoIterator<Item = (String, Vec<Word>)>,
        char_frequency: impl IntoIterator<Item = (char, u64)>,
    ) -> Self {
        Self {
            words: words.into_iter().collect(),
            char_frequency: char_frequency.into_iter().collect(),
        }
    }

    /// 輸入碼對應的詞，依建置時的順序；查不到時為空。
    pub fn lookup(&self, code: &str) -> &[Word] {
        self.words.get(code).map_or(&[], Vec::as_slice)
    }

    /// 依輸入碼的字典序列出全部輸入碼與它的詞。
    pub fn iter(&self) -> impl Iterator<Item = (&str, &[Word])> {
        self.words
            .iter()
            .map(|(code, words)| (code.as_str(), words.as_slice()))
    }

    /// 字的出現次數；沒有字頻的字是 0。
    pub fn char_frequency(&self, ch: char) -> u64 {
        self.char_frequency.get(&ch).copied().unwrap_or(0)
    }
}
