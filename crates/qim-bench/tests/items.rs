//! 題目檔的格式：`bench/fixed.tsv` 與 `bench/corpus/<名>.tsv`。

use qim_bench::{Item, parse_corpus, parse_fixed};

fn item(input: &str, expected: &str, label: &str) -> Item {
    Item {
        input: input.into(),
        expected: expected.into(),
        label: label.into(),
    }
}

#[test]
fn fixed_reads_three_columns_and_skips_comments_and_blank_lines() {
    let text = "# 輸入碼\t期望首選\t來源\n\nanr\t門口\tRime 案例\n";
    assert_eq!(
        parse_fixed(text).unwrap(),
        vec![item("anr", "門口", "Rime 案例")]
    );
}

#[test]
fn fixed_rejects_a_line_without_three_columns() {
    let error = parse_fixed("a\t日\tM0\nanr\t門口\n").unwrap_err();
    assert!(error.contains("第 2 行"), "{error}");
}

#[test]
fn fixed_rejects_an_empty_expected_text() {
    let error = parse_fixed("a\t\tM0\n").unwrap_err();
    assert!(error.contains("第 1 行"), "{error}");
}

#[test]
fn corpus_joins_per_character_codes_into_one_input() {
    assert_eq!(
        parse_corpus("ha sf fu\t白馬省\n").unwrap(),
        vec![item("hasffu", "白馬省", "第 1 行")]
    );
}

#[test]
fn corpus_rejects_a_line_without_a_tab() {
    let error = parse_corpus("a\t日\nha sf fu 白馬省\n").unwrap_err();
    assert!(error.contains("第 2 行"), "{error}");
}
