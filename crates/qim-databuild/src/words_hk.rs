//! 粵典（words.hk）「細數據」頁的詞頭表與詞頻 → 可選詞庫資料包的詞（plan〈詞庫〉，M3 票 24）。
//!
//! 兩個檔都是 CRLF、沒有引號的 CSV，首行是 `Source,<頁面網址>`：`wordslist.csv` 每行「詞頭,讀音,…」
//! （多讀音多欄），`existingwordcount.csv` 每行「詞,次數」（詞在粵文庫的出現次數，只含見過的詞）。
//! 授權照較嚴的非商業條款處理，只進 `quick-lexicon-words-hk`（`data/sources.toml` 的註解）。

use std::collections::BTreeMap;

use crate::Error;

/// `data/sources.toml` 的 `[source.words_hk]`。
pub const WORDS_HK: &str = "words_hk";
/// 詞頭表。
pub const WORDSLIST: &str = "wordslist.csv";
/// 詞在粵文庫的出現次數。
pub const WORD_COUNT: &str = "existingwordcount.csv";

/// 含異體字的詞頭結尾加的記號（頁面的簡介）。
const VARIANT_MARK: char = '*';

/// 兩字以上的詞頭（去掉 [`VARIANT_MARK`]、去重）與它在粵文庫的次數，沒見過的是 0。
/// 字集、寫法與要不要收由 `lexicon::build` 決定；單字不收（字由碼表給）。
pub fn headwords(wordslist: &str, word_count: &str) -> Result<BTreeMap<String, u64>, Error> {
    let mut counts: BTreeMap<&str, u64> = BTreeMap::new();
    for (i, line) in rows(WORD_COUNT, word_count)? {
        let invalid = || Error::invalid(format!("{WORD_COUNT} 第 {i} 行"), "應是「詞,次數」");
        let (word, count) = line.split_once(',').ok_or_else(invalid)?;
        counts.insert(word, count.parse().map_err(|_| invalid())?);
    }
    let mut words = BTreeMap::new();
    for (i, line) in rows(WORDSLIST, wordslist)? {
        let marked = line.split(',').next().unwrap_or_default();
        let word = marked.strip_suffix(VARIANT_MARK).unwrap_or(marked);
        if word.is_empty() {
            return Err(Error::invalid(
                format!("{WORDSLIST} 第 {i} 行"),
                "詞頭是空的",
            ));
        }
        if word.chars().count() >= 2 {
            let count = counts
                .get(marked)
                .or(counts.get(word))
                .copied()
                .unwrap_or(0);
            words.insert(word.to_string(), count);
        }
    }
    Ok(words)
}

/// 首行 `Source,…` 之後的每一行與它的行號（從 1 算）。
fn rows<'a>(file: &str, text: &'a str) -> Result<impl Iterator<Item = (usize, &'a str)>, Error> {
    let mut lines = text.lines().enumerate().map(|(i, line)| (i + 1, line));
    match lines.next() {
        Some((_, first)) if first.starts_with("Source,") => {}
        _ => {
            return Err(Error::invalid(
                format!("{file} 第 1 行"),
                "應是「Source,<頁面網址>」：上游的格式變了",
            ));
        }
    }
    Ok(lines.filter(|(_, line)| !line.is_empty()))
}
