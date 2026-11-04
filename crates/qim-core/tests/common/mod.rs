//! 測試共用：由字頻造一份只有 unigram 的語言模型。

use qim_core::{CharModel, LanguageModel, LanguageModelPack};

/// 字的分數是 `ln((字頻 + 1) ÷ (字頻總和 + 連打字集的字數))`，不論前一字（沒有 bigram、α = 1）。
/// `continuous` 是連打字集的字數，沒有字頻的字也要算進分母。
pub fn unigram(id: &str, frequency: &[(char, u64)], continuous: usize) -> LanguageModelPack {
    let total = frequency.iter().map(|(_, f)| *f as f64).sum::<f64>() + continuous as f64;
    let model = |f: u64| {
        let p = ((f + 1) as f64 / total).ln();
        CharModel {
            start: p,
            continuation: p,
            backoff: 0.0,
        }
    };
    LanguageModelPack {
        id: id.into(),
        model: LanguageModel::new(frequency.iter().map(|&(c, f)| (c, model(f))), model(0), []),
    }
}
