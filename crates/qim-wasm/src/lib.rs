//! 網頁用的 WebAssembly 綁定：只把 JS 的呼叫轉給 `qim-core`，不放邏輯（plan §1.3）。

use qim_core::{Engine, Key, Options, Session, Snapshot, UserModel};
use wasm_bindgen::prelude::*;

/// 網頁的一個輸入框：速成與詞庫資料包、一個個人模型、一個 `Session`。JS 端叫 `Session`。
#[wasm_bindgen(js_name = Session)]
pub struct WebSession {
    engine: Engine,
    user: UserModel,
    session: Session,
    options: Options,
}

#[wasm_bindgen(js_class = Session)]
impl WebSession {
    /// 由速成資料包與方案指名的詞庫資料包的位元組建立；資料包由網頁 fetch。
    #[wasm_bindgen(constructor)]
    pub fn new(pack: &[u8], lexicon: Option<Vec<u8>>) -> Result<WebSession, JsError> {
        let engine = Engine::from_pack_bytes(pack, lexicon.as_deref())
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self {
            engine,
            user: UserModel::default(),
            session: Session::default(),
            options: Options::default(),
        })
    }

    /// 無痕模式：開著時不學習（plan §2.4）。
    #[wasm_bindgen(js_name = setIncognito)]
    pub fn set_incognito(&mut self, on: bool) {
        self.options.incognito = on;
        self.session.set_options(self.options);
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
            selected: s.composition.selected_text(),
            unselected: s.composition.unselected_input().to_string(),
            candidates: s.candidates.into_iter().map(|c| c.text).collect(),
            commit: s.commit,
        }
    }
}
