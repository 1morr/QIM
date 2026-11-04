//! `cargo xtask data --verify` 的檢查（plan §4.5）。Unihan 交叉比對不是檢查、不影響通過與否，
//! 由 `pipeline` 算好放進 [`Report::unihan`]，比對本身在 `unihan` 模組。

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;
use std::io::{self, BufRead};
use std::path::PathBuf;

use qim_core::{
    CangjieVersion, CodeTable, Engine, EnglishPack, Key, LanguageModelPack, Lexicon, LexiconPack,
    Options, Pack, RadicalHintPack, RadicalHints, Session, SimplifierPack, UserModel,
};

use crate::corpus::Counts;
use crate::libcangjie::Row;
use crate::sources::sha256_hex;
use crate::transcode::Transcoder;
use crate::{bigram, english, lexicon};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckId {
    /// `qim-core` 解得開建出的速成、詞庫、語言模型、字根提示、簡繁與英文資料包，詞庫、語言模型與
    /// 字根提示是方案指名的那份。
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
    /// `a` 的首選是「日」（字序依字頻、Unihan `kGradeLevel`，plan §4.1）。
    RiIsTopForA,
    /// 詞庫的每個碼都由詞裡每個字的一個速成碼接成（plan §4.5）。
    WordCodesFromCharacters,
    /// 詞庫的 `anr` 有「門口」；連打用「門」+「口」也拼得出，所以直接查詞庫，首選由 `bench/fixed.tsv` 擋。
    MenKouForAnr,
    /// 碼表的每個字（含擴展區）在字根提示裡都至少有一個倉頡碼。
    RadicalHintCoversCharacters,
    /// 抽查字根提示：「丟」三代 `hgi`、五代先列 `mgi`；「問」兩代都是 `anr`。
    RadicalHintByVersion,
    /// 簡體模式打 `anr` 的候選有「门口」、沒有「門口」：簡繁資料包接得上核心（首選由核心測試擋）。
    SimplifiedMenKou,
    /// 中文狀態打 `the`，第 2 個候選是 `the`：英文資料包接得上核心（排位規則 4）。
    EnglishThe,
    /// 語言模型資料包不超過 [`bigram::MAX_BYTES`]（plan §4.3）。
    LanguageModelSize,
    /// 同樣的來源建兩次，六份資料包的 sha256 都相同。
    Deterministic,
    /// 語料的基準句與訓練切分每句都轉得了碼，基準句的碼與轉碼結果相同。
    CorpusTranscodes,
    /// 基準句沒有一句原樣出現在任何語料的 train（plan §5.2 的防洩漏；bigram 用所有 train 訓練）。
    CorpusNoLeak,
    /// 同樣的來源建兩次，基準句、訓練切分與字頻的 sha256 相同。
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
            Self::WordCodesFromCharacters => "詞的碼由字的速成碼組成",
            Self::MenKouForAnr => "抽查：詞庫的 anr 有「門口」",
            Self::RadicalHintCoversCharacters => "字根提示涵蓋碼表的字",
            Self::RadicalHintByVersion => "抽查：字根提示的三代與五代碼",
            Self::SimplifiedMenKou => "抽查：簡體模式 anr 有「门口」",
            Self::EnglishThe => "抽查：the 的第 2 個候選是 the",
            Self::LanguageModelSize => "bigram 資料包大小",
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
    /// 速成資料包的 sha256。
    pub sha256: String,
    pub lexicon_sha256: String,
    pub language_model_sha256: String,
    pub radical_hint_sha256: String,
    pub simplifier_sha256: String,
    pub english_sha256: String,
    /// 英文詞表的建置統計；[`run`] 不算，由呼叫端填。
    pub english: Option<english::Counts>,
    /// 詞庫的建置統計與有字頻的字數；[`run`] 不算，由呼叫端填。
    pub lexicon: Option<LexiconSummary>,
    /// bigram 的訓練統計；[`run`] 不算，由呼叫端填。
    pub language_model: Option<bigram::Summary>,
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

/// 詞庫資料包的建置統計。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LexiconSummary {
    pub counts: lexicon::Counts,
    /// 有字頻的字數。
    pub characters: usize,
}

impl fmt::Display for LexiconSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = &self.counts;
        write!(
            f,
            "詞庫：rime-essay {} 列，單字 {} 列不收；詞頻 < {} 濾掉 {} 詞、含連打字集以外的字濾掉 {} 詞；\
             粵語口語詞表補上 {} 詞；\
             收 {} 詞，碼 {} 個、詞條 {} 個（{} 詞的碼組合超過 {} 個，只收前 {} 個）\n\
             字頻：{} 字（各語料 train 切分乘上權重相加）",
            c.entries,
            c.single_characters,
            lexicon::MIN_FREQUENCY,
            c.below_threshold,
            c.outside_continuous,
            c.colloquial,
            c.words,
            c.codes,
            c.indexed,
            c.capped,
            lexicon::MAX_COMBINATIONS,
            lexicon::MAX_COMBINATIONS,
            self.characters,
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
        writeln!(f, "詞庫 sha256：{}", self.lexicon_sha256)?;
        writeln!(f, "bigram sha256：{}", self.language_model_sha256)?;
        writeln!(f, "字根提示 sha256：{}", self.radical_hint_sha256)?;
        writeln!(f, "簡繁 sha256：{}", self.simplifier_sha256)?;
        writeln!(f, "英文 sha256：{}", self.english_sha256)?;
        if let Some(lexicon) = &self.lexicon {
            writeln!(f, "{lexicon}")?;
        }
        if let Some(language_model) = &self.language_model {
            writeln!(f, "{language_model}")?;
        }
        if let Some(english) = &self.english {
            writeln!(f, "{}", english_summary(english))?;
        }
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

/// 英文詞表的統計一行；`cargo xtask data` 不驗證時也印。
pub fn english_summary(c: &english::Counts) -> String {
    format!(
        "英文：詞幹 {} 個，禁忌詞 {} 個不收；展開 {} 個詞形，打不出（z、撇號、重音…）{} 個不收；\
         依小寫歸併後收 {} 詞",
        c.stems, c.taboo, c.forms, c.untypable, c.words
    )
}

/// 一次建置寫出的六份資料包的位元組。
#[derive(Debug, Clone, Copy)]
pub struct PackBytes<'a> {
    pub quick: &'a [u8],
    pub lexicon: &'a [u8],
    pub language_model: &'a [u8],
    pub radical_hint: &'a [u8],
    pub simplifier: &'a [u8],
    pub english: &'a [u8],
}

/// `first` 是寫出的資料包，`second` 是由同樣的來源再建一次的結果。
pub fn run(rows: &[Row], first: PackBytes, second: PackBytes) -> Report {
    let sha256 = |p: PackBytes| {
        (
            sha256_hex(p.quick),
            sha256_hex(p.lexicon),
            sha256_hex(p.language_model),
            sha256_hex(p.radical_hint),
            sha256_hex(p.simplifier),
            sha256_hex(p.english),
        )
    };
    let (first_sha256, second_sha256) = (sha256(first), sha256(second));
    let mut report = Report {
        continuous: 0,
        extension: 0,
        codes: 0,
        sha256: first_sha256.0.clone(),
        lexicon_sha256: first_sha256.1.clone(),
        language_model_sha256: first_sha256.2.clone(),
        radical_hint_sha256: first_sha256.3.clone(),
        simplifier_sha256: first_sha256.4.clone(),
        english_sha256: first_sha256.5.clone(),
        english: None,
        lexicon: None,
        language_model: None,
        checks: Vec::new(),
        unihan: None,
        corpora: Vec::new(),
    };
    match decode(first) {
        Ok(Decoded {
            pack,
            lexicon,
            engine,
            hints,
        }) => pack_checks(&mut report, rows, &pack, &lexicon, &engine, &hints),
        Err(e) => report.checks.push(Check {
            id: CheckId::Decodes,
            passed: false,
            detail: e.to_string(),
        }),
    }
    report.checks.push(Check {
        id: CheckId::LanguageModelSize,
        passed: first.language_model.len() <= bigram::MAX_BYTES,
        detail: format!(
            "{} 位元組，上限 {}",
            first.language_model.len(),
            bigram::MAX_BYTES
        ),
    });
    report.checks.push(Check {
        id: CheckId::Deterministic,
        passed: first_sha256 == second_sha256,
        detail: format!(
            "第二次速成 {}、詞庫 {}、bigram {}、字根提示 {}、簡繁 {}、英文 {}",
            second_sha256.0,
            second_sha256.1,
            second_sha256.2,
            second_sha256.3,
            second_sha256.4,
            second_sha256.5
        ),
    });
    report
}

/// 解開的資料包；字根提示經殼用的 [`RadicalHints::for_scheme`] 取出，簡繁與英文資料包掛在
/// `engine` 上。
struct Decoded {
    pack: Pack,
    lexicon: Lexicon,
    engine: Engine,
    hints: RadicalHints,
}

fn decode(bytes: PackBytes) -> Result<Decoded, qim_core::PackError> {
    let pack = Pack::from_bytes(bytes.quick)?;
    let lexicon = LexiconPack::from_bytes(bytes.lexicon)?;
    let language_model = LanguageModelPack::from_bytes(bytes.language_model)?;
    let hints = RadicalHints::for_scheme(
        RadicalHintPack::from_bytes(bytes.radical_hint)?,
        &pack.scheme,
    )?;
    let simplifier = SimplifierPack::from_bytes(bytes.simplifier)?;
    let english = EnglishPack::from_bytes(bytes.english)?;
    let engine = Engine::new(pack.clone(), Some(lexicon.clone()), Some(language_model))?
        .with_simplifier(simplifier)
        .with_english(english);
    Ok(Decoded {
        pack,
        lexicon: lexicon.lexicon,
        engine,
        hints,
    })
}

/// 解得開之後的各項檢查，依 plan §4.5 的順序加進 `report`。
fn pack_checks(
    report: &mut Report,
    rows: &[Row],
    pack: &Pack,
    lexicon: &Lexicon,
    engine: &Engine,
    hints: &RadicalHints,
) {
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
    let user = UserModel::default();
    let top = |input: &str| engine.candidates(&user, input).into_iter().next();

    report.continuous = continuous.len();
    report.extension = extension.len();
    report.codes = pack.code_table.iter().count();
    report.checks.extend([
        Check {
            id: CheckId::Decodes,
            passed: true,
            detail: format!(
                "格式版本 {}，速成、詞庫、bigram、字根提示、簡繁與英文",
                qim_core::FORMAT_VERSION
            ),
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
        spot(CheckId::RiIsTopForA, top("a").as_deref() == Some("日")),
        word_codes(&pack.code_table, lexicon),
        spot(
            CheckId::MenKouForAnr,
            lexicon.lookup("anr").iter().any(|w| w.text == "門口"),
        ),
        hint_coverage(&continuous, &extension, hints),
        hint_by_version(hints),
        simplified_men_kou(engine),
        english_the(engine),
    ]);
}

/// 中文狀態（預設選項）經 `Session` 打 `the`，看第 2 個候選。
fn english_the(engine: &Engine) -> Check {
    let mut session = Session::default();
    session.set_options(Options {
        incognito: true,
        ..Options::default()
    });
    let mut user = UserModel::default();
    let mut candidates = Vec::new();
    for c in "the".chars() {
        candidates = session.handle(engine, &mut user, Key::Letter(c)).candidates;
    }
    let texts: Vec<&str> = candidates.iter().map(|c| c.text.as_str()).collect();
    Check {
        id: CheckId::EnglishThe,
        passed: texts.get(1) == Some(&"the"),
        detail: format!("第一頁：{}", texts.join(" ")),
    }
}

/// 簡體模式經 `Session` 打 `anr`，看第一頁的候選。
fn simplified_men_kou(engine: &Engine) -> Check {
    let mut session = Session::default();
    session.set_options(Options {
        simplified: true,
        incognito: true,
        ..Options::default()
    });
    let mut user = UserModel::default();
    let mut candidates = Vec::new();
    for c in "anr".chars() {
        candidates = session.handle(engine, &mut user, Key::Letter(c)).candidates;
    }
    let texts: Vec<&str> = candidates.iter().map(|c| c.text.as_str()).collect();
    Check {
        id: CheckId::SimplifiedMenKou,
        passed: texts.contains(&"门口") && !texts.contains(&"門口"),
        detail: format!("第一頁：{}", texts.join(" ")),
    }
}

/// 碼表的每個字都查得到至少一個倉頡碼，提示才不會在某個候選上落空。
fn hint_coverage(
    continuous: &BTreeSet<char>,
    extension: &BTreeSet<char>,
    hints: &RadicalHints,
) -> Check {
    let missing: Vec<String> = continuous
        .iter()
        .chain(extension)
        .filter(|&&c| {
            hints.codes(c, CangjieVersion::Cangjie3).is_empty()
                && hints.codes(c, CangjieVersion::Cangjie5).is_empty()
        })
        .map(char::to_string)
        .collect();
    const SHOWN: usize = 10;
    Check {
        id: CheckId::RadicalHintCoversCharacters,
        passed: missing.is_empty(),
        detail: if missing.is_empty() {
            format!("{} 字都有", continuous.len() + extension.len())
        } else {
            format!(
                "缺 {} 字：{}",
                missing.len(),
                missing[..missing.len().min(SHOWN)].join("")
            )
        },
    }
}

/// 三代與五代不同的「丟」、相同的「問」，各代的碼要對得上 libcangjie。
fn hint_by_version(hints: &RadicalHints) -> Check {
    let codes = |c: char, version: CangjieVersion| hints.codes(c, version).join(",");
    let diu = (
        codes('丟', CangjieVersion::Cangjie3),
        codes('丟', CangjieVersion::Cangjie5),
    );
    let wen = (
        codes('問', CangjieVersion::Cangjie3),
        codes('問', CangjieVersion::Cangjie5),
    );
    Check {
        id: CheckId::RadicalHintByVersion,
        passed: diu.0 == "hgi"
            && diu.1.split(',').next() == Some("mgi")
            && wen.0 == "anr"
            && wen.1 == "anr",
        detail: format!(
            "丟 三代 {}、五代 {}；問 三代 {}、五代 {}",
            diu.0, diu.1, wen.0, wen.1
        ),
    }
}

/// 詞庫的每個（碼, 詞）：碼要能切成詞裡每個字在碼表（連打字集）裡的一個速成碼。
/// 只用碼表反查，不經建詞庫的程式。
fn word_codes(code_table: &CodeTable, lexicon: &Lexicon) -> Check {
    let mut codes_of: BTreeMap<char, Vec<&str>> = BTreeMap::new();
    for (code, entries) in code_table.iter() {
        for e in entries.iter().filter(|e| !e.extension) {
            codes_of.entry(e.text).or_default().push(code);
        }
    }
    // 讀到第 i 個字時，碼已對上的長度有哪些可能。
    let composes = |code: &str, text: &str| {
        let mut ends = BTreeSet::from([0]);
        for ch in text.chars() {
            let quick = codes_of.get(&ch).map_or(&[][..], Vec::as_slice);
            ends = ends
                .iter()
                .flat_map(|&end| {
                    quick
                        .iter()
                        .filter(move |q| code[end..].starts_with(**q))
                        .map(move |q| end + q.len())
                })
                .collect();
        }
        ends.contains(&code.len())
    };
    let mut total = 0;
    let mut wrong = Vec::new();
    for (code, words) in lexicon.iter() {
        for word in words {
            total += 1;
            if !composes(code, &word.text) {
                wrong.push(format!("{code} {}", word.text));
            }
        }
    }
    const SHOWN: usize = 10;
    Check {
        id: CheckId::WordCodesFromCharacters,
        passed: wrong.is_empty(),
        detail: if wrong.is_empty() {
            format!("詞條 {total} 個都符合")
        } else {
            format!(
                "詞條 {total} 個，不合 {} 個：{}",
                wrong.len(),
                wrong[..wrong.len().min(SHOWN)].join("、")
            )
        },
    }
}

/// 一次語料建置寫出的四個檔的 sha256。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpusDigests {
    pub bench: String,
    pub train: String,
    pub dev: String,
    pub chars: String,
}

/// 一個語料的轉碼與決定性檢查；洩漏要看所有語料的 train，另由 [`leaks`] 檢查。
/// `bench` 是基準檔的內容（一行「逐字速成碼<TAB>句子」），`train`、`dev` 一行一句；
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
    let mut failures = bench
        .lines()
        .filter(|line| {
            !line.split_once('\t').is_some_and(|(codes, sentence)| {
                transcoder
                    .transcode(sentence)
                    .is_ok_and(|expected| expected.join(" ") == codes)
            })
        })
        .count();
    for line in train.lines().chain(dev.lines()) {
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

/// 每個基準集（名字與基準檔內容）各一項：有幾個不重複的基準句原樣出現在 `trains` 的任一份。
pub fn leaks(
    benches: &[(&str, &str)],
    trains: impl IntoIterator<Item = impl BufRead>,
) -> io::Result<Vec<Check>> {
    let sets: Vec<HashSet<&str>> = benches
        .iter()
        .map(|(_, bench)| {
            bench
                .lines()
                .filter_map(|line| line.split_once('\t').map(|(_, sentence)| sentence))
                .collect()
        })
        .collect();
    let mut leaked: Vec<HashSet<String>> = vec![HashSet::new(); benches.len()];
    for train in trains {
        for line in train.lines() {
            let line = line?;
            for (set, leaked) in sets.iter().zip(&mut leaked) {
                if set.contains(line.as_str()) {
                    leaked.insert(line.clone());
                }
            }
        }
    }
    Ok(benches
        .iter()
        .zip(leaked)
        .map(|((name, _), leaked)| Check {
            id: CheckId::CorpusNoLeak,
            passed: leaked.is_empty(),
            detail: format!("{name} test 句原樣出現在 train {} 句", leaked.len()),
        })
        .collect())
}

fn spot(id: CheckId, passed: bool) -> Check {
    Check {
        id,
        passed,
        detail: if passed { "符合" } else { "不符" }.into(),
    }
}
