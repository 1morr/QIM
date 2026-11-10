//! 詞表匯入（plan §2.4，M3 票 17）：把別的輸入法匯出、或使用者自己整理的詞表加進使用者詞。
//!
//! 收三種格式，照各自產品的慣例讀：Rime 碼表（`.dict.yaml`：YAML 檔頭，`...` 之後是 Tab 分隔的
//! 列，欄位依 `columns:`、沒寫時第一欄是字詞）、Gboard 個人字典匯出的 `dictionary.txt`（第一行
//! `# Gboard Dictionary …`，每列「快捷鍵<Tab>詞<Tab>語言」）、一行一詞的純文字。檔裡的碼不看
//! （多半是別的方案的），碼照手動加詞由字推出；Gboard 的快捷鍵預設不設成觸發詞（[`Shortcuts`]）。
//!
//! 和匯入學習資料（[`UserModel::import`]）不同，這裡是加進現有的使用者詞：壞列略過、報行號，
//! 其餘照加（librime、libime 讀使用者詞庫也是略過壞列，`docs/research/userdict-text-formats.md`）。

use std::collections::HashMap;
use std::fmt;

use crate::user::manual_word_code;
use crate::{Engine, UserModel};

/// 詞表的格式，由檔的開頭判斷。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordListFormat {
    /// Rime 碼表 `.dict.yaml`：開頭的註解之後是 `---`。
    Rime,
    /// Gboard 個人字典的 `dictionary.txt`：第一行以 `# Gboard Dictionary` 開頭。
    Gboard,
    /// 其餘：一行一詞，`#` 開頭的行是註解，Tab 之後的欄不看。
    PlainText,
}

impl fmt::Display for WordListFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Rime => "Rime 碼表（.dict.yaml）",
            Self::Gboard => "Gboard 個人字典",
            Self::PlainText => "純文字（一行一詞）",
        })
    }
}

/// Gboard 的快捷鍵要不要設成觸發詞。快捷鍵是在別的鍵盤（英文、拼音）設的，同一串字母在速成
/// 多半是別的字詞的碼，設了會搶走那個碼的首選；所以預設不設，由使用者在匯入前決定。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Shortcuts {
    /// 不設：詞照加，碼由字推出；有字沒有碼的詞推不出碼。
    #[default]
    Ignore,
    /// 是方案的鍵的快捷鍵設成觸發詞（有字沒有碼的詞也用它當碼）。
    AsTriggers,
}

/// 一次匯入的結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordListReport {
    pub format: WordListFormat,
    /// 加進使用者詞的詞數。
    pub added: usize,
    /// 沒加的列，依行號。
    pub skipped: Vec<SkippedLine>,
    /// 快捷鍵是方案的鍵、能設成觸發詞的列（只有 Gboard 有快捷鍵）；不論設了沒有都列出，殼拿它
    /// 問使用者要不要設。詞已是使用者詞的不列（設不設都不會加）。
    pub trigger_shortcuts: Vec<TriggerShortcut>,
    /// 加了、但快捷鍵不是方案的鍵而不能當觸發詞的詞。
    pub unused_shortcuts: Vec<UnusedShortcut>,
}

/// 快捷鍵能設成觸發詞的一列。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerShortcut {
    pub line: usize,
    pub text: String,
    pub shortcut: String,
    /// 打這串快捷鍵時現在的首選（不是這個詞時）；設成觸發詞就會搶走它的首選。
    pub displaces: Option<String>,
}

impl fmt::Display for TriggerShortcut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "第 {} 行「{}」：快捷鍵「{}」",
            self.line, self.text, self.shortcut
        )?;
        if let Some(top) = &self.displaces {
            write!(f, "，會搶走「{top}」的首選")?;
        }
        Ok(())
    }
}

/// 沒加的一列。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedLine {
    /// 第幾行，從 1 起。
    pub line: usize,
    /// 那列的詞；讀不出詞時是整列。
    pub text: String,
    pub reason: SkipReason,
}

/// 詞加了、Gboard 的快捷鍵不是方案的鍵而不能當觸發詞的一列。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnusedShortcut {
    pub line: usize,
    pub text: String,
    pub shortcut: String,
}

impl fmt::Display for SkippedLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "第 {} 行「{}」：{}", self.line, self.text, self.reason)
    }
}

impl fmt::Display for UnusedShortcut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "第 {} 行「{}」：快捷鍵「{}」不是方案的鍵，不能設成觸發詞",
            self.line, self.text, self.shortcut
        )
    }
}

/// 一列沒加的原因。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// 讀不出詞。
    Malformed(&'static str),
    /// 詞含 Tab 以外的控制字元（手動加詞也不收）。
    ControlCharacter,
    /// 有字沒有碼、又沒有可當觸發詞的快捷鍵，打不出來。
    NoCode,
    /// 同一份檔前面加過同樣的詞（第二個快捷鍵也不收，觸發詞照第一列）。
    Duplicate { first_line: usize },
    /// 已是這個方案的使用者詞；原本的（含觸發詞）不動。
    AlreadyUserWord,
    /// 碼表或詞庫已有同碼的這個詞，加了也一樣打得出；有觸發詞時照加。
    InLexicon,
}

impl fmt::Display for SkipReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(reason) => f.write_str(reason),
            Self::ControlCharacter => f.write_str("含控制字元"),
            Self::NoCode => f.write_str("有字沒有碼，打不出來"),
            Self::Duplicate { first_line } => write!(f, "和第 {first_line} 行重複"),
            Self::AlreadyUserWord => f.write_str("已是使用者詞"),
            Self::InLexicon => f.write_str("詞庫已有"),
        }
    }
}

/// 整份讀不了的詞表：第幾行（從 1 起）、為什麼。這時什麼都不加。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordListError {
    pub line: usize,
    pub reason: String,
}

impl fmt::Display for WordListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "第 {} 行：{}", self.line, self.reason)
    }
}

impl std::error::Error for WordListError {}

/// 檔裡的一行：行號（從 1 起）與內容（去掉 `\r`）。
type Line<'a> = (usize, &'a str);

/// 從檔裡讀出的一列。
enum Row<'a> {
    Word {
        line: usize,
        text: &'a str,
        /// Gboard 的快捷鍵；空的是 `None`。
        shortcut: Option<&'a str>,
    },
    Bad {
        line: usize,
        text: &'a str,
        reason: &'static str,
    },
}

const GBOARD_HEADER: &str = "# Gboard Dictionary";

impl UserModel {
    /// 把詞表 `text` 的詞加進目前方案的使用者詞（plan §2.4）：碼照 [`UserModel::add_word`] 由字
    /// 推出，來源是 `manual`；Gboard 的快捷鍵照 `shortcuts` 決定設不設成觸發詞。壞列、重複、已有的
    /// 略過並記在結果裡；整份讀不了時回傳錯誤、什麼都不加。要先看會加幾個，在複本上呼叫。
    pub fn import_word_list(
        &mut self,
        engine: &Engine,
        text: &str,
        shortcuts: Shortcuts,
    ) -> Result<WordListReport, WordListError> {
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let lines: Vec<Line> = text
            .lines()
            .map(|l| l.strip_suffix('\r').unwrap_or(l))
            .zip(1..)
            .map(|(l, n)| (n, l))
            .collect();
        let (format, rows) = parse(&lines)?;
        let mut report = WordListReport {
            format,
            added: 0,
            skipped: Vec::new(),
            trigger_shortcuts: Vec::new(),
            unused_shortcuts: Vec::new(),
        };
        // 加進去的詞 → 它的行號；只記加了的，重複才指得到一列真的加了的。
        let mut added: HashMap<&str, usize> = HashMap::new();
        for row in rows {
            let (line, word, shortcut) = match row {
                Row::Word {
                    line,
                    text,
                    shortcut,
                } => (line, text, shortcut),
                Row::Bad { line, text, reason } => {
                    report.skip(line, text, SkipReason::Malformed(reason));
                    continue;
                }
            };
            if let Some(&first_line) = added.get(word) {
                report.skip(line, word, SkipReason::Duplicate { first_line });
                continue;
            }
            if word.chars().any(char::is_control) {
                report.skip(line, word, SkipReason::ControlCharacter);
                continue;
            }
            let usable = shortcut.filter(|s| engine.scheme().spells(s));
            // 只列設了才有差的：用快捷鍵時的碼（字推不出就是快捷鍵）已有這個使用者詞的不列。
            let offered = usable.filter(|&s| {
                manual_word_code(engine, word, Some(s))
                    .is_some_and(|code| !self.for_scheme(&engine.scheme().id).is_word(&code, word))
            });
            if let Some(usable) = offered {
                let displaces = engine
                    .candidates(self, usable)
                    .into_iter()
                    .next()
                    .filter(|top| top != word);
                report.trigger_shortcuts.push(TriggerShortcut {
                    line,
                    text: word.to_string(),
                    shortcut: usable.to_string(),
                    displaces,
                });
            }
            let trigger = usable.filter(|_| shortcuts == Shortcuts::AsTriggers);
            let Some(code) = manual_word_code(engine, word, trigger) else {
                report.skip(line, word, SkipReason::NoCode);
                continue;
            };
            if self.for_scheme(&engine.scheme().id).is_word(&code, word) {
                report.skip(line, word, SkipReason::AlreadyUserWord);
                continue;
            }
            if trigger.is_none() && engine.has_entry(&code, word) {
                report.skip(line, word, SkipReason::InLexicon);
                continue;
            }
            self.insert_manual_word(engine, &code, word, trigger);
            added.insert(word, line);
            report.added += 1;
            if let (Some(shortcut), None) = (shortcut, usable) {
                report.unused_shortcuts.push(UnusedShortcut {
                    line,
                    text: word.to_string(),
                    shortcut: shortcut.to_string(),
                });
            }
        }
        Ok(report)
    }
}

impl WordListReport {
    fn skip(&mut self, line: usize, text: &str, reason: SkipReason) {
        self.skipped.push(SkippedLine {
            line,
            text: text.to_string(),
            reason,
        });
    }
}

/// 判斷格式並讀出各列；空行與註解不算列。
fn parse<'a>(lines: &[Line<'a>]) -> Result<(WordListFormat, Vec<Row<'a>>), WordListError> {
    let is_comment = |l: &str| l.trim().is_empty() || l.trim_start().starts_with('#');
    if lines
        .first()
        .is_some_and(|(_, l)| l.starts_with(GBOARD_HEADER))
    {
        let rows = lines[1..]
            .iter()
            .filter(|(_, l)| !is_comment(l))
            .map(|&(line, l)| gboard_row(line, l))
            .collect();
        return Ok((WordListFormat::Gboard, rows));
    }
    if let Some(start) = lines.iter().position(|(_, l)| !is_comment(l))
        && lines[start].1.trim_end() == "---"
    {
        let (column, body) = rime_header(&lines[start..])?;
        let rows = body
            .iter()
            .filter(|(_, l)| !is_comment(l))
            .map(|&(line, l)| rime_row(line, l, column))
            .collect();
        return Ok((WordListFormat::Rime, rows));
    }
    let rows = lines
        .iter()
        .filter(|(_, l)| !is_comment(l))
        .map(
            |&(line, l)| match trim_word(l.split('\t').next().unwrap_or(l)) {
                "" => Row::Bad {
                    line,
                    text: l.trim(),
                    reason: "詞是空的",
                },
                text => Row::Word {
                    line,
                    text,
                    shortcut: None,
                },
            },
        )
        .collect();
    Ok((WordListFormat::PlainText, rows))
}

/// 詞前後的空白去掉；同時是控制字元的（`\u{b}`、`\u{c}`…）留著，好報成控制字元。
fn trim_word(field: &str) -> &str {
    field.trim_matches(|c: char| c.is_whitespace() && !c.is_control())
}

/// Gboard 的一列：快捷鍵、詞、語言（之後可能還有詞性），Tab 分隔；快捷鍵可以是空的。
fn gboard_row(line: usize, l: &str) -> Row<'_> {
    let mut fields = l.split('\t');
    match (fields.next(), fields.next().map(trim_word)) {
        (Some(shortcut), Some(text)) if !text.is_empty() => Row::Word {
            line,
            text,
            shortcut: Some(shortcut.trim()).filter(|s| !s.is_empty()),
        },
        _ => Row::Bad {
            line,
            text: l.trim(),
            reason: "Gboard 的列要是「快捷鍵<Tab>詞<Tab>語言」",
        },
    }
}

/// Rime 碼表的一列：Tab 分隔，字詞在第 `column` 欄。
fn rime_row(line: usize, l: &str, column: usize) -> Row<'_> {
    match l.split('\t').nth(column).map(trim_word) {
        Some(text) if !text.is_empty() => Row::Word {
            line,
            text,
            shortcut: None,
        },
        Some(_) => Row::Bad {
            line,
            text: l.trim(),
            reason: "詞是空的",
        },
        None => Row::Bad {
            line,
            text: l.trim(),
            reason: "少了 columns 裡的 text 欄",
        },
    }
}

/// Rime 的 YAML 檔頭：`lines` 從 `---` 起，到 `...` 止。回傳字詞是第幾欄（`columns:` 裡 `text`
/// 的位置，沒寫 `columns:` 時是第一欄）與檔頭之後的行。只讀 `columns:`，其餘鍵（`name`、
/// `import_tables`…）不看；`columns:` 收區塊清單（`- text` 一行一個）與 `[code, text]`。
fn rime_header<'l, 'a>(lines: &'l [Line<'a>]) -> Result<(usize, &'l [Line<'a>]), WordListError> {
    let error = |line: usize, reason: &str| WordListError {
        line,
        reason: reason.to_string(),
    };
    let start = lines[0].0;
    let end = lines
        .iter()
        .position(|(_, l)| l.trim_end() == "...")
        .ok_or_else(|| error(start, "Rime 的檔頭要以 `...` 一行結束"))?;
    let header = &lines[1..end];
    let without_comment = |l: &'a str| l.split('#').next().unwrap_or("").trim();
    let Some(at) = header.iter().position(|(_, l)| l.starts_with("columns:")) else {
        return Ok((0, &lines[end + 1..]));
    };
    let (columns_line, l) = header[at];
    let rest = without_comment(&l["columns:".len()..]);
    let columns: Vec<&str> = if let Some(flow) = rest.strip_prefix('[') {
        flow.trim_end_matches(']')
            .split(',')
            .map(str::trim)
            .collect()
    } else {
        header[at + 1..]
            .iter()
            .map(|&(_, l)| without_comment(l))
            .filter(|l| !l.is_empty())
            .map_while(|l| l.strip_prefix('-').map(str::trim))
            .collect()
    };
    let column = columns
        .iter()
        .map(|c| c.trim_matches(|q| q == '"' || q == '\''))
        .position(|c| c == "text")
        .ok_or_else(|| error(columns_line, "columns 裡沒有 text 欄，讀不出詞"))?;
    Ok((column, &lines[end + 1..]))
}
