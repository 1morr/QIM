//! QIM 的核心：輸入方案、資料包讀取（碼表、詞庫、語言模型與字根提示）、候選與 `Session`。
//!
//! 純邏輯，不做 I/O；依賴規則見 `docs/plan.md` §1.3。

mod code_table;
mod engine;
mod language_model;
mod lattice;
mod lexicon;
mod pack;
mod punctuation;
mod radical_hint;
mod scheme;
mod session;
mod user;

pub use code_table::{CodeEntry, CodeTable};
pub use engine::Engine;
pub use language_model::{CharModel, LOG_PROB_STEP, LanguageModel};
pub use lexicon::{Lexicon, Word};
pub use pack::{FORMAT_VERSION, LanguageModelPack, LexiconPack, Pack, PackError, RadicalHintPack};
pub use radical_hint::{CangjieCodes, CangjieVersion, RadicalHint, RadicalHints};
pub use scheme::{Packs, Ranking, Scheme};
pub use session::{Candidate, Composition, Key, Options, PAGE_SIZE, Segment, Session, Snapshot};
pub use user::{ImportError, UserModel, UserWordError};
