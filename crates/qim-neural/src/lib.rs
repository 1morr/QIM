//! 停鍵層的神經重排（plan〈模型分層〉、ADR 0005）：Gboard 級的字級 CIFG-LSTM（約 1.4M 參數）給 lattice
//! 的前幾條路徑打分，照 `路徑分 + λ·神經分` 回名次。
//!
//! 模型與字表由殼以位元組交進來（模型資料包，plan〈資料包 manifest〉），不碰檔案、不開執行緒；殼在背景執行緒或
//! Web Worker 呼叫。實作從模型階梯的實驗（`scripts/experiments/model-ladder/src/neural.rs`）搬來，
//! 只留採用的 LSTM。依賴規則見 `docs/plan.md` §1.3。

mod lstm;
mod vocab;

use std::fmt;

pub use lstm::{Format, Model, to_gguf};
pub(crate) use vocab::Vocab;

/// 交給模型的前文最多幾個字（brief〈隱私與安全〉，模型階梯定的 50 字）：取最後這麼多字。
pub const CONTEXT_CHARS: usize = 50;

/// 句首與子句之間的記號：訓練語料在標點處切句，前文裡的標點、空白、符號也換成它。
pub(crate) const SEP: u32 = 0;
/// 字表外的字；前文裡連續的英文、數字也收成一個。
pub(crate) const UNK: u32 = 1;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// 權重讀不了，或不是這個 crate 認得的模型。
    InvalidModel(String),
    /// 字表和模型對不上，或字重複。
    InvalidVocab(String),
    /// λ 要是有限的非負數。
    InvalidLambda(f64),
    /// 推論失敗：candle 回的錯誤，或交進來的 token 超出字表、路徑是空的（[`Model::score`]）。
    Inference(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidModel(e) => write!(f, "模型讀不了：{e}"),
            Self::InvalidVocab(e) => write!(f, "字表不合：{e}"),
            Self::InvalidLambda(l) => write!(f, "λ 要是有限的非負數：{l}"),
            Self::Inference(e) => write!(f, "推論失敗：{e}"),
        }
    }
}

impl std::error::Error for Error {}

/// 停鍵層的重排：模型、字表與神經分的權重 λ。
pub struct Reranker {
    model: Model,
    vocab: Vocab,
    lambda: f64,
}

impl Reranker {
    /// `weights` 是 GGUF 權重（q8 或 f32，[`to_gguf`]）；`vocab` 是字表裡兩個記號之後的字，依序
    /// 對應 token 2、3…；`unknown_chars` 是訓練語料裡字表外的字數（[`Vocab::new`]）。
    pub fn new(
        weights: &[u8],
        vocab: &[char],
        unknown_chars: u32,
        lambda: f64,
    ) -> Result<Self, Error> {
        if !lambda.is_finite() || lambda < 0.0 {
            return Err(Error::InvalidLambda(lambda));
        }
        let model = Model::from_gguf(weights)?;
        let vocab = Vocab::new(vocab, unknown_chars)?;
        if vocab.len() != model.vocab_size() {
            return Err(Error::InvalidVocab(format!(
                "字表連記號 {} 個，模型的 embedding 有 {} 列",
                vocab.len(),
                model.vocab_size()
            )));
        }
        Ok(Self {
            model,
            vocab,
            lambda,
        })
    }

    /// 各條路徑的神經分（nat）：前文取最後 [`CONTEXT_CHARS`] 字，路徑各字 log 機率的和；字表外的字
    /// 另扣 [`Vocab::unknown_penalty`]。空字串的路徑是 0（沒有字）。
    pub(crate) fn scores(&self, before: &str, paths: &[&str]) -> Result<Vec<f32>, Error> {
        let skip = before.chars().count().saturating_sub(CONTEXT_CHARS);
        let tail: String = before.chars().skip(skip).collect();
        let context = self.vocab.context(&tail);
        let tokens: Vec<(Vec<u32>, usize)> = paths.iter().map(|p| self.vocab.path(p)).collect();
        // 空字串沒有字可算，分數是 0；其餘的一批算。
        let nonempty: Vec<Vec<u32>> = tokens
            .iter()
            .filter(|(t, _)| !t.is_empty())
            .map(|(t, _)| t.clone())
            .collect();
        let mut scored = self.model.score(&context, &nonempty)?.into_iter();
        Ok(tokens
            .iter()
            .map(|(t, unknown)| {
                if t.is_empty() {
                    return 0.0;
                }
                let raw = scored.next().expect("每條非空的路徑都有分數");
                raw - self.vocab.unknown_penalty() * *unknown as f32
            })
            .collect())
    }

    /// 路徑（文字、同步層的路徑分）依 `路徑分 + λ·神經分` 由高到低的序號；同分的照原本的順序。
    pub fn rank(&self, before: &str, paths: &[(&str, f64)]) -> Result<Vec<usize>, Error> {
        Ok(self.rank_scored(before, paths)?.order)
    }

    /// 同 [`Reranker::rank`]，另給各路徑的 `路徑分 + λ·神經分`（網頁的 debug 面板顯示）。
    pub fn rank_scored(&self, before: &str, paths: &[(&str, f64)]) -> Result<Ranking, Error> {
        let texts: Vec<&str> = paths.iter().map(|(t, _)| *t).collect();
        let neural = self.scores(before, &texts)?;
        let totals: Vec<f64> = paths
            .iter()
            .zip(&neural)
            .map(|((_, score), &n)| score + self.lambda * f64::from(n))
            .collect();
        let mut order: Vec<usize> = (0..paths.len()).collect();
        // 穩定排序：同分的照原本的順序。
        order.sort_by(|&a, &b| totals[b].total_cmp(&totals[a]));
        Ok(Ranking { order, totals })
    }
}

/// [`Reranker::rank_scored`] 的結果。
#[derive(Debug, Clone, PartialEq)]
pub struct Ranking {
    /// 路徑的序號，好的在前。
    pub order: Vec<usize>,
    /// 各路徑的 `路徑分 + λ·神經分`，和輸入的路徑同順序。
    pub totals: Vec<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const TINY: &[u8] = include_bytes!("../tests/fixtures/tiny-lstm.safetensors");
    /// 測試模型的字表 12 個：兩個記號加這 10 個字。
    const CHARS: [char; 10] = ['我', '你', '的', '是', '好', '人', '天', '大', '小', '中'];

    fn model(format: Format) -> Model {
        Model::from_gguf(&to_gguf(TINY, format).unwrap()).unwrap()
    }

    fn reranker(lambda: f64) -> Reranker {
        Reranker::new(&to_gguf(TINY, Format::Q8).unwrap(), &CHARS, 5, lambda).unwrap()
    }

    #[test]
    fn prefix_cache_scores_like_scoring_each_path_from_the_start() {
        let context = [3, 7, 1, 4, 0, 9];
        let paths = vec![vec![2, 5, 11], vec![6], vec![10, 3, 3, 8]];
        for format in [Format::F32, Format::Q8] {
            let m = model(format);
            let batched = m.score(&context, &paths).unwrap();
            for (path, got) in paths.iter().zip(&batched) {
                let alone = m.score_one(&context, path).unwrap();
                assert!(
                    (got - alone).abs() < 1e-4,
                    "{format:?} {path:?}：{got} 對 {alone}"
                );
            }
        }
    }

    #[test]
    fn padding_is_masked_when_paths_differ_in_length() {
        // 最短的路徑補了 8 個記號；遮罩漏掉的話它會多算那幾個位置的 log 機率。
        let m = model(Format::F32);
        let paths = vec![vec![4], vec![2, 3, 4, 5, 6, 7, 8, 9, 10]];
        let batched = m.score(&[5, 6], &paths).unwrap();
        let alone = m.score(&[5, 6], &paths[..1]).unwrap();
        assert!(
            (batched[0] - alone[0]).abs() < 1e-5,
            "{} 對 {}",
            batched[0],
            alone[0]
        );
    }

    #[test]
    fn empty_paths_do_not_panic() {
        let r = reranker(2.0);
        assert_eq!(r.rank("你好", &[]).unwrap(), Vec::<usize>::new());
        assert_eq!(r.scores("你好", &[]).unwrap(), Vec::<f32>::new());
        let scores = r.scores("", &["", "我", ""]).unwrap();
        assert_eq!(scores[0], 0.0);
        assert_eq!(scores[2], 0.0);
        assert!(scores[1] < 0.0);
        assert_eq!(r.rank("", &[("", 0.0)]).unwrap(), vec![0]);
    }

    #[test]
    fn rank_adds_lambda_times_the_neural_score() {
        let texts = ["我的", "你好", "大人", "中天小"];
        let r = reranker(2.0);
        let neural = r.scores("天", &texts).unwrap();
        let paths: Vec<(&str, f64)> = texts.iter().map(|t| (*t, -3.0)).collect();
        // 路徑分相同時只看神經分。
        let mut by_neural: Vec<usize> = (0..texts.len()).collect();
        by_neural.sort_by(|&a, &b| neural[b].total_cmp(&neural[a]));
        assert_eq!(r.rank("天", &paths).unwrap(), by_neural);
        // λ 是 0 時只看路徑分，同分照原本的順序。
        let r0 = reranker(0.0);
        let paths = [("我的", -2.0), ("你好", -1.0), ("大人", -2.0)];
        assert_eq!(r0.rank("天", &paths).unwrap(), vec![1, 0, 2]);
        // 路徑分差得比 λ·神經分的差大時路徑分贏。
        let (hi, lo) = (by_neural[0], by_neural[texts.len() - 1]);
        let gap = 2.0 * f64::from(neural[hi] - neural[lo]);
        let paths = [(texts[hi], -11.0 - gap), (texts[lo], -10.0)];
        assert_eq!(r.rank("天", &paths).unwrap(), vec![1, 0]);
    }

    #[test]
    fn rank_scored_gives_each_path_its_total_alongside_the_order() {
        // λ 是 0 時總分就是路徑分。
        let r0 = reranker(0.0);
        let paths = [("我的", -2.0), ("你好", -1.0), ("大人", -2.5)];
        let ranking = r0.rank_scored("天", &paths).unwrap();
        assert_eq!(ranking.order, vec![1, 0, 2]);
        assert_eq!(ranking.totals, vec![-2.0, -1.0, -2.5]);
        // λ 不是 0 時是路徑分加 λ·神經分，名次同 `rank`。
        let r = reranker(2.0);
        let neural = r.scores("天", &["我的", "你好", "大人"]).unwrap();
        let ranking = r.rank_scored("天", &paths).unwrap();
        for (i, (_, score)) in paths.iter().enumerate() {
            let expected = score + 2.0 * f64::from(neural[i]);
            assert!((ranking.totals[i] - expected).abs() < 1e-9);
        }
        assert_eq!(ranking.order, r.rank("天", &paths).unwrap());
    }

    #[test]
    fn only_the_last_context_chars_reach_the_model() {
        let r = reranker(2.0);
        let tail: String = "我你的是好人天大小中".repeat(5);
        assert_eq!(tail.chars().count(), CONTEXT_CHARS);
        let long = format!("中中中中中中{tail}");
        assert_eq!(
            r.scores(&long, &["好人"]).unwrap(),
            r.scores(&tail, &["好人"]).unwrap()
        );
        assert_ne!(
            r.scores("中", &["好人"]).unwrap(),
            r.scores("", &["好人"]).unwrap(),
            "前文要有影響，上面的比較才有意義"
        );
    }

    #[test]
    fn unknown_characters_cost_the_penalty() {
        let r = reranker(2.0);
        // 「龍」不在字表：當成 <unk>，另扣 ln(5)。
        let unk = r.scores("", &["龍"]).unwrap()[0];
        let token = model(Format::Q8).score(&[], &[vec![UNK]]).unwrap()[0];
        assert!((unk - (token - 5f32.ln())).abs() < 1e-5, "{unk} 對 {token}");
    }

    #[test]
    fn new_checks_vocab_and_lambda() {
        let q8 = to_gguf(TINY, Format::Q8).unwrap();
        assert!(matches!(
            Reranker::new(&q8, &CHARS[..9], 5, 2.0),
            Err(Error::InvalidVocab(_))
        ));
        let mut dup = CHARS;
        dup[9] = '我';
        assert!(matches!(
            Reranker::new(&q8, &dup, 5, 2.0),
            Err(Error::InvalidVocab(_))
        ));
        for bad in [-1.0, f64::NAN, f64::INFINITY] {
            assert!(matches!(
                Reranker::new(&q8, &CHARS, 5, bad),
                Err(Error::InvalidLambda(_))
            ));
        }
        assert!(matches!(
            Reranker::new(b"junk", &CHARS, 5, 2.0),
            Err(Error::InvalidModel(_))
        ));
    }
}
