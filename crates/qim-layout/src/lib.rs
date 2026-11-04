//! 佈局：畫布模型的資料格式 `qim-layout/1`、JSON Schema 與兩級驗證、點擊判定（plan §2.6、ADR 0004）。
//!
//! 純邏輯，不做 I/O；依賴規則見 `docs/plan.md` §1.3。

mod geometry;
mod model;
mod validate;

pub use model::{
    Action, Composing, Device, FORMAT, Frame, Grid, Insets, Key, Layout, LayoutVariant,
    Orientation, Page, Shape, Size, Style, Swipe, ThemeColor, When,
};
pub use validate::{
    LayoutError, Report, SCHEMA, StrictIssue, Target, Warning, check_builtin, validate,
};

/// 讀不了的佈局檔：[`validate`] 有錯誤時，把錯誤接成一句。
#[derive(Debug)]
pub struct Error(pub(crate) String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl Layout {
    /// 讀佈局檔；[`validate`]（[`Target::default`]）有錯誤時失敗，警告不管。
    pub fn from_json(text: &str) -> Result<Self, Error> {
        validate(text, Target::default())
            .into_layout()
            .map(|(layout, _)| layout)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("佈局只有字串、數字與列舉，一定序列化得了")
    }
}
