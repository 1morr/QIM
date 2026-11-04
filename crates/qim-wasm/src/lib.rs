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

    /// 按下 `layout` 第 `index` 個鍵（[`WebLayout::hit`] 的結果）的動作；組字中照 `composing`。
    /// 切頁、文字、切換鍵盤、中英切換在 M1 票 20 接上，現在當 `Key::Other`：組字中吃下，
    /// 沒組字時交還網頁（文字鍵由網頁插入）。
    pub fn tap(&mut self, layout: &WebLayout, index: usize) -> Result<WebSnapshot, JsError> {
        let key = layout
            .page()
            .keys
            .get(index)
            .ok_or_else(|| JsError::new(&format!("佈局沒有第 {index} 個鍵")))?;
        Ok(self.handle(match key.tap_action(self.composing) {
            Action::Letter(c) => Key::Letter(*c),
            Action::Space => Key::Space,
            Action::Backspace => Key::Backspace,
            Action::Enter => Key::Enter,
            Action::Text(_)
            | Action::SwitchPage(_)
            | Action::NextKeyboard
            | Action::ToggleAscii => Key::Other,
        }))
    }

    fn handle(&mut self, key: Key) -> WebSnapshot {
        let snapshot: WebSnapshot = self
            .session
            .handle(&self.engine, &mut self.user, key)
            .into();
        self.composing = !snapshot.selected.is_empty() || !snapshot.unselected.is_empty();
        snapshot
    }
}

/// `Snapshot` 給 JS 的樣子：攤平成網頁用得到的欄位，其他欄位等網頁用到時再加。JS 端叫 `Snapshot`。
#[wasm_bindgen(js_name = Snapshot, getter_with_clone)]
pub struct WebSnapshot {
    /// 為假時網頁不攔截這個鍵，交給瀏覽器。
    #[wasm_bindgen(readonly)]
    pub handled: bool,
    /// 已選定的分段接起來的字詞，組字顯示在剩下的輸入碼前面。
    #[wasm_bindgen(readonly)]
    pub selected: String,
    /// 還沒選定的輸入碼。
    #[wasm_bindgen(readonly)]
    pub unselected: String,
    /// 還沒選定的輸入碼的字根，組字顯示它而不是輸入碼（brief §4.3）。
    #[wasm_bindgen(readonly)]
    pub radicals: String,
    /// 目前頁的候選。
    #[wasm_bindgen(readonly)]
    pub candidates: Vec<String>,
    #[wasm_bindgen(readonly)]
    pub commit: Option<String>,
}

impl From<Snapshot> for WebSnapshot {
    fn from(s: Snapshot) -> Self {
        Self {
            handled: s.handled,
            selected: s.composition.selected_text(),
            unselected: s.composition.unselected_input().to_string(),
            radicals: s.composition.radicals,
            candidates: s.candidates.into_iter().map(|c| c.text).collect(),
            commit: s.commit,
        }
    }
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
        let wanted = When {
            device: device.parse().map_err(|e: String| JsError::new(&e))?,
            orientation: orientation.parse().map_err(|e: String| JsError::new(&e))?,
        };
        let (layout, warnings) = qim_layout::validate(json, Target::default())
            .into_layout()
            .map_err(|e| JsError::new(&e.to_string()))?;
        let (variant, selection) = layout.select(wanted);
        let page = layout.variants[variant]
            .initial_page_index()
            .expect("validate 保證 initialPage 存在、每個版本至少一頁");
        let warnings = warnings
            .iter()
            .chain(&selection)
            .map(ToString::to_string)
            .collect();
        Ok(Self {
            layout,
            warnings,
            variant,
            page,
        })
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
                let tap = key.tap_action(session.composing);
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
                    fill: style.fill.map(|c| c.light),
                    label_color: style.label_color.map(|c| c.light),
                    opacity: style.opacity,
                    label_size: style.label_size,
                    label_offset_x,
                    label_offset_y,
                    tap: tap.name().into(),
                    letter: match tap {
                        Action::Letter(c) => Some(c.to_string()),
                        _ => None,
                    },
                    text: match tap {
                        Action::Text(text) => Some(text.clone()),
                        _ => None,
                    },
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
    fn variant(&self) -> &qim_layout::LayoutVariant {
        &self.layout.variants[self.variant]
    }

    fn page(&self) -> &qim_layout::Page {
        &self.variant().pages[self.page]
    }
}

/// 佈局的一個鍵給 JS 畫的樣子，座標是設計尺寸，顏色是淺色（深色在 M1 票 20）。JS 端叫 `LayoutKey`。
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
    #[wasm_bindgen(readonly)]
    pub fill: Option<String>,
    #[wasm_bindgen(readonly, js_name = labelColor)]
    pub label_color: Option<String>,
    #[wasm_bindgen(readonly)]
    pub opacity: Option<f64>,
    /// 主標籤字級，設計尺寸。
    #[wasm_bindgen(readonly, js_name = labelSize)]
    pub label_size: Option<f64>,
    #[wasm_bindgen(readonly, js_name = labelOffsetX)]
    pub label_offset_x: f64,
    #[wasm_bindgen(readonly, js_name = labelOffsetY)]
    pub label_offset_y: f64,
    /// 點一下的動作種類（`Action::name`），組字中照 `composing`；核心沒吃下時網頁照它做宿主的預設動作。
    #[wasm_bindgen(readonly)]
    pub tap: String,
    /// `tap` 是 `letter` 時的字母。
    #[wasm_bindgen(readonly)]
    pub letter: Option<String>,
    /// `tap` 是 `text` 時的文字。
    #[wasm_bindgen(readonly)]
    pub text: Option<String>,
}
