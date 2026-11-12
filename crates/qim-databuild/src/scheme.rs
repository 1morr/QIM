//! 讀方案檔（`data/schemes/*.toml`，plan §2.5）並轉成 `qim_core::Scheme`。

use std::collections::{BTreeMap, BTreeSet};

use qim_core::{Packs, Ranking, Scheme};
use serde::Deserialize;

use crate::Error;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SchemeFile {
    id: String,
    name: String,
    version: String,
    alphabet: String,
    max_code_length: u8,
    /// 萬用鍵（plan §2.5）：選填，一個不在 alphabet 的 ASCII 小寫字母。
    wildcard: Option<String>,
    radicals: BTreeMap<String, String>,
    #[serde(default)]
    packs: PacksTable,
    /// `[ranking]`：連打的打分參數（plan §2.3），每個方案自己調，所以必填。
    ranking: Ranking,
}

/// 方案檔的 `[packs]`：另外的資料包的 id。
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct PacksTable {
    lexicon: Option<String>,
    language_model: Option<String>,
    word_trigram: Option<String>,
    radical_hint: Option<String>,
}

/// 字母表是 ASCII 字母、每個鍵剛好一個字根，字根表沒有字母表與萬用鍵以外的鍵；萬用鍵是一個
/// 不在字母表的 ASCII 小寫字母（分隔記號 `'` 不是字母，自然排除），可以沒有字根。
pub fn parse(text: &str) -> Result<Scheme, Error> {
    let file: SchemeFile =
        toml::from_str(text).map_err(|e| Error::invalid("方案檔", e.to_string()))?;
    let invalid = |reason: String| Error::invalid(format!("方案檔 {}", file.id), reason);

    if !file.alphabet.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(invalid(format!(
            "alphabet 只能有 ASCII 字母：{}",
            file.alphabet
        )));
    }
    file.ranking.check().map_err(|e| invalid(e.to_string()))?;
    let keys: BTreeSet<char> = file.alphabet.chars().collect();
    if keys.len() != file.alphabet.chars().count() {
        return Err(invalid(format!("alphabet 有重複的鍵：{}", file.alphabet)));
    }
    if file.max_code_length == 0 {
        return Err(invalid("max_code_length 至少是 1".into()));
    }
    let wildcard = match file.wildcard.as_deref() {
        None => None,
        Some(w) => {
            let mut chars = w.chars();
            match (chars.next(), chars.next()) {
                (Some(c), None) if c.is_ascii_lowercase() && !keys.contains(&c) => Some(c),
                _ => {
                    return Err(invalid(format!(
                        "wildcard 要是一個不在 alphabet 的 ASCII 小寫字母：{w:?}"
                    )));
                }
            }
        }
    };
    let mut radicals = BTreeMap::new();
    for (key, radical) in &file.radicals {
        let mut chars = key.chars();
        match (chars.next(), chars.next()) {
            (Some(k), None) if keys.contains(&k) || wildcard == Some(k) => {
                radicals.insert(k, radical.clone())
            }
            _ => {
                return Err(invalid(format!(
                    "radicals 的鍵 {key:?} 不在 alphabet 裡，也不是萬用鍵"
                )));
            }
        };
    }
    if let Some(k) = keys.iter().find(|k| !radicals.contains_key(k)) {
        return Err(invalid(format!("鍵 {k} 沒有字根")));
    }

    Ok(Scheme {
        id: file.id,
        name: file.name,
        version: file.version,
        alphabet: file.alphabet,
        max_code_length: file.max_code_length,
        wildcard,
        radicals,
        packs: Packs {
            lexicon: file.packs.lexicon,
            language_model: file.packs.language_model,
            word_trigram: file.packs.word_trigram,
            radical_hint: file.packs.radical_hint,
        },
        ranking: file.ranking,
    })
}
