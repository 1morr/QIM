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
