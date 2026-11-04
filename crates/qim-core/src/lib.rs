//! QIM 的核心：輸入方案、資料包讀取（碼表與詞庫）、候選與 `Session`。
//!
//! 純邏輯，不做 I/O；依賴規則見 `docs/plan.md` §1.3。

mod code_table;
mod engine;
mod lattice;
mod lexicon;
mod pack;
mod punctuation;
mod scheme;
mod session;
mod user;

pub use code_table::{CodeEntry, CodeTable};
pub use engine::Engine;
pub use lexicon::{Lexicon, Word};
pub use pack::{FORMAT_VERSION, LexiconPack, Pack, PackError};
pub use scheme::{Packs, Ranking, Scheme};
pub use session::{Candidate, Composition, Key, Options, PAGE_SIZE, Segment, Session, Snapshot};
pub use user::UserModel;
