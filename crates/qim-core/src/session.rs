//! 殼與核心之間的介面（plan §2.1）：殼把按鍵交給 [`Session`]，拿回一份完整的 [`Snapshot`]。

use crate::lattice::{self, CandidateSpan, Lattice};
use crate::{Engine, UserModel, punctuation};

/// 候選每頁的個數，對應數字鍵 1–9（brief §4.3）。
pub const PAGE_SIZE: usize = 9;

/// 殼給的選項（plan §2.1），由 [`Session::set_options`] 換。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Options {
    /// 簡體輸出（M1 票 16）；還沒有作用。
    pub simplified: bool,
    /// 無痕模式：不寫任何學習資料（plan §2.4）。
    pub incognito: bool,
    /// 英文候選（M1 票 15）；還沒有作用。
    pub english_candidates: bool,
}

/// 殼轉過來的按鍵。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// 字母，含大寫；方案的字母表以外的也是，由核心決定怎麼處理。
    Letter(char),
    /// 數字、字母、空白以外的可見 ASCII 字元（`,`、`.`、`@`…）；`-` / `=` 殼轉成翻頁。
    Punct(char),
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
    /// 殼不轉成上面各種的鍵（Tab、Delete、↑ ↓、Home、End…）：組字中吃下，免得宿主拿它改輸入框。
    Other,
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
    options: Options,
}

impl Session {
    pub fn set_options(&mut self, options: Options) {
        self.options = options;
    }

    pub fn handle(&mut self, engine: &Engine, user: &mut UserModel, key: Key) -> Snapshot {
        if let Key::Letter(c) = key
            && engine.scheme().alphabet.contains(c)
        {
            self.lattice.push(engine, user, c);
            self.refresh_candidates(engine, user);
            return self.snapshot(true, None);
        }
        // 沒有組字時，有全形的標點上屏全形，其他鍵都交還宿主：Backspace 刪輸入框的字、數字照打。
        if self.lattice.input().is_empty() {
            return match key {
                Key::Punct(c) if let Some(full) = punctuation::full_width(c) => {
                    self.snapshot(true, Some(full.to_string()))
                }
                _ => self.snapshot(false, None),
            };
        }
        // 組字中每個鍵都吃下：交還宿主的話宿主照打那個字元，組字卻還留著。
        match key {
            Key::Backspace => {
                self.lattice.pop();
                // 剩下的輸入碼刪光了，就把最後選定的分段放回來重選。
                if self.lattice.input().is_empty()
                    && let Some(segment) = self.selected.pop()
                {
                    self.recompose(engine, user, &segment.code);
                }
                self.refresh_candidates(engine, user);
                self.snapshot(true, None)
            }
            Key::Escape => {
                self.clear();
                self.snapshot(true, None)
            }
            // 空白一律選全部候選的第一個，與目前在第幾頁無關（brief §4.3）。
            Key::Space => self.select_at(engine, user, 0),
            Key::Digit(digit) => match usize::from(digit).checked_sub(1) {
                Some(index) if index < self.page_candidates().len() => {
                    self.select_at(engine, user, self.page * PAGE_SIZE + index)
                }
                _ => self.snapshot(true, None),
            },
            Key::PageUp if self.page > 0 => self.turn_to(self.page - 1),
            Key::PageDown if self.has_more() => self.turn_to(self.page + 1),
            Key::PageUp | Key::PageDown => self.snapshot(true, None),
            Key::Enter => {
                let commit = self.selected_text() + self.lattice.input();
                self.commit(commit)
            }
            Key::Punct(c) => {
                let punct = punctuation::full_width(c).map_or_else(|| c.to_string(), String::from);
                self.commit_all_then(engine, user, &punct)
            }
            Key::Letter(c) if !c.is_ascii_lowercase() => {
                self.commit_all_then(engine, user, &c.to_string())
            }
            // 方案以外的小寫字母（速成的 `z`，萬用鍵在 M4）、← / →（組字中修改，票 21）、
            // 切換中英（票 23）、其他鍵：先吃下不動作。
            Key::Letter(_) | Key::Left | Key::Right | Key::ToggleAscii | Key::Other => {
                self.snapshot(true, None)
            }
        }
    }

    /// 直接輸入一段文字，不經組字（觸控鍵盤的 `text` 動作，plan §2.6）：沒有組字時上屏 `text`；
    /// 組字中和標點相同，先上屏整個組字再接 `text`。文字照原樣，不轉全形。
    pub fn text(&mut self, engine: &Engine, _user: &mut UserModel, text: &str) -> Snapshot {
        if self.lattice.input().is_empty() {
            return self.snapshot(true, Some(text.to_string()));
        }
        self.commit_all_then(engine, text)
    }

    /// 上屏整個組字再接 `tail`：選定的分段，接著和一直按空白相同，逐次取剩下的輸入碼的首選；
    /// 剩下沒有字的碼原樣上屏。整個組字都有字時才學習，`tail` 不學。
    fn commit_all_then(&mut self, engine: &Engine, user: &mut UserModel, tail: &str) -> Snapshot {
        let mut segments = self.selected_segments();
        let mut rest = self.lattice.input().to_string();
        let mut top = self.candidates.first().cloned();
        let mut raw = String::new();
        while !rest.is_empty() {
            let Some(choice) = top else {
                raw = rest;
                break;
            };
            segments.extend(owned_segments(&rest, &choice));
            rest.drain(..choice.len);
            top = lattice::candidates(engine, user, &Lattice::from_input(engine, user, &rest))
                .into_iter()
                .next();
        }
        let mut commit: String = segments.iter().map(|(_, text)| text.as_str()).collect();
        if raw.is_empty() {
            self.learn(engine, user, &segments);
        }
        commit.push_str(&raw);
        commit.push_str(tail);
        self.commit(commit)
    }

    /// 選定的分段的（輸入碼, 字詞）。
    fn selected_segments(&self) -> Vec<(String, String)> {
        self.selected
            .iter()
            .map(|s| (s.code.clone(), s.text.clone()))
            .collect()
    }

    /// 學一次上屏（plan §2.4）；無痕模式不學。
    fn learn(&self, engine: &Engine, user: &mut UserModel, segments: &[(String, String)]) {
        if self.options.incognito {
            return;
        }
        let segments: Vec<(&str, &str)> = segments
            .iter()
            .map(|(code, text)| (code.as_str(), text.as_str()))
            .collect();
        user.learn(&engine.scheme().id, &segments);
    }

    /// 已選定的分段接起來的字詞。
    fn selected_text(&self) -> String {
        self.selected.iter().map(|s| s.text.as_str()).collect()
    }

    /// 上屏 `text`、清空組字。
    fn commit(&mut self, text: String) -> Snapshot {
        self.clear();
        self.snapshot(true, Some(text))
    }

    /// 選全部候選的第 `index` 個：蓋滿剩下的輸入碼就連同選定的分段一起上屏、清空組字；
    /// 只蓋住開頭一段就把那段固定，其餘的輸入碼重算。沒有這個候選時不動作，鍵照樣吃下。
    fn select_at(&mut self, engine: &Engine, user: &mut UserModel, index: usize) -> Snapshot {
        let Some(choice) = self.candidates.get(index).cloned() else {
            return self.snapshot(true, None);
        };
        let input = self.lattice.input().to_string();
        if choice.len == input.len() {
            let mut segments = self.selected_segments();
            segments.extend(owned_segments(&input, &choice));
            self.learn(engine, user, &segments);
            return self.commit(self.selected_text() + &choice.text);
        }
        let (code, rest) = input.split_at(choice.len);
        self.selected.push(Segment {
            code: code.to_string(),
            text: choice.text,
            selected: true,
        });
        self.recompose(engine, user, rest);
        self.refresh_candidates(engine, user);
        self.snapshot(true, None)
    }

    /// 剩下的輸入碼換成 `input`，lattice 從頭建。
    fn recompose(&mut self, engine: &Engine, user: &UserModel, input: &str) {
        self.lattice = Lattice::from_input(engine, user, input);
    }

    fn turn_to(&mut self, page: usize) -> Snapshot {
        self.page = page;
        self.generation += 1;
        self.snapshot(true, None)
    }

    /// 輸入碼變了：重列候選、回到第一頁。
    fn refresh_candidates(&mut self, engine: &Engine, user: &UserModel) {
        self.candidates = lattice::candidates(engine, user, &self.lattice);
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

/// 候選 `choice` 的各段的（輸入碼, 字詞）；`input` 是它所在的那串輸入碼。
fn owned_segments(input: &str, choice: &CandidateSpan) -> Vec<(String, String)> {
    choice
        .segments
        .iter()
        .map(|s| (input[s.start..s.end].to_string(), s.text.clone()))
        .collect()
}
