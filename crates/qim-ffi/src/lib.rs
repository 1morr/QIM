//! Apple 與 Android 殼用的綁定（uniffi，plan §1.1）：只把呼叫轉給 `qim-core`，不放邏輯（plan §1.3）。
//!
//! 殼先把各資料包的位元組解開成 [`SchemePack`] 等物件、建一個 [`Engine`]，每個輸入框一個
//! [`Session`] 共用它（plan §2.1）；`Session` 帶著 `UserModel` 與核心的 `Session`。按鍵經
//! [`Session::press`] 一個入口（`Key::parse`，uniffi 沒有 `char`，收不了核心的 `Key`）。

use std::sync::{Arc, Mutex, MutexGuard};

use qim_core::{Key, Pack, UserModel};

uniffi::setup_scaffolding!("qim");

/// 核心與綁定的錯誤（資料包讀不了、按鍵種類不認得、一份資料包給了兩個 `Engine`），訊息照原樣；
/// Swift 端是丟出的錯誤。
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

/// 錯誤都經這裡轉成 `QimError::Core`（同 `qim-wasm` 的 `js_error`）。
fn core_error(error: impl std::fmt::Display) -> QimError {
    QimError::Core {
        message: error.to_string(),
    }
}

/// 一份資料包解開後的內容，交給 [`Engine::new`] 一次：建 `Engine` 時被拿走，不複製（資料包解開後可達
/// 數十 MB，ios-feasibility〈Mac 上的資料包記憶體〉）。殼能分開讀每份，量各自的記憶體。
macro_rules! pack {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(uniffi::Object)]
        pub struct $name(Mutex<Option<qim_core::$name>>);

        #[uniffi::export]
        impl $name {
            /// 由資料包檔的位元組解開；讀檔是殼的事（plan §1.3）。
            #[uniffi::constructor]
            pub fn new(bytes: Vec<u8>) -> Result<Arc<Self>, QimError> {
                let pack = qim_core::$name::from_bytes(&bytes).map_err(core_error)?;
                Ok(Arc::new(Self(Mutex::new(Some(pack)))))
            }
        }

        impl $name {
            fn take(&self) -> Result<qim_core::$name, QimError> {
                self.0
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take()
                    .ok_or_else(|| core_error(concat!(stringify!($name), " 已經給過一個 Engine")))
            }
        }
    };
}

pack!(
    /// 速成等方案的資料包（方案與碼表）。
    SchemePack
);
pack!(
    /// 方案指名的詞庫資料包。
    LexiconPack
);
pack!(
    /// 方案指名的語言模型資料包（字級 bigram）。
    LanguageModelPack
);
pack!(
    /// 簡繁資料包，各方案共用。
    SimplifierPack
);
pack!(
    /// 英文資料包，各方案共用。
    EnglishPack
);

/// 字根提示資料包（plan §4.1 步驟 5），不給 `Engine`。目前只能解開：鍵盤擴充用它量記憶體
/// （M2 票 09）；查詢與顯示的介面由殼要顯示字根提示時的票加。
#[derive(uniffi::Object)]
pub struct RadicalHintPack(
    // 還沒有讀它的介面，解開後只為了佔住記憶體（見上方說明）。
    #[allow(dead_code)] qim_core::RadicalHintPack,
);

#[uniffi::export]
impl RadicalHintPack {
    /// 由資料包檔的位元組解開；讀檔是殼的事（plan §1.3）。
    #[uniffi::constructor]
    pub fn new(bytes: Vec<u8>) -> Result<Arc<Self>, QimError> {
        let pack = qim_core::RadicalHintPack::from_bytes(&bytes).map_err(core_error)?;
        Ok(Arc::new(Self(pack)))
    }
}

/// 核心的 `Engine`：不可變，多個 [`Session`] 共用（plan §2.1）。
#[derive(uniffi::Object)]
pub struct Engine(qim_core::Engine);

#[uniffi::export]
impl Engine {
    /// 參數同核心的 `Engine::new` 加 `with_simplifier`、`with_english`；每份資料包只能給一個 `Engine`。
    #[uniffi::constructor]
    pub fn new(
        scheme: Arc<SchemePack>,
        lexicon: Option<Arc<LexiconPack>>,
        language_model: Option<Arc<LanguageModelPack>>,
        simplifier: Option<Arc<SimplifierPack>>,
        english: Option<Arc<EnglishPack>>,
    ) -> Result<Arc<Self>, QimError> {
        let mut engine = qim_core::Engine::new(
            scheme.take()?,
            lexicon.map(|p| p.take()).transpose()?,
            language_model.map(|p| p.take()).transpose()?,
        )
        .map_err(core_error)?;
        if let Some(pack) = simplifier {
            engine = engine.with_simplifier(pack.take()?);
        }
        if let Some(pack) = english {
            engine = engine.with_english(pack.take()?);
        }
        Ok(Arc::new(Self(engine)))
    }
}

/// 一個輸入框：共用的 [`Engine`]、一個個人模型、一個核心的 `Session`。uniffi 的物件要能跨執行緒
/// 共用，可變的狀態放在 `Mutex` 裡。
#[derive(uniffi::Object)]
pub struct Session {
    engine: Arc<Engine>,
    state: Mutex<State>,
}

struct State {
    user: UserModel,
    session: qim_core::Session,
}

#[uniffi::export]
impl Session {
    #[uniffi::constructor]
    pub fn new(engine: Arc<Engine>) -> Arc<Self> {
        Arc::new(Self {
            engine,
            state: Mutex::new(State {
                user: UserModel::default(),
                session: qim_core::Session::default(),
            }),
        })
    }

    /// 按一個鍵：種類與值照 `Key::parse`，例如 `("letter", "a")`、`("toggleAscii", "left")`、
    /// `("space", nil)`。
    pub fn press(&self, kind: String, value: Option<String>) -> Result<Snapshot, QimError> {
        let key = Key::parse(&kind, value.as_deref()).map_err(core_error)?;
        let state = &mut *self.lock();
        Ok(state
            .session
            .handle(&self.engine.0, &mut state.user, key)
            .into())
    }

    /// 選全部候選的第 `index` 個（目前頁第 i 個是 `page × 9 + i`）。
    pub fn select(&self, index: u32) -> Snapshot {
        let state = &mut *self.lock();
        state
            .session
            .select(&self.engine.0, &mut state.user, index as usize)
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
