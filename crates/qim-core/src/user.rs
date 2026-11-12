//! 個人模型（plan §2.4）：使用者詞與學習計數。

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::english_user::{self, ENGLISH_COMPLETION_AFTER, EnglishUserWord, EnglishUserWords};
use crate::{Date, Engine, Segment};

mod undo;
pub(crate) use undo::{Undo, UndoOwner};

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

/// 學習的時限（plan §2.4，M4 票 13）：最後使用的日期距今天超過這麼多天就算一年沒用。整串上屏隔了
/// 這麼久從 1 重算、規則 2、3 不再用它，一年沒用的條目停用（不刪，[`SchemeUser`]）。產品決定（使用者 2026-11-12），不由基準調；
/// LatinIME 是 300 天。
const EXPIRY_DAYS: i32 = 365;

/// 日期 `last` 距 `today` 超過 [`EXPIRY_DAYS`]；任一個不知道（殼沒給日期）時不算，時限跳過它。
pub(crate) fn expired(last: Option<Date>, today: Option<Date>) -> bool {
    matches!((last, today), (Some(last), Some(today)) if today.days_after(last) > EXPIRY_DAYS)
}

/// 學習資料的條目上限（plan §2.4，M4 票 13），照 LatinIME 的結構：條目數到上限的 1.2 倍才清到上限，
/// 不必每次學習都淘汰。LatinIME 分 unigram（1 萬）與 bigram（3 萬）兩個上限，這裡也分兩個。
/// LatinIME 的上限可由字典檔頭覆寫（`MAX_UNIGRAM_ENTRY_COUNT`），這裡由殼用 [`UserModel::set_limits`] 改。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// （方案, 輸入碼, 詞）的列：學習計數與自動造詞，各方案合計；手動加詞與常用語不算、不刪。
    pub entries: usize,
    /// 個人 bigram 的組數。
    pub bigrams: usize,
}

impl Limits {
    /// 到這個數就清到 `limit`：LatinIME 的 `getEntryCountHardLimit`（上限 × 1.2）。
    fn hard(limit: usize) -> usize {
        limit + limit / 5
    }
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            entries: ENTRY_LIMIT,
            bigrams: BIGRAM_LIMIT,
        }
    }
}

/// [`Limits::entries`] 的預設值：學完全部公開集合（約 9,700 句）是約 1.9 萬列，「學過」的每鍵延遲就是量
/// 這個大小；使用者的 Rime 資料約 4,600 條。量法見 M4 票 13 的 Comments。
const ENTRY_LIMIT: usize = 20_000;
/// [`Limits::bigrams`] 的預設值：同上約 2.8 萬組；LatinIME 的 bigram 上限也是 3 萬。
const BIGRAM_LIMIT: usize = 30_000;

/// 使用者詞與學習計數；殼負責保存（plan §2.1）。各輸入方案分開記，互不影響（plan §2.4）。
#[derive(Debug, Clone, Default)]
pub struct UserModel {
    schemes: BTreeMap<String, SchemeModel>,
    /// 個人 bigram：只看文字，各方案共用（plan §2.4）。
    bigrams: Bigrams,
    /// 英文使用者詞（plan §2.4，M4 票 14）：各方案共用，有自己的 tick。
    english: EnglishUserWords,
    /// 殼交給核心的今天（[`UserModel::set_today`]）：學到或改到的條目記成最後使用的日期。核心不讀
    /// 系統時間（plan §1.3）；殼還沒給時是 `None`，這時學到的條目沒有日期。
    today: Option<Date>,
    /// 這台裝置的 id（[`UserModel::set_device`]），匯出時寫進檔頭；跨裝置合併時分辨各台（M4 票 17）。
    device: Option<String>,
    /// 條目上限（[`UserModel::set_limits`]）；不進匯出檔。
    limits: Limits,
    /// 最近一個動作學到的東西的撤銷紀錄（退格撤銷，[`UserModel::take_undo`]）；不進匯出檔，比較時不看。
    undo: Undo,
}

/// 比較學習資料與殼給的設定；撤銷紀錄不是學習資料（匯出檔也沒有它），不比。
impl PartialEq for UserModel {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            schemes,
            bigrams,
            english,
            today,
            device,
            limits,
            undo: _,
        } = self;
        *schemes == other.schemes
            && *bigrams == other.bigrams
            && *english == other.english
            && *today == other.today
            && *device == other.device
            && *limits == other.limits
    }
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
pub(crate) struct Decayed {
    value: f64,
    last_tick: u64,
    /// 最後使用的日期；殼沒給日期時是 `None`。
    pub(crate) last_used: Option<Date>,
}

impl Decayed {
    /// 最後更新的 tick。
    pub(crate) fn last_tick(self) -> u64 {
        self.last_tick
    }

    /// 衰減到 `tick` 的值。
    pub(crate) fn at(self, tick: u64) -> f64 {
        self.value * decay(tick - self.last_tick)
    }

    /// 在 `tick`、`today` 又上屏（或打）一次：衰減到現在再加一（librime `formula_d`）。
    pub(crate) fn bump(&mut self, tick: u64, today: Option<Date>) {
        self.value = 1.0 + self.at(tick);
        self.last_tick = tick;
        self.last_used = today;
    }
}

/// 接在一個前一個字詞後面的個人 bigram（[`UserModel::bigrams_after`]）。
#[derive(Debug, Clone, Copy)]
pub(crate) struct Following<'a> {
    next: &'a BTreeMap<String, Decayed>,
    tick: u64,
    /// 殼給的今天：一年沒用的組停用（當作沒學過）。
    today: Option<Date>,
    /// 這個字詞後面各段的衰減值衰減到現在的總和：條件機率的分母。
    total: f64,
}

impl Following<'_> {
    /// `text` 接在這個字詞後面的個人機率；沒學過是 0。
    pub(crate) fn probability(&self, text: &str) -> f64 {
        self.next
            .get(text)
            .filter(|d| !expired(d.last_used, self.today))
            .map_or(0.0, |d| d.at(self.tick) / self.total)
    }

    /// 學過接在這個字詞後面的各段與它們的個人機率，依文字排序（聯想用）。
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, f64)> + '_ {
        self.next
            .iter()
            .filter(|(_, d)| !expired(d.last_used, self.today))
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
    /// 最後使用的日期：加進來（或改觸發詞）的那天，之後同碼同字學到時跟著計數更新
    /// （[`SchemeModel::stamp`]），所以有計數時和計數的一樣。殼沒給日期時是 `None`。
    last_used: Option<Date>,
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
    /// 最後使用的日期：最後一次（整串或句中）上屏時殼給的今天；殼沒給時是 `None`。一年沒用的條目停用（[`SchemeUser`]）。
    last_used: Option<Date>,
    /// 最後一次整串上屏的日期；`commits` 是 0 或殼沒給日期時是 `None`。距今超過一年時整串上屏的次數
    /// 當作 0（[`Count::commits_on`]）：規則 2、3 只看一年內的整串上屏，句中用過的不延長它。
    last_commit_date: Option<Date>,
}

impl Count {
    /// 沒有計數：手動加了還沒用過的詞匯出時計數都寫 0，匯入時不建計數。日期不算。
    fn is_empty(&self) -> bool {
        self.commits == 0
            && self.last_commit_tick == 0
            && self.decayed == 0.0
            && self.last_tick == 0
    }

    /// `today` 時算數的整串上屏次數：最後一次整串上屏距今超過一年時是 0（下一次從 1 重算，plan §2.4）。
    fn commits_on(&self, today: Option<Date>) -> u64 {
        if expired(self.last_commit_date, today) {
            0
        } else {
            self.commits
        }
    }
}

/// 條目過了出現門檻沒有（上限淘汰先刪沒到的）。中文條目沒有出現門檻，都是 `Passed`；英文使用者詞
/// 一年內打過 [`crate::ENGLISH_SHOWN_AFTER`] 次才算過（UserModel 不知道英文詞典，詞典有的詞打過一次也算沒到）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Threshold {
    Below,
    Passed,
}

/// 上限淘汰的一個條目：（方案, 輸入碼, 詞）的列或英文使用者詞（鍵是小寫）。同序時依這個順序刪。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Victim {
    Row {
        scheme: String,
        code: String,
        text: String,
    },
    English(String),
}

/// 上限淘汰的排序鍵（[`UserModel::enforce_limits`]），[`Eviction::order`] 小的先刪：沒到出現門檻的、
/// 最後使用的日期最舊的（沒有日期的排在有日期的後面）、次數（衰減到現在的衰減值）小的。
#[derive(Debug, Clone, Copy)]
struct Eviction {
    threshold: Threshold,
    last_used: Option<Date>,
    used: f64,
}

impl Eviction {
    fn order(&self, other: &Self) -> std::cmp::Ordering {
        let key = |e: &Self| (e.threshold, e.last_used.is_none(), e.last_used);
        key(self)
            .cmp(&key(other))
            .then(self.used.total_cmp(&other.used))
    }
}

impl UserModel {
    /// 方案 `scheme` 的個人模型，照殼給的今天看（停用一年沒用的條目）；沒記過時是空的。
    pub(crate) fn for_scheme(&self, scheme: &str) -> SchemeUser<'_> {
        static EMPTY: SchemeModel = SchemeModel {
            tick: 0,
            entries: BTreeMap::new(),
            words: BTreeMap::new(),
        };
        SchemeUser {
            model: self.schemes.get(scheme).unwrap_or(&EMPTY),
            today: self.today,
        }
    }

    /// 學一次上屏（plan §2.4）：方案 `scheme` 的學習計數（[`SchemeModel::learn`]），加上個人
    /// bigram。`previous` 是組字前面的一個字詞（前文給的，見 [`crate::Session::set_context`]），
    /// 沒有時第一段不記；`segments` 同 [`SchemeModel::learn`]，空的不記。
    pub(crate) fn learn(&mut self, scheme: &str, previous: Option<&str>, segments: &[Segment]) {
        if segments.is_empty() {
            return;
        }
        let today = self.today;
        let (model, scheme_undo) = self.learning_scheme(scheme);
        model.learn(segments, today, scheme_undo);
        self.bigrams
            .learn(previous, segments, today, &mut self.undo);
        self.enforce_limits();
    }

    /// 選了一個聯想（plan §2.4）：只記個人 bigram「`previous` → `text`」。聯想沒有輸入碼，
    /// 不記學習計數。
    pub(crate) fn learn_association(&mut self, previous: &str, text: &str) {
        self.bigrams.learn(
            Some(previous),
            &[Segment::new(String::new(), text.to_string())],
            self.today,
            &mut self.undo,
        );
        self.enforce_limits();
    }

    /// 退格撤銷（plan §2.4）：`owner` 開始一個動作，收起最近一個動作的撤銷紀錄交給它（不是它寫的時
    /// 是空的），之後學到的記進新的一份。[`crate::Session`] 在每個動作（按鍵、點候選、打文字）開頭
    /// 呼叫，動作結束時決定撤銷它（[`UserModel::revert`]）、留著給下一個動作（[`UserModel::keep_undo`]）
    /// 或丟掉。手動加詞、常用語、刪英文使用者詞、匯入這類使用者明確的操作會丟掉紀錄，撤銷不會蓋掉
    /// 它們的改動。
    pub(crate) fn take_undo(&mut self, owner: UndoOwner) -> Undo {
        let pending = std::mem::replace(&mut self.undo, Undo::new(owner));
        if pending.is_by(owner) {
            pending
        } else {
            Undo::default()
        }
    }

    /// 這個動作不算夾在中間（只收起聯想的退格、修飾鍵，它們不學東西）：前一個動作的紀錄放回去，下一個
    /// 鍵還能撤銷它。
    pub(crate) fn keep_undo(&mut self, undo: Undo) {
        debug_assert!(self.undo.is_empty(), "不算夾在中間的鍵學到了東西");
        self.undo = undo;
    }

    /// `owner` 寫的撤銷紀錄失效（殼的時限到了、換了欄位，[`crate::Session::expire_undo`]）；別的 Session
    /// 寫的不動。
    pub(crate) fn expire_undo(&mut self, owner: UndoOwner) {
        if self.undo.is_by(owner) {
            self.discard_undo();
        }
    }

    /// 丟掉撤銷紀錄，誰都不能撤銷：使用者明確的操作（手動加詞、常用語、刪英文使用者詞、匯入）改了資料，
    /// 撤銷不能蓋掉它們的改動。
    fn discard_undo(&mut self) {
        self.undo = Undo::default();
    }

    /// 撤銷 `undo` 記的學習：改到的條目與 tick 回到那個動作之前。沒學到東西時回傳 `false`。
    pub(crate) fn revert(&mut self, undo: Undo) -> bool {
        if undo.is_empty() {
            return false;
        }
        undo.revert(self);
        true
    }

    /// 學過個人 bigram 沒有。
    pub(crate) fn has_bigrams(&self) -> bool {
        !self.bigrams.entries.is_empty()
    }

    /// 前一個字詞是 `previous` 的個人 bigram；沒學過時是 `None`。
    pub(crate) fn bigrams_after(&self, previous: &str) -> Option<Following<'_>> {
        let next = self.bigrams.entries.get(previous)?;
        let (tick, today) = (self.bigrams.tick, self.today);
        let total: f64 = next
            .values()
            .filter(|d| !expired(d.last_used, today))
            .map(|d| d.at(tick))
            .sum();
        (total > 0.0).then_some(Following {
            next,
            tick,
            today,
            total,
        })
    }

    /// 學一個英文使用者詞（[`crate::Session`] 從送出去的字裡認出來的串，[`english_user::to_learn`]
    /// 過的）：次數與衰減值加一，英文使用者詞自己的 tick 加一。
    pub(crate) fn learn_english(&mut self, text: &str) {
        self.undo
            .save_english(&self.english, &english_user::key(text));
        self.english.learn(text, self.today);
        self.enforce_limits();
    }

    /// 英文使用者詞（最後打的寫法），依小寫排；殼的清單用，使用者可以逐條刪
    /// （[`UserModel::forget_english_user_word`]）。沒到出現門檻的也列出。
    pub fn english_user_words(&self) -> Vec<String> {
        self.english.list()
    }

    /// 忘掉英文使用者詞 `text`（不分大小寫），匯出檔裡不再有它；沒學過時回傳 `false`。
    pub fn forget_english_user_word(&mut self, text: &str) -> bool {
        self.discard_undo();
        self.english.forget(text)
    }

    /// 輸入碼 `input` 剛好是打過 [`crate::ENGLISH_SHOWN_AFTER`] 次的英文使用者詞時是它的寫法（排位規則 4，
    /// [`EnglishUserWords::typed_as`]）。
    pub(crate) fn english_typed_as(&self, input: &str, alphabet: &str) -> Option<&str> {
        self.english.typed_as(input, alphabet, self.today)
    }

    /// 前綴補完（排位規則 5，[`EnglishUserWords::completions`]）；`in_dictionary` 判斷英文詞典有沒有這個詞。
    pub(crate) fn english_completions(
        &self,
        prefix: &str,
        alphabet: &str,
        in_dictionary: impl Fn(&str) -> bool,
    ) -> Vec<&str> {
        self.english
            .completions(prefix, alphabet, self.today, in_dictionary)
    }

    /// 殼交給核心今天的日期（使用者當地的日曆日）；之後學到或改到的條目記成最後使用的日期。核心不讀
    /// 系統時間（plan §1.3）。殼的契約（plan §2.4）：在第一次學習前、之後每次使用者操作前給，換日後
    /// 才不會記成前一天。沒給過時學到的條目沒有日期（不寫假日期），一年的時限跳過它們。
    ///
    /// 距今超過一年沒用的條目停用（不再影響排序）但不刪：殼的時鐘可能錯（重開機後亂掉、被改到幾年後），
    /// 刪了改回來也救不回（使用者 2026-11-12 決定）；時鐘改回來就恢復作用。實體刪除只在到上限時淘汰。
    pub fn set_today(&mut self, today: Date) {
        self.today = Some(today);
    }

    /// 改條目上限（[`Limits`]）；超過新上限 1.2 倍的照樣等下一次學習才清。殼不改時是 [`Limits::default`]。
    pub fn set_limits(&mut self, limits: Limits) {
        self.limits = limits;
    }

    /// 條目數到上限的 1.2 倍時清到上限（plan §2.4，LatinIME 的 `truncateEntries`）。（方案, 輸入碼, 詞）的列
    /// 與英文使用者詞算同一個上限（LatinIME 的 unigram 也不分來源）。淘汰順序：先刪沒到出現門檻的
    /// （[`Threshold`]：只有英文使用者詞會沒到），再刪最後使用的日期最舊的（沒有日期的當作最新，同時限
    /// 跳過它們）、同一天的刪衰減值小的（最近用得少的），再依鍵的順序。手動加詞與常用語不算、不刪。
    fn enforce_limits(&mut self) {
        let Limits { entries, bigrams } = self.limits;
        let rows: usize = self
            .schemes
            .values()
            .map(|m| m.evictable().count())
            .sum::<usize>()
            + self.english.words.len();
        if rows >= Limits::hard(entries) {
            let mut order: Vec<(Eviction, Victim)> = Vec::with_capacity(rows);
            for (scheme, model) in &self.schemes {
                for (code, text) in model.evictable_rows() {
                    let eviction = model.eviction(&code, &text);
                    let scheme = scheme.clone();
                    order.push((eviction, Victim::Row { scheme, code, text }));
                }
            }
            let (today, tick) = (self.today, self.english.tick);
            order.extend(self.english.words.iter().map(|(key, word)| {
                let eviction = Eviction {
                    threshold: if word.passed(today) {
                        Threshold::Passed
                    } else {
                        Threshold::Below
                    },
                    last_used: word.decayed.last_used,
                    used: word.decayed.at(tick),
                };
                (eviction, Victim::English(key.clone()))
            }));
            order.sort_by(|a, b| a.0.order(&b.0).then_with(|| a.1.cmp(&b.1)));
            for (_, victim) in order.into_iter().take(rows - entries) {
                match victim {
                    Victim::Row { scheme, code, text } => {
                        self.undo.save_row(&self.schemes, &scheme, &code, &text);
                        if let Some(model) = self.schemes.get_mut(&scheme) {
                            model.remove_row(&code, &text);
                        }
                    }
                    Victim::English(key) => {
                        self.undo.save_english(&self.english, &key);
                        self.english.words.remove(&key);
                    }
                }
            }
        }
        let pairs: usize = self.bigrams.entries.values().map(BTreeMap::len).sum();
        if pairs >= Limits::hard(bigrams) {
            let tick = self.bigrams.tick;
            let mut order: Vec<(Eviction, String, String)> = self
                .bigrams
                .entries
                .iter()
                .flat_map(|(previous, next)| {
                    next.iter().map(move |(text, d)| {
                        (
                            Eviction {
                                threshold: Threshold::Passed,
                                last_used: d.last_used,
                                used: d.at(tick),
                            },
                            previous.clone(),
                            text.clone(),
                        )
                    })
                })
                .collect();
            order.sort_by(|a, b| {
                a.0.order(&b.0)
                    .then_with(|| (&a.1, &a.2).cmp(&(&b.1, &b.2)))
            });
            let doomed: BTreeSet<(String, String)> = order
                .into_iter()
                .take(pairs - bigrams)
                .map(|(_, previous, text)| (previous, text))
                .collect();
            for (previous, text) in &doomed {
                self.undo.save_bigram(&self.bigrams, previous, text);
            }
            self.bigrams.retain(|previous, text, _| {
                !doomed.contains(&(previous.to_string(), text.to_string()))
            });
        }
    }

    /// 殼最後給的今天（[`UserModel::set_today`]）；沒給過時是 `None`。
    pub fn today(&self) -> Option<Date> {
        self.today
    }

    /// 這台裝置的 id：由殼產生一次、存起來（網頁用 `crypto.randomUUID()`，照 librime 每次安裝一個
    /// `installation_id`），匯出時寫進檔頭。只能用英數、`-`、`_`，1–64 字。
    pub fn set_device(&mut self, id: &str) -> Result<(), InvalidDeviceId> {
        if !is_device_id(id) {
            return Err(InvalidDeviceId(id.to_string()));
        }
        self.device = Some(id.to_string());
        Ok(())
    }

    /// 這台裝置的 id；殼沒給過時是 `None`。匯入的檔讀出來的是那個檔的裝置。
    pub fn device(&self) -> Option<&str> {
        self.device.as_deref()
    }

    /// 拿 `learned`（[`UserModel::import`] 讀出來的檔，或空白的個人模型）取代全部學習資料；殼給的今天與
    /// 這台裝置的 id 留著，不換成檔裡的（檔可能是別台匯出的，換過來各台分開的次數會記錯格）。
    /// 條目上限也留著自己的。很久以前的備份照收：一年沒用的條目停用、不刪。
    pub fn replace_learning(&mut self, learned: UserModel) {
        let today = self.today;
        let device = self.device.take();
        let limits = self.limits;
        *self = UserModel {
            today,
            device,
            limits,
            undo: Undo::default(),
            ..learned
        };
    }

    /// 自動造詞（plan §2.4）：把方案 `scheme` 的（`code`, `text`）記成使用者詞；已經是使用者詞的不動。
    /// 上屏次數由同一次上屏的 [`UserModel::learn`] 記。
    pub(crate) fn add_auto_phrase(&mut self, scheme: &str, code: &str, text: &str) {
        let today = self.today;
        let (model, scheme_undo) = self.learning_scheme(scheme);
        model.add_auto_phrase(code, text, today, scheme_undo);
        self.enforce_limits();
    }

    /// 學習要改方案 `scheme`：它的個人模型（沒有就建一個空的）與這個動作的撤銷紀錄裡它的那份。
    fn learning_scheme(&mut self, scheme: &str) -> (&mut SchemeModel, &mut undo::SchemeUndo) {
        let scheme_undo = self.undo.scheme(&self.schemes, scheme);
        let model = self.schemes.entry(scheme.to_string()).or_default();
        (model, scheme_undo)
    }

    /// 方案 `scheme` 的個人模型，沒有就建一個空的。
    fn for_scheme_mut(&mut self, scheme: &str) -> &mut SchemeModel {
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
        self.discard_undo();
        let today = self.today;
        let model = self.for_scheme_mut(&engine.scheme().id);
        model.words_at(code).insert(
            text.to_string(),
            UserWord {
                source: WordSource::Manual,
                trigger: trigger.map(String::from),
                last_used: today,
            },
        );
        model.stamp(code, text, today);
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
        self.discard_undo();
        let today = self.today;
        let model = self.for_scheme_mut(&engine.scheme().id);
        let old = model.words_at(PHRASE_CODE).insert(
            text.to_string(),
            UserWord {
                source: WordSource::Phrase,
                trigger: trigger.map(String::from),
                last_used: today,
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
        self.schemes
            .get(&engine.scheme().id)
            .and_then(|model| model.words.get(PHRASE_CODE))
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
        self.discard_undo();
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
    /// 只留 `keep` 為真的組；前一個字詞後面都刪光時連它一起刪。
    fn retain(&mut self, mut keep: impl FnMut(&str, &str, &Decayed) -> bool) {
        self.entries.retain(|previous, next| {
            next.retain(|text, decayed| keep(previous, text, decayed));
            !next.is_empty()
        });
    }

    /// 一次上屏的個人 bigram：`previous` → 第一段、相鄰兩段各記一次；有兩段以上時 `previous` →
    /// 整串也記（整串可能是自動造詞的使用者詞，同 [`SchemeModel::learn`] 整串和各段都記）。
    /// 同一組只記一次。
    fn learn(
        &mut self,
        previous: Option<&str>,
        segments: &[Segment],
        today: Option<Date>,
        undo: &mut Undo,
    ) {
        undo.save_bigram_tick(self);
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
            undo.save_bigram(self, previous, text);
            let decayed = self
                .entries
                .entry(previous.to_string())
                .or_default()
                .entry(text.to_string())
                .or_default();
            decayed.bump(tick, today);
        }
    }
}

impl SchemeModel {
    /// 碼 `code`、詞 `text` 的使用者詞。
    fn word(&self, code: &str, text: &str) -> Option<&UserWord> {
        self.words.get(code)?.get(text)
    }

    /// 上限與時限管得到的列（[`Limits::entries`]）：有學習計數或是自動造詞，而且不是手動加詞或常用語
    /// （那兩種的觸發詞學到的計數另是一列學習計數，照樣管）。依（碼, 詞）排。
    fn evictable(&self) -> impl Iterator<Item = (&str, &str)> {
        let counted = self.entries.iter().flat_map(move |(code, texts)| {
            texts
                .keys()
                .map(move |text| (code.as_str(), text.as_str()))
                .filter(|&(code, text)| {
                    self.word(code, text)
                        .is_none_or(|w| w.source == WordSource::AutoPhrase)
                })
        });
        let uncounted_auto = self.words.iter().flat_map(move |(code, words)| {
            words
                .iter()
                .filter(move |(text, w)| {
                    w.source == WordSource::AutoPhrase && self.count(code, text).is_none()
                })
                .map(move |(text, _)| (code.as_str(), text.as_str()))
        });
        counted.chain(uncounted_auto)
    }

    /// [`SchemeModel::evictable`] 的列，複製出來給要邊走邊刪的呼叫端。
    fn evictable_rows(&self) -> Vec<(String, String)> {
        self.evictable()
            .map(|(code, text)| (code.to_string(), text.to_string()))
            .collect()
    }

    /// 一列最後使用的日期：有計數時是計數的（和使用者詞一樣，[`SchemeModel::stamp`]），否則是使用者詞的。
    fn row_last_used(&self, code: &str, text: &str) -> Option<Date> {
        match self.count(code, text) {
            Some(count) => count.last_used,
            None => self.word(code, text).and_then(|w| w.last_used),
        }
    }

    /// 一列在上限淘汰時的順序（[`UserModel::enforce_limits`]）：次數用衰減到現在的衰減值，整串和句中的
    /// 上屏都算；只有使用者詞、沒有計數的是 0。
    fn eviction(&self, code: &str, text: &str) -> Eviction {
        let used = self
            .count(code, text)
            .map_or(0.0, |c| c.decayed * decay(self.tick - c.last_tick));
        Eviction {
            threshold: Threshold::Passed,
            last_used: self.row_last_used(code, text),
            used,
        }
    }

    /// 刪掉一整列：學習計數與使用者詞。
    fn remove_row(&mut self, code: &str, text: &str) {
        self.forget(code, text);
        if let Some(words) = self.words.get_mut(code) {
            words.remove(text);
            if words.is_empty() {
                self.words.remove(code);
            }
        }
    }

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
    fn learn(&mut self, segments: &[Segment], today: Option<Date>, undo: &mut undo::SchemeUndo) {
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
            undo.save(self, &segment.code, &segment.text);
            let count = self
                .entries
                .entry(segment.code.clone())
                .or_default()
                .entry(segment.text.clone())
                .or_default();
            count.decayed = 1.0 + count.decayed * decay(tick - count.last_tick);
            count.last_tick = tick;
            // `keys` 的第一個是整串；距上一次整串上屏超過一年時從 1 重算。
            if i == 0 {
                count.commits = count.commits_on(today) + 1;
                count.last_commit_tick = tick;
                count.last_commit_date = today;
            }
            self.stamp(&segment.code, &segment.text, today);
        }
    }

    /// （`code`, `text`）這天用過或改過：它的計數與使用者詞都記成最後使用的日期 `today`，兩者是匯出檔
    /// 的同一列、只有一個日期。
    fn stamp(&mut self, code: &str, text: &str, today: Option<Date>) {
        if let Some(count) = self.entries.get_mut(code).and_then(|t| t.get_mut(text)) {
            count.last_used = today;
        }
        if let Some(word) = self.words.get_mut(code).and_then(|w| w.get_mut(text)) {
            word.last_used = today;
        }
    }

    /// [`UserModel::add_auto_phrase`]。
    fn add_auto_phrase(
        &mut self,
        code: &str,
        text: &str,
        today: Option<Date>,
        undo: &mut undo::SchemeUndo,
    ) {
        undo.save(self, code, text);
        self.words_at(code)
            .entry(text.to_string())
            .or_insert(UserWord {
                source: WordSource::AutoPhrase,
                trigger: None,
                last_used: today,
            });
        self.stamp(code, text, today);
    }

    fn count(&self, code: &str, text: &str) -> Option<&Count> {
        self.entries.get(code)?.get(text)
    }
}

/// 一個輸入方案的個人模型，照殼給的今天看（[`UserModel::for_scheme`]）：查詢都經它，最後使用的日期距今
/// 超過一年的條目當作沒有（停用，plan §2.4〈學習的時限〉），但留在資料裡，時鐘改回來就恢復作用。
#[derive(Debug, Clone, Copy)]
pub(crate) struct SchemeUser<'a> {
    model: &'a SchemeModel,
    today: Option<Date>,
}

impl<'a> SchemeUser<'a> {
    /// 沒停用的計數。
    fn count(&self, code: &str, text: &str) -> Option<&'a Count> {
        self.model
            .count(code, text)
            .filter(|c| !expired(c.last_used, self.today))
    }

    /// 沒停用的使用者詞：手動加詞與常用語一直有效，自動造詞照它那列最後使用的日期。
    fn active(&self, code: &str, text: &str, word: &UserWord) -> bool {
        word.source != WordSource::AutoPhrase
            || !expired(self.model.row_last_used(code, text), self.today)
    }

    /// 觸發詞剛好是 `input` 的使用者詞（排位規則 1）；有幾個時，以 `input` 最近整串上屏的在前，
    /// 其餘依（碼, 詞）的順序。
    pub(crate) fn triggered(&self, input: &str) -> Vec<&'a str> {
        let mut found: Vec<(u64, &str)> = self
            .model
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

    /// （`code`, `text`）是不是使用者詞（任何來源，沒停用的）。
    pub(crate) fn is_word(&self, code: &str, text: &str) -> bool {
        self.model
            .word(code, text)
            .is_some_and(|w| self.active(code, text, w))
    }

    /// 碼剛好是 `code` 的使用者詞（沒停用的）。
    pub(crate) fn words(self, code: &str) -> impl Iterator<Item = &'a str> + use<'a> {
        let this = self;
        self.model
            .words
            .get_key_value(code)
            .into_iter()
            .flat_map(move |(code, words)| {
                words
                    .iter()
                    .filter(move |(text, w)| this.active(code, text, w))
                    .map(|(text, _)| text.as_str())
            })
    }

    /// 有沒有以 `prefix` 開頭的使用者詞的碼（萬用鍵展開時剪枝用；停用的也算，只是少剪一點，
    /// 實際列詞仍經 [`SchemeUser::words`] 濾掉停用的）。
    pub(crate) fn has_word_code_prefix(&self, prefix: &str) -> bool {
        crate::code_table::has_key_with_prefix(&self.model.words, prefix)
    }

    /// 最長的使用者詞的碼長；lattice 往回找詞的上限要算進它（停用的也算，上限只是放寬）。
    pub(crate) fn longest_word_code(&self) -> usize {
        self.model.words.keys().map(String::len).max().unwrap_or(0)
    }

    /// （碼, 字詞）算數的整串上屏次數（一年內，[`Count::commits_on`]）；排位規則 2 的「剛好 1 次的不靠
    /// 分數成首選」用。
    pub(crate) fn whole_commits(&self, code: &str, text: &str) -> u64 {
        self.count(code, text)
            .map_or(0, |count| count.commits_on(self.today))
    }

    /// 排位規則 3（plan §2.4）：（`code`, `text`）是使用者詞且一年內整串上屏過時，回傳最後一次整串
    /// 上屏的 tick，越大越近；否則是 `None`（一年沒上屏的回到一般排序）。
    pub(crate) fn last_tick_if_used_word(&self, code: &str, text: &str) -> Option<u64> {
        if !self.is_word(code, text) {
            return None;
        }
        let count = self.count(code, text)?;
        (count.commits_on(self.today) >= 1).then_some(count.last_commit_tick)
    }

    /// 這組（碼, 字詞）學過沒有（沒停用的）。
    pub(crate) fn has_learned(&self, code: &str, text: &str) -> bool {
        self.count(code, text).is_some()
    }

    /// 排位規則 2（plan §2.4）：一年內整串上屏 ≥ 2 次（[`Count::commits_on`]），且距最後一次整串
    /// 上屏不超過 [`RECENT_COMMITS`] 次上屏時，回傳那次的 tick，越大越近；否則是 `None`。句中各段的
    /// 上屏不算。
    pub(crate) fn last_tick_if_chosen_twice(&self, code: &str, text: &str) -> Option<u64> {
        let count = self.count(code, text)?;
        (count.commits_on(self.today) >= 2
            && self.model.tick - count.last_commit_tick <= RECENT_COMMITS)
            .then_some(count.last_commit_tick)
    }

    /// 個人機率（plan §2.3）：衰減值衰減到現在，除以它的上限；沒學過或停用時是 0。
    pub(crate) fn personal_probability(&self, code: &str, text: &str) -> f64 {
        self.count(code, text).map_or(0.0, |count| {
            count.decayed * decay(self.model.tick - count.last_tick) / max_decayed()
        })
    }
}

/// 匯出檔的第一行（plan §2.4）；格式改了要升版本（brief §5.3）。v3 每列開頭是種類、帶最後使用的
/// 日期，檔頭可有裝置 id；M4 收尾前還沒定稿，M4 的票可以往裡加種類與欄位（M4 spec〈共同規則〉）。
const HEADER: &str = "# qim-userdict v3";
/// M3 定稿的第一行：沒有種類欄，列的種類由空的欄分辨；詞欄有跳脫、可以有常用語，沒有日期。
const HEADER_V2: &str = "# qim-userdict v2";
/// M1 定稿的第一行：詞欄照字面讀、沒有常用語與個人 bigram。v1、v2 匯入一直要收（M4 spec〈共同規則〉）。
const HEADER_V1: &str = "# qim-userdict v1";

/// 匯入的檔是哪一版，由第一行決定。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Version {
    V1,
    V2,
    V3,
}
/// 方案 tick 的 metadata 行開頭，後接方案 id、Tab、tick；照 librime userdb 文字檔的
/// `#@/tick`（`docs/research/userdict-text-formats.md`）。
const TICK_KEY: &str = "#@/tick/";
/// 個人 bigram 的 tick 的 metadata 行開頭，後接 Tab、tick；個人 bigram 各方案共用，所以不帶方案。
const SHARED_TICK_KEY: &str = "#@/tick\t";
/// 英文使用者詞自己的 tick 的 metadata 行開頭（v3 起，M4 票 14），後接 Tab、tick。
const ENGLISH_TICK_KEY: &str = "#@/english_tick\t";
/// 裝置 id 的 metadata 行開頭（v3 起），後接 Tab、id；照 librime userdb 文字檔的 `#@/user_id`。
const DEVICE_KEY: &str = "#@/device\t";
/// v3 的種類：（方案, 輸入碼, 詞）的學習計數與使用者詞。
const ENTRY: &str = "entry";
/// v3 的種類：個人 bigram。
const BIGRAM: &str = "bigram";
/// v3 的種類：英文使用者詞（M4 票 14）。
const ENGLISH: &str = "english";
/// 欄名註解行；匯入時和其他 `#` 開頭的行一樣略過。
const COLUMNS: &str = "# kind\tscheme\tword\tcode\tcommits\tlast_commit_tick\tdecayed\tlast_tick\tlast_used\tlast_commit_date\tsource\ttrigger";
/// 個人 bigram 的欄名註解行，有個人 bigram 時寫在它們前面。
const BIGRAM_COLUMNS: &str = "# kind\tprevious\tword\tdecayed\tlast_tick\tlast_used";
/// 英文使用者詞的欄名註解行，有英文使用者詞時寫在它們前面。
const ENGLISH_COLUMNS: &str = "# kind\tword\tcount\tdecayed\tlast_tick\tlast_used";
/// 日期欄讀不出來時的原因。
const BAD_DATE: &str = "最後使用的日期要是 YYYY-MM-DD 的真實日期，不知道時留空";

/// v1、v2 的列分不出種類時的分法，與 v3 的種類欄對應；欄位是去掉種類欄（或空的欄）之後的。
enum Kind<'a> {
    Entry(&'a str),
    Bigram(&'a str),
    /// v3 的英文使用者詞。
    English(&'a str),
    /// v2 方案欄、前一個字詞欄都空的列（記住的字串）：匯入成已過門檻的英文使用者詞。
    V2String(&'a str),
}

impl UserModel {
    /// 匯出成 `# qim-userdict v3` 的 TSV（plan §2.4），給殼存檔、換機或備份。每列開頭是種類：
    /// 每個（方案, 輸入碼, 詞）一列 `entry`，依方案、輸入碼、詞的順序；接著個人 bigram 一列一組
    /// `bigram`，依前一個字詞、這一段的順序；再來英文使用者詞 `english`，依小寫的順序。每列都有最後使用的日期欄，
    /// 殼沒給過日期時學到的留空（不寫假日期）。
    /// 詞欄與前一個字詞經 [`escape`]。
    pub fn export(&self) -> String {
        let mut out = format!("{HEADER}\n");
        if let Some(device) = &self.device {
            out.push_str(&format!("{DEVICE_KEY}{device}\n"));
        }
        if self.bigrams.tick > 0 {
            out.push_str(&format!("{SHARED_TICK_KEY}{}\n", self.bigrams.tick));
        }
        if self.english.tick > 0 {
            out.push_str(&format!("{ENGLISH_TICK_KEY}{}\n", self.english.tick));
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
                let count = model.entries.get(code).and_then(|t| t.get(text));
                let word = model.words.get(code).and_then(|w| w.get(text));
                // 有計數時兩者的日期一樣（`SchemeModel::stamp`）；手動加了還沒用過的詞只有詞的日期。
                let last_used = count
                    .map(|c| c.last_used)
                    .or(word.map(|w| w.last_used))
                    .expect("每組鍵不是有計數就是使用者詞");
                let last_used = DateField(last_used);
                let count = count.copied().unwrap_or_default();
                let last_commit_date = DateField(count.last_commit_date);
                let source = word.map(|w| w.source);
                let (_, source) = SOURCE_NAMES
                    .into_iter()
                    .find(|&(s, _)| s == source)
                    .expect("每種來源都有名字");
                let trigger = word.and_then(|w| w.trigger.as_deref()).unwrap_or("");
                let text = escape(text);
                out.push_str(&format!(
                    "{ENTRY}\t{scheme}\t{text}\t{code}\t{}\t{}\t{}\t{}\t{last_used}\t{last_commit_date}\t{source}\t{trigger}\n",
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
                    "{BIGRAM}\t{}\t{}\t{}\t{}\t{}\n",
                    escape(previous),
                    escape(text),
                    decayed.value,
                    decayed.last_tick,
                    DateField(decayed.last_used),
                ));
            }
        }
        if !self.english.words.is_empty() {
            out.push_str(ENGLISH_COLUMNS);
            out.push('\n');
        }
        for word in self.english.words.values() {
            out.push_str(&format!(
                "{ENGLISH}\t{}\t{}\t{}\t{}\t{}\n",
                word.text,
                word.count,
                word.decayed.value,
                word.decayed.last_tick,
                DateField(word.decayed.last_used),
            ));
        }
        out
    }

    /// 讀 [`UserModel::export`] 的檔（v3，也收 v1、v2）成一份新的個人模型，今天是 `today`（殼還沒給
    /// 時是 `None`）。v1、v2 沒有日期，每列的最後使用日期取 `today`（匯入那天）；v3 的日期欄空的是
    /// 不知道。裝置 id 是檔裡的（沒有時是 `None`）；殼拿它取代自己的學習資料用
    /// [`UserModel::replace_learning`]，今天與裝置 id 留著自己的。
    ///
    /// 第一行不是三者之一、或已知種類的列有一列不合格式時整份不收，錯誤帶行號；v3 不認得的種類
    /// （之後的版本加的）略過，回警告帶行號，其他列照收。
    pub fn import(text: &str, today: Option<Date>) -> Result<Imported, ImportError> {
        let mut lines = text
            .lines()
            .map(|l| l.strip_suffix('\r').unwrap_or(l))
            .zip(1..);
        let version = match lines.next().map(|(l, _)| l) {
            Some(HEADER) => Version::V3,
            Some(HEADER_V2) => Version::V2,
            Some(HEADER_V1) => Version::V1,
            _ => {
                return Err(ImportError::at(
                    1,
                    format!("第一行要是「{HEADER}」「{HEADER_V2}」或「{HEADER_V1}」"),
                ));
            }
        };
        let mut user = UserModel {
            today,
            ..UserModel::default()
        };
        let mut warnings = Vec::new();
        let mut ticks: BTreeMap<&str, u64> = BTreeMap::new();
        let mut shared_tick: Option<u64> = None;
        let mut english_tick: Option<u64> = None;
        let mut seen: BTreeSet<(&str, &str, Cow<str>)> = BTreeSet::new();
        for (line, number) in lines {
            let error = |reason: &str| ImportError::at(number, reason.to_string());
            // v1、v2 沒有裝置 id，這行照其他 `#` 開頭的行是註解。
            if let Some(id) = line.strip_prefix(DEVICE_KEY)
                && version == Version::V3
            {
                if user.device.is_some() {
                    return Err(error("裝置 id 重複"));
                }
                if !is_device_id(id) {
                    return Err(error("裝置 id 只能用英數、-、_，1–64 字"));
                }
                user.device = Some(id.to_string());
                continue;
            }
            // v1 沒有共用的 tick，這行照其他 `#` 開頭的行是註解。
            if let Some(tick) = line.strip_prefix(SHARED_TICK_KEY)
                && version >= Version::V2
            {
                let tick = tick.parse().map_err(|_| error("tick 不是整數"))?;
                // 讀到這裡時，個人 bigram 的 tick 是前面各列最大的最後 tick。
                if user.bigrams.tick > tick {
                    return Err(error("tick 比前面個人 bigram 列的最後 tick 小"));
                }
                shared_tick = Some(tick);
                continue;
            }
            if let Some(tick) = line.strip_prefix(ENGLISH_TICK_KEY)
                && version == Version::V3
            {
                let tick = tick.parse().map_err(|_| error("tick 不是整數"))?;
                // 讀到這裡時，英文使用者詞的 tick 是前面各列最大的最後 tick。
                if user.english.tick > tick {
                    return Err(error("tick 比前面英文使用者詞列的最後 tick 小"));
                }
                english_tick = Some(tick);
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
            let kind = if version == Version::V3 {
                let (kind, fields) = line.split_once('\t').unwrap_or((line, ""));
                match kind {
                    ENTRY => Kind::Entry(fields),
                    BIGRAM => Kind::Bigram(fields),
                    ENGLISH => Kind::English(fields),
                    "" => return Err(error("種類欄不能是空的")),
                    other => {
                        warnings.push(ImportWarning {
                            line: number,
                            reason: format!("不認得的種類「{other}」，略過這列"),
                        });
                        continue;
                    }
                }
            } else if let Some(shared) = line.strip_prefix('\t') {
                // 方案欄是空的：個人 bigram 或記住的字串（v2 起）；前一個字詞欄也是空的是記住的字串。
                if version == Version::V1 {
                    return Err(error("方案不能是空的（v1 沒有個人 bigram 與記住的字串）"));
                }
                match shared.strip_prefix('\t') {
                    Some(string) => Kind::V2String(string),
                    None => Kind::Bigram(shared),
                }
            } else {
                Kind::Entry(line)
            };
            match kind {
                Kind::English(fields) => {
                    let word = parse_english(fields).map_err(error)?;
                    insert_english(&mut user, word, english_tick).map_err(error)?;
                }
                Kind::V2String(fields) => {
                    let word = parse_string(fields, today).map_err(error)?;
                    insert_english(&mut user, word, shared_tick).map_err(error)?;
                }
                Kind::Bigram(fields) => {
                    let (previous, text, decayed) =
                        parse_bigram(fields, version, today).map_err(error)?;
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
                }
                Kind::Entry(fields) => {
                    let row = Row::parse(fields, version, today).map_err(error)?;
                    let tick = ticks.get(row.scheme).copied();
                    if tick.is_some_and(|t| row.count.last_tick.max(row.count.last_commit_tick) > t)
                    {
                        return Err(error("最後 tick 比方案的 tick 大"));
                    }
                    if !seen.insert((row.scheme, row.code, row.text.clone())) {
                        return Err(error("同方案、同輸入碼、同詞的列重複"));
                    }
                    let model = user.for_scheme_mut(row.scheme);
                    // 手動加了還沒用過的詞沒有計數，匯出時寫 0；這裡也不建計數。
                    if !row.count.is_empty() {
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
            }
        }
        // 只剩 tick 行、沒有資料列的方案（資料列都刪光了）也要還原。
        for (scheme, &tick) in &ticks {
            user.for_scheme_mut(scheme).tick = tick;
        }
        // 沒有 tick 行時，個人 bigram 的 tick 取最大的最後 tick（同方案）。
        if let Some(tick) = shared_tick {
            user.bigrams.tick = tick;
        }
        // v2 的記住的字串用共用的 tick；換成英文使用者詞自己的 tick 時接著用它，衰減的先後不變。
        if let Some(tick) = english_tick
            .or(shared_tick.filter(|_| version == Version::V2 && !user.english.words.is_empty()))
        {
            user.english.tick = tick;
        }
        Ok(Imported { user, warnings })
    }
}

/// [`UserModel::import`] 讀出來的：個人模型與略過的列。
#[derive(Debug, Clone, PartialEq)]
pub struct Imported {
    pub user: UserModel,
    /// 略過的列（v3 不認得的種類），依行號。
    pub warnings: Vec<ImportWarning>,
}

/// [`UserModel::import`] 略過的一列：第幾行（從 1 起）、為什麼。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportWarning {
    pub line: usize,
    pub reason: String,
}

impl fmt::Display for ImportWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "第 {} 行：{}", self.line, self.reason)
    }
}

/// 裝置 id 只能用英數、`-`、`_`，1–64 字：放得進 TSV 的 metadata 行，UUID 也收得下。
fn is_device_id(id: &str) -> bool {
    (1..=64).contains(&id.len())
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// 日期欄（v3）：空的是不知道（殼沒給過日期時學到的）。
fn parse_date(field: &str) -> Result<Option<Date>, &'static str> {
    if field.is_empty() {
        return Ok(None);
    }
    field.parse().map(Some).map_err(|_| BAD_DATE)
}

/// 匯出的日期欄：不知道時留空。
struct DateField(Option<Date>);

impl fmt::Display for DateField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(date) => date.fmt(f),
            None => Ok(()),
        }
    }
}

/// 匯出檔的一列 `entry`。
struct Row<'a> {
    scheme: &'a str,
    /// 已還原跳脫。
    text: Cow<'a, str>,
    code: &'a str,
    count: Count,
    word: Option<UserWord>,
}

impl<'a> Row<'a> {
    /// `line` 是去掉種類欄之後的（v1、v2 沒有種類欄）。v1 的詞欄照字面讀，沒有常用語；v1、v2 沒有
    /// 日期，最後使用的日期取 `today`。
    fn parse(line: &'a str, version: Version, today: Option<Date>) -> Result<Self, &'static str> {
        let v1 = version == Version::V1;
        let fields: Vec<&str> = line.split('\t').collect();
        let (fields, last_used, last_commit_date) = match (version, &fields[..]) {
            (
                Version::V3,
                &[
                    scheme,
                    text,
                    code,
                    commits,
                    last_commit_tick,
                    decayed,
                    last_tick,
                    last_used,
                    last_commit_date,
                    source,
                    trigger,
                ],
            ) => (
                [
                    scheme,
                    text,
                    code,
                    commits,
                    last_commit_tick,
                    decayed,
                    last_tick,
                    source,
                    trigger,
                ],
                parse_date(last_used)?,
                Some(parse_date(last_commit_date)?),
            ),
            (Version::V3, _) => return Err("entry 要有 12 欄（含種類），以 Tab 分隔"),
            // v1、v2 沒有日期：整串上屏過的，最後一次整串上屏的日期也取 `today`。
            (_, &[a, b, c, d, e, f, g, h, i]) => ([a, b, c, d, e, f, g, h, i], today, None),
            _ => return Err("要有 9 欄，以 Tab 分隔"),
        };
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
        ] = fields;
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
        let commits = integer(commits)?;
        let last_commit_date = match last_commit_date {
            Some(Some(_)) if commits == 0 => {
                return Err("沒有整串上屏過（commits 是 0）時最後整串上屏的日期要留空");
            }
            Some(date) => date,
            None if commits == 0 => None,
            None => today,
        };
        let count = Count {
            commits,
            last_commit_tick: integer(last_commit_tick)?,
            decayed,
            last_tick: integer(last_tick)?,
            last_used,
            last_commit_date,
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
        if phrase && !count.is_empty() {
            return Err("常用語（phrase）沒有計數，觸發詞學到的另是一列 learned");
        }
        let word = match source {
            None if trigger.is_some() => return Err("學到的（learned）不是使用者詞，沒有觸發詞"),
            None if count.decayed == 0.0 => return Err("學到的（learned）要有衰減值"),
            Some(WordSource::AutoPhrase) if trigger.is_some() => {
                return Err("自動造詞沒有觸發詞");
            }
            None => None,
            Some(source) => Some(UserWord {
                source,
                trigger,
                last_used,
            }),
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

/// 個人 bigram 的一列去掉種類欄（v2 是空的方案欄）之後：前一個字詞、這一段、衰減值、最後 tick，
/// v3 再接最後使用的日期（v2 取 `today`）。
fn parse_bigram(
    line: &str,
    version: Version,
    today: Option<Date>,
) -> Result<(Cow<'_, str>, Cow<'_, str>, Decayed), &'static str> {
    let fields: Vec<&str> = line.split('\t').collect();
    let ([previous, text, value, last_tick], last_used) = match (version, &fields[..]) {
        (Version::V3, &[previous, text, value, last_tick, last_used]) => {
            ([previous, text, value, last_tick], parse_date(last_used)?)
        }
        (Version::V3, _) => return Err("bigram 要有 6 欄（含種類），以 Tab 分隔"),
        (_, &[previous, text, value, last_tick]) => ([previous, text, value, last_tick], today),
        _ => return Err("個人 bigram（方案欄空的）要有 5 欄，以 Tab 分隔"),
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
        Decayed {
            value,
            last_tick,
            last_used,
        },
    ))
}

/// 英文使用者詞的一列去掉種類欄之後（v3）：寫法、次數、衰減值、最後 tick、最後使用的日期。寫法要是
/// 打字時會學的串（[`english_user::to_learn`]），不跳脫（串裡沒有反斜線、換行與 Tab）。
fn parse_english(line: &str) -> Result<EnglishUserWord, &'static str> {
    let &[text, count, value, last_tick, last_used] = &line.split('\t').collect::<Vec<_>>()[..]
    else {
        return Err("english 要有 6 欄（含種類），以 Tab 分隔");
    };
    let count: u64 = count.parse().map_err(|_| "次數要是正整數")?;
    if count == 0 {
        return Err("次數要是正整數");
    }
    english_word(text, count, value, last_tick, parse_date(last_used)?)
}

/// v2 的記住的字串（方案欄、前一個字詞欄空的列）去掉兩個空欄之後：字串、衰減值、最後 tick；最後使用
/// 的日期取 `today`。匯入成已過門檻的英文使用者詞（次數 [`ENGLISH_COMPLETION_AFTER`]，前綴補完也給）。
fn parse_string(line: &str, today: Option<Date>) -> Result<EnglishUserWord, &'static str> {
    let &[text, value, last_tick] = &line.split('\t').collect::<Vec<_>>()[..] else {
        return Err("記住的字串（方案欄、前一個字詞欄空的）要有 5 欄，以 Tab 分隔");
    };
    english_word(
        &unescape(text)?,
        ENGLISH_COMPLETION_AFTER,
        value,
        last_tick,
        today,
    )
}

/// 把匯入的一個英文使用者詞加進 `user`；最後 tick 比 tick 行（`tick`，沒有時不查）大、或不分大小寫
/// 重複時是錯誤。
fn insert_english(
    user: &mut UserModel,
    word: EnglishUserWord,
    tick: Option<u64>,
) -> Result<(), &'static str> {
    if tick.is_some_and(|t| word.decayed.last_tick > t) {
        return Err("最後 tick 比 tick 行的 tick 大");
    }
    if !user.english.insert(word) {
        return Err("英文使用者詞重複（不分大小寫）");
    }
    Ok(())
}

/// 檢查並組成一個英文使用者詞。
fn english_word(
    text: &str,
    count: u64,
    value: &str,
    last_tick: &str,
    last_used: Option<Date>,
) -> Result<EnglishUserWord, &'static str> {
    if english_user::to_learn(text) != Some(text) {
        return Err(
            "英文使用者詞要是打字時會學的串（英文、數字與 . _ % + - @ '，夠長，頭尾不是 . 或 '）",
        );
    }
    let value: f64 = value.parse().map_err(|_| "衰減值要是數字")?;
    if !value.is_finite() || value <= 0.0 {
        return Err("英文使用者詞的衰減值要是正的有限數");
    }
    let last_tick = last_tick.parse().map_err(|_| "tick 要是非負整數")?;
    Ok(EnglishUserWord {
        text: text.to_string(),
        count,
        decayed: Decayed {
            value,
            last_tick,
            last_used,
        },
    })
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

/// [`UserModel::set_device`] 不收的裝置 id。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidDeviceId(pub String);

impl fmt::Display for InvalidDeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "裝置 id「{}」只能用英數、-、_，1–64 字", self.0)
    }
}

impl std::error::Error for InvalidDeviceId {}

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
