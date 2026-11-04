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
    /// 連打的各分段：選定的在前，接著剩下的輸入碼的首選路徑；碼接起來是 `input`。
    pub segments: Vec<Segment>,
    /// 目前分段在 `segments` 的位置，候選列它的字詞（← / →、[`Session::focus_segment`]）。
    /// 等於 `segments.len()` 時沒有目前分段：候選列剩下沒選定的整串輸入碼，是組字的預設。
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
    /// 目前分段在組字的分段的位置；`None` 是沒有目前分段，候選列剩下的整串輸入碼。
    focus: Option<usize>,
    /// 目前的全部候選：有目前分段時是那一段的碼的，否則是剩下的輸入碼的。
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

    /// 選全部候選（不只目前頁）的第 `index` 個，和按那個候選的數字鍵相同。
    pub fn select(&mut self, engine: &Engine, user: &mut UserModel, index: usize) -> Snapshot {
        if !self.composing() {
            return self.snapshot(false, None);
        }
        self.select_at(engine, user, index)
    }

    /// 目前分段換成第 `segment` 段，候選改列那一段的字詞；`segment` 是分段數時回到沒有
    /// 目前分段（候選列剩下的整串）。超出範圍時不動作。觸控點組字的某一段用它（brief §4.4）。
    pub fn focus_segment(
        &mut self,
        engine: &Engine,
        user: &mut UserModel,
        segment: usize,
    ) -> Snapshot {
        if !self.composing() {
            return self.snapshot(false, None);
        }
        if segment <= self.segments().len() {
            self.move_focus(engine, user, segment);
        }
        self.snapshot(true, None)
    }

    pub fn handle(&mut self, engine: &Engine, user: &mut UserModel, key: Key) -> Snapshot {
        if let Key::Letter(c) = key
            && engine.scheme().alphabet.contains(c)
        {
            self.focus = None;
            self.lattice.push(engine, user, c);
            self.refresh_candidates(engine, user);
            return self.snapshot(true, None);
        }
        // 沒有組字時，有全形的標點上屏全形，其他鍵都交還宿主：Backspace 刪輸入框的字、數字照打。
        if !self.composing() {
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
                self.focus = None;
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
            // 空白一律選全部候選的第一個，與目前在第幾頁無關（brief §4.3）；
            // 有目前分段時上屏整個組字（照顯示的分段，剩下的逐次取首選）。
            Key::Space if self.focus.is_some() => self.commit_all_then(engine, user, ""),
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
            // 最前面的 ← 與最後面的 → 不動作。
            Key::Left => {
                if let Some(to) = self.focus_index().checked_sub(1) {
                    self.move_focus(engine, user, to);
                }
                self.snapshot(true, None)
            }
            Key::Right => {
                let to = self.focus_index() + 1;
                if to <= self.segments().len() {
                    self.move_focus(engine, user, to);
                }
                self.snapshot(true, None)
            }
            // 方案以外的小寫字母（速成的 `z`，萬用鍵在 M4）、切換中英（票 23）、其他鍵：
            // 先吃下不動作。
            Key::Letter(_) | Key::ToggleAscii | Key::Other => self.snapshot(true, None),
        }
    }

    fn composing(&self) -> bool {
        // 剩下的輸入碼選完就上屏，所以組字中它一定不是空的。
        !self.lattice.input().is_empty()
    }

    /// 組字的各分段：選定的分段接著剩下的輸入碼的首選路徑。
    /// 剩下的輸入碼沒有路徑蓋得滿時，整段當一個沒有字的分段。
    fn segments(&self) -> Vec<Segment> {
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
        self.selected.iter().cloned().chain(unselected).collect()
    }

    /// [`Composition::focus`]：目前分段，沒有時是分段數。
    fn focus_index(&self) -> usize {
        self.focus.unwrap_or_else(|| self.segments().len())
    }

    /// 目前分段換成第 `to` 段（分段數以上是沒有目前分段），候選跟著換。
    fn move_focus(&mut self, engine: &Engine, user: &UserModel, to: usize) {
        let to = (to < self.segments().len()).then_some(to);
        if to != self.focus {
            self.focus = to;
            self.refresh_candidates(engine, user);
        }
    }

    /// 回到沒有目前分段，候選改回剩下的整串的。
    fn clear_focus(&mut self, engine: &Engine, user: &UserModel) {
        if self.focus.take().is_some() {
            self.refresh_candidates(engine, user);
        }
    }

    /// 上屏整個組字再接 `tail`：選定的分段，接著和一直按空白相同，逐次取剩下的輸入碼的首選；
    /// 剩下沒有字的碼原樣上屏。整個組字都有字時才學習，`tail` 不學。
    fn commit_all_then(&mut self, engine: &Engine, user: &mut UserModel, tail: &str) -> Snapshot {
        // 剩下的輸入碼的首選要用它自己的候選，不是目前分段的。
        self.clear_focus(engine, user);
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

    /// 選全部候選的第 `index` 個，沒有這個候選時不動作、鍵照樣吃下。
    ///
    /// 沒有目前分段時候選是剩下的輸入碼的：選了蓋住的那段固定；目前分段是第 i 段時候選是它的：
    /// 前面的分段照顯示的字固定，第 i 段換成選的字詞。選的只蓋住第 i 段的開頭時，那段剩下的碼
    /// 連同後面的分段一起重算，否則後面已選定的分段不動、沒選定的重算。都選定了就上屏。
    fn select_at(&mut self, engine: &Engine, user: &mut UserModel, index: usize) -> Snapshot {
        let Some(choice) = self.candidates.get(index).cloned() else {
            return self.snapshot(true, None);
        };
        let segments = self.segments();
        // 候選列的是從第 `at` 段起的 `code`，`after` 是它後面的輸入碼。
        let (at, code, after) = match self.focus {
            Some(i) => (
                i,
                segments[i].code.clone(),
                segments[i + 1..].iter().map(|s| s.code.as_str()).collect(),
            ),
            None => (
                self.selected.len(),
                self.lattice.input().to_string(),
                String::new(),
            ),
        };
        let mut fixed: Vec<Segment> = segments[..at]
            .iter()
            .map(|s| Segment {
                selected: true,
                ..s.clone()
            })
            .chain(
                owned_segments(&code, &choice)
                    .into_iter()
                    .map(|(code, text)| Segment {
                        code,
                        text,
                        selected: true,
                    }),
            )
            .collect();
        let rest = if choice.len == code.len() && at < self.selected.len() {
            fixed.extend_from_slice(&self.selected[at + 1..]);
            self.lattice.input().to_string()
        } else {
            code[choice.len..].to_string() + &after
        };
        self.selected = fixed;
        self.focus = None;
        if rest.is_empty() {
            self.learn(engine, user, &self.selected_segments());
            return self.commit(self.selected_text());
        }
        self.recompose(engine, user, &rest);
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

    /// 輸入碼或目前分段變了：重列候選、回到第一頁。目前分段的候選從頭建，和單打它的碼相同。
    fn refresh_candidates(&mut self, engine: &Engine, user: &UserModel) {
        self.candidates = match self.focus {
            Some(i) => {
                let code = &self.segments()[i].code;
                lattice::candidates(engine, user, &Lattice::from_input(engine, user, code))
            }
            None => lattice::candidates(engine, user, &self.lattice),
        };
        self.page = 0;
        self.generation += 1;
    }

    fn clear(&mut self) {
        self.selected.clear();
        self.lattice = Lattice::default();
        self.focus = None;
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

    fn composition(&self) -> Composition {
        let segments = self.segments();
        Composition {
            input: segments.iter().map(|s| s.code.as_str()).collect(),
            focus: self.focus.unwrap_or(segments.len()),
            segments,
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
