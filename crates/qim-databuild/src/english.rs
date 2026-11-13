//! ESDB（SCOWL）的 Hunspell 字典 → 英文資料包的詞表（plan〈英文、簡繁與 emoji〉）。
//!
//! 取 `en-wl/wordlist` release 的 `hunspell-en_US-<版本>.zip`：官方的 size 60、美式、預設變體
//! 詞表（README 的 `mk-list --accents=strip en_US 60`）。`en_US.dic` 一行一個詞幹與旗標，
//! 照 `en_US.aff` 的字首、字尾規則展開成全部詞形。只支援這份 `.aff` 用到的寫法：單字元旗標、
//! `PFX` / `SFX` 的 strip、add、條件（字元、`.`、`[...]`、`[^...]`），沒有接續旗標；
//! 會改變有哪些詞的其他指令（[`UNSUPPORTED`]）讓建置失敗，不默默略過。只影響拼字檢查與
//! 建議的指令（`TRY`、`REP`…）照略過。

use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;

use qim_core::EnglishWords;

use crate::Error;

/// release zip 裡的字典與規則檔。
pub const DIC: &str = "en_US.dic";
pub const AFF: &str = "en_US.aff";

/// 會改變有哪些詞、這裡卻沒實作的 `.aff` 指令：出現就讓建置失敗。
pub const UNSUPPORTED: [&str; 8] = [
    "FLAG",
    "COMPLEXPREFIXES",
    "NEEDAFFIX",
    "CIRCUMFIX",
    "FORBIDDENWORD",
    "AF",
    "IGNORE",
    "FULLSTRIP",
];

/// `.aff` 的字首、字尾規則與特殊旗標。
#[derive(Debug, Clone, Default)]
pub struct Affixes {
    prefixes: BTreeMap<char, Affix>,
    suffixes: BTreeMap<char, Affix>,
    /// 禁忌詞（README：最強烈的禁忌詞標 NOSUGGEST）；拼字檢查不建議它，英文候選也不出。
    nosuggest: Option<char>,
    /// 只能在複合詞裡出現（序數 `1th` 之類）。
    only_in_compound: Option<char>,
    /// `COMPOUNDRULE` 用到的旗標：只組數字（`21st`），不展開。
    compound: BTreeSet<char>,
}

#[derive(Debug, Clone)]
struct Affix {
    /// 字首與字尾都可交叉時，兩者可以疊在同一個詞上。
    cross_product: bool,
    rules: Vec<Rule>,
}

#[derive(Debug, Clone)]
struct Rule {
    strip: String,
    add: String,
    condition: Vec<Class>,
}

/// 條件的一個位置。
#[derive(Debug, Clone)]
enum Class {
    Any,
    Set { negated: bool, chars: Vec<char> },
}

impl Class {
    fn matches(&self, c: char) -> bool {
        match self {
            Self::Any => true,
            Self::Set { negated, chars } => chars.contains(&c) != *negated,
        }
    }
}

impl Affixes {
    fn table_mut(&mut self, prefix: bool) -> &mut BTreeMap<char, Affix> {
        if prefix {
            &mut self.prefixes
        } else {
            &mut self.suffixes
        }
    }

    /// 讀 `.aff`：只取 `PFX`、`SFX`、`NOSUGGEST`、`ONLYINCOMPOUND`、`COMPOUNDRULE` 的旗標，
    /// 其他指令（`TRY`、`REP`…）只影響拼字檢查與建議，不影響有哪些詞。
    pub fn parse(text: &str) -> Result<Self, Error> {
        let mut affixes = Self::default();
        // 正在讀的規則組：（是不是字首, 旗標, 宣告的規則數）。
        let mut open: Option<(bool, char, usize)> = None;
        for (i, line) in text.lines().enumerate() {
            let at = || format!("{AFF} 第 {} 列", i + 1);
            let fields: Vec<&str> = line.split_whitespace().collect();
            match fields[..] {
                ["NOSUGGEST", flag] => affixes.nosuggest = Some(single_flag(flag, &at)?),
                ["ONLYINCOMPOUND", flag] => {
                    affixes.only_in_compound = Some(single_flag(flag, &at)?);
                }
                // 第一個 `COMPOUNDRULE` 只寫規則數。
                ["COMPOUNDRULE", rule] if rule.parse::<usize>().is_err() => affixes
                    .compound
                    .extend(rule.chars().filter(|c| !matches!(c, '*' | '?' | '(' | ')'))),
                [kind @ ("PFX" | "SFX"), flag, cross, count]
                    if open.is_none_or(|(_, f, _)| f.to_string() != flag) =>
                {
                    close(&affixes, open, &at)?;
                    let flag = single_flag(flag, &at)?;
                    let count = count
                        .parse()
                        .map_err(|_| Error::invalid(at(), "規則數不是數字"))?;
                    let prefix = kind == "PFX";
                    let affix = Affix {
                        cross_product: match cross {
                            "Y" => true,
                            "N" => false,
                            _ => return Err(Error::invalid(at(), "交叉欄要是 Y 或 N")),
                        },
                        rules: Vec::new(),
                    };
                    if affixes.table_mut(prefix).insert(flag, affix).is_some() {
                        return Err(Error::invalid(at(), format!("旗標 {flag} 重複")));
                    }
                    open = Some((prefix, flag, count));
                }
                [kind @ ("PFX" | "SFX"), flag, strip, add, condition] => {
                    let Some((prefix, open_flag, _)) = open else {
                        return Err(Error::invalid(at(), "規則前沒有規則組的宣告"));
                    };
                    if (kind == "PFX") != prefix || open_flag.to_string() != flag {
                        return Err(Error::invalid(at(), "規則不屬於正在讀的規則組"));
                    }
                    if add.contains('/') {
                        return Err(Error::invalid(at(), "不支援接續旗標"));
                    }
                    let rule = Rule {
                        strip: none_if_zero(strip),
                        add: none_if_zero(add),
                        condition: parse_condition(condition)
                            .map_err(|r| Error::invalid(at(), r))?,
                    };
                    affixes
                        .table_mut(prefix)
                        .get_mut(&open_flag)
                        .expect("宣告時插入過")
                        .rules
                        .push(rule);
                }
                [kind @ ("PFX" | "SFX"), ..] => {
                    return Err(Error::invalid(at(), format!("{kind} 的欄位數不對")));
                }
                [directive, ..] if UNSUPPORTED.contains(&directive) => {
                    return Err(Error::invalid(at(), format!("不支援 {directive}")));
                }
                _ => {}
            }
        }
        close(&affixes, open, &|| format!("{AFF} 結尾"))?;
        Ok(affixes)
    }
}

/// 確認剛讀完的規則組有宣告的規則數。
fn close(
    affixes: &Affixes,
    open: Option<(bool, char, usize)>,
    at: &dyn Fn() -> String,
) -> Result<(), Error> {
    let Some((prefix, flag, count)) = open else {
        return Ok(());
    };
    let table = if prefix {
        &affixes.prefixes
    } else {
        &affixes.suffixes
    };
    let found = table[&flag].rules.len();
    if found == count {
        Ok(())
    } else {
        Err(Error::invalid(
            at(),
            format!("旗標 {flag} 宣告 {count} 條規則，讀到 {found} 條"),
        ))
    }
}

fn single_flag(flag: &str, at: &dyn Fn() -> String) -> Result<char, Error> {
    let mut chars = flag.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => Ok(c),
        _ => Err(Error::invalid(at(), format!("旗標 {flag} 不是單一字元"))),
    }
}

fn none_if_zero(field: &str) -> String {
    if field == "0" {
        String::new()
    } else {
        field.to_string()
    }
}

fn parse_condition(text: &str) -> Result<Vec<Class>, String> {
    let mut classes = Vec::new();
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        classes.push(match c {
            '.' => Class::Any,
            '[' => {
                let mut set: Vec<char> = Vec::new();
                loop {
                    match chars.next() {
                        Some(']') => break,
                        Some(c) => set.push(c),
                        None => return Err(format!("條件 {text} 的 [ 沒有關上")),
                    }
                }
                let negated = set.first() == Some(&'^');
                if negated {
                    set.remove(0);
                }
                Class::Set {
                    negated,
                    chars: set,
                }
            }
            c => Class::Set {
                negated: false,
                chars: vec![c],
            },
        });
    }
    Ok(classes)
}

impl Rule {
    /// 字尾規則套在 `word` 上：結尾符合條件、也有要去掉的部分時，換上 `add`。
    fn suffix(&self, word: &str) -> Option<String> {
        let chars: Vec<char> = word.chars().collect();
        let n = self.condition.len();
        if chars.len() < n || !word.ends_with(&self.strip) || word.len() == self.strip.len() {
            return None;
        }
        let tail = &chars[chars.len() - n..];
        tail.iter()
            .zip(&self.condition)
            .all(|(&c, class)| class.matches(c))
            .then(|| format!("{}{}", &word[..word.len() - self.strip.len()], self.add))
    }

    /// 字首規則套在 `word` 上。
    fn prefix(&self, word: &str) -> Option<String> {
        if word.chars().count() < self.condition.len()
            || !word.starts_with(&self.strip)
            || word.len() == self.strip.len()
        {
            return None;
        }
        word.chars()
            .zip(&self.condition)
            .all(|(c, class)| class.matches(c))
            .then(|| format!("{}{}", self.add, &word[self.strip.len()..]))
    }
}

/// 展開的結果。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Expansion {
    pub words: BTreeSet<String>,
    /// `.dic` 的詞幹數（第一行的數字不算）。
    pub stems: usize,
    /// 標了禁忌旗標而整個不收的詞幹數。
    pub taboo: usize,
}

/// 照 `affixes` 展開 `.dic` 的每個詞幹：詞幹本身、每個字尾、每個字首，兩者都可交叉時再疊起來。
pub fn expand(affixes: &Affixes, dic: &str) -> Result<Expansion, Error> {
    let mut expansion = Expansion::default();
    for (i, line) in dic.lines().enumerate().skip(1) {
        let entry = line.split('\t').next().unwrap_or_default().trim();
        if entry.is_empty() {
            continue;
        }
        expansion.stems += 1;
        let (stem, flags) = entry.split_once('/').unwrap_or((entry, ""));
        if affixes.nosuggest.is_some_and(|f| flags.contains(f)) {
            expansion.taboo += 1;
            continue;
        }
        if affixes.only_in_compound.is_some_and(|f| flags.contains(f)) {
            continue;
        }
        let mut suffixed = vec![(stem.to_string(), true)];
        let mut prefixes = Vec::new();
        for flag in flags.chars() {
            if Some(flag) == affixes.nosuggest
                || Some(flag) == affixes.only_in_compound
                || affixes.compound.contains(&flag)
            {
                continue;
            }
            if let Some(affix) = affixes.suffixes.get(&flag) {
                for rule in &affix.rules {
                    suffixed.extend(rule.suffix(stem).map(|w| (w, affix.cross_product)));
                }
            } else if let Some(affix) = affixes.prefixes.get(&flag) {
                prefixes.push(affix);
            } else {
                return Err(Error::invalid(
                    format!("{DIC} 第 {} 列", i + 1),
                    format!("{AFF} 沒有旗標 {flag}"),
                ));
            }
        }
        for affix in prefixes {
            for (word, cross) in &suffixed.clone() {
                // 詞幹本身（cross 為真）一定接得上字首；字尾形只在兩邊都可交叉時。
                if *word != stem && !(affix.cross_product && *cross) {
                    continue;
                }
                for rule in &affix.rules {
                    suffixed.extend(rule.prefix(word).map(|w| (w, false)));
                }
            }
        }
        expansion.words.extend(suffixed.into_iter().map(|(w, _)| w));
    }
    Ok(expansion)
}

/// 只留轉成小寫後每個字母都在 `alphabet` 的詞（速成打不出 `z`、撇號、重音），交給
/// [`EnglishWords::new`] 依小寫歸併大小寫；另回傳打不出而不收的詞數。
pub fn typable(words: &BTreeSet<String>, alphabet: &str) -> (EnglishWords, usize) {
    let (kept, dropped): (Vec<&String>, Vec<&String>) = words.iter().partition(|w| {
        w.chars()
            .flat_map(char::to_lowercase)
            .all(|c| alphabet.contains(c))
    });
    (EnglishWords::new(kept.into_iter().cloned()), dropped.len())
}

/// 英文詞表的建置統計。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    pub stems: usize,
    pub taboo: usize,
    /// 展開後的詞形數。
    pub forms: usize,
    pub untypable: usize,
    /// 依小寫歸併大小寫後收的詞數。
    pub words: usize,
}

/// 由 release zip 建詞表。
pub fn words(zip: &[u8], alphabet: &str) -> Result<(EnglishWords, Counts), Error> {
    let what = "Hunspell en_US zip";
    let invalid = |e: &dyn std::fmt::Display| Error::invalid(what, e.to_string());
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip)).map_err(|e| invalid(&e))?;
    let mut read = |name: &str| -> Result<String, Error> {
        let mut text = String::new();
        archive
            .by_name(name)
            .map_err(|e| invalid(&e))?
            .read_to_string(&mut text)
            .map_err(|e| Error::invalid(format!("{what} 的 {name}"), e.to_string()))?;
        Ok(text)
    };
    let affixes = Affixes::parse(&read(AFF)?)?;
    let expansion = expand(&affixes, &read(DIC)?)?;
    let (words, untypable) = typable(&expansion.words, alphabet);
    let counts = Counts {
        stems: expansion.stems,
        taboo: expansion.taboo,
        forms: expansion.words.len(),
        untypable,
        words: words.len(),
    };
    Ok((words, counts))
}
