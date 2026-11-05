//! 個人模型（plan §2.4）：使用者詞與學習計數。

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::Engine;

/// 衰減的時間常數，以上屏次數計（librime `formula_d`，brief §12.2）。
const DECAY_STEP: f64 = 200.0;

/// 個人機率在 `personal` 裡的權重 w（plan §2.3，libime `UserLanguageModel` 的式子）。libime 用
/// 0.2；M1 票 12 換上口語權重與新的 β、γ 後，0.2 讓口語集學習開反而多修正，dev 切分上 0.05
/// 對口語與維基都最好。
const PERSONAL_WEIGHT: f64 = 0.05;

/// 排位規則 2 的期限：距最後上屏不超過這麼多次上屏（plan §2.4 的初值）。
pub(crate) const RECENT_COMMITS: u64 = 10_000;

/// 使用者詞與學習計數；殼負責保存（plan §2.1）。各輸入方案分開記，互不影響（plan §2.4）。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UserModel {
    schemes: BTreeMap<String, SchemeModel>,
}

/// 一個輸入方案的使用者詞與學習計數。
#[derive(Debug, Clone, Default, PartialEq)]
struct SchemeModel {
    /// 這個方案學習過的上屏次數；學習計數的時間（plan §2.4），不讀系統時間。
    tick: u64,
    /// 輸入碼 → 候選文字 → 計數。
    entries: BTreeMap<String, BTreeMap<String, Count>>,
    /// 輸入碼 → 詞 → 使用者詞。計數另記在 `entries`，手動加了還沒用過的詞沒有計數。
    words: BTreeMap<String, BTreeMap<String, UserWord>>,
}

/// 一個使用者詞（plan §2.4）：像詞庫的詞一樣進 lattice。
#[derive(Debug, Clone, PartialEq, Eq)]
struct UserWord {
    source: WordSource,
    /// 打它就讓它成首選的自訂輸入碼（排位規則 1）；只有手動加的詞有。
    trigger: Option<String>,
}

/// 使用者詞怎麼來的。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WordSource {
    /// 使用者手動加的。
    Manual,
    /// 自動造詞：分段選字上屏學到的。
    AutoPhrase,
}

/// 匯出檔「來源」欄的值：只有學習計數的是 `learned`，使用者詞照它的來源。
const SOURCE_NAMES: [(Option<WordSource>, &str); 3] = [
    (None, "learned"),
    (Some(WordSource::Manual), "manual"),
    (Some(WordSource::AutoPhrase), "auto"),
];

impl SchemeModel {
    /// 碼 `code` 的使用者詞，沒有就建一個空的。
    fn words_at(&mut self, code: &str) -> &mut BTreeMap<String, UserWord> {
        self.words.entry(code.to_string()).or_default()
    }
}

/// 一個（輸入方案, 輸入碼, 候選文字）的計數。兩種上屏分開記（plan §2.4）：
/// - 整串上屏：整串輸入碼就是這個碼、上屏的就是這個候選；給排位規則 2 用。
/// - 句中各段的上屏也算進衰減值，給 `personal` 用。
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Count {
    /// 整串上屏的次數，不衰減，排位規則 2 的門檻用；只在句中出現過時是 0。
    commits: u64,
    /// 最後一次整串上屏的 tick；`commits` 是 0 時沒有意義。
    last_commit_tick: u64,
    /// 衰減值：整串上屏與句中各段的上屏每次加一，舊的隨上屏次數指數衰減（librime
    /// `formula_d`）；記的是 `last_tick` 當時的值。
    decayed: f64,
    /// 最後一次（整串或句中）上屏時的 tick。
    last_tick: u64,
}

impl UserModel {
    /// 一次上屏：整串輸入碼和每一段（輸入碼, 字詞）的衰減值各記一次，同一個鍵只記一次；
    /// 整串另記一次整串上屏（plan §2.4）。`segments` 依輸入碼的順序、接起來是整串；空的不記。
    pub(crate) fn learn(&mut self, scheme: &str, segments: &[(&str, &str)]) {
        if segments.is_empty() {
            return;
        }
        let model = self.scheme_mut(scheme);
        model.tick += 1;
        let tick = model.tick;
        let whole: (String, String) = (
            segments.iter().map(|s| s.0).collect(),
            segments.iter().map(|s| s.1).collect(),
        );
        let mut keys: Vec<(&str, &str)> = vec![(&whole.0, &whole.1)];
        for &segment in segments {
            if !keys.contains(&segment) {
                keys.push(segment);
            }
        }
        for (i, (code, text)) in keys.into_iter().enumerate() {
            let count = model
                .entries
                .entry(code.to_string())
                .or_default()
                .entry(text.to_string())
                .or_default();
            count.decayed = 1.0 + count.decayed * decay(tick - count.last_tick);
            count.last_tick = tick;
            // `keys` 的第一個是整串。
            if i == 0 {
                count.commits += 1;
                count.last_commit_tick = tick;
            }
        }
    }

    /// 手動加一個使用者詞（brief §4.5）。輸入碼是各字的碼接起來，所以連打裡也打得出；有字沒有
    /// 碼（英文、數字…）時改用觸發詞當輸入碼。`trigger` 是打它就讓它成首選的自訂輸入碼（排位
    /// 規則 1），只能用方案的鍵。已有同碼同字的使用者詞時改成手動加的、換上這個觸發詞。
    pub fn add_word(
        &mut self,
        engine: &Engine,
        text: &str,
        trigger: Option<&str>,
    ) -> Result<(), UserWordError> {
        if text.is_empty() {
            return Err(UserWordError::EmptyWord);
        }
        if text.chars().any(char::is_control) {
            return Err(UserWordError::ControlCharacter);
        }
        let scheme = engine.scheme();
        if let Some(trigger) = trigger
            && (trigger.is_empty() || !scheme.spells(trigger))
        {
            return Err(UserWordError::InvalidTrigger(trigger.to_string()));
        }
        let code = engine
            .code_of(text)
            .or_else(|| trigger.map(String::from))
            .ok_or(UserWordError::NoCode)?;
        self.scheme_mut(&scheme.id).words_at(&code).insert(
            text.to_string(),
            UserWord {
                source: WordSource::Manual,
                trigger: trigger.map(String::from),
            },
        );
        Ok(())
    }

    /// 觸發詞剛好是 `input` 的使用者詞（排位規則 1）；有幾個時，以 `input` 最近整串上屏的在前，
    /// 其餘依（碼, 詞）的順序。
    pub(crate) fn triggered<'a>(&'a self, scheme: &str, input: &str) -> Vec<&'a str> {
        let Some(model) = self.schemes.get(scheme) else {
            return Vec::new();
        };
        let mut found: Vec<(u64, &str)> = model
            .words
            .values()
            .flatten()
            .filter(|(_, word)| word.trigger.as_deref() == Some(input))
            .map(|(text, _)| {
                let tick = self
                    .get(scheme, input, text)
                    .map_or(0, |(_, c)| c.last_commit_tick);
                (tick, text.as_str())
            })
            .collect();
        found.sort_by_key(|&(tick, _)| std::cmp::Reverse(tick));
        found.into_iter().map(|(_, text)| text).collect()
    }

    /// 自動造詞（plan §2.4）：把（`code`, `text`）記成使用者詞；已經是使用者詞的不動。
    /// 上屏次數由同一次上屏的 [`UserModel::learn`] 記。
    pub(crate) fn add_auto_phrase(&mut self, scheme: &str, code: &str, text: &str) {
        self.scheme_mut(scheme)
            .words_at(code)
            .entry(text.to_string())
            .or_insert(UserWord {
                source: WordSource::AutoPhrase,
                trigger: None,
            });
    }

    fn scheme_mut(&mut self, scheme: &str) -> &mut SchemeModel {
        self.schemes.entry(scheme.to_string()).or_default()
    }

    /// 碼剛好是 `code` 的使用者詞。
    pub(crate) fn words<'a>(&'a self, scheme: &str, code: &str) -> impl Iterator<Item = &'a str> {
        self.schemes
            .get(scheme)
            .and_then(|m| m.words.get(code))
            .into_iter()
            .flat_map(|words| words.keys().map(String::as_str))
    }

    /// 方案 `scheme` 最長的使用者詞的碼長；lattice 往回找詞的上限要算進它。
    pub(crate) fn longest_word_code(&self, scheme: &str) -> usize {
        self.schemes
            .get(scheme)
            .and_then(|m| m.words.keys().map(String::len).max())
            .unwrap_or(0)
    }

    /// （方案, 碼, 字詞）整串上屏的次數；排位規則 2 的「剛好 1 次的不靠分數成首選」用。
    pub(crate) fn whole_commits(&self, scheme: &str, code: &str, text: &str) -> u64 {
        self.get(scheme, code, text)
            .map_or(0, |(_, count)| count.commits)
    }

    /// 排位規則 3（plan §2.4）：（`code`, `text`）是使用者詞且整串上屏過時，回傳最後一次整串
    /// 上屏的 tick，越大越近；否則是 `None`。
    pub(crate) fn last_tick_if_used_word(
        &self,
        scheme: &str,
        code: &str,
        text: &str,
    ) -> Option<u64> {
        let model = self.schemes.get(scheme)?;
        model.words.get(code)?.get(text)?;
        let count = model.entries.get(code)?.get(text)?;
        (count.commits >= 1).then_some(count.last_commit_tick)
    }

    /// 這組（方案, 碼, 字詞）學過沒有。
    pub(crate) fn has_learned(&self, scheme: &str, code: &str, text: &str) -> bool {
        self.get(scheme, code, text).is_some()
    }

    /// 排位規則 2（plan §2.4）：整串上屏 ≥ 2 次且距最後一次整串上屏不超過 [`RECENT_COMMITS`]
    /// 次上屏時，回傳那次的 tick，越大越近；否則是 `None`。句中各段的上屏不算。
    pub(crate) fn last_tick_if_chosen_twice(
        &self,
        scheme: &str,
        code: &str,
        text: &str,
    ) -> Option<u64> {
        let (model, count) = self.get(scheme, code, text)?;
        (count.commits >= 2 && model.tick - count.last_commit_tick <= RECENT_COMMITS)
            .then_some(count.last_commit_tick)
    }

    /// 個人機率（plan §2.3）：衰減值衰減到現在，除以它的上限；沒學過時是 0。
    pub(crate) fn personal_probability(&self, scheme: &str, code: &str, text: &str) -> f64 {
        self.get(scheme, code, text).map_or(0.0, |(model, count)| {
            count.decayed * decay(model.tick - count.last_tick) / max_decayed()
        })
    }

    fn get(&self, scheme: &str, code: &str, text: &str) -> Option<(&SchemeModel, &Count)> {
        let model = self.schemes.get(scheme)?;
        Some((model, model.entries.get(code)?.get(text)?))
    }
}

/// 匯出檔的第一行（plan §2.4）；格式改了要升版本（brief §5.3）。
const HEADER: &str = "# qim-userdict v1";
/// 方案 tick 的 metadata 行開頭，後接方案 id、Tab、tick；照 librime userdb 文字檔的
/// `#@/tick`（`docs/research/userdict-text-formats.md`）。
const TICK_KEY: &str = "#@/tick/";
/// 欄名註解行；匯入時和其他 `#` 開頭的行一樣略過。
const COLUMNS: &str =
    "# scheme\tword\tcode\tcommits\tlast_commit_tick\tdecayed\tlast_tick\tsource\ttrigger";

impl UserModel {
    /// 匯出成 `# qim-userdict v1` 的 TSV（plan §2.4），給殼存檔、換機或備份。
    /// 每個（方案, 輸入碼, 詞）一列，依方案、輸入碼、詞的順序。
    pub fn export(&self) -> String {
        let mut out = format!("{HEADER}\n");
        for (scheme, model) in &self.schemes {
            out.push_str(&format!("{TICK_KEY}{scheme}\t{}\n", model.tick));
        }
        out.push_str(COLUMNS);
        out.push('\n');
        for (scheme, model) in &self.schemes {
            let mut keys: Vec<(&str, &str)> = Vec::new();
            for (code, texts) in &model.entries {
                keys.extend(texts.keys().map(|t| (code.as_str(), t.as_str())));
            }
            for (code, words) in &model.words {
                keys.extend(words.keys().map(|t| (code.as_str(), t.as_str())));
            }
            keys.sort_unstable();
            keys.dedup();
            for (code, text) in keys {
                let count = model
                    .entries
                    .get(code)
                    .and_then(|t| t.get(text))
                    .copied()
                    .unwrap_or_default();
                let word = model.words.get(code).and_then(|w| w.get(text));
                let source = word.map(|w| w.source);
                let (_, source) = SOURCE_NAMES
                    .into_iter()
                    .find(|&(s, _)| s == source)
                    .expect("每種來源都有名字");
                let trigger = word.and_then(|w| w.trigger.as_deref()).unwrap_or("");
                out.push_str(&format!(
                    "{scheme}\t{text}\t{code}\t{}\t{}\t{}\t{}\t{source}\t{trigger}\n",
                    count.commits, count.last_commit_tick, count.decayed, count.last_tick,
                ));
            }
        }
        out
    }

    /// 讀 [`UserModel::export`] 的檔，取代整個個人模型。第一行不是 `# qim-userdict v1`、
    /// 或有一列不合格式時整份不收，錯誤帶行號。
    pub fn import(text: &str) -> Result<UserModel, ImportError> {
        let mut lines = text
            .lines()
            .map(|l| l.strip_suffix('\r').unwrap_or(l))
            .zip(1..);
        if lines.next().map(|(l, _)| l) != Some(HEADER) {
            return Err(ImportError::at(1, format!("第一行要是「{HEADER}」")));
        }
        let mut user = UserModel::default();
        let mut ticks: BTreeMap<&str, u64> = BTreeMap::new();
        let mut seen: BTreeSet<(&str, &str, &str)> = BTreeSet::new();
        for (line, number) in lines {
            let error = |reason: &str| ImportError::at(number, reason.to_string());
            if let Some(tick) = line.strip_prefix(TICK_KEY) {
                let (scheme, tick) = tick
                    .split_once('\t')
                    .ok_or_else(|| error("tick 行少了 Tab"))?;
                let tick = tick.parse().map_err(|_| error("tick 不是整數"))?;
                ticks.insert(scheme, tick);
                continue;
            }
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let row = Row::parse(line).map_err(error)?;
            let tick = ticks.get(row.scheme).copied();
            if tick.is_some_and(|t| row.count.last_tick.max(row.count.last_commit_tick) > t) {
                return Err(error("最後 tick 比方案的 tick 大"));
            }
            if !seen.insert((row.scheme, row.code, row.text)) {
                return Err(error("同方案、同輸入碼、同詞的列重複"));
            }
            let model = user.scheme_mut(row.scheme);
            // 手動加了還沒用過的詞沒有計數，匯出時寫 0；這裡也不建計數。
            if row.count != Count::default() {
                model
                    .entries
                    .entry(row.code.to_string())
                    .or_default()
                    .insert(row.text.to_string(), row.count);
            }
            if let Some(word) = row.word {
                model.words_at(row.code).insert(row.text.to_string(), word);
            }
            // 沒有 tick 行時，方案的 tick 取最大的最後 tick。
            model.tick = model.tick.max(row.count.last_tick);
        }
        for (scheme, model) in &mut user.schemes {
            if let Some(&tick) = ticks.get(scheme.as_str()) {
                model.tick = tick;
            }
        }
        Ok(user)
    }
}

/// 匯出檔的一列。
struct Row<'a> {
    scheme: &'a str,
    text: &'a str,
    code: &'a str,
    count: Count,
    word: Option<UserWord>,
}

impl<'a> Row<'a> {
    fn parse(line: &'a str) -> Result<Self, &'static str> {
        let fields: Vec<&str> = line.split('\t').collect();
        let [
            scheme,
            text,
            code,
            commits,
            last_commit_tick,
            decayed,
            last_tick,
            source,
            trigger,
        ] = fields[..]
        else {
            return Err("要有 9 欄，以 Tab 分隔");
        };
        if scheme.is_empty() || text.is_empty() || code.is_empty() {
            return Err("方案、詞、輸入碼不能是空的");
        }
        let integer = |s: &str| s.parse::<u64>().map_err(|_| "次數與 tick 要是非負整數");
        let decayed: f64 = decayed.parse().map_err(|_| "衰減值要是數字")?;
        if !decayed.is_finite() || decayed < 0.0 {
            return Err("衰減值要是非負的有限數");
        }
        let count = Count {
            commits: integer(commits)?,
            last_commit_tick: integer(last_commit_tick)?,
            decayed,
            last_tick: integer(last_tick)?,
        };
        let trigger = (!trigger.is_empty()).then(|| trigger.to_string());
        let (source, _) = SOURCE_NAMES
            .into_iter()
            .find(|&(_, name)| name == source)
            .ok_or("來源要是 manual、learned 或 auto")?;
        let word = match source {
            None if trigger.is_some() => return Err("學到的（learned）不是使用者詞，沒有觸發詞"),
            None if count.decayed == 0.0 => return Err("學到的（learned）要有衰減值"),
            Some(WordSource::AutoPhrase) if trigger.is_some() => {
                return Err("自動造詞沒有觸發詞");
            }
            None => None,
            Some(source) => Some(UserWord { source, trigger }),
        };
        Ok(Self {
            scheme,
            text,
            code,
            count,
            word,
        })
    }
}

/// [`UserModel::import`] 不收的檔：第幾行（從 1 起）、為什麼。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportError {
    pub line: usize,
    pub reason: String,
}

impl ImportError {
    fn at(line: usize, reason: String) -> Self {
        Self { line, reason }
    }
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "第 {} 行：{}", self.line, self.reason)
    }
}

impl std::error::Error for ImportError {}

/// [`UserModel::add_word`] 加不進去的原因。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserWordError {
    EmptyWord,
    /// 詞含 Tab、換行這類控制字元；匯出的 TSV 放不下。
    ControlCharacter,
    /// 觸發詞是空的或含方案以外的鍵。
    InvalidTrigger(String),
    /// 有字沒有碼，又沒給觸發詞，打不出來。
    NoCode,
}

impl fmt::Display for UserWordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyWord => write!(f, "詞是空的"),
            Self::ControlCharacter => write!(f, "詞不能含 Tab、換行這類控制字元"),
            Self::InvalidTrigger(t) => write!(f, "觸發詞「{t}」要是方案的鍵，不能是空的"),
            Self::NoCode => write!(f, "詞裡有字沒有碼，要給觸發詞才打得出"),
        }
    }
}

impl std::error::Error for UserWordError {}

/// `personal`（plan §2.3）：個人機率 `p` 在一段原本的 ln 機率 `log_static` 上加的分，只加不減；
/// `p` 是 0（沒學過）時是 0。
pub(crate) fn personal_bonus(log_static: f64, p: f64) -> f64 {
    if p == 0.0 {
        return 0.0;
    }
    let mixed = ((1.0 - PERSONAL_WEIGHT) * log_static.exp() + PERSONAL_WEIGHT * p).ln();
    (mixed - log_static).max(0.0)
}

/// 衰減值的上限：每次上屏都是同一個（輸入碼, 字詞）時趨近的值，`1 / (1 − e^(−1/200))` ≈ 200.5。
/// 個人機率是衰減值除以它（libime 的歷史機率也除以滿載的總量，`docs/research/libime-user-model.md` §2）。
fn max_decayed() -> f64 {
    1.0 / (1.0 - decay(1))
}

/// 經過 `steps` 次上屏後剩下的比例。
fn decay(steps: u64) -> f64 {
    (-(steps as f64) / DECAY_STEP).exp()
}
