//! 繁體模式排在同碼的繁體字之後的簡體字（brief §4.1、plan §4.1 步驟 4，M3 票 19）。
//!
//! 不能只看「在 OpenCC 對照表的簡體那邊出現過」：台（臺）、后（後）、里（裡）這類字本身就是
//! 正規繁體字。這裡只算 HKSCS 為了香港用字補進來、Big5 沒有的字，再用 OpenCC 與香港
//! 《常用字字形表》（Unihan `kHKGlyph`）排掉繁體字。

use std::collections::{BTreeMap, BTreeSet};

use crate::corpora::CANTONESE_KEPT;
use crate::libcangjie::Row;
use crate::opencc::Dictionary;

/// 判定簡體字用的資料。
pub struct References<'a> {
    /// OpenCC STCharacters：簡體字 → 繁體字。
    pub st: &'a Dictionary,
    /// OpenCC TSCharacters：繁體字 → 簡體字。
    pub ts: &'a Dictionary,
    /// OpenCC HKVariants：標準繁體字 → 香港寫法。
    pub hk_variants: &'a Dictionary,
    /// Unihan `kHKGlyph`：香港《常用字字形表》的序號（[`crate::unihan::hk_glyphs`]）。
    pub hk_glyphs: &'a BTreeMap<char, u16>,
}

/// 簡體字：連打字集裡只有 HKSCS 旗標（不在 Big5）、而且 OpenCC 認定只是簡體寫法的字：
///
/// - `st` 簡轉繁一定換掉它：它是鍵、值裡沒有它自己（「叶 → 葉 叶」不算）；
/// - `ts` 繁轉簡不把它當繁體字留著（「麽 → 麽」不算）；
/// - 不是 `hk_variants` 的香港寫法（温、户、囱）；
/// - 不是香港《常用字字形表》的標準字形：字表收了它，`st` 給的繁體寫法卻沒有一個和它同一條
///   （够收了、夠沒收）；和繁體寫法同一條的是列在一起的異體（叙和敘都是 1715），照算；
/// - 不是前處理保留的粵語用字（[`CANTONESE_KEPT`]：吓、响）。
pub fn characters(rows: &[Row], references: &References) -> BTreeSet<char> {
    let References {
        st,
        ts,
        hk_variants,
        hk_glyphs,
    } = references;
    // 同一個字的旗標可能分在幾列。
    let mut flags: BTreeMap<char, (bool, bool)> = BTreeMap::new();
    for row in rows.iter().filter(|r| r.hanzi) {
        let (big5, hkscs) = flags.entry(row.text).or_default();
        *big5 |= row.big5;
        *hkscs |= row.hkscs;
    }
    let maps_to_itself = |dictionary: &Dictionary, c: &str| {
        dictionary
            .entries
            .iter()
            .any(|(key, values)| key == c && values.iter().any(|v| v == c))
    };
    // 簡轉繁一定換掉的字 → 它的繁體寫法。
    let always_converted: BTreeMap<&str, &[String]> = st
        .entries
        .iter()
        .filter(|(key, values)| !values.contains(key))
        .map(|(key, values)| (key.as_str(), values.as_slice()))
        .collect();
    let hong_kong_standard = |c: char, traditional: &[String]| {
        hk_glyphs.get(&c).is_some_and(|index| {
            !traditional
                .iter()
                .filter_map(|t| t.chars().next())
                .any(|t| hk_glyphs.get(&t) == Some(index))
        })
    };
    let hong_kong_forms: BTreeSet<&str> = hk_variants
        .entries
        .iter()
        .flat_map(|(key, values)| values.iter().filter(move |v| *v != key))
        .map(String::as_str)
        .collect();
    flags
        .into_iter()
        .filter(|&(_, (big5, hkscs))| hkscs && !big5)
        .map(|(c, _)| c)
        .filter(|&c| {
            let text = c.to_string();
            always_converted
                .get(text.as_str())
                .is_some_and(|traditional| !hong_kong_standard(c, traditional))
                && !maps_to_itself(ts, &text)
                && !hong_kong_forms.contains(text.as_str())
                && !CANTONESE_KEPT.contains(&c)
        })
        .collect()
}
