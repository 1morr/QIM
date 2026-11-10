//! 網頁用的 WebAssembly 綁定：只把 JS 的呼叫轉給 `qim-core` 與 `qim-layout`，不放邏輯（plan §1.3）。

use qim_core::{
    CangjieVersion, EmojiPack, Engine, EnglishPack, Key, Options, Pack, RadicalHintPack,
    RadicalHints, Scheme, Session, ShiftKey, SimplifierPack, Snapshot, UserModel, WordListReport,
};
use qim_layout::{Action, Keyboard, Output, Target, When};
use wasm_bindgen::prelude::*;

/// 核心與 `qim-layout` 的錯誤轉成 JS 的例外，訊息照原樣。
fn js_error(error: impl std::fmt::Display) -> JsError {
    JsError::new(&error.to_string())
}

/// 網頁的一個輸入框：速成、詞庫、語言模型、簡繁、英文與 emoji 資料包、一個個人模型、一個 `Session`。
/// JS 端叫 `Session`。
#[wasm_bindgen(js_name = Session)]
pub struct WebSession {
    engine: Engine,
    user: UserModel,
    session: Session,
}

#[wasm_bindgen(js_class = Session)]
impl WebSession {
    /// 由速成資料包與方案指名的詞庫、語言模型資料包，加上簡繁資料包（沒有時簡體模式照原樣
    /// 輸出）、英文資料包（沒有時沒有英文候選）、emoji 資料包（沒有時沒有 emoji 候選）的位元組
    /// 建立；資料包由網頁 fetch。
    #[wasm_bindgen(constructor)]
    pub fn new(
        scheme_pack: &[u8],
        lexicon: Option<Vec<u8>>,
        language_model: Option<Vec<u8>>,
        simplifier: Option<Vec<u8>>,
        english: Option<Vec<u8>>,
        emoji: Option<Vec<u8>>,
    ) -> Result<WebSession, JsError> {
        let mut engine =
            Engine::from_pack_bytes(scheme_pack, lexicon.as_deref(), language_model.as_deref())
                .map_err(js_error)?;
        if let Some(bytes) = simplifier {
            engine = engine.with_simplifier(SimplifierPack::from_bytes(&bytes).map_err(js_error)?);
        }
        if let Some(bytes) = english {
            engine = engine.with_english(EnglishPack::from_bytes(&bytes).map_err(js_error)?);
        }
        if let Some(bytes) = emoji {
            engine = engine.with_emoji(EmojiPack::from_bytes(&bytes).map_err(js_error)?);
        }
        Ok(Self {
            engine,
            user: UserModel::default(),
            session: Session::default(),
        })
    }

    /// 無痕模式：開著時不學習（plan §2.4）。
    #[wasm_bindgen(js_name = setIncognito)]
    pub fn set_incognito(&mut self, on: bool) {
        self.set_options(|options| options.incognito = on);
    }

    /// 手動加一個使用者詞；`trigger` 是空字串時不設觸發詞（brief §4.5）。
    #[wasm_bindgen(js_name = addWord)]
    pub fn add_word(&mut self, text: &str, trigger: &str) -> Result<(), JsError> {
        let trigger = (!trigger.is_empty()).then_some(trigger);
        self.user
            .add_word(&self.engine, text, trigger)
            .map_err(js_error)
    }

    /// 存一條常用語；`trigger` 是空字串時沒有觸發詞，只能從清單選（plan §2.4）。同樣的文字再存
    /// 一次換上新的觸發詞。
    #[wasm_bindgen(js_name = addPhrase)]
    pub fn add_phrase(&mut self, text: &str, trigger: &str) -> Result<(), JsError> {
        let trigger = (!trigger.is_empty()).then_some(trigger);
        self.user
            .add_phrase(&self.engine, text, trigger)
            .map_err(js_error)
    }

    /// 刪掉文字是 `text` 的常用語；沒有這條時回傳 `false`。
    #[wasm_bindgen(js_name = removePhrase)]
    pub fn remove_phrase(&mut self, text: &str) -> bool {
        self.user.remove_phrase(&self.engine, text)
    }

    /// 常用語的清單，依文字排。
    pub fn phrases(&self) -> Vec<WebPhrase> {
        self.user
            .phrases(&self.engine)
            .into_iter()
            .map(|p| WebPhrase {
                text: p.text,
                trigger: p.trigger.unwrap_or_default(),
            })
            .collect()
    }

    /// 從清單選了常用語：照觸控鍵盤的 `text` 上屏（組字中先上屏組字），回傳快照給網頁插入。
    #[wasm_bindgen(js_name = commitText)]
    pub fn commit_text(&mut self, text: &str) -> WebSnapshot {
        self.session.text(&self.engine, &mut self.user, text).into()
    }

    /// 使用者詞與學習計數匯出成 `# qim-userdict v2` 的 TSV（plan §2.4），網頁存 `localStorage`、
    /// 下載成檔。
    #[wasm_bindgen(js_name = exportUserDict)]
    pub fn export_user_dict(&self) -> String {
        self.user.export()
    }

    /// 檢查 `text` 是不是匯入得了的檔，不動目前的資料；網頁請使用者確認覆蓋之前先檢查。
    #[wasm_bindgen(js_name = checkUserDict)]
    pub fn check_user_dict(&self, text: &str) -> Result<(), JsError> {
        UserModel::import(text).map(|_| ()).map_err(js_error)
    }

    /// 讀匯出的檔，取代目前的使用者詞與學習計數；不合格式時整份不收、原本的不動。
    #[wasm_bindgen(js_name = importUserDict)]
    pub fn import_user_dict(&mut self, text: &str) -> Result<(), JsError> {
        self.user = UserModel::import(text).map_err(js_error)?;
        Ok(())
    }

    /// 先看匯入詞表 `text` 會加幾個詞、略過哪些列（在個人模型的複本上匯入），不動目前的資料；
    /// 網頁請使用者確認之前呼叫。整份讀不了時丟例外。
    #[wasm_bindgen(js_name = checkWordList)]
    pub fn check_word_list(&self, text: &str) -> Result<WebWordListReport, JsError> {
        let report = self
            .user
            .clone()
            .import_word_list(&self.engine, text)
            .map_err(js_error)?;
        Ok(report.into())
    }

    /// 把詞表 `text` 的詞加進使用者詞（plan §2.4）：Rime `.dict.yaml`、Gboard 個人字典、一行一詞。
    /// 是加進去，不取代學習資料；整份讀不了時丟例外、什麼都不加。
    #[wasm_bindgen(js_name = importWordList)]
    pub fn import_word_list(&mut self, text: &str) -> Result<WebWordListReport, JsError> {
        let report = self
            .user
            .import_word_list(&self.engine, text)
            .map_err(js_error)?;
        Ok(report.into())
    }

    /// 清掉全部使用者詞與學習計數。
    #[wasm_bindgen(js_name = clearUserDict)]
    pub fn clear_user_dict(&mut self) {
        self.user = UserModel::default();
    }

    /// 簡體模式（M1 票 16）：候選、組字與上屏轉成簡體，學習仍記繁體。組字中切換也立刻
    /// 重列候選，回傳新的狀態給網頁畫。
    #[wasm_bindgen(js_name = setSimplified)]
    pub fn set_simplified(&mut self, on: bool) -> WebSnapshot {
        self.set_options(|options| options.simplified = on);
        self.session.refresh(&self.engine, &self.user).into()
    }

    /// 中文狀態的英文候選（M1 票 15）：整串輸入碼剛好是英文單字時排第 2 位，預設開。組字中
    /// 切換也立刻重列候選。
    #[wasm_bindgen(js_name = setEnglishCandidates)]
    pub fn set_english_candidates(&mut self, on: bool) -> WebSnapshot {
        self.set_options(|options| options.english_candidates = on);
        self.session.refresh(&self.engine, &self.user).into()
    }

    /// emoji 候選（M3 票 16）：整串的候選是 emoji 資料包裡的詞時，它的 emoji 接在它後面，預設開。
    /// 組字中切換也立刻重列候選。
    #[wasm_bindgen(js_name = setEmojiCandidates)]
    pub fn set_emoji_candidates(&mut self, on: bool) -> WebSnapshot {
        self.set_options(|options| options.emoji_candidates = on);
        self.session.refresh(&self.engine, &self.user).into()
    }

    /// 數字後的 `.` `:` 用半形（M1 票 26，brief §4.7），預設開。只影響下一個標點，組字不必重列。
    #[wasm_bindgen(js_name = setDigitSeparators)]
    pub fn set_digit_separators(&mut self, on: bool) {
        self.set_options(|options| options.digit_separators = on);
    }

    /// 組字中單按 Shift 的處理（`qim_core::ShiftSwitch`）：`rime`（預設）、`commit-text`、
    /// `inline-ascii`。不是按鍵，臨時英文中換了也不動組字。
    #[wasm_bindgen(js_name = setShiftSwitch)]
    pub fn set_shift_switch(&mut self, value: &str) -> Result<(), JsError> {
        let shift_switch = value.parse().map_err(js_error)?;
        self.set_options(|options| options.shift_switch = shift_switch);
        Ok(())
    }

    /// 按一個鍵（實體鍵盤）：種類與值照 [`Key::parse`]，例如 `("letter", "a")`、`("digit", "1")`、
    /// `("toggleAscii", "left")`、`("space")`。網頁的按鍵都經這裡。
    pub fn press(&mut self, kind: &str, value: Option<String>) -> Result<WebSnapshot, JsError> {
        let key = Key::parse(kind, value.as_deref()).map_err(js_error)?;
        Ok(self.handle(key).into())
    }

    /// 打出輸入碼裡的字元 `c`（`Key::for_input`：分隔記號送標點鍵，其餘送字母）；照輸入碼
    /// 重打一次組字時用（WebAssembly 的每鍵計時）。
    #[wasm_bindgen(js_name = inputChar)]
    pub fn input_char(&mut self, c: char) -> WebSnapshot {
        self.handle(Key::for_input(c)).into()
    }

    /// 全部候選，不分頁（候選展開面板用）；選字照位置呼叫 [`WebSession::select`]。
    #[wasm_bindgen(js_name = allCandidates)]
    pub fn all_candidates(&self) -> Vec<String> {
        self.session
            .all_candidates()
            .into_iter()
            .map(|c| c.text)
            .collect()
    }

    /// 給背景模型的重排要求（plan §2.7）；沒有組字時是 `undefined`。
    #[wasm_bindgen(js_name = rerankRequest)]
    pub fn rerank_request(&self) -> Option<WebRerankRequest> {
        self.session.rerank_request().map(|r| WebRerankRequest {
            generation: r.generation,
            context: r.context,
            candidates: r.candidates.into_iter().map(|c| c.original).collect(),
        })
    }

    /// 套用重排結果：`order` 是要求的候選的位置，好的在前；`generation` 過期時是 `undefined`。
    #[wasm_bindgen(js_name = applyRerank)]
    pub fn apply_rerank(&mut self, generation: u64, order: Vec<usize>) -> Option<WebSnapshot> {
        self.session
            .apply_rerank(&self.engine, generation, &order)
            .map(Into::into)
    }

    /// 選全部候選的第 `index` 個（目前頁第 i 個是 `page × 9 + i`），例如點候選列。
    pub fn select(&mut self, index: usize) -> WebSnapshot {
        self.session
            .select(&self.engine, &mut self.user, index)
            .into()
    }

    /// 移到組字的第 `segment` 段，例如點組字裡的那一段；分段數是回到整串。
    #[wasm_bindgen(js_name = focusSegment)]
    pub fn focus_segment(&mut self, segment: usize) -> WebSnapshot {
        self.session
            .focus_segment(&self.engine, &mut self.user, segment)
            .into()
    }

    /// 點一下 `layout` 目前頁第 `index` 個鍵（[`WebLayout::hit`] 的結果），見 [`Keyboard::tap`]。
    /// `quick_repeat` 是同一個鍵剛點過（網頁量時間）。
    pub fn tap(
        &mut self,
        layout: &mut WebLayout,
        index: usize,
        quick_repeat: bool,
    ) -> Result<WebSnapshot, JsError> {
        let output = layout
            .keyboard
            .tap(index, self.session.composing(), quick_repeat)
            .map_err(js_error)?;
        Ok(self.send(output))
    }

    /// 長按 `layout` 目前頁第 `index` 個鍵、選了長按清單第 `choice` 個（0 是預設）。
    #[wasm_bindgen(js_name = longPress)]
    pub fn long_press(
        &mut self,
        layout: &mut WebLayout,
        index: usize,
        choice: usize,
    ) -> Result<WebSnapshot, JsError> {
        let output = layout
            .keyboard
            .long_press(index, choice)
            .map_err(js_error)?;
        Ok(self.send(output))
    }
}

impl WebSession {
    /// 改目前選項的一項再交給核心。
    fn set_options(&mut self, change: impl FnOnce(&mut Options)) {
        let mut options = self.session.options();
        change(&mut options);
        self.session.set_options(options);
    }

    fn handle(&mut self, key: Key) -> Snapshot {
        self.session.handle(&self.engine, &mut self.user, key)
    }

    /// 觸控鍵盤的輸入交給核心：`Output` 轉成 `Key`，文字經 `Session::text`。結果帶著輸入的種類與
    /// 字元，核心沒吃下時網頁照它做。
    fn send(&mut self, output: Output) -> WebSnapshot {
        let snapshot = match &output {
            Output::Text(text) => self.session.text(&self.engine, &mut self.user, text),
            Output::Letter(c) => self.handle(Key::Letter(*c)),
            Output::Punct(c) => self.handle(Key::Punct(*c)),
            Output::Space => self.handle(Key::Space),
            Output::Backspace => self.handle(Key::Backspace),
            Output::Enter => self.handle(Key::Enter),
            Output::ToggleAscii => self.handle(Key::ToggleAscii(ShiftKey::Right)),
            Output::Modifier => self.handle(Key::Modifier),
        };
        let mut snapshot = WebSnapshot::from(snapshot);
        snapshot.action = Some(output.kind().into());
        snapshot.character = output.character().map(String::from);
        snapshot
    }
}

/// `Snapshot` 給 JS 的樣子：攤平成網頁用得到的欄位，其他欄位等網頁用到時再加。JS 端叫 `Snapshot`。
#[wasm_bindgen(js_name = Snapshot, getter_with_clone)]
pub struct WebSnapshot {
    /// 為假時網頁不攔截這個鍵，交給瀏覽器。
    #[wasm_bindgen(readonly)]
    pub handled: bool,
    /// 組字的整串輸入碼，含已選定的分段的碼；「回報」用。
    #[wasm_bindgen(readonly)]
    pub input: String,
    /// 已選定的分段接起來的字詞；「回報」用。
    #[wasm_bindgen(readonly)]
    pub selected: String,
    /// 組字的各分段，依輸入碼的順序。
    #[wasm_bindgen(readonly)]
    pub segments: Vec<WebSegment>,
    /// 目前分段在 `segments` 的位置；等於 `segments.length` 時候選列整串（沒有目前分段）。
    #[wasm_bindgen(readonly)]
    pub focus: usize,
    /// 目前頁的候選，經過後處理（簡體模式是簡體）。
    #[wasm_bindgen(readonly)]
    pub candidates: Vec<String>,
    /// 目前頁的候選的繁體原文，和 `candidates` 一一對應；字根提示與「回報」用。
    #[wasm_bindgen(readonly, js_name = candidatesOriginal)]
    pub candidates_original: Vec<String>,
    /// 目前頁，從 0 起。
    #[wasm_bindgen(readonly)]
    pub page: usize,
    #[wasm_bindgen(readonly)]
    pub commit: Option<String>,
    /// 觸控鍵盤交給核心的輸入的種類（`qim_layout::Output::kind`），長按時是選到的那個；核心沒吃下時
    /// 網頁照它做宿主的預設動作。實體鍵盤的鍵是 `None`。
    #[wasm_bindgen(readonly)]
    pub action: Option<String>,
    /// `action` 是 `letter` 或 `punct` 時的字元；核心沒吃下時網頁照打它。
    #[wasm_bindgen(readonly)]
    pub character: Option<String>,
    /// `commit` 的繁體原文；「回報」用。
    #[wasm_bindgen(readonly, js_name = commitOriginal)]
    pub commit_original: Option<String>,
    /// 英文直通（單按 Shift 切換）；候選列旁顯示目前狀態。
    #[wasm_bindgen(readonly)]
    pub ascii: bool,
    /// 每次候選變動加一；和上次要求重排時不同，才再要求（plan §2.7）。
    #[wasm_bindgen(readonly)]
    pub generation: u64,
}

/// 給背景模型的重排要求（`qim_core::RerankRequest`）。JS 端叫 `RerankRequest`。
#[wasm_bindgen(js_name = RerankRequest, getter_with_clone)]
pub struct WebRerankRequest {
    #[wasm_bindgen(readonly)]
    pub generation: u64,
    /// 候選前面的字詞（繁體原文）。
    #[wasm_bindgen(readonly)]
    pub context: String,
    /// 前幾個候選的繁體原文，依目前的順序；和 `context` 一樣用原文，簡體模式下模型也看繁體。
    #[wasm_bindgen(readonly)]
    pub candidates: Vec<String>,
}

impl From<Snapshot> for WebSnapshot {
    fn from(s: Snapshot) -> Self {
        Self {
            handled: s.handled,
            input: s.composition.input.clone(),
            selected: s.composition.selected_text(),
            focus: s.composition.focus,
            segments: s
                .composition
                .segments
                .into_iter()
                .map(|segment| WebSegment {
                    code: segment.code,
                    radicals: segment.radicals,
                    text: segment.text,
                    selected: segment.selected,
                    raw: segment.raw,
                })
                .collect(),
            candidates_original: s.candidates.iter().map(|c| c.original.clone()).collect(),
            candidates: s.candidates.into_iter().map(|c| c.text).collect(),
            page: s.page,
            commit: s.commit,
            action: None,
            character: None,
            commit_original: s.commit_original,
            ascii: s.ascii,
            generation: s.generation,
        }
    }
}

/// 候選每頁的個數（`qim_core::PAGE_SIZE`）：網頁把目前頁的位置換算成 [`WebSession::select`] 的位置。
#[wasm_bindgen(js_name = pageSize)]
pub fn page_size() -> usize {
    qim_core::PAGE_SIZE
}

/// 組字的一段（`qim_core::Segment`）。JS 端叫 `Segment`。
#[wasm_bindgen(js_name = Segment, getter_with_clone)]
#[derive(Clone)]
pub struct WebSegment {
    #[wasm_bindgen(readonly)]
    pub code: String,
    /// `code` 的字根；組字裡沒選定的分段顯示它（brief §4.3）。
    #[wasm_bindgen(readonly)]
    pub radicals: String,
    /// 沒有路徑蓋得滿剩下的輸入碼時是空的。
    #[wasm_bindgen(readonly)]
    pub text: String,
    #[wasm_bindgen(readonly)]
    pub selected: bool,
    /// 臨時英文（組字中單按留碼的 Shift）：`text` 是原樣的英文，顯示它而不是字根。
    #[wasm_bindgen(readonly)]
    pub raw: bool,
}

/// 字根提示：方案指名的字根提示資料包，查候選每個字的倉頡碼與字根（plan §2.5）。JS 端叫 `RadicalHints`。
#[wasm_bindgen(js_name = RadicalHints)]
pub struct WebRadicalHints {
    hints: RadicalHints,
    /// 建立時的方案，換字根用。
    scheme: Scheme,
}

#[wasm_bindgen(js_class = RadicalHints)]
impl WebRadicalHints {
    /// 由字根提示資料包的位元組建立；要是 `session` 的方案指名的那份。
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: &[u8], session: &WebSession) -> Result<WebRadicalHints, JsError> {
        let scheme = session.engine.scheme().clone();
        let pack = RadicalHintPack::from_bytes(bytes).map_err(js_error)?;
        let hints = RadicalHints::for_scheme(pack, &scheme).map_err(js_error)?;
        Ok(Self { hints, scheme })
    }

    /// 候選 `text` 每個字的提示；`version` 是 `cangjie5` 或 `cangjie3`。
    pub fn hint(&self, text: &str, version: &str) -> Result<Vec<WebRadicalHint>, JsError> {
        let version: CangjieVersion = version.parse().map_err(js_error)?;
        Ok(self
            .hints
            .hint(&self.scheme, text, version)
            .into_iter()
            .map(|hint| WebRadicalHint {
                text: hint.text.to_string(),
                codes: hint.codes,
                radicals: hint.radicals,
            })
            .collect())
    }
}

/// 詞表匯入的結果。JS 端叫 `WordListReport`。
#[wasm_bindgen(js_name = WordListReport, getter_with_clone)]
pub struct WebWordListReport {
    /// 認出的格式，給人看的名字。
    #[wasm_bindgen(readonly)]
    pub format: String,
    /// 加進使用者詞的詞數。
    #[wasm_bindgen(readonly)]
    pub added: usize,
    /// 沒加的列，一列一句（「第 n 行「詞」：原因」）。
    #[wasm_bindgen(readonly)]
    pub skipped: Vec<String>,
    /// 加了、但快捷鍵沒當觸發詞的列，一列一句。
    #[wasm_bindgen(readonly, js_name = withoutTrigger)]
    pub without_trigger: Vec<String>,
}

impl From<WordListReport> for WebWordListReport {
    fn from(report: WordListReport) -> Self {
        Self {
            format: report.format.to_string(),
            added: report.added,
            skipped: report.skipped.iter().map(ToString::to_string).collect(),
            without_trigger: report
                .without_trigger
                .iter()
                .map(ToString::to_string)
                .collect(),
        }
    }
}

/// 一條常用語。JS 端叫 `Phrase`。
#[wasm_bindgen(js_name = Phrase, getter_with_clone)]
pub struct WebPhrase {
    #[wasm_bindgen(readonly)]
    pub text: String,
    /// 觸發詞；沒有時是空字串。
    #[wasm_bindgen(readonly)]
    pub trigger: String,
}

/// 候選裡一個字的字根提示。JS 端叫 `RadicalHint`。
#[wasm_bindgen(js_name = RadicalHint, getter_with_clone)]
pub struct WebRadicalHint {
    #[wasm_bindgen(readonly)]
    pub text: String,
    /// 倉頡碼，沒有時是空的。
    #[wasm_bindgen(readonly)]
    pub codes: Vec<String>,
    /// 每個碼的字根，和 `codes` 一一對應。
    #[wasm_bindgen(readonly)]
    pub radicals: Vec<String>,
}

/// 觸控鍵盤（[`Keyboard`]）：網頁 fetch 的佈局檔讀進 `qim-layout`，網頁只照 [`WebLayout::keys`] 畫、
/// 把點擊位置交給 [`WebLayout::hit`]。JS 端叫 `Layout`。
#[wasm_bindgen(js_name = Layout)]
pub struct WebLayout {
    keyboard: Keyboard,
    /// 驗證與選版本的警告，給網頁印在 console。
    warnings: Vec<String>,
}

/// 網頁傳來的裝置（`phone`、`tablet`）與方向（`portrait`、`landscape`）。
fn when(device: &str, orientation: &str) -> Result<When, JsError> {
    Ok(When {
        device: device.parse().map_err(js_error)?,
        orientation: orientation.parse().map_err(js_error)?,
    })
}

#[wasm_bindgen(js_class = Layout)]
impl WebLayout {
    /// 讀佈局檔並依裝置與方向選版本；有錯誤（plan §2.6）時失敗。
    #[wasm_bindgen(constructor)]
    pub fn new(json: &str, device: &str, orientation: &str) -> Result<WebLayout, JsError> {
        let when = when(device, orientation)?;
        let (layout, warnings) = qim_layout::validate(json, Target::default())
            .into_layout()
            .map_err(js_error)?;
        let (keyboard, warning) = Keyboard::new(layout, when);
        Ok(Self {
            keyboard,
            warnings: warnings
                .iter()
                .chain(&warning)
                .map(ToString::to_string)
                .collect(),
        })
    }

    /// 依裝置與方向重選佈局版本（轉向時叫），見 [`Keyboard::select`]；沒有相符的版本時回傳警告。
    pub fn select(&mut self, device: &str, orientation: &str) -> Result<Option<String>, JsError> {
        let warning = self.keyboard.select(when(device, orientation)?);
        Ok(warning.map(|w| w.to_string()))
    }

    /// 換成同一個鍵盤的另一份佈局檔（數字列開關）時接手 `old` 的 Shift 與目前頁
    /// （[`Keyboard::keep_state_of`]）。
    #[wasm_bindgen(js_name = keepStateOf)]
    pub fn keep_state_of(&mut self, old: &WebLayout) {
        self.keyboard.keep_state_of(&old.keyboard);
    }

    /// 觸控 Shift 的狀態：`off`、`once`（下一個字母大寫）、`locked`（鎖定大寫）；網頁依它畫 Shift 鍵、
    /// 狀態變了就重畫標籤。
    #[wasm_bindgen(getter)]
    pub fn shift(&self) -> String {
        match self.keyboard.shift() {
            qim_layout::Shift::Off => "off",
            qim_layout::Shift::Once => "once",
            qim_layout::Shift::Locked => "locked",
        }
        .into()
    }

    /// 目前頁的 id；切頁鍵按下後網頁比對它決定要不要重畫。
    #[wasm_bindgen(getter, js_name = pageId)]
    pub fn page_id(&self) -> String {
        self.keyboard.page().id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }

    /// 設計尺寸的寬，按鍵座標以它為準。
    #[wasm_bindgen(getter, js_name = canvasWidth)]
    pub fn canvas_width(&self) -> f64 {
        self.keyboard.variant().canvas.width
    }

    #[wasm_bindgen(getter, js_name = canvasHeight)]
    pub fn canvas_height(&self) -> f64 {
        self.keyboard.variant().canvas.height
    }

    /// 目前頁的按鍵：標籤省略時用 `session` 方案的字根，組字中照 `composing`，Shift 開著時字母鍵
    /// 顯示大寫字母。
    pub fn keys(&self, session: &WebSession) -> Vec<WebLayoutKey> {
        let radicals = &session.engine.scheme().radicals;
        let state = self.keyboard.key_state(session.session.composing());
        self.keyboard
            .page()
            .keys
            .iter()
            .map(|key| {
                let label = key.display_label(state, |c| radicals.get(&c).map(String::as_str));
                let style = key.style.clone().unwrap_or_default();
                let [label_offset_x, label_offset_y] = style.label_offset.unwrap_or([0.0, 0.0]);
                WebLayoutKey {
                    id: key.id.clone(),
                    x: key.frame.x,
                    y: key.frame.y,
                    w: key.frame.w,
                    h: key.frame.h,
                    rotate: key.rotate,
                    z: key.z,
                    radius: key.shape.corner_radius(),
                    a11y_label: key.accessibility_label(state, &label),
                    label,
                    hint: key.display_hint(state).map(str::to_owned),
                    is_shift: key.tap_action(state) == Action::Shift,
                    fill_light: style.fill.as_ref().map(|c| c.light.clone()),
                    fill_dark: style.fill.map(|c| c.dark),
                    label_color_light: style.label_color.as_ref().map(|c| c.light.clone()),
                    label_color_dark: style.label_color.map(|c| c.dark),
                    opacity: style.opacity,
                    label_size: key.display_label_size(state),
                    label_offset_x,
                    label_offset_y,
                    preview: key.shows_preview(state),
                    long_press: key.long_press.iter().map(Action::spoken).collect(),
                }
            })
            .collect()
    }

    /// 點在畫布座標 `(x, y)` 時按到的鍵在 [`WebLayout::keys`] 的位置。
    pub fn hit(&self, x: f64, y: f64) -> Option<usize> {
        self.keyboard.page().hit(x, y)
    }
}

/// 佈局的一個鍵給 JS 畫的樣子，座標是設計尺寸；顏色淺色與深色都給，由網頁依目前的配色選。
/// JS 端叫 `LayoutKey`。
#[wasm_bindgen(js_name = LayoutKey, getter_with_clone)]
pub struct WebLayoutKey {
    #[wasm_bindgen(readonly)]
    pub id: String,
    #[wasm_bindgen(readonly)]
    pub x: f64,
    #[wasm_bindgen(readonly)]
    pub y: f64,
    #[wasm_bindgen(readonly)]
    pub w: f64,
    #[wasm_bindgen(readonly)]
    pub h: f64,
    /// 以中心順時針旋轉的角度（度）。
    #[wasm_bindgen(readonly)]
    pub rotate: f64,
    #[wasm_bindgen(readonly)]
    pub z: i32,
    /// 圓角半徑，設計尺寸。
    #[wasm_bindgen(readonly)]
    pub radius: f64,
    /// 主標籤，已照 [`qim_layout::Key::display_label`] 補上字根。
    #[wasm_bindgen(readonly)]
    pub label: String,
    /// 角標（[`qim_layout::Key::display_hint`]），Shift 開著時字母鍵沒有。
    #[wasm_bindgen(readonly)]
    pub hint: Option<String>,
    /// 現在點下去是不是 Shift（組字中 ⇧ 是分隔鍵，不是）；網頁依 [`WebLayout::shift`] 畫它的狀態。
    #[wasm_bindgen(readonly, js_name = isShift)]
    pub is_shift: bool,
    #[wasm_bindgen(readonly, js_name = a11yLabel)]
    pub a11y_label: String,
    #[wasm_bindgen(readonly, js_name = fillLight)]
    pub fill_light: Option<String>,
    #[wasm_bindgen(readonly, js_name = fillDark)]
    pub fill_dark: Option<String>,
    #[wasm_bindgen(readonly, js_name = labelColorLight)]
    pub label_color_light: Option<String>,
    #[wasm_bindgen(readonly, js_name = labelColorDark)]
    pub label_color_dark: Option<String>,
    #[wasm_bindgen(readonly)]
    pub opacity: Option<f64>,
    /// 主標籤字級，設計尺寸。
    #[wasm_bindgen(readonly, js_name = labelSize)]
    pub label_size: Option<f64>,
    #[wasm_bindgen(readonly, js_name = labelOffsetX)]
    pub label_offset_x: f64,
    #[wasm_bindgen(readonly, js_name = labelOffsetY)]
    pub label_offset_y: f64,
    /// 按住時要不要顯示放大預覽（[`qim_layout::Key::shows_preview`]），組字中照 `composing`。
    #[wasm_bindgen(readonly)]
    pub preview: bool,
    /// 長按清單各選項的標籤（[`Action::spoken`]），依序排、第一個是預設；空的就沒有長按。
    #[wasm_bindgen(readonly, js_name = longPress)]
    pub long_press: Vec<String>,
}
