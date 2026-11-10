//! QIM 的核心：輸入方案、資料包讀取（碼表、詞庫、語言模型、字根提示、簡繁與英文）、候選與 `Session`。
//!
//! 純邏輯，不做 I/O；依賴規則見 `docs/plan.md` §1.3。

mod code_table;
mod engine;
mod english;
mod language_model;
mod lattice;
mod lexicon;
mod pack;
mod punctuation;
mod radical_hint;
mod scheme;
mod session;
mod simplifier;
mod user;

pub use code_table::{CodeEntry, CodeTable};
pub use engine::Engine;
pub use english::EnglishWords;
pub use language_model::{CLAUSE_START, CharModel, LOG_PROB_STEP, LanguageModel};
pub use lexicon::{Lexicon, Word};
pub use pack::{
    EnglishPack, FORMAT_VERSION, LanguageModelPack, LexiconPack, Pack, PackError, RadicalHintPack,
    SchemePack, SimplifierPack,
};
pub use radical_hint::{CangjieCodes, CangjieVersion, RadicalHint, RadicalHints};
pub use scheme::{Packs, Ranking, RankingError, SEPARATOR, Scheme};
pub use session::{
    Candidate, Composition, Key, KeyError, Options, PAGE_SIZE, RERANK_LIMIT, RerankRequest,
    Segment, Session, ShiftKey, ShiftSwitch, Snapshot,
};
pub use simplifier::{ConversionTable, Simplifier};
pub use user::{ImportError, Phrase, UserModel, UserWordError};
