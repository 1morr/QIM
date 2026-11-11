//! 簡繁轉換（plan §2.2 的後處理、§4.4）：照 OpenCC 的轉換規則，把繁體轉成簡體。
//! 表由簡繁資料包給（OpenCC `hk2s` 用的字典），核心不內嵌字典。

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

/// 一張轉換表：鍵 → 轉換結果（OpenCC 字典每列的第一個值）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "BTreeMap<String, String>", into = "BTreeMap<String, String>")]
pub struct ConversionTable {
    map: HashMap<String, String>,
    /// 最長的鍵有幾個字元，找前綴的上限。
    longest: usize,
}

impl ConversionTable {
    /// 空的鍵不收。
    pub fn new(entries: impl IntoIterator<Item = (String, String)>) -> Self {
        let map: HashMap<String, String> =
            entries.into_iter().filter(|(k, _)| !k.is_empty()).collect();
        let longest = map.keys().map(|k| k.chars().count()).max().unwrap_or(0);
        Self { map, longest }
    }

    /// `text` 開頭最長的鍵：（鍵的位元組長度, 轉換結果）。
    fn longest_prefix<'a>(&'a self, text: &str) -> Option<(usize, &'a str)> {
        text.char_indices()
            .map(|(i, c)| i + c.len_utf8())
            .take(self.longest)
            .filter_map(|end| self.map.get(&text[..end]).map(|v| (end, v.as_str())))
            .last()
    }
}

impl From<BTreeMap<String, String>> for ConversionTable {
    fn from(entries: BTreeMap<String, String>) -> Self {
        Self::new(entries)
    }
}

impl From<ConversionTable> for BTreeMap<String, String> {
    fn from(table: ConversionTable) -> Self {
        table.map.into_iter().collect()
    }
}

/// 一組表依序找，第一張有前綴匹配的表勝出，後面的表較長也不看（OpenCC 的
/// `short_circuit`）。
fn group_prefix<'a>(group: &'a [ConversionTable], text: &str) -> Option<(usize, &'a str)> {
    group.iter().find_map(|table| table.longest_prefix(text))
}

/// 用一組表從左到右轉換：每個位置取最長的匹配，沒有匹配就照抄一個字元
/// （OpenCC `Conversion::Convert`）。
fn convert_with(group: &[ConversionTable], text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(c) = rest.chars().next() {
        let (len, converted) =
            group_prefix(group, rest).unwrap_or((c.len_utf8(), &rest[..c.len_utf8()]));
        out.push_str(converted);
        rest = &rest[len..];
    }
    out
}

/// OpenCC 一份設定的轉換：正規化 → 分段 → 各段依序經過每一組表。
/// 不處理表意文字描述序列（IDS）：候選不會有。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Simplifier {
    /// 整串先依序套用的各組（OpenCC 的 `normalization`）。
    normalization: Vec<Vec<ConversionTable>>,
    /// 分段用的詞表（OpenCC 的 `mmseg`）：詞自成一段，詞之間沒匹配的字併成一段。
    segmentation: ConversionTable,
    /// 每段依序套用的各組（OpenCC 的 `conversion_chain`）。
    chain: Vec<Vec<ConversionTable>>,
}

impl Simplifier {
    pub fn new(
        normalization: Vec<Vec<ConversionTable>>,
        segmentation: ConversionTable,
        chain: Vec<Vec<ConversionTable>>,
    ) -> Self {
        Self {
            normalization,
            segmentation,
            chain,
        }
    }

    pub fn convert(&self, text: &str) -> String {
        let normalized = self
            .normalization
            .iter()
            .fold(text.to_string(), |text, group| convert_with(group, &text));
        self.segments(&normalized)
            .into_iter()
            .map(|segment| {
                self.chain.iter().fold(segment.to_string(), |text, group| {
                    convert_with(group, &text)
                })
            })
            .collect()
    }

    /// 逐字轉回繁體的表（簡體字 → 繁體字）：轉換表裡每個單字的鍵，整個轉換後是另一個單字時，算
    /// 那個字的繁體寫法。一個簡體字有幾個繁體寫法（干 ← 乾、幹、干）時，連它自己一起比 `weight`，
    /// 取最大的，同分時取繁體寫法裡碼位最小的；選中它自己的不列。給停鍵層模型轉前文用
    /// （[`crate::Session::rerank_request`]），不是 OpenCC 的 `s2t`：只看單字、不看詞。
    pub(crate) fn traditional_chars(
        &self,
        weight: impl Fn(char) -> Option<f64>,
    ) -> HashMap<char, char> {
        let keys = self
            .normalization
            .iter()
            .chain(&self.chain)
            .flatten()
            .chain([&self.segmentation])
            .flat_map(|table| table.map.keys())
            .filter_map(|key| single_char(key));
        let mut forms: HashMap<char, Vec<char>> = HashMap::new();
        for traditional in keys {
            if let Some(simplified) = single_char(&self.convert(&traditional.to_string()))
                && simplified != traditional
            {
                forms.entry(simplified).or_default().push(traditional);
            }
        }
        forms
            .into_iter()
            .filter_map(|(simplified, mut traditional)| {
                traditional.sort_unstable();
                traditional.dedup();
                let score = |c: char| weight(c).unwrap_or(f64::NEG_INFINITY);
                // 先比繁體寫法（同分取碼位小的），再看它自己是不是更常見。
                let best = traditional
                    .into_iter()
                    .reduce(|a, b| if score(b) > score(a) { b } else { a })?;
                (score(best) >= score(simplified)).then_some((simplified, best))
            })
            .collect()
    }

    /// 分段（OpenCC `MaxMatchSegmentation`）：分段詞表的詞自成一段，其餘相鄰的字併成一段。
    fn segments<'a>(&self, text: &'a str) -> Vec<&'a str> {
        let mut segments = Vec::new();
        let mut unmatched = 0;
        let mut at = 0;
        while let Some(c) = text[at..].chars().next() {
            match self.segmentation.longest_prefix(&text[at..]) {
                Some((len, _)) => {
                    if unmatched < at {
                        segments.push(&text[unmatched..at]);
                    }
                    segments.push(&text[at..at + len]);
                    at += len;
                    unmatched = at;
                }
                None => at += c.len_utf8(),
            }
        }
        if unmatched < text.len() {
            segments.push(&text[unmatched..]);
        }
        segments
    }
}

/// `text` 剛好是一個字元時的那個字元。
fn single_char(text: &str) -> Option<char> {
    let mut chars = text.chars();
    let c = chars.next()?;
    chars.next().is_none().then_some(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simplifier(entries: &[(&str, &str)]) -> Simplifier {
        let table =
            ConversionTable::new(entries.iter().map(|&(k, v)| (k.to_string(), v.to_string())));
        Simplifier::new(vec![], ConversionTable::new([]), vec![vec![table]])
    }

    /// 一對多取 `weight` 最大的；簡體字本身（也是繁體字，如「后」）比各寫法都常見時不轉，不列。
    #[test]
    fn traditional_chars_pick_the_heaviest_form_and_skip_the_simplified_char_itself() {
        let s = simplifier(&[("乾", "干"), ("幹", "干"), ("後", "后"), ("詞", "词")]);
        let weight = |c: char| match c {
            '乾' => Some(-1.0),
            '幹' => Some(-2.0),
            '干' => Some(-3.0),
            '後' => Some(-5.0),
            '后' => Some(-4.0),
            _ => None,
        };
        let table = s.traditional_chars(weight);
        assert_eq!(table.get(&'干'), Some(&'乾'));
        assert_eq!(table.get(&'后'), None);
        // 都沒有統計時照轉表的寫法。
        assert_eq!(table.get(&'词'), Some(&'詞'));
        assert_eq!(table.len(), 2);
    }

    /// 同分時取碼位小的繁體寫法，結果不隨雜湊表的順序變。
    #[test]
    fn ties_between_traditional_forms_take_the_smallest_code_point() {
        let s = simplifier(&[("髮", "发"), ("發", "发")]);
        let table = s.traditional_chars(|_| None);
        assert!('發' < '髮');
        assert_eq!(table.get(&'发'), Some(&'發'));
    }
}
