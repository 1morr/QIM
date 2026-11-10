//! 題目檔的格式：`bench/fixed.tsv`、`bench/corpus/<名>.tsv`、「有前文」的 `bench/corpus/context/<名>.tsv`
//! 與個人集 `bench/personal/rime-userdb.tsv`。

use qim_bench::{Item, Unit, parse_context, parse_corpus, parse_fixed, parse_personal};

fn unit(code: &str, text: &str) -> Unit {
    Unit {
        code: code.into(),
        text: text.into(),
    }
}

fn item(units: Vec<Unit>, line: usize, label: &str, weight: u64) -> Item {
    Item {
        context: None,
        units,
        line,
        label: label.into(),
        weight,
    }
}

#[test]
fn fixed_reads_three_columns_and_skips_comments_and_blank_lines() {
    let text = "# 輸入碼\t期望首選\t來源\n\nanr\t門口\tRime 案例\n";
    let items = parse_fixed(text).unwrap();
    // 固定題不知道每個字用了幾個鍵：整題是一個單位。
    assert_eq!(
        items,
        vec![item(vec![unit("anr", "門口")], 3, "Rime 案例", 1)]
    );
    assert_eq!(items[0].input(), "anr");
    assert_eq!(items[0].expected(), "門口");
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
fn corpus_keeps_one_unit_per_character() {
    let items = parse_corpus("ha sf fu\t白馬省\n").unwrap();
    assert_eq!(
        items,
        vec![item(
            vec![unit("ha", "白"), unit("sf", "馬"), unit("fu", "省")],
            1,
            "第 1 行",
            1
        )]
    );
    assert_eq!(items[0].input(), "hasffu");
    assert_eq!(items[0].expected(), "白馬省");
}

#[test]
fn corpus_rejects_a_line_without_a_tab() {
    let error = parse_corpus("a\t日\nha sf fu 白馬省\n").unwrap_err();
    assert!(error.contains("第 2 行"), "{error}");
}

#[test]
fn corpus_rejects_codes_that_do_not_match_the_characters() {
    let error = parse_corpus("ha sf\t白馬省\n").unwrap_err();
    assert!(error.contains("第 1 行"), "{error}");
}

#[test]
fn personal_reads_the_commit_count_as_the_weight() {
    let items = parse_personal("an r\t門口\t12\n").unwrap();
    assert_eq!(
        items,
        vec![item(
            vec![unit("an", "門"), unit("r", "口")],
            1,
            "第 1 行",
            12
        )]
    );
}

#[test]
fn personal_errors_name_the_line_but_not_the_word() {
    let error = parse_personal("an r\t門口\t12\nan r\t門口\tx\n").unwrap_err();
    assert!(error.contains("第 2 行"), "{error}");
    assert!(
        !error.contains("門口"),
        "個人集的內容不能出現在訊息裡：{error}"
    );
}

#[test]
fn context_reads_the_context_before_the_codes_and_sentence() {
    let items = parse_context("今日，ATM\tar a\t問日\n").unwrap();
    assert_eq!(
        items,
        [Item {
            context: Some("今日，ATM".into()),
            ..item(vec![unit("ar", "問"), unit("a", "日")], 1, "第 1 行", 1)
        }]
    );
}

#[test]
fn context_rejects_an_empty_context_or_a_missing_column() {
    assert!(parse_context("\tar\t問\n").unwrap_err().contains("第 1 行"));
    assert!(parse_context("ar\t問\n").unwrap_err().contains("3 欄"));
}
