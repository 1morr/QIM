//! 中文狀態的標點（brief〈桌面鍵盤操作〉）：半形鍵 → 全形。

/// 鍵 → 全形標點：rime-prelude `punctuation.yaml` 的 `half_shape` 裡第一個選項是全形的鍵，
/// 只取第一個、不出選單。不轉的鍵與理由見 brief〈桌面鍵盤操作〉（引號要配對、`/` `$` 不學 Rime）。
const FULL_WIDTH: [(char, &str); 17] = [
    (',', "，"),
    ('.', "。"),
    ('?', "？"),
    ('!', "！"),
    (';', "；"),
    (':', "："),
    ('(', "（"),
    (')', "）"),
    ('[', "「"),
    (']', "」"),
    ('{', "『"),
    ('}', "』"),
    ('<', "《"),
    ('>', "》"),
    ('\\', "、"),
    ('^', "……"),
    ('_', "——"),
];

/// `c` 的全形標點；沒有就是 `None`。
pub(crate) fn full_width(c: char) -> Option<&'static str> {
    FULL_WIDTH
        .iter()
        .find(|(key, _)| *key == c)
        .map(|(_, p)| *p)
}

/// 數字後上屏半形的標點（brief〈英文模式與欄位類型〉）：同 librime `digit_separators` 的預設 `.` `:`。逗號照舊全形：
/// 中文裡數字後的逗號多半是句子的逗號（「共3，」，使用者決定，M1 票 26）。
const DIGIT_SEPARATORS: [char; 2] = ['.', ':'];

/// 前文以半形數字結尾時，`c` 上屏半形（`Session` 判斷前文）。
pub(crate) fn is_digit_separator(c: char) -> bool {
    DIGIT_SEPARATORS.contains(&c)
}

/// 英文模式連按兩下空白輸出句點（brief〈英文模式與欄位類型〉，M4 票 27）要的游標前狀態：只記布林值、不記文字，
/// 無痕時也照記（同數字後半形）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct SpaceTail {
    /// 最後一字後面可以接句點（[`can_precede_period`]）。
    last_takes_period: bool,
    /// 游標前是一個空白，空白前的字可以接句點：再按一下空白就換成「. 」。
    pub(crate) after_space: bool,
}

impl SpaceTail {
    /// 接著打出 `text` 之後的狀態。
    pub(crate) fn after(self, text: &str) -> Self {
        text.chars().fold(self, |tail, c| Self {
            last_takes_period: can_precede_period(c),
            after_space: c == ' ' && tail.last_takes_period,
        })
    }
}

/// 連按兩下空白時，第一個空白前是 `c` 才換成句點：照 LatinIME 的 `canBeFollowedByDoubleSpacePeriod`（事實 M52）
/// 是字母、數字，`'` `"` `)` `]` `}` `>` `+` `%`。LatinIME 另收 Unicode 的「其他符號」（emoji 這類），Rust 標準庫
/// 沒有這個分類，不收。
fn can_precede_period(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '\'' | '"' | ')' | ']' | '}' | '>' | '+' | '%')
}

/// 中文狀態的 `"` `'` 左右輪流（[`crate::Options::paired_quotes`]，使用者定，M4 票 27）：照 librime 的 `pair`
/// （rime-prelude 的 “” ‘’，事實 R12、M53），每個鍵各記下一下該出哪半邊；換欄位時從左邊重來（fcitx5，事實 M53）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Quotes {
    /// `"`、`'` 下一下出右半邊。
    closing: [bool; 2],
}

impl Quotes {
    /// 按了 `c` 要上屏的引號，並換到另一半邊；`c` 不是 `"` `'` 時是 `None`。
    pub(crate) fn next(&mut self, c: char) -> Option<&'static str> {
        let (at, pair) = match c {
            '"' => (0, ["“", "”"]),
            '\'' => (1, ["‘", "’"]),
            _ => return None,
        };
        let quote = pair[usize::from(self.closing[at])];
        self.closing[at] = !self.closing[at];
        Some(quote)
    }
}
