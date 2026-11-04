//! 語料管線的切句、切分與基準取樣（plan §4.3、§5.2）。
//!
//! 一句是兩個分隔之間的文字；分隔是標點、空白與符號（不是字母也不是數字的字元）。
//! 含連打字集以外字元（英文字母、數字、擴展區的字）的句子整句丟掉：拿掉那些字會拼出原文
//! 沒有的相鄰字，污染 bigram，也不是使用者會一口氣連打的句子。
//!
//! 切分看句子本身的雜湊，同一句永遠落在同一個切分，test 句原樣出現在 train 的數量因此為 0。

use std::collections::HashSet;
use std::io::{self, Write};

use sha2::{Digest, Sha256};

use crate::transcode::Transcoder;

/// 每個語料進 `bench/` 的 test 句數上限：句子進 git，夠量出正確率的變化就好。
pub const BENCH_SENTENCES: usize = 2000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Split {
    Train,
    Dev,
    Test,
}

/// 文字裡的句子，依原順序。
pub fn clauses(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
}

/// train 90%、dev 5%、test 5%，由句子的 sha256 決定。
pub fn split_of(sentence: &str) -> Split {
    match key(sentence) % 100 {
        0..5 => Split::Test,
        5..10 => Split::Dev,
        _ => Split::Train,
    }
}

/// sha256 的前 8 位元組；切分與取樣都用它。
fn key(sentence: &str) -> u64 {
    let digest = Sha256::digest(sentence.as_bytes());
    u64::from_be_bytes(digest[..8].try_into().expect("sha256 有 32 位元組"))
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    /// 寫進 train 的句數（重複的句子照算，bigram 要頻次）。
    pub train: usize,
    pub dev: usize,
    /// 落在 test 的不重複句數；其中最多 [`BENCH_SENTENCES`] 句進基準，其餘只是不參與訓練。
    pub test: usize,
    /// 含連打字集以外字元而丟掉的句數。
    pub dropped: usize,
}

/// 一句一行寫出 train 與 dev，留下 test 句最後取樣。
pub struct Builder<'t, W> {
    transcoder: &'t Transcoder,
    train: W,
    dev: W,
    test: HashSet<String>,
    counts: Counts,
}

impl<'t, W: Write> Builder<'t, W> {
    pub fn new(transcoder: &'t Transcoder, train: W, dev: W) -> Self {
        Self {
            transcoder,
            train,
            dev,
            test: HashSet::new(),
            counts: Counts::default(),
        }
    }

    /// 加入一段已轉成繁體的內文。
    pub fn add(&mut self, text: &str) -> io::Result<()> {
        for clause in clauses(text) {
            if self.transcoder.transcode(clause).is_err() {
                self.counts.dropped += 1;
                continue;
            }
            match split_of(clause) {
                Split::Train => {
                    self.counts.train += 1;
                    writeln!(self.train, "{clause}")?;
                }
                Split::Dev => {
                    self.counts.dev += 1;
                    writeln!(self.dev, "{clause}")?;
                }
                Split::Test => {
                    self.test.insert(clause.to_string());
                }
            }
        }
        Ok(())
    }

    /// 回傳句數、基準的內容（`bench/` 的 TSV）與兩個 writer。基準取雜湊最小的
    /// [`BENCH_SENTENCES`] 句、依雜湊排序，和來源的順序無關。
    pub fn finish(mut self) -> io::Result<Finished<W>> {
        self.train.flush()?;
        self.dev.flush()?;
        self.counts.test = self.test.len();
        let mut test: Vec<(u64, String)> = self.test.into_iter().map(|s| (key(&s), s)).collect();
        test.sort_unstable();
        let bench = test
            .iter()
            .take(BENCH_SENTENCES)
            .map(|(_, sentence)| {
                let codes = self
                    .transcoder
                    .transcode(sentence)
                    .expect("加入時已確認轉得了碼");
                format!("{}\t{sentence}\n", codes.join(" "))
            })
            .collect();
        Ok(Finished {
            counts: self.counts,
            bench,
            train: self.train,
            dev: self.dev,
        })
    }
}

pub struct Finished<W> {
    pub counts: Counts,
    /// 一行一句：逐字速成碼以空格分隔，tab，句子。
    pub bench: String,
    pub train: W,
    pub dev: W,
}
