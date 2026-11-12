//! 英文使用者詞（plan §2.1、§2.4，M4 票 14，取代 M3 的記住的字串）：核心看得到送出去的英文串都學
//! （照 LatinIME 上屏都學，不另認 email），之後出在中文組字（排位規則 4、5）。這裡是正在打的一串
//! （[`TypedRun`]）、什麼串要學（[`to_learn`]）與學到的詞（[`EnglishUserWords`]）；記在
//! [`crate::UserModel`]，由 [`crate::Session`] 從送出去的字裡認出來。

use std::collections::BTreeMap;

use crate::Date;
use crate::user::{Decayed, expired};

/// 一串最多幾個位元組：email 地址的上限（RFC 5321 的 path 上限 256 減去角括號），長過的多半不是
/// 要再打一次的字串。
pub(crate) const MAX_LEN: usize = 254;

/// 至少幾個字元才學（使用者 2026-11-12 決定，M4 檢視）：短的串多半是單字或縮寫，學了徒增條目。
const MIN_CHARS: usize = 4;

/// 英文詞典沒有的串打過幾次（一年內）才出在中文組字的完整串（排位規則 4）。詞典有的詞不受它管。
pub const ENGLISH_SHOWN_AFTER: u64 = 2;

/// 打過幾次（一年內）才給中文組字的前綴補完（排位規則 5）：補完是打開頭就給，誤觸比完整串多。
pub const ENGLISH_COMPLETION_AFTER: u64 = 3;

/// 前綴補完只給最常用（衰減值最大）的幾條：每多一條，中文組字撞上它開頭的機會就多一份（3 個字母約
/// 每 1 萬次組字一次，`scripts/experiments/remembered-prefix/`），20 條合計約 0.3%。
pub const ENGLISH_COMPLETION_POOL: usize = 20;

/// 打它開頭的字母時，候選列最多接幾個前綴補完（同 emoji 的 [`crate::EMOJI_LIMIT`]）。
pub const ENGLISH_COMPLETION_LIMIT: usize = 3;

/// 串裡的字元：英文字母、數字與 email、縮寫常有的 `. _ % + - @ '`。送出去的字遇到其他字元，前面的
/// 就是打完的一串。
fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '%' | '+' | '-' | '@' | '\'')
}

/// 正在打的一串（plan §2.1〈英文使用者詞〉）：送出去的字的結尾連續的串字元（[`is_word_char`]），
/// 遇到其他字元就是打完了。開頭不是使用者經 QIM 打的（接在輸入框原有的串字元後面、退格刪到它前面、
/// 有一段是無痕或學習關掉時打的、長過上限）時是 `partial`：打完也不學，到下一個分隔的字元重新開始。
#[derive(Debug, Clone, Default)]
pub(crate) struct TypedRun {
    text: String,
    partial: bool,
}

impl TypedRun {
    /// 送出去一個字元。串字元接上（`recording` 為假時不接、標成 `partial`）；其他字元讓這串打完，
    /// 回傳要學的串（[`TypedRun::finish`]）。
    pub(crate) fn push(&mut self, c: char, recording: bool) -> Option<String> {
        if !is_word_char(c) {
            return self.finish();
        }
        if recording && self.text.len() < MAX_LEN + 1 {
            self.text.push(c);
        } else {
            // 長過上限的不學；之後退格留下的也不是輸入框裡的樣子。
            self.partial = true;
        }
        None
    }

    /// 打完了：整串都看得到時回傳要學的部分（[`to_learn`]），然後重新開始。
    pub(crate) fn finish(&mut self) -> Option<String> {
        let done = (!self.partial)
            .then(|| to_learn(&self.text).map(String::from))
            .flatten();
        *self = Self::default();
        done
    }

    /// 交還宿主的退格刪掉最後一個字元；已經是空的時刪到了這串前面的字，之後接在不知道的字後面。
    pub(crate) fn backspace(&mut self) {
        if self.text.pop().is_none() {
            self.partial = true;
        }
    }

    /// 游標移走了（方向鍵這類交還宿主的鍵）：不知道接在什麼後面，到下一個分隔的字元重新開始。
    pub(crate) fn lose_track(&mut self) {
        self.text.clear();
        self.partial = true;
    }

    /// 殼給了輸入框游標前的文字 `before`：不以這串結尾時（游標移走、宿主改了字）丟掉這串；去掉這串後
    /// 最後一字還是串字元時，這串的開頭不是使用者經 QIM 打的。
    pub(crate) fn sync(&mut self, before: &str) {
        let rest = match before.strip_suffix(self.text.as_str()) {
            Some(rest) => rest,
            None => {
                self.text.clear();
                before
            }
        };
        self.partial = rest.ends_with(is_word_char);
    }
}

/// 一串打完的字裡要學的部分：去掉頭尾的 `.` 與 `'`（句尾的句點、引號）後，全是串字元、至少
/// [`MIN_CHARS`] 個字元、不長過 [`MAX_LEN`]、有英文字母或數字時是它，否則是 `None`。匯入時也用它
/// 檢查一列（`to_learn(text) == Some(text)`）。
pub(crate) fn to_learn(typed: &str) -> Option<&str> {
    let text = typed.trim_matches(['.', '\'']);
    let ok = text.chars().count() >= MIN_CHARS
        && text.len() <= MAX_LEN
        && text.chars().all(is_word_char)
        && text.chars().any(|c| c.is_ascii_alphanumeric());
    ok.then_some(text)
}

/// 中文組字裡打得出的部分：字串的英文字母轉小寫後，方案字母表 `alphabet` 裡有的；其他字元去掉
/// （組字裡打不出 `.`、`@`、數字與速成沒有的 `z`）。輸入碼是它的開頭時給這個字串（前綴補完）：
/// `ivan.test@example.com` 打 `iva`、`ivant` 都有，`zz98765@example.org` 打 `exa` 起有。
pub(crate) fn letters(text: &str, alphabet: &str) -> String {
    text.chars()
        .map(|c| c.to_ascii_lowercase())
        .filter(|&c| c.is_ascii_lowercase() && alphabet.contains(c))
        .collect()
}

/// 學到的英文使用者詞，各方案共用。不分大小寫：鍵是小寫，留最後打的寫法。有自己的 tick（每學一次
/// 加一），不推進個人 bigram 的 tick。
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct EnglishUserWords {
    pub(crate) tick: u64,
    pub(crate) words: BTreeMap<String, EnglishUserWord>,
}

/// 一個英文使用者詞。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct EnglishUserWord {
    /// 最後打的寫法。
    pub(crate) text: String,
    /// 打過的次數，不衰減，出現門檻用；最後一次距今超過一年時當作 0、下一次從 1 重算（同中文的整串
    /// 上屏次數，plan §2.4〈學習的時限〉）。
    pub(crate) count: u64,
    /// 衰減值與最後使用的日期：誰最常用（前綴補完的先後與 20 條）、時限與淘汰用。
    pub(crate) decayed: Decayed,
}

impl EnglishUserWord {
    /// `today` 時算數的次數。
    pub(crate) fn count_on(&self, today: Option<Date>) -> u64 {
        if expired(self.decayed.last_used, today) {
            0
        } else {
            self.count
        }
    }

    /// `today` 時過了出現門檻（一年內打過 [`ENGLISH_SHOWN_AFTER`] 次）沒有。
    pub(crate) fn passed(&self, today: Option<Date>) -> bool {
        self.count_on(today) >= ENGLISH_SHOWN_AFTER
    }
}

/// 英文使用者詞的鍵：不分大小寫，所以是小寫。
pub(crate) fn key(text: &str) -> String {
    text.to_ascii_lowercase()
}

impl EnglishUserWords {
    /// 學一次 `text`（[`to_learn`] 過的）：次數、衰減值加一，tick 加一。
    pub(crate) fn learn(&mut self, text: &str, today: Option<Date>) {
        self.tick += 1;
        let word = self
            .words
            .entry(key(text))
            .or_insert_with(|| EnglishUserWord {
                text: String::new(),
                count: 0,
                decayed: Decayed::default(),
            });
        word.count = word.count_on(today) + 1;
        word.text = text.to_string();
        word.decayed.bump(self.tick, today);
    }

    /// 輸入碼 `input` 剛好是一個英文使用者詞（全是方案字母表 `alphabet` 的字母、不分大小寫），又打過
    /// [`ENGLISH_SHOWN_AFTER`] 次時是它的寫法（排位規則 4）。
    pub(crate) fn typed_as(
        &self,
        input: &str,
        alphabet: &str,
        today: Option<Date>,
    ) -> Option<&str> {
        if !input.chars().all(|c| alphabet.contains(c)) {
            return None;
        }
        self.words
            .get(input)
            .filter(|w| w.passed(today))
            .map(|w| w.text.as_str())
    }

    /// 前綴補完（排位規則 5）：英文詞典沒有（`in_dictionary` 為假）、打過 [`ENGLISH_COMPLETION_AFTER`] 次的
    /// 詞裡最常用的 [`ENGLISH_COMPLETION_POOL`] 條中，打得出的字母（[`letters`]）以 `prefix` 開頭的，
    /// 衰減值大的在前，同值依寫法。詞典有的詞走規則 4 的完整串（與第二期的英文建議列），不佔名額：
    /// 常打的 `that`、`have` 才不會把常寄的 email 擠出去（協調者 2026-11-12 決定）。
    pub(crate) fn completions(
        &self,
        prefix: &str,
        alphabet: &str,
        today: Option<Date>,
        in_dictionary: impl Fn(&str) -> bool,
    ) -> Vec<&str> {
        let mut pool: Vec<(f64, &str)> = self
            .words
            .values()
            .filter(|w| w.count_on(today) >= ENGLISH_COMPLETION_AFTER)
            .filter(|w| !in_dictionary(&w.text))
            .map(|w| (w.decayed.at(self.tick), w.text.as_str()))
            .collect();
        pool.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(b.1)));
        pool.into_iter()
            .take(ENGLISH_COMPLETION_POOL)
            .map(|(_, text)| text)
            .filter(|text| letters(text, alphabet).starts_with(prefix))
            .collect()
    }

    /// 寫法，依小寫排；殼的清單用。
    pub(crate) fn list(&self) -> Vec<String> {
        self.words.values().map(|w| w.text.clone()).collect()
    }

    /// 加進匯入的一個詞，tick 至少到它的最後 tick；不分大小寫已有時不加、回傳 `false`。
    pub(crate) fn insert(&mut self, word: EnglishUserWord) -> bool {
        let key = key(&word.text);
        if self.words.contains_key(&key) {
            return false;
        }
        self.tick = self.tick.max(word.decayed.last_tick());
        self.words.insert(key, word);
        true
    }

    /// 忘掉 `text`（不分大小寫）；沒學過時回傳 `false`。
    pub(crate) fn forget(&mut self, text: &str) -> bool {
        self.words.remove(&key(text)).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_are_learned_without_surrounding_full_stops_and_quotes() {
        for (typed, learned) in [
            ("ivan.test@example.com", "ivan.test@example.com"),
            ("bc12345@example.edu.", "bc12345@example.edu"),
            ("'quoted'", "quoted"),
            ("don't", "don't"),
            ("12345678", "12345678"),
            ("2345-6789", "2345-6789"),
            ("rrrr", "rrrr"),
        ] {
            assert_eq!(to_learn(typed), Some(learned), "{typed}");
        }
    }

    #[test]
    fn short_or_symbol_only_strings_are_not_learned() {
        for typed in ["abc", "abc.", "....", "--__", "a b c d", "台灣台灣", ""] {
            assert_eq!(to_learn(typed), None, "{typed}");
        }
        assert_eq!(to_learn(&"a".repeat(MAX_LEN + 1)), None);
    }

    #[test]
    fn letters_keep_only_the_letters_of_the_alphabet_in_lowercase() {
        let quick = "abcdefghijklmnopqrstuvwxy";
        assert_eq!(
            letters("Ivan.Test@example.com", quick),
            "ivantestexamplecom"
        );
        assert_eq!(letters("bc12345@example.edu", quick), "bcexampleedu");
        assert_eq!(letters("zeta.Z@example.net", quick), "etaexamplenet");
    }
}
