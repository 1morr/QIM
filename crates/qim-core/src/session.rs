//! 殼與核心之間的介面（plan §2.1）：殼把按鍵交給 [`Session`]，拿回一份完整的 [`Snapshot`]。

use crate::Engine;
use crate::lattice::{self, CandidateSpan, Lattice};

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
    /// 連打的各分段：選定的在前，接著剩下的輸入碼的首選路徑。
    pub segments: Vec<Segment>,
    /// 目前分段在 `segments` 的位置。
    pub focus: usize,
}

impl Composition {
    /// 已選定的分段接起來的字詞。
    pub fn selected_text(&self) -> String {
        self.segments
            .iter()
            .filter(|s| s.selected)
            .map(|s| s.text.as_str())
            .collect()
    }

    /// 還沒選定的輸入碼：整串輸入碼去掉開頭選定的分段的碼。
    pub fn unselected_input(&self) -> &str {
        let selected: usize = self
            .segments
            .iter()
            .filter(|s| s.selected)
            .map(|s| s.code.len())
            .sum();
        &self.input[selected..]
    }
}

/// 連打的一個分段：一段輸入碼和它的字詞。
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
    /// 已選定的分段，依輸入碼的順序；重算時不動。
    selected: Vec<Segment>,
    /// 選定的分段之後、還沒選定的輸入碼的 lattice，逐鍵補算。
    lattice: Lattice,
    /// 剩下的輸入碼的全部候選。
    candidates: Vec<CandidateSpan>,
    /// 目前頁，從 0 起。
    page: usize,
    generation: u64,
}

impl Session {
    pub fn handle(&mut self, engine: &Engine, _user: &mut UserModel, key: Key) -> Snapshot {
        if let Key::Letter(c) = key
            && engine.scheme().alphabet.contains(c)
        {
            self.lattice.push(engine, c);
            self.refresh_candidates(engine);
            return self.snapshot(true, None);
        }
        // 沒有組字時其他鍵都交還宿主：Backspace 刪輸入框的字、數字照打。
        if self.lattice.input().is_empty() {
            return self.snapshot(false, None);
        }
        match key {
            Key::Backspace => {
                self.lattice.pop();
                // 剩下的輸入碼刪光了，就把最後選定的分段放回來重選。
                if self.lattice.input().is_empty()
                    && let Some(segment) = self.selected.pop()
                {
                    self.recompose(engine, &segment.code);
                }
                self.refresh_candidates(engine);
                self.snapshot(true, None)
            }
            Key::Escape => {
                self.clear();
                self.snapshot(true, None)
            }
            // 空白一律選全部候選的第一個，與目前在第幾頁無關（brief §4.3）。
            Key::Space => self.select_at(engine, 0),
            Key::Digit(digit) => match usize::from(digit).checked_sub(1) {
                Some(index) if index < self.page_candidates().len() => {
                    self.select_at(engine, self.page * PAGE_SIZE + index)
                }
                _ => self.snapshot(true, None),
            },
            Key::PageUp if self.page > 0 => self.turn_to(self.page - 1),
            Key::PageDown if self.has_more() => self.turn_to(self.page + 1),
            Key::PageUp | Key::PageDown => self.snapshot(true, None),
            _ => self.snapshot(false, None),
        }
    }

    /// 選全部候選的第 `index` 個：蓋滿剩下的輸入碼就連同選定的分段一起上屏、清空組字；
    /// 只蓋住開頭一段就把那段固定，其餘的輸入碼重算。沒有這個候選時不動作，鍵照樣吃下。
    fn select_at(&mut self, engine: &Engine, index: usize) -> Snapshot {
        let Some(choice) = self.candidates.get(index).cloned() else {
            return self.snapshot(true, None);
        };
        let input = self.lattice.input().to_string();
        if choice.len == input.len() {
            let commit: String = self
                .selected
                .iter()
                .map(|s| s.text.as_str())
                .chain([choice.text.as_str()])
                .collect();
            self.clear();
            return self.snapshot(true, Some(commit));
        }
        let (code, rest) = input.split_at(choice.len);
        self.selected.push(Segment {
            code: code.to_string(),
            text: choice.text,
            selected: true,
        });
        self.recompose(engine, rest);
        self.refresh_candidates(engine);
        self.snapshot(true, None)
    }

    /// 剩下的輸入碼換成 `input`，lattice 從頭建。
    fn recompose(&mut self, engine: &Engine, input: &str) {
        self.lattice = Lattice::from_input(engine, input);
    }

    fn turn_to(&mut self, page: usize) -> Snapshot {
        self.page = page;
        self.generation += 1;
        self.snapshot(true, None)
    }

    /// 輸入碼變了：重列候選、回到第一頁。
    fn refresh_candidates(&mut self, engine: &Engine) {
        self.candidates = lattice::candidates(engine, &self.lattice);
        self.page = 0;
        self.generation += 1;
    }

    fn clear(&mut self) {
        self.selected.clear();
        self.lattice = Lattice::default();
        self.candidates.clear();
        self.page = 0;
        self.generation += 1;
    }

    /// 目前頁的候選。
    fn page_candidates(&self) -> &[CandidateSpan] {
        let start = self.page * PAGE_SIZE;
        &self.candidates[start..self.candidates.len().min(start + PAGE_SIZE)]
    }

    fn has_more(&self) -> bool {
        self.candidates.len() > (self.page + 1) * PAGE_SIZE
    }

    /// 組字：選定的分段接著剩下的輸入碼的首選路徑；目前分段是第一個沒選定的。
    /// 剩下的輸入碼沒有路徑蓋得滿時，整段當一個沒有字的分段。
    fn composition(&self) -> Composition {
        let rest = self.lattice.input();
        let mut unselected: Vec<Segment> = self
            .lattice
            .best_segments()
            .into_iter()
            .map(|s| Segment {
                code: rest[s.start..s.end].to_string(),
                text: s.text,
                selected: false,
            })
            .collect();
        if unselected.is_empty() && !rest.is_empty() {
            unselected.push(Segment {
                code: rest.to_string(),
                text: String::new(),
                selected: false,
            });
        }
        Composition {
            input: self
                .selected
                .iter()
                .map(|s| s.code.as_str())
                .chain([rest])
                .collect(),
            segments: self.selected.iter().cloned().chain(unselected).collect(),
            focus: self.selected.len(),
            ..Composition::default()
        }
    }

    fn snapshot(&self, handled: bool, commit: Option<String>) -> Snapshot {
        Snapshot {
            handled,
            composition: self.composition(),
            candidates: self
                .page_candidates()
                .iter()
                .map(|c| Candidate {
                    text: c.text.clone(),
                })
                .collect(),
            page: self.page,
            has_more: self.has_more(),
            commit,
            ascii: false,
            generation: self.generation,
        }
    }
}
