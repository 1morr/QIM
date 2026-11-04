//! 網頁用的 WebAssembly 綁定：只把 JS 的呼叫轉給 `qim-core` 與 `qim-layout`，不放邏輯（plan §1.3）。

use qim_core::{Engine, Key, Session, Snapshot, UserModel};
use qim_layout::Action;
use wasm_bindgen::prelude::*;

/// 網頁的一個輸入框：速成、詞庫與語言模型資料包、一個個人模型、一個 `Session`。JS 端叫 `Session`。
#[wasm_bindgen(js_name = Session)]
pub struct WebSession {
    engine: Engine,
    user: UserModel,
    session: Session,
}

#[wasm_bindgen(js_class = Session)]
impl WebSession {
    /// 由速成資料包與方案指名的詞庫、語言模型資料包的位元組建立；資料包由網頁 fetch。
    #[wasm_bindgen(constructor)]
    pub fn new(
        pack: &[u8],
        lexicon: Option<Vec<u8>>,
        language_model: Option<Vec<u8>>,
    ) -> Result<WebSession, JsError> {
        let engine = Engine::from_pack_bytes(pack, lexicon.as_deref(), language_model.as_deref())
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self {
            engine,
            user: UserModel::default(),
            session: Session::default(),
        })
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

    /// 按下 `layout` 第 `index` 個鍵（[`WebLayout::hit`] 的結果）的 `tap` 動作。
    pub fn tap(&mut self, layout: &WebLayout, index: usize) -> Result<WebSnapshot, JsError> {
        let key = layout
            .page()
            .keys
            .get(index)
            .ok_or_else(|| JsError::new(&format!("佈局沒有第 {index} 個鍵")))?;
        Ok(self.handle(match key.tap {
            Action::Letter(c) => Key::Letter(c),
            Action::Space => Key::Space,
            Action::Backspace => Key::Backspace,
            Action::Enter => Key::Enter,
        }))
    }

    fn handle(&mut self, key: Key) -> WebSnapshot {
        self.session
            .handle(&self.engine, &mut self.user, key)
            .into()
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
    /// 已選定的分段接起來的字詞，組字顯示在剩下的輸入碼前面。
    #[wasm_bindgen(readonly)]
    pub selected: String,
    /// 還沒選定的輸入碼。
    #[wasm_bindgen(readonly)]
    pub unselected: String,
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
            input: s.composition.input.clone(),
            selected: s.composition.selected_text(),
            unselected: s.composition.unselected_input().to_string(),
            candidates: s.candidates.into_iter().map(|c| c.text).collect(),
            commit: s.commit,
        }
    }
}

/// 觸控佈局：網頁 fetch 的佈局檔讀進 `qim-layout`，網頁只照 [`WebLayout::keys`] 畫、
/// 把點擊位置交給 [`WebLayout::hit`]。JS 端叫 `Layout`。
#[wasm_bindgen(js_name = Layout)]
pub struct WebLayout {
    layout: qim_layout::Layout,
}

#[wasm_bindgen(js_class = Layout)]
impl WebLayout {
    #[wasm_bindgen(constructor)]
    pub fn new(json: &str) -> Result<WebLayout, JsError> {
        let layout =
            qim_layout::Layout::from_json(json).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self { layout })
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

    /// 目前頁的按鍵，標籤省略時用 `session` 方案的字根。
    pub fn keys(&self, session: &WebSession) -> Vec<WebLayoutKey> {
        let radicals = &session.engine.scheme().radicals;
        self.page()
            .keys
            .iter()
            .map(|key| WebLayoutKey {
                id: key.id.clone(),
                x: key.frame.x,
                y: key.frame.y,
                w: key.frame.w,
                h: key.frame.h,
                z: key.z,
                label: key.display_label(|c| radicals.get(&c).map(String::as_str)),
                hint: key.hint.clone(),
                tap: key.tap.name().into(),
                letter: match key.tap {
                    Action::Letter(c) => Some(c.to_string()),
                    _ => None,
                },
            })
            .collect()
    }

    /// 點在畫布座標 `(x, y)` 時按到的鍵在 [`WebLayout::keys`] 的位置。
    pub fn hit(&self, x: f64, y: f64) -> Option<usize> {
        self.page().hit(x, y)
    }
}

impl WebLayout {
    /// 佈局版本的選法在 M1 票 19；現在只有一個版本、一頁（`Layout::from_json` 保證至少一個）。
    fn variant(&self) -> &qim_layout::LayoutVariant {
        &self.layout.variants[0]
    }

    fn page(&self) -> &qim_layout::Page {
        &self.variant().pages[0]
    }
}

/// 佈局的一個鍵給 JS 畫的樣子，座標是設計尺寸。JS 端叫 `LayoutKey`。
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
    #[wasm_bindgen(readonly)]
    pub z: i32,
    /// 主標籤，已照 [`qim_layout::Key::display_label`] 補上字根。
    #[wasm_bindgen(readonly)]
    pub label: String,
    #[wasm_bindgen(readonly)]
    pub hint: Option<String>,
    /// `tap` 動作的種類：`letter`、`space`、`backspace`、`enter`；核心沒吃下時網頁照它做宿主的預設動作。
    #[wasm_bindgen(readonly)]
    pub tap: String,
    /// `tap` 是 `letter` 時的字母。
    #[wasm_bindgen(readonly)]
    pub letter: Option<String>,
}
