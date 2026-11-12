//! 個人模型（plan §2.4）：使用者詞與學習計數。

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::english_user::{self, EnglishUserWords};
use crate::{Date, Engine, Segment};

mod format;
pub(crate) mod sync;
mod undo;
pub use format::{ImportError, ImportWarning, Imported};
pub use sync::Merged;
pub(crate) use sync::{Tally, Versioned};
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
    /// 最近一次清空的清空記號（[`UserModel::clear`]）；沒清空過是 `None`（第 0 代）。跨裝置合併時代數大的
    /// 那邊整份算數（使用者 2026-11-13 決定，ADR 0007）。
    cleared: Option<Cleared>,
    /// 條目上限（[`UserModel::set_limits`]）；不進匯出檔。
    limits: Limits,
    /// 最近一個動作學到的東西的撤銷紀錄（退格撤銷，[`UserModel::take_undo`]）；不進匯出檔，比較時不看。
    undo: Undo,
}

/// 清空記號（[`UserModel::clear`]）：清空學習資料時留下，跨裝置合併時把清空傳到別台。一台的學習資料永遠只屬於
/// 一代：收到較新的代數就先清掉自己的，所以不必每列記代數。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Cleared {
    /// 代數：清空時取這台看過最大的加一。合併時代數大的那邊整份算數、代數小的整份不收。
    pub(crate) generation: u64,
    /// 清空的日期；殼沒給日期時是 `None`。只給人看與合併時挑一個記號，不決定誰算數。
    pub(crate) on: Option<Date>,
    /// 清空的裝置 id；殼沒給時是空字串。
    pub(crate) device: String,
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
            cleared,
            limits,
            undo: _,
        } = self;
        *schemes == other.schemes
            && *bigrams == other.bigrams
            && *english == other.english
            && *today == other.today
            && *device == other.device
            && *cleared == other.cleared
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
    /// 前一個字詞 → 這一段 → 各台的次數與墓碑（[`Tally`]）；只放還在的組。
    tallies: BTreeMap<String, BTreeMap<String, Tally>>,
    /// 刪掉的組（墓碑表）：[`Bigrams::digest`] → 各台的次數與墓碑，不存原文。
    graveyard: BTreeMap<String, Tally>,
}

/// 一個衰減值與它最後更新的 tick（librime `formula_d`）。
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct Decayed {
    value: f64,
    last_tick: u64,
    /// 最後使用的日期；殼沒給日期時是 `None`。
    pub(crate) last_used: Option<Date>,
    /// 寫入時那個條目的合計次數（[`Tally::uses`]）：跨裝置合併時版本大的留下（[`sync`]）。
    pub(crate) version: u64,
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

    /// 在 `tick`、`today` 又上屏（或打）一次：衰減到現在再加一（librime `formula_d`）；`version` 是這次學到後
    /// 條目的合計次數。
    pub(crate) fn bump(&mut self, tick: u64, today: Option<Date>, version: u64) {
        self.value = 1.0 + self.at(tick);
        self.last_tick = tick;
        self.last_used = today;
        self.version = version;
    }

    /// tick 往後挪 `by`（合併時對齊兩邊的 tick，[`align`]）：距現在的上屏次數不變。
    pub(crate) fn shifted(self, by: u64) -> Self {
        Self {
            last_tick: self.last_tick + by,
            ..self
        }
    }
}

impl Versioned for Decayed {
    fn version(&self) -> u64 {
        self.version
    }

    /// 同版本是同一次寫入：留 tick 較舊的（對齊後距現在較久，[`align`]），它來自寫入後又上屏過較多次的那台，
    /// 衰減得比較準；兩台互相同步一輪就收斂。
    fn order(&self, other: &Self) -> std::cmp::Ordering {
        let key = |d: &Self| (d.version, Reverse(d.last_tick), d.last_used);
        key(self)
            .cmp(&key(other))
            .then(self.value.total_cmp(&other.value))
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
    /// 輸入碼 → 詞 → 各台的次數與墓碑（[`Tally`]）。有計數或使用者詞的列一定有；只放還在的列。
    tallies: BTreeMap<String, BTreeMap<String, Tally>>,
    /// 刪掉的列（墓碑表）：[`SchemeModel::digest`] → 各台的次數與墓碑，不存原文。再用到同一列時移回 `tallies`。
    graveyard: BTreeMap<String, Tally>,
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
    /// 加進來（或改觸發詞、自動造詞）時這列的合計次數（[`Tally::uses`]），跨裝置合併用（[`sync`]）。
    version: u64,
}

impl WordSource {
    /// 合併時同版本的先後（固定的順序就好）。
    fn rank(self) -> u8 {
        match self {
            Self::AutoPhrase => 0,
            Self::Manual => 1,
            Self::Phrase => 2,
        }
    }
}

impl Versioned for UserWord {
    fn version(&self) -> u64 {
        self.version
    }

    fn order(&self, other: &Self) -> std::cmp::Ordering {
        let key = |w: &Self| (w.version, w.last_used, w.source.rank(), w.trigger.clone());
        key(self).cmp(&key(other))
    }
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

/// 一個（輸入碼, 候選文字）在這台學到的值；方案由所在的 [`SchemeModel`] 決定。兩種上屏分開記（plan §2.4）：
/// - 整串上屏：整串輸入碼就是這個碼、上屏的就是這個候選；次數各台分開記在這列的 [`Tally`]，給排位規則 2 用。
/// - 句中各段的上屏也算進衰減值，給 `personal` 用。
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Count {
    /// 衰減值：整串上屏與句中各段的上屏每次加一，舊的隨上屏次數指數衰減（librime
    /// `formula_d`）；記的是 `last_tick` 當時的值。
    decayed: f64,
    /// 最後一次（整串或句中）上屏時的 tick。
    last_tick: u64,
    /// 最後一次整串上屏的 tick；沒有整串上屏過是 `None`。排位規則 2 的期限與「最近」用。
    last_commit_tick: Option<u64>,
    /// 最後使用的日期：最後一次（整串或句中）上屏時殼給的今天；殼沒給時是 `None`。一年沒用的條目停用（[`SchemeUser`]）。
    last_used: Option<Date>,
    /// 最後一次學到時這列的合計次數（[`Tally::uses`]），跨裝置合併用（[`sync`]）。
    version: u64,
}

impl Count {
    /// tick 往後挪 `by`（合併時對齊兩邊的 tick，[`align`]）。
    fn shifted(self, by: u64) -> Self {
        Self {
            last_tick: self.last_tick + by,
            last_commit_tick: self.last_commit_tick.map(|t| t + by),
            ..self
        }
    }
}

impl Versioned for Count {
    fn version(&self) -> u64 {
        self.version
    }

    /// 同版本留 tick 較舊的，同 [`Decayed`]。
    fn order(&self, other: &Self) -> std::cmp::Ordering {
        let key = |c: &Self| {
            (
                c.version,
                Reverse(c.last_tick),
                Reverse(c.last_commit_tick),
                c.last_used,
            )
        };
        key(self)
            .cmp(&key(other))
            .then(self.decayed.total_cmp(&other.decayed))
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
            tallies: BTreeMap::new(),
            graveyard: BTreeMap::new(),
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
        let device = self.own_device();
        let (model, scheme_undo) = self.learning_scheme(scheme);
        model.learn(segments, today, &device, scheme_undo);
        self.bigrams
            .learn(previous, segments, today, &device, &mut self.undo);
        self.enforce_limits();
    }

    /// 這台在各台次數（[`Tally`]）裡的格子：殼給的裝置 id，沒給時是空字串（不知道哪台）。
    fn own_device(&self) -> String {
        self.device.clone().unwrap_or_default()
    }

    /// 選了一個聯想（plan §2.4）：只記個人 bigram「`previous` → `text`」。聯想沒有輸入碼，
    /// 不記學習計數。
    pub(crate) fn learn_association(&mut self, previous: &str, text: &str) {
        let device = self.own_device();
        self.bigrams.learn(
            Some(previous),
            &[Segment::new(String::new(), text.to_string())],
            self.today,
            &device,
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

    /// 這個動作不算夾在中間（不送字元的修飾鍵，它不學東西）：前一個動作的紀錄放回去，下一個
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
        let device = self.own_device();
        self.english.learn(text, self.today, &device);
        self.enforce_limits();
    }

    /// 英文使用者詞（最後打的寫法），依小寫排；殼的清單用，使用者可以逐條刪
    /// （[`UserModel::forget_english_user_word`]）。沒到出現門檻的也列出。
    pub fn english_user_words(&self) -> Vec<String> {
        self.english.list()
    }

    /// 忘掉英文使用者詞 `text`（不分大小寫）：匯出檔裡只剩它的墓碑（各台的次數與刪除時的合計，跨裝置合併時
    /// 把刪除傳到別台，ADR 0007）；沒學過時回傳 `false`。
    pub fn forget_english_user_word(&mut self, text: &str) -> bool {
        self.discard_undo();
        self.english.forget(text, self.today)
    }

    /// 撤銷學習（plan §2.4，M4 票 16）：清掉方案 `scheme` 裡（`code`, `text`）學到的，經
    /// [`crate::Session::forget`] 呼叫。刪學習資料從嚴（協調者 2026-11-12 決定）：清這一列的計數（整串上屏
    /// 次數、衰減值）與這一列的自動造詞；個人 bigram 只清「`previous` → `text`」那一組（`previous` 是候選列
    /// 現在接的前一個字詞，沒有時不清），其他前一個字詞後面學到的不動，免得清一個常用字抹掉它在所有前文
    /// 後面的學習。自動造詞刪掉時，落在它裡面前後相接的組（「乙丁」的「乙 → 丁」，造詞那次學到的段間接續）
    /// 一起清，否則同樣的字會以句子候選回到前面。手動加的詞與常用語只清計數、詞留著。句子候選的各段是各自
    /// 的學習，不清（同 librime 刪候選只刪那個詞條）。清掉的各留一個墓碑（M4 票 17，ADR 0007）：同步時把
    /// 刪除傳到別台；再選就重新學、復活，不做永久屏蔽。撤銷紀錄跟著丟掉（使用者明確的操作）。有清到東西時
    /// 回傳 `true`。
    pub(crate) fn forget(
        &mut self,
        scheme: &str,
        code: &str,
        text: &str,
        previous: Option<&str>,
    ) -> bool {
        self.discard_undo();
        let (today, device) = (self.today, self.own_device());
        let mut changed = false;
        let mut auto_phrase = false;
        if let Some(model) = self.schemes.get_mut(scheme) {
            auto_phrase = model
                .word(code, text)
                .is_some_and(|w| w.source == WordSource::AutoPhrase);
            changed |= model.forget_counts(code, text, today, &device);
        }
        let doomed: Vec<(String, String)> = self
            .bigrams
            .entries
            .iter()
            .flat_map(|(before, next)| next.keys().map(move |n| (before, n)))
            .filter(|(before, next)| {
                Some(before.as_str()) == previous && next.as_str() == text
                    || auto_phrase && text.contains(&format!("{before}{next}"))
            })
            .map(|(before, next)| (before.clone(), next.clone()))
            .collect();
        for (before, next) in &doomed {
            self.bigrams.delete(before, next, today);
        }
        changed || !doomed.is_empty()
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
                    threshold: if self.english.passed(key, today) {
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
                        self.english.remove(&key);
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
            self.bigrams.retain(|previous, text| {
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
        if !format::is_device_id(id) {
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
        self.replace_data(learned);
    }

    /// 清空全部學習資料（範圍同網頁的「清空學習資料」：學習計數、使用者詞含手動加詞與常用語、個人 bigram、
    /// 英文使用者詞、墓碑），留清空記號（[`Cleared`]），跨裝置合併時把清空傳到別台：代數小的（清空前的）快照
    /// 整份不收，別台合併這台的檔時清掉它清空前學的（使用者 2026-11-13 決定，ADR 0007）。今天、裝置 id 與上限
    /// 留著。撤銷紀錄跟著丟掉。
    pub fn clear(&mut self) {
        let cleared = Cleared {
            generation: self.generation() + 1,
            on: self.today,
            device: self.own_device(),
        };
        self.replace_data(UserModel {
            cleared: Some(cleared),
            ..UserModel::default()
        });
    }

    /// 從備份還原（協調者 2026-11-13 決定，和合併的匯入分開）：留一個新的清空記號（代數取這台與備份較大的加一），
    /// 再把備份 `backup`（[`UserModel::import`] 讀出來的）的學習資料當成清空之後學到的收進來。是這台的新事件：
    /// 清空前做的備份也還原得回來，別台的舊資料（代數較小）同步後不回來，合併的三條律照樣成立。今天、裝置 id
    /// 與上限留著；撤銷紀錄丟掉。
    pub fn restore(&mut self, backup: UserModel) {
        let cleared = Cleared {
            generation: self.generation().max(backup.generation()) + 1,
            on: self.today,
            device: self.own_device(),
        };
        self.replace_data(UserModel {
            cleared: Some(cleared),
            ..backup
        });
    }

    /// 清空記號的代數；沒清空過是 0。
    fn generation(&self) -> u64 {
        self.cleared.as_ref().map_or(0, |c| c.generation)
    }

    /// 拿 `learned` 的學習資料（含清空記號）取代自己的，今天、裝置 id 與上限留著。
    fn replace_data(&mut self, learned: UserModel) {
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
        let device = self.own_device();
        let (model, scheme_undo) = self.learning_scheme(scheme);
        model.add_auto_phrase(code, text, today, &device, scheme_undo);
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
        let (today, device) = (self.today, self.own_device());
        let model = self.for_scheme_mut(&engine.scheme().id);
        model.put_word(code, text, WordSource::Manual, trigger, today, &device);
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
        let (today, device) = (self.today, self.own_device());
        let model = self.for_scheme_mut(&engine.scheme().id);
        let old = model.put_word(
            PHRASE_CODE,
            text,
            WordSource::Phrase,
            trigger,
            today,
            &device,
        );
        if let Some(old) = old.and_then(|w| w.trigger)
            && Some(old.as_str()) != trigger
        {
            model.forget_counts(&old, text, today, &device);
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

    /// 刪掉文字是 `text` 的常用語，連同它的觸發詞學到的計數，匯出檔裡只剩墓碑（同步時把刪除傳到別台，
    /// ADR 0007）；沒有這條時回傳 `false`。觸發詞剛好是這段文字自己的碼時，平常打字學到的同一組計數也一起刪：兩者是同一個鍵、
    /// 分不出來，寧可少一筆學習，也不留使用者刪掉的字。
    pub fn remove_phrase(&mut self, engine: &Engine, text: &str) -> bool {
        self.discard_undo();
        let (today, device) = (self.today, self.own_device());
        // 沒學過的方案不建出空的方案（`for_scheme_mut` 會建）。
        let Some(model) = self.schemes.get_mut(&engine.scheme().id) else {
            return false;
        };
        let Some(word) = model.word(PHRASE_CODE, text).cloned() else {
            return false;
        };
        model.delete_row(PHRASE_CODE, text, today);
        if let Some(trigger) = word.trigger {
            model.forget_counts(&trigger, text, today, &device);
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
    /// 只留 `keep` 為真的組，連同各台的次數一起刪（上限淘汰，不留墓碑）；前一個字詞後面都刪光時連它一起刪。
    fn retain(&mut self, mut keep: impl FnMut(&str, &str) -> bool) {
        self.entries.retain(|previous, next| {
            next.retain(|text, _| keep(previous, text));
            !next.is_empty()
        });
        self.tallies.retain(|previous, next| {
            next.retain(|text, _| keep(previous, text));
            !next.is_empty()
        });
    }

    /// 在 `today` 刪掉一組（撤銷學習）：留墓碑、移進墓碑表，衰減值丟掉。
    fn delete(&mut self, previous: &str, text: &str, today: Option<Date>) {
        if let Some(mut tally) = remove_in(&mut self.tallies, previous, text) {
            tally.delete(today);
            self.graveyard
                .insert(Bigrams::digest(previous, text), tally);
        }
        remove_in(&mut self.entries, previous, text);
    }

    /// 一組在墓碑表裡的鍵（[`sync::digest`]）。
    pub(super) fn digest(previous: &str, text: &str) -> String {
        sync::digest(&["bigram", previous, text])
    }

    /// （`previous`, `text`）各台的次數與墓碑。
    pub(super) fn tally(&self, previous: &str, text: &str) -> Option<&Tally> {
        self.tallies.get(previous)?.get(text)
    }

    /// （`previous`, `text`）各台的次數：沒有時從墓碑表移回（刪過又用到），都沒有就建一個空的。
    fn tally_mut(&mut self, previous: &str, text: &str) -> &mut Tally {
        if self.tally(previous, text).is_none() {
            let buried = sync::exhume(&mut self.graveyard, || Bigrams::digest(previous, text));
            put_in(
                &mut self.tallies,
                previous,
                text,
                Some(buried.unwrap_or_default()),
            );
        }
        self.tallies
            .get_mut(previous)
            .and_then(|n| n.get_mut(text))
            .expect("剛放進去")
    }

    /// 一次上屏的個人 bigram：`previous` → 第一段、相鄰兩段各記一次；有兩段以上時 `previous` →
    /// 整串也記（整串可能是自動造詞的使用者詞，同 [`SchemeModel::learn`] 整串和各段都記）。
    /// 同一組只記一次。
    fn learn(
        &mut self,
        previous: Option<&str>,
        segments: &[Segment],
        today: Option<Date>,
        device: &str,
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
            let version = self.tally_mut(previous, text).touch(device);
            let decayed = self
                .entries
                .entry(previous.to_string())
                .or_default()
                .entry(text.to_string())
                .or_default();
            decayed.bump(tick, today, version);
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

    /// 刪掉一整列（上限淘汰）：學習計數、使用者詞與各台的次數，不留墓碑。淘汰是這台自己的空間管理，不是
    /// 使用者刪的：別台還有這列時合併回來照樣有（ADR 0007）。
    fn remove_row(&mut self, code: &str, text: &str) {
        remove_in(&mut self.entries, code, text);
        remove_in(&mut self.words, code, text);
        remove_in(&mut self.tallies, code, text);
    }

    /// 在 `today` 刪掉一整列（使用者刪的）：留墓碑（[`Tally::delete`]），學習計數與使用者詞丟掉。這列沒有
    /// 任何東西時不動。
    fn delete_row(&mut self, code: &str, text: &str, today: Option<Date>) {
        // 手動加詞與常用語一直有效（不受一年時限）：墓碑不記日期、不回收，否則回收後才回來的舊快照會讓
        // 刪掉的詞帶著原文復活（墓碑的保留期靠的是條目也過了一年而停用，ADR 0007）。
        let never_expires = self
            .word(code, text)
            .is_some_and(|w| w.source != WordSource::AutoPhrase);
        if let Some(mut tally) = remove_in(&mut self.tallies, code, text) {
            tally.delete(today.filter(|_| !never_expires));
            self.graveyard
                .insert(SchemeModel::digest(code, text), tally);
        }
        remove_in(&mut self.entries, code, text);
        remove_in(&mut self.words, code, text);
    }

    /// 一列在墓碑表裡的鍵（[`sync::digest`]）；方案是墓碑表所在的方案。
    pub(super) fn digest(code: &str, text: &str) -> String {
        sync::digest(&["entry", code, text])
    }

    /// 撤銷（`code`, `text`）學到的（[`UserModel::forget`]、常用語換掉或刪掉觸發詞）：有計數或是自動造詞時整列
    /// 刪掉、留墓碑；手動加詞與常用語只清計數、詞留著：刪掉後由這台再加回一次（`device` 自己那格加一，版本
    /// 在墓碑之後）。有清到東西時回傳 `true`。
    fn forget_counts(&mut self, code: &str, text: &str, today: Option<Date>, device: &str) -> bool {
        let word = self.word(code, text).cloned();
        let auto_phrase = word
            .as_ref()
            .is_some_and(|w| w.source == WordSource::AutoPhrase);
        if self.count(code, text).is_none() && !auto_phrase {
            return false;
        }
        self.delete_row(code, text, today);
        if let Some(word) = word.filter(|_| !auto_phrase) {
            let version = self.tally_mut(code, text).touch(device);
            self.words_at(code)
                .insert(text.to_string(), UserWord { version, ..word });
        }
        true
    }

    /// 把（`code`, `text`）寫成來源 `source`、觸發詞 `trigger` 的使用者詞（手動加詞、常用語）：明確加入算
    /// 這台用了一次（`device` 自己那格加一，刪過的也因此復活）。回傳原本的使用者詞。
    fn put_word(
        &mut self,
        code: &str,
        text: &str,
        source: WordSource,
        trigger: Option<&str>,
        today: Option<Date>,
        device: &str,
    ) -> Option<UserWord> {
        let version = self.tally_mut(code, text).touch(device);
        self.words_at(code).insert(
            text.to_string(),
            UserWord {
                source,
                trigger: trigger.map(String::from),
                last_used: today,
                version,
            },
        )
    }

    /// 碼 `code` 的使用者詞，沒有就建一個空的。
    fn words_at(&mut self, code: &str) -> &mut BTreeMap<String, UserWord> {
        self.words.entry(code.to_string()).or_default()
    }

    /// （`code`, `text`）各台的次數：沒有時從墓碑表移回（刪過又用到），都沒有就建一個空的。
    fn tally_mut(&mut self, code: &str, text: &str) -> &mut Tally {
        if self.tally(code, text).is_none() {
            let buried = sync::exhume(&mut self.graveyard, || SchemeModel::digest(code, text));
            put_in(
                &mut self.tallies,
                code,
                text,
                Some(buried.unwrap_or_default()),
            );
        }
        self.tallies
            .get_mut(code)
            .and_then(|t| t.get_mut(text))
            .expect("剛放進去")
    }

    /// （`code`, `text`）各台的次數與墓碑。
    fn tally(&self, code: &str, text: &str) -> Option<&Tally> {
        self.tallies.get(code)?.get(text)
    }

    /// 一次上屏：整串輸入碼和每一段的衰減值各記一次，（碼, 字詞）相同的只記一次；整串另記一次
    /// 整串上屏（plan §2.4）。`segments` 依輸入碼的順序、接起來是整串，只看碼與字詞，不是空的。次數記在
    /// `device` 自己那格。
    fn learn(
        &mut self,
        segments: &[Segment],
        today: Option<Date>,
        device: &str,
        undo: &mut undo::SchemeUndo,
    ) {
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
            let tally = self.tally_mut(&segment.code, &segment.text);
            // `keys` 的第一個是整串；這台距上一次整串上屏超過一年時從 1 重算。
            if i == 0 {
                tally.commit(device, today);
            }
            let version = tally.touch(device);
            let count = self
                .entries
                .entry(segment.code.clone())
                .or_default()
                .entry(segment.text.clone())
                .or_default();
            count.decayed = 1.0 + count.decayed * decay(tick - count.last_tick);
            count.last_tick = tick;
            count.version = version;
            if i == 0 {
                count.last_commit_tick = Some(tick);
            }
            self.stamp(&segment.code, &segment.text, today);
        }
    }

    /// （`code`, `text`）這天用過或改過：它的計數與使用者詞都記成最後使用的日期 `today`，兩者是匯出檔
    /// 的同一列。
    fn stamp(&mut self, code: &str, text: &str, today: Option<Date>) {
        if let Some(count) = self.entries.get_mut(code).and_then(|t| t.get_mut(text)) {
            count.last_used = today;
        }
        if let Some(word) = self.words.get_mut(code).and_then(|w| w.get_mut(text)) {
            word.last_used = today;
        }
    }

    /// [`UserModel::add_auto_phrase`]：版本是同一次上屏學到後的合計次數（[`SchemeModel::learn`] 先記）；沒先記
    /// 時（這列不在世）算這台用了一次。
    fn add_auto_phrase(
        &mut self,
        code: &str,
        text: &str,
        today: Option<Date>,
        device: &str,
        undo: &mut undo::SchemeUndo,
    ) {
        undo.save(self, code, text);
        let tally = self.tally_mut(code, text);
        let version = if tally.alive() && tally.uses() > 0 {
            tally.uses()
        } else {
            tally.touch(device)
        };
        self.words_at(code)
            .entry(text.to_string())
            .or_insert(UserWord {
                source: WordSource::AutoPhrase,
                trigger: None,
                last_used: today,
                version,
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
                let tick = self
                    .count(input, text)
                    .and_then(|c| c.last_commit_tick)
                    .unwrap_or(0);
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

    /// （碼, 字詞）算數的整串上屏次數（各台相加、各台只算一年內的，[`Tally::commits_on`]）；排位規則 2 的
    /// 「剛好 1 次的不靠分數成首選」用。停用的列是 0。
    pub(crate) fn whole_commits(&self, code: &str, text: &str) -> u64 {
        self.count(code, text)
            .and(self.model.tally(code, text))
            .map_or(0, |tally| tally.commits_on(self.today))
    }

    /// 排位規則 3（plan §2.4）：（`code`, `text`）是使用者詞且一年內整串上屏過時，回傳最後一次整串
    /// 上屏的 tick，越大越近；否則是 `None`（一年沒上屏的回到一般排序）。
    pub(crate) fn last_tick_if_used_word(&self, code: &str, text: &str) -> Option<u64> {
        if !self.is_word(code, text) {
            return None;
        }
        let tick = self.count(code, text)?.last_commit_tick?;
        (self.whole_commits(code, text) >= 1).then_some(tick)
    }

    /// 這組（碼, 字詞）學過沒有（沒停用的）。
    pub(crate) fn has_learned(&self, code: &str, text: &str) -> bool {
        self.count(code, text).is_some()
    }

    /// 排位規則 2（plan §2.4）：一年內整串上屏 ≥ 2 次（各台相加，[`SchemeUser::whole_commits`]），且距最後
    /// 一次整串上屏不超過 [`RECENT_COMMITS`] 次上屏時，回傳那次的 tick，越大越近；否則是 `None`。句中各段的
    /// 上屏不算。
    pub(crate) fn last_tick_if_chosen_twice(&self, code: &str, text: &str) -> Option<u64> {
        let tick = self.count(code, text)?.last_commit_tick?;
        (self.whole_commits(code, text) >= 2 && self.model.tick - tick <= RECENT_COMMITS)
            .then_some(tick)
    }

    /// 個人機率（plan §2.3）：衰減值衰減到現在，除以它的上限；沒學過或停用時是 0。
    pub(crate) fn personal_probability(&self, code: &str, text: &str) -> f64 {
        self.count(code, text).map_or(0.0, |count| {
            count.decayed * decay(self.model.tick - count.last_tick) / max_decayed()
        })
    }
}

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

/// 兩層的表裡放一個值，`None` 是刪掉（裡層刪光時連外層一起刪）。
fn put_in<V>(
    map: &mut BTreeMap<String, BTreeMap<String, V>>,
    outer: &str,
    inner: &str,
    value: Option<V>,
) {
    match value {
        Some(value) => {
            map.entry(outer.to_string())
                .or_default()
                .insert(inner.to_string(), value);
        }
        None => {
            remove_in(map, outer, inner);
        }
    }
}

/// 兩層的表裡刪掉一個值，裡層刪光時連外層一起刪；回傳刪掉的值。
fn remove_in<V>(
    map: &mut BTreeMap<String, BTreeMap<String, V>>,
    outer: &str,
    inner: &str,
) -> Option<V> {
    let next = map.get_mut(outer)?;
    let removed = next.remove(inner);
    if next.is_empty() {
        map.remove(outer);
    }
    removed
}

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
