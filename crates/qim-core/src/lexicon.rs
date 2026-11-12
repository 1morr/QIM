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

    /// 有沒有以 `prefix` 開頭的輸入碼（萬用鍵展開時剪枝用）。
    pub(crate) fn has_code_prefix(&self, prefix: &str) -> bool {
        crate::code_table::has_key_with_prefix(&self.words, prefix)
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

/// 詞庫的詞依文字排序、同文字只留一個，和它的 ln unigram：聯想查「以某串字開頭的詞」用（plan §2.3）。
/// 文字接成一整串、只記位移，不為每個詞各配一塊記憶體（約 26 萬詞時約 4 MB，各自配是約 12 MB）。
#[derive(Debug, Clone, Default)]
pub(crate) struct WordsByText {
    /// 各詞依序接起來。
    text: String,
    /// 第 i 個詞在 `text[starts[i]..starts[i + 1]]`；比詞數多一個。
    starts: Vec<u32>,
    log_prob: Vec<f32>,
}

impl WordsByText {
    /// `log_prob` 算每個詞的 ln unigram；同文字的詞取碼的字典序最前的那個。
    pub(crate) fn new(lexicon: &Lexicon, log_prob: impl Fn(&Word) -> f64) -> Self {
        // 只排參照，建的時候不多配一份文字。
        let mut words: Vec<&Word> = lexicon.iter().flat_map(|(_, words)| words).collect();
        // 穩定排序，同文字的留第一個。
        words.sort_by(|a, b| a.text.cmp(&b.text));
        words.dedup_by(|b, a| a.text == b.text);
        let mut index = Self {
            text: String::with_capacity(words.iter().map(|w| w.text.len()).sum()),
            starts: Vec::with_capacity(words.len() + 1),
            log_prob: Vec::with_capacity(words.len()),
        };
        for word in words {
            index.starts.push(offset(index.text.len()));
            index.text.push_str(&word.text);
            #[allow(clippy::cast_possible_truncation)] // 存成 f32 是刻意的精度取捨，同語言模型
            index.log_prob.push(log_prob(word) as f32);
        }
        index.starts.push(offset(index.text.len()));
        index
    }

    fn len(&self) -> usize {
        self.log_prob.len()
    }

    fn text_at(&self, i: usize) -> &str {
        &self.text[self.starts[i] as usize..self.starts[i + 1] as usize]
    }

    /// 以 `prefix` 開頭、比它長的詞與 ln unigram，依文字排序。
    pub(crate) fn starting_with<'a>(
        &'a self,
        prefix: &'a str,
    ) -> impl Iterator<Item = (&'a str, f64)> + 'a {
        let start = partition_point(self.len(), |i| self.text_at(i) <= prefix);
        (start..self.len())
            .map(|i| (self.text_at(i), f64::from(self.log_prob[i])))
            .take_while(move |(text, _)| text.starts_with(prefix))
    }

    /// 詞 `text` 的 ln unigram；不是詞庫的詞時是 `None`。
    pub(crate) fn log_prob(&self, text: &str) -> Option<f64> {
        let at = partition_point(self.len(), |i| self.text_at(i) < text);
        (at < self.len() && self.text_at(at) == text).then(|| f64::from(self.log_prob[at]))
    }
}

/// `0..len` 裡第一個 `before` 為假的位置；`before` 在前段為真、後段為假。
fn partition_point(len: usize, before: impl Fn(usize) -> bool) -> usize {
    let (mut low, mut high) = (0, len);
    while low < high {
        let mid = low + (high - low) / 2;
        if before(mid) {
            low = mid + 1;
        } else {
            high = mid;
        }
    }
    low
}

fn offset(at: usize) -> u32 {
    u32::try_from(at).expect("詞庫的文字放得進 u32 的位移")
}
