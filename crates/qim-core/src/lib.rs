//! QIM 的核心：輸入方案、資料包讀取（碼表、詞庫、字級 bigram 與詞級 trigram、字根提示、簡繁、英文與 emoji）、候選與 `Session`。
//!
//! 純邏輯，不做 I/O；依賴規則見 `docs/plan.md` §1.3。

mod association;
mod code_table;
mod date;
mod emoji;
mod engine;
mod english;
mod language_model;
mod lattice;
mod lexicon;
mod manifest;
mod pack;
mod pack_index;
mod punctuation;
mod radical_hint;
mod remembered;
mod scheme;
mod session;
mod simplifier;
mod user;
mod word_list;
mod word_trigram;

pub use code_table::{CodeEntry, CodeTable};
pub use date::{Date, ParseDateError};
pub use emoji::{EMOJI_LIMIT, EmojiTable};
pub use engine::Engine;
pub use english::EnglishWords;
pub use language_model::{CLAUSE_START, CharModel, LOG_PROB_STEP, LanguageModel};
pub use lexicon::{Lexicon, Word};
pub use manifest::{CORE_VERSION, MANIFEST_FILE, MANIFEST_VERSION, Manifest, PackKind};
pub use pack::{
    EmojiPack, EnglishPack, FORMAT_VERSION, LanguageModelPack, LexiconPack, ModelPack, Pack,
    PackError, RadicalHintPack, SchemePack, SimplifierPack, WordTrigramPack,
};
pub use pack_index::{IndexEntry, PACK_INDEX_FILE, PACK_INDEX_VERSION, PackIndex};
pub use radical_hint::{CangjieCodes, CangjieVersion, RadicalHint, RadicalHints};
pub use remembered::{REMEMBERED_LIMIT, REMEMBERED_STRINGS_MAX};
pub use scheme::{Packs, Ranking, RankingError, SEPARATOR, Scheme};
pub use session::{
    CONTEXT_LIMIT, Candidate, Composition, Key, KeyError, Options, PAGE_SIZE, RERANK_CONTEXT,
    RERANK_PATHS, RerankRequest, ScoredPath, Segment, Session, ShiftKey, ShiftSwitch, Snapshot,
};
pub use simplifier::{ConversionTable, Simplifier};
pub use user::{
    ImportError, ImportWarning, Imported, InvalidDeviceId, Phrase, UserModel, UserWordError,
};
pub use word_list::{
    Shortcuts, SkipReason, SkippedLine, TriggerShortcut, UnusedShortcut, WordListError,
    WordListFormat, WordListReport,
};
pub use word_trigram::{NgramRow, WORD_TRIGRAM_MAX_WORDS, WordTrigram};
