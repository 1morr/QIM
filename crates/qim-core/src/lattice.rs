//! 連打的 lattice（plan §2.2、§2.3）：位置是輸入碼的字母邊界，邊是碼表的字與詞庫的詞，
//! Viterbi + beam 逐鍵往右補一個位置。做法參考 libime（`docs/research/libime-lattice.md`）。

use std::collections::{BTreeMap, HashSet};

use crate::engine::Spanned;
use crate::remembered::REMEMBERED_LIMIT;
use crate::scheme::{SEPARATOR, without_separators};
use crate::user::{Following, SchemeModel};
use crate::{EMOJI_LIMIT, Engine, Segment, UserModel};

/// 一串輸入碼的 lattice：`columns[j]` 是結束在位置 `j` 的部分路徑，依分數由高到低，
/// 最多 beam 條、文字互不相同。`columns[0]` 只有空路徑。
#[derive(Debug, Clone)]
pub(crate) struct Lattice {
    input: String,
    /// 組字前面是什麼：開頭的字接它。
    before: Preceding,
    columns: Vec<Vec<Node>>,
}

/// 組字（或選定的分段之後剩下的輸入碼）前面是什麼（plan §2.3）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Preceding {
    /// 前一個字：開頭的字接它的字級 bigram；`None` 是用 unigram。
    pub character: Option<char>,
    /// 前一個字詞：開頭的一段接它的個人 bigram；`None` 是沒有。
    pub word: Option<String>,
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
    pub kind: SpanKind,
}

/// 候選從哪裡來。英文、emoji 與記住的字串不在 lattice 裡、只給剩下的整串輸入碼，選了原樣上屏、不學。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpanKind {
    /// 碼表的字、詞庫的詞、使用者詞或多段路徑。
    Word,
    /// 英文候選（排位規則 4）。
    English,
    /// 接在字詞後面的 emoji（plan §2.4）。
    Emoji,
    /// 記住的字串（排位規則 5，plan §2.4）。
    Remembered,
}

impl SpanKind {
    /// 選了和 Enter 一樣原樣上屏、整次不學也不造詞。
    pub(crate) fn is_literal(self) -> bool {
        self != Self::Word
    }
}

impl CandidateSpan {
    /// 只有一段的候選：碼表的字或詞庫的詞，蓋住輸入碼開頭的 `code`。
    fn single(text: String, code: &str) -> Self {
        Self {
            segments: vec![Segment::new(code.to_string(), text.clone())],
            text,
            len: code.len(),
            kind: SpanKind::Word,
        }
    }
}

/// 列候選時看的選項，由 [`crate::Options`] 算出。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Listing {
    /// 整串輸入碼是英文單字時加英文候選（排位規則 4）；目前分段的候選列沒有。
    pub english_candidates: bool,
    /// 整串的候選是 emoji 資料包裡的詞時接上它的 emoji（M3 票 16）；目前分段的候選列沒有。
    pub emoji_candidates: bool,
    /// 整串輸入碼是記住的字串的開頭時加上它（排位規則 5，M3 票 15）；目前分段的候選列沒有。
    pub remembered_strings: bool,
    /// 碼表標成簡體字的字排到同一段的其他字詞之後（brief §4.1）：繁體模式、
    /// [`crate::Options::simplified_after_traditional`] 開著時。
    pub simplified_last: bool,
}

impl Listing {
    /// 這個字詞要不要排到同一段的其他字詞之後。
    fn demotes(self, spanned: &Spanned) -> bool {
        self.simplified_last && spanned.simplified
    }
}

impl Default for Lattice {
    fn default() -> Self {
        Self {
            input: String::new(),
            before: Preceding::default(),
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
    pub fn starting_after(before: Preceding) -> Self {
        Self {
            before,
            ..Self::default()
        }
    }

    /// 從頭建一串接在 `before` 後面的輸入碼的 lattice，和逐鍵 [`Lattice::push`] 相同。
    pub fn from_input(engine: &Engine, user: &UserModel, before: Preceding, input: &str) -> Self {
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
        // 起點欄 → 那一欄每條部分路徑的最後一段後面的個人 bigram；每條部分路徑只查一次。
        // 沒學過個人 bigram 時不查（每鍵都走這裡）。
        let has_bigrams = user.has_bigrams();
        let mut following: BTreeMap<usize, Vec<Option<Following>>> = BTreeMap::new();
        let mut extended: Vec<(f64, usize, usize)> = Vec::new();
        for (e, edge) in edges.iter().enumerate() {
            let after = has_bigrams.then(|| {
                &*following.entry(edge.start).or_insert_with(|| {
                    (0..self.columns[edge.start].len())
                        .map(|k| {
                            self.last_word(edge.start, k)
                                .and_then(|word| user.bigrams_after(word))
                        })
                        .collect()
                })
            });
            for (k, node) in self.columns[edge.start].iter().enumerate() {
                let prev = node.text.chars().next_back().or(self.before.character);
                let bigram = after
                    .and_then(|after| after[k])
                    .map_or(0.0, |f| f.probability(&edge.text));
                extended.push((node.score + engine.edge_score(prev, edge, bigram), e, k));
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

    /// 結束在第 `j` 欄、在那一欄第 `k` 名的部分路徑的最後一段的字詞；空路徑是組字前面的字詞。
    fn last_word(&self, j: usize, k: usize) -> Option<&str> {
        let node = &self.columns[j][k];
        match node.prev {
            Some((i, prev)) => Some(&node.text[self.columns[i][prev].text.len()..]),
            None => self.before.word.as_deref(),
        }
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
///    `ranking.sentences`），依分數排；簡體字排後（[`Listing::simplified_last`]）時簡體字排在它們之後；
/// 2. 整串輸入碼是擴展區的字的碼時，那些字；
/// 3. 只蓋住開頭一段的字詞，蓋得長的在前，同長依分數排，簡體字排後時簡體字排在同長的最後。
///
/// 文字重複的只留第一個。[`Listing::english_candidates`] 為真、整串輸入碼是英文單字
/// （[`Engine::english_candidate`]）、又有其他候選時，最後加上那個英文字。接著依序套用排位規則
/// （[`RULES`]），所以選兩次的簡體字照樣成首選。
///
/// 分隔記號（[`SEPARATOR`]）：字不跨過它，詞只在它落在兩字的碼之間時跨過它；結尾的分隔記號
/// 算在最後一段裡，所以 `mi'` 的候選是 `mi` 的、蓋住整串。只蓋住開頭一段的字詞後面緊接分隔記號
/// 時，連它一起蓋住，選了之後剩下的輸入碼不以分隔記號開頭。
pub(crate) fn candidates(
    engine: &Engine,
    user: &UserModel,
    lattice: &Lattice,
    listing: Listing,
) -> Vec<CandidateSpan> {
    let input = lattice.input();
    let n = input.len();
    if n == 0 {
        return Vec::new();
    }
    // 碼表、詞庫查的碼：結尾的分隔記號去掉；中間的分隔記號由 `Engine::span` 處理。
    let lookup_code = input.trim_end_matches(SEPARATOR);
    let before = &lattice.before;
    let mut full: Vec<(f64, bool, CandidateSpan)> = engine
        .span(user, before, lookup_code)
        .into_iter()
        .map(|s| {
            let late = listing.demotes(&s);
            (s.score, late, CandidateSpan::single(s.text, input))
        })
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
                    kind: SpanKind::Word,
                };
                (node.score, false, span)
            }),
    );
    full.sort_by(|a, b| a.1.cmp(&b.1).then(b.0.total_cmp(&a.0)));

    let full = full.into_iter().map(|(_, _, span)| span);
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
            let mut spans = engine.span(user, before, &input[..len]);
            // 穩定排序：同一組裡仍依分數。
            spans.sort_by_key(|s| listing.demotes(s));
            spans
                .into_iter()
                .map(move |s| CandidateSpan::single(s.text, &input[..covered]))
        });
    let mut seen = HashSet::new();
    let mut list: Vec<CandidateSpan> = full
        .chain(triggered)
        .chain(extension)
        .chain(prefixes)
        .filter(|c| seen.insert(c.text.clone()))
        .collect();
    if listing.emoji_candidates {
        list = with_emoji(engine, list, input, &mut seen);
    }
    // 沒有中文候選時不出英文：它不搶首選（brief §4.3）。
    if let Some(word) = listing
        .english_candidates
        .then(|| engine.english_candidate(input))
        .flatten()
        && !list.is_empty()
        && seen.insert(word.to_string())
    {
        list.push(CandidateSpan {
            kind: SpanKind::English,
            ..CandidateSpan::single(word.to_string(), input)
        });
    }
    if listing.remembered_strings && !list.is_empty() {
        list.extend(
            remembered(engine, user, input)
                .filter(|text| seen.insert(text.to_string()))
                .take(REMEMBERED_LIMIT)
                .map(|text| CandidateSpan {
                    kind: SpanKind::Remembered,
                    ..CandidateSpan::single(text.to_string(), input)
                }),
        );
    }
    let mut ranked = Ranked {
        engine,
        listing,
        user: scheme_user,
        following: before.word.as_deref().and_then(|w| user.bigrams_after(w)),
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

/// 輸入碼 `input`（去掉分隔記號）是方案打得出的字母開頭（[`crate::remembered::letters`]）的記住的字串，最近常打的在前；
/// 輸入碼要長過方案一個字的最長碼（同英文候選，速成 3 碼起），1–2 碼是單字的碼，第 2 位最常用。
fn remembered<'a>(
    engine: &Engine,
    user: &'a UserModel,
    input: &str,
) -> impl Iterator<Item = &'a str> {
    let typed = without_separators(input);
    let long_enough = typed.len() > usize::from(engine.scheme().max_code_length);
    long_enough
        .then(|| user.strings_starting_with(&typed, &engine.scheme().alphabet))
        .into_iter()
        .flatten()
}

/// 蓋滿整串的字詞後面接上它的 emoji（rime-emoji 的做法，plan §2.4），依候選列的順序、整列最多
/// [`EMOJI_LIMIT`] 個，和前面的候選文字重複的不接。查的是繁體原文，簡體模式也一樣。在排位規則
/// 之前接上，所以規則排定的位置（英文候選的第 2 位）照舊；emoji 不在 lattice、沒有學習計數，
/// 規則不會移動它。
fn with_emoji(
    engine: &Engine,
    list: Vec<CandidateSpan>,
    input: &str,
    seen: &mut HashSet<String>,
) -> Vec<CandidateSpan> {
    let mut left = EMOJI_LIMIT;
    let mut out = Vec::with_capacity(list.len());
    for span in list {
        let emoji: Vec<String> = if left > 0 && span.len == input.len() {
            engine
                .emoji(&span.text)
                .iter()
                .filter(|e| seen.insert(e.to_string()))
                .take(left)
                .cloned()
                .collect()
        } else {
            Vec::new()
        };
        left -= emoji.len();
        out.push(span);
        out.extend(emoji.into_iter().map(|e| CandidateSpan {
            kind: SpanKind::Emoji,
            ..CandidateSpan::single(e, input)
        }));
    }
    out
}

/// 排位規則（plan §2.4），依序套用；前面的規則排定的位置後面的規則不動。
const RULES: [fn(&mut Ranked); 5] = [
    triggered_first,
    chosen_twice_first,
    used_word_second,
    english_second,
    remembered_after_english,
];

/// 套用排位規則中的候選列：排定的候選釘在它的位置，其餘的依原本的順序填進空著的位置。
struct Ranked<'a> {
    engine: &'a Engine,
    /// 這列候選的選項；規則 2 比對不學習時的首選要用同樣的。
    listing: Listing,
    /// 這個方案的個人模型。
    user: &'a SchemeModel,
    /// 輸入碼前面的字詞後面的個人 bigram；規則 2 看單打的候選有沒有學到的分。
    following: Option<Following<'a>>,
    /// 這列候選的輸入碼，含分隔記號。
    input: &'a str,
    /// 輸入碼前面是什麼（[`Lattice::before`]）。
    before: &'a Preceding,
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
/// 候選整串上屏不到 2 次、靠學到的分排第一，而不學習時的首選不是它，就改回不學習時的首選。
/// 靠學到的分：整串上屏剛好 1 次；或 0 次但它只有一段（就是單打這串碼）、這段有學習計數或接在
/// 前一個字詞後的個人 bigram，也就是只在句中用過（句中用過的只在連打裡加分，M3 票 12）。多段的
/// 候選是連打，學過的段照樣加分。不學習時的首選不是蓋滿整串的候選時不動。首選已由規則 1 排定時，
/// 排第一的不是它，也不動。
fn once_chosen_keeps_the_unlearned_top(ranked: &mut Ranked) {
    if ranked.pinned.contains_key(&0) {
        return;
    }
    let (user, input, code) = (ranked.user, ranked.input, &ranked.user_code);
    let learned_alone = |c: &CandidateSpan| {
        c.segments.len() == 1
            && (user.has_learned(code, &c.text)
                || ranked
                    .following
                    .is_some_and(|f| f.probability(&c.text) > 0.0))
    };
    let top_learned = ranked.free.first().is_some_and(|c| {
        c.len == input.len()
            && match user.whole_commits(code, &c.text) {
                0 => learned_alone(c),
                1 => true,
                _ => false,
            }
    });
    if !top_learned {
        return;
    }
    // 空白的個人模型沒有計數，這次呼叫的排位規則什麼都不做，不會再走到這裡。
    let (engine, blank) = (ranked.engine, UserModel::default());
    let unlearned_top = candidates(
        engine,
        &blank,
        &Lattice::from_input(engine, &blank, ranked.before.clone(), input),
        Listing {
            english_candidates: false,
            emoji_candidates: false,
            remembered_strings: false,
            ..ranked.listing
        },
    )
    .into_iter()
    .next();
    let Some(unlearned_top) = unlearned_top else {
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
    if let Some(i) = ranked.free.iter().position(|c| c.kind == SpanKind::English) {
        ranked.pin_at_or_after(i, 1);
    }
}

/// 規則 5：記住的字串依序排在第 2 位起的空位（M3 票 15）：英文候選在第 2 位時排在它後面，不搶首選。
fn remembered_after_english(ranked: &mut Ranked) {
    while let Some(i) = ranked
        .free
        .iter()
        .position(|c| c.kind == SpanKind::Remembered)
    {
        ranked.pin_at_or_after(i, 1);
    }
}
