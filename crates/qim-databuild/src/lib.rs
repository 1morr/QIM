//! 資料管線：取得來源、推導速成碼表、建資料包並驗證（plan §4）。
//!
//! 入口是 `cargo xtask data [--verify]`，用法見 `docs/development.md`。

mod error;
pub mod libcangjie;
mod pipeline;
pub mod quick;
pub mod scheme;
pub mod sources;
pub mod verify;

pub use error::Error;
pub use pipeline::{Outcome, run};

use qim_core::{Pack, Scheme};

/// 由 libcangjie 的列與方案建速成資料包。
pub fn build_pack(rows: &[libcangjie::Row], scheme: Scheme) -> Pack {
    Pack {
        scheme,
        code_table: quick::code_table(rows),
    }
}
