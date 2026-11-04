//! 中文狀態的標點（brief §4.3）：半形鍵 → 全形。

/// 鍵 → 全形標點：rime-prelude `punctuation.yaml` 的 `half_shape` 裡第一個選項是全形的鍵，
/// 只取第一個、不出選單。不轉的鍵與理由見 brief §4.3（引號要配對、`/` `$` 不學 Rime）。
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

/// 數字後保持半形的標點（brief §4.7）：librime `digit_separators` 預設的 `.` `:`，加上使用者
/// 要的 `,`（M1 票 26）。
const DIGIT_SEPARATORS: [char; 3] = ['.', ',', ':'];

/// 前一個上屏的字是半形數字時，`c` 上屏半形。
pub(crate) fn is_digit_separator(c: char) -> bool {
    DIGIT_SEPARATORS.contains(&c)
}
