//! AOSP LatinIME 的英文詞表 → 英文資料包的詞表。
//!
//! 取 `dictionaries/en_US_wordlist.combined.gz`：LatinIME 建 `main_en_us.dict` 用的文字檔，
//! 第一行是檔頭（`dictionary=main:en_us,…`），之後每個詞一行 ` word=<詞>,f=<0–255>,…`，
//! 那個詞的 bigram 與 shortcut 接在下面、以兩個空白開頭。`f` 是頻率的對數，大的常見。
//! 欄位的意思照 aosp-dictionaries 的 README〈Wordlist information〉（事實 T39）。

use std::io::Read;

use flate2::read::GzDecoder;
use qim_core::EnglishWords;

use crate::Error;

/// LatinIME repo 裡的詞表檔。
pub const WORDLIST: &str = "dictionaries/en_US_wordlist.combined.gz";

/// 英文詞表的建置統計。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    /// 詞表的詞數（不含 bigram 與 shortcut）。
    pub entries: usize,
    /// 標 `possibly_offensive=true` 或 `flags=offensive` 而不收的。
    pub offensive: usize,
    /// 只當 shortcut 用的錯拼（`not_a_word=true`，例如 `acomodate`）而不收的。
    pub not_a_word: usize,
    /// `f=0`（LatinIME 不建議，例如 `DOS`、`AMD`）而不收的。
    pub zero_frequency: usize,
    /// 依小寫歸併大小寫後收的詞數。
    pub words: usize,
}

/// 由 gzip 壓縮的詞表建詞表：LatinIME 不建議的詞（可能冒犯的、`not_a_word`、`f=0`）不收，
/// 其餘照收（含 `z`、撇號、連字號：英文模式也用它，`z` 只在中文組字比對時濾掉），交給
/// [`EnglishWords::new`] 依小寫歸併大小寫。
pub fn words(gz: &[u8]) -> Result<(EnglishWords, Counts), Error> {
    let mut text = String::new();
    GzDecoder::new(gz)
        .read_to_string(&mut text)
        .map_err(|e| Error::invalid(WORDLIST, e.to_string()))?;
    let mut lines = text.lines().enumerate();
    match lines.next() {
        Some((_, header)) if header.starts_with("dictionary=") => {}
        _ => {
            return Err(Error::invalid(
                WORDLIST,
                "第一行不是 dictionary= 開頭的檔頭",
            ));
        }
    }
    let mut counts = Counts::default();
    let mut kept = Vec::new();
    for (i, line) in lines {
        // bigram 與 shortcut 以兩個空白開頭，接在它的詞下面。
        let Some(entry) = line.strip_prefix(" word=") else {
            if line.starts_with("  ") || line.is_empty() {
                continue;
            }
            return Err(Error::invalid(
                format!("{WORDLIST} 第 {} 列", i + 1),
                "不是 word、bigram 或 shortcut",
            ));
        };
        let at = || format!("{WORDLIST} 第 {} 列", i + 1);
        let (word, attributes) = entry.split_once(',').unwrap_or((entry, ""));
        if word.is_empty() {
            return Err(Error::invalid(at(), "詞是空的"));
        }
        let mut frequency = None;
        let (mut offensive, mut not_a_word) = (false, false);
        for attribute in attributes.split(',') {
            match attribute.split_once('=') {
                Some(("f", f)) => {
                    frequency = Some(
                        f.parse::<u8>()
                            .map_err(|_| Error::invalid(at(), format!("f={f} 不是 0–255")))?,
                    );
                }
                Some(("possibly_offensive", "true")) | Some(("flags", "offensive")) => {
                    offensive = true;
                }
                Some(("not_a_word", "true")) => not_a_word = true,
                _ => {}
            }
        }
        let frequency = frequency.ok_or_else(|| Error::invalid(at(), "沒有 f"))?;
        counts.entries += 1;
        if offensive {
            counts.offensive += 1;
        } else if not_a_word {
            counts.not_a_word += 1;
        } else if frequency == 0 {
            counts.zero_frequency += 1;
        } else {
            kept.push((word.to_string(), frequency));
        }
    }
    let words = EnglishWords::new(kept);
    counts.words = words.len();
    Ok((words, counts))
}
