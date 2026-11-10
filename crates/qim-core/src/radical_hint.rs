use std::collections::BTreeMap;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{PackError, RadicalHintPack, Scheme};

/// 字根提示用哪一代倉頡碼；預設五代，可改三代（brief §4.2）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CangjieVersion {
    Cangjie3,
    #[default]
    Cangjie5,
}

impl FromStr for CangjieVersion {
    type Err = String;

    /// 殼的設定值：`cangjie5` 或 `cangjie3`。
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cangjie5" => Ok(Self::Cangjie5),
            "cangjie3" => Ok(Self::Cangjie3),
            other => Err(format!("不認得的倉頡版本：{other}（cangjie5 或 cangjie3）")),
        }
    }
}

/// 一個字的倉頡碼，三代與五代各可能有幾個（libcangjie 的順序）。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CangjieCodes {
    pub cangjie3: Vec<String>,
    pub cangjie5: Vec<String>,
}

/// 字根提示：字 → 倉頡碼。只給殼顯示「這個字怎麼打」，不進 lattice（ADR 0003）：
/// [`crate::Engine`] 不持有它，倉頡碼也就不會變成速成的輸入碼。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RadicalHints {
    chars: BTreeMap<char, CangjieCodes>,
}

/// 候選裡一個字的字根提示：它在選定那一代的倉頡碼與每個碼的字根，沒有碼就都是空的。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadicalHint {
    pub text: char,
    pub codes: Vec<String>,
    /// 和 `codes` 一一對應，由方案的 `[radicals]` 換成字根（[`Scheme::radicals_of`]）。
    pub radicals: Vec<String>,
}

impl RadicalHints {
    pub fn new(chars: impl IntoIterator<Item = (char, CangjieCodes)>) -> Self {
        Self {
            chars: chars.into_iter().collect(),
        }
    }

    /// 取出字根提示資料包的提示；方案要指名字根提示（[`crate::Packs::radical_hint`]），指名的是預設
    /// 的那份，殼可以換成另一份（plan §4.6）。
    pub fn for_scheme(pack: RadicalHintPack, scheme: &Scheme) -> Result<Self, PackError> {
        let expected = scheme.packs.radical_hint.clone();
        if expected.is_none() {
            return Err(PackError::RadicalHintMismatch {
                expected,
                found: Some(pack.id),
            });
        }
        Ok(pack.hints)
    }

    /// 字 `c` 在 `version` 的倉頡碼；沒有時是空的。
    pub fn codes(&self, c: char, version: CangjieVersion) -> &[String] {
        self.chars.get(&c).map_or(&[], |codes| match version {
            CangjieVersion::Cangjie3 => &codes.cangjie3,
            CangjieVersion::Cangjie5 => &codes.cangjie5,
        })
    }

    /// 候選文字 `text` 每個字的提示，依字的順序；詞也逐字給。字根用 `scheme` 的 `[radicals]`。
    pub fn hint(&self, scheme: &Scheme, text: &str, version: CangjieVersion) -> Vec<RadicalHint> {
        text.chars()
            .map(|c| {
                let codes = self.codes(c, version).to_vec();
                RadicalHint {
                    text: c,
                    radicals: codes.iter().map(|code| scheme.radicals_of(code)).collect(),
                    codes,
                }
            })
            .collect()
    }

    /// 依碼位列出每個字與它的碼。
    pub fn iter(&self) -> impl Iterator<Item = (char, &CangjieCodes)> {
        self.chars.iter().map(|(&c, codes)| (c, codes))
    }
}
