//! OpenCC 字典 → 簡繁資料包（plan〈英文、簡繁與 emoji〉）：照 OpenCC `data/config/hk2s.json` 組出香港繁體轉簡體的
//! 轉換表。字典格式與反轉規則照 OpenCC `data/scripts/common.py`；釘的版本在 `data/sources.toml`。

use std::collections::{BTreeMap, HashSet};

use qim_core::{ConversionTable, Simplifier};

use crate::Error;

/// CJK 相容表意文字 → 統一表意文字（正規化）。
pub const COMPATIBILITY: &str = "data/dictionary/CJK_Compatibility_Ideographs.txt";
/// 繁體詞 → 簡體詞，也是分段的詞表。
pub const TS_PHRASES: &str = "data/dictionary/TSPhrases.txt";
/// 繁體字 → 簡體字。
pub const TS_CHARACTERS: &str = "data/dictionary/TSCharacters.txt";
/// 簡體字 → 繁體字；不進簡繁資料包，只給速成碼表判定簡體字（`crate::simplified`）。
pub const ST_CHARACTERS: &str = "data/dictionary/STCharacters.txt";
/// 標準繁體字 → 香港異體字；反轉成 HKVariantsRev。
pub const HK_VARIANTS: &str = "data/dictionary/HKVariants.txt";
/// 香港異體詞 → 標準繁體詞。
pub const HK_VARIANTS_REV_PHRASES: &str = "data/dictionary/HKVariantsRevPhrases.txt";

/// 反轉時指定某個值的首選鍵（`# @reverse-prefer: 值 [鍵]`，只寫一個時鍵就是值本身）。
const REVERSE_PREFERENCE: &str = "# @reverse-prefer:";

/// 一份 OpenCC 字典：依檔案順序的（鍵, 值），一個鍵可以有幾個值，第一個是預設。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dictionary {
    pub entries: Vec<(String, Vec<String>)>,
    /// 反轉後值 → 首選鍵。
    pub reverse_preferences: BTreeMap<String, String>,
}

/// 讀一份字典：`#` 開頭的列與空列略過，其餘每列「鍵<TAB>值（空格分隔）」。
/// 鍵重複、少了 TAB 或值是空的都當格式錯誤。
pub fn parse(file: &str, text: &str) -> Result<Dictionary, Error> {
    let mut entries = Vec::new();
    let mut reverse_preferences = BTreeMap::new();
    let mut keys = HashSet::new();
    for (i, line) in text.lines().enumerate() {
        let at = || format!("{file} 第 {} 列", i + 1);
        if let Some(rest) = line.strip_prefix(REVERSE_PREFERENCE) {
            let fields: Vec<&str> = rest.split_whitespace().collect();
            let (value, key) = match fields[..] {
                [value] => (value, value),
                [value, key] => (value, key),
                _ => return Err(Error::invalid(at(), "reverse-prefer 要一或兩個欄位")),
            };
            reverse_preferences.insert(value.to_string(), key.to_string());
            continue;
        }
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let (key, values) = line
            .split_once('\t')
            .ok_or_else(|| Error::invalid(at(), "少了 TAB"))?;
        let values: Vec<String> = values.split(' ').map(str::to_string).collect();
        if key.is_empty() || values.iter().any(String::is_empty) {
            return Err(Error::invalid(at(), "鍵或值是空的"));
        }
        if !keys.insert(key.to_string()) {
            return Err(Error::invalid(at(), format!("鍵「{key}」重複")));
        }
        entries.push((key.to_string(), values));
    }
    Ok(Dictionary {
        entries,
        reverse_preferences,
    })
}

/// 反轉鍵與值（OpenCC `Dict.swap`）：每個值 → 依檔案順序列出的鍵，再把偏好的鍵移到第一個。
/// 偏好指向不存在的對應時是錯誤。
pub fn reverse(file: &str, dictionary: &Dictionary) -> Result<Dictionary, Error> {
    let mut reversed: Vec<(String, Vec<String>)> = Vec::new();
    let mut index: BTreeMap<String, usize> = BTreeMap::new();
    for (key, values) in &dictionary.entries {
        for value in values {
            match index.get(value) {
                Some(&i) => reversed[i].1.push(key.clone()),
                None => {
                    index.insert(value.clone(), reversed.len());
                    reversed.push((value.clone(), vec![key.clone()]));
                }
            }
        }
    }
    for (value, preferred) in &dictionary.reverse_preferences {
        let keys = index
            .get(value)
            .map(|&i| &mut reversed[i].1)
            .filter(|keys| keys.contains(preferred))
            .ok_or_else(|| {
                Error::invalid(
                    file,
                    format!("reverse-prefer {value} → {preferred} 沒有對應"),
                )
            })?;
        let at = keys
            .iter()
            .position(|k| k == preferred)
            .expect("上面確認過");
        let key = keys.remove(at);
        keys.insert(0, key);
    }
    Ok(Dictionary {
        entries: reversed,
        reverse_preferences: BTreeMap::new(),
    })
}

/// 字典每列的第一個值組成的轉換表。
pub fn table(dictionary: &Dictionary) -> ConversionTable {
    ConversionTable::new(
        dictionary
            .entries
            .iter()
            .map(|(key, values)| (key.clone(), values[0].clone())),
    )
}

/// 照 `hk2s.json` 組出轉換：`read(檔)` 讀上面列的各字典。
pub fn hk2s(read: impl Fn(&str) -> Result<String, Error>) -> Result<Simplifier, Error> {
    let load = |file: &str| parse(file, &read(file)?);
    let phrases = table(&load(TS_PHRASES)?);
    Ok(Simplifier::new(
        vec![vec![table(&load(COMPATIBILITY)?)]],
        phrases.clone(),
        vec![
            vec![
                table(&load(HK_VARIANTS_REV_PHRASES)?),
                table(&reverse(HK_VARIANTS, &load(HK_VARIANTS)?)?),
            ],
            vec![phrases, table(&load(TS_CHARACTERS)?)],
        ],
    ))
}
