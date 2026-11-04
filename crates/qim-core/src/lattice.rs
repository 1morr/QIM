//! 連打的 lattice（plan §2.2、§2.3）：位置是輸入碼的字母邊界，邊是碼表的字與詞庫的詞，
//! Viterbi + beam 逐鍵往右補一個位置。做法參考 libime（`docs/research/libime-lattice.md`）。

use std::collections::HashSet;

use crate::Engine;

/// 一串輸入碼的 lattice：`columns[j]` 是結束在位置 `j` 的部分路徑，依分數由高到低，
/// 最多 beam 條、文字互不相同。`columns[0]` 只有空路徑。
#[derive(Debug, Clone)]
pub(crate) struct Lattice {
    input: String,
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
}

/// 首選路徑的一段：輸入碼 `[start, end)` 與它的字詞。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PathSegment {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

impl Default for Lattice {
    fn default() -> Self {
        Self {
            input: String::new(),
            columns: vec![vec![Node {
                score: 0.0,
                text: String::new(),
                prev: None,
            }]],
        }
    }
}

impl Lattice {
    /// 從頭建一串輸入碼的 lattice，和逐鍵 [`Lattice::push`] 相同。
    pub fn from_input(engine: &Engine, input: &str) -> Self {
        let mut lattice = Self::default();
        for c in input.chars() {
            lattice.push(engine, c);
        }
        lattice
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    /// 多打一個字母：只算新的一欄，前面的欄不動。
    pub fn push(&mut self, engine: &Engine, c: char) {
        self.input.push(c);
        let edges = engine.edges_ending(&self.input);
        let mut extended: Vec<(f64, usize, usize)> = Vec::new();
        for (e, edge) in edges.iter().enumerate() {
            for (k, node) in self.columns[edge.start].iter().enumerate() {
                let prev = node.text.chars().next_back();
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

    /// 首選路徑的各段；沒有路徑蓋滿整串時是空的。
    pub fn best_segments(&self) -> Vec<PathSegment> {
        let end = self.input.len();
        if self.columns[end].is_empty() || end == 0 {
            return Vec::new();
        }
        let mut segments = Vec::new();
        let (mut j, mut k) = (end, 0);
        while let Some((i, prev)) = self.columns[j][k].prev {
            let text = &self.columns[j][k].text[self.columns[i][prev].text.len()..];
            segments.push(PathSegment {
                start: i,
                end: j,
                text: text.to_string(),
            });
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
/// 文字重複的只留第一個。
pub(crate) fn candidates(engine: &Engine, lattice: &Lattice) -> Vec<CandidateSpan> {
    let input = lattice.input();
    let n = input.len();
    if n == 0 {
        return Vec::new();
    }
    let mut full = engine.span(input);
    full.extend(
        lattice.columns[n]
            .iter()
            .filter(|node| node.prev.is_some_and(|(i, _)| i > 0))
            .take(engine.sentences())
            .map(|node| (node.score, node.text.clone())),
    );
    full.sort_by(|a, b| b.0.total_cmp(&a.0));

    let full = full
        .into_iter()
        .map(|(_, text)| CandidateSpan { text, len: n });
    let extension = engine.extension(input).map(|c| CandidateSpan {
        text: c.to_string(),
        len: n,
    });
    let prefixes = (1..n).rev().flat_map(|len| {
        engine
            .span(&input[..len])
            .into_iter()
            .map(move |(_, text)| CandidateSpan { text, len })
    });
    let mut seen = HashSet::new();
    full.chain(extension)
        .chain(prefixes)
        .filter(|c| seen.insert(c.text.clone()))
        .collect()
}
