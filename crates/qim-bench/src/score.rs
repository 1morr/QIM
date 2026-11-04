//! 用核心實際打每一題、算分數：只經 `Session::handle`，不重寫排序（plan §1.3）。
//!
//! 每題打兩次，都用新的 `Session` 與空白的 `UserModel`，題與題互不影響（plan §5.2；
//! 學習模式 [`learning`] 例外）：
//! - 逐鍵打完輸入碼不選字：最後的首選算正確率，途中的首選算跳動。
//! - 模擬使用者選字打出整題：算修正次數與每字按鍵數（[`simulate`]）。

use std::collections::BTreeSet;
use std::fmt;

use qim_core::{Candidate, Engine, Key, Options, Session, Snapshot, UserModel};

use crate::items::{Item, Unit};

/// 一題的結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemResult {
    pub line: usize,
    pub weight: u64,
    /// 期望首選的字數。
    pub chars: usize,
    /// 整句首選正確。
    pub correct: bool,
    /// 首選與期望的編輯距離（以字計，最多算到該題字數）。
    pub char_errors: usize,
    /// 模擬打出整題時選非首選、翻頁、清掉重打的次數。
    pub correction_count: usize,
    /// 模擬打出整題的總按鍵數，含輸入碼。
    pub keys: usize,
    /// 逐鍵打字時，首選裡已打完的字被下一鍵改掉的次數。
    pub jitter: usize,
    /// 模擬打得出整題；一個單位一個單位打、翻完所有頁都接不上時為假。
    pub finished: bool,
}

/// 一個集合的結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetResult {
    pub name: String,
    /// 個人集：輸出只有數字與行號，baseline 在本機的另一個檔（plan §5.2）。
    pub private: bool,
    pub items: Vec<ItemResult>,
    /// 答錯的題：題目與實際的首選。
    pub failures: Vec<Failure>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub item: Item,
    /// 打完輸入碼後的首選；沒有候選時是空字串。
    pub top: String,
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}（{}）期望「{}」，首選「{}」",
            self.item.input(),
            self.item.label,
            self.item.expected(),
            self.top
        )
    }
}

/// 一組題目的比率；不加權時每題算一次，加權時每題算 `weight` 次。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// 整句首選正確率。
    pub sentence_accuracy: f64,
    /// 字正確率：1 − 編輯距離總和 ÷ 總字數。
    pub char_accuracy: f64,
    /// 每句平均修正次數。
    pub correction_count: f64,
    /// 每字按鍵數（KSPC）：總按鍵數 ÷ 總字數。
    pub kspc: f64,
    /// 每句平均首選跳動。
    pub jitter: f64,
}

/// 依長度分組的上界（含）與標籤：目標字數 1、2、3–4、5–8、9 以上（plan §5.2）。
pub const LENGTH_GROUPS: [(usize, &str); 5] = [
    (1, "1 字"),
    (2, "2 字"),
    (4, "3–4 字"),
    (8, "5–8 字"),
    (usize::MAX, "9 字以上"),
];

/// 一個長度組：題數、整句首選正確率、每句平均修正次數（不加權）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroupStats {
    pub total: usize,
    pub sentence_accuracy: f64,
    pub correction_count: f64,
}

impl SetResult {
    pub fn total(&self) -> usize {
        self.items.len()
    }

    /// 整句首選正確的題數。
    pub fn correct(&self) -> usize {
        self.items.iter().filter(|i| i.correct).count()
    }

    pub fn wrong(&self) -> usize {
        self.total() - self.correct()
    }

    /// 修正次數的總和；baseline 記它。
    pub fn correction_count(&self) -> usize {
        self.items.iter().map(|i| i.correction_count).sum()
    }

    /// 模擬打不出整題的題數。
    pub fn unfinished(&self) -> usize {
        self.items.iter().filter(|i| !i.finished).count()
    }

    /// 答錯的題的行號。
    pub fn wrong_lines(&self) -> BTreeSet<usize> {
        self.items
            .iter()
            .filter(|i| !i.correct)
            .map(|i| i.line)
            .collect()
    }

    pub fn metrics(&self, weighted: bool) -> Metrics {
        metrics(self.items.iter(), weighted)
    }

    /// 依 [`LENGTH_GROUPS`] 分組；沒有題的組是 `None`。
    pub fn length_groups(&self) -> [Option<GroupStats>; 5] {
        let mut lower = 0;
        LENGTH_GROUPS.map(|(upper, _)| {
            let group: Vec<&ItemResult> = self
                .items
                .iter()
                .filter(|i| i.chars > lower && i.chars <= upper)
                .collect();
            lower = upper;
            (!group.is_empty()).then(|| {
                let m = metrics(group.iter().copied(), false);
                GroupStats {
                    total: group.len(),
                    sentence_accuracy: m.sentence_accuracy,
                    correction_count: m.correction_count,
                }
            })
        })
    }
}

fn metrics<'a>(items: impl Iterator<Item = &'a ItemResult> + Clone, weighted: bool) -> Metrics {
    let w = |i: &ItemResult| if weighted { i.weight as f64 } else { 1.0 };
    let sum = |f: &dyn Fn(&ItemResult) -> f64| items.clone().map(|i| w(i) * f(i)).sum::<f64>();
    let count = sum(&|_| 1.0);
    let chars = sum(&|i| i.chars as f64);
    Metrics {
        sentence_accuracy: ratio(sum(&|i| f64::from(u8::from(i.correct))), count),
        char_accuracy: 1.0 - ratio(sum(&|i| i.char_errors as f64), chars),
        correction_count: ratio(sum(&|i| i.correction_count as f64), count),
        kspc: ratio(sum(&|i| i.keys as f64), chars),
        jitter: ratio(sum(&|i| i.jitter as f64), count),
    }
}

fn ratio(part: f64, whole: f64) -> f64 {
    if whole == 0.0 { 0.0 } else { part / whole }
}

pub fn score(engine: &Engine, name: &str, items: &[Item], private: bool) -> SetResult {
    let mut result = SetResult {
        name: name.to_string(),
        private,
        items: Vec::with_capacity(items.len()),
        failures: Vec::new(),
    };
    for item in items {
        let expected = item.expected();
        let chars = expected.chars().count();
        let (top, jitter) = type_through(engine, &item.units);
        let typing = simulate(engine, None, &item.units);
        let correct = top == expected;
        result.items.push(ItemResult {
            line: item.line,
            weight: item.weight,
            chars,
            correct,
            char_errors: strsim::levenshtein(&top, &expected).min(chars),
            correction_count: typing.correction_count,
            keys: typing.keys,
            jitter,
            finished: typing.finished,
        });
        if !correct {
            result.failures.push(Failure {
                item: item.clone(),
                top,
            });
        }
    }
    result
}

/// 學習模式的一個集合：修正次數的總和，學習關與學習開。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearningSet {
    pub name: String,
    /// 個人集：輸出只有數字（plan §5.2）。
    pub private: bool,
    pub total: usize,
    /// 每題用空白的 `UserModel`，不學。
    pub corrections_off: usize,
    /// 照順序打，整個集合共用一個 `UserModel`。
    pub corrections_on: usize,
}

/// 學習模式（plan §5.2）：照 `items` 的順序各模擬一次，學習關與學習開。
pub fn learning(engine: &Engine, name: &str, items: &[Item], private: bool) -> LearningSet {
    let mut learned = UserModel::default();
    let (mut off, mut on) = (0, 0);
    for item in items {
        off += simulate(engine, None, &item.units).correction_count;
        on += simulate(engine, Some(&mut learned), &item.units).correction_count;
    }
    LearningSet {
        name: name.to_string(),
        private,
        total: items.len(),
        corrections_off: off,
        corrections_on: on,
    }
}

fn top_of(snapshot: &Snapshot) -> String {
    snapshot
        .candidates
        .first()
        .map(|c| c.text.clone())
        .unwrap_or_default()
}

/// 逐鍵打完輸入碼、不選字：回傳最後的首選與跳動次數。
///
/// 跳動：每一鍵之前，首選開頭屬於已打完的單位的那幾個字（首選比它短就取整個首選），
/// 被這一鍵改掉就算一次。
fn type_through(engine: &Engine, units: &[Unit]) -> (String, usize) {
    let mut session = Session::default();
    let mut user = UserModel::default();
    let (mut top, mut done_chars, mut jitter) = (String::new(), 0, 0);
    for unit in units {
        for c in unit.code.chars() {
            let next = top_of(&session.handle(engine, &mut user, Key::Letter(c)));
            let settled: String = top.chars().take(done_chars).collect();
            if !next.starts_with(&settled) {
                jitter += 1;
            }
            top = next;
        }
        done_chars += unit.text.chars().count();
    }
    (top, jitter)
}

/// 模擬使用者打出一題的結果。
#[derive(Debug, Default)]
struct Typing {
    keys: usize,
    correction_count: usize,
    finished: bool,
}

/// 修正次數的模擬（plan §5.2）：打完整題的輸入碼，接著反覆
/// 1. 第一頁的首選接得上還沒打出的文字（已選定的分段之後）就按空白；
/// 2. 否則選這頁接得上的最長候選（數字鍵）；
/// 3. 這頁都接不上就翻頁。
///
/// 選字、翻頁各算一次修正。選的候選只蓋住剩下輸入碼的開頭時，那段固定、其餘繼續組字；
/// 蓋滿時連同固定的分段一起上屏。上屏後組字清空、題目還沒打完，就把剩下的單位的碼再打一次。
/// 翻完所有頁都接不上（卡住）時：還剩兩個以上單位就按 Esc 清掉（算一次修正），
/// 之後一個單位一個單位打；只剩一個單位，或已經一個一個打了還卡住，就是打不出這題。
///
/// 有 `learning` 時上屏的字詞照常學進它；沒有時用空白的 `UserModel` 並開無痕，題內也不學。
fn simulate(engine: &Engine, learning: Option<&mut UserModel>, units: &[Unit]) -> Typing {
    let mut session = Session::default();
    let mut blank = UserModel::default();
    let user = match learning {
        Some(user) => user,
        None => {
            session.set_options(Options {
                incognito: true,
                ..Options::default()
            });
            &mut blank
        }
    };
    let mut typing = Typing::default();
    let mut press = |key: Key, typing: &mut Typing| {
        typing.keys += 1;
        session.handle(engine, user, key)
    };
    let target: String = units.iter().map(|u| u.text.as_str()).collect();
    // 每個單位在目標裡的開頭（以字計）。
    let starts: Vec<usize> = units
        .iter()
        .scan(0, |at, u| {
            let start = *at;
            *at += u.text.chars().count();
            Some(start)
        })
        .collect();
    let mut done = 0;
    let mut one_by_one = false;
    let mut snapshot = Snapshot::default();
    loop {
        let rest: String = target.chars().skip(done).collect();
        if rest.is_empty() {
            typing.finished = true;
            return typing;
        }
        if snapshot.composition.input.is_empty() {
            // 打出的字停在單位中間時無從重打。
            let Some(next) = starts.iter().position(|&s| s == done) else {
                return typing;
            };
            let end = if one_by_one { next + 1 } else { units.len() };
            for c in units[next..end].iter().flat_map(|u| u.code.chars()) {
                snapshot = press(Key::Letter(c), &mut typing);
            }
            if snapshot.composition.input.is_empty() {
                return typing;
            }
            continue;
        }
        let selected = snapshot.composition.selected_text();
        // 只選接得上的候選，所以固定的分段一定是 `rest` 的開頭。
        let Some(open) = rest.strip_prefix(selected.as_str()) else {
            return typing;
        };
        let connects = |c: &Candidate| !c.text.is_empty() && open.starts_with(&c.text);
        let chosen = |index: usize| snapshot.candidates[index].text.clone();
        let (key, text) = if snapshot.page == 0 && snapshot.candidates.first().is_some_and(connects)
        {
            (Key::Space, Some(chosen(0)))
        } else if let Some(index) = longest_connecting(&snapshot.candidates, connects) {
            typing.correction_count += 1;
            let digit = u8::try_from(index + 1).expect("每頁最多 9 個候選");
            (Key::Digit(digit), Some(chosen(index)))
        } else if snapshot.has_more {
            typing.correction_count += 1;
            (Key::PageDown, None)
        } else if !one_by_one && starts.iter().filter(|&&s| s >= done).count() > 1 {
            typing.correction_count += 1;
            one_by_one = true;
            (Key::Escape, None)
        } else {
            return typing;
        };
        snapshot = press(key, &mut typing);
        let Some(text) = text else {
            continue;
        };
        let expected = format!("{selected}{text}");
        match &snapshot.commit {
            Some(commit) if *commit == expected => done += commit.chars().count(),
            None if snapshot.composition.selected_text() == expected => {}
            // 上屏或固定的不是選的字：核心的行為變了，模擬要跟著改。
            _ => return typing,
        }
    }
}

/// 這頁接得上的最長候選的位置；一樣長取前面的。
fn longest_connecting(
    candidates: &[Candidate],
    connects: impl Fn(&Candidate) -> bool,
) -> Option<usize> {
    candidates
        .iter()
        .enumerate()
        .filter(|(_, c)| connects(c))
        .max_by_key(|(i, c)| (c.text.chars().count(), std::cmp::Reverse(*i)))
        .map(|(i, _)| i)
}
