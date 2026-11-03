use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    /// 快取或下載的檔案與 `data/sources.toml` 釘的 sha256 不符。
    ChecksumMismatch {
        path: PathBuf,
        expected: String,
        actual: String,
    },
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Fetch {
        url: String,
        reason: String,
    },
    /// 輸入檔的內容不合格式；`what` 是檔案或位置。
    Invalid {
        what: String,
        reason: String,
    },
}

impl Error {
    pub(crate) fn invalid(what: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::Invalid {
            what: what.into(),
            reason: reason.into(),
        }
    }

    pub(crate) fn io(path: impl Into<PathBuf>) -> impl FnOnce(std::io::Error) -> Self {
        let path = path.into();
        move |source| Self::Io { path, source }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChecksumMismatch {
                path,
                expected,
                actual,
            } => write!(
                f,
                "{}：sha256 不符，應為 {expected}，實為 {actual}；確認來源後刪掉這個檔重新下載，或更新 data/sources.toml",
                path.display()
            ),
            Self::Io { path, source } => write!(f, "{}：{source}", path.display()),
            Self::Fetch { url, reason } => write!(f, "下載 {url} 失敗：{reason}"),
            Self::Invalid { what, reason } => write!(f, "{what}：{reason}"),
        }
    }
}

impl std::error::Error for Error {}
