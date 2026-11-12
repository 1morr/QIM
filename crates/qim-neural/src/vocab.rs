//! 字表：字 → token，照 `scripts/neural-rerank/train.py prepare` 寫的字表。

use std::collections::HashMap;

use crate::{Error, SEP, UNK};

/// 字表：token 0、1 是兩個記號（[`SEP`]、[`UNK`]），之後依訓練語料的字次數排。
pub struct Vocab {
    index: HashMap<char, u32>,
    /// 路徑裡每個字表外的字多扣的分（nat）。
    unknown_penalty: f32,
}

impl Vocab {
    /// `chars` 是兩個記號之後的字，依序對應 token 2、3…；`unknown_chars` 是訓練語料裡字表外的
    /// 字數：模型把 `<unk>` 的機率給了它們全部，一個字表外的字只分到 `1 / unknown_chars`。
    pub fn new(chars: &[char], unknown_chars: u32) -> Result<Self, Error> {
        let mut index = HashMap::with_capacity(chars.len());
        for (i, &c) in chars.iter().enumerate() {
            if index.insert(c, i as u32 + 2).is_some() {
                return Err(Error::InvalidVocab(format!("「{c}」出現兩次")));
            }
        }
        Ok(Self {
            index,
            unknown_penalty: (unknown_chars.max(1) as f32).ln(),
        })
    }

    /// 含兩個記號的 token 數，等於模型的 embedding 列數。
    pub fn len(&self) -> usize {
        self.index.len() + 2
    }

    /// 路徑裡每個字表外的字多扣的分（nat）：`ln(unknown_chars)`。
    pub fn unknown_penalty(&self) -> f32 {
        self.unknown_penalty
    }

    /// 前文：字表裡的字照查；其他字母、數字（英文、擴展區的字）連續的收成一個 `<unk>`；
    /// 標點、空白、符號連續的收成一個 `<sep>`。開頭的 `<sep>` 不留（模型的句首就是它）。
    pub fn context(&self, text: &str) -> Vec<u32> {
        let mut out: Vec<u32> = Vec::new();
        for c in text.chars() {
            let t = match self.index.get(&c) {
                Some(&i) => i,
                None if c.is_alphanumeric() => UNK,
                None => SEP,
            };
            if (t == SEP || t == UNK) && out.last() == Some(&t) {
                continue;
            }
            if t == SEP && out.is_empty() {
                continue;
            }
            out.push(t);
        }
        out
    }

    /// 路徑：字表外的字是 `<unk>`；回 token 與字表外的字數。
    pub fn path(&self, text: &str) -> (Vec<u32>, usize) {
        let ids: Vec<u32> = text
            .chars()
            .map(|c| *self.index.get(&c).unwrap_or(&UNK))
            .collect();
        let unknown = ids.iter().filter(|&&t| t == UNK).count();
        (ids, unknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_maps_punctuation_and_unknowns() {
        let v = Vocab::new(&['的', '我'], 3).unwrap();
        // 開頭的標點不留；英文、數字收成一個 <unk>；連續標點收成一個 <sep>；「罕」不在字表。
        assert_eq!(
            v.context("，我的ABC12。」我罕"),
            vec![3, 2, UNK, SEP, 3, UNK]
        );
        assert_eq!(v.path("我罕"), (vec![3, UNK], 1));
        assert_eq!(v.len(), 4);
        assert!((v.unknown_penalty() - 3f32.ln()).abs() < 1e-6);
    }
}
