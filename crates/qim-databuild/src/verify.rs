//! `cargo xtask data --verify` 的檢查（plan §4.5）。Unihan 交叉比對不是檢查、不影響通過與否，
//! 由 `pipeline` 算好放進 [`Report::unihan`]，比對本身在 `unihan` 模組。

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;
use std::io::{self, BufRead};
use std::path::PathBuf;

use qim_core::{
    CangjieVersion, CodeTable, Engine, EnglishPack, Key, LanguageModel, LanguageModelPack, Lexicon,
    LexiconPack, Options, RadicalHintPack, RadicalHints, SchemePack, Session, SimplifierPack,
    UserModel,
};

use crate::built::{self, BuiltPack};
use crate::common_forms::{BothForms, CommonForms, ONE_WAY_DISTINCT, REVERSED_ONE_WAY};
use crate::corpus::Counts;
use crate::libcangjie::Row;
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
    /// 繁簡同形與本身是正規繁體字的字（[`NOT_SIMPLIFIED`]）沒有標成簡體字（brief §4.1，M3 票 19）。
    NotSimplified,
    /// 繁體模式 `ls` 的「门」排在同碼的所有繁體字之後（M3 票 19）。
    MenAfterTraditional,
    /// 語言模型資料包不超過 [`bigram::MAX_BYTES`]（plan §4.3）。
    LanguageModelSize,
    /// 同樣的來源建兩次，每份資料包的 sha256 都相同、清單一樣。
    Deterministic,
    /// 語料的基準句與訓練切分每句都轉得了碼，基準句的碼與轉碼結果相同。
    CorpusTranscodes,
    /// 基準句沒有一句原樣出現在任何語料的 train（plan §5.2 的防洩漏；bigram 用所有 train 訓練）。
    CorpusNoLeak,
    /// 同樣的來源建兩次，基準句、訓練切分與字頻的 sha256 相同。
    CorpusDeterministic,
    /// 前處理保留的粵語用字（`corpora::CANTONESE_KEPT`）在 train 的字頻都大於 0（M1 票 17b）。
    KeptCharacters,
    /// 字形對照表（`common_forms`）換掉的香港寫法、並存寫法的另一種寫法在 train 的字頻都是 0
    /// （M1 票 17b、17c，brief §4.1）。
    CommonForms,
    /// 詞庫沒有對照表換掉的寫法與反向的錯字；並存寫法的詞每種寫法都收（`verify::lexicon_forms`，
    /// M1 票 17c）。
    LexiconForms,
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
            Self::NotSimplified => "抽查：正規繁體字不是簡體字",
            Self::MenAfterTraditional => "抽查：ls 的「门」在繁體字之後",
            Self::LanguageModelSize => "bigram 資料包大小",
            Self::Deterministic => "決定性：建兩次 sha256 相同",
            Self::CorpusTranscodes => "語料轉碼",
            Self::CorpusNoLeak => "語料不洩漏",
            Self::CorpusDeterministic => "語料決定性",
            Self::KeptCharacters => "前處理保留粵語用字",
            Self::CommonForms => "字形用常見寫法",
            Self::LexiconForms => "詞庫用常見寫法",
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
    /// 進「有前文」基準檔的題數。
    pub context_sentences: usize,
    /// 「有前文」的基準檔，相對 repo 根目錄。
    pub context_bench: PathBuf,
}

impl fmt::Display for CorpusSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = &self.counts;
        write!(
            f,
            "語料 {}：train {} 句、dev {} 句、test {} 句（不重複），其中 {} 句進 {}、有前文的 {} 句進 {}；含字集外字元丟掉 {} 句",
            self.name,
            c.train,
            c.dev,
            c.test,
            self.bench_sentences,
            self.bench.display(),
            self.context_sentences,
            self.context_bench.display(),
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
            "詞庫：rime-essay {} 列，單字 {} 列不收；換成常見寫法 {} 詞（其中 {} 詞和別的詞合併）；\
             詞頻 < {} 濾掉 {} 詞、含連打字集以外的字濾掉 {} 詞；\
             粵語口語詞表補上 {} 詞；\
             收 {} 詞，並存寫法另收 {} 種寫法；碼 {} 個、詞條 {} 個（{} 詞的碼組合超過 {} 個，只收前 {} 個）\n\
             字頻：{} 字（各語料 train 切分乘上權重相加，並存寫法兩字共用）",
            c.entries,
            c.single_characters,
            c.converted,
            c.merged,
            lexicon::MIN_FREQUENCY,
            c.below_threshold,
            c.outside_continuous,
            c.colloquial,
            c.words,
            c.spellings,
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

/// `first` 是寫出的資料包，`second` 是由同樣的來源再建一次的結果。
pub fn run(rows: &[Row], first: &[BuiltPack], second: &[BuiltPack]) -> Report {
    let mut report = Report {
        continuous: 0,
        extension: 0,
        codes: 0,
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
        Err(detail) => report.checks.push(Check {
            id: CheckId::Decodes,
            passed: false,
            detail,
        }),
    }
    report
        .checks
        .push(match built::find::<LanguageModelPack>(first) {
            Ok(model) => Check {
                id: CheckId::LanguageModelSize,
                passed: model.bytes.len() <= bigram::MAX_BYTES,
                detail: format!("{} 位元組，上限 {}", model.bytes.len(), bigram::MAX_BYTES),
            },
            Err(detail) => Check {
                id: CheckId::LanguageModelSize,
                passed: false,
                detail,
            },
        });
    let digests = |packs: &[BuiltPack]| {
        packs
            .iter()
            .map(|p| (p.name, p.sha256()))
            .collect::<Vec<_>>()
    };
    let second_digests = digests(second);
    report.checks.push(Check {
        id: CheckId::Deterministic,
        passed: digests(first) == second_digests,
        detail: format!(
            "第二次{}",
            second_digests
                .iter()
                .map(|(name, sha256)| format!("{name} {sha256}"))
                .collect::<Vec<_>>()
                .join("、")
        ),
    });
    report
}

/// 解開的資料包；字根提示經殼用的 [`RadicalHints::for_scheme`] 取出，簡繁與英文資料包掛在
/// `engine` 上。
struct Decoded {
    pack: SchemePack,
    lexicon: Lexicon,
    engine: Engine,
    hints: RadicalHints,
}

fn decode(packs: &[BuiltPack]) -> Result<Decoded, String> {
    let pack = built::decode::<SchemePack>(packs)?;
    let lexicon = built::decode::<LexiconPack>(packs)?;
    let language_model = built::decode::<LanguageModelPack>(packs)?;
    let hints = RadicalHints::for_scheme(built::decode::<RadicalHintPack>(packs)?, &pack.scheme)
        .map_err(|e| e.to_string())?;
    let simplifier = built::decode::<SimplifierPack>(packs)?;
    let english = built::decode::<EnglishPack>(packs)?;
    let engine = Engine::new(pack.clone(), Some(lexicon.clone()), Some(language_model))
        .map_err(|e| e.to_string())?
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
    pack: &SchemePack,
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
        not_simplified(&pack.code_table),
        men_after_traditional(&pack.code_table, engine),
    ]);
}

/// 不能標成簡體字的字：繁簡同形的人、日，與本身是正規繁體字、又是別的字的簡體寫法的字
/// （M3 票 19 列的台（臺）、后（後）、里（裡）…）。
pub const NOT_SIMPLIFIED: [char; 17] = [
    '人', '日', '台', '后', '里', '干', '面', '系', '云', '松', '只', '范', '谷', '斗', '制', '准',
    '几',
];

/// 碼表裡標成簡體字的字都不在 [`NOT_SIMPLIFIED`]；細節列簡體字的字數。
fn not_simplified(code_table: &CodeTable) -> Check {
    let simplified: BTreeSet<char> = code_table
        .iter()
        .flat_map(|(_, entries)| entries.iter().filter(|e| e.simplified).map(|e| e.text))
        .collect();
    let wrong: String = NOT_SIMPLIFIED
        .iter()
        .filter(|c| simplified.contains(c))
        .collect();
    Check {
        id: CheckId::NotSimplified,
        passed: wrong.is_empty(),
        detail: if wrong.is_empty() {
            format!(
                "簡體字 {} 個，不含 {}",
                simplified.len(),
                NOT_SIMPLIFIED.iter().collect::<String>()
            )
        } else {
            format!("簡體字 {} 個，誤標 {wrong}", simplified.len())
        },
    }
}

/// 繁體模式（`Engine::candidates`）的 `ls`：「门」在，而且排在碼表裡 `ls` 每個沒標成簡體字的
/// 連打字集的字之後。
fn men_after_traditional(code_table: &CodeTable, engine: &Engine) -> Check {
    let listed = engine.candidates(&UserModel::default(), "ls");
    let at = |text: char| listed.iter().position(|c| *c == text.to_string());
    let traditional = code_table
        .lookup("ls")
        .iter()
        .filter(|e| !e.extension && !e.simplified)
        .map(|e| at(e.text));
    let passed = match at('门') {
        Some(men) => traditional.into_iter().all(|t| t.is_some_and(|t| t < men)),
        None => false,
    };
    // 只列到「门」為止：後面是只蓋住 `l` 的字。
    let shown = at('门').map_or(listed.len(), |men| men + 1);
    Check {
        id: CheckId::MenAfterTraditional,
        passed,
        detail: format!("ls：{}", listed[..shown].join(" ")),
    }
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

/// 一次語料建置寫出的五個檔的 sha256。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpusDigests {
    pub bench: String,
    pub context_bench: String,
    pub train: String,
    pub dev: String,
    pub chars: String,
}

/// 一個語料的轉碼與決定性檢查；洩漏要看所有語料的 train，另由 [`leaks`] 檢查。
/// `bench` 是基準檔的內容（一行「逐字速成碼<TAB>句子」），`context_bench` 是「有前文」的
/// 基準檔（一行「前文<TAB>逐字速成碼<TAB>句子」），`train`、`dev` 一行一句；
/// `first`、`second` 是兩次建置的 sha256。
#[allow(clippy::too_many_arguments)] // 兩份基準、兩份切分與兩次 sha256，拆成結構只為了湊參數數
pub fn corpus(
    name: &str,
    transcoder: &Transcoder,
    bench: &str,
    context_bench: &str,
    train: impl BufRead,
    dev: impl BufRead,
    first: &CorpusDigests,
    second: &CorpusDigests,
) -> io::Result<Vec<Check>> {
    let codes_match = |line: &str, fields: usize| {
        let fields_of: Vec<&str> = line.split('\t').collect();
        let [.., codes, sentence] = fields_of.as_slice() else {
            return false;
        };
        fields_of.len() == fields
            && transcoder
                .transcode(sentence)
                .is_ok_and(|expected| expected.join(" ") == *codes)
    };
    let mut failures = bench.lines().filter(|l| !codes_match(l, 2)).count()
        + context_bench.lines().filter(|l| !codes_match(l, 3)).count();
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
/// 句子是每行最後一欄，基準檔與「有前文」的基準檔都適用。
pub fn leaks(
    benches: &[(&str, &str)],
    trains: impl IntoIterator<Item = impl BufRead>,
) -> io::Result<Vec<Check>> {
    let sets: Vec<HashSet<&str>> = benches
        .iter()
        .map(|(_, bench)| {
            bench
                .lines()
                .filter_map(|line| line.rsplit_once('\t').map(|(_, sentence)| sentence))
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

/// `kept` 的每個字在 `frequency`（各語料 train 乘上權重相加的字頻）都大於 0；
/// 沒保留住時字頻是 0，例如 `s2hk` 把「晒」換成「曬」。
pub fn kept_characters(frequency: &BTreeMap<char, u64>, kept: &[char]) -> Check {
    let counts: Vec<(char, u64)> = kept
        .iter()
        .map(|&ch| (ch, frequency.get(&ch).copied().unwrap_or(0)))
        .collect();
    Check {
        id: CheckId::KeptCharacters,
        passed: counts.iter().all(|&(_, n)| n > 0),
        detail: counts
            .iter()
            .map(|(ch, n)| format!("{ch} {n}"))
            .collect::<Vec<_>>()
            .join("、"),
    }
}

/// `forms` 換掉的字與並存寫法的另一種寫法在 `frequency`（各語料 train 乘上權重相加的字頻）都不
/// 出現：前處理確實接上了對照表。
pub fn common_forms(frequency: &BTreeMap<char, u64>, forms: &CommonForms) -> Check {
    let joined = |pairs: Vec<(char, char)>| {
        let list: Vec<String> = pairs.iter().map(|(f, t)| format!("{f}→{t}")).collect();
        format!("{} 組（{}）", list.len(), list.join("、"))
    };
    let both: Vec<(char, char)> = forms.both().two_way_pairs().map(|(r, o)| (o, r)).collect();
    let mut pairs = joined(forms.pairs().collect());
    if !both.is_empty() {
        pairs = format!("{pairs}、並存寫法 {}", joined(both.clone()));
    }
    let left: Vec<String> = forms
        .pairs()
        .chain(both)
        .filter_map(|(from, _)| {
            frequency
                .get(&from)
                .filter(|&&n| n > 0)
                .map(|n| format!("{from} {n}"))
        })
        .collect();
    Check {
        id: CheckId::CommonForms,
        passed: left.is_empty(),
        detail: if left.is_empty() {
            format!("對照表 {pairs}，換掉的寫法在 train 都沒有")
        } else {
            format!("對照表 {pairs}，換掉的寫法還在 train：{}", left.join("、"))
        },
    }
}

/// 詞庫與語言模型（`pipeline` 建出、解回來的那份）的寫法，確認建詞庫與 bigram 接上了 `forms`：
/// - 沒有 `forms` 換掉的寫法，也沒有 [`REVERSED_ONE_WAY`]；
/// - 雙向的並存寫法：含它的詞每種寫法都收、詞頻相同，兩個字的字頻相同；詞裡含它的相鄰字，
///   每種寫法的 bigram 相同，兩個字的 unigram 也相同（[`two_way_probability_mismatches`]）；
/// - 單向的：含前一個字的詞，換成後一個字的寫法也收，詞頻不低於它（[`ONE_WAY_DISTINCT`] 除外）。
pub fn lexicon_forms(lexicon: &Lexicon, model: &LanguageModel, forms: &CommonForms) -> Check {
    let replaced: BTreeSet<char> = forms.pairs().map(|(from, _)| from).collect();
    let both = forms.both();
    let two_way = |c: char| both.other(c).is_some() || both.representative(c) != c;
    let mut frequency: BTreeMap<&str, u64> = BTreeMap::new();
    for (_, words) in lexicon.iter() {
        for w in words {
            frequency.insert(&w.text, w.frequency);
        }
    }
    let (mut left, mut missing, mut unequal) = (vec![], vec![], vec![]);
    let (mut with_two_way, mut with_one_way) = (0, 0);
    for (&text, &n) in &frequency {
        if text.chars().any(|c| replaced.contains(&c)) || REVERSED_ONE_WAY.contains(&text) {
            left.push(text.to_string());
        }
        if text.chars().any(two_way) {
            with_two_way += 1;
            let representative: String = text.chars().map(|c| both.representative(c)).collect();
            for spelling in both.two_way_spellings(&representative) {
                match frequency.get(spelling.as_str()) {
                    None => missing.push(spelling),
                    // 每組只在代表寫法那個詞報一次。
                    Some(&m) if m != n && text == representative => {
                        unequal.push(format!("{text} {n}／{spelling} {m}"));
                    }
                    Some(_) => {}
                }
            }
        }
        // `ONE_WAY_DISTINCT` 照設計不生出另一種寫法（那是別的詞，詞頻可以較低），不查。
        if text.chars().any(|c| both.one_way(c).is_some()) && !ONE_WAY_DISTINCT.contains(&text) {
            with_one_way += 1;
            let also: String = text.chars().map(|c| both.one_way(c).unwrap_or(c)).collect();
            match frequency.get(also.as_str()) {
                None => missing.push(also),
                Some(&m) if m < n => unequal.push(format!("{text} {n}／{also} {m}")),
                Some(_) => {}
            }
        }
    }
    missing.sort();
    missing.dedup();
    let chars: Vec<String> = both
        .two_way_pairs()
        .filter_map(|(r, o)| {
            let (a, b) = (lexicon.char_frequency(r), lexicon.char_frequency(o));
            (a != b).then(|| format!("{r} {a}、{o} {b}"))
        })
        .collect();
    let bigrams = two_way_probability_mismatches(lexicon, model, both);
    const SHOWN: usize = 10;
    let few = |items: &[String]| {
        let shown = items[..items.len().min(SHOWN)].join("、");
        if items.len() > SHOWN {
            format!("{shown} 等 {} 個", items.len())
        } else {
            shown
        }
    };
    let mut problems = Vec::new();
    for (label, items) in [
        ("不該有的寫法還在詞庫", &left),
        ("缺", &missing),
        ("詞頻不合", &unequal),
        ("字頻不同", &chars),
        ("bigram 不同", &bigrams),
    ] {
        if !items.is_empty() {
            problems.push(format!("{label}：{}", few(items)));
        }
    }
    Check {
        id: CheckId::LexiconForms,
        passed: problems.is_empty(),
        detail: if problems.is_empty() {
            format!(
                "對照表 {} 組換掉的寫法與反向的錯字在詞庫都沒有；雙向並存寫法 {} 組：含它的詞 \
                 {with_two_way} 個，每種寫法都收、詞頻相同，字頻與 bigram 相同；單向 {} 組：含它的詞 \
                 {with_one_way} 個都另收另一種寫法",
                replaced.len(),
                both.two_way_pairs().count(),
                both.one_way_pairs().count()
            )
        } else {
            problems.join("；")
        },
    }
}

/// 雙向並存寫法在 `model` 裡機率不同的地方：每組兩個字的 unigram，以及詞庫裡含它的詞的每對相鄰字
/// （代表寫法的那對，和前後字各換或不換成另一種寫法的每一對）。每項寫成「代表寫法的那對／不同的那對」。
fn two_way_probability_mismatches(
    lexicon: &Lexicon,
    model: &LanguageModel,
    both: &BothForms,
) -> Vec<String> {
    // (前一字, 字)，都寫成代表寫法；前一字是 `None` 的是 unigram。
    let mut pairs: BTreeSet<(Option<char>, char)> =
        both.two_way_pairs().map(|(r, _)| (None, r)).collect();
    for (_, words) in lexicon.iter() {
        for w in words {
            let chars: Vec<char> = w.text.chars().map(|c| both.representative(c)).collect();
            for pair in chars.windows(2) {
                let (prev, next) = (pair[0], pair[1]);
                if both.other(prev).is_some() || both.other(next).is_some() {
                    pairs.insert((Some(prev), next));
                }
            }
        }
    }
    let spellings_of = |c: char| std::iter::once(c).chain(both.other(c));
    let shown = |prev: Option<char>, next: char| prev.into_iter().chain([next]).collect::<String>();
    let mut different = Vec::new();
    for (prev, next) in pairs {
        let expected = model.log_prob(prev, next);
        let prevs: Vec<Option<char>> =
            prev.map_or(vec![None], |p| spellings_of(p).map(Some).collect());
        for other_prev in prevs {
            for other_next in spellings_of(next) {
                if model.log_prob(other_prev, other_next) != expected {
                    different.push(format!(
                        "{}／{}",
                        shown(prev, next),
                        shown(other_prev, other_next)
                    ));
                }
            }
        }
    }
    different
}

fn spot(id: CheckId, passed: bool) -> Check {
    Check {
        id,
        passed,
        detail: if passed { "符合" } else { "不符" }.into(),
    }
}
