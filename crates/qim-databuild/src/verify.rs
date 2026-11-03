//! `cargo xtask data --verify` 的檢查（plan §4.5）；Unihan 交叉比對另由票 06 加。

use std::collections::BTreeSet;
use std::fmt;

use qim_core::Pack;

use crate::libcangjie::Row;
use crate::sources::sha256_hex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckId {
    /// `qim-core` 解得開建出的資料包。
    Decodes,
    /// 連打字集與擴展區的字數都等於來源中有倉頡碼的漢字數。
    NoMissingCharacters,
    /// 每個鍵都是 1 或 2 碼（ADR 0003 的守門）。
    KeyLength,
    /// `anr` 查不到「問」：倉頡碼沒有滲進速成碼表。
    NoCangjieCodeForWen,
    /// 「問」的速成碼含 `ar`。
    WenHasAr,
    /// 「丟」同時有三代的 `hi` 與五代的 `mi`。
    DiuHasHiAndMi,
    /// 同樣的來源建兩次，sha256 相同。
    Deterministic,
}

impl fmt::Display for CheckId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Decodes => "qim-core 解得開資料包",
            Self::NoMissingCharacters => "不掉字",
            Self::KeyLength => "鍵長只有 1 或 2",
            Self::NoCangjieCodeForWen => "抽查：anr 查不到「問」",
            Self::WenHasAr => "抽查：「問」的速成碼含 ar",
            Self::DiuHasHiAndMi => "抽查：「丟」同時有 hi 與 mi",
            Self::Deterministic => "決定性：建兩次 sha256 相同",
        })
    }
}

#[derive(Debug)]
pub struct Check {
    pub id: CheckId,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug)]
pub struct Report {
    /// 資料包裡連打字集的字數。
    pub continuous: usize,
    /// 資料包裡擴展區的字數。
    pub extension: usize,
    /// 資料包裡的速成碼個數。
    pub codes: usize,
    pub sha256: String,
    pub checks: Vec<Check>,
}

impl Report {
    pub fn passed(&self) -> bool {
        self.checks.iter().all(|c| c.passed)
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "連打字集：{} 字", self.continuous)?;
        writeln!(f, "擴展區：{} 字", self.extension)?;
        writeln!(f, "速成碼：{} 個", self.codes)?;
        writeln!(f, "sha256：{}", self.sha256)?;
        for c in &self.checks {
            let mark = if c.passed { "通過" } else { "失敗" };
            writeln!(f, "[{mark}] {}：{}", c.id, c.detail)?;
        }
        write!(f, "verify {}", if self.passed() { "通過" } else { "失敗" })
    }
}

/// `first` 是寫出的資料包，`second` 是由同樣的來源再建一次的結果。
pub fn run(rows: &[Row], first: &[u8], second: &[u8]) -> Report {
    let sha256 = sha256_hex(first);
    let second_sha256 = sha256_hex(second);
    let deterministic = Check {
        id: CheckId::Deterministic,
        passed: sha256 == second_sha256,
        detail: format!("第二次 {second_sha256}"),
    };
    let pack = match Pack::from_bytes(first) {
        Ok(pack) => pack,
        Err(e) => {
            return Report {
                continuous: 0,
                extension: 0,
                codes: 0,
                sha256,
                checks: vec![
                    Check {
                        id: CheckId::Decodes,
                        passed: false,
                        detail: e.to_string(),
                    },
                    deterministic,
                ],
            };
        }
    };

    let mut continuous = BTreeSet::new();
    let mut extension = BTreeSet::new();
    for (_, entries) in pack.code_table.iter() {
        for e in entries {
            if e.extension {
                &mut extension
            } else {
                &mut continuous
            }
            .insert(e.text);
        }
    }
    // 由來源獨立重數，不經推導程式。以字為單位：同一個字的旗標與碼可能分在不同列。
    let hanzi = || rows.iter().filter(|r| r.hanzi);
    let coded: BTreeSet<char> = hanzi()
        .filter(|r| !(r.cangjie3.is_empty() && r.cangjie5.is_empty()))
        .map(|r| r.text)
        .collect();
    let flagged: BTreeSet<char> = hanzi()
        .filter(|r| r.big5 || r.hkscs)
        .map(|r| r.text)
        .collect();
    let source_continuous: BTreeSet<char> = coded.intersection(&flagged).copied().collect();
    let source_extension: BTreeSet<char> = coded.difference(&flagged).copied().collect();

    let long_keys: Vec<&str> = pack
        .code_table
        .iter()
        .map(|(code, _)| code)
        .filter(|code| !(1..=2).contains(&code.chars().count()))
        .collect();
    let has = |code: &str, ch: char| pack.code_table.lookup(code).iter().any(|e| e.text == ch);

    let checks = vec![
        Check {
            id: CheckId::Decodes,
            passed: true,
            detail: format!("格式版本 {}", qim_core::FORMAT_VERSION),
        },
        Check {
            id: CheckId::NoMissingCharacters,
            passed: continuous == source_continuous && extension == source_extension,
            detail: format!(
                "連打字集 {} / 來源 {}，擴展區 {} / 來源 {}",
                continuous.len(),
                source_continuous.len(),
                extension.len(),
                source_extension.len()
            ),
        },
        Check {
            id: CheckId::KeyLength,
            passed: long_keys.is_empty(),
            detail: if long_keys.is_empty() {
                "沒有長度 3 以上的鍵".into()
            } else {
                format!("不合的鍵：{}", long_keys.join(" "))
            },
        },
        spot(CheckId::NoCangjieCodeForWen, !has("anr", '問')),
        spot(CheckId::WenHasAr, has("ar", '問')),
        spot(CheckId::DiuHasHiAndMi, has("hi", '丟') && has("mi", '丟')),
        deterministic,
    ];
    Report {
        continuous: continuous.len(),
        extension: extension.len(),
        codes: pack.code_table.iter().count(),
        sha256,
        checks,
    }
}

fn spot(id: CheckId, passed: bool) -> Check {
    Check {
        id,
        passed,
        detail: if passed { "符合" } else { "不符" }.into(),
    }
}
