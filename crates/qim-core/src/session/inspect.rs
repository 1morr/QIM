//! 網頁 debug 面板的查詢（plan §3.1，M4 票 01）：核心為什麼這樣排。只給網頁用、不進 [`Snapshot`]，
//! 殼只在面板開著時呼叫，所以每鍵不多花時間；平台殼與 `qim-ffi` 不用。
//!
//! [`Snapshot`]: super::Snapshot

use super::{Session, ShownCandidate};
use crate::lattice::{Preceding, SpanKind};
use crate::{CLAUSE_START, Engine, Segment, UserModel, Weights};

/// [`Session::inspect`] 的結果。
#[derive(Debug, Clone, PartialEq)]
pub struct Inspection {
    /// 核心記著的前文（plan §2.1〈上下文〉）。
    pub context: ContextState,
    /// 沒選定的輸入碼開頭的字接的前一字（字級 bigram，plan §2.3）。
    pub preceding_char: PrecedingChar,
    /// 開頭的一段接的前一個字詞（個人 bigram）；沒有是 `None`。
    pub preceding_word: Option<String>,
    /// 組字的各分段（同 [`crate::Composition::segments`]）：輸入碼怎麼切。
    pub segments: Vec<Segment>,
    /// 全部候選（不分頁），依目前的順序。
    pub candidates: Vec<InspectedCandidate>,
    /// 最近一個鍵是觸控的 [`crate::Key::Weighted`] 時，它的各字母機率（M4 票 06）；其他鍵是 `None`。
    pub touch: Option<Weights>,
}

/// 核心記著的前文是什麼。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextState {
    /// 殼給的前文接上核心送出去的字。
    Known(String),
    /// 不知道前文：殼沒給、也還沒上屏過，或交還宿主的鍵、換欄位讓它作廢。
    Unknown,
    /// 無痕：不讀前文（brief §8）。
    Incognito,
}

/// 一個候選與它的排序原因。
#[derive(Debug, Clone, PartialEq)]
pub struct InspectedCandidate {
    /// 顯示的文字（簡體模式是轉換後的）。
    pub text: String,
    pub kind: CandidateKind,
    /// 字詞的各段（碼、字詞、來源）；英文、emoji、記住的字串與聯想沒有。
    pub segments: Vec<InspectedSegment>,
    /// 路徑分（plan §2.3）：各段接前一段的分數總和，含學習加分；不在 lattice 裡的候選是 `None`。
    pub score: Option<f64>,
    /// 路徑分裡學到的加分（各段 `personal` 的和）；沒有路徑分時是 `None`。
    pub learning: Option<f64>,
    /// 排定它位置的排位規則（1–5，plan §2.4）。
    pub rule: Option<u8>,
    /// 停鍵重排放到這裡的。
    pub rerank: Option<Reranked>,
}

/// 停鍵重排放到第 2 位的候選（[`Session::apply_rerank`]）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reranked {
    /// 重排要求的第幾條路徑（[`crate::RerankRequest::paths`] 的序號）。
    pub path: usize,
    /// 候選列原本沒有、插入的；`false` 是從後面移上來的。
    pub inserted: bool,
}

/// 組字開頭的字接什麼（[`Inspection::preceding_char`]）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrecedingChar {
    /// 句首：前文是空的，或以標點、空白、符號結尾。
    ClauseStart,
    /// 這個字：前文的最後一字或選定的分段的最後一字。
    Char(char),
    /// 不接前一字、用 unigram：不知道前文、無痕、那個字沒有統計，或候選是目前分段單打的碼。
    Unigram,
}

/// 候選的種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateKind {
    /// 碼表的字、詞庫的詞、使用者詞或多段路徑（各段的來源在 [`InspectedSegment::source`]）。
    Word,
    /// 英文候選（排位規則 4）。
    English,
    /// 接在字詞後面的 emoji。
    Emoji,
    /// 記住的字串（排位規則 5）。
    Remembered,
    /// 聯想（上屏之後的候選）。
    Association,
}

/// 字詞候選的一段。
#[derive(Debug, Clone, PartialEq)]
pub struct InspectedSegment {
    /// 輸入碼，含分隔記號。
    pub code: String,
    /// 原文（繁體）。
    pub text: String,
    pub source: SegmentSource,
    /// 有這一段的學習計數。
    pub learned: bool,
    /// 有它接在前一個字詞後面的個人 bigram。
    pub personal_bigram: bool,
}

/// 一段字詞從哪裡來。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentSource {
    /// 碼表的字。
    CodeTable,
    /// 詞庫的詞。
    Lexicon,
    /// 使用者詞：手動加的、匯入的、自動造詞。
    UserWord,
}

impl Session {
    /// 給網頁 debug 面板看的排序原因（plan §3.1）。不改組字；殼只在面板開著時呼叫。
    pub fn inspect(&self, engine: &Engine, user: &UserModel) -> Inspection {
        let context = match (&self.context, self.options.incognito) {
            (_, true) => ContextState::Incognito,
            (Some(context), false) => ContextState::Known(context.clone()),
            (None, false) => ContextState::Unknown,
        };
        // 候選接在什麼後面：目前分段的候選和單打那個碼相同、不接前面（`refresh_candidates`）。
        let before = match self.focus {
            Some(_) => Preceding::default(),
            None => self.lattice.before().clone(),
        };
        let association = self.showing_association();
        let candidates = self
            .candidates
            .iter()
            .map(|c| inspect_candidate(engine, user, &before, c, association))
            .collect();
        Inspection {
            context,
            preceding_char: match before.character {
                Some(CLAUSE_START) => PrecedingChar::ClauseStart,
                Some(c) => PrecedingChar::Char(c),
                None => PrecedingChar::Unigram,
            },
            preceding_word: before.word,
            segments: self.composition(engine).segments,
            candidates,
            touch: self.touch.clone(),
        }
    }
}

fn inspect_candidate(
    engine: &Engine,
    user: &UserModel,
    before: &Preceding,
    candidate: &ShownCandidate,
    association: bool,
) -> InspectedCandidate {
    let span = &candidate.span;
    let kind = match span.kind {
        _ if association => CandidateKind::Association,
        SpanKind::Word => CandidateKind::Word,
        SpanKind::English => CandidateKind::English,
        SpanKind::Emoji => CandidateKind::Emoji,
        SpanKind::Remembered => CandidateKind::Remembered,
    };
    let mut inspected = InspectedCandidate {
        text: candidate.text().to_string(),
        kind,
        segments: Vec::new(),
        score: None,
        learning: None,
        rule: span.rule,
        rerank: candidate.rerank,
    };
    if kind != CandidateKind::Word {
        return inspected;
    }
    let scored = engine.inspect_path(user, before, &span.segments);
    inspected.segments = span
        .segments
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let scored = scored.as_ref().map(|all| all[i]);
            InspectedSegment {
                code: s.code.clone(),
                text: s.text.clone(),
                source: scored.map_or_else(|| engine.source_of(&s.code, &s.text), |x| x.source),
                learned: scored.is_some_and(|x| x.learned),
                personal_bigram: scored.is_some_and(|x| x.personal_bigram),
            }
        })
        .collect();
    if let Some(scored) = scored {
        inspected.score = Some(scored.iter().map(|s| s.score).sum());
        inspected.learning = Some(scored.iter().map(|s| s.learning).sum());
    }
    inspected
}
