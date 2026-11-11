//! 語料管線的切句、切分與基準取樣（plan §4.3、§5.2）。
//!
//! 一句是兩個分隔之間的文字；分隔是標點、空白與符號（不是字母也不是數字的字元）。
//! 含連打字集以外字元（英文字母、數字、擴展區的字）的句子整句丟掉：拿掉那些字會拼出原文
//! 沒有的相鄰字，污染 bigram，也不是使用者會一口氣連打的句子。
//!
//! 切分看句子本身的雜湊，同一句永遠落在同一個切分，test 句原樣出現在 train 的數量因此為 0；
//! 各語料共用同一個切分函式，同一句出現在兩份語料也不會一邊 train、一邊 test。
//!
//! 字頻只數 train（plan §4.2），dev、test 不參與。
//!
//! 「有前文」的基準題（plan §5.2）：test 切分的句子配上同一段文字裡它前面的
//! [`CONTEXT_CLAUSES`] 句與中間的標點，前文原樣保留，不轉碼。dev 切分用同樣的取法另出一份調參題。
//! 同一批題另出一份長前文（同一段裡前面最多 [`LONG_CONTEXT_CHARS`] 字），給模型階梯量前文長度
//! （M3 票 07，`scripts/experiments/model-ladder/`）。
//!
//! 口語長前文的題（M3 票 25）：一段對話（HKCanCor 的一段錄音）的話輪各是一段，「有前文」與長前文
//! 不跨話輪；另出一份題，前文接同一段對話裡前面的話輪，最多 [`LONG_CONTEXT_CHARS`] 字，給模型階梯量
//! 口語的長前文。

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::io::{self, Write};

use sha2::{Digest, Sha256};

use crate::transcode::Transcoder;

/// 每個語料進 `bench/` 的 test 句數上限：句子進 git，夠量出正確率的變化就好。dev 切分的調參題
/// （`build/corpus/<名>/dev.tsv`）用同樣的上限與取法。
pub const BENCH_SENTENCES: usize = 2000;

/// 每個語料「有前文」的基準題數上限（`bench/<目錄>/context/<名>.tsv`）。
pub const CONTEXT_SENTENCES: usize = 1000;

/// 「有前文」的題，前文取目標句前面幾句。
pub const CONTEXT_CLAUSES: usize = 2;

/// 長前文最多幾個字：模型階梯量前文 0、50、200 字（`.scratch/m3/spec.md`〈模型階梯實驗〉）。
pub const LONG_CONTEXT_CHARS: usize = 200;

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

/// 一句一行寫出 train 與 dev，留下 test 與 dev 的不重複句最後取樣。
pub struct Builder<'t, W> {
    transcoder: &'t Transcoder,
    train: W,
    dev: W,
    test: HashSet<String>,
    dev_sentences: HashSet<String>,
    /// 「有前文」的題：（句子的雜湊, 句子）→ 前文；只留雜湊最小的 [`CONTEXT_SENTENCES`] 句。
    context: ContextItems,
    /// dev 切分的「有前文」調參題，取法同 `context`。
    dev_context: ContextItems,
    /// 口語長前文的題（[`Self::add_dialogue`]），test、dev 各一份；最多 [`BENCH_SENTENCES`] 句。
    dialogue: ContextItems,
    dev_dialogue: ContextItems,
    chars: HashMap<char, u64>,
    counts: Counts,
}

impl<'t, W: Write> Builder<'t, W> {
    pub fn new(transcoder: &'t Transcoder, train: W, dev: W) -> Self {
        Self {
            transcoder,
            train,
            dev,
            test: HashSet::new(),
            dev_sentences: HashSet::new(),
            context: ContextItems::new(CONTEXT_SENTENCES),
            dev_context: ContextItems::new(CONTEXT_SENTENCES),
            dialogue: ContextItems::new(BENCH_SENTENCES),
            dev_dialogue: ContextItems::new(BENCH_SENTENCES),
            chars: HashMap::new(),
            counts: Counts::default(),
        }
    }

    /// 加入一段已轉成繁體的內文。
    pub fn add(&mut self, text: &str) -> io::Result<()> {
        // 前面幾句（含丟掉的句子：它們也是前文）在 `text` 的開頭位置。
        let mut recent: VecDeque<usize> = VecDeque::with_capacity(CONTEXT_CLAUSES + 1);
        for clause in clauses(text) {
            let start = clause.as_ptr() as usize - text.as_ptr() as usize;
            let context = recent.front().map(|&from| line_tail(&text[from..start]));
            let long = last_chars(line_tail(&text[..start]), LONG_CONTEXT_CHARS);
            recent.push_back(start);
            if recent.len() > CONTEXT_CLAUSES {
                recent.pop_front();
            }
            if self.transcoder.transcode(clause).is_err() {
                self.counts.dropped += 1;
                continue;
            }
            // 前文要有字：只有標點的不算。
            let context = context.filter(|c| c.chars().any(char::is_alphanumeric));
            match split_of(clause) {
                Split::Train => {
                    self.counts.train += 1;
                    writeln!(self.train, "{clause}")?;
                    for c in clause.chars() {
                        *self.chars.entry(c).or_default() += 1;
                    }
                }
                Split::Dev => {
                    self.counts.dev += 1;
                    writeln!(self.dev, "{clause}")?;
                    self.dev_sentences.insert(clause.to_string());
                    if let Some(context) = context {
                        self.dev_context.offer(clause, context, long);
                    }
                }
                Split::Test => {
                    self.test.insert(clause.to_string());
                    if let Some(context) = context {
                        self.context.offer(clause, context, long);
                    }
                }
            }
        }
        Ok(())
    }

    /// 加入一段對話：每個話輪照 [`Self::add`] 當一段，另把 dev、test 的句子配上同一段對話裡它前面的
    /// 文字（前面的話輪接起來，最多 [`LONG_CONTEXT_CHARS`] 字）當口語長前文的題。話輪結尾不是標點時
    /// 補一個空格，免得兩個話輪的字黏成一句。前文要有字，所以對話的第一句不出題。
    pub fn add_dialogue(&mut self, turns: &[String]) -> io::Result<()> {
        let mut before = String::new();
        for turn in turns {
            self.add(turn)?;
            for clause in clauses(turn) {
                if self.transcoder.transcode(clause).is_err() {
                    continue;
                }
                let start = clause.as_ptr() as usize - turn.as_ptr() as usize;
                let context = format!("{before}{}", &turn[..start]);
                let context = last_chars(line_tail(&context), LONG_CONTEXT_CHARS);
                if !context.chars().any(char::is_alphanumeric) {
                    continue;
                }
                match split_of(clause) {
                    Split::Train => {}
                    Split::Dev => self.dev_dialogue.offer(clause, context, context),
                    Split::Test => self.dialogue.offer(clause, context, context),
                }
            }
            before.push_str(turn);
            if turn.chars().last().is_some_and(char::is_alphanumeric) {
                before.push(' ');
            }
        }
        Ok(())
    }

    /// 回傳句數、基準的內容（`bench/` 的 TSV）、dev 的調參題（同格式）、字頻與兩個 writer。
    /// 基準與調參題各取雜湊最小的 [`BENCH_SENTENCES`] 句、依雜湊排序，和來源的順序無關。
    pub fn finish(mut self) -> io::Result<Finished<W>> {
        self.train.flush()?;
        self.dev.flush()?;
        self.counts.test = self.test.len();
        let bench = self.sample(&self.test);
        let dev_items = self.sample(&self.dev_sentences);
        let context_bench = self.context_lines(&self.context, false);
        let dev_context_items = self.context_lines(&self.dev_context, false);
        let context_long_bench = self.context_lines(&self.context, true);
        let dev_context_long_items = self.context_lines(&self.dev_context, true);
        let dialogue_items = self.context_lines(&self.dialogue, false);
        let dev_dialogue_items = self.context_lines(&self.dev_dialogue, false);
        let mut chars: Vec<(char, u64)> = self.chars.into_iter().collect();
        chars.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        let chars = chars.iter().map(|(c, n)| format!("{c}\t{n}\n")).collect();
        Ok(Finished {
            counts: self.counts,
            bench,
            dev_items,
            context_bench,
            dev_context_items,
            context_long_bench,
            dev_context_long_items,
            dialogue_items,
            dev_dialogue_items,
            chars,
            train: self.train,
            dev: self.dev,
        })
    }

    /// 雜湊最小的 [`BENCH_SENTENCES`] 句，依雜湊排序，一行「逐字速成碼<TAB>句子」。
    fn sample(&self, sentences: &HashSet<String>) -> String {
        let mut keyed: Vec<(u64, &str)> = sentences.iter().map(|s| (key(s), s.as_str())).collect();
        keyed.sort_unstable();
        keyed
            .iter()
            .take(BENCH_SENTENCES)
            .map(|(_, sentence)| format!("{}\t{sentence}\n", self.codes(sentence)))
            .collect()
    }

    /// 「有前文」的題，一行「前文<TAB>逐字速成碼<TAB>句子」，依句子的雜湊排序；`long` 時前文是長前文。
    fn context_lines(&self, items: &ContextItems, long: bool) -> String {
        items
            .items
            .iter()
            .map(|((_, sentence), (context, long_context))| {
                let context = if long { long_context } else { context };
                format!("{context}\t{}\t{sentence}\n", self.codes(sentence))
            })
            .collect()
    }

    /// 逐字速成碼，空格分隔。
    fn codes(&self, sentence: &str) -> String {
        self.transcoder
            .transcode(sentence)
            .expect("加入時已確認轉得了碼")
            .join(" ")
    }
}

/// 「有前文」的題：（句子的雜湊, 句子）→（前文, 長前文）；只留雜湊最小的 `limit` 句。
struct ContextItems {
    items: BTreeMap<(u64, String), (String, String)>,
    limit: usize,
}

impl ContextItems {
    fn new(limit: usize) -> Self {
        Self {
            items: BTreeMap::new(),
            limit,
        }
    }

    /// 同一句有幾種前文時取雜湊最小的前文（長前文跟著它），和來源的順序無關。
    fn offer(&mut self, clause: &str, context: &str, long: &str) {
        let items = &mut self.items;
        let entry = (key(clause), clause.to_string());
        if let Some((kept, kept_long)) = items.get_mut(&entry) {
            if (key(context), context) < (key(kept), kept.as_str()) {
                *kept = context.to_string();
                *kept_long = long.to_string();
            }
            return;
        }
        let full = items.len() >= self.limit;
        if full
            && items
                .last_key_value()
                .is_some_and(|(last, _)| entry > *last)
        {
            return;
        }
        items.insert(entry, (context.to_string(), long.to_string()));
        if items.len() > self.limit {
            items.pop_last();
        }
    }
}

/// `text` 的最後 `n` 個字（不足 `n` 字時整串）。
fn last_chars(text: &str, n: usize) -> &str {
    let skip = text.chars().count().saturating_sub(n);
    text.char_indices()
        .nth(skip)
        .map_or("", |(i, _)| &text[i..])
}

/// `text` 最後一個換行或 tab 之後的部分：前文不跨段落，題目檔也就沒有換行與 tab。
fn line_tail(text: &str) -> &str {
    text.rfind(['\n', '\r', '\t'])
        .map_or(text, |i| &text[i + 1..])
}

pub struct Finished<W> {
    pub counts: Counts,
    /// 一行一句：逐字速成碼以空格分隔，tab，句子。
    pub bench: String,
    /// dev 切分的調參題，格式同 `bench`（`cargo xtask bench --dev` 讀）。
    pub dev_items: String,
    /// 「有前文」的題，一行一題：前文，tab，逐字速成碼，tab，句子；依句子的雜湊排序。
    pub context_bench: String,
    /// dev 切分的「有前文」調參題，格式同 `context_bench`（`cargo xtask bench --dev` 讀）。
    pub dev_context_items: String,
    /// `context_bench` 的長前文版：同樣的題與順序，前文是同一段裡前面最多 [`LONG_CONTEXT_CHARS`] 字。
    pub context_long_bench: String,
    /// `dev_context_items` 的長前文版。
    pub dev_context_long_items: String,
    /// 口語長前文的題（[`Builder::add_dialogue`]），格式同 `context_bench`；沒加過對話是空的。
    pub dialogue_items: String,
    /// 口語長前文的 dev 調參題。
    pub dev_dialogue_items: String,
    /// train 的字頻，一行「字<TAB>次數」：次數多的在前，同次數依碼位。
    pub chars: String,
    pub train: W,
    pub dev: W,
}
