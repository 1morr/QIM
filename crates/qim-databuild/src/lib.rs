//! 資料管線：取得來源、推導速成碼表、建速成、詞庫、語言模型、字根提示、簡繁與英文資料包並驗證（plan §4）。
//!
//! 入口是 `cargo xtask data [--verify]`，用法見 `docs/development.md`。

pub mod bigram;
pub mod common_forms;
pub mod corpora;
pub mod corpus;
pub mod english;
mod error;
pub mod gboard;
pub mod hkcancor;
pub mod lexicon;
pub mod libcangjie;
pub mod mediawiki;
pub mod opencc;
mod packing;
pub mod personal;
mod pipeline;
pub mod quick;
pub mod radical_hint;
pub mod scheme;
pub mod sources;
pub mod transcode;
pub mod unihan;
pub mod verify;
pub mod wiki;
pub mod wikitext;

pub use error::Error;
pub use pipeline::{Outcome, run};

use std::collections::BTreeMap;

use qim_core::{Scheme, SchemePack};

/// 由 libcangjie 的列、Unihan `kGradeLevel` 與方案建速成資料包。
pub fn build_scheme_pack(
    rows: &[libcangjie::Row],
    grades: &BTreeMap<char, u8>,
    scheme: Scheme,
) -> SchemePack {
    SchemePack {
        scheme,
        code_table: quick::code_table(rows, grades),
    }
}
