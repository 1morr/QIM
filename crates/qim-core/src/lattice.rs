//! 連打的 lattice（plan §2.2、§2.3）：位置是輸入碼的字母邊界，邊是碼表的字與詞庫的詞，
//! Viterbi + beam 逐鍵往右補一個位置。做法參考 libime（`docs/research/libime-lattice.md`）。

use std::collections::{BTreeMap, HashSet};

use crate::scheme::{SEPARATOR, without_separators};
use crate::user::SchemeModel;
use crate::{Engine, Segment, UserModel};

/// 一串輸入碼的 lattice：`columns[j]` 是結束在位置 `j` 的部分路徑，依分數由高到低，
/// 最多 beam 條、文字互不相同。`columns[0]` 只有空路徑。
#[derive(Debug, Clone)]
pub(crate) struct Lattice {
    input: String,
    /// 組字前面的一個字（plan §2.3）：開頭的字接它的 bigram；`None` 是用 unigram。
    before: Option<char>,
    columns: Vec<Vec<Node>>,
}

/// 一條部分路徑：從位置 0 到這一欄的字詞接起來。
#[derive(Debug, Clone)]
struct Node {
    score: f64,
    text: String,
    /// 最後一段的起點欄與在那一欄的位置；空路徑是 `None`。
    prev: Option<(usize, usize)>,
}

/// 一個候選與它蓋住的輸入碼長度（從剩下的輸入碼開頭算）。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CandidateSpan {
    pub text: String,
    pub len: usize,
    /// 組成它的各段（碼與字詞），依輸入碼的順序、碼接起來是它蓋住的輸入碼、字詞接起來是
    /// `text`；選定與上屏時用（plan §2.4）。
    pub segments: Vec<Segment>,
    /// 英文候選（排位規則 4）：選了它和 Enter 一樣不學。
    pub english_candidate: bool,
}

impl CandidateSpan {
    /// 只有一段的候選：碼表的字或詞庫的詞，蓋住輸入碼開頭的 `code`。
    fn single(text: String, code: &str) -> Self {
        Self {
            segments: vec![Segment::new(code.to_string(), text.clone())],
            text,
            len: code.len(),
            english_candidate: false,
        }
    }
}

impl Default for Lattice {
    fn default() -> Self {
        Self {
            input: String::new(),
            before: None,
            columns: vec![vec![Node {
                score: 0.0,
                text: String::new(),
                prev: None,
            }]],
        }
    }
}

impl Lattice {
    /// 還沒有輸入碼、接在 `before` 後面的 lattice。
    pub fn starting_after(before: Option<char>) -> Self {
        Self {
            before,
            ..Self::default()
        }
    }

    /// 從頭建一串接在 `before` 後面的輸入碼的 lattice，和逐鍵 [`Lattice::push`] 相同。
    pub fn from_input(
        engine: &Engine,
        user: &UserModel,
        before: Option<char>,
        input: &str,
    ) -> Self {
        let mut lattice = Self::starting_after(before);
        for c in input.chars() {
            lattice.push(engine, user, c);
        }
        lattice
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    /// 多打一個字母：只算新的一欄，前面的欄不動。分隔記號（[`SEPARATOR`]）的一欄照抄前一欄：
    /// 沒有邊結束或開始在它上面，路徑在它這裡斷開時它算在前一段的碼尾。邊只有詞能跨過它，
    /// 而且它要落在詞內兩字的碼之間（`Engine::exact`）。
    pub fn push(&mut self, engine: &Engine, user: &UserModel, c: char) {
        self.input.push(c);
        if c == SEPARATOR {
            let previous = self.columns.last().expect("至少有空路徑的一欄").clone();
            self.columns.push(previous);
            return;
        }
        let edges = engine.edges_ending(user, &self.input);
        let mut extended: Vec<(f64, usize, usize)> = Vec::new();
        for (e, edge) in edges.iter().enumerate() {
            for (k, node) in self.columns[edge.start].iter().enumerate() {
                let prev = node.text.chars().next_back().or(self.before);
                extended.push((node.score + engine.edge_score(prev, edge), e, k));
            }
        }
        // 同分時照邊的順序（起點由前到後、字在詞前）與前一欄的名次，結果才固定。
        extended.sort_by(|a, b| b.0.total_cmp(&a.0));
        let mut column: Vec<Node> = Vec::new();
        for (score, e, k) in extended {
            if column.len() == engine.beam() {
                break;
            }
            let edge = &edges[e];
            let text = format!("{}{}", self.columns[edge.start][k].text, edge.text);
            if column.iter().all(|n| n.text != text) {
                column.push(Node {
                    score,
                    text,
                    prev: Some((edge.start, k)),
                });
            }
        }
        self.columns.push(column);
    }

    /// 刪掉最後一個字母和它的一欄。
    pub fn pop(&mut self) {
        if self.input.pop().is_some() {
            self.columns.pop();
        }
    }

    /// 首選路徑的各段（碼與字詞）；沒有路徑蓋滿整串時是空的。
    pub fn best_segments(&self) -> Vec<Segment> {
        let end = self.input.len();
        if self.columns[end].is_empty() || end == 0 {
            return Vec::new();
        }
        self.path(end, 0)
    }

    /// 結束在位置 `end`、在那一欄第 `k` 名的部分路徑的各段。
    fn path(&self, end: usize, k: usize) -> Vec<Segment> {
        let mut segments = Vec::new();
        let (mut j, mut k) = (end, k);
        while let Some((i, prev)) = self.columns[j][k].prev {
            let text = &self.columns[j][k].text[self.columns[i][prev].text.len()..];
            segments.push(Segment::new(self.input[i..j].to_string(), text.to_string()));
            (j, k) = (i, prev);
        }
        segments.reverse();
        segments
    }
}

/// 候選列（plan §2.3）：
/// 1. 蓋滿整串輸入碼的：碼表的字、詞庫的詞，與分數最高的幾條多段路徑（方案的
///    `ranking.sentences`），依分數排；
/// 2. 整串輸入碼是擴展區的字的碼時，那些字；
/// 3. 只蓋住開頭一段的字詞，蓋得長的在前，同長依分數排。
///
/// 文字重複的只留第一個。`english_candidates` 為真、整串輸入碼是英文單字（[`Engine::english_candidate`]）、
/// 又有其他候選時，最後加上那個英文字。接著依序套用排位規則（[`RULES`]）。
///
/// 分隔記號（[`SEPARATOR`]）：字不跨過它，詞只在它落在兩字的碼之間時跨過它；結尾的分隔記號
/// 算在最後一段裡，所以 `mi'` 的候選是 `mi` 的、蓋住整串。只蓋住開頭一段的字詞後面緊接分隔記號
/// 時，連它一起蓋住，選了之後剩下的輸入碼不以分隔記號開頭。
pub(crate) fn candidates(
    engine: &Engine,
    user: &UserModel,
    lattice: &Lattice,
    english_candidates: bool,
) -> Vec<CandidateSpan> {
    let input = lattice.input();
    let n = input.len();
    if n == 0 {
        return Vec::new();
    }
    // 碼表、詞庫查的碼：結尾的分隔記號去掉；中間的分隔記號由 `Engine::span` 處理。
    let lookup_code = input.trim_end_matches(SEPARATOR);
    let before = lattice.before;
    let mut full: Vec<(f64, CandidateSpan)> = engine
        .span(user, before, lookup_code)
        .into_iter()
        .map(|(score, text)| (score, CandidateSpan::single(text, input)))
        .collect();
    full.extend(
        lattice.columns[n]
            .iter()
            .enumerate()
            .filter(|(_, node)| node.prev.is_some_and(|(i, _)| i > 0))
            .take(engine.sentences())
            .map(|(k, node)| {
                let span = CandidateSpan {
                    text: node.text.clone(),
                    len: n,
                    segments: lattice.path(n, k),
                    english_candidate: false,
                };
                (node.score, span)
            }),
    );
    full.sort_by(|a, b| b.0.total_cmp(&a.0));

    let full = full.into_iter().map(|(_, span)| span);
    let extension = engine
        .extension(lookup_code)
        .map(|c| CandidateSpan::single(c.to_string(), input));
    // 觸發詞是整串的使用者詞，由排位規則 1 移到首選；它的碼多半不是整串，lattice 裡沒有。
    let scheme_user = user.for_scheme(&engine.scheme().id);
    let triggered = scheme_user
        .triggered(lookup_code)
        .into_iter()
        .map(|text| CandidateSpan::single(text.to_string(), input));
    let prefixes = (1..n)
        .rev()
        .filter(|&len| !input[..len].ends_with(SEPARATOR))
        .filter_map(|len| {
            let covered = n - input[len..].trim_start_matches(SEPARATOR).len();
            // 後面只剩分隔記號的是蓋滿整串的，上面已經列過。
            (covered < n).then_some((len, covered))
        })
        .flat_map(|(len, covered)| {
            engine
                .span(user, before, &input[..len])
                .into_iter()
                .map(move |(_, text)| CandidateSpan::single(text, &input[..covered]))
        });
    let mut seen = HashSet::new();
    let mut list: Vec<CandidateSpan> = full
        .chain(triggered)
        .chain(extension)
        .chain(prefixes)
        .filter(|c| seen.insert(c.text.clone()))
        .collect();
    // 沒有中文候選時不出英文：它不搶首選（brief §4.3）。
    if let Some(word) = english_candidates
        .then(|| engine.english_candidate(input))
        .flatten()
        && !list.is_empty()
        && seen.insert(word.to_string())
    {
        list.push(CandidateSpan {
            english_candidate: true,
            ..CandidateSpan::single(word.to_string(), input)
        });
    }
    let mut ranked = Ranked {
        engine,
        user: scheme_user,
        input,
        before,
        lookup_code,
        user_code: without_separators(input),
        free: list,
        pinned: BTreeMap::new(),
    };
    for rule in RULES {
        rule(&mut ranked);
    }
    ranked.into_list()
}

/// 排位規則（plan §2.4），依序套用；前面的規則排定的位置後面的規則不動。
const RULES: [fn(&mut Ranked); 4] = [
    triggered_first,
    chosen_twice_first,
    used_word_second,
    english_second,
];

/// 套用排位規則中的候選列：排定的候選釘在它的位置，其餘的依原本的順序填進空著的位置。
struct Ranked<'a> {
    engine: &'a Engine,
    /// 這個方案的個人模型。
    user: &'a SchemeModel,
    /// 這列候選的輸入碼，含分隔記號。
    input: &'a str,
    /// 輸入碼前面的一個字（[`Lattice::before`]）。
    before: Option<char>,
    /// 查碼表、詞庫與觸發詞的碼：`input` 去掉結尾的分隔記號。
    lookup_code: &'a str,
    /// `input` 去掉全部分隔記號：學習計數記的碼，規則 2、3 比對它（plan §2.4）。
    user_code: String,
    /// 還沒排定的候選，依原本的順序。
    free: Vec<CandidateSpan>,
    /// 已由前面的規則排定的：位置 → 候選。
    pinned: BTreeMap<usize, CandidateSpan>,
}

impl Ranked<'_> {
    /// 還沒排定、蓋滿整串的候選與它在 `free` 的位置。
    fn free_full(&self) -> impl Iterator<Item = (usize, &CandidateSpan)> {
        let n = self.input.len();
        self.free
            .iter()
            .enumerate()
            .filter(move |(_, c)| c.len == n)
    }

    /// 還沒排定、蓋滿整串的候選裡，`tick` 有值且最大（最近）的一個在 `free` 的位置。
    fn most_recent(&self, tick: impl Fn(&SchemeModel, &str, &str) -> Option<u64>) -> Option<usize> {
        self.free_full()
            .filter_map(|(i, c)| Some((tick(self.user, &self.user_code, &c.text)?, i)))
            .max()
            .map(|(_, i)| i)
    }
    /// 把 `free` 的第 `from` 個排定在第 `position` 位或更前面：本來就在那裡就釘在原位，
    /// 否則移到不超過 `position` 的空位裡最後面的一個；那些位置都排定了，就移到最前面的空位。
    fn pin_at_most(&mut self, from: usize, position: usize) {
        let current = self.position_of_free(from);
        let total = self.free.len() + self.pinned.len();
        let open = |p: &usize| !self.pinned.contains_key(p);
        let target = if current <= position {
            current
        } else {
            (0..=position)
                .rev()
                .find(open)
                .or_else(|| (position + 1..total).find(open))
                .expect("free 不是空的，總有空位")
        };
        let candidate = self.free.remove(from);
        self.pinned.insert(target, candidate);
    }

    /// 把 `free` 的第 `from` 個排定在第 `position` 位；那裡已排定時移到它後面最前的空位，不往前移。
    fn pin_at_or_after(&mut self, from: usize, position: usize) {
        let current = self.position_of_free(from);
        // 它現在的位置是空位，所以 `position` 起找得到、不會超過它；排得比 `position` 前時留在原位。
        let target = if current < position {
            current
        } else {
            (position..=current)
                .find(|p| !self.pinned.contains_key(p))
                .expect("它自己的位置是空位")
        };
        let candidate = self.free.remove(from);
        self.pinned.insert(target, candidate);
    }

    /// `free` 的第 `index` 個現在排在第幾位。
    fn position_of_free(&self, index: usize) -> usize {
        (0..)
            .filter(|p| !self.pinned.contains_key(p))
            .nth(index)
            .expect("位置數不限")
    }

    /// 排好的候選列。
    fn into_list(self) -> Vec<CandidateSpan> {
        let total = self.free.len() + self.pinned.len();
        let mut pinned = self.pinned;
        let mut free = self.free.into_iter();
        (0..total)
            .map(|p| {
                pinned
                    .remove(&p)
                    .or_else(|| free.next())
                    .expect("位置數等於候選數")
            })
            .collect()
    }
}

/// 規則 1：觸發詞剛好是整串的使用者詞成首選；多個時取 [`SchemeModel::triggered`] 的第一個。
fn triggered_first(ranked: &mut Ranked) {
    let first = ranked.user.triggered(ranked.lookup_code).into_iter().next();
    let at = first.and_then(|text| ranked.free_full().find(|(_, c)| c.text == text));
    if let Some((i, _)) = at {
        ranked.pin_at_most(i, 0);
    }
}

/// 規則 2：蓋滿整串、上屏次數 ≥ 2 且最近上屏過的候選成首選；多個取最近上屏的。沒有這樣的
/// 候選時，整串上屏剛好 1 次的不靠學到的分數成首選（[`once_chosen_keeps_the_unlearned_top`]）。
fn chosen_twice_first(ranked: &mut Ranked) {
    if let Some(i) = ranked.most_recent(SchemeModel::last_tick_if_chosen_twice) {
        ranked.pin_at_most(i, 0);
    } else {
        once_chosen_keeps_the_unlearned_top(ranked);
    }
}

/// 「選兩次才成首選」（brief §4.5）：學過一次的個人分常勝過靜態機率很低的詞與句子；排第一的
/// 是整串上屏剛好 1 次的候選、而不學習時的首選不是它，就改回不學習時的首選。不學習時的首選
/// 不是蓋滿整串的候選時不動。
fn once_chosen_keeps_the_unlearned_top(ranked: &mut Ranked) {
    let (user, input) = (ranked.user, ranked.input);
    let top_once = ranked.free.first().is_some_and(|c| {
        c.len == input.len() && user.whole_commits(&ranked.user_code, &c.text) == 1
    });
    if !top_once {
        return;
    }
    // 空白的個人模型沒有計數，這次呼叫的排位規則什麼都不做，不會再走到這裡。
    let (engine, blank) = (ranked.engine, UserModel::default());
    let unlearned = candidates(
        engine,
        &blank,
        &Lattice::from_input(engine, &blank, ranked.before, input),
        false,
    );
    let Some(unlearned_top) = unlearned.first() else {
        return;
    };
    let at = ranked
        .free_full()
        .find(|(_, c)| c.text == unlearned_top.text);
    if let Some((i, _)) = at {
        ranked.pin_at_most(i, 0);
    }
}

/// 規則 3：碼是整串、上屏過的使用者詞（含自動造詞）至少排第 2 位；多個取最近上屏的。
fn used_word_second(ranked: &mut Ranked) {
    if let Some(i) = ranked.most_recent(SchemeModel::last_tick_if_used_word) {
        ranked.pin_at_most(i, 1);
    }
}

/// 規則 4：英文候選排第 2 位（brief §4.3）；第 2 位已由前面的規則排定時排在它後面最前的空位。
fn english_second(ranked: &mut Ranked) {
    if let Some(i) = ranked.free.iter().position(|c| c.english_candidate) {
        ranked.pin_at_or_after(i, 1);
    }
}
