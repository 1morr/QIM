use std::collections::BTreeMap;

use qim_databuild::words_hk;

/// 照上游檔的樣子：CRLF、首行是 `Source,<頁面網址>`、讀音可以有好幾欄。
const WORDSLIST: &str = "Source,https://words.hk/faiman/analysis/wordslist/\r\n\
%,pat6 sen1\r\n\
千祈,cin1 kei4\r\n\
千祈*,cin1 kei4\r\n\
BB床*,bi4 bi1 cong4\r\n\
作實,zok3 sat6\r\n\
襟,kam1\r\n\
真係,zan1 hai6\r\n";

const WORD_COUNT: &str = "Source,https://words.hk/faiman/analysis/existingwordcount/\r\n\
真係,6365\r\n\
千祈,12\r\n\
襟,3\r\n\
冇收嘅詞,9\r\n";

fn headwords(wordslist: &str, word_count: &str) -> BTreeMap<String, u64> {
    words_hk::headwords(wordslist, word_count).unwrap()
}

#[test]
fn headwords_of_two_or_more_characters_get_their_count_or_zero() {
    let words = headwords(WORDSLIST, WORD_COUNT);
    assert_eq!(
        words,
        BTreeMap::from([
            ("BB床".to_string(), 0),
            ("作實".to_string(), 0),
            ("千祈".to_string(), 12),
            ("真係".to_string(), 6365),
        ])
    );
}

#[test]
fn the_variant_mark_is_dropped_and_the_word_counted_once() {
    let words = headwords(
        "Source,x\r\n千祈*,a\r\n千祈,b\r\n",
        "Source,y\r\n千祈,5\r\n",
    );
    assert_eq!(words, BTreeMap::from([("千祈".to_string(), 5)]));
}

#[test]
fn a_file_without_the_source_line_or_with_a_bad_count_is_an_error() {
    for (wordslist, word_count, file) in [
        ("千祈,cin1\r\n", "Source,y\r\n", words_hk::WORDSLIST),
        ("Source,x\r\n", "千祈,5\r\n", words_hk::WORD_COUNT),
        (
            "Source,x\r\n",
            "Source,y\r\n千祈,many\r\n",
            words_hk::WORD_COUNT,
        ),
        ("Source,x\r\n", "Source,y\r\n千祈\r\n", words_hk::WORD_COUNT),
        ("Source,x\r\n,cin1\r\n", "Source,y\r\n", words_hk::WORDSLIST),
    ] {
        let err = words_hk::headwords(wordslist, word_count).unwrap_err();
        assert!(
            err.to_string().contains(file),
            "{wordslist:?} {word_count:?}：{err}"
        );
    }
}
