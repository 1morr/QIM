//! HKCanCor（香港粵語語料庫）的轉寫檔 → 話輪文字（plan §4.3 的口語語料）。
//!
//! 取 fcbond/hkcancor 的 `data/hkcancor-utf8.zip`：`utf8/` 下每個檔是一段錄音的轉寫，
//! 一個話輪一個 `<sent>`，`<sent_tag>` 裡一行一詞，寫成 `詞/詞性/粵拼/`。只取詞，
//! 照原順序接成一個話輪；標點也是詞（`，/w/VQ2/`），留給切句當分隔。夾英文詞或粵拼的話輪
//! 在切句後轉碼失敗、整句丟掉，同其他語料。
//!
//! 語料已分詞，詞庫另收 train 切分裡的詞（[`TrainWords`]，plan §4.2）。

use std::collections::BTreeMap;
use std::io::Read;

use crate::Error;
use crate::corpus::{Split, clauses, split_of};
use crate::transcode::Transcoder;

const WHAT: &str = "HKCanCor zip";

/// 轉寫裡的一個詞與它的詞性。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tagged<'a> {
    pub text: &'a str,
    pub tag: &'a str,
}

/// 一個轉寫檔裡每個話輪的詞，依原順序。
pub fn tagged_utterances(transcript: &str) -> Vec<Vec<Tagged<'_>>> {
    let mut out = Vec::new();
    let mut current: Option<Vec<Tagged>> = None;
    for line in transcript.lines().map(str::trim) {
        match line {
            "<sent_tag>" => current = Some(Vec::new()),
            "</sent_tag>" => out.extend(current.take()),
            word => {
                if let Some(words) = &mut current {
                    let mut fields = word.split('/');
                    words.push(Tagged {
                        text: fields.next().unwrap_or_default(),
                        tag: fields.next().unwrap_or_default(),
                    });
                }
            }
        }
    }
    out
}

/// 一個轉寫檔裡每個話輪的文字，依原順序。
pub fn utterances(transcript: &str) -> Vec<String> {
    tagged_utterances(transcript)
        .iter()
        .map(|words| words.iter().map(|w| w.text).collect())
        .collect()
}

/// 人名的詞性：人名不是使用者會當詞打的（「子欣」「艾爾頓」），不進詞庫。
const PERSON_NAME: &str = "nr";

/// 已分詞語料 train 切分裡兩字以上的詞與次數，加上對不上詞位置而略過的話輪數。
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WordCounts {
    pub words: BTreeMap<String, u64>,
    pub unaligned: usize,
}

impl WordCounts {
    /// 把另一份加進來，次數相加。
    pub fn merge(&mut self, other: WordCounts) {
        for (word, count) in other.words {
            *self.words.entry(word).or_default() += count;
        }
        self.unaligned += other.unaligned;
    }
}

/// 數 train 切分裡每個兩字以上的詞出現幾次（plan §4.2）。
///
/// 切句（[`clauses`]）、切分與丟句同語料管線（`corpus::Builder`），看轉換後的話輪，所以 dev、test
/// 的句子（首選字基準的題目）裡的詞不數；詞照轉換後的寫法數。轉換改了字數的話輪對不上詞的位置，
/// 整句不數。跨句的詞（含標點）、單字與人名不數。
pub struct TrainWords<'a> {
    convert: &'a dyn Fn(&str) -> String,
    transcoder: &'a Transcoder,
    pub counts: WordCounts,
}

impl<'a> TrainWords<'a> {
    /// `convert` 是這份語料的轉換（`corpora::Converter::convert`）。
    pub fn new(convert: &'a dyn Fn(&str) -> String, transcoder: &'a Transcoder) -> Self {
        Self {
            convert,
            transcoder,
            counts: WordCounts::default(),
        }
    }

    pub fn add(&mut self, utterance: &[Tagged]) {
        let raw: String = utterance.iter().map(|w| w.text).collect();
        let converted = (self.convert)(&raw);
        // 每個字在轉換後的話輪裡的位元組位置，最後多一個結尾。
        let offsets: Vec<usize> = converted
            .char_indices()
            .map(|(i, _)| i)
            .chain([converted.len()])
            .collect();
        if offsets.len() != raw.chars().count() + 1 {
            self.counts.unaligned += 1;
            return;
        }
        // 每個詞在轉換後的話輪裡的位元組範圍。
        let mut spans = Vec::with_capacity(utterance.len());
        let mut chars = 0;
        for word in utterance {
            let len = word.text.chars().count();
            if len >= 2 && word.tag != PERSON_NAME {
                spans.push(offsets[chars]..offsets[chars + len]);
            }
            chars += len;
        }
        for clause in clauses(&converted) {
            if self.transcoder.transcode(clause).is_err() || split_of(clause) != Split::Train {
                continue;
            }
            let from = clause.as_ptr() as usize - converted.as_ptr() as usize;
            let to = from + clause.len();
            for span in spans.iter().filter(|s| s.start >= from && s.end <= to) {
                *self
                    .counts
                    .words
                    .entry(converted[span.clone()].to_string())
                    .or_default() += 1;
            }
        }
    }
}

/// zip 裡 `utf8/` 下每個轉寫檔的話輪，檔案依名稱排序，和 zip 的存放順序無關。
pub fn archive(
    zip: &[u8],
    mut utterance: impl FnMut(&str) -> Result<(), Error>,
) -> Result<(), Error> {
    transcripts(zip, |text| {
        for u in utterances(text) {
            utterance(&u)?;
        }
        Ok(())
    })
}

/// zip 裡 `utf8/` 下每個轉寫檔（一段對話）的全部話輪，順序同 [`archive`]。
pub fn archive_dialogues(
    zip: &[u8],
    mut dialogue: impl FnMut(&[String]) -> Result<(), Error>,
) -> Result<(), Error> {
    transcripts(zip, |text| dialogue(&utterances(text)))
}

/// zip 裡 `utf8/` 下每個轉寫檔的話輪的詞，順序同 [`archive`]。
pub fn archive_tagged(zip: &[u8], mut utterance: impl FnMut(&[Tagged])) -> Result<(), Error> {
    transcripts(zip, |text| {
        for u in tagged_utterances(text) {
            utterance(&u);
        }
        Ok(())
    })
}

/// zip 裡 `utf8/` 下每個轉寫檔的全文，檔案依名稱排序。
fn transcripts(
    zip: &[u8],
    mut transcript: impl FnMut(&str) -> Result<(), Error>,
) -> Result<(), Error> {
    let invalid = |e: &dyn std::fmt::Display| Error::invalid(WHAT, e.to_string());
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip)).map_err(|e| invalid(&e))?;
    let mut names = Vec::new();
    for name in archive.file_names() {
        let name = name.map_err(|e| invalid(&e))?;
        if name.starts_with("utf8/") && !name.ends_with('/') {
            names.push(name.into_owned());
        }
    }
    names.sort();
    for name in names {
        let mut text = String::new();
        archive
            .by_name(&name)
            .map_err(|e| invalid(&e))?
            .read_to_string(&mut text)
            .map_err(|e| Error::invalid(format!("{WHAT} 的 {name}"), e.to_string()))?;
        transcript(&text)?;
    }
    Ok(())
}
