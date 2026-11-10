//! 詞庫資料包的內容（plan §4.2）：rime-essay、粵語口語詞表與 HKCanCor train 切分的多字詞展開成
//! 速成碼索引，加上語料的字頻。

use std::collections::{BTreeMap, BTreeSet, HashMap};

use qim_core::{CodeTable, Lexicon, Word};

use crate::Error;
use crate::common_forms::BothForms;
use crate::transcode::Transcoder;

/// 最低詞頻，低於它的詞不收；初值與 rime-quick 舊版的 `min_phrase_weight` 相同，由基準調整。
pub const MIN_FREQUENCY: u64 = 100;

/// 一個詞最多收幾個碼的組合。rime-essay `054920d` 門檻以上、字都在連打字集的 25 萬詞，
/// 99.4% 不超過 4 個組合，超過 8 個的只有 80 詞（票 06）。
pub const MAX_COMBINATIONS: usize = 8;

/// 語料的詞至少在 train 切分出現幾次才收：只出現一次的多是轉寫瑕疵與專名（M3 票 18）。
pub const CORPUS_MIN_COUNT: u64 = 2;

/// 語料的詞每出現一次算多少詞頻（rime-essay 的尺度）：兩邊都有的詞，詞頻比次數的下四分位，
/// 怎麼定見 plan §4.2 與 M3 票 18 Comments。
pub const CORPUS_WORD_FREQUENCY: u64 = 300;

/// 建一次詞庫的統計，給 `--verify` 的報告。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    /// rime-essay 的列數。
    pub entries: usize,
    /// 單字的列，字由碼表給，不進詞庫。
    pub single_characters: usize,
    /// 詞頻低於 [`MIN_FREQUENCY`] 的詞。
    pub below_threshold: usize,
    /// 門檻以上、但含連打字集以外的字的詞。
    pub outside_continuous: usize,
    /// 收進詞庫的詞（重複的列算一個）。
    pub words: usize,
    /// 碼的組合超過 [`MAX_COMBINATIONS`]、只收了一部分的詞。
    pub capped: usize,
    /// 詞庫裡的輸入碼個數。
    pub codes: usize,
    /// 詞庫裡（輸入碼, 詞）的個數。
    pub indexed: usize,
    /// 粵語口語詞表的詞數。
    pub colloquial: usize,
    /// 語料的詞補進來的詞數（essay 與口語詞表沒收的）。
    pub corpus: usize,
    /// rime-essay 的詞（兩字以上，重複的列算一個）經 [`Forms::normalize`] 換了寫法的詞數。
    pub converted: usize,
    /// 換了寫法之後和別的詞撞成同一個、詞頻相加的詞數。
    pub merged: usize,
    /// 並存寫法另外收的寫法數（一個詞的每種寫法各算一個，不含詞本身；撞上別的詞的不算）。
    pub spellings: usize,
}

/// 詞的寫法（brief §4.1、plan §4.2）。
pub struct Forms<'a> {
    /// 把詞換成語料的寫法：`t2hk` 再換回常見寫法、並存寫法換成代表寫法，和
    /// `corpora::Converter`（`Conversion::T2hk`）同一套。
    pub normalize: &'a dyn Fn(&str) -> String,
    /// 並存寫法：代表寫法的詞也用另一種寫法收一份，詞頻相同；字頻兩種寫法相加後共用。
    pub both: &'a BothForms,
}

impl Forms<'_> {
    /// 詞換成語料的寫法，單向寫法反過來寫的錯字改回原字。
    fn common(&self, text: &str) -> String {
        self.both.correct(&(self.normalize)(text))
    }
}

/// 粵語口語詞表在 repo 裡的位置（plan §4.2、brief §5.2）：QIM 自行整理，CC0。
pub const COLLOQUIAL_PATH: &str = "data/lexicon/yue-colloquial.tsv";

/// 由 rime-essay 的 `essay.txt`（一行「詞<TAB>詞頻」）與粵語口語詞表 `colloquial`（[`COLLOQUIAL_PATH`]，
/// 同格式，`#` 開頭的行與空行略過）、語料 train 切分的詞與次數 `corpus`（`hkcancor::TrainWords`）建詞庫。
///
/// - 口語詞表只補 essay 沒收的詞，是人工整理的：單字、詞頻低於 [`MIN_FREQUENCY`]、含連打字集以外
///   的字、重複、essay 已收（詞頻達門檻）的詞都算錯，不默默濾掉；essay 有但低於門檻的詞用詞表的詞頻。
/// - 語料的詞只補 essay（達門檻）與口語詞表都沒收的詞：次數 ≥ [`CORPUS_MIN_COUNT`] 才收，詞頻是
///   次數乘 [`CORPUS_WORD_FREQUENCY`]；是機器抽的，含連打字集以外的字的詞默默略過、不算錯。
/// - 單字、詞頻低於 [`MIN_FREQUENCY`] 的詞、含連打字集以外的字的詞不收；同一個詞列了幾次取最大詞頻。
/// - essay 與詞表的詞都經 `forms.normalize` 換成語料的寫法（「羣組」→「群組」）：換完撞成同一個詞的
///   照 [`combined`] 合併，門檻與字集看換完的詞；單向寫法反過來寫的錯字（「顯着」）改回原字。
///   並存寫法的詞每種寫法都收（`forms.both`），詞頻相同；單向生出的寫法撞上 essay 本來就有的詞時
///   也照 [`combined`] 合併。
/// - 詞的碼是每個字的一個速成碼接起來（`code_table` 的連打字集）。多碼字的組合依換掉主碼
///   （`transcoder` 的碼，基準也用它）的字數由少到多取，最多 [`MAX_COMBINATIONS`] 個。
/// - 每個碼的詞依詞頻由高到低、同頻依碼位排。
/// - `char_frequency`（[`char_frequency`]）放進詞庫，並存寫法的兩個字共用兩者的和。
pub fn build(
    essay: &str,
    colloquial: &str,
    corpus: &BTreeMap<String, u64>,
    code_table: &CodeTable,
    transcoder: &Transcoder,
    mut char_frequency: BTreeMap<char, u64>,
    forms: &Forms,
) -> Result<(Lexicon, Counts), Error> {
    let codes = char_codes(code_table, transcoder);
    let mut counts = Counts::default();
    let mut essay_words: BTreeMap<&str, u64> = BTreeMap::new();
    for (i, line) in essay.lines().enumerate() {
        let invalid = |reason: &str| Error::invalid(format!("essay.txt 第 {} 行", i + 1), reason);
        let (text, frequency) = word_and_frequency(line, &invalid)?;
        counts.entries += 1;
        match text.chars().count() {
            0 => return Err(invalid("詞是空的")),
            1 => counts.single_characters += 1,
            _ => {
                let kept = essay_words.entry(text).or_insert(frequency);
                *kept = (*kept).max(frequency);
            }
        }
    }
    let mut merging: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    for (text, frequency) in essay_words {
        let common = forms.common(text);
        counts.converted += usize::from(common != text);
        merging.entry(common).or_default().push(frequency);
    }
    counts.merged = merging.values().map(|f| f.len() - 1).sum();
    let mut words: BTreeMap<String, u64> = merging
        .into_iter()
        .map(|(text, frequencies)| (text, combined(&frequencies)))
        .collect();

    let mut listed: BTreeSet<String> = BTreeSet::new();
    for (i, line) in colloquial.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let invalid =
            |reason: &str| Error::invalid(format!("{COLLOQUIAL_PATH} 第 {} 行", i + 1), reason);
        let (text, frequency) = word_and_frequency(line, &invalid)?;
        if text.chars().count() < 2 {
            return Err(invalid("只收兩字以上的詞"));
        }
        if frequency < MIN_FREQUENCY {
            return Err(invalid(&format!("詞頻要 ≥ {MIN_FREQUENCY}")));
        }
        if let Some(ch) = text.chars().find(|ch| !codes.contains_key(ch)) {
            return Err(invalid(&format!("「{ch}」不在連打字集")));
        }
        let common = forms.common(text);
        if !listed.insert(common.clone()) {
            return Err(invalid(&format!("「{text}」重複")));
        }
        let kept = words.entry(common).or_insert(0);
        if *kept >= MIN_FREQUENCY {
            return Err(invalid(&format!(
                "「{text}」rime-essay 已收（詞頻 {kept}），不必列"
            )));
        }
        *kept = frequency;
        counts.colloquial += 1;
    }

    let mut corpus_counts: BTreeMap<String, u64> = BTreeMap::new();
    for (text, count) in corpus {
        *corpus_counts.entry(forms.common(text)).or_default() += count;
    }
    for (text, count) in corpus_counts {
        let coded = text.chars().all(|ch| codes.contains_key(&ch));
        if count < CORPUS_MIN_COUNT || !coded || listed.contains(&text) {
            continue;
        }
        let kept = words.entry(text).or_insert(0);
        if *kept < MIN_FREQUENCY {
            *kept = count * CORPUS_WORD_FREQUENCY;
            counts.corpus += 1;
        }
    }

    let per_char = |text: &str| {
        text.chars()
            .map(|ch| codes.get(&ch).map(Vec::as_slice))
            .collect::<Option<Vec<_>>>()
    };
    // 每個詞的每種寫法：單向的寫法可能和另一個詞撞成同一個（隨着寫成隨著、essay 也有隨著），
    // 兩者是同一個義項（義項不同的不生出這個寫法，見 `BothForms::spellings`）。
    let mut spelled: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    for (text, frequency) in words {
        if frequency < MIN_FREQUENCY {
            counts.below_threshold += 1;
            continue;
        }
        if per_char(&text).is_none() {
            counts.outside_continuous += 1;
            continue;
        }
        counts.words += 1;
        for spelling in forms.both.spellings(&text) {
            spelled.entry(spelling).or_default().push(frequency);
        }
    }
    counts.spellings = spelled.len() - counts.words;
    let mut index: BTreeMap<String, Vec<Word>> = BTreeMap::new();
    for (text, frequencies) in spelled {
        let frequency = combined(&frequencies);
        // 並存寫法的字都在連打字集（`CommonForms::new` 檢查過），每種寫法都查得到碼。
        let Some(per_char) = per_char(&text) else {
            continue;
        };
        let (combinations, capped) = combinations(&per_char);
        counts.capped += usize::from(capped);
        for code in combinations {
            index.entry(code).or_default().push(Word {
                text: text.clone(),
                frequency,
            });
        }
    }
    for words in index.values_mut() {
        words.sort_by(|a, b| b.frequency.cmp(&a.frequency).then(a.text.cmp(&b.text)));
    }
    counts.codes = index.len();
    counts.indexed = index.values().map(Vec::len).sum();
    forms.both.share_frequency(&mut char_frequency);
    Ok((Lexicon::new(index, char_frequency), counts))
}

/// 撞成同一個詞的幾筆詞頻：相同的是同一份計數列了幾次（rime-essay 的「巔峯」「巔峰」都是 9448），
/// 只算一次；不同的是各自的計數（「才是」「纔是」），相加。
fn combined(frequencies: &[u64]) -> u64 {
    frequencies
        .iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .sum()
}

/// 「詞<TAB>詞頻」的一行；`invalid` 帶上檔名與行號。
fn word_and_frequency<'a>(
    line: &'a str,
    invalid: &dyn Fn(&str) -> Error,
) -> Result<(&'a str, u64), Error> {
    let mut fields = line.split('\t');
    let (Some(text), Some(frequency), None) = (fields.next(), fields.next(), fields.next()) else {
        return Err(invalid("應是「詞<TAB>詞頻」"));
    };
    let frequency = frequency.parse().map_err(|_| invalid("詞頻應是整數"))?;
    Ok((text, frequency))
}

/// 連打字集每個字的速成碼，轉碼用的主碼在第一個，其餘依字母序。
fn char_codes<'a>(
    code_table: &'a CodeTable,
    transcoder: &Transcoder,
) -> HashMap<char, Vec<&'a str>> {
    let mut codes: HashMap<char, Vec<&str>> = HashMap::new();
    for (code, entries) in code_table.iter() {
        for e in entries.iter().filter(|e| !e.extension) {
            codes.entry(e.text).or_default().push(code);
        }
    }
    for (&ch, list) in &mut codes {
        // 碼表依字母序列出，所以 `list` 已依字母序；主碼移到最前面，其餘順序不變。
        if let Some(main) = transcoder.code(ch)
            && let Some(at) = list.iter().position(|c| *c == main)
        {
            let main = list.remove(at);
            list.insert(0, main);
        }
    }
    codes
}

/// 每個字取一個碼接起來的組合，與不同的碼是否超過上限。依換掉主碼的字數、再依各字取第幾個碼
/// 排序，取前 [`MAX_COMBINATIONS`] 個不同的碼：不同取法可能接出同一個碼，重複的不佔上限。
fn combinations(per_char: &[&[&str]]) -> (BTreeSet<String>, bool) {
    let mut picks: Vec<Vec<usize>> = vec![Vec::new()];
    for codes in per_char {
        picks = picks
            .into_iter()
            .flat_map(|pick| {
                (0..codes.len()).map(move |i| {
                    let mut next = pick.clone();
                    next.push(i);
                    next
                })
            })
            .collect();
    }
    picks.sort_by_key(|pick| (pick.iter().filter(|&&i| i != 0).count(), pick.clone()));
    let mut codes = BTreeSet::new();
    for pick in picks {
        let code: String = pick.iter().zip(per_char).map(|(&i, c)| c[i]).collect();
        if codes.len() == MAX_COMBINATIONS {
            if !codes.contains(&code) {
                return (codes, true);
            }
        } else {
            codes.insert(code);
        }
    }
    (codes, false)
}

/// 各語料 train 切分的字頻（`build/corpus/<名>/chars.tsv`，一行「字<TAB>次數」）乘上語料的
/// 權重再相加；權重和 bigram 的次數用同一個（`corpora.rs` 的 `weight`，plan §4.3）。
pub fn char_frequency(tables: &[(&str, u64)]) -> Result<BTreeMap<char, u64>, Error> {
    let mut total = BTreeMap::new();
    for &(table, weight) in tables {
        for (i, line) in table.lines().enumerate() {
            let parsed = line.split_once('\t').and_then(|(ch, count)| {
                let mut chars = ch.chars();
                match (chars.next(), chars.next(), count.parse::<u64>()) {
                    (Some(ch), None, Ok(count)) => Some((ch, count)),
                    _ => None,
                }
            });
            let (ch, count) = parsed.ok_or_else(|| {
                Error::invalid(format!("chars.tsv 第 {} 行", i + 1), "應是「字<TAB>次數」")
            })?;
            *total.entry(ch).or_insert(0) += count * weight;
        }
    }
    Ok(total)
}
