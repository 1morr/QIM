//! 殼與核心之間的介面（plan §2.1）：殼把按鍵交給 [`Session`]，拿回一份完整的 [`Snapshot`]。

use crate::Engine;

/// 候選每頁的個數，對應數字鍵 1–9（brief §4.3）。
pub const PAGE_SIZE: usize = 9;

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
    /// 整串輸入碼的全部候選。
    candidates: Vec<String>,
    /// 目前頁，從 0 起。
    page: usize,
    generation: u64,
}

impl Session {
    pub fn handle(&mut self, engine: &Engine, _user: &mut UserModel, key: Key) -> Snapshot {
        if let Key::Letter(c) = key
            && engine.scheme().alphabet.contains(c)
        {
            self.input.push(c);
            self.refresh_candidates(engine);
            return self.snapshot(true, None);
        }
        // 沒有組字時其他鍵都交還宿主：Backspace 刪輸入框的字、數字照打。
        if self.input.is_empty() {
            return self.snapshot(false, None);
        }
        match key {
            Key::Backspace => {
                self.input.pop();
                self.refresh_candidates(engine);
                self.snapshot(true, None)
            }
            Key::Escape => {
                self.clear();
                self.snapshot(true, None)
            }
            // 空白一律上屏首選，與目前在第幾頁無關（brief §4.3）。
            Key::Space => self.commit(self.candidates.first().cloned()),
            Key::Digit(digit) => match usize::from(digit).checked_sub(1) {
                Some(index) => self.commit(self.page_candidates().get(index).cloned()),
                None => self.snapshot(true, None),
            },
            Key::PageUp if self.page > 0 => self.turn_to(self.page - 1),
            Key::PageDown if self.has_more() => self.turn_to(self.page + 1),
            Key::PageUp | Key::PageDown => self.snapshot(true, None),
            _ => self.snapshot(false, None),
        }
    }

    /// 上屏 `commit` 並清空組字；沒有候選可上屏時不動作，鍵照樣吃下。
    fn commit(&mut self, commit: Option<String>) -> Snapshot {
        if commit.is_some() {
            self.clear();
        }
        self.snapshot(true, commit)
    }

    fn turn_to(&mut self, page: usize) -> Snapshot {
        self.page = page;
        self.generation += 1;
        self.snapshot(true, None)
    }

    /// 輸入碼變了：整串查候選、回到第一頁。
    fn refresh_candidates(&mut self, engine: &Engine) {
        self.candidates = engine.candidates(&self.input);
        self.page = 0;
        self.generation += 1;
    }

    fn clear(&mut self) {
        self.input.clear();
        self.candidates.clear();
        self.page = 0;
        self.generation += 1;
    }

    /// 目前頁的候選。
    fn page_candidates(&self) -> &[String] {
        let start = self.page * PAGE_SIZE;
        &self.candidates[start..self.candidates.len().min(start + PAGE_SIZE)]
    }

    fn has_more(&self) -> bool {
        self.candidates.len() > (self.page + 1) * PAGE_SIZE
    }

    fn snapshot(&self, handled: bool, commit: Option<String>) -> Snapshot {
        Snapshot {
            handled,
            composition: Composition {
                input: self.input.clone(),
                ..Composition::default()
            },
            candidates: self
                .page_candidates()
                .iter()
                .map(|text| Candidate { text: text.clone() })
                .collect(),
            page: self.page,
            has_more: self.has_more(),
            commit,
            ascii: false,
            generation: self.generation,
        }
    }
}
