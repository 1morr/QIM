//! `cargo xtask data --verify` 的檢查（plan §4.5）。Unihan 交叉比對不是檢查、不影響通過與否，
//! 由 `pipeline` 算好放進 [`Report::unihan`]，比對本身在 `unihan` 模組。

use std::collections::{BTreeSet, HashSet};
use std::fmt;
use std::io::{self, BufRead};
use std::path::PathBuf;

use qim_core::Pack;

use crate::corpus::Counts;
use crate::libcangjie::Row;
use crate::sources::sha256_hex;
use crate::transcode::Transcoder;

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
    /// `a` 的首選是「日」（字序依 Unihan `kGradeLevel`，plan §4.1）。
    RiIsTopForA,
    /// 同樣的來源建兩次，sha256 相同。
    Deterministic,
    /// 語料的基準句與訓練切分每句都轉得了碼，基準句的碼與轉碼結果相同。
    CorpusTranscodes,
    /// 基準句沒有一句原樣出現在 train（plan §5.2 的防洩漏）。
    CorpusNoLeak,
    /// 同樣的來源建兩次，基準句與訓練切分的 sha256 相同。
    CorpusDeterministic,
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
            Self::RiIsTopForA => "抽查：a 的首選是「日」",
            Self::Deterministic => "決定性：建兩次 sha256 相同",
            Self::CorpusTranscodes => "語料轉碼",
            Self::CorpusNoLeak => "語料不洩漏",
            Self::CorpusDeterministic => "語料決定性",
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
    /// Unihan 交叉比對的摘要；[`run`] 不算，由呼叫端填。
    pub unihan: Option<UnihanSummary>,
    /// 各語料的句數；[`run`] 不算，由呼叫端填，檢查用 [`corpus`] 算好加進 `checks`。
    pub corpora: Vec<CorpusSummary>,
}

/// 一個語料建出的句數與基準檔。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpusSummary {
    /// 基準檔與訓練切分的名字，例如 `yue-wiki`。
    pub name: String,
    pub counts: Counts,
    /// 進基準檔的句數。
    pub bench_sentences: usize,
    /// 基準檔，相對 repo 根目錄。
    pub bench: PathBuf,
}

impl fmt::Display for CorpusSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = &self.counts;
        write!(
            f,
            "語料 {}：train {} 句、dev {} 句、test {} 句（不重複），其中 {} 句進 {}；含字集外字元丟掉 {} 句",
            self.name,
            c.train,
            c.dev,
            c.test,
            self.bench_sentences,
            self.bench.display(),
            c.dropped
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnihanSummary {
    /// 兩邊都有碼的字數。
    pub compared: usize,
    /// 碼不同的字數。
    pub differences: usize,
    /// 差異中 Unihan 的碼等於 libcangjie 五代碼的字數。
    pub matching_cangjie5: usize,
    /// 差異報表，相對 repo 根目錄。
    pub report: PathBuf,
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
        if let Some(u) = &self.unihan {
            writeln!(
                f,
                "Unihan kCangjie：比對 {} 字，差異 {} 字（其中 {} 字等於五代碼），報表 {}（不影響 verify 結果）",
                u.compared,
                u.differences,
                u.matching_cangjie5,
                u.report.display()
            )?;
        }
        for corpus in &self.corpora {
            writeln!(f, "{corpus}")?;
        }
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
                unihan: None,
                corpora: Vec::new(),
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
        spot(
            CheckId::RiIsTopForA,
            pack.code_table.lookup("a").first().map(|e| e.text) == Some('日'),
        ),
        deterministic,
    ];
    Report {
        continuous: continuous.len(),
        extension: extension.len(),
        codes: pack.code_table.iter().count(),
        sha256,
        checks,
        unihan: None,
        corpora: Vec::new(),
    }
}

/// 一次語料建置寫出的三個檔的 sha256。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpusDigests {
    pub bench: String,
    pub train: String,
    pub dev: String,
}

/// 一個語料的檢查。`bench` 是基準檔的內容（一行「逐字速成碼<TAB>句子」），`train`、`dev` 一行一句；
/// `first`、`second` 是兩次建置的 sha256。
pub fn corpus(
    name: &str,
    transcoder: &Transcoder,
    bench: &str,
    train: impl BufRead,
    dev: impl BufRead,
    first: &CorpusDigests,
    second: &CorpusDigests,
) -> io::Result<Vec<Check>> {
    let mut failures = 0;
    let mut sentences = HashSet::new();
    for line in bench.lines() {
        let transcoded = line.split_once('\t').filter(|(codes, sentence)| {
            transcoder
                .transcode(sentence)
                .is_ok_and(|expected| expected.join(" ") == *codes)
        });
        match transcoded {
            Some((_, sentence)) => {
                sentences.insert(sentence);
            }
            None => failures += 1,
        }
    }
    let mut leaked = HashSet::new();
    for line in train.lines() {
        let line = line?;
        if transcoder.transcode(&line).is_err() {
            failures += 1;
        }
        if let Some(&sentence) = sentences.get(line.as_str()) {
            leaked.insert(sentence);
        }
    }
    for line in dev.lines() {
        if transcoder.transcode(&line?).is_err() {
            failures += 1;
        }
    }
    Ok(vec![
        Check {
            id: CheckId::CorpusTranscodes,
            passed: failures == 0,
            detail: format!("{name} 失敗 {failures} 句"),
        },
        Check {
            id: CheckId::CorpusNoLeak,
            passed: leaked.is_empty(),
            detail: format!("{name} test 句原樣出現在 train {} 句", leaked.len()),
        },
        Check {
            id: CheckId::CorpusDeterministic,
            passed: first == second,
            detail: if first == second {
                format!("{name} 建兩次 sha256 相同")
            } else {
                format!("{name} 第一次 {first:?}，第二次 {second:?}")
            },
        },
    ])
}

fn spot(id: CheckId, passed: bool) -> Check {
    Check {
        id,
        passed,
        detail: if passed { "符合" } else { "不符" }.into(),
    }
}
