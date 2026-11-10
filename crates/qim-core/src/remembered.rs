//! 記住的字串（plan §2.4，M3 票 15）：使用者經 QIM 打出的 email，之後在中文組字打它開頭的字母時給候選。
//! 這裡是判斷字串的規則與正在打的一串（[`TypedRun`]）；記在 [`crate::UserModel`]，由 [`crate::Session`]
//! 從送出去的字裡認出來。

/// 一個記住的字串最多幾個位元組：email 地址的上限（RFC 5321 的 path 上限 256 減去角括號）。
pub(crate) const MAX_LEN: usize = 254;

/// 打它開頭的字母時，候選列最多接幾個記住的字串（同 emoji 的 [`crate::EMOJI_LIMIT`]）。
pub const REMEMBERED_LIMIT: usize = 3;

/// 可以出現在 email 裡的字元（RFC 5322 dot-atom 常用的子集，同 HTML `type=email` 的寬鬆版）：
/// 英文字母、數字與 `. _ % + - @`。送出去的字遇到其他字元，前面的就是打完的一串。
pub(crate) fn is_email_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '%' | '+' | '-' | '@')
}

/// 正在打的一串（plan §2.1〈記住的字串〉）：送出去的字的結尾連續的 email 字元（[`is_email_char`]），
/// 遇到其他字元就是打完了。開頭不是使用者經 QIM 打的（接在輸入框原有的 email 字元後面、退格刪到它前面、
/// 有一段是無痕或學習關掉時打的、長過上限）時是 `partial`：打完也不記，到下一個分隔的字元重新開始。
#[derive(Debug, Clone, Default)]
pub(crate) struct TypedRun {
    text: String,
    partial: bool,
}

impl TypedRun {
    /// 送出去一個字元。email 字元接上（`recording` 為假時不接、標成 `partial`）；其他字元讓這串打完，
    /// 回傳要記住的 email（[`TypedRun::finish`]）。
    pub(crate) fn push(&mut self, c: char, recording: bool) -> Option<String> {
        if !is_email_char(c) {
            return self.finish();
        }
        if recording && self.text.len() < MAX_LEN + 1 {
            self.text.push(c);
        } else {
            // 長過上限的已不是 email；之後退格留下的也不是輸入框裡的樣子。
            self.partial = true;
        }
        None
    }

    /// 打完了：整串都看得到、去掉結尾的 `.` 後是 email 時回傳它（[`to_remember`]），然後重新開始。
    pub(crate) fn finish(&mut self) -> Option<String> {
        let done = (!self.partial)
            .then(|| to_remember(&self.text).map(String::from))
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
    /// 最後一字還是 email 字元時，這串的開頭不是使用者經 QIM 打的。
    pub(crate) fn sync(&mut self, before: &str) {
        let rest = match before.strip_suffix(self.text.as_str()) {
            Some(rest) => rest,
            None => {
                self.text.clear();
                before
            }
        };
        self.partial = rest.ends_with(is_email_char);
    }
}

/// 一串打完的字裡要記住的部分：去掉結尾的 `.`（句尾的句點）後是 email 時是它，否則是 `None`。
pub(crate) fn to_remember(typed: &str) -> Option<&str> {
    let text = typed.trim_end_matches('.');
    is_email(text).then_some(text)
}

/// `local@domain`：本地部分是 [`is_email_char`] 除 `@` 以外的字元，不以 `.` 開頭或結尾、沒有連續的
/// `..`；網域至少兩段，每段是英文字母、數字與 `-`，不以 `-` 開頭或結尾，最後一段至少兩個英文字母。
/// 照 HTML `type=email` 的規則再要求頂層網域，`ivan@localhost` 這類不記。
fn is_email(text: &str) -> bool {
    if text.len() > MAX_LEN {
        return false;
    }
    let Some((local, domain)) = text.split_once('@') else {
        return false;
    };
    let local_ok = !local.is_empty()
        && !local.starts_with('.')
        && !local.ends_with('.')
        && !local.contains("..")
        && local.chars().all(|c| is_email_char(c) && c != '@');
    let labels: Vec<&str> = domain.split('.').collect();
    let label_ok = |label: &&str| {
        !label.is_empty()
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    };
    let top_ok = labels
        .last()
        .is_some_and(|top| top.len() >= 2 && top.chars().all(|c| c.is_ascii_alphabetic()));
    local_ok && labels.len() >= 2 && labels.iter().all(label_ok) && top_ok
}

/// 中文組字裡打得出的部分：字串的英文字母轉小寫後，方案字母表 `alphabet` 裡有的；其他字元去掉
/// （組字裡打不出 `.`、`@`、數字與速成沒有的 `z`）。輸入碼是它的開頭時給這個字串：
/// `ivan.test@example.com` 打 `iva`、`ivant` 都有，`zz98765@example.org` 打 `exa` 起有。
pub(crate) fn letters(text: &str, alphabet: &str) -> String {
    text.chars()
        .map(|c| c.to_ascii_lowercase())
        .filter(|&c| c.is_ascii_lowercase() && alphabet.contains(c))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emails_are_remembered_without_a_trailing_full_stop() {
        assert_eq!(
            to_remember("ivan.test@example.com"),
            Some("ivan.test@example.com")
        );
        assert_eq!(
            to_remember("bc12345@example.edu."),
            Some("bc12345@example.edu")
        );
        assert_eq!(
            to_remember("a+b_c%d-e@mail.example.co.uk"),
            Some("a+b_c%d-e@mail.example.co.uk")
        );
    }

    #[test]
    fn strings_that_are_not_emails_are_not_remembered() {
        for typed in [
            "ivan",
            "example.com",
            "ivan@localhost",
            "ivan@example.c",
            "ivan@example.123",
            "@example.com",
            "ivan@",
            ".ivan@example.com",
            "iv..an@example.com",
            "ivan@@example.com",
            "ivan@-example.com",
            "ivan@example..com",
            "",
        ] {
            assert_eq!(to_remember(typed), None, "{typed}");
        }
        let long = format!("{}@example.com", "a".repeat(MAX_LEN));
        assert_eq!(to_remember(&long), None);
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
