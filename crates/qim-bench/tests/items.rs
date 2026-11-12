//! 題目檔的格式：`bench/fixed.tsv`、`bench/corpus/<名>.tsv`、「有前文」的 `bench/corpus/context/<名>.tsv`
//! 與個人集 `bench/personal/rime-userdb.tsv`。

use qim_bench::{
    Item, TouchSurface, Unit, parse_context, parse_corpus, parse_fixed, parse_personal,
    parse_touch, parse_typo, physical_key, top_char,
};
use qim_core::Key;

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
        typed: None,
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

/// 「含打錯碼」的題（M4 票 08）：`<逐字實際按的鍵><TAB><逐字速成碼><TAB><句子>`，實際按的鍵照實體鍵位轉成按鍵機率。
#[test]
fn typo_reads_typed_keys_per_character_and_allows_an_omitted_character() {
    let items = parse_typo("ra  b\tar a b\t問日月\n").unwrap();
    assert_eq!(items.len(), 1);
    let item = &items[0];
    assert_eq!(item.input(), "arab");
    assert_eq!(item.expected(), "問日月");
    let typed = item.typed.as_ref().unwrap();
    assert_eq!(typed.len(), 3);
    assert_eq!(typed[0], vec![physical_key('r'), physical_key('a')]);
    assert!(typed[1].is_empty(), "漏光的字是空的");
    let tops: Vec<char> = typed[0].iter().filter_map(top_char).collect();
    assert_eq!(tops, ['r', 'a']);
    assert!(
        matches!(typed[0][0], Key::Weighted(_)),
        "實體鍵送按鍵機率，同網頁"
    );
}

#[test]
fn typo_rejects_a_typed_column_that_does_not_match_the_characters() {
    let error = parse_typo("ar\tar a\t問日\n").unwrap_err();
    assert!(error.contains("第 1 行"), "{error}");
    let error = parse_typo("ar a\t問日\n").unwrap_err();
    assert!(error.contains("3 欄"), "{error}");
}

/// 兩個並排的字母鍵 a、b 的最小佈局：觸控的題把點擊經空間模型轉成按鍵機率。
const TWO_KEYS: &str = r#"{
  "format": "qim-layout/1",
  "variants": [{
    "when": { "device": "phone", "orientation": "portrait" },
    "canvas": { "width": 100, "height": 50 },
    "pages": [{ "id": "letters", "keys": [
      { "id": "a", "frame": { "x": 0, "y": 0, "w": 50, "h": 50 }, "tap": { "letter": "a" } },
      { "id": "b", "frame": { "x": 50, "y": 0, "w": 50, "h": 50 }, "tap": { "letter": "b" } }
    ]}]
  }]
}"#;

#[test]
fn touch_reads_taps_and_turns_them_into_key_probabilities() {
    let surface = TouchSurface::from_json(TWO_KEYS, 0.2).unwrap();
    let items = parse_touch("10.0,25.0;60.5,25.0 70.0,25.0\tab b\t問日\n", &surface).unwrap();
    let typed = items[0].typed.as_ref().unwrap();
    let tops: Vec<Vec<char>> = typed
        .iter()
        .map(|unit| unit.iter().filter_map(top_char).collect())
        .collect();
    assert_eq!(tops, [vec!['a', 'b'], vec!['b']]);
    let Key::Weighted(weights) = &typed[0][1] else {
        panic!("觸控送按鍵機率");
    };
    assert!(weights.entries().len() == 2, "鄰鍵 a 也有機率：{weights:?}");
}

#[test]
fn touch_rejects_tap_counts_that_differ_from_the_code() {
    let surface = TouchSurface::from_json(TWO_KEYS, 0.2).unwrap();
    let error = parse_touch("10.0,25.0\tab\t問\n", &surface).unwrap_err();
    assert!(error.contains("第 1 行"), "{error}");
    let error = parse_touch("10.0;25.0\ta\t問\n", &surface).unwrap_err();
    assert!(error.contains("第 1 行"), "{error}");
}

/// 基準照殼的做法送實體鍵（M4 票 08）：小寫字母是實體鍵位的按鍵機率（同網頁的 `pressCode`），分隔記號是標點鍵，
/// 其他照 `Key::for_input`。
#[test]
fn physical_keys_carry_neighbour_probabilities_like_the_web() {
    let Key::Weighted(weights) = physical_key('f') else {
        panic!("字母送按鍵機率");
    };
    assert_eq!(weights.top(), 'f');
    assert!(
        weights.entries().iter().any(|&(c, _)| c == 'g'),
        "{weights:?}"
    );
    assert_eq!(physical_key('\''), Key::Punct('\''));
    assert_eq!(physical_key('A'), Key::Letter('A'));
}
