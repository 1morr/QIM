//! 字形用常見寫法（brief §4.1、plan §4.3）：語料轉成香港繁體之後，把香港寫法與 OpenCC 標準寫法
//! 換回常見寫法（「説」→「說」、「温」→「溫」、「羣」→「群」）。使用者打常見寫法（M1 票 17b）。
//!
//! 對照表用轉語料的同一套 OpenCC（ferrous-opencc 內建的字典）逐字查碼表裡的每個字：`t2hk` 把
//! 字換成香港寫法（反過來就是香港 → 常見），`t2tw` 把 OpenCC 標準寫法換成台灣常見寫法（`s2hk` 的
//! 簡轉繁出的是前者，例如「羣」）。只收常見寫法在連打字集、而且被換掉的字不在連打字集（不換回來，
//! 含它的句子會整句丟掉）或兩字同碼（打的時候互搶）的組；不同碼的「户／戶」「台／臺」不換，免得
//! 把「吃」換成「喫」、「才」換成「纔」這類罕用寫法。

use std::collections::{BTreeMap, BTreeSet};

use ferrous_opencc::OpenCC;
use ferrous_opencc::config::BuiltinConfig;
use qim_core::CodeTable;

use crate::Error;

/// 被換掉的字（香港寫法或 OpenCC 標準寫法）→ 常見寫法。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommonForms(BTreeMap<char, char>);

impl CommonForms {
    /// 用 ferrous-opencc 內建的 `t2hk`、`t2tw` 建表，和 `corpora::Converter` 同一套字典。
    pub fn from_opencc(table: &CodeTable) -> Result<Self, Error> {
        let open = |config, name: &str| {
            OpenCC::from_config(config).map_err(|e| Error::invalid(name, e.to_string()))
        };
        let (hong_kong, taiwan) = (
            open(BuiltinConfig::T2hk, "OpenCC t2hk")?,
            open(BuiltinConfig::T2tw, "OpenCC t2tw")?,
        );
        Self::new(table, one_char(&hong_kong), one_char(&taiwan))
    }

    /// `to_hong_kong`、`to_taiwan` 把一個字換成香港寫法、台灣常見寫法，沒有就回原字；
    /// 查的是 `table`（速成碼表）裡的每個字，連打字集與同碼也由它判斷。
    /// 同一個字要換成兩種寫法，或換完的字又在表裡（一次換不完），都讓建置失敗。
    pub fn new(
        table: &CodeTable,
        to_hong_kong: impl Fn(char) -> char,
        to_taiwan: impl Fn(char) -> char,
    ) -> Result<Self, Error> {
        let mut codes: BTreeMap<char, BTreeSet<&str>> = BTreeMap::new();
        let mut all = BTreeSet::new();
        for (code, entries) in table.iter() {
            for entry in entries {
                all.insert(entry.text);
                if !entry.extension {
                    codes.entry(entry.text).or_default().insert(code);
                }
            }
        }
        let continuous = |c: char| codes.contains_key(&c);
        let same_code = |a: char, b: char| match (codes.get(&a), codes.get(&b)) {
            (Some(a), Some(b)) => !a.is_disjoint(b),
            _ => false,
        };
        // (被換掉的字, 常見寫法)：香港寫法換回原字；OpenCC 標準寫法換成台灣寫法。
        let candidates = all
            .iter()
            .map(|&c| (to_hong_kong(c), c))
            .chain(all.iter().map(|&c| (c, to_taiwan(c))));
        let mut forms = BTreeMap::new();
        for (from, to) in candidates {
            if from == to || !continuous(to) || (continuous(from) && !same_code(from, to)) {
                continue;
            }
            if let Some(other) = forms.insert(from, to).filter(|&other| other != to) {
                return Err(Error::invalid(
                    "字形對照表",
                    format!("「{from}」同時要換成「{other}」與「{to}」"),
                ));
            }
        }
        if let Some((from, to)) = forms.iter().find(|(_, to)| forms.contains_key(to)) {
            return Err(Error::invalid(
                "字形對照表",
                format!("「{from}」換成「{to}」後還要再換，一次換不完"),
            ));
        }
        Ok(Self(forms))
    }

    /// 每一組（被換掉的字, 常見寫法），依被換掉的字的碼位。
    pub fn pairs(&self) -> impl Iterator<Item = (char, char)> + '_ {
        self.0.iter().map(|(&from, &to)| (from, to))
    }

    pub fn apply(&self, text: &str) -> String {
        text.chars()
            .map(|c| self.0.get(&c).copied().unwrap_or(c))
            .collect()
    }
}

/// OpenCC 換一個字；換出來不是一個字（字典裡沒有這種列）時當作不換。
fn one_char(opencc: &OpenCC) -> impl Fn(char) -> char + '_ {
    move |c| {
        let out = opencc.convert(c.encode_utf8(&mut [0; 4]));
        let mut chars = out.chars();
        match (chars.next(), chars.next()) {
            (Some(one), None) => one,
            _ => c,
        }
    }
}
