//! 殼與核心之間的介面（plan §2.1）：殼把按鍵交給 [`Session`]，拿回一份完整的 [`Snapshot`]。

use crate::{CodeTable, Pack, PackError, Scheme};

/// 候選每頁的個數，對應數字鍵 1–9（brief §4.3）。
pub const PAGE_SIZE: usize = 9;

/// 不可變的方案與資料包，多個 [`Session`] 共用。
#[derive(Debug, Clone)]
pub struct Engine {
    scheme: Scheme,
    code_table: CodeTable,
}

impl Engine {
    /// 由資料包的位元組建立；讀檔是呼叫端的事（plan §1.3）。
    pub fn from_pack_bytes(bytes: &[u8]) -> Result<Self, PackError> {
        let Pack { scheme, code_table } = Pack::from_bytes(bytes)?;
        Ok(Self { scheme, code_table })
    }
}

/// 使用者詞與學習計數；內容在 M1 做，現在只讓簽章成立。
#[derive(Debug, Clone, Default)]
pub struct UserModel {}

/// 殼給的選項；實際作用在 M1 做，現在只讓簽章成立。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Options {
    pub simplified: bool,
    pub incognito: bool,
    pub english_candidates: bool,
}

/// 殼轉過來的按鍵。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Letter(char),
    Space,
    Enter,
    Backspace,
    Escape,
    Digit(u8),
    PageUp,
    PageDown,
    Left,
    Right,
    ToggleAscii,
}

/// 組字狀態。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Composition {
    /// 已打的輸入碼。
    pub input: String,
    /// 已打的字根，如「日弓口」（M1）。
    pub radicals: String,
    /// 連打的各分段（M1）。
    pub segments: Vec<Segment>,
    /// 目前分段在 `segments` 的位置。
    pub focus: usize,
}

/// 連打的一個分段（M1）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Segment {
    pub code: String,
    pub text: String,
    /// 使用者已選定，重算時不動。
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub text: String,
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
    pub ascii: bool,
    /// 每次候選變動加一，非同步重排用。
    pub generation: u64,
}

/// 一個輸入框的組字狀態。
#[derive(Debug, Clone, Default)]
pub struct Session {
    input: String,
    candidates: Vec<String>,
    generation: u64,
}

impl Session {
    pub fn handle(&mut self, engine: &Engine, _user: &mut UserModel, key: Key) -> Snapshot {
        match key {
            Key::Letter(c) if engine.scheme.alphabet.contains(c) => {
                self.input.push(c);
                self.candidates = engine
                    .code_table
                    .lookup(&self.input)
                    .iter()
                    .map(|e| e.text.to_string())
                    .collect();
                self.generation += 1;
                self.snapshot(true, None)
            }
            Key::Space if !self.input.is_empty() => {
                let commit = self.candidates.first().cloned();
                if commit.is_some() {
                    self.clear();
                }
                self.snapshot(true, commit)
            }
            _ => self.snapshot(false, None),
        }
    }

    fn clear(&mut self) {
        self.input.clear();
        self.candidates.clear();
        self.generation += 1;
    }

    fn snapshot(&self, handled: bool, commit: Option<String>) -> Snapshot {
        Snapshot {
            handled,
            composition: Composition {
                input: self.input.clone(),
                ..Composition::default()
            },
            candidates: self
                .candidates
                .iter()
                .take(PAGE_SIZE)
                .map(|text| Candidate { text: text.clone() })
                .collect(),
            page: 0,
            has_more: self.candidates.len() > PAGE_SIZE,
            commit,
            ascii: false,
            generation: self.generation,
        }
    }
}
