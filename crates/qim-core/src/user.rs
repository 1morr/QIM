//! 個人模型（plan §2.4）：使用者詞與學習計數。

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::remembered::{self, REMEMBERED_STRINGS_MAX};
use crate::{Engine, Segment};

/// 衰減的時間常數，以上屏次數計（librime `formula_d`，brief §12.2）。
const DECAY_STEP: f64 = 200.0;

/// 個人機率在 `personal` 裡的權重 w（plan §2.3，libime `UserLanguageModel` 的式子）。libime 用
/// 0.2；M1 票 12 換上口語權重與新的 β、γ 後，0.2 讓口語集學習開反而多修正，dev 切分上 0.05
/// 對口語與維基都最好。
const PERSONAL_WEIGHT: f64 = 0.05;

/// 個人機率裡個人 bigram 的權重 λ（plan §2.3）：`P_personal = P_unigram + (1 − P_unigram)·λ·P_bigram`。
/// libime `HistoryBigram` 是 `(1 − λ)·P_unigram + λ·P_bigram`、λ = 0.8
/// （`docs/research/libime-user-model.md` §2），沒有前一個字詞時學過的段只剩兩成的個人分；
/// 這裡 bigram 只往上加，學習計數的分照舊。dev 切分的學習模式掃 0–1，0.8 修正次數最少（M3 票 12）。
const BIGRAM_WEIGHT: f64 = 0.8;

/// 排位規則 2 的期限：距最後上屏不超過這麼多次上屏（plan §2.4 的初值）。
pub(crate) const RECENT_COMMITS: u64 = 10_000;

/// 使用者詞與學習計數；殼負責保存（plan §2.1）。各輸入方案分開記，互不影響（plan §2.4）。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UserModel {
    schemes: BTreeMap<String, SchemeModel>,
    /// 個人 bigram：只看文字，各方案共用（plan §2.4）。它的 tick 也是記住的字串的時間。
    bigrams: Bigrams,
    /// 記住的字串（plan §2.4，M3 票 15）：使用者打過的 email → 衰減值，各方案共用。每記一次共用的
    /// tick（`bigrams.tick`）加一，衰減同個人 bigram。
    remembered: BTreeMap<String, Decayed>,
}

/// 個人 bigram（plan §2.3）：上屏時記「前一個字詞 → 這一段」的衰減值，算分時給這一段接在
/// 那個字詞後面的條件機率。
#[derive(Debug, Clone, Default, PartialEq)]
struct Bigrams {
    /// 學習過的上屏次數，各方案一起算；衰減的時間。
    tick: u64,
    /// 前一個字詞 → 這一段 → 衰減值。
    entries: BTreeMap<String, BTreeMap<String, Decayed>>,
}

/// 一個衰減值與它最後更新的 tick（librime `formula_d`）。
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Decayed {
    value: f64,
    last_tick: u64,
}

impl Decayed {
    /// 衰減到 `tick` 的值。
    fn at(self, tick: u64) -> f64 {
        self.value * decay(tick - self.last_tick)
    }

    /// 在 `tick` 又上屏（或記住）一次：衰減到現在再加一（librime `formula_d`）。
    fn bump(&mut self, tick: u64) {
        self.value = 1.0 + self.at(tick);
        self.last_tick = tick;
    }
}

/// 接在一個前一個字詞後面的個人 bigram（[`UserModel::bigrams_after`]）。
#[derive(Debug, Clone, Copy)]
pub(crate) struct Following<'a> {
    next: &'a BTreeMap<String, Decayed>,
    tick: u64,
    /// 這個字詞後面各段的衰減值衰減到現在的總和：條件機率的分母。
    total: f64,
}

impl Following<'_> {
    /// `text` 接在這個字詞後面的個人機率；沒學過是 0。
    pub(crate) fn probability(&self, text: &str) -> f64 {
        self.next
            .get(text)
            .map_or(0.0, |d| d.at(self.tick) / self.total)
    }

    /// 學過接在這個字詞後面的各段與它們的個人機率，依文字排序（聯想用）。
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, f64)> + '_ {
        self.next
            .iter()
            .map(|(text, d)| (text.as_str(), d.at(self.tick) / self.total))
    }
}

/// 一個輸入方案的個人模型：使用者詞與學習計數。核心內部經 [`UserModel::for_scheme`] 取用，
/// 所以查詢與學習的方法不必逐一帶方案。
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct SchemeModel {
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
    /// 常用語：使用者存的句子或段落，碼是 [`PHRASE_CODE`]。
    Phrase,
}

/// 常用語的碼：空字串。常用語不是由字的碼打出來的，lattice 查的碼都不是空的，所以它不進連打，
/// 只靠觸發詞（排位規則 1）或從清單選（[`crate::Session::text`]）上屏。
const PHRASE_CODE: &str = "";

/// 匯出檔「來源」欄的值：只有學習計數的是 `learned`，使用者詞照它的來源。
const SOURCE_NAMES: [(Option<WordSource>, &str); 4] = [
    (None, "learned"),
    (Some(WordSource::Manual), "manual"),
    (Some(WordSource::AutoPhrase), "auto"),
    (Some(WordSource::Phrase), "phrase"),
];

/// 一個（輸入碼, 候選文字）的計數；方案由所在的 [`SchemeModel`] 決定。兩種上屏分開記（plan §2.4）：
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
    /// 方案 `scheme` 的個人模型；沒記過時是空的。
    pub(crate) fn for_scheme(&self, scheme: &str) -> &SchemeModel {
        static EMPTY: SchemeModel = SchemeModel {
            tick: 0,
            entries: BTreeMap::new(),
            words: BTreeMap::new(),
        };
        self.schemes.get(scheme).unwrap_or(&EMPTY)
    }

    /// 學一次上屏（plan §2.4）：方案 `scheme` 的學習計數（[`SchemeModel::learn`]），加上個人
    /// bigram。`previous` 是組字前面的一個字詞（前文給的，見 [`crate::Session::set_context`]），
    /// 沒有時第一段不記；`segments` 同 [`SchemeModel::learn`]，空的不記。
    pub(crate) fn learn(&mut self, scheme: &str, previous: Option<&str>, segments: &[Segment]) {
        if segments.is_empty() {
            return;
        }
        self.for_scheme_mut(scheme).learn(segments);
        self.bigrams.learn(previous, segments);
    }

    /// 選了一個聯想（plan §2.4）：只記個人 bigram「`previous` → `text`」。聯想沒有輸入碼，
    /// 不記學習計數。
    pub(crate) fn learn_association(&mut self, previous: &str, text: &str) {
        self.bigrams.learn(
            Some(previous),
            &[Segment::new(String::new(), text.to_string())],
        );
    }

    /// 學過個人 bigram 沒有。
    pub(crate) fn has_bigrams(&self) -> bool {
        !self.bigrams.entries.is_empty()
    }

    /// 前一個字詞是 `previous` 的個人 bigram；沒學過時是 `None`。
    pub(crate) fn bigrams_after(&self, previous: &str) -> Option<Following<'_>> {
        let next = self.bigrams.entries.get(previous)?;
        let tick = self.bigrams.tick;
        let total = next.values().map(|d| d.at(tick)).sum();
        Some(Following { next, tick, total })
    }

    /// 記住一個字串（[`crate::Session`] 從送出去的字裡認出來的 email）：衰減值加一，共用的 tick 加一。
    /// email 不分大小寫（網域本來就不分，本地部分實務上也不分）：只留一條，換成這次打的寫法。超過
    /// [`REMEMBERED_STRINGS_MAX`] 條時淘汰最久沒用的（最後 tick 最小）。
    pub(crate) fn remember_string(&mut self, text: &str) {
        self.bigrams.tick += 1;
        let same = self
            .remembered
            .keys()
            .find(|old| old.eq_ignore_ascii_case(text))
            .cloned();
        let mut decayed = same
            .and_then(|old| self.remembered.remove(&old))
            .unwrap_or_default();
        decayed.bump(self.bigrams.tick);
        self.remembered.insert(text.to_string(), decayed);
        if self.remembered.len() > REMEMBERED_STRINGS_MAX
            && let Some(oldest) = self
                .remembered
                .iter()
                .min_by_key(|(_, d)| d.last_tick)
                .map(|(text, _)| text.clone())
        {
            self.remembered.remove(&oldest);
        }
    }

    /// 記住的字串，依文字排；殼的清單用，使用者可以逐條刪（[`UserModel::forget_string`]）。
    pub fn remembered_strings(&self) -> Vec<String> {
        self.remembered.keys().cloned().collect()
    }

    /// 忘掉記住的字串 `text`，匯出檔裡不再有它；沒記過時回傳 `false`。
    pub fn forget_remembered_string(&mut self, text: &str) -> bool {
        self.remembered.remove(text).is_some()
    }

    /// 方案字母表 `alphabet` 裡的字母（小寫，[`crate::remembered::letters`]）以 `prefix` 開頭的記住的
    /// 字串，衰減值大的在前，同值依文字。
    pub(crate) fn strings_starting_with(&self, prefix: &str, alphabet: &str) -> Vec<&str> {
        let tick = self.bigrams.tick;
        let mut found: Vec<(f64, &str)> = self
            .remembered
            .iter()
            .filter(|(text, _)| remembered::letters(text, alphabet).starts_with(prefix))
            .map(|(text, d)| (d.at(tick), text.as_str()))
            .collect();
        found.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(b.1)));
        found.into_iter().map(|(_, text)| text).collect()
    }

    /// 方案 `scheme` 的個人模型，沒有就建一個空的。
    pub(crate) fn for_scheme_mut(&mut self, scheme: &str) -> &mut SchemeModel {
        self.schemes.entry(scheme.to_string()).or_default()
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
        check_user_word(engine, text, trigger, |_| false)?;
        let code = manual_word_code(engine, text, trigger).ok_or(UserWordError::NoCode)?;
        self.insert_manual_word(engine, &code, text, trigger);
        Ok(())
    }

    /// 把（`code`, `text`）記成手動加的使用者詞，已有的換上這個觸發詞；不檢查，檢查由呼叫的
    /// [`UserModel::add_word`] 或詞表匯入做。
    pub(crate) fn insert_manual_word(
        &mut self,
        engine: &Engine,
        code: &str,
        text: &str,
        trigger: Option<&str>,
    ) {
        self.for_scheme_mut(&engine.scheme().id)
            .words_at(code)
            .insert(
                text.to_string(),
                UserWord {
                    source: WordSource::Manual,
                    trigger: trigger.map(String::from),
                },
            );
    }

    /// 存一條常用語（plan §2.4）：可以長、可以含換行與 Tab；`trigger` 是打它就讓它成首選的
    /// 輸入碼，只能用方案的鍵，沒有時只能從清單選。同一個方案裡同樣的文字只存一條：再存一次
    /// 換上新的觸發詞，舊觸發詞學到的計數一起刪掉。
    pub fn add_phrase(
        &mut self,
        engine: &Engine,
        text: &str,
        trigger: Option<&str>,
    ) -> Result<(), UserWordError> {
        check_user_word(engine, text, trigger, |c| c == '\n' || c == '\t')?;
        let scheme = engine.scheme();
        let model = self.for_scheme_mut(&scheme.id);
        let old = model.words_at(PHRASE_CODE).insert(
            text.to_string(),
            UserWord {
                source: WordSource::Phrase,
                trigger: trigger.map(String::from),
            },
        );
        if let Some(old) = old.and_then(|w| w.trigger)
            && Some(old.as_str()) != trigger
        {
            model.forget(&old, text);
        }
        Ok(())
    }

    /// 這個方案的常用語，依文字排（同 iOS 文字替代的清單依字母排）。
    pub fn phrases(&self, engine: &Engine) -> Vec<Phrase> {
        let model = self.for_scheme(&engine.scheme().id);
        model
            .words
            .get(PHRASE_CODE)
            .into_iter()
            .flatten()
            .map(|(text, word)| Phrase {
                text: text.clone(),
                trigger: word.trigger.clone(),
            })
            .collect()
    }

    /// 刪掉文字是 `text` 的常用語，連同它的觸發詞學到的計數，匯出檔裡不再有它；沒有這條時回傳
    /// `false`。觸發詞剛好是這段文字自己的碼時，平常打字學到的同一組計數也一起刪：兩者是同一個鍵、
    /// 分不出來，寧可少一筆學習，也不留使用者刪掉的字。
    pub fn remove_phrase(&mut self, engine: &Engine, text: &str) -> bool {
        // 沒學過的方案不建出空的方案（`for_scheme_mut` 會建）。
        let Some(model) = self.schemes.get_mut(&engine.scheme().id) else {
            return false;
        };
        let Some(phrases) = model.words.get_mut(PHRASE_CODE) else {
            return false;
        };
        let Some(word) = phrases.remove(text) else {
            return false;
        };
        if phrases.is_empty() {
            model.words.remove(PHRASE_CODE);
        }
        if let Some(trigger) = word.trigger {
            model.forget(&trigger, text);
        }
        true
    }
}

/// 一條常用語（[`UserModel::phrases`]）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phrase {
    pub text: String,
    /// 打它就讓這條常用語成首選的輸入碼；沒有時只能從清單選。
    pub trigger: Option<String>,
}

/// 手動加的詞的輸入碼（plan §2.4）：各字的碼接起來；有字沒有碼時用觸發詞，都沒有時是 `None`。
pub(crate) fn manual_word_code(
    engine: &Engine,
    text: &str,
    trigger: Option<&str>,
) -> Option<String> {
    engine.code_of(text).or_else(|| trigger.map(String::from))
}

/// 使用者詞與常用語共同的檢查：文字不能空、不能含控制字元（`allowed` 放行的除外），觸發詞只能用
/// 方案的鍵。
fn check_user_word(
    engine: &Engine,
    text: &str,
    trigger: Option<&str>,
    allowed: impl Fn(char) -> bool,
) -> Result<(), UserWordError> {
    if text.is_empty() {
        return Err(UserWordError::EmptyWord);
    }
    if text.chars().any(|c| c.is_control() && !allowed(c)) {
        return Err(UserWordError::ControlCharacter);
    }
    if let Some(trigger) = trigger
        && (trigger.is_empty() || !engine.scheme().spells(trigger))
    {
        return Err(UserWordError::InvalidTrigger(trigger.to_string()));
    }
    Ok(())
}

impl Bigrams {
    /// 一次上屏的個人 bigram：`previous` → 第一段、相鄰兩段各記一次；有兩段以上時 `previous` →
    /// 整串也記（整串可能是自動造詞的使用者詞，同 [`SchemeModel::learn`] 整串和各段都記）。
    /// 同一組只記一次。
    fn learn(&mut self, previous: Option<&str>, segments: &[Segment]) {
        self.tick += 1;
        let tick = self.tick;
        let whole: String = segments.iter().map(|s| s.text.as_str()).collect();
        let mut pairs: Vec<(&str, &str)> = Vec::new();
        if let Some(previous) = previous {
            if segments.len() > 1 {
                pairs.push((previous, &whole));
            }
            pairs.push((previous, &segments[0].text));
        }
        pairs.extend(
            segments
                .windows(2)
                .map(|w| (w[0].text.as_str(), w[1].text.as_str())),
        );
        let mut seen = BTreeSet::new();
        for pair in pairs {
            if !seen.insert(pair) {
                continue;
            }
            let (previous, text) = pair;
            let decayed = self
                .entries
                .entry(previous.to_string())
                .or_default()
                .entry(text.to_string())
                .or_default();
            decayed.bump(tick);
        }
    }
}

impl SchemeModel {
    /// 刪掉（`code`, `text`）學到的計數。
    fn forget(&mut self, code: &str, text: &str) {
        if let Some(texts) = self.entries.get_mut(code) {
            texts.remove(text);
            if texts.is_empty() {
                self.entries.remove(code);
            }
        }
    }

    /// 碼 `code` 的使用者詞，沒有就建一個空的。
    fn words_at(&mut self, code: &str) -> &mut BTreeMap<String, UserWord> {
        self.words.entry(code.to_string()).or_default()
    }

    /// 一次上屏：整串輸入碼和每一段的衰減值各記一次，（碼, 字詞）相同的只記一次；整串另記一次
    /// 整串上屏（plan §2.4）。`segments` 依輸入碼的順序、接起來是整串，只看碼與字詞，不是空的。
    fn learn(&mut self, segments: &[Segment]) {
        self.tick += 1;
        let tick = self.tick;
        let whole = Segment::new(
            segments.iter().map(|s| s.code.as_str()).collect(),
            segments.iter().map(|s| s.text.as_str()).collect(),
        );
        let mut keys: Vec<&Segment> = vec![&whole];
        for segment in segments {
            if !keys
                .iter()
                .any(|k| k.code == segment.code && k.text == segment.text)
            {
                keys.push(segment);
            }
        }
        for (i, segment) in keys.into_iter().enumerate() {
            let count = self
                .entries
                .entry(segment.code.clone())
                .or_default()
                .entry(segment.text.clone())
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

    /// 觸發詞剛好是 `input` 的使用者詞（排位規則 1）；有幾個時，以 `input` 最近整串上屏的在前，
    /// 其餘依（碼, 詞）的順序。
    pub(crate) fn triggered(&self, input: &str) -> Vec<&str> {
        let mut found: Vec<(u64, &str)> = self
            .words
            .values()
            .flatten()
            .filter(|(_, word)| word.trigger.as_deref() == Some(input))
            .map(|(text, _)| {
                let tick = self.count(input, text).map_or(0, |c| c.last_commit_tick);
                (tick, text.as_str())
            })
            .collect();
        found.sort_by_key(|&(tick, _)| std::cmp::Reverse(tick));
        found.into_iter().map(|(_, text)| text).collect()
    }

    /// 自動造詞（plan §2.4）：把（`code`, `text`）記成使用者詞；已經是使用者詞的不動。
    /// 上屏次數由同一次上屏的 [`SchemeModel::learn`] 記。
    pub(crate) fn add_auto_phrase(&mut self, code: &str, text: &str) {
        self.words_at(code)
            .entry(text.to_string())
            .or_insert(UserWord {
                source: WordSource::AutoPhrase,
                trigger: None,
            });
    }

    /// （`code`, `text`）是不是使用者詞（任何來源）。
    pub(crate) fn is_word(&self, code: &str, text: &str) -> bool {
        self.words.get(code).is_some_and(|w| w.contains_key(text))
    }

    /// 碼剛好是 `code` 的使用者詞。
    pub(crate) fn words(&self, code: &str) -> impl Iterator<Item = &str> {
        self.words
            .get(code)
            .into_iter()
            .flat_map(|words| words.keys().map(String::as_str))
    }

    /// 最長的使用者詞的碼長；lattice 往回找詞的上限要算進它。
    pub(crate) fn longest_word_code(&self) -> usize {
        self.words.keys().map(String::len).max().unwrap_or(0)
    }

    /// （碼, 字詞）整串上屏的次數；排位規則 2 的「剛好 1 次的不靠分數成首選」用。
    pub(crate) fn whole_commits(&self, code: &str, text: &str) -> u64 {
        self.count(code, text).map_or(0, |count| count.commits)
    }

    /// 排位規則 3（plan §2.4）：（`code`, `text`）是使用者詞且整串上屏過時，回傳最後一次整串
    /// 上屏的 tick，越大越近；否則是 `None`。
    pub(crate) fn last_tick_if_used_word(&self, code: &str, text: &str) -> Option<u64> {
        self.words.get(code)?.get(text)?;
        let count = self.count(code, text)?;
        (count.commits >= 1).then_some(count.last_commit_tick)
    }

    /// 這組（碼, 字詞）學過沒有。
    pub(crate) fn has_learned(&self, code: &str, text: &str) -> bool {
        self.count(code, text).is_some()
    }

    /// 排位規則 2（plan §2.4）：整串上屏 ≥ 2 次且距最後一次整串上屏不超過 [`RECENT_COMMITS`]
    /// 次上屏時，回傳那次的 tick，越大越近；否則是 `None`。句中各段的上屏不算。
    pub(crate) fn last_tick_if_chosen_twice(&self, code: &str, text: &str) -> Option<u64> {
        let count = self.count(code, text)?;
        (count.commits >= 2 && self.tick - count.last_commit_tick <= RECENT_COMMITS)
            .then_some(count.last_commit_tick)
    }

    /// 個人機率（plan §2.3）：衰減值衰減到現在，除以它的上限；沒學過時是 0。
    pub(crate) fn personal_probability(&self, code: &str, text: &str) -> f64 {
        self.count(code, text).map_or(0.0, |count| {
            count.decayed * decay(self.tick - count.last_tick) / max_decayed()
        })
    }

    fn count(&self, code: &str, text: &str) -> Option<&Count> {
        self.entries.get(code)?.get(text)
    }
}

/// 匯出檔的第一行（plan §2.4）；格式改了要升版本（brief §5.3）。v2 的詞欄有跳脫，可以有常用語。
const HEADER: &str = "# qim-userdict v2";
/// 舊版的第一行；v1 的詞欄照字面讀、沒有常用語，匯入一直要收（M3 spec〈共同規則〉）。
const HEADER_V1: &str = "# qim-userdict v1";

/// 匯入的檔是哪一版，由第一行決定。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Version {
    V1,
    V2,
}
/// 方案 tick 的 metadata 行開頭，後接方案 id、Tab、tick；照 librime userdb 文字檔的
/// `#@/tick`（`docs/research/userdict-text-formats.md`）。
const TICK_KEY: &str = "#@/tick/";
/// 個人 bigram 的 tick 的 metadata 行開頭，後接 Tab、tick；個人 bigram 各方案共用，所以不帶方案。
const SHARED_TICK_KEY: &str = "#@/tick\t";
/// 欄名註解行；匯入時和其他 `#` 開頭的行一樣略過。
const COLUMNS: &str =
    "# scheme\tword\tcode\tcommits\tlast_commit_tick\tdecayed\tlast_tick\tsource\ttrigger";
/// 個人 bigram 的欄名註解行，有個人 bigram 時寫在它們前面。方案欄是空的：各方案共用。
const BIGRAM_COLUMNS: &str = "# \tprevious\tword\tdecayed\tlast_tick";
/// 記住的字串的欄名註解行，有記住的字串時寫在它們前面。方案欄、前一個字詞欄都是空的。
const STRING_COLUMNS: &str = "# \t\tstring\tdecayed\tlast_tick";

impl UserModel {
    /// 匯出成 `# qim-userdict v2` 的 TSV（plan §2.4），給殼存檔、換機或備份。
    /// 每個（方案, 輸入碼, 詞）一列，依方案、輸入碼、詞的順序；接著每個個人 bigram 一列，方案欄
    /// 是空的，依前一個字詞、這一段的順序。詞欄與前一個字詞經 [`escape`]。
    pub fn export(&self) -> String {
        let mut out = format!("{HEADER}\n");
        if self.bigrams.tick > 0 {
            out.push_str(&format!("{SHARED_TICK_KEY}{}\n", self.bigrams.tick));
        }
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
                let text = escape(text);
                out.push_str(&format!(
                    "{scheme}\t{text}\t{code}\t{}\t{}\t{}\t{}\t{source}\t{trigger}\n",
                    count.commits, count.last_commit_tick, count.decayed, count.last_tick,
                ));
            }
        }
        if !self.bigrams.entries.is_empty() {
            out.push_str(BIGRAM_COLUMNS);
            out.push('\n');
        }
        for (previous, next) in &self.bigrams.entries {
            for (text, decayed) in next {
                out.push_str(&format!(
                    "\t{}\t{}\t{}\t{}\n",
                    escape(previous),
                    escape(text),
                    decayed.value,
                    decayed.last_tick,
                ));
            }
        }
        if !self.remembered.is_empty() {
            out.push_str(STRING_COLUMNS);
            out.push('\n');
        }
        for (text, decayed) in &self.remembered {
            out.push_str(&format!(
                "\t\t{}\t{}\t{}\n",
                escape(text),
                decayed.value,
                decayed.last_tick,
            ));
        }
        out
    }

    /// 讀 [`UserModel::export`] 的檔（v2，也收 v1），取代整個個人模型。第一行不是兩者之一、
    /// 或有一列不合格式時整份不收，錯誤帶行號。
    pub fn import(text: &str) -> Result<UserModel, ImportError> {
        let mut lines = text
            .lines()
            .map(|l| l.strip_suffix('\r').unwrap_or(l))
            .zip(1..);
        let version = match lines.next().map(|(l, _)| l) {
            Some(HEADER) => Version::V2,
            Some(HEADER_V1) => Version::V1,
            _ => {
                return Err(ImportError::at(
                    1,
                    format!("第一行要是「{HEADER}」或「{HEADER_V1}」"),
                ));
            }
        };
        let mut user = UserModel::default();
        let mut ticks: BTreeMap<&str, u64> = BTreeMap::new();
        let mut shared_tick: Option<u64> = None;
        let mut seen: BTreeSet<(&str, &str, Cow<str>)> = BTreeSet::new();
        for (line, number) in lines {
            let error = |reason: &str| ImportError::at(number, reason.to_string());
            // v1 沒有共用的 tick，這行照其他 `#` 開頭的行是註解。
            if let Some(tick) = line.strip_prefix(SHARED_TICK_KEY)
                && version == Version::V2
            {
                let tick = tick.parse().map_err(|_| error("tick 不是整數"))?;
                // 讀到這裡時，個人 bigram 的 tick 是前面各列最大的最後 tick。
                if user.bigrams.tick > tick {
                    return Err(error("tick 比前面個人 bigram 列的最後 tick 小"));
                }
                shared_tick = Some(tick);
                continue;
            }
            // 方案欄是空的：個人 bigram 或記住的字串（v2 起）。
            if let Some(shared) = line.strip_prefix('\t') {
                if version == Version::V1 {
                    return Err(error("方案不能是空的（v1 沒有個人 bigram 與記住的字串）"));
                }
                // 前一個字詞欄也是空的：記住的字串。
                if let Some(string) = shared.strip_prefix('\t') {
                    let (text, decayed) = parse_string(string).map_err(error)?;
                    if shared_tick.is_some_and(|t| decayed.last_tick > t) {
                        return Err(error("最後 tick 比共用的 tick 大"));
                    }
                    if user.remembered.insert(text.into_owned(), decayed).is_some() {
                        return Err(error("記住的字串重複"));
                    }
                    if user.remembered.len() > REMEMBERED_STRINGS_MAX {
                        return Err(ImportError::at(
                            number,
                            format!("記住的字串超過 {REMEMBERED_STRINGS_MAX} 條"),
                        ));
                    }
                    user.bigrams.tick = user.bigrams.tick.max(decayed.last_tick);
                    continue;
                }
                let (previous, text, decayed) = parse_bigram(shared).map_err(error)?;
                if shared_tick.is_some_and(|t| decayed.last_tick > t) {
                    return Err(error("最後 tick 比個人 bigram 的 tick 大"));
                }
                let next = user
                    .bigrams
                    .entries
                    .entry(previous.into_owned())
                    .or_default();
                if next.insert(text.into_owned(), decayed).is_some() {
                    return Err(error("同前一個字詞、同一段的個人 bigram 重複"));
                }
                user.bigrams.tick = user.bigrams.tick.max(decayed.last_tick);
                continue;
            }
            if let Some(tick) = line.strip_prefix(TICK_KEY) {
                let (scheme, tick) = tick
                    .split_once('\t')
                    .ok_or_else(|| error("tick 行少了 Tab"))?;
                let tick = tick.parse().map_err(|_| error("tick 不是整數"))?;
                // 讀到這裡時，方案的 tick 是前面各列最大的最後 tick。
                if user.schemes.get(scheme).is_some_and(|m| m.tick > tick) {
                    return Err(error("tick 比前面這個方案的列的最後 tick 小"));
                }
                ticks.insert(scheme, tick);
                continue;
            }
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let row = Row::parse(line, version).map_err(error)?;
            let tick = ticks.get(row.scheme).copied();
            if tick.is_some_and(|t| row.count.last_tick.max(row.count.last_commit_tick) > t) {
                return Err(error("最後 tick 比方案的 tick 大"));
            }
            if !seen.insert((row.scheme, row.code, row.text.clone())) {
                return Err(error("同方案、同輸入碼、同詞的列重複"));
            }
            let model = user.for_scheme_mut(row.scheme);
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
            // 沒有 tick 行時，方案的 tick 取最大的最後 tick（兩種最後 tick 都算，否則算期限時相減會小於 0）。
            model.tick = model
                .tick
                .max(row.count.last_tick)
                .max(row.count.last_commit_tick);
        }
        // 只剩 tick 行、沒有資料列的方案（資料列都刪光了）也要還原。
        for (scheme, &tick) in &ticks {
            user.for_scheme_mut(scheme).tick = tick;
        }
        // 沒有 tick 行時，個人 bigram 的 tick 取最大的最後 tick（同方案）。
        if let Some(tick) = shared_tick {
            user.bigrams.tick = tick;
        }
        Ok(user)
    }
}

/// 匯出檔的一列。
struct Row<'a> {
    scheme: &'a str,
    /// 已還原跳脫。
    text: Cow<'a, str>,
    code: &'a str,
    count: Count,
    word: Option<UserWord>,
}

impl<'a> Row<'a> {
    /// v1 的詞欄照字面讀，沒有常用語。
    fn parse(line: &'a str, version: Version) -> Result<Self, &'static str> {
        let v1 = version == Version::V1;
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
        if scheme.is_empty() || text.is_empty() {
            return Err("方案、詞不能是空的");
        }
        let text = if v1 {
            Cow::Borrowed(text)
        } else {
            unescape(text)?
        };
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
            .filter(|&(s, _)| !(v1 && s == Some(WordSource::Phrase)))
            .find(|&(_, name)| name == source)
            .ok_or(if v1 {
                "來源要是 manual、learned 或 auto"
            } else {
                "來源要是 manual、learned、auto 或 phrase"
            })?;
        let phrase = source == Some(WordSource::Phrase);
        if phrase != (code == PHRASE_CODE) {
            return Err("常用語（phrase）的輸入碼是空的，其他列不能是空的");
        }
        if phrase && count != Count::default() {
            return Err("常用語（phrase）沒有計數，觸發詞學到的另是一列 learned");
        }
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

/// 個人 bigram 的一列去掉開頭的 Tab（空的方案欄）之後：前一個字詞、這一段、衰減值、最後 tick。
fn parse_bigram(line: &str) -> Result<(Cow<'_, str>, Cow<'_, str>, Decayed), &'static str> {
    let fields: Vec<&str> = line.split('\t').collect();
    let [previous, text, value, last_tick] = fields[..] else {
        return Err("個人 bigram（方案欄空的）要有 5 欄，以 Tab 分隔");
    };
    if previous.is_empty() || text.is_empty() {
        return Err("個人 bigram 的前一個字詞、這一段不能是空的");
    }
    let value: f64 = value.parse().map_err(|_| "衰減值要是數字")?;
    if !value.is_finite() || value <= 0.0 {
        return Err("個人 bigram 的衰減值要是正的有限數");
    }
    let last_tick = last_tick.parse().map_err(|_| "tick 要是非負整數")?;
    Ok((
        unescape(previous)?,
        unescape(text)?,
        Decayed { value, last_tick },
    ))
}

/// 記住的字串的一列去掉開頭兩個 Tab（空的方案欄與前一個字詞欄）之後：字串、衰減值、最後 tick。
/// 字串要是 email（同打字時記的規則），照詞欄跳脫。
fn parse_string(line: &str) -> Result<(Cow<'_, str>, Decayed), &'static str> {
    let fields: Vec<&str> = line.split('\t').collect();
    let [text, value, last_tick] = fields[..] else {
        return Err("記住的字串（方案欄、前一個字詞欄空的）要有 5 欄，以 Tab 分隔");
    };
    let text = unescape(text)?;
    if remembered::to_remember(&text) != Some(&text) {
        return Err("記住的字串要是 email");
    }
    let value: f64 = value.parse().map_err(|_| "衰減值要是數字")?;
    if !value.is_finite() || value <= 0.0 {
        return Err("記住的字串的衰減值要是正的有限數");
    }
    let last_tick = last_tick.parse().map_err(|_| "tick 要是非負整數")?;
    Ok((text, Decayed { value, last_tick }))
}

/// 詞欄的跳脫（v2）：反斜線、換行、Tab 寫成 `\\`、`\n`、`\t`，照 PostgreSQL `COPY` 的文字格式與
/// Linear TSV；常用語可以含換行與 Tab，TSV 放不下原字。
fn escape(text: &str) -> Cow<'_, str> {
    if !text.contains(['\\', '\n', '\t']) {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len() + 2);
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    Cow::Owned(out)
}

/// [`escape`] 的反向；反斜線後不是 `\\`、`n`、`t` 時是錯的。
fn unescape(field: &str) -> Result<Cow<'_, str>, &'static str> {
    if !field.contains('\\') {
        return Ok(Cow::Borrowed(field));
    }
    let mut out = String::with_capacity(field.len());
    let mut chars = field.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        out.push(match chars.next() {
            Some('\\') => '\\',
            Some('n') => '\n',
            Some('t') => '\t',
            _ => return Err("詞欄的反斜線後面要是 \\、n 或 t"),
        });
    }
    Ok(Cow::Owned(out))
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

/// `personal`（plan §2.3）：個人機率在一段原本的 ln 機率 `log_static` 上加的分，只加不減。個人機率
/// 由這一段的學習計數 `unigram`（[`SchemeModel::personal_probability`]）與它接在前一個字詞後面的
/// 個人 bigram `bigram`（[`Following::probability`]）內插；兩者都是 0（沒學過）時是 0。
pub(crate) fn personal_bonus(log_static: f64, unigram: f64, bigram: f64) -> f64 {
    let p = unigram + (1.0 - unigram) * BIGRAM_WEIGHT * bigram;
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
