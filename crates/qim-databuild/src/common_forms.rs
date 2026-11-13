//! 字形用常見寫法（brief〈字集與字形〉、plan〈字級 bigram 與詞級 trigram〉）：語料轉成香港繁體之後，把香港寫法與 OpenCC 標準寫法
//! 換回常見寫法（「説」→「說」、「温」→「溫」、「羣」→「群」）。使用者打常見寫法（M1 票 17b）。
//!
//! 對照表用轉語料的同一套 OpenCC（ferrous-opencc 內建的字典）逐字查碼表裡的每個字：`t2hk` 把
//! 字換成香港寫法（反過來就是香港 → 常見），`t2tw` 把 OpenCC 標準寫法換成台灣常見寫法（`s2hk` 的
//! 簡轉繁出的是前者，例如「羣」）。只收常見寫法在連打字集、而且被換掉的字不在連打字集（不換回來，
//! 含它的句子會整句丟掉）或兩字同碼（打的時候互搶）的組；不同碼的「户／戶」「台／臺」不換，免得
//! 把「吃」換成「喫」、「才」換成「纔」這類罕用寫法。
//!
//! 不同碼、兩種寫法都常見的字是並存寫法（M1 票 17c）：兩種都收，打哪個碼出哪個。
//! - 雙向（[`TWO_WAY_FORMS`]，純異體）：計數時歸到代表寫法（[`CommonForms::apply`] 把另一種換成它），
//!   建資料包時再由 [`BothForms`] 把代表寫法的字頻、詞與 bigram 照抄給另一種，兩種寫法的機率相同。
//! - 單向（[`ONE_WAY_FORMS`]）：前一個字的每個用法都能寫成後一個，反過來不行（「著名」不能寫成
//!   「着名」、「台州」不能寫成「臺州」）。只有詞庫照抄：含前一個字的詞另收後一個字的寫法；語料照
//!   原文寫（OpenCC 換掉的放回去），字頻與 bigram 各算各的。

use std::collections::{BTreeMap, BTreeSet};

use ferrous_opencc::OpenCC;
use ferrous_opencc::config::BuiltinConfig;
use qim_core::CodeTable;

use crate::Error;

/// 雙向的並存寫法（代表寫法, 另一種寫法），使用者 2026-11-05 決定（M1 票 17c、brief〈字集與字形〉）。
/// 代表寫法是計數歸到的那一個，也是基準題目的寫法；兩種寫法打出來的機率相同。
/// 後 6 組是 `t2hk` 換掉 rime-essay 寫法、兩種寫法同義的字，代表寫法是 `t2hk` 換成的那個（M1 票 17d）。
pub const TWO_WAY_FORMS: [(char, char); 17] = [
    ('線', '綫'),
    ('啟', '啓'),
    ('麵', '麪'),
    ('床', '牀'),
    ('戶', '户'),
    ('敘', '敍'),
    ('臥', '卧'),
    ('裏', '裡'),
    ('污', '汙'),
    ('泄', '洩'),
    ('枱', '檯'),
    ('鈎', '鉤'),
    ('濕', '溼'),
    ('抬', '擡'),
    ('粽', '糉'),
    ('枴', '柺'),
    ('囱', '囪'),
];

/// 單向的並存寫法（寫法, 也可寫成）：
/// - 「着」的用法都能寫成「著」（隨着、隨著），「著」的 zhù 義項（著名、著作、土著）不能寫成「着」
///   （M1 票 17c，2026-11-05）。
/// - 「臺」「菸」都能寫成「台」「煙」（臺灣、台灣；香菸、香煙），「台州」「煙霧」不能寫成「臺州」
///   「菸霧」（M1 票 17d，使用者 2026-11-05 同意）。`t2hk` 把「臺」「菸」換成「台」「煙」，前處理
///   照原文放回去（[`BothForms::as_written`]）。
pub const ONE_WAY_FORMS: [(char, char); 3] = [('着', '著'), ('臺', '台'), ('菸', '煙')];

/// 單向寫法反過來寫的錯字（「著」的 zhù 義項寫成「着」）：建詞庫時改回「著」（[`BothForms::correct`]），
/// `--verify` 查詞庫沒有這些詞。「顯着」是 rime-essay 原有的（詞頻 279，使用者 2026-11-05 決定
/// 併進「顯著」），其餘是防反向生出的。
pub const REVERSED_ONE_WAY: [&str; 13] = [
    "着名",
    "着作",
    "着作權",
    "原着",
    "名着",
    "着稱",
    "土着",
    "卓着",
    "着述",
    "着者",
    "專着",
    "巨着",
    "顯着",
];

/// 單向寫法生出的另一種寫法是別的詞、義項不同的詞（M1 票 17d）：只收原寫法，不生出另一種、
/// 也不併進那個詞。「合着」（原來如此）≠「合著」（合寫的書）；「所着」（穿的）≠「所著」（寫的）；
/// 「舊着」≠「舊著」（舊作）。「刻着／刻著」「張着／張著」這類是同一個詞，照生。
pub const ONE_WAY_DISTINCT: [&str; 3] = ["合着", "所着", "舊着"];

/// 並存寫法的對照：雙向的每組一個代表寫法與另一種寫法，單向的每組一個寫法與也可寫成的字。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BothForms {
    /// 雙向：代表寫法 → 另一種寫法。
    other: BTreeMap<char, char>,
    /// 雙向：另一種寫法 → 代表寫法。
    representative: BTreeMap<char, char>,
    /// 單向：寫法 → 也可寫成的字。
    one_way: BTreeMap<char, char>,
}

impl BothForms {
    /// `two_way` 是雙向的（代表寫法, 另一種寫法），`one_way` 是單向的（寫法, 也可寫成）；
    /// 一個字出現在兩組以上、或一組的兩個字相同時失敗。
    pub fn new(two_way: &[(char, char)], one_way: &[(char, char)]) -> Result<Self, Error> {
        let mut forms = Self::default();
        let mut seen = BTreeSet::new();
        for &(a, b) in two_way.iter().chain(one_way) {
            if a == b || !seen.insert(a) || !seen.insert(b) {
                return Err(Error::invalid("並存寫法", format!("「{a}／{b}」的字重複")));
            }
        }
        for &(representative, other) in two_way {
            forms.other.insert(representative, other);
            forms.representative.insert(other, representative);
        }
        forms.one_way.extend(one_way.iter().copied());
        Ok(forms)
    }

    /// 單向的每一組（寫法, 也可寫成），依寫法的碼位。
    pub fn one_way_pairs(&self) -> impl Iterator<Item = (char, char)> + '_ {
        self.one_way.iter().map(|(&from, &to)| (from, to))
    }

    /// 雙向的每一組（代表寫法, 另一種寫法），依代表寫法的碼位。
    pub fn two_way_pairs(&self) -> impl Iterator<Item = (char, char)> + '_ {
        self.other.iter().map(|(&r, &o)| (r, o))
    }

    /// 雙向：`c` 是另一種寫法時換成代表寫法，其餘不變（單向的不換）。
    pub fn representative(&self, c: char) -> char {
        self.representative.get(&c).copied().unwrap_or(c)
    }

    /// 雙向：`c` 是代表寫法時，它的另一種寫法。
    pub fn other(&self, c: char) -> Option<char> {
        self.other.get(&c).copied()
    }

    /// 單向：`c` 是寫法時，它也可寫成的字。
    pub fn one_way(&self, c: char) -> Option<char> {
        self.one_way.get(&c).copied()
    }

    /// 單向的寫法照原文：`converted` 是 `raw` 經 OpenCC 轉換的結果，被換掉的單向寫法（「臺」→「台」）
    /// 放回原字。轉換改了字數時照 `converted`（OpenCC 的轉換逐字對應，M1 票 17b 查過四份語料）。
    pub fn as_written(&self, raw: &str, converted: &str) -> String {
        if raw.chars().count() != converted.chars().count() {
            return converted.to_string();
        }
        raw.chars()
            .zip(converted.chars())
            .map(|(r, c)| if self.one_way.contains_key(&r) { r } else { c })
            .collect()
    }

    /// 單向寫法反過來寫的錯字（[`REVERSED_ONE_WAY`]，例如「顯着」）改回原字（「顯著」），其餘不變。
    pub fn correct(&self, text: &str) -> String {
        if !REVERSED_ONE_WAY.contains(&text) {
            return text.to_string();
        }
        text.chars().map(|c| self.one_way(c).unwrap_or(c)).collect()
    }

    /// `text` 的每一種寫法：雙向的代表寫法、單向的寫法各換或不換成另一個字，第一個是 `text` 本身。
    /// 單向的不反過來換：「著名」只有「著名」；[`ONE_WAY_DISTINCT`] 的詞只換雙向的。
    pub fn spellings(&self, text: &str) -> Vec<String> {
        if ONE_WAY_DISTINCT.contains(&text) {
            return self.two_way_spellings(text);
        }
        expand(text, |c| self.other(c).or_else(|| self.one_way(c)))
    }

    /// 只換雙向的代表寫法的 [`Self::spellings`]。
    pub fn two_way_spellings(&self, text: &str) -> Vec<String> {
        expand(text, |c| self.other(c))
    }

    /// 雙向的兩種寫法，字頻都改成兩者的和；單向的不動。
    pub fn share_frequency(&self, frequency: &mut BTreeMap<char, u64>) {
        for (representative, other) in self.two_way_pairs() {
            let total = [representative, other]
                .iter()
                .filter_map(|c| frequency.get(c))
                .sum::<u64>();
            if total > 0 {
                frequency.insert(representative, total);
                frequency.insert(other, total);
            }
        }
    }
}

/// `text` 的字各換或不換成 `other` 給的字，第一個是 `text` 本身。
fn expand(text: &str, other: impl Fn(char) -> Option<char>) -> Vec<String> {
    let mut spellings = vec![String::new()];
    for c in text.chars() {
        let other = other(c);
        spellings = spellings
            .into_iter()
            .flat_map(|s| {
                let with = |c: char| {
                    let mut s = s.clone();
                    s.push(c);
                    s
                };
                std::iter::once(with(c)).chain(other.map(with))
            })
            .collect();
    }
    spellings
}

/// 寫法對照：被換掉的字（香港寫法或 OpenCC 標準寫法）→ 常見寫法，加上並存寫法。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommonForms {
    /// 被換掉的字 → 常見寫法。
    forms: BTreeMap<char, char>,
    both: BothForms,
}

impl CommonForms {
    /// 用 ferrous-opencc 內建的 `t2hk`、`t2tw` 建表，和 `corpora::Converter` 同一套字典；
    /// 並存寫法是 [`TWO_WAY_FORMS`] 與 [`ONE_WAY_FORMS`]。
    pub fn from_opencc(table: &CodeTable) -> Result<Self, Error> {
        let open = |config, name: &str| {
            OpenCC::from_config(config).map_err(|e| Error::invalid(name, e.to_string()))
        };
        let (hong_kong, taiwan) = (
            open(BuiltinConfig::T2hk, "OpenCC t2hk")?,
            open(BuiltinConfig::T2tw, "OpenCC t2tw")?,
        );
        Self::new(
            table,
            one_char(&hong_kong),
            one_char(&taiwan),
            BothForms::new(&TWO_WAY_FORMS, &ONE_WAY_FORMS)?,
        )
    }

    /// `to_hong_kong`、`to_taiwan` 把一個字換成香港寫法、台灣常見寫法，沒有就回原字；
    /// 查的是 `table`（速成碼表）裡的每個字，連打字集與同碼也由它判斷。
    /// 同一個字要換成兩種寫法，或換完的字又在表裡（一次換不完），都讓建置失敗。
    /// `both` 的字都要在連打字集，也不能是對照表換掉或換成的字；兩個字都不在 `table` 的組略過。
    pub fn new(
        table: &CodeTable,
        to_hong_kong: impl Fn(char) -> char,
        to_taiwan: impl Fn(char) -> char,
        both: BothForms,
    ) -> Result<Self, Error> {
        let mut codes: BTreeMap<char, BTreeSet<&str>> = BTreeMap::new();
        let mut all = BTreeSet::new();
        for (code, entries) in table.iter() {
            for entry in entries {
                all.insert(entry.text);
                if !entry.extension {
                    codes.entry(entry.text).or_default().insert(code);
                }
            }
        }
        let continuous = |c: char| codes.contains_key(&c);
        let same_code = |a: char, b: char| match (codes.get(&a), codes.get(&b)) {
            (Some(a), Some(b)) => !a.is_disjoint(b),
            _ => false,
        };
        // (被換掉的字, 常見寫法)：香港寫法換回原字；OpenCC 標準寫法換成台灣寫法。
        let candidates = all
            .iter()
            .map(|&c| (to_hong_kong(c), c))
            .chain(all.iter().map(|&c| (c, to_taiwan(c))));
        let mut forms = BTreeMap::new();
        for (from, to) in candidates {
            if from == to || !continuous(to) || (continuous(from) && !same_code(from, to)) {
                continue;
            }
            if let Some(other) = forms.insert(from, to).filter(|&other| other != to) {
                return Err(Error::invalid(
                    "字形對照表",
                    format!("「{from}」同時要換成「{other}」與「{to}」"),
                ));
            }
        }
        if let Some((from, to)) = forms.iter().find(|(_, to)| forms.contains_key(to)) {
            return Err(Error::invalid(
                "字形對照表",
                format!("「{from}」換成「{to}」後還要再換，一次換不完"),
            ));
        }
        // 碼表兩個字都沒有（測試用的小碼表）就略過；有其中一個就兩個都要在連打字集。
        let check = |pairs: &mut dyn Iterator<Item = (char, char)>| {
            let mut kept = Vec::new();
            for (a, b) in pairs {
                if !all.contains(&a) && !all.contains(&b) {
                    continue;
                }
                kept.push((a, b));
                for c in [a, b] {
                    if !continuous(c) {
                        return Err(Error::invalid("並存寫法", format!("「{c}」不在連打字集")));
                    }
                    if forms.contains_key(&c) || forms.values().any(|&to| to == c) {
                        return Err(Error::invalid("並存寫法", format!("「{c}」也在字形對照表")));
                    }
                }
            }
            Ok(kept)
        };
        let two_way = check(&mut both.two_way_pairs())?;
        let one_way = check(&mut both.one_way_pairs())?;
        Ok(Self {
            forms,
            both: BothForms::new(&two_way, &one_way)?,
        })
    }

    /// 對照表每一組（被換掉的字, 常見寫法），依被換掉的字的碼位；不含並存寫法。
    pub fn pairs(&self) -> impl Iterator<Item = (char, char)> + '_ {
        self.forms.iter().map(|(&from, &to)| (from, to))
    }

    /// 並存寫法（雙向與單向），建資料包時展開詞的寫法用。
    pub fn both(&self) -> &BothForms {
        &self.both
    }

    /// 換成常見寫法，並存寫法換成代表寫法。
    pub fn apply(&self, text: &str) -> String {
        text.chars()
            .map(|c| {
                let c = self.forms.get(&c).copied().unwrap_or(c);
                self.both.representative(c)
            })
            .collect()
    }
}

/// OpenCC 換一個字；換出來不是一個字（字典裡沒有這種列）時當作不換。
fn one_char(opencc: &OpenCC) -> impl Fn(char) -> char + '_ {
    move |c| {
        let out = opencc.convert(c.encode_utf8(&mut [0; 4]));
        let mut chars = out.chars();
        match (chars.next(), chars.next()) {
            (Some(one), None) => one,
            _ => c,
        }
    }
}
