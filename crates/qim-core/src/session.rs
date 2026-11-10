//! 殼與核心之間的介面（plan §2.1）：殼把按鍵交給 [`Session`]，拿回一份完整的 [`Snapshot`]。

use std::collections::HashSet;

use crate::lattice::{self, CandidateSpan, Lattice};
use crate::scheme::{SEPARATOR, without_separators};
use crate::{Engine, UserModel, punctuation};

/// 候選每頁的個數，對應數字鍵 1–9（brief §4.3）。
pub const PAGE_SIZE: usize = 9;

/// 重排要求最多帶幾個候選：第一頁。重排只動第一頁，翻到後面的頁時看得到的不會變。
pub const RERANK_LIMIT: usize = PAGE_SIZE;

/// 給背景模型的重排要求（[`Session::rerank_request`]）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RerankRequest {
    /// 結果交回 [`Session::apply_rerank`] 時帶著它；不是目前的就丟掉。
    pub generation: u64,
    /// 候選前面的字詞（繁體原文）：選定的分段，或目前分段前面的分段；輸入框的前文在 M3 加。
    pub context: String,
    /// 前 [`RERANK_LIMIT`] 個候選，依目前的順序。
    pub candidates: Vec<Candidate>,
}

/// 殼給的選項（plan §2.1），由 [`Session::set_options`] 換。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    /// 簡體輸出：候選、組字與上屏轉成簡體，學習仍記繁體（plan §2.2 的後處理）；
    /// 引擎要掛上簡繁資料包（[`Engine::with_simplifier`]）。
    pub simplified: bool,
    /// 無痕模式：不寫任何學習資料（plan §2.4）。
    pub incognito: bool,
    /// 中文狀態的英文候選（排位規則 4，plan §2.4）：整串輸入碼剛好是英文單字時排第 2 位；
    /// 引擎要掛上英文資料包（[`Engine::with_english`]）。預設開（brief §7）。
    pub english_candidates: bool,
    /// 數字後的 `.` `:` 用半形（brief §4.7）：前一個上屏的字是半形數字時直接上屏半形，同 librime
    /// `digit_separators` 預設加 `digit_separator_action: commit`；逗號照舊全形。預設開（brief §7）。
    pub digit_separators: bool,
    /// emoji 候選（plan §2.4）：整串的候選是 emoji 資料包裡的詞時，它的 emoji 接在它後面；
    /// 引擎要掛上 emoji 資料包（[`Engine::with_emoji`]）。預設開（brief §7，同 rime-emoji）。
    pub emoji_candidates: bool,
    /// 組字中單按左、右 Shift 怎麼處理（brief §4.3）；預設照 Rime。
    pub shift_switch: ShiftSwitch,
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
    /// 兩份選項列出的候選相同：只有簡體、英文候選與 emoji 候選影響候選，無痕、Shift、數字後半形
    /// 都不影響。
    fn lists_like(&self, other: &Options) -> bool {
        self.simplified == other.simplified
            && self.english_candidates == other.english_candidates
            && self.emoji_candidates == other.emoji_candidates
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            simplified: false,
            incognito: false,
            english_candidates: true,
            digit_separators: true,
            emoji_candidates: true,
            shift_switch: ShiftSwitch::Rime,
        }
    }
}

/// 殼轉過來的按鍵。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

impl Key {
    /// 由「種類 + 值」讀出按鍵，綁定收不了帶資料的列舉時用（網頁、Swift）；同 librime 能從字串讀按鍵。
    /// 種類是 `letter`、`punct`、`digit`、`space`、`enter`、`backspace`、`escape`、`pageUp`、`pageDown`、
    /// `left`、`right`、`toggleAscii`、`other`、`modifier`；`letter`、`punct` 的值是一個字元，`digit` 是 `0`–`9`，
    /// `toggleAscii` 是 `left` 或 `right`，其他種類沒有值。加一種鍵時這裡與殼的按鍵對照一起改。
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
    /// 最近一次送出去的文字（上屏或交還宿主打的鍵）以半形數字結尾。只看核心自己經手的：
    /// 輸入框的前文要等 M3 的 `set_context`，所以游標移到別處、宿主自己改了字都看不到。
    after_digit: bool,
    /// 臨時英文（Rime `inline_ascii`）：組字中單按留碼的 Shift 切到的狀態，存的是英文。
    /// 沒選定的輸入碼原樣當英文，接著打的可見字元與空白接在後面；選定的分段照舊在前面。
    /// 中文的組字（選定的分段、lattice）留著，英文沒動過時切回不必重算。
    inline: Option<String>,
}

impl Session {
    /// 換選項。組字中要畫面立刻跟著換，接著呼叫 [`Session::refresh`]。沒呼叫時，下一個
    /// `handle`、`select`、`focus_segment` 或 `text` 先依新選項重列候選再處理；重列前
    /// `all_candidates` 是空的，`rerank_request` 與 `apply_rerank` 回 `None`。
    pub fn set_options(&mut self, options: Options) {
        self.options = options;
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
        }
        with_radicals(engine, self.snapshot(engine, true, None))
    }

    /// 選全部候選（不只目前頁）的第 `index` 個，和按那個候選的數字鍵相同。
    pub fn select(&mut self, engine: &Engine, user: &mut UserModel, index: usize) -> Snapshot {
        self.apply_pending_options(engine, user);
        let snapshot = if self.composing() {
            self.select_at(engine, user, index)
        } else {
            self.snapshot(engine, false, None)
        };
        self.finish(engine, snapshot)
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

    /// 給背景模型的重排要求（plan §2.7）：目前的 `generation`、上下文與前 [`RERANK_LIMIT`]
    /// 個候選。沒有組字、沒有候選，或換了選項還沒重列（候選列馬上會換）時是 `None`。
    pub fn rerank_request(&self) -> Option<RerankRequest> {
        if self.candidates.is_empty() || self.options_pending() {
            return None;
        }
        // 上下文是候選所在的碼前面的分段：有目前分段時是它前面的，否則是選定的分段。
        let context = match self.focus {
            Some(i) => self.segments()[..i]
                .iter()
                .map(|s| s.text.as_str())
                .collect(),
            None => self.selected.iter().map(|s| s.text.as_str()).collect(),
        };
        Some(RerankRequest {
            generation: self.generation,
            context,
            candidates: self.candidates[..self.rerank_limit()]
                .iter()
                .map(ShownCandidate::to_candidate)
                .collect(),
        })
    }

    /// 套用背景模型的重排結果（plan §2.7、ADR 0005）：`order` 是 [`RerankRequest::candidates`]
    /// 的位置，好的在前。首選不動，`order` 裡其餘的照它的順序排在第 2 位起，沒提到的照原順序
    /// 接在後面；超出範圍、重複的位置不算。
    ///
    /// `generation` 不是目前的，或換了選項還沒重列（結果是給舊候選列的），就丟掉、
    /// 回 `None`。名次真的變了才算候選變動、`generation` 加一，
    /// 同一份要求的另一個結果（另一層模型）因此也作廢：它的位置是照重排前的順序。殼不要因為套用了
    /// 重排的結果再要求重排。翻頁不改 `generation`，翻到第 2 頁後收到的結果照樣套用：重排只動
    /// 第一頁，看得到的那頁不變（M1 票 23）。
    pub fn apply_rerank(
        &mut self,
        engine: &Engine,
        generation: u64,
        order: &[usize],
    ) -> Option<Snapshot> {
        if generation != self.generation || self.options_pending() {
            return None;
        }
        let limit = self.rerank_limit();
        let mut ranked: Vec<usize> = Vec::with_capacity(limit);
        ranked.extend((limit > 0).then_some(0));
        for i in order.iter().copied().chain(0..limit) {
            if i < limit && !ranked.contains(&i) {
                ranked.push(i);
            }
        }
        if ranked.iter().enumerate().any(|(at, &i)| at != i) {
            let top: Vec<ShownCandidate> =
                ranked.iter().map(|&i| self.candidates[i].clone()).collect();
            self.candidates.splice(..limit, top);
            self.generation += 1;
        }
        Some(with_radicals(engine, self.snapshot(engine, true, None)))
    }

    /// 重排看得到的候選數：最多 [`RERANK_LIMIT`] 個。
    fn rerank_limit(&self) -> usize {
        self.candidates.len().min(RERANK_LIMIT)
    }

    pub fn handle(&mut self, engine: &Engine, user: &mut UserModel, key: Key) -> Snapshot {
        self.apply_pending_options(engine, user);
        let snapshot = self.handle_key(engine, user, key);
        // 交還宿主的鍵宿主照打：只有數字鍵讓下一個字緊接在數字後。其他鍵都當成數字結束；librime
        // 只認可見字元、退格與 Enter，不學它：方向鍵也會把游標移離數字。不送字元的修飾鍵不動它。
        if !snapshot.handled && key != Key::Modifier {
            self.after_digit = matches!(key, Key::Digit(_));
        }
        self.finish(engine, snapshot)
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

    /// 會上屏的公開方法回傳前都經過這裡：記下上屏的字是不是以半形數字結尾
    /// （[`Session::after_digit`]，沒上屏時不變，同 librime 的 commit history），再補分段的字根。
    fn finish(&mut self, engine: &Engine, snapshot: Snapshot) -> Snapshot {
        if let Some(commit) = &snapshot.commit {
            self.after_digit = commit.ends_with(|c: char| c.is_ascii_digit());
        }
        with_radicals(engine, snapshot)
    }

    fn handle_key(&mut self, engine: &Engine, user: &mut UserModel, key: Key) -> Snapshot {
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
            && engine.scheme().alphabet.contains(c)
        {
            self.focus = None;
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
            // 方案以外的小寫字母（速成的 `z`，萬用鍵在 M4）、其他鍵：吃下不動作。
            // 切中英在上面處理過了。
            Key::Letter(_) | Key::ToggleAscii(_) | Key::Other | Key::Modifier => {
                self.snapshot(engine, true, None)
            }
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
        self.finish(engine, snapshot)
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
            top = lattice::candidates(
                engine,
                user,
                &Lattice::from_input(engine, user, &rest),
                false,
                false,
            )
            .into_iter()
            .next();
        }
        if raw.is_empty() {
            self.learn(engine, user, &segments);
        }
        commit.text.push_str(&raw);
        commit.text.push_str(&tail.text);
        commit.original.push_str(&raw);
        commit.original.push_str(&tail.original);
        self.commit(engine, commit)
    }

    /// 學一次上屏（plan §2.4）；無痕模式不學。分段的碼去掉分隔記號：`mi'r` 選「歹口」記的是
    /// `mi` → 歹、`r` → 口與整串 `mir` → 歹口，同 Rime 使用者詞典的碼不含分隔符。
    fn learn(&self, engine: &Engine, user: &mut UserModel, segments: &[Segment]) {
        // 沒有分段時不取方案的個人模型：取了會建出空的一份，匯出多一行 tick。
        if self.options.incognito || segments.is_empty() {
            return;
        }
        let segments: Vec<Segment> = segments
            .iter()
            .map(|s| Segment::new(without_separators(&s.code), s.text.clone()))
            .collect();
        user.for_scheme_mut(&engine.scheme().id).learn(&segments);
    }

    /// 把這次上屏的整串輸入碼 → 文字記成使用者詞；碼表或詞庫已有的不記（libime 也略過已有的
    /// 詞，`docs/research/libime-user-model.md` §5）。無痕模式不記。
    fn add_auto_phrase(&self, engine: &Engine, user: &mut UserModel, segments: &[Segment]) {
        if self.options.incognito {
            return;
        }
        let code: String = segments
            .iter()
            .map(|s| without_separators(&s.code))
            .collect();
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        if !engine.has_entry(&code, &text) {
            user.for_scheme_mut(&engine.scheme().id)
                .add_auto_phrase(&code, &text);
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
        // 英文候選與 emoji 蓋住剩下的整串（分段的候選列沒有它們）：和 Enter 一樣，選定的分段
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
            if auto_phrase {
                self.add_auto_phrase(engine, user, &self.selected);
            }
            let commit = self.selected_commit(engine);
            return self.commit(engine, commit);
        }
        self.recompose(engine, user, &rest);
        self.refresh_candidates(engine, user);
        self.snapshot(engine, true, None)
    }

    /// 剩下的輸入碼換成 `input`，lattice 從頭建。
    fn recompose(&mut self, engine: &Engine, user: &UserModel, input: &str) {
        self.lattice = Lattice::from_input(engine, user, input);
    }

    fn turn_to(&mut self, engine: &Engine, page: usize) -> Snapshot {
        // 翻頁不是候選變動，`generation` 不加（M1 票 23）。
        self.page = page;
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
                    &Lattice::from_input(engine, user, code),
                    false,
                    false,
                )
            }
            None => {
                let options = &self.options;
                lattice::candidates(
                    engine,
                    user,
                    &self.lattice,
                    options.english_candidates,
                    options.emoji_candidates,
                )
            }
        };
        // 排序時已去掉原文重複的，沒轉換的不必再比。
        let mut seen = HashSet::new();
        self.candidates = spans
            .into_iter()
            .map(|span| ShownCandidate {
                converted: self.options.simplified.then(|| self.display(engine, &span)),
                span,
            })
            .filter(|c| c.converted.as_ref().is_none_or(|t| seen.insert(t.clone())))
            .collect();
        self.listed_with = self.options;
        self.page = 0;
        self.generation += 1;
    }

    fn clear(&mut self) {
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

/// 補上各分段的字根：分段在 `Session` 裡不帶字根，公開的方法回傳前都經過這裡。
fn with_radicals(engine: &Engine, mut snapshot: Snapshot) -> Snapshot {
    for segment in snapshot.composition.segments.iter_mut().filter(|s| !s.raw) {
        segment.radicals = engine.scheme().radicals_of(&segment.code);
    }
    snapshot
}
