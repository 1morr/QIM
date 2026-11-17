//! 英文模式的建議列（ADR 0008，M4 票 24）：正在打的串的前綴補完，英文詞典加英文使用者詞；不出下一個詞。
//! 英文容錯與自動修正（M4 票 25）：一處拼錯的修正列進建議列，原樣不是詞、修正又夠明確時串打完那一下換掉。

use std::collections::{HashMap, HashSet};

use crate::english_user::{ENGLISH_SHOWN_AFTER, UsedWord};
use crate::lattice::{CandidateSpan, SpanKind};
use crate::{Engine, UserModel};

/// 游標前的字在句子的哪裡（句首自動大寫，`Snapshot::auto_shift`）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Sentence {
    /// 不知道游標前是什麼（前文作廢、無痕時剛換欄位）：不自動大寫。
    #[default]
    Unknown,
    /// 句子開頭：空的欄位、句尾標點後接空白、換行之後。
    Start,
    /// 剛打了句尾標點（`.` `?` `!`），再接空白才是句子開頭（`e.g` 的 `.` 後面接字母就不是）。
    Ended,
    /// 句子中間。
    Middle,
}

impl Sentence {
    /// 從這裡接著打出 `text` 之後在哪裡。照 Gboard 實測（事實 M48）：空的欄位、`.` `?` 後接空白、換行之後
    /// 自動大寫；`!` 同 `?`。`e.g. this` 這類縮寫後 Gboard 不大寫，這裡照樣大寫。
    pub(crate) fn after(self, text: &str) -> Self {
        text.chars().fold(self, |state, c| match c {
            '\n' => Self::Start,
            '.' | '?' | '!' => Self::Ended,
            c if c.is_whitespace() => match state {
                Self::Ended | Self::Start => Self::Start,
                other => other,
            },
            _ => Self::Middle,
        })
    }
}

/// 點了建議、補了空白之後緊接著打 `c` 時，先刪掉補的空白：句讀與右括號（`hello,`、`hello.`、`(hello)`）。
/// Gboard 實測打逗號會刪（事實 M48），其餘照同一個道理：這些字元前面不留空白。
pub(crate) fn takes_picked_space(c: char) -> bool {
    matches!(c, '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']' | '}')
}

/// 建議列最多幾個：兩頁（[`crate::PAGE_SIZE`]），候選展開面板看得到第二頁。
pub(crate) const ENGLISH_SUGGESTION_LIMIT: usize = 2 * crate::PAGE_SIZE;

/// LatinIME 的機率刻度：機率每多一倍，0–255 的編碼值多這麼多（`PROBABILITY_ENCODING_SCALER`，事實 M50）。詞典的
/// 頻率 `f` 就是這個刻度。
const PROBABILITY_SCALE: f64 = 8.589_237;

/// 學過一次的加分相當於機率乘幾倍（[`learned_bonus`]）。LatinIME 沒有加分（同一個詞取主詞典與使用者歷史較大的，
/// 事實 M50），打一次的詞典詞照它不會往前；這裡要它照 Gboard 打過一次就出在建議列（事實 M47），又不壓過常用詞。
/// AOSP 詞表裡 `hel` 開頭第 3 名比 helmet 高 31，`t` 開頭第 3 名比 tumors 高 97：64 倍（+51.5）落在中間。
const FIRST_USE_FACTOR: f64 = 64.0;

/// 學過 `count` 次（一年內）的加分：機率乘上 `FIRST_USE_FACTOR × count` 倍，打得越多越前，每多一倍次數多
/// [`PROBABILITY_SCALE`]（1 次 +51.5、2 次 +60.1、5 次 +71.5、20 次 +88.7）。
fn learned_bonus(count: u64) -> f64 {
    PROBABILITY_SCALE * (FIRST_USE_FACTOR * count as f64).log2()
}

/// 詞典沒有的英文使用者詞的底分：照 LatinIME 把使用者歷史的次數換成同一刻度的機率（事實 M50）：
/// `255 + 刻度 × log2(次數 ÷ 8192) − 32`。分母是 LatinIME 總輸入次數的下限 8192（QIM 不記總輸入次數，取下限），
/// −32 是它沒有前一個詞時的退回權重。打過 2 次是 120。
fn history_probability(count: u64) -> f64 {
    255.0 + PROBABILITY_SCALE * (count as f64 / 8192.0).log2() - 32.0
}

/// 建議列怎麼列（[`suggestions`]）。
#[derive(Debug, Clone, Copy)]
pub(crate) struct Listing<'a> {
    /// 英文使用者詞開著（[`crate::Options::english_user_words`]）：學過的加分、過了出現門檻的列。
    pub(crate) user_words: bool,
    /// 英文容錯開著（[`crate::Options::english_corrections`]）：也列一處拼錯的修正。
    pub(crate) corrections: bool,
    /// 自動修正會換成的字（[`autocorrection`]）：排第一，殼看得到打完那一下會換成什麼。
    pub(crate) autocorrection: Option<&'a str>,
}

/// 一個詞在 LatinIME 刻度上的分數與種類。
type Scored<'a> = (f64, &'a str, SpanKind);

/// 正在打的詞 `typed`（[`crate::english_user::TypedRun::word`]）的建議，照使用頻率排（使用者 2026-11-17 定照 LatinIME
/// 合比、不分兩層）：每個詞一個分數，在 LatinIME 的 0–255 對數刻度上比，大的在前，同分依寫法。
///
/// - 英文詞典的詞：詞典的頻率；學過的（英文使用者詞，`user_words` 開著時，一年內打過，詞典有的詞不受出現門檻管）
///   再加 [`learned_bonus`]。
/// - 詞典沒有的英文使用者詞：過了出現門檻（[`ENGLISH_SHOWN_AFTER`]）才列，[`history_probability`] 加
///   [`learned_bonus`]。
/// - 英文容錯開著時，一處拼錯的修正（[`corrections`]）也列：同上的分數減掉拼錯的扣分與 [`CORRECTION_PENALTY`]。
///   補完沒列到的才列。
///
/// 比對不分大小寫；詞典的詞用詞典的寫法、詞典沒有的用最後打的寫法，再照 `typed` 的大小寫改（[`Case`]）。
/// 改完和 `typed` 一字不差的不列（選了也不變）。自動修正會換成的字排第一。最多 [`ENGLISH_SUGGESTION_LIMIT`] 個。
pub(crate) fn suggestions(
    engine: &Engine,
    user: &UserModel,
    typed: &str,
    listing: Listing<'_>,
) -> Vec<CandidateSpan> {
    let prefix = typed.to_lowercase();
    let dictionary = engine.english_words();
    let learned: HashMap<&str, UsedWord> = if listing.user_words {
        user.english_starting_with(&prefix)
            .map(|w| (w.key, w))
            .collect()
    } else {
        HashMap::new()
    };
    let mut scored: Vec<Scored> = dictionary
        .starting_with(&prefix)
        .map(|(word, frequency)| {
            let bonus = learned
                .get(word.to_lowercase().as_str())
                .map_or(0.0, |w| learned_bonus(w.count));
            (f64::from(frequency) + bonus, word, SpanKind::English)
        })
        .collect();
    scored.extend(
        learned
            .values()
            .filter(|w| w.count >= ENGLISH_SHOWN_AFTER && dictionary.lookup(w.key).is_none())
            .map(|w| {
                let score = history_probability(w.count) + learned_bonus(w.count);
                (score, w.text, SpanKind::EnglishUserWord)
            }),
    );
    if listing.corrections {
        scored.extend(
            corrections(engine, user, &prefix, listing.user_words)
                .into_iter()
                .map(|c| {
                    let score = c.score - c.edit.penalty() - CORRECTION_PENALTY;
                    (score, c.text, SpanKind::EnglishCorrection)
                }),
        );
    }
    // 只取分數最高的一批再排：一個字母開頭的詞有上萬個。多留一個給和 `typed` 相同而不列的。
    let wanted = ENGLISH_SUGGESTION_LIMIT + 1;
    let by_score = |a: &Scored, b: &Scored| b.0.total_cmp(&a.0).then_with(|| a.1.cmp(b.1));
    if scored.len() > wanted {
        scored.select_nth_unstable_by(wanted, by_score);
        scored.truncate(wanted);
    }
    scored.sort_by(by_score);

    let case = Case::of(typed);
    let first = listing
        .autocorrection
        .map(|text| (text.to_string(), SpanKind::EnglishCorrection));
    let mut seen: HashSet<String> = HashSet::new();
    first
        .into_iter()
        .chain(
            scored
                .into_iter()
                .map(|(_, text, kind)| (case.apply(text), kind)),
        )
        .filter_map(|(shown, kind)| {
            (shown != typed && seen.insert(shown.clone())).then(|| CandidateSpan {
                text: shown,
                len: typed.len(),
                segments: Vec::new(),
                kind,
                rule: None,
                neighbor: None,
            })
        })
        .take(ENGLISH_SUGGESTION_LIMIT)
        .collect()
}

/// 修正在建議列比補完多扣的分：一處拼錯的詞要比打的開頭補得出的詞常見 64 倍（6 倍刻度，51.5）才排到它前面，
/// `hel` 給的 hello（120）才不被 her（176）擠下去。
const CORRECTION_PENALTY: f64 = 6.0 * PROBABILITY_SCALE;

/// 一處拼錯的種類（[`corrections`]）：決定扣多少分、能不能自動修。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Edit {
    /// 漏了撇號（`dont` → don't），或重複的字母多打、少打一個（`adress` → address、`neccessary` → necessary）：
    /// 觸控最常見的打法，不扣分。
    Slip,
    /// 打錯一個字母、相鄰兩個字母對調、漏打一個字母：扣一倍（機率減半）。
    Typo,
    /// 多打一個不是重複的字母：扣三倍，只列在建議列、不自動修。去掉一個字母常得到很常見的短詞，
    /// `finaly` 會修成 final、`youd` 修成 you。
    Extra,
}

impl Edit {
    fn penalty(self) -> f64 {
        match self {
            Self::Slip => 0.0,
            Self::Typo => PROBABILITY_SCALE,
            Self::Extra => 3.0 * PROBABILITY_SCALE,
        }
    }
}

/// 一處拼錯換得到的字元：英文字母、撇號與連字號（LatinIME 的 word connectors）。
const EDIT_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz'-";

/// 小寫的 `word` 改一處（Damerau：換一個字元、對調相鄰兩個、漏打或多打一個）得到的每個字串，留最便宜的種類。
fn one_edit(word: &str) -> HashMap<String, Edit> {
    let chars: Vec<char> = word.chars().collect();
    let n = chars.len();
    let mut out: HashMap<String, Edit> = HashMap::new();
    let mut add = |candidate: Vec<char>, edit: Edit| {
        if candidate.is_empty() || candidate == chars {
            return;
        }
        let entry = out.entry(candidate.into_iter().collect()).or_insert(edit);
        *entry = (*entry).min(edit);
    };
    let at = |i: usize| chars.get(i).copied();
    for i in 0..=n {
        let (head, tail) = chars.split_at(i);
        let before = i.checked_sub(1).and_then(at);
        if let Some(&c) = tail.first() {
            // 多打了 `c`：它和左右一樣時是重複字母。
            let repeated = before == Some(c) || at(i + 1) == Some(c);
            add(
                [head, &tail[1..]].concat(),
                if repeated { Edit::Slip } else { Edit::Extra },
            );
            if let Some(&next) = tail.get(1)
                && next != c
            {
                add([head, &[next, c], &tail[2..]].concat(), Edit::Typo);
            }
            for x in EDIT_ALPHABET.chars().filter(|&x| x != c) {
                add([head, &[x], &tail[1..]].concat(), Edit::Typo);
            }
        }
        // 漏打了 `x`：撇號與重複的字母不扣分。
        for x in EDIT_ALPHABET.chars() {
            let slip = x == '\'' || before == Some(x) || at(i) == Some(x);
            add(
                [head, &[x], tail].concat(),
                if slip { Edit::Slip } else { Edit::Typo },
            );
        }
    }
    out
}

/// 一個修正：詞的分數（拼錯的扣分還沒減）、寫法與拼錯的種類。
#[derive(Debug, Clone, Copy)]
pub(crate) struct Correction<'a> {
    score: f64,
    text: &'a str,
    edit: Edit,
}

/// 小寫的 `key` 一處拼錯（[`one_edit`]）讀得通的詞：英文詞典的詞，或 `user_words` 開著時過了出現門檻的英文使用者詞；
/// 分數同 [`suggestions`] 的補完（詞典的頻率加學過的加分）。
fn corrections<'a>(
    engine: &'a Engine,
    user: &'a UserModel,
    key: &str,
    user_words: bool,
) -> Vec<Correction<'a>> {
    let dictionary = engine.english_words();
    one_edit(key)
        .into_iter()
        .filter_map(|(candidate, edit)| {
            let used = user_words.then(|| user.english_used(&candidate)).flatten();
            let bonus = used.map_or(0.0, |w| learned_bonus(w.count));
            match dictionary.entry(&candidate) {
                Some((text, frequency)) => Some(Correction {
                    score: f64::from(frequency) + bonus,
                    text,
                    edit,
                }),
                None => used
                    .filter(|w| w.count >= ENGLISH_SHOWN_AFTER)
                    .map(|w| Correction {
                        score: history_probability(w.count) + bonus,
                        text: w.text,
                        edit,
                    }),
            }
        })
        .collect()
}

/// 自動修正要贏過次一個修正多少分：機率至少 4 倍（刻度兩倍）。差不多常見的（`peice` 的 piece 137、peace 135；
/// `helo` 扣分後的 held 149 與 help 140）不修，留在建議列給使用者選。沒有空間模型，`helo` 的 o 換成遠處的 d 和換成
/// 鄰鍵 p 扣一樣的分，所以門檻要高：AOSP 詞表常用的 3,000 詞隨機造一處錯字，2 倍時修對 61.2%、修錯 9.0%，
/// 4 倍時 58.9%、6.6%（M4 票 25 Comments）。
const AUTOCORRECT_MARGIN: f64 = 2.0 * PROBABILITY_SCALE;

/// 自動修正至少幾個字元（去掉詞尾的 `.` `'`）：兩個字母的詞一處拼錯幾乎都讀得通好幾個詞（`os` 是 of、is、on）。
const AUTOCORRECT_MIN_CHARS: usize = 3;

/// 正在打的詞 `typed`（[`crate::english_user::TypedRun::word`]）打完時要換成的字（含原本的詞尾 `.` `'`）；不修時
/// 是 `None`。照 LatinIME（`Suggest.java`）對原樣讀得通的保守：
///
/// - 只修去掉詞尾 `.` `'` 後全是字母、中間可有 `'` `-`、至少 [`AUTOCORRECT_MIN_CHARS`] 個字元的詞；含數字、`@`
///   這類 email 字元、中間有 `.` 的不修（ADR 0008）。大寫字母超過一個的不修（LatinIME 的 `isMostlyCaps`，
///   全大寫也在內）；句中首字大寫照修、保留大寫（Gboard，事實 M47）。
/// - 原樣是詞的不修：英文詞典有、學過的英文使用者詞（`user_words` 開著時，一年內打過，不論次數；LatinIME 與
///   Gboard 都把使用者歷史當有效詞），或使用者在這個 Session 還原過的（`kept`）。
/// - 修正的對象只有不多打字母的一處拼錯（[`Edit::Extra`] 不算），分數最高的要贏過次一個 [`AUTOCORRECT_MARGIN`]；
///   有漏撇號的修正（`youd` 的 you'd）而最高的不是它時也不修：撇號詞在詞表裡頻率低，常被 your 這類詞蓋過。
pub(crate) fn autocorrection(
    engine: &Engine,
    user: &UserModel,
    typed: &str,
    user_words: bool,
    kept: &HashSet<String>,
) -> Option<String> {
    let core = typed.trim_end_matches(['.', '\'']);
    let suffix = &typed[core.len()..];
    let fits = core.chars().count() >= AUTOCORRECT_MIN_CHARS
        && core.starts_with(|c: char| c.is_ascii_alphabetic())
        && core
            .chars()
            .all(|c| c.is_ascii_alphabetic() || matches!(c, '\'' | '-'))
        && core.chars().filter(char::is_ascii_uppercase).count() <= 1;
    if !fits {
        return None;
    }
    let key = core.to_lowercase();
    let valid = engine.english_words().lookup(&key).is_some()
        || (user_words && user.english_used(&key).is_some())
        || kept.contains(&key);
    if valid {
        return None;
    }
    let mut found: Vec<Correction> = corrections(engine, user, &key, user_words)
        .into_iter()
        .filter(|c| c.edit != Edit::Extra)
        .collect();
    found.sort_by(|a, b| {
        (b.score - b.edit.penalty())
            .total_cmp(&(a.score - a.edit.penalty()))
            .then_with(|| a.text.cmp(b.text))
    });
    let score = |c: &Correction| c.score - c.edit.penalty();
    let best = found.first()?;
    let runner_up = found.get(1).map_or(f64::NEG_INFINITY, score);
    let apostrophe_elsewhere =
        !best.text.contains('\'') && found.iter().any(|c| c.text.contains('\''));
    if score(best) - runner_up < AUTOCORRECT_MARGIN || apostrophe_elsewhere {
        return None;
    }
    Some(Case::of(core).apply(best.text) + suffix)
}

/// 打的詞的大小寫，建議跟著它（Gboard 句中首字大寫的詞，修正也保留大寫，事實 M47）：第一個字母大寫時建議的
/// 開頭也大寫；兩個字母以上而全是大寫時整個建議大寫；其他照建議原本的寫法（`nasa` 給 NASA）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Case {
    AsIs,
    Capitalized,
    Upper,
}

impl Case {
    fn of(typed: &str) -> Self {
        let letters: Vec<char> = typed.chars().filter(|c| c.is_alphabetic()).collect();
        match letters.first() {
            Some(c) if c.is_uppercase() => {
                if letters.len() >= 2 && letters.iter().all(|c| c.is_uppercase()) {
                    Self::Upper
                } else {
                    Self::Capitalized
                }
            }
            _ => Self::AsIs,
        }
    }

    fn apply(self, text: &str) -> String {
        match self {
            Self::AsIs => text.to_string(),
            Self::Upper => text.to_uppercase(),
            Self::Capitalized => {
                let mut chars = text.chars();
                chars
                    .next()
                    .map(|first| first.to_uppercase().chain(chars).collect())
                    .unwrap_or_default()
            }
        }
    }
}
