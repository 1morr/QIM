use std::collections::BTreeMap;

use qim_databuild::{libcangjie, quick};

/// 前 9 列照抄 libcangjie `346026b` 的 `data/table.txt`；後面幾列是排序用的假資料。
const TABLE: &str = "\
# See the README.table.md file in this folder for details about this data.
日 日 1 1 0 0 1 0 0 0 0 a a NA 23061
問 问 1 1 0 0 1 0 0 0 0 anr anr NA 20001
丟 丢 1 1 0 0 1 0 0 0 0 hgi mgi,xxmgi NA 22813
乍 乍 1 1 0 0 1 0 0 0 0 hs os NA 22969
乍 乍 1 1 0 0 1 0 0 0 0 os NA NA 0
嘅 嘅 1 0 1 0 1 0 0 0 0 rhpu rhpu NA 0
嘅 嘅 1 0 1 0 1 0 0 0 0 raiu raiu NA 0
° NA 0 0 0 0 0 0 0 0 1 yyycg yyycg NA 0
門 门 1 1 0 0 1 0 0 0 0 an an NA 20000
曰 曰 1 1 0 0 1 0 0 0 0 a a NA 20002
㗎 㗎 1 0 1 0 0 0 0 0 0 NA a NA 0
㐃 㐃 1 0 1 0 0 0 0 0 0 a NA NA 0
𠀉 𠀉 1 0 0 0 0 0 0 0 0 NA a NA 0
𠀀 𠀀 1 0 0 0 0 0 0 0 0 a NA NA 0
";

fn table() -> qim_core::CodeTable {
    table_with(&BTreeMap::new())
}

/// `grades` 是 Unihan `kGradeLevel`：字到香港小學的年級。
fn table_with(grades: &BTreeMap<char, u8>) -> qim_core::CodeTable {
    quick::code_table(&libcangjie::parse(TABLE).unwrap(), grades)
}

fn texts(table: &qim_core::CodeTable, code: &str) -> String {
    table.lookup(code).iter().map(|e| e.text).collect()
}

fn codes_of(table: &qim_core::CodeTable, ch: char) -> Vec<String> {
    table
        .iter()
        .filter(|(_, entries)| entries.iter().any(|e| e.text == ch))
        .map(|(code, _)| code.to_string())
        .collect()
}

#[test]
fn quick_code_is_first_and_last_letter_of_cangjie_code() {
    let table = table();
    assert_eq!(codes_of(&table, '問'), ["ar"]);
    assert!(!texts(&table, "anr").contains('問'));
}

#[test]
fn one_and_two_letter_cangjie_codes_stay_as_they_are() {
    let table = table();
    assert_eq!(codes_of(&table, '日'), ["a"]);
    assert_eq!(codes_of(&table, '門'), ["an"]);
}

#[test]
fn quick_codes_from_every_cangjie3_and_cangjie5_code_are_unioned() {
    let table = table();
    assert_eq!(codes_of(&table, '丟'), ["hi", "mi", "xi"]);
    // 同一個字分在兩列，兩列的碼都收。
    assert_eq!(codes_of(&table, '乍'), ["hs", "os"]);
    // 推導出同一個碼時只收一次：嘅的 rhpu、raiu 都是 ru；問的三代、五代都是 ar。
    assert_eq!(codes_of(&table, '嘅'), ["ru"]);
    assert_eq!(texts(&table, "ar"), "問");
}

#[test]
fn characters_that_are_not_hanzi_are_left_out() {
    assert_eq!(codes_of(&table(), '°'), Vec::<String>::new());
}

#[test]
fn big5_order_first_then_other_hkscs_then_extension_by_code_point() {
    let table = table();
    assert_eq!(texts(&table, "a"), "曰日㐃㗎𠀀𠀉");
    let extension: Vec<bool> = table.lookup("a").iter().map(|e| e.extension).collect();
    assert_eq!(extension, [false, false, false, false, true, true]);
}

#[test]
fn characters_with_a_grade_level_come_first_by_grade_then_the_rest_by_big5_order() {
    // 日、㗎有年級，排在沒年級的曰之前，雖然曰的 Big5 排序值較小、㗎沒有排序值。
    // 𠀉在擴展區，有年級也只在擴展區內提前。
    let table = table_with(&BTreeMap::from([('㗎', 2), ('日', 1), ('𠀉', 3)]));
    assert_eq!(texts(&table, "a"), "日㗎曰㐃𠀉𠀀");
    let extension: Vec<bool> = table.lookup("a").iter().map(|e| e.extension).collect();
    assert_eq!(extension, [false, false, false, false, true, true]);
}

#[test]
fn lower_grade_comes_first_and_same_grade_falls_back_to_big5_order() {
    let table = table_with(&BTreeMap::from([('日', 2), ('曰', 2), ('㐃', 1)]));
    assert_eq!(texts(&table, "a"), "㐃曰日㗎𠀀𠀉");
}

#[test]
fn malformed_line_is_reported_with_its_line_number() {
    let err = libcangjie::parse("# header\n日 日 1 1 0\n").unwrap_err();
    assert!(err.to_string().contains("第 2 行"), "{err}");
}
