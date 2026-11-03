//! 網頁用的 WebAssembly 綁定：只把 JS 的呼叫轉給 `qim-core`，不放邏輯（plan §1.3）。

use qim_core::{Engine, Key, Session, UserModel};
use wasm_bindgen::prelude::*;

/// 網頁的一個輸入框：一份資料包、一個個人模型、一個 `Session`。
#[wasm_bindgen]
pub struct Ime {
    engine: Engine,
    user: UserModel,
    session: Session,
}

#[wasm_bindgen]
impl Ime {
    /// 由資料包的位元組建立；資料包由網頁 fetch。
    #[wasm_bindgen(constructor)]
    pub fn new(pack: &[u8]) -> Result<Ime, JsError> {
        let engine = Engine::from_pack_bytes(pack).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self {
            engine,
            user: UserModel::default(),
            session: Session::default(),
        })
    }

    pub fn letter(&mut self, c: char) -> Snapshot {
        self.handle(Key::Letter(c))
    }

    pub fn space(&mut self) -> Snapshot {
        self.handle(Key::Space)
    }

    fn handle(&mut self, key: Key) -> Snapshot {
        self.session
            .handle(&self.engine, &mut self.user, key)
            .into()
    }
}

/// `qim_core::Snapshot` 給 JS 的樣子：攤平成 JS 能直接讀的欄位，連打的分段到 M1 用到時再加。
#[wasm_bindgen(getter_with_clone)]
pub struct Snapshot {
    /// 為假時網頁不攔截這個鍵，交給瀏覽器。
    #[wasm_bindgen(readonly)]
    pub handled: bool,
    #[wasm_bindgen(readonly)]
    pub input: String,
    #[wasm_bindgen(readonly)]
    pub radicals: String,
    #[wasm_bindgen(readonly)]
    pub candidates: Vec<String>,
    #[wasm_bindgen(readonly)]
    pub page: usize,
    #[wasm_bindgen(readonly, js_name = hasMore)]
    pub has_more: bool,
    #[wasm_bindgen(readonly)]
    pub commit: Option<String>,
    #[wasm_bindgen(readonly)]
    pub ascii: bool,
    #[wasm_bindgen(readonly)]
    pub generation: u64,
}

impl From<qim_core::Snapshot> for Snapshot {
    fn from(s: qim_core::Snapshot) -> Self {
        Self {
            handled: s.handled,
            input: s.composition.input,
            radicals: s.composition.radicals,
            candidates: s.candidates.into_iter().map(|c| c.text).collect(),
            page: s.page,
            has_more: s.has_more,
            commit: s.commit,
            ascii: s.ascii,
            generation: s.generation,
        }
    }
}
