//! 網頁用的 WebAssembly 綁定：只把 JS 的呼叫轉給 `qim-core` 與 `qim-layout`，不放邏輯（plan §1.3）。

use qim_core::{
    CangjieVersion, Engine, Key, Options, RadicalHintPack, RadicalHints, Scheme, Session,
    SimplifierPack, Snapshot, UserModel,
};
use qim_layout::{Action, Target, When};
use wasm_bindgen::prelude::*;

/// 網頁的一個輸入框：速成、詞庫、語言模型與簡繁資料包、一個個人模型、一個 `Session`。JS 端叫 `Session`。
#[wasm_bindgen(js_name = Session)]
pub struct WebSession {
    engine: Engine,
    user: UserModel,
    session: Session,
    options: Options,
    /// 上一個鍵之後還在組字：觸控鍵盤依它選 `composing` 的標籤與動作。
    composing: bool,
}

#[wasm_bindgen(js_class = Session)]
impl WebSession {
    /// 由速成資料包與方案指名的詞庫、語言模型資料包，加上簡繁資料包（沒有時簡體模式照原樣
    /// 輸出）的位元組建立；資料包由網頁 fetch。
    #[wasm_bindgen(constructor)]
    pub fn new(
        pack: &[u8],
        lexicon: Option<Vec<u8>>,
        language_model: Option<Vec<u8>>,
        simplifier: Option<Vec<u8>>,
    ) -> Result<WebSession, JsError> {
        let error = |e: qim_core::PackError| JsError::new(&e.to_string());
        let mut engine =
            Engine::from_pack_bytes(pack, lexicon.as_deref(), language_model.as_deref())
                .map_err(error)?;
        if let Some(bytes) = simplifier {
            engine = engine.with_simplifier(SimplifierPack::from_bytes(&bytes).map_err(error)?);
        }
        Ok(Self {
            engine,
            user: UserModel::default(),
            session: Session::default(),
            options: Options::default(),
            composing: false,
        })
    }

    /// 無痕模式：開著時不學習（plan §2.4）。
    #[wasm_bindgen(js_name = setIncognito)]
    pub fn set_incognito(&mut self, on: bool) {
        self.options.incognito = on;
        self.session.set_options(self.options);
    }

    /// 手動加一個使用者詞；`trigger` 是空字串時不設觸發詞（brief §4.5）。
    #[wasm_bindgen(js_name = addWord)]
    pub fn add_word(&mut self, text: &str, trigger: &str) -> Result<(), JsError> {
        let trigger = (!trigger.is_empty()).then_some(trigger);
        self.user
            .add_word(&self.engine, text, trigger)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// 使用者詞與學習計數匯出成 `# qim-userdict v1` 的 TSV（plan §2.4），網頁存 `localStorage`、
    /// 下載成檔。
    #[wasm_bindgen(js_name = exportUserDict)]
    pub fn export_user_dict(&self) -> String {
        self.user.export()
    }

    /// 檢查 `text` 是不是匯入得了的檔，不動目前的資料；網頁請使用者確認覆蓋之前先檢查。
    #[wasm_bindgen(js_name = checkUserDict)]
    pub fn check_user_dict(&self, text: &str) -> Result<(), JsError> {
        UserModel::import(text)
            .map(|_| ())
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// 讀匯出的檔，取代目前的使用者詞與學習計數；不合格式時整份不收、原本的不動。
    #[wasm_bindgen(js_name = importUserDict)]
    pub fn import_user_dict(&mut self, text: &str) -> Result<(), JsError> {
        self.user = UserModel::import(text).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(())
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
        self.options.simplified = on;
        self.session.set_options(self.options);
        self.session.refresh(&self.engine, &self.user).into()
    }

    pub fn letter(&mut self, c: char) -> WebSnapshot {
        self.handle(Key::Letter(c))
    }

    /// 數字、字母、空白以外的可見 ASCII 字元。
    pub fn punct(&mut self, c: char) -> WebSnapshot {
        self.handle(Key::Punct(c))
    }

    pub fn enter(&mut self) -> WebSnapshot {
        self.handle(Key::Enter)
    }

    pub fn left(&mut self) -> WebSnapshot {
        self.handle(Key::Left)
    }

    pub fn right(&mut self) -> WebSnapshot {
        self.handle(Key::Right)
    }

    /// 網頁不轉成其他鍵的按鍵：組字中吃下，沒有組字時交還瀏覽器。
    pub fn other(&mut self) -> WebSnapshot {
        self.handle(Key::Other)
    }

    pub fn space(&mut self) -> WebSnapshot {
        self.handle(Key::Space)
    }

    pub fn backspace(&mut self) -> WebSnapshot {
        self.handle(Key::Backspace)
    }

    pub fn escape(&mut self) -> WebSnapshot {
        self.handle(Key::Escape)
    }

    pub fn digit(&mut self, digit: u8) -> WebSnapshot {
        self.handle(Key::Digit(digit))
    }

    #[wasm_bindgen(js_name = pageUp)]
    pub fn page_up(&mut self) -> WebSnapshot {
        self.handle(Key::PageUp)
    }

    #[wasm_bindgen(js_name = pageDown)]
    pub fn page_down(&mut self) -> WebSnapshot {
        self.handle(Key::PageDown)
    }

    /// 選全部候選的第 `index` 個（目前頁第 i 個是 `page × 9 + i`），例如點候選列。
    pub fn select(&mut self, index: usize) -> WebSnapshot {
        let snapshot = self.session.select(&self.engine, &mut self.user, index);
        self.track_composing(snapshot)
    }

    /// 移到組字的第 `segment` 段，例如點組字裡的那一段；分段數是回到整串。
    #[wasm_bindgen(js_name = focusSegment)]
    pub fn focus_segment(&mut self, segment: usize) -> WebSnapshot {
        let snapshot = self
            .session
            .focus_segment(&self.engine, &mut self.user, segment);
        self.track_composing(snapshot)
    }

    /// 點一下 `layout` 第 `index` 個鍵（[`WebLayout::hit`] 的結果）：做它的 `tap`，組字中照 `composing`。
    pub fn tap(&mut self, layout: &mut WebLayout, index: usize) -> Result<WebSnapshot, JsError> {
        let action = layout.key(index)?.tap_action(self.composing).clone();
        Ok(self.perform(layout, action))
    }

    /// 長按 `layout` 第 `index` 個鍵、選了長按清單第 `choice` 個（0 是預設）。
    #[wasm_bindgen(js_name = longPress)]
    pub fn long_press(
        &mut self,
        layout: &mut WebLayout,
        index: usize,
        choice: usize,
    ) -> Result<WebSnapshot, JsError> {
        let key = layout.key(index)?;
        let action =
            key.long_press.get(choice).cloned().ok_or_else(|| {
                JsError::new(&format!("鍵 {} 的長按清單沒有第 {choice} 個", key.id))
            })?;
        Ok(self.perform(layout, action))
    }

    /// 做一個佈局動作：文字經 `Session::text`（組字中先上屏組字）；切頁換 `layout` 的目前頁，
    /// 核心收到 `Key::Other`（組字中吃下、組字留著）；網頁沒有下一個鍵盤，也當 `Key::Other`；
    /// 中英切換在票 23 接，現在也當 `Key::Other`。結果帶著做了的動作，核心沒吃下時網頁照它做。
    fn perform(&mut self, layout: &mut WebLayout, action: Action) -> WebSnapshot {
        let snapshot = match &action {
            Action::Text(text) => self.session.text(&self.engine, &mut self.user, text),
            other => {
                let key = match other {
                    Action::Letter(c) => Key::Letter(*c),
                    Action::Space => Key::Space,
                    Action::Backspace => Key::Backspace,
                    Action::Enter => Key::Enter,
                    Action::SwitchPage(page) => {
                        layout.switch_page(page);
                        Key::Other
                    }
                    Action::NextKeyboard | Action::ToggleAscii | Action::Text(_) => Key::Other,
                };
                self.session.handle(&self.engine, &mut self.user, key)
            }
        };
        let mut snapshot = self.track_composing(snapshot);
        snapshot.action = Some(action.name().into());
        snapshot.letter = match action {
            Action::Letter(c) => Some(c.to_string()),
            _ => None,
        };
        snapshot
    }

    fn handle(&mut self, key: Key) -> WebSnapshot {
        let snapshot = self.session.handle(&self.engine, &mut self.user, key);
        self.track_composing(snapshot)
    }

    /// 記下事件之後還在不在組字（[`WebSession::composing`]），再轉成給 JS 的樣子。
    fn track_composing(&mut self, snapshot: Snapshot) -> WebSnapshot {
        self.composing = !snapshot.composition.input.is_empty();
        snapshot.into()
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
    /// 觸控鍵盤做了的佈局動作（`Action::name`），長按時是選到的那個；核心沒吃下時網頁照它做宿主的預設動作。
    /// 實體鍵盤的鍵是 `None`。
    #[wasm_bindgen(readonly)]
    pub action: Option<String>,
    /// `action` 是 `letter` 時的字母。
    #[wasm_bindgen(readonly)]
    pub letter: Option<String>,
    /// `commit` 的繁體原文；「回報」用。
    #[wasm_bindgen(readonly, js_name = commitOriginal)]
    pub commit_original: Option<String>,
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
                })
                .collect(),
            candidates_original: s.candidates.iter().map(|c| c.original.clone()).collect(),
            candidates: s.candidates.into_iter().map(|c| c.text).collect(),
            page: s.page,
            commit: s.commit,
            action: None,
            letter: None,
            commit_original: s.commit_original,
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
        let error = |e: qim_core::PackError| JsError::new(&e.to_string());
        let scheme = session.engine.scheme().clone();
        let pack = RadicalHintPack::from_bytes(bytes).map_err(error)?;
        let hints = RadicalHints::for_scheme(pack, &scheme).map_err(error)?;
        Ok(Self { hints, scheme })
    }

    /// 候選 `text` 每個字的提示；`version` 是 `cangjie5` 或 `cangjie3`。
    pub fn hint(&self, text: &str, version: &str) -> Result<Vec<WebRadicalHint>, JsError> {
        let version: CangjieVersion = version.parse().map_err(|e: String| JsError::new(&e))?;
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

/// 觸控佈局：網頁 fetch 的佈局檔讀進 `qim-layout`，網頁只照 [`WebLayout::keys`] 畫、
/// 把點擊位置交給 [`WebLayout::hit`]。JS 端叫 `Layout`。
#[wasm_bindgen(js_name = Layout)]
pub struct WebLayout {
    layout: qim_layout::Layout,
    /// 驗證與選版本的警告，給網頁印在 console。
    warnings: Vec<String>,
    /// 選到的佈局版本與目前頁在陣列的位置。
    variant: usize,
    page: usize,
}

#[wasm_bindgen(js_class = Layout)]
impl WebLayout {
    /// 讀佈局檔並依裝置（`phone`、`tablet`）與方向（`portrait`、`landscape`）選版本；
    /// 有錯誤（plan §2.6）時失敗。
    #[wasm_bindgen(constructor)]
    pub fn new(json: &str, device: &str, orientation: &str) -> Result<WebLayout, JsError> {
        let (layout, warnings) = qim_layout::validate(json, Target::default())
            .into_layout()
            .map_err(|e| JsError::new(&e.to_string()))?;
        let mut web = Self {
            layout,
            warnings: warnings.iter().map(ToString::to_string).collect(),
            variant: 0,
            page: 0,
        };
        if let Some(warning) = web.select(device, orientation)? {
            web.warnings.push(warning);
        }
        Ok(web)
    }

    /// 依裝置與方向重選佈局版本（轉向時叫）；新版本有同 id 的頁就留在那頁，否則回到一開始的頁。
    /// 沒有相符的版本時回傳警告（用第一個版本等比例縮放）。
    pub fn select(&mut self, device: &str, orientation: &str) -> Result<Option<String>, JsError> {
        let wanted = When {
            device: device.parse().map_err(|e: String| JsError::new(&e))?,
            orientation: orientation.parse().map_err(|e: String| JsError::new(&e))?,
        };
        let current = self
            .layout
            .variants
            .get(self.variant)
            .and_then(|variant| variant.pages.get(self.page))
            .map(|page| page.id.clone());
        let (variant, warning) = self.layout.select(wanted);
        self.variant = variant;
        self.page = self
            .variant()
            .page_index_or_initial(current.as_deref())
            .expect("validate 保證 initialPage 存在、每個版本至少一頁");
        Ok(warning.map(|w| w.to_string()))
    }

    /// 目前頁的 id；切頁鍵按下後網頁比對它決定要不要重畫。
    #[wasm_bindgen(getter, js_name = pageId)]
    pub fn page_id(&self) -> String {
        self.page().id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }

    /// 設計尺寸的寬，按鍵座標以它為準。
    #[wasm_bindgen(getter, js_name = canvasWidth)]
    pub fn canvas_width(&self) -> f64 {
        self.variant().canvas.width
    }

    #[wasm_bindgen(getter, js_name = canvasHeight)]
    pub fn canvas_height(&self) -> f64 {
        self.variant().canvas.height
    }

    /// 目前頁的按鍵：標籤省略時用 `session` 方案的字根，組字中照 `composing`。
    pub fn keys(&self, session: &WebSession) -> Vec<WebLayoutKey> {
        let radicals = &session.engine.scheme().radicals;
        self.page()
            .keys
            .iter()
            .map(|key| {
                let label =
                    key.display_label(session.composing, |c| radicals.get(&c).map(String::as_str));
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
                    a11y_label: key.accessibility_label(&label),
                    label,
                    hint: key.hint.clone(),
                    fill_light: style.fill.as_ref().map(|c| c.light.clone()),
                    fill_dark: style.fill.map(|c| c.dark),
                    label_color_light: style.label_color.as_ref().map(|c| c.light.clone()),
                    label_color_dark: style.label_color.map(|c| c.dark),
                    opacity: style.opacity,
                    label_size: style.label_size,
                    label_offset_x,
                    label_offset_y,
                    preview: key.shows_preview(session.composing),
                    long_press: key.long_press.iter().map(Action::spoken).collect(),
                }
            })
            .collect()
    }

    /// 點在畫布座標 `(x, y)` 時按到的鍵在 [`WebLayout::keys`] 的位置。
    pub fn hit(&self, x: f64, y: f64) -> Option<usize> {
        self.page().hit(x, y)
    }
}

impl WebLayout {
    fn key(&self, index: usize) -> Result<&qim_layout::Key, JsError> {
        self.page()
            .keys
            .get(index)
            .ok_or_else(|| JsError::new(&format!("佈局沒有第 {index} 個鍵")))
    }

    /// 切到目前版本的 `id` 頁；`validate` 保證 `switchPage` 指到的頁存在。
    fn switch_page(&mut self, id: &str) {
        if let Some(page) = self.variant().page_index(id) {
            self.page = page;
        }
    }

    fn variant(&self) -> &qim_layout::LayoutVariant {
        &self.layout.variants[self.variant]
    }

    fn page(&self) -> &qim_layout::Page {
        &self.variant().pages[self.page]
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
    #[wasm_bindgen(readonly)]
    pub hint: Option<String>,
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
