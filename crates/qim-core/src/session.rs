//! 殼與核心之間的介面（plan §2.1）：殼把按鍵交給 [`Session`]，拿回一份完整的 [`Snapshot`]。

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::association::{self, ASSOCIATION_TAIL};
use crate::english_user::TypedRun;
use crate::lattice::{self, CandidateSpan, Lattice, Listing, Preceding};
use crate::scheme::{SEPARATOR, without_separators};
use crate::{Engine, UserModel, Weights, punctuation};

mod inspect;

pub use inspect::{
    CandidateKind, ContextState, InspectedCandidate, InspectedSegment, Inspection, PrecedingChar,
    Reranked, SegmentSource,
};

/// 候選每頁的個數，對應數字鍵 1–9（brief §4.3）。
pub const PAGE_SIZE: usize = 9;

/// 核心留著的游標前文字最多幾個字（[`Session::set_context`]）：送雲端的上限（brief §8），bigram
/// 只看最後一字。殼可以給更長的，核心只留最後這麼多。
pub const CONTEXT_LIMIT: usize = 200;

/// 重排要求最多帶幾條整句路徑（plan §2.1，M3 票 07、11）：lattice 的前 9 條，候選列沒有的也排得上來。
pub const RERANK_PATHS: usize = 9;

/// 重排要求的前文最多幾個字（brief §8）：M3 模型階梯量過 200 字不比 50 字好。
pub const RERANK_CONTEXT: usize = 50;

/// 給背景模型的重排要求（[`Session::rerank_request`]，plan §2.1、§2.7）。
#[derive(Debug, Clone, PartialEq)]
pub struct RerankRequest {
    /// 結果交回 [`Session::apply_rerank`] 時帶著它；不是目前的就丟掉。
    pub generation: u64,
    /// 路徑前面的字：前文（簡體模式下逐字轉回繁體）接上選定的分段（繁體原文），最多
    /// [`RERANK_CONTEXT`] 字。
    pub context: String,
    /// lattice 蓋滿沒選定的輸入碼的前 [`RERANK_PATHS`] 條路徑，依路徑分由高到低；結果的 `order` 是它的序號。
    pub paths: Vec<ScoredPath>,
}

/// lattice 裡蓋滿沒選定的輸入碼的一條路徑（[`Session::paths`]）；可序列化，模型階梯匯出給外部模型。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoredPath {
    /// 字詞接起來的文字（繁體原文，不經簡體模式的後處理）。
    pub text: String,
    /// 各段的字詞（繁體原文），接起來是 `text`。
    pub segments: Vec<String>,
    /// 同步層的路徑分（plan §2.3），只在同一串輸入碼內可比。
    pub score: f64,
}

/// 上屏的組字的最後一段（[`Session::context_word`]）。
#[derive(Debug, Clone, PartialEq, Eq)]
struct LastWord {
    /// 原文（繁體），學習記的。
    original: String,
    /// 顯示、上屏的字：簡體模式下是轉換後的，前文裡是它。
    shown: String,
}

/// 聯想（plan §2.1，M3 票 13）在接什麼：候選列是聯想時才有。
#[derive(Debug, Clone, PartialEq, Eq)]
struct Association {
    /// 剛上屏的各段（繁體原文）：這次組字上屏的分段，接著選過的聯想，最多留到蓋住最後
    /// [`ASSOCIATION_TAIL`] 字。最後一段是個人 bigram 的前一個字詞。
    segments: Vec<String>,
    /// 這次上屏顯示的字（簡體模式是轉換後的）：殼之後給的前文不以它結尾，就是游標移走了。
    shown: String,
}

/// 殼給的選項（plan §2.1），由 [`Session::set_options`] 換。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    /// 簡體輸出：候選、組字與上屏轉成簡體，學習仍記繁體（plan §2.2 的後處理）；
    /// 引擎要掛上簡繁資料包（[`Engine::with_simplifier`]）。
    pub simplified: bool,
    /// 無痕模式：不寫任何學習資料、不用前文（plan §2.4、brief §8）。
    pub incognito: bool,
    /// 學習（brief §4.5、§7）：關掉時不寫任何學習資料，已學的照用、前文照用（和無痕不同）。
    /// 預設開。
    pub learning: bool,
    /// 中文狀態的英文候選（排位規則 4，plan §2.4）：整串輸入碼剛好是英文單字時排第 2 位；
    /// 引擎要掛上英文資料包（[`Engine::with_english`]）。預設開（brief §7）。
    pub english_candidates: bool,
    /// 數字後的 `.` `:` 用半形（brief §4.7）：前文以半形數字結尾時直接上屏半形（[`Session::set_context`]），同 librime
    /// `digit_separators` 預設加 `digit_separator_action: commit`；逗號照舊全形。預設開（brief §7）。
    pub digit_separators: bool,
    /// emoji 候選（plan §2.4）：整串的候選是 emoji 資料包裡的詞時，它的 emoji 接在它後面；
    /// 引擎要掛上 emoji 資料包（[`Engine::with_emoji`]）。預設開（brief §7，同 rime-emoji）。
    pub emoji_candidates: bool,
    /// 組字中單按左、右 Shift 怎麼處理（brief §4.3）；預設照 Rime。
    pub shift_switch: ShiftSwitch,
    /// 繁體模式把碼表標成簡體字的字排在同碼的其他候選之後（brief §4.1，M3 票 19）：同 macOS
    /// 速成，預設開（brief §7）；關掉時照分數混排。簡體模式不管它。
    pub simplified_after_traditional: bool,
    /// 英文使用者詞（plan §2.4，M4 票 14）：學經核心送出去的英文串（4 字元起），英文詞典沒有、打過 2 次
    /// 的在中文組字打完整串時排第 2（排位規則 4），打過 3 次又是最常用的 20 條之一時打它開頭的字母
    /// （長過一個字的最長碼）也給（排位規則 5）。關掉時不學也不給；無痕、學習關掉時不學、已學的照給。
    /// 預設開（brief §7）。
    pub english_user_words: bool,
    /// 聯想（brief §7，M3 票 13）：上屏組字或選了聯想之後、還沒打下一個字時，候選列給接下來可能要
    /// 打的字詞（[`Session::select`] 或數字鍵選）。預設開（brief §7）。
    pub association: bool,
}

/// 組字中單按 Shift 的處理，對應 Rime `ascii_composer/switch_key` 的兩種做法：`commit_text`
/// 先上屏組字再切英文，`inline_ascii` 留著沒選定的輸入碼當英文接下去（[臨時英文]）。
/// 沒有組字時兩邊都只切換。
///
/// [臨時英文]: Snapshot::ascii
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ShiftSwitch {
    /// rime-prelude `default.yaml` 的預設：左 Shift `inline_ascii`、右 Shift `commit_text`。
    #[default]
    Rime,
    /// 左右都先上屏組字（同一直按空白）再切英文。
    CommitText,
    /// 左右都留碼接英文。
    InlineAscii,
}

impl std::str::FromStr for ShiftSwitch {
    type Err = String;

    /// 殼的選單用的名字：`rime`、`commit-text`、`inline-ascii`。
    fn from_str(name: &str) -> Result<Self, String> {
        match name {
            "rime" => Ok(Self::Rime),
            "commit-text" => Ok(Self::CommitText),
            "inline-ascii" => Ok(Self::InlineAscii),
            _ => Err(format!(
                "不認得的 Shift 處理：{name}（只有 rime、commit-text、inline-ascii）"
            )),
        }
    }
}

impl ShiftSwitch {
    /// 按了 `shift` 時要不要留碼接英文；否則先上屏組字。
    fn keeps_code(self, shift: ShiftKey) -> bool {
        match self {
            Self::Rime => shift == ShiftKey::Left,
            Self::CommitText => false,
            Self::InlineAscii => true,
        }
    }
}

/// 單按的是哪一邊的 Shift；殼分辨（網頁用 `KeyboardEvent.code`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShiftKey {
    Left,
    Right,
}

impl Options {
    /// 兩份選項列出的候選相同：只有簡體、英文候選、emoji 候選、簡體字排後與英文使用者詞影響候選，
    /// 無痕、學習、Shift、數字後半形都不影響。
    fn lists_like(&self, other: &Options) -> bool {
        self.simplified == other.simplified
            && self.english_candidates == other.english_candidates
            && self.emoji_candidates == other.emoji_candidates
            && self.simplified_after_traditional == other.simplified_after_traditional
            && self.english_user_words == other.english_user_words
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            simplified: false,
            incognito: false,
            learning: true,
            english_candidates: true,
            digit_separators: true,
            emoji_candidates: true,
            shift_switch: ShiftSwitch::Rime,
            simplified_after_traditional: true,
            english_user_words: true,
            association: true,
        }
    }
}

/// 殼轉過來的按鍵。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    /// 字母，含大寫；方案的字母表以外的也是，由核心決定怎麼處理。
    Letter(char),
    /// 數字、字母、空白以外的可見 ASCII 字元（`,`、`.`、`@`…）；`-` / `=` 殼轉成翻頁。
    Punct(char),
    Space,
    Enter,
    Backspace,
    Escape,
    Digit(u8),
    PageUp,
    PageDown,
    Left,
    Right,
    /// 單按 Shift（切中英，brief §4.3）；單按的判定在殼。
    ToggleAscii(ShiftKey),
    /// 殼不轉成上面各種的鍵（Tab、Delete、↑ ↓、Home、End…）：組字中吃下，免得宿主拿它改輸入框。
    Other,
    /// 只改鍵盤狀態、不送字元的鍵：單按 Ctrl、Alt、Cmd、CapsLock 這類修飾鍵，觸控鍵盤的切頁、⇧、
    /// 換鍵盤。組字中同 `Other` 吃下；沒有組字時交還宿主，但不結束數字（brief §4.7，同 librime
    /// 不記沒有字元的鍵）。
    Modifier,
    /// 觸控點一下字母鍵：各字母的機率，由殼經 `qim-layout` 依點擊位置與按鍵幾何算出（M4 票 06）。
    /// 容錯進核心之前只用機率最高的字母，其餘同 `Letter`；[`Session::inspect`] 看得到這組機率。
    Weighted(Weights),
}

impl Key {
    /// 由「種類 + 值」讀出按鍵，綁定收不了帶資料的列舉時用（網頁、Swift）；同 librime 能從字串讀按鍵。
    /// 種類是 `letter`、`punct`、`digit`、`space`、`enter`、`backspace`、`escape`、`pageUp`、`pageDown`、
    /// `left`、`right`、`toggleAscii`、`other`、`modifier`、`weighted`；`letter`、`punct` 的值是一個字元，`digit` 是
    /// `0`–`9`，`toggleAscii` 是 `left` 或 `right`，`weighted` 是 `字母:機率` 以逗號分隔（`a:0.62,s:0.38`，
    /// 見 [`Weights::new`]），其他種類沒有值。加一種鍵時這裡與殼的按鍵對照一起改。
    pub fn parse(kind: &str, value: Option<&str>) -> Result<Key, KeyError> {
        let error = || KeyError {
            kind: kind.to_owned(),
            value: value.map(str::to_owned),
        };
        let single = || {
            let mut chars = value?.chars();
            let c = chars.next()?;
            chars.next().is_none().then_some(c)
        };
        let key = match (kind, value) {
            ("letter", _) => Key::Letter(single().ok_or_else(error)?),
            ("punct", _) => Key::Punct(single().ok_or_else(error)?),
            ("digit", _) => {
                let digit = single().and_then(|c| c.to_digit(10)).ok_or_else(error)?;
                Key::Digit(digit as u8)
            }
            ("toggleAscii", Some("left")) => Key::ToggleAscii(ShiftKey::Left),
            ("toggleAscii", Some("right")) => Key::ToggleAscii(ShiftKey::Right),
            ("space", None) => Key::Space,
            ("enter", None) => Key::Enter,
            ("backspace", None) => Key::Backspace,
            ("escape", None) => Key::Escape,
            ("pageUp", None) => Key::PageUp,
            ("pageDown", None) => Key::PageDown,
            ("left", None) => Key::Left,
            ("right", None) => Key::Right,
            ("other", None) => Key::Other,
            ("modifier", None) => Key::Modifier,
            ("weighted", Some(value)) => Key::Weighted(Weights::parse(value).ok_or_else(error)?),
            _ => return Err(error()),
        };
        Ok(key)
    }

    /// 打出輸入碼裡的字元 `c` 要按的鍵（實體鍵盤）：分隔記號是 `Punct`，其餘是 `Letter`。
    /// 照輸入碼重打一次組字時用（基準、測試）。
    pub fn for_input(c: char) -> Key {
        if c == SEPARATOR {
            Key::Punct(c)
        } else {
            Key::Letter(c)
        }
    }
}

/// [`Key::parse`] 讀不懂的種類或值。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyError {
    kind: String,
    value: Option<String>,
}

impl std::fmt::Display for KeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            Some(value) => write!(f, "不認得的按鍵：{} {value:?}", self.kind),
            None => write!(f, "不認得的按鍵：{}（沒有值）", self.kind),
        }
    }
}

impl std::error::Error for KeyError {}

/// 組字狀態。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Composition {
    /// 已打的輸入碼。
    pub input: String,
    /// 連打的各分段：選定的在前，接著剩下的輸入碼的首選路徑；碼接起來是 `input`。
    pub segments: Vec<Segment>,
    /// 目前分段在 `segments` 的位置，候選列它的字詞（← / →、[`Session::focus_segment`]）。
    /// 等於 `segments.len()` 時沒有目前分段：候選列剩下沒選定的整串輸入碼，是組字的預設。
    pub focus: usize,
}

impl Composition {
    /// 已選定的分段接起來的字詞。
    pub fn selected_text(&self) -> String {
        self.segments
            .iter()
            .filter(|s| s.selected)
            .map(|s| s.text.as_str())
            .collect()
    }

    /// 還沒選定的輸入碼：整串輸入碼去掉開頭選定的分段的碼。
    pub fn unselected_input(&self) -> &str {
        let selected: usize = self
            .segments
            .iter()
            .filter(|s| s.selected)
            .map(|s| s.code.len())
            .sum();
        &self.input[selected..]
    }

    /// 還沒選定的輸入碼的字根，如 `anr` 的「日弓口」；殼在選定的字詞後面顯示它（brief §4.3）。
    pub fn unselected_radicals(&self) -> String {
        self.segments
            .iter()
            .filter(|s| !s.selected)
            .map(|s| s.radicals.as_str())
            .collect()
    }
}

/// 連打的一個分段：一段輸入碼和它的字詞。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Segment {
    pub code: String,
    /// `code` 的字根，如 `anr` 的「日弓口」（方案的 `[radicals]`）。
    pub radicals: String,
    /// 顯示、上屏的字詞，經過後處理（簡體模式轉成簡體）。
    pub text: String,
    /// 後處理前的原文（繁體），字根提示與回報用；簡體模式以外和 `text` 相同。
    pub original: String,
    /// 使用者已選定，重算時不動。
    pub selected: bool,
    /// 臨時英文的分段（Rime 的 `raw` 標記）：`code` 原樣當英文，`text` 就是它，沒有字根。
    pub raw: bool,
}

impl Segment {
    /// 核心內部的分段：只有碼與原文的字詞（lattice 的路徑、選定的分段、學習的鍵）。字根、
    /// 後處理與 `original` 在回傳快照前才補上（`Session::composition`、`with_radicals`）。
    pub(crate) fn new(code: String, text: String) -> Self {
        Self {
            code,
            text,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// 顯示、上屏的字詞，經過後處理（簡體模式轉成簡體）。
    pub text: String,
    /// 後處理前的原文（繁體），字根提示與回報用；簡體模式以外和 `text` 相同。
    pub original: String,
}

/// 一次事件後殼要畫的全部狀態。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Snapshot {
    /// 核心吃下了這個鍵；為假時殼把鍵交還宿主（librime `process_key` 的回傳值）。
    pub handled: bool,
    pub composition: Composition,
    /// 目前頁的候選。
    pub candidates: Vec<Candidate>,
    pub page: usize,
    pub has_more: bool,
    /// 這次事件要上屏的文字。
    pub commit: Option<String>,
    /// `commit` 後處理前的原文（繁體），回報用；簡體模式以外和 `commit` 相同。
    pub commit_original: Option<String>,
    /// 英文：沒有組字時是英文直通，鍵都交還宿主；組字中是臨時英文（[`ShiftSwitch`]），
    /// 最後一段是 `raw` 的分段。
    pub ascii: bool,
    /// 每次候選變動加一，非同步重排用。
    pub generation: u64,
}

/// 後處理過的一個候選：排序時的候選（學習用）與轉換過的文字。
#[derive(Debug, Clone)]
struct ShownCandidate {
    span: CandidateSpan,
    /// 簡體模式轉換後的文字；不轉換時是 `None`，顯示原文。
    converted: Option<String>,
    /// 停鍵重排放到這裡的（[`Session::apply_rerank`]）；debug 面板顯示。
    rerank: Option<Reranked>,
}

impl ShownCandidate {
    /// 顯示、上屏的文字。
    fn text(&self) -> &str {
        self.converted.as_deref().unwrap_or(&self.span.text)
    }

    fn to_candidate(&self) -> Candidate {
        Candidate {
            text: self.text().to_string(),
            original: self.span.text.clone(),
        }
    }
}

/// 一次上屏：顯示的文字與後處理前的原文。
struct Commit {
    text: String,
    original: String,
}

impl Commit {
    /// 不經後處理的文字（標點、原樣上屏的輸入碼），兩者相同。
    fn plain(text: String) -> Self {
        Self {
            original: text.clone(),
            text,
        }
    }
}

/// 一個輸入框的組字狀態。
#[derive(Debug, Clone, Default)]
pub struct Session {
    /// 已選定的分段，依輸入碼的順序；重算時不動。
    selected: Vec<Segment>,
    /// 選定的分段之後、還沒選定的輸入碼的 lattice，逐鍵補算。
    lattice: Lattice,
    /// 目前分段在組字的分段的位置；`None` 是沒有目前分段，候選列剩下的整串輸入碼。
    focus: Option<usize>,
    /// 目前的全部候選，已經過後處理：有目前分段時是那一段的碼的，否則是剩下的輸入碼的。
    candidates: Vec<ShownCandidate>,
    /// 列 `candidates` 時的選項；影響候選的選項不同時下一個鍵先重列。
    listed_with: Options,
    /// 目前頁，從 0 起。
    page: usize,
    generation: u64,
    options: Options,
    /// 英文直通（[`Key::ToggleAscii`]）：沒有組字、鍵都交還宿主。
    ascii: bool,
    /// 前文（plan §2.1，名詞表），最多 [`CONTEXT_LIMIT`] 字：殼給的前文（[`Session::set_context`]）
    /// 接上之後核心送出去的字（上屏的、交還宿主打的可見字元與空白）。殼沒給過時只有核心送出去的，
    /// 同 librime 的 commit history。`None` 是不知道：沒給過也還沒送出過字、交還宿主的鍵改了游標前的
    /// 字或移走游標（[`Session::passed_to_host`]）、殼換了欄位（[`Session::clear_context`]），或在
    /// 無痕中：無痕時一個字都不記，關掉後也不會冒出無痕期間的字（brief §8）。
    context: Option<String>,
    /// 上一次上屏的組字的最後一段。前文以它結尾時，它是下一次組字的前一個字詞
    /// （[`Session::context_word`]）。
    last_word: Option<LastWord>,
    /// 這次組字開頭時前文給的前一個字詞（[`Session::context_word`]）；學個人 bigram 用，和打分時
    /// 接的相同：組字中再給的前文留到下一次組字（plan §2.1）。
    composition_word: Option<String>,
    /// 游標前的字是半形數字：數字後的 `.` `:` 上屏半形（brief §4.7）。和前文同時更新，但只記
    /// 是不是數字、不記文字，所以無痕時也照記，無痕的欄位一樣打得出「3.5」。
    after_digit: bool,
    /// 正在打的一串字（英文使用者詞，plan §2.1）：送出去的字的結尾連續的串字元。遇到其他字元、
    /// 交還宿主的 Enter 這類鍵或離開欄位（[`Session::clear_context`]）就是打完了，學成英文使用者詞
    /// （[`Session::end_typed`]）；邊界的規則在 [`TypedRun`]。
    typed: TypedRun,
    /// 候選列是聯想（[`Options::association`]）：沒有組字，`candidates` 是聯想。選了一個就換成下一輪；
    /// 其他鍵（不送字元的修飾鍵除外）、`text`、[`Session::clear_context`]、不以剛上屏的字結尾的
    /// [`Session::set_context`] 收起它。
    association: Option<Association>,
    /// 臨時英文（Rime `inline_ascii`）：組字中單按留碼的 Shift 切到的狀態，存的是英文。
    /// 沒選定的輸入碼原樣當英文，接著打的可見字元與空白接在後面；選定的分段照舊在前面。
    /// 中文的組字（選定的分段、lattice）留著，英文沒動過時切回不必重算。
    inline: Option<String>,
    /// 翻到後面的頁時收到的重排結果：那時的 `generation` 與 `order`。回到第一頁時 `generation`
    /// 沒變（候選沒變）才套用（[`Session::apply_rerank`]）。
    waiting_rerank: Option<(u64, Vec<usize>)>,
    /// 最近一個鍵是觸控的 [`Key::Weighted`] 時它的各字母機率，給 debug 面板（[`Session::inspect`]）；
    /// 其他鍵與 [`Session::text`] 清掉。容錯的票再決定要不要逐碼記。
    touch_weights: Option<Weights>,
}

impl Session {
    /// 換選項。組字中要畫面立刻跟著換，接著呼叫 [`Session::refresh`]。沒呼叫時，下一個
    /// `handle`、`select`、`focus_segment` 或 `text` 先依新選項重列候選再處理；重列前
    /// `all_candidates` 是空的，`rerank_request` 與 `apply_rerank` 回 `None`。
    pub fn set_options(&mut self, options: Options) {
        // 無痕不讀上下文（brief §8）：打開時丟掉前文，無痕期間也不記（`Session::sent`）。
        if options.incognito && !self.options.incognito {
            self.forget_context();
        }
        self.options = options;
    }

    /// 輸入框游標前的文字（plan §2.1），取代核心記著的；只留最後 [`CONTEXT_LIMIT`] 字。組字開頭的
    /// 字接它最後一字的 bigram，以標點、空白、符號結尾或是空的時接句首，最後一字在語言模型裡沒有
    /// 統計時用 unigram（plan §2.3、[`Engine::context_char`]）；數字後半形看它的最後一字。殼在聚焦、
    /// 移動游標、上屏之後給；組字中給的由下一次從頭建組字時用（新的組字、選定的分段都退回去時）。
    /// 之後上屏的字與交還宿主打的字元接在後面，殼不給時也知道游標前是什麼（同 librime 的 commit
    /// history）。無痕模式殼不給，給了也不收（brief §8）。
    pub fn set_context(&mut self, before: &str) {
        if self.options.incognito {
            return;
        }
        self.after_digit = before.ends_with(|c: char| c.is_ascii_digit());
        self.context = Some(last_chars(before, CONTEXT_LIMIT).to_string());
        self.typed.sync(before);
        // 游標前不是剛上屏的字：殼點了別處、移了游標，聯想接不上了。
        if self
            .association
            .as_ref()
            .is_some_and(|a| !before.ends_with(&a.shown))
        {
            self.end_association();
        }
    }

    /// 忘掉前文（不知道游標前是什麼）：殼換到別的欄位、輸入框失焦，又沒有前文可給時呼叫，上一個
    /// 欄位上屏的字就不會被當成前文。有前文可給時直接 [`Session::set_context`]，同樣整份取代。
    /// 離開欄位也是打完了一串字：正在打的串學成英文使用者詞（plan §2.4）。
    pub fn clear_context(&mut self, user: &mut UserModel) {
        self.end_typed(user);
        self.forget_context();
    }

    /// 不知道游標前是什麼了：前文與數字後半形的狀態都作廢；剛上屏的字不在游標前了，聯想也收起。
    fn forget_context(&mut self) {
        self.context = None;
        self.after_digit = false;
        self.end_association();
    }

    /// 正在打的一串字打完了：整串看得到又學得了（[`Session::learns_english`]）時學成英文使用者詞。
    fn end_typed(&mut self, user: &mut UserModel) {
        let done = self.typed.finish();
        self.learn_english(user, done);
    }

    fn learn_english(&self, user: &mut UserModel, done: Option<String>) {
        if let Some(text) = done.filter(|_| self.learns_english()) {
            user.learn_english(&text);
        }
    }

    /// 現在打的串學不學：在學習（不在無痕、學習沒關）、英文使用者詞開著。
    fn learns_english(&self) -> bool {
        self.writes_learning() && self.options.english_user_words
    }

    /// 目前的選項（[`Session::set_options`] 設的）；殼改一項時讀出來改，不必另存一份。
    pub fn options(&self) -> Options {
        self.options
    }

    /// 依目前的選項重列候選、回到第一頁，組字不變；殼換了選項後呼叫（librime 換選項後也
    /// 重算組字）。不是按鍵，`handled` 恆為真。
    pub fn refresh(&mut self, engine: &Engine, user: &UserModel) -> Snapshot {
        if self.composing_chinese() {
            self.refresh_candidates(engine, user);
        } else if self.association.is_some() {
            if self.options.association {
                self.list_association(engine, user);
            } else {
                self.end_association();
            }
        }
        with_radicals(engine, self.snapshot(engine, true, None))
    }

    /// 選全部候選（不只目前頁）的第 `index` 個，和按那個候選的數字鍵相同。
    pub fn select(&mut self, engine: &Engine, user: &mut UserModel, index: usize) -> Snapshot {
        self.apply_pending_options(engine, user);
        let snapshot = if self.composing() {
            self.select_at(engine, user, index)
        } else if self.showing_association() {
            self.select_association(engine, user, index)
        } else {
            // 只呼叫 `set_options` 關掉聯想、還沒 `refresh` 時，照沒有聯想收起。
            self.end_association();
            self.snapshot(engine, false, None)
        };
        self.finish(engine, user, snapshot)
    }

    /// 目前分段換成第 `segment` 段，候選改列那一段的字詞；`segment` 是分段數時回到沒有
    /// 目前分段（候選列剩下的整串）。超出範圍時不動作。觸控點組字的某一段用它（brief §4.4）。
    pub fn focus_segment(
        &mut self,
        engine: &Engine,
        user: &mut UserModel,
        segment: usize,
    ) -> Snapshot {
        self.apply_pending_options(engine, user);
        if !self.composing() {
            return self.snapshot(engine, false, None);
        }
        if self.composing_chinese() && segment <= self.segments().len() {
            self.move_focus(engine, user, segment);
        }
        with_radicals(engine, self.snapshot(engine, true, None))
    }

    /// 全部候選，不分頁、經過後處理，依目前的順序；候選展開面板用（brief §4.4，librime 的
    /// `RimeCandidateListBegin`、fcitx5 的 `BulkCandidateList`）。選字照位置呼叫 [`Session::select`]。
    /// 換了選項還沒重列時是空的：`select` 會先依新選項重列，舊列表的位置會選錯字。
    pub fn all_candidates(&self) -> Vec<Candidate> {
        if self.options_pending() {
            return Vec::new();
        }
        self.candidates
            .iter()
            .map(ShownCandidate::to_candidate)
            .collect()
    }

    /// lattice 蓋滿沒選定的輸入碼的前 `limit` 條路徑與分數，依分數由高到低，最多方案的 beam 條
    /// （plan §2.3）。和候選列不同：多段路徑不受方案的 `sentences` 限制，也不套排位規則。模型階梯
    /// 實驗讀它看重排模型能從幾條路徑裡挑（M3 票 07）；沒有組字時是空的。
    pub fn paths(&self, limit: usize) -> Vec<ScoredPath> {
        self.lattice.whole_paths(limit)
    }

    /// 給背景模型的重排要求（plan §2.1、§2.7）：目前的 `generation`、前文與 lattice 的前
    /// [`RERANK_PATHS`] 條整句路徑。沒有組字（聯想不重排）、臨時英文、移到某一段改字（候選是那一段
    /// 單打的字詞，整句插不進去）、沒有蓋滿的路徑，或換了選項還沒重列（候選列馬上會換）時是 `None`。
    /// 簡體模式下殼給的前文是簡體字，逐字轉回繁體（停鍵層模型的字表是繁體，M3 票 11e）。
    pub fn rerank_request(&self, engine: &Engine) -> Option<RerankRequest> {
        if !self.can_rerank() {
            return None;
        }
        let paths = self.lattice.whole_paths(RERANK_PATHS);
        if paths.is_empty() {
            return None;
        }
        // 路徑前面的字：前文（無痕時沒有）接上選定的分段。
        let mut context = self.context.clone().unwrap_or_default();
        if self.options.simplified {
            context = engine.traditional(&context);
        }
        context.extend(self.selected.iter().map(|s| s.text.as_str()));
        Some(RerankRequest {
            generation: self.generation,
            context: last_chars(&context, RERANK_CONTEXT).to_string(),
            paths,
        })
    }

    /// 套用背景模型的重排結果（plan §2.1、§2.7、ADR 0005）：`order` 是 [`RerankRequest::paths`]
    /// 的序號，好的在前。取第一條和首選文字不同的路徑（超出範圍的序號不算）放到第 2 位：排位規則
    /// 排在第 2 位起的候選不讓位，它排在那些後面（plan §2.4）。候選列有同樣文字的就移上來，沒有就
    /// 插入那條路徑，其餘照原順序往後接；首選不動。
    ///
    /// `generation` 不是目前的、換了選項還沒重列，或現在不給要求（[`Session::rerank_request`]），
    /// 結果是給舊候選列的，就丟掉、回 `None`。名次真的變了才算候選變動、`generation` 加一，同一份
    /// 要求的另一個結果（另一層模型）因此也作廢。殼不要因為套用了重排的結果再要求重排。翻到後面的頁
    /// 時先不套用、名次不變（插入會把第一頁最後一個擠到下一頁，看得到的那頁會變，M1 票 23），候選
    /// 沒變的話回到第一頁時才套用，那次翻頁的快照帶著新的 `generation`。
    pub fn apply_rerank(
        &mut self,
        engine: &Engine,
        generation: u64,
        order: &[usize],
    ) -> Option<Snapshot> {
        if generation != self.generation || !self.can_rerank() {
            return None;
        }
        if self.page > 0 {
            self.waiting_rerank = Some((generation, order.to_vec()));
        } else if self.raise(engine, order) {
            self.generation += 1;
        }
        Some(with_radicals(engine, self.snapshot(engine, true, None)))
    }

    /// 給重排要求、收重排結果：中文組字中、沒有目前分段、候選列是目前選項的。
    fn can_rerank(&self) -> bool {
        self.composing_chinese() && self.focus.is_none() && !self.options_pending()
    }

    /// 把 `order` 排出的路徑放到重排的位置（[`Session::apply_rerank`]）；名次變了時回真。
    fn raise(&mut self, engine: &Engine, order: &[usize]) -> bool {
        let Some(top) = self.candidates.first().map(|c| c.text().to_string()) else {
            return false;
        };
        let Some((k, path)) = order
            .iter()
            .filter(|&&k| k < RERANK_PATHS)
            .filter_map(|&k| Some((k, self.lattice.whole_path(k)?)))
            .map(|(k, span)| {
                let shown = ShownCandidate {
                    converted: self.options.simplified.then(|| self.display(engine, &span)),
                    span,
                    rerank: None,
                };
                (k, shown)
            })
            .find(|(_, c)| c.text() != top)
        else {
            return false;
        };
        // 排位規則排在第 2 位起的候選之後。
        let slot = 1 + self.candidates[1..]
            .iter()
            .take_while(|c| c.span.rule.is_some())
            .count();
        let (moved, inserted) = match self.candidates.iter().position(|c| c.text() == path.text()) {
            Some(at) if at <= slot => return false,
            Some(at) => (self.candidates.remove(at), false),
            None => (path, true),
        };
        let moved = ShownCandidate {
            rerank: Some(Reranked { path: k, inserted }),
            ..moved
        };
        self.candidates.insert(slot, moved);
        true
    }

    pub fn handle(&mut self, engine: &Engine, user: &mut UserModel, key: Key) -> Snapshot {
        self.apply_pending_options(engine, user);
        // 容錯進核心之前，觸控的機率只記給 debug 面板，鍵照機率最高的字母處理（M4 票 06）。
        let key = match key {
            Key::Weighted(weights) => {
                let top = weights.top();
                self.touch_weights = Some(weights);
                Key::Letter(top)
            }
            key => {
                self.touch_weights = None;
                key
            }
        };
        let snapshot = self.handle_key(engine, user, key.clone());
        if !snapshot.handled {
            self.passed_to_host(user, key);
        }
        self.finish(engine, user, snapshot)
    }

    /// 交還宿主的鍵宿主照打：可見字元與空白接在前文後面（librime 的 commit history 記這些
    /// `thru` 的鍵）。其他鍵改了游標前的字或移走游標，前文就作廢了，等殼再給；librime
    /// 只在退格與 Enter 清掉，不學它：方向鍵也把游標移走。不送字元的修飾鍵沒改輸入框，不動它。
    /// 正在打的一串字（英文使用者詞）：退格刪它的最後一個字元；Enter 讓它打完；其他作廢前文的鍵（Tab、
    /// 方向鍵…）也讓它打完，之後游標可能移走，接著打的到下一個分隔的字元才重新算。
    fn passed_to_host(&mut self, user: &mut UserModel, key: Key) {
        match key {
            Key::Letter(c) | Key::Punct(c) => self.sent(user, c.encode_utf8(&mut [0; 4])),
            Key::Digit(d) => self.sent(user, &d.to_string()),
            Key::Space => self.sent(user, " "),
            Key::Modifier => {}
            Key::Backspace => {
                self.typed.backspace();
                self.forget_context();
            }
            Key::Enter => self.clear_context(user),
            _ => {
                self.clear_context(user);
                self.typed.lose_track();
            }
        }
    }

    /// 核心送出去的字（上屏的、交還宿主打的字元）：記下游標前是不是數字；不在無痕中時接在前文後面，
    /// 只留最後 [`CONTEXT_LIMIT`] 字，不知道前文時就從它開始，也接在正在打的一串字後面（遇到串
    /// 裡不會有的字元時那串打完，[`Session::end_typed`]）。
    fn sent(&mut self, user: &mut UserModel, text: &str) {
        self.after_digit = text.ends_with(|c: char| c.is_ascii_digit());
        let recording = self.learns_english();
        for c in text.chars() {
            let done = self.typed.push(c, recording);
            self.learn_english(user, done);
        }
        if self.options.incognito {
            return;
        }
        let context = self.context.get_or_insert_with(String::new);
        context.push_str(text);
        let start = context.len() - last_chars(context, CONTEXT_LIMIT).len();
        context.drain(..start);
    }

    /// 換了影響候選的選項、還沒重列（[`Session::set_options`] 後沒呼叫 [`Session::refresh`]）：
    /// 候選列還是舊選項的。
    fn options_pending(&self) -> bool {
        self.composing_chinese() && !self.listed_with.lists_like(&self.options)
    }

    /// `handle`、`select`、`focus_segment`、`text` 開頭都經過這裡：有待生效的選項時先依
    /// 新選項重列候選，殼只呼叫 `set_options` 時選字、移分段、打文字才用得到新的候選列。
    /// 拿不到 `Engine` 的讀取方法改看 [`Session::options_pending`]，重列前不給舊候選。
    fn apply_pending_options(&mut self, engine: &Engine, user: &UserModel) {
        if self.options_pending() {
            self.refresh_candidates(engine, user);
        }
    }

    /// 會上屏的公開方法回傳前都經過這裡：上屏的字接在前文後面（[`Session::context`]，
    /// 同 librime 的 commit history），再補分段的字根。
    fn finish(&mut self, engine: &Engine, user: &mut UserModel, snapshot: Snapshot) -> Snapshot {
        if let Some(commit) = &snapshot.commit {
            self.sent(user, commit);
        }
        with_radicals(engine, snapshot)
    }

    fn handle_key(&mut self, engine: &Engine, user: &mut UserModel, key: Key) -> Snapshot {
        if self.association.is_some()
            && let Some(snapshot) = self.handle_association(engine, user, key.clone())
        {
            return snapshot;
        }
        if let Key::ToggleAscii(shift) = key {
            return self.toggle_ascii(engine, user, shift);
        }
        if self.inline.is_some() {
            return self.handle_inline(engine, user, key);
        }
        // 英文直通沒有組字，鍵都交還宿主。
        if self.ascii {
            return self.snapshot(engine, false, None);
        }
        if let Key::Letter(c) = key
            && engine.scheme().is_code_key(c)
        {
            self.focus = None;
            if self.lattice.input().is_empty() {
                if self.selected.is_empty() {
                    self.composition_word = self.context_word(engine);
                }
                self.lattice = Lattice::starting_after(self.before_unselected(engine));
            }
            self.lattice.push(engine, user, c);
            self.refresh_candidates(engine, user);
            return self.snapshot(engine, true, None);
        }
        // 沒有組字時，有全形的標點上屏全形，其他鍵都交還宿主：Backspace 刪輸入框的字、數字照打。
        if !self.composing() {
            return match key {
                Key::Punct(c)
                    if self.after_digit
                        && self.options.digit_separators
                        && punctuation::is_digit_separator(c) =>
                {
                    self.snapshot(engine, true, Some(Commit::plain(c.to_string())))
                }
                Key::Punct(c) if let Some(full) = punctuation::full_width(c) => {
                    self.snapshot(engine, true, Some(Commit::plain(full.to_string())))
                }
                _ => self.snapshot(engine, false, None),
            };
        }
        // 組字中每個鍵都吃下：交還宿主的話宿主照打那個字元，組字卻還留著。
        match key {
            // 分隔記號（brief §4.3）：接在剩下的輸入碼後面，像字母一樣回到整串；已經以它結尾時不再加。
            Key::Punct(SEPARATOR) => {
                if !self.lattice.input().ends_with(SEPARATOR) {
                    self.focus = None;
                    self.lattice.push(engine, user, SEPARATOR);
                    self.refresh_candidates(engine, user);
                }
                self.snapshot(engine, true, None)
            }
            Key::Backspace => {
                self.focus = None;
                self.lattice.pop();
                // 剩下的輸入碼刪光了，就把最後選定的分段放回來重選。
                if self.lattice.input().is_empty()
                    && let Some(segment) = self.selected.pop()
                {
                    self.recompose(engine, user, &segment.code);
                }
                self.refresh_candidates(engine, user);
                self.snapshot(engine, true, None)
            }
            Key::Escape => {
                self.clear();
                self.snapshot(engine, true, None)
            }
            // 空白一律選全部候選的第一個，與目前在第幾頁無關（brief §4.3）；
            // 有目前分段時上屏整個組字（照顯示的分段，剩下的逐次取首選）。
            Key::Space if self.focus.is_some() => {
                self.commit_all_then(engine, user, Commit::plain(String::new()))
            }
            Key::Space => self.select_at(engine, user, 0),
            Key::Digit(digit) => match usize::from(digit).checked_sub(1) {
                Some(index) if index < self.page_candidates().len() => {
                    self.select_at(engine, user, self.page * PAGE_SIZE + index)
                }
                _ => self.snapshot(engine, true, None),
            },
            Key::PageUp if self.page > 0 => self.turn_to(engine, self.page - 1),
            Key::PageDown if self.has_more() => self.turn_to(engine, self.page + 1),
            Key::PageUp | Key::PageDown => self.snapshot(engine, true, None),
            Key::Enter => {
                let mut commit = self.selected_commit(engine);
                commit.text.push_str(self.lattice.input());
                commit.original.push_str(self.lattice.input());
                self.commit(engine, commit)
            }
            Key::Punct(c) => {
                let punct = punctuation::full_width(c).map_or_else(|| c.to_string(), String::from);
                self.commit_all_then(engine, user, Commit::plain(punct))
            }
            Key::Letter(c) if !c.is_ascii_lowercase() => {
                self.commit_all_then(engine, user, Commit::plain(c.to_string()))
            }
            // 最前面的 ← 與最後面的 → 不動作。
            Key::Left => {
                if let Some(to) = self.focus_index().checked_sub(1) {
                    self.move_focus(engine, user, to);
                }
                self.snapshot(engine, true, None)
            }
            Key::Right => {
                let to = self.focus_index() + 1;
                if to <= self.segments().len() {
                    self.move_focus(engine, user, to);
                }
                self.snapshot(engine, true, None)
            }
            // 方案以外的小寫字母（方案沒設萬用鍵時的 `z`）、其他鍵：吃下不動作。
            // 切中英在上面處理過了。
            // `Weighted` 在 `handle` 已換成機率最高的字母，到不了這裡。
            Key::Letter(_)
            | Key::ToggleAscii(_)
            | Key::Other
            | Key::Modifier
            | Key::Weighted(_) => self.snapshot(engine, true, None),
        }
    }

    /// 聯想顯示中的鍵（plan §2.1），照 macOS 速成：數字鍵選這頁的聯想；Esc 與退格只收起聯想
    /// （librime-predict、Gboard 的第一下退格也是），不交還宿主；不送字元的修飾鍵不動它。其他鍵
    /// 收起聯想後照沒有聯想時處理（回 `None`）：字母開始新的組字，空白、Enter、方向鍵、沒有
    /// 那個聯想的數字交還宿主，標點照常上屏。
    fn handle_association(
        &mut self,
        engine: &Engine,
        user: &mut UserModel,
        key: Key,
    ) -> Option<Snapshot> {
        if !self.showing_association() {
            self.end_association();
            return None;
        }
        match key {
            Key::Modifier => Some(self.snapshot(engine, false, None)),
            Key::Digit(digit) => match usize::from(digit).checked_sub(1) {
                Some(index) if index < self.page_candidates().len() => {
                    Some(self.select_association(engine, user, self.page * PAGE_SIZE + index))
                }
                _ => {
                    self.end_association();
                    None
                }
            },
            Key::Escape | Key::Backspace => {
                self.end_association();
                Some(self.snapshot(engine, true, None))
            }
            _ => {
                self.end_association();
                None
            }
        }
    }

    /// 候選列是聯想、聯想的選項開著（殼只呼叫 `set_options` 關掉、還沒 `refresh` 時不算）。
    fn showing_association(&self) -> bool {
        self.association.is_some() && self.options.association
    }

    /// 選第 `index` 個聯想：上屏它，記個人 bigram「剛上屏的最後一段 → 它」（無痕、學習關掉時
    /// 不記），接著給下一輪。沒有這個聯想時不動作、鍵照樣吃下。
    fn select_association(
        &mut self,
        engine: &Engine,
        user: &mut UserModel,
        index: usize,
    ) -> Snapshot {
        let Some(choice) = self.candidates.get(index).cloned() else {
            return self.snapshot(engine, true, None);
        };
        let mut segments = self.association.take().expect("聯想顯示中").segments;
        let original = choice.span.text.clone();
        if self.writes_learning()
            && let Some(previous) = segments.last()
        {
            user.learn_association(previous, &original);
        }
        self.remember_last_word(engine, Some(&Segment::new(String::new(), original.clone())));
        let commit = Commit {
            text: choice.text().to_string(),
            original: original.clone(),
        };
        segments.push(original);
        self.commit_and_associate(engine, user, commit, segments)
    }

    /// 上屏 `commit`、清空組字，接著給接在 `segments`（剛上屏的各段，原文）後面的聯想。聯想關掉、
    /// 切到英文直通或無痕時不給：剛上屏的字是下一個字的前文，無痕不讀前文（brief §8）。
    fn commit_and_associate(
        &mut self,
        engine: &Engine,
        user: &UserModel,
        commit: Commit,
        mut segments: Vec<String>,
    ) -> Snapshot {
        self.clear();
        if self.options.association && !self.ascii && !self.options.incognito {
            // 留到蓋住最後 ASSOCIATION_TAIL 字的段，最後一段一定留。
            let mut chars = 0;
            let keep = segments
                .iter()
                .rev()
                .take_while(|s| {
                    let fits = chars < ASSOCIATION_TAIL;
                    chars += s.chars().count();
                    fits
                })
                .count();
            segments.drain(..segments.len() - keep);
            self.association = Some(Association {
                segments,
                shown: commit.text.clone(),
            });
            self.list_association(engine, user);
        }
        self.snapshot(engine, true, Some(commit))
    }

    /// 依目前的聯想與選項列候選；一個都沒有時收起。
    fn list_association(&mut self, engine: &Engine, user: &UserModel) {
        let Some(association) = &self.association else {
            return;
        };
        let texts = association::associations(engine, user, &association.segments);
        if texts.is_empty() {
            self.end_association();
            return;
        }
        let spans = texts
            .into_iter()
            .map(|text| CandidateSpan::single(text, ""))
            .collect();
        self.candidates = self.shown_candidates(engine, spans);
        self.listed_with = self.options;
        self.page = 0;
        self.generation += 1;
    }

    /// 收起聯想；沒有聯想時不動作。
    fn end_association(&mut self) {
        if self.association.take().is_some() {
            self.candidates.clear();
            self.page = 0;
            self.generation += 1;
        }
    }

    /// 切中英（單按 Shift，brief §4.3），照 Rime `ascii_composer`：沒有組字時只切換；組字中
    /// 依 [`ShiftSwitch`] 先上屏整個組字（`commit_text`）或留碼接英文（`inline_ascii`）。
    /// 臨時英文中按留碼的 Shift 照 librime 1.17.0：英文整串當中文的輸入碼重新
    /// 組字；英文含方案字母與分隔記號以外的字元時中文組字裝不下，和另一邊一樣上屏英文。兩種都回中文。
    fn toggle_ascii(&mut self, engine: &Engine, user: &mut UserModel, shift: ShiftKey) -> Snapshot {
        let keeps_code = self.options.shift_switch.keeps_code(shift);
        if self.inline.is_some() {
            let scheme = engine.scheme();
            let Some(english) = self
                .inline
                .take_if(|english| keeps_code && scheme.composes(english))
            else {
                return self.commit_inline(engine, Commit::plain(String::new()));
            };
            if english != self.lattice.input() {
                self.recompose(engine, user, &english);
            }
            self.refresh_candidates(engine, user);
            return self.snapshot(engine, true, None);
        }
        if !self.composing() {
            self.ascii = !self.ascii;
            return self.snapshot(engine, true, None);
        }
        // 組字中一定是中文：英文直通沒有組字。
        if !keeps_code {
            self.ascii = true;
            return self.commit_all_then(engine, user, Commit::plain(String::new()));
        }
        self.focus = None;
        self.inline = Some(self.lattice.input().to_string());
        // 臨時英文沒有候選，同 librime 抹掉中文的候選選單。
        self.candidates.clear();
        self.page = 0;
        self.generation += 1;
        self.snapshot(engine, true, None)
    }

    /// 臨時英文中的鍵，照 librime 1.17.0 的 `ascii_composer`：可見字元與空白接在英文
    /// 後面（數字、`-` / `=` 也是字元）；Enter 上屏英文；退格刪光或 Esc 放棄。三種都回中文。
    /// 其他鍵吃下不動作。
    fn handle_inline(&mut self, engine: &Engine, user: &mut UserModel, key: Key) -> Snapshot {
        let typed = match key {
            Key::Letter(c) | Key::Punct(c) => Some(c),
            Key::Digit(d) => char::from_digit(u32::from(d), 10),
            Key::Space => Some(' '),
            _ => None,
        };
        let english = self.inline.as_mut().expect("臨時英文中");
        match key {
            _ if typed.is_some() => {
                english.extend(typed);
                self.snapshot(engine, true, None)
            }
            Key::Enter => self.commit_inline(engine, Commit::plain(String::new())),
            Key::Backspace => {
                english.pop();
                if english.is_empty() {
                    self.inline = None;
                    // 和中文的退格相同：剩下的碼刪光了，最後選定的分段放回來重選。
                    self.lattice = Lattice::default();
                    match self.selected.pop() {
                        Some(segment) => {
                            self.recompose(engine, user, &segment.code);
                            self.refresh_candidates(engine, user);
                        }
                        None => self.clear(),
                    }
                }
                self.snapshot(engine, true, None)
            }
            Key::Escape => {
                self.inline = None;
                self.clear();
                self.snapshot(engine, true, None)
            }
            _ => self.snapshot(engine, true, None),
        }
    }

    /// 上屏選定的分段、臨時英文，再接 `tail`，回中文。和 Enter 一樣不學。
    fn commit_inline(&mut self, engine: &Engine, tail: Commit) -> Snapshot {
        let english = self.inline.take().expect("臨時英文中");
        let mut commit = self.selected_commit(engine);
        commit.text.push_str(&english);
        commit.text.push_str(&tail.text);
        commit.original.push_str(&english);
        commit.original.push_str(&tail.original);
        self.commit(engine, commit)
    }

    /// 中文組字中：組字中、不是臨時英文。只有這時才有候選、分段可移。
    fn composing_chinese(&self) -> bool {
        self.inline.is_none() && self.composing()
    }

    /// 在組字：有還沒上屏的輸入碼（臨時英文也算）。觸控鍵盤依它選按鍵的 `composing` 標籤與動作，
    /// 殼不必自己從快照記。
    pub fn composing(&self) -> bool {
        // 剩下的輸入碼選完就上屏，所以組字中它一定不是空的。
        !self.lattice.input().is_empty()
    }

    /// 組字的各分段：選定的分段接著剩下的輸入碼的首選路徑。
    /// 剩下的輸入碼沒有路徑蓋得滿時，整段當一個沒有字的分段。
    fn segments(&self) -> Vec<Segment> {
        let rest = self.lattice.input();
        let mut unselected = self.lattice.best_segments();
        if unselected.is_empty() && !rest.is_empty() {
            unselected.push(Segment {
                code: rest.to_string(),
                ..Segment::default()
            });
        }
        self.selected.iter().cloned().chain(unselected).collect()
    }

    /// [`Composition::focus`]：目前分段，沒有時是分段數。
    fn focus_index(&self) -> usize {
        self.focus.unwrap_or_else(|| self.segments().len())
    }

    /// 目前分段換成第 `to` 段（分段數以上是沒有目前分段），候選跟著換。
    fn move_focus(&mut self, engine: &Engine, user: &UserModel, to: usize) {
        let to = (to < self.segments().len()).then_some(to);
        if to != self.focus {
            self.focus = to;
            self.refresh_candidates(engine, user);
        }
    }

    /// 回到沒有目前分段，候選改回剩下的整串的。
    fn clear_focus(&mut self, engine: &Engine, user: &UserModel) {
        if self.focus.take().is_some() {
            self.refresh_candidates(engine, user);
        }
    }

    /// 直接輸入一段文字，不經組字（觸控鍵盤的 `text` 動作，plan §2.6）：沒有組字時上屏 `text`；
    /// 組字中和標點相同，先上屏整個組字再接 `text`；臨時英文中和實體鍵盤的可見字元一樣接在英文後
    /// （M1 票 27b）。文字照原樣、不轉全形，也不套數字後半形；簡體模式轉成簡體，原文留繁體（M1 票 23）。
    pub fn text(&mut self, engine: &Engine, user: &mut UserModel, text: &str) -> Snapshot {
        self.apply_pending_options(engine, user);
        self.end_association();
        self.touch_weights = None;
        let text = Commit {
            text: self.output(engine, text),
            original: text.to_string(),
        };
        let snapshot = if let Some(english) = &mut self.inline {
            english.push_str(&text.text);
            self.snapshot(engine, true, None)
        } else if self.composing() {
            self.commit_all_then(engine, user, text)
        } else {
            self.snapshot(engine, true, Some(text))
        };
        // 兩條路回來時組字都已清空，補字根現在沒有作用；照其他公開方法一樣經過它，組字留下時才不會漏。
        self.finish(engine, user, snapshot)
    }

    /// 上屏整個組字再接 `tail`：選定的分段，接著和一直按空白相同，逐次取剩下的輸入碼的首選；
    /// 剩下沒有字的碼原樣上屏。整個組字都有字時才學習，`tail` 不學。
    fn commit_all_then(&mut self, engine: &Engine, user: &mut UserModel, tail: Commit) -> Snapshot {
        // 剩下的輸入碼的首選要用它自己的候選，不是目前分段的。
        self.clear_focus(engine, user);
        let mut segments = self.selected.clone();
        let mut commit = self.selected_commit(engine);
        let mut rest = self.lattice.input().to_string();
        let mut top = self.candidates.first().map(|c| c.span.clone());
        let mut raw = String::new();
        while !rest.is_empty() {
            let Some(choice) = top else {
                raw = rest;
                break;
            };
            commit.text.push_str(&self.display(engine, &choice));
            commit.original.push_str(&choice.text);
            segments.extend_from_slice(&choice.segments);
            rest.drain(..choice.len);
            let before = Preceding {
                character: choice.text.chars().next_back(),
                word: choice.segments.last().map(|s| s.text.clone()),
            };
            top = lattice::candidates(
                engine,
                user,
                &Lattice::from_input(engine, user, before, &rest),
                self.listing(false),
            )
            .into_iter()
            .next();
        }
        if raw.is_empty() {
            self.learn(engine, user, &segments);
            self.remember_last_word(engine, segments.last());
        }
        // 只上屏組字（目前分段時的空白）才接聯想；接了標點、文字或原樣的碼不接（同 librime-predict
        // 遇到 `punct`、`raw` 的上屏就停）。
        if raw.is_empty() && tail.text.is_empty() {
            let texts = segments.iter().map(|s| s.text.clone()).collect();
            return self.commit_and_associate(engine, user, commit, texts);
        }
        commit.text.push_str(&raw);
        commit.text.push_str(&tail.text);
        commit.original.push_str(&raw);
        commit.original.push_str(&tail.original);
        self.commit(engine, commit)
    }

    /// 這次上屏要不要寫學習資料：無痕或學習關掉時不寫（plan §2.4）。
    fn writes_learning(&self) -> bool {
        self.options.learning && !self.options.incognito
    }

    /// 學一次上屏（plan §2.4）；無痕或學習關掉時不學。分段的碼去掉分隔記號：`mi'r` 選「歹口」記的是
    /// `mi` → 歹、`r` → 口與整串 `mir` → 歹口，同 Rime 使用者詞典的碼不含分隔符。有萬用鍵的碼換成
    /// 字詞實際的碼（[`Engine::learned_code`]），換不了時整次不學。個人 bigram 的第一段接組字開頭時
    /// 前文給的字詞（[`Session::composition_word`]）。
    fn learn(&self, engine: &Engine, user: &mut UserModel, segments: &[Segment]) {
        if !self.writes_learning() {
            return;
        }
        let Some(segments) = learned_segments(engine, user, segments) else {
            return;
        };
        let previous = self.composition_word.as_deref();
        user.learn(&engine.scheme().id, previous, &segments);
    }

    /// 記下上屏的組字的最後一段 `last`，給下一次組字接前文用（[`Session::context_word`]）；學習
    /// 關掉時也記，已學的照用。無痕不用前文，也不記。
    fn remember_last_word(&mut self, engine: &Engine, last: Option<&Segment>) {
        self.last_word = last.filter(|_| !self.options.incognito).map(|s| LastWord {
            original: s.text.clone(),
            shown: self.output(engine, &s.text),
        });
    }

    /// 把這次上屏的整串輸入碼 → 文字記成使用者詞；碼表或詞庫已有的不記（libime 也略過已有的
    /// 詞，`docs/research/libime-user-model.md` §5）。無痕或學習關掉時不記。碼同 [`Session::learn`]
    /// 去掉分隔記號、萬用鍵換成實際的碼，換不了時不記。
    fn add_auto_phrase(&self, engine: &Engine, user: &mut UserModel, segments: &[Segment]) {
        if !self.writes_learning() {
            return;
        }
        let Some(segments) = learned_segments(engine, user, segments) else {
            return;
        };
        let code: String = segments.iter().map(|s| s.code.as_str()).collect();
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        if !engine.has_entry(&code, &text) {
            user.add_auto_phrase(&engine.scheme().id, &code, &text);
        }
    }

    /// 列候選的選項：簡體字排後照選項（簡體模式不排）；英文候選與 emoji 照選項，但只給剩下的
    /// 整串輸入碼（`whole_input`），目前分段與逐段取首選時沒有。
    fn listing(&self, whole_input: bool) -> Listing {
        let options = &self.options;
        Listing {
            english_candidates: whole_input && options.english_candidates,
            emoji_candidates: whole_input && options.emoji_candidates,
            english_user_words: whole_input && options.english_user_words,
            simplified_last: options.simplified_after_traditional && !options.simplified,
        }
    }

    /// 已選定的分段接起來：顯示的字詞與原文。
    fn selected_commit(&self, engine: &Engine) -> Commit {
        Commit {
            text: self
                .selected
                .iter()
                .map(|s| self.output(engine, &s.text))
                .collect(),
            original: self.selected.iter().map(|s| s.text.as_str()).collect(),
        }
    }

    /// 字詞經過後處理（plan §2.2）要顯示、上屏的樣子：簡體模式轉成簡體。
    fn output(&self, engine: &Engine, text: &str) -> String {
        if self.options.simplified {
            engine.simplify(text)
        } else {
            text.to_string()
        }
    }

    /// 候選要顯示的文字：轉換的單位是一個分段，選了它之後組字與上屏也是各段分開轉換，
    /// 所以看到的就是上屏的。
    fn display(&self, engine: &Engine, span: &CandidateSpan) -> String {
        span.segments
            .iter()
            .map(|s| self.output(engine, &s.text))
            .collect()
    }

    /// 上屏 `commit`、清空組字。
    fn commit(&mut self, engine: &Engine, commit: Commit) -> Snapshot {
        self.clear();
        self.snapshot(engine, true, Some(commit))
    }

    /// 選全部候選的第 `index` 個，沒有這個候選時不動作、鍵照樣吃下。
    ///
    /// 沒有目前分段時候選是剩下的輸入碼的：選了蓋住的那段固定；目前分段是第 i 段時候選是它的：
    /// 前面的分段照顯示的字固定，第 i 段換成選的字詞。選的只蓋住第 i 段的開頭時，那段剩下的碼
    /// 連同後面的分段一起重算，否則後面已選定的分段不動、沒選定的重算。都選定了就上屏。
    fn select_at(&mut self, engine: &Engine, user: &mut UserModel, index: usize) -> Snapshot {
        let Some(choice) = self.candidates.get(index).map(|c| c.span.clone()) else {
            return self.snapshot(engine, true, None);
        };
        // 英文候選、emoji 與英文使用者詞蓋住剩下的整串（分段的候選列沒有它們）：和 Enter 一樣，選定的分段
        // 上屏字詞、其餘上屏它，整次不學也不造詞（plan §2.4）。
        if choice.kind.is_literal() {
            let mut commit = self.selected_commit(engine);
            commit.text.push_str(&choice.text);
            commit.original.push_str(&choice.text);
            return self.commit(engine, commit);
        }
        let segments = self.segments();
        // 分段在 `Session` 裡記原文（繁體），學習用；顯示與上屏時再轉換。
        // 候選列的是從第 `at` 段起的 `code`，`after` 是它後面的輸入碼。
        let (at, code, after) = match self.focus {
            Some(i) => (
                i,
                segments[i].code.clone(),
                segments[i + 1..].iter().map(|s| s.code.as_str()).collect(),
            ),
            None => (
                self.selected.len(),
                self.lattice.input().to_string(),
                String::new(),
            ),
        };
        // 這次之前已選定分段，或改選的是第 2 段以後：整個組字是分兩次以上選定的，整串上屏時
        // 自動造詞（plan §2.4）。
        let auto_phrase = at > 0 || !self.selected.is_empty();
        let mut fixed: Vec<Segment> = segments[..at]
            .iter()
            .map(|s| Segment {
                selected: true,
                ..s.clone()
            })
            .chain(choice.segments.iter().map(|s| Segment {
                selected: true,
                ..s.clone()
            }))
            .collect();
        let rest = if choice.len == code.len() && at < self.selected.len() {
            fixed.extend_from_slice(&self.selected[at + 1..]);
            self.lattice.input().to_string()
        } else {
            code[choice.len..].to_string() + &after
        };
        self.selected = fixed;
        self.focus = None;
        if rest.is_empty() {
            self.learn(engine, user, &self.selected);
            let last = self.selected.last().cloned();
            self.remember_last_word(engine, last.as_ref());
            if auto_phrase {
                self.add_auto_phrase(engine, user, &self.selected);
            }
            let commit = self.selected_commit(engine);
            let texts = self.selected.iter().map(|s| s.text.clone()).collect();
            return self.commit_and_associate(engine, user, commit, texts);
        }
        self.recompose(engine, user, &rest);
        self.refresh_candidates(engine, user);
        self.snapshot(engine, true, None)
    }

    /// 剩下的輸入碼換成 `input`，lattice 從頭建。
    fn recompose(&mut self, engine: &Engine, user: &UserModel, input: &str) {
        let before = self.before_unselected(engine);
        self.lattice = Lattice::from_input(engine, user, before, input);
    }

    /// 沒選定的輸入碼前面是什麼（plan §2.3）：最後選定的分段（它的最後一字與它）；沒有選定的
    /// 分段時看前文（[`Engine::context_char`]、[`Session::context_word`]），不知道前文或無痕時沒有。
    fn before_unselected(&self, engine: &Engine) -> Preceding {
        match (self.selected.last(), &self.context) {
            (Some(segment), _) => Preceding {
                character: segment.text.chars().next_back(),
                word: Some(segment.text.clone()),
            },
            (None, Some(context)) if !self.options.incognito => Preceding {
                character: engine.context_char(self.context_last_char(engine, context)),
                word: self.context_word(engine),
            },
            (None, _) => Preceding::default(),
        }
    }

    /// 前文給的前一個字詞（個人 bigram，plan §2.3）：前文以上一次上屏的組字的最後一段結尾時是
    /// 那一段（原文；簡體模式下前文是顯示的字，也認），否則是前文的最後一字（字母或數字才算，
    /// 標點、空白之後沒有；簡體模式轉回繁體，個人 bigram 記的是原文）。不知道前文或無痕時沒有。
    /// 前文由 `Session` 接上上屏的字（[`Session::context`]），所以剛上屏的那段接在前文結尾。
    fn context_word(&self, engine: &Engine) -> Option<String> {
        if self.options.incognito {
            return None;
        }
        let context = self.context.as_deref()?;
        if let Some(last) = self.last_word_ending(context) {
            return Some(last.original.clone());
        }
        self.context_last_char(engine, context)
            .filter(|c| c.is_alphanumeric())
            .map(String::from)
    }

    /// 前文 `context` 的最後一字，換成繁體原文（plan §2.3）：前文以上一次上屏的那一段結尾時是
    /// 那段原文的最後一字（簡體模式下一對多的字照實際打的，「幹」顯示「干」仍接「幹」）；否則
    /// 簡體模式下逐字轉回繁體（[`Engine::traditional_char`]，同停鍵層的前文，plan §2.7）。空的
    /// 前文是 `None`。
    fn context_last_char(&self, engine: &Engine, context: &str) -> Option<char> {
        if let Some(c) = self
            .last_word_ending(context)
            .and_then(|last| last.original.chars().next_back())
        {
            return Some(c);
        }
        let last = context.chars().next_back()?;
        Some(if self.options.simplified {
            engine.traditional_char(last)
        } else {
            last
        })
    }

    /// 上一次上屏的最後一段，前文 `context` 以它（原文或顯示的字）結尾時。
    fn last_word_ending(&self, context: &str) -> Option<&LastWord> {
        self.last_word
            .as_ref()
            .filter(|last| context.ends_with(&last.original) || context.ends_with(&last.shown))
    }

    fn turn_to(&mut self, engine: &Engine, page: usize) -> Snapshot {
        // 翻頁不是候選變動，`generation` 不加（M1 票 23）；回到第一頁時套用翻頁中收到的重排結果。
        self.page = page;
        if page == 0
            && let Some((generation, order)) = self.waiting_rerank.take()
            && generation == self.generation
            && self.can_rerank()
            && self.raise(engine, &order)
        {
            self.generation += 1;
        }
        self.snapshot(engine, true, None)
    }

    /// 輸入碼、目前分段或選項變了：重列候選、回到第一頁。目前分段的候選從頭建，和單打它的碼
    /// 相同，但沒有英文候選：它只給剩下的整串輸入碼。後處理（plan §2.2）在排序之後：簡體模式
    /// 轉成簡體，轉換後文字重複的只留第一個。
    fn refresh_candidates(&mut self, engine: &Engine, user: &UserModel) {
        let spans = match self.focus {
            Some(i) => {
                let code = &self.segments()[i].code;
                lattice::candidates(
                    engine,
                    user,
                    &Lattice::from_input(engine, user, Preceding::default(), code),
                    self.listing(false),
                )
            }
            None => lattice::candidates(engine, user, &self.lattice, self.listing(true)),
        };
        self.candidates = self.shown_candidates(engine, spans);
        self.listed_with = self.options;
        self.page = 0;
        self.generation += 1;
    }

    /// 候選經過後處理（plan §2.2）：簡體模式轉成簡體，轉換後文字重複的只留第一個。排序時已去掉
    /// 原文重複的，沒轉換的不必再比。
    fn shown_candidates(&self, engine: &Engine, spans: Vec<CandidateSpan>) -> Vec<ShownCandidate> {
        let mut seen = HashSet::new();
        spans
            .into_iter()
            .map(|span| ShownCandidate {
                converted: self.options.simplified.then(|| self.display(engine, &span)),
                span,
                rerank: None,
            })
            .filter(|c| c.converted.as_ref().is_none_or(|t| seen.insert(t.clone())))
            .collect()
    }

    fn clear(&mut self) {
        self.association = None;
        self.selected.clear();
        self.lattice = Lattice::default();
        self.focus = None;
        self.candidates.clear();
        self.page = 0;
        self.generation += 1;
    }

    /// 目前頁的候選。
    fn page_candidates(&self) -> &[ShownCandidate] {
        let start = self.page * PAGE_SIZE;
        &self.candidates[start..self.candidates.len().min(start + PAGE_SIZE)]
    }

    fn has_more(&self) -> bool {
        self.candidates.len() > (self.page + 1) * PAGE_SIZE
    }

    /// 組字：各分段的字詞經過後處理，另留原文。
    fn composition(&self, engine: &Engine) -> Composition {
        let segments = match &self.inline {
            Some(english) => self
                .selected
                .iter()
                .cloned()
                .chain([Segment {
                    code: english.clone(),
                    text: english.clone(),
                    raw: true,
                    ..Segment::default()
                }])
                .collect(),
            None => self.segments(),
        };
        let segments: Vec<Segment> = segments
            .into_iter()
            .map(|s| Segment {
                text: self.output(engine, &s.text),
                original: s.text.clone(),
                ..s
            })
            .collect();
        Composition {
            input: segments.iter().map(|s| s.code.as_str()).collect(),
            focus: self.focus.unwrap_or(segments.len()),
            segments,
        }
    }

    fn snapshot(&self, engine: &Engine, handled: bool, commit: Option<Commit>) -> Snapshot {
        let (commit, commit_original) = match commit {
            Some(c) => (Some(c.text), Some(c.original)),
            None => (None, None),
        };
        Snapshot {
            handled,
            composition: self.composition(engine),
            candidates: self
                .page_candidates()
                .iter()
                .map(ShownCandidate::to_candidate)
                .collect(),
            page: self.page,
            has_more: self.has_more(),
            commit,
            commit_original,
            ascii: self.ascii || self.inline.is_some(),
            generation: self.generation,
        }
    }
}

/// 學習記的分段（plan §2.4）：碼去掉分隔記號，有萬用鍵的換成字詞實際的碼
/// （[`Engine::learned_code`]）；有一段換不了時是 `None`。
fn learned_segments(
    engine: &Engine,
    user: &UserModel,
    segments: &[Segment],
) -> Option<Vec<Segment>> {
    segments
        .iter()
        .map(|s| {
            let code = engine.learned_code(user, &without_separators(&s.code), &s.text)?;
            Some(Segment::new(code, s.text.clone()))
        })
        .collect()
}

/// 補上各分段的字根：分段在 `Session` 裡不帶字根，公開的方法回傳前都經過這裡。
fn with_radicals(engine: &Engine, mut snapshot: Snapshot) -> Snapshot {
    for segment in snapshot.composition.segments.iter_mut().filter(|s| !s.raw) {
        segment.radicals = engine.scheme().radicals_of(&segment.code);
    }
    snapshot
}

/// `text` 的最後 `limit`（≥ 1）個字（Unicode 純量值）；不到 `limit` 個時是整串。
fn last_chars(text: &str, limit: usize) -> &str {
    match text.char_indices().nth_back(limit - 1) {
        Some((start, _)) => &text[start..],
        None => text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_chars_keeps_the_tail_by_characters() {
        assert_eq!(last_chars("今天天氣", 2), "天氣");
        assert_eq!(last_chars("今天", 5), "今天");
        assert_eq!(last_chars("", 3), "");
    }

    /// 前文與之後送出去的字合起來只留最後 [`CONTEXT_LIMIT`] 字。
    #[test]
    fn the_context_keeps_only_the_last_limit_characters() {
        let mut session = Session::default();
        session.set_context(&"天".repeat(CONTEXT_LIMIT + 5));
        assert_eq!(
            session.context.as_deref().map(|c| c.chars().count()),
            Some(CONTEXT_LIMIT)
        );
        session.sent(&mut UserModel::default(), "今日");
        let context = session.context.as_deref().unwrap();
        assert_eq!(context.chars().count(), CONTEXT_LIMIT);
        assert!(context.ends_with("天今日"));
    }
}
