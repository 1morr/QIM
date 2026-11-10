//! Apple 與 Android 殼用的綁定（uniffi，plan §1.1）：只把呼叫轉給 `qim-core`，不放邏輯（plan §1.3）。
//!
//! 介面同網頁的 `qim-wasm`：一個 [`Session`] 帶著 `Engine`、`UserModel` 與核心的 `Session`，按鍵經
//! [`Session::press`] 一個入口（`Key::parse`，uniffi 沒有 `char`，收不了核心的 `Key`）。

use std::sync::{Arc, Mutex, MutexGuard};

use qim_core::{EnglishPack, Key, SimplifierPack, UserModel};

uniffi::setup_scaffolding!("qim");

/// 核心的錯誤（資料包讀不了、按鍵種類不認得），訊息照原樣；Swift 端是丟出的錯誤。
#[derive(Debug, uniffi::Error)]
pub enum QimError {
    Core { message: String },
}

impl std::fmt::Display for QimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Core { message } => f.write_str(message),
        }
    }
}

impl std::error::Error for QimError {}

/// 核心的錯誤都經這裡轉換。
fn error(error: impl std::fmt::Display) -> QimError {
    QimError::Core {
        message: error.to_string(),
    }
}

/// 一個輸入框：資料包、一個個人模型、一個核心的 `Session`。uniffi 的物件要能跨執行緒共用，
/// 可變的狀態放在 `Mutex` 裡。
#[derive(uniffi::Object)]
pub struct Session {
    state: Mutex<State>,
}

struct State {
    engine: qim_core::Engine,
    user: UserModel,
    session: qim_core::Session,
}

#[uniffi::export]
impl Session {
    /// 由速成資料包與方案指名的詞庫、語言模型資料包，加上簡繁、英文資料包的位元組建立；
    /// 參數同 `qim-wasm` 的 `Session`。
    #[uniffi::constructor]
    pub fn new(
        scheme_pack: Vec<u8>,
        lexicon: Option<Vec<u8>>,
        language_model: Option<Vec<u8>>,
        simplifier: Option<Vec<u8>>,
        english: Option<Vec<u8>>,
    ) -> Result<Arc<Self>, QimError> {
        let mut engine = qim_core::Engine::from_pack_bytes(
            &scheme_pack,
            lexicon.as_deref(),
            language_model.as_deref(),
        )
        .map_err(error)?;
        if let Some(bytes) = simplifier {
            engine = engine.with_simplifier(SimplifierPack::from_bytes(&bytes).map_err(error)?);
        }
        if let Some(bytes) = english {
            engine = engine.with_english(EnglishPack::from_bytes(&bytes).map_err(error)?);
        }
        Ok(Arc::new(Self {
            state: Mutex::new(State {
                engine,
                user: UserModel::default(),
                session: qim_core::Session::default(),
            }),
        }))
    }

    /// 按一個鍵：種類與值照 `Key::parse`，例如 `("letter", "a")`、`("toggleAscii", "left")`、
    /// `("space", nil)`。
    pub fn press(&self, kind: String, value: Option<String>) -> Result<Snapshot, QimError> {
        let key = Key::parse(&kind, value.as_deref()).map_err(error)?;
        let state = &mut *self.lock();
        Ok(state
            .session
            .handle(&state.engine, &mut state.user, key)
            .into())
    }

    /// 選全部候選的第 `index` 個（目前頁第 i 個是 `page × 9 + i`）。
    pub fn select(&self, index: u32) -> Snapshot {
        let state = &mut *self.lock();
        state
            .session
            .select(&state.engine, &mut state.user, index as usize)
            .into()
    }
}

impl Session {
    /// 前一個呼叫在核心裡 panic 過時照樣用：組字可能停在一半，但鍵盤不該因此每個鍵都失敗。
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// 核心的 `Snapshot`，欄位照搬；`usize` 轉 `u32`（uniffi 沒有 `usize`）。組字攤平成
/// `input`、`segments`、`focus`。
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct Snapshot {
    /// 為假時殼交還宿主。
    pub handled: bool,
    pub input: String,
    pub segments: Vec<Segment>,
    /// 目前分段在 `segments` 的位置；等於分段數時候選列整串。
    pub focus: u32,
    /// 目前頁的候選。
    pub candidates: Vec<Candidate>,
    pub page: u32,
    pub has_more: bool,
    pub commit: Option<String>,
    pub commit_original: Option<String>,
    pub ascii: bool,
    pub generation: u64,
}

/// 核心的 `Segment`。
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct Segment {
    pub code: String,
    pub radicals: String,
    pub text: String,
    pub original: String,
    pub selected: bool,
    pub raw: bool,
}

/// 核心的 `Candidate`：顯示的字與後處理前的原文。
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct Candidate {
    pub text: String,
    pub original: String,
}

impl From<qim_core::Snapshot> for Snapshot {
    fn from(s: qim_core::Snapshot) -> Self {
        Self {
            handled: s.handled,
            input: s.composition.input,
            segments: s
                .composition
                .segments
                .into_iter()
                .map(|segment| Segment {
                    code: segment.code,
                    radicals: segment.radicals,
                    text: segment.text,
                    original: segment.original,
                    selected: segment.selected,
                    raw: segment.raw,
                })
                .collect(),
            focus: s.composition.focus as u32,
            candidates: s
                .candidates
                .into_iter()
                .map(|c| Candidate {
                    text: c.text,
                    original: c.original,
                })
                .collect(),
            page: s.page as u32,
            has_more: s.has_more,
            commit: s.commit,
            commit_original: s.commit_original,
            ascii: s.ascii,
            generation: s.generation,
        }
    }
}
