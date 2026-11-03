use std::collections::BTreeMap;

use qim_databuild::libcangjie;
use qim_databuild::unihan::{self, CrossCheck, Difference};

/// 日：兩邊同碼。丟：三代碼不同。問：只有 libcangjie 有。
/// 乍：一個字分兩列、各有一個三代碼，Unihan 的碼是其中之一。
/// 𠀀：libcangjie 沒有三代碼（只有五代），不算兩邊都有。
const TABLE: &str = "\
日 日 1 1 0 0 1 0 0 0 0 a a NA 23061
丟 丢 1 1 0 0 1 0 0 0 0 hgi mgi,xxmgi NA 22813
問 问 1 1 0 0 1 0 0 0 0 anr anr NA 20001
乍 乍 1 1 0 0 1 0 0 0 0 hs os NA 5000
乍 乍 1 1 0 0 1 0 0 0 0 ohs NA NA 0
𠀀 𠀀 1 0 0 0 0 0 0 0 0 NA a NA 0
";

fn kcangjie(pairs: &[(char, &str)]) -> BTreeMap<char, String> {
    pairs.iter().map(|&(c, code)| (c, code.into())).collect()
}

fn compare(pairs: &[(char, &str)]) -> CrossCheck {
    unihan::compare(&libcangjie::parse(TABLE).unwrap(), &kcangjie(pairs))
}

#[test]
fn same_code_on_both_sides_is_not_a_difference() {
    let check = compare(&[('日', "a")]);
    assert_eq!(check.compared, 1);
    assert_eq!(check.differences, []);
}

#[test]
fn different_code_is_a_difference_listing_every_libcangjie_code() {
    let check = compare(&[('日', "a"), ('丟', "mgi")]);
    assert_eq!(check.compared, 2);
    assert_eq!(
        check.differences,
        [Difference {
            text: '丟',
            unihan: "mgi".into(),
            cangjie3: vec!["hgi".into()],
            cangjie5: vec!["mgi".into(), "xxmgi".into()],
        }]
    );
    assert_eq!(check.matching_cangjie5(), 1);
}

#[test]
fn differences_not_matching_cangjie5_are_counted_apart() {
    let check = compare(&[('日', "b"), ('丟', "mgi")]);
    assert_eq!(check.differences.len(), 2);
    assert_eq!(check.matching_cangjie5(), 1);
}

#[test]
fn characters_on_only_one_side_are_not_compared() {
    // 問只在 libcangjie；㐀只在 Unihan；𠀀在 libcangjie 沒有三代碼。
    let check = compare(&[('㐀', "tm"), ('𠀀', "b")]);
    assert_eq!(check.compared, 0);
    assert_eq!(check.differences, []);
}

#[test]
fn matching_any_code_of_a_character_split_over_rows_is_not_a_difference() {
    let check = compare(&[('乍', "ohs")]);
    assert_eq!((check.compared, check.differences.len()), (1, 0));
}

/// 取自 Unihan 18.0.0 `Unihan_DictionaryLikeData.txt` 的格式：檔頭註解、其他欄位、大寫的碼。
const DICTIONARY_LIKE_DATA: &str = "\
# Unihan_DictionaryLikeData.txt
# Unicode Version 18.0.0
#\tkCangjie
U+3400\tkCangjie\tTM
U+3400\tkFourCornerCode\t5000.7

U+554F\tkGradeLevel\t1
U+65E5\tkCangjie\tA
U+65E5\tkGradeLevel\t1
U+20000\tkCangjie\tMV
U+95CA\tkGradeLevel\t6
";

#[test]
fn parse_takes_kcangjie_lowercased_and_skips_other_fields() {
    assert_eq!(
        unihan::parse(DICTIONARY_LIKE_DATA).unwrap(),
        kcangjie(&[('㐀', "tm"), ('日', "a"), ('𠀀', "mv")])
    );
}

#[test]
fn parse_rejects_malformed_kcangjie_lines() {
    for line in [
        "U+65E5\tkCangjie\tA1",
        "U+65E5\tkCangjie",
        "65E5\tkCangjie\tA",
        "U+D800\tkCangjie\tA",
    ] {
        assert!(unihan::parse(line).is_err(), "{line:?}");
    }
}

#[test]
fn grade_levels_takes_kgradelevel_and_skips_other_fields() {
    assert_eq!(
        unihan::grade_levels(DICTIONARY_LIKE_DATA).unwrap(),
        BTreeMap::from([('問', 1), ('日', 1), ('闊', 6)])
    );
}

#[test]
fn grade_levels_rejects_values_outside_1_to_6() {
    for line in [
        "U+65E5\tkGradeLevel\t0",
        "U+65E5\tkGradeLevel\t7",
        "U+65E5\tkGradeLevel\t1 2",
        "U+65E5\tkGradeLevel",
        "U+D800\tkGradeLevel\t1",
    ] {
        assert!(unihan::grade_levels(line).is_err(), "{line:?}");
    }
}

#[test]
fn read_archive_extracts_dictionary_like_data_from_the_zip() {
    use std::io::Write;
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default();
    zip.start_file("Unihan_Readings.txt", options).unwrap();
    zip.write_all(b"U+65E5\tkCantonese\tjat6\n").unwrap();
    zip.start_file(unihan::ARCHIVE_ENTRY, options).unwrap();
    zip.write_all(DICTIONARY_LIKE_DATA.as_bytes()).unwrap();
    let bytes = zip.finish().unwrap().into_inner();
    assert_eq!(unihan::read_archive(&bytes).unwrap(), DICTIONARY_LIKE_DATA);
    assert!(unihan::read_archive(b"not a zip").is_err());
}

#[test]
fn tsv_has_a_header_and_one_line_per_difference() {
    let check = compare(&[('日', "b"), ('丟', "mgi"), ('乍', "x")]);
    assert_eq!(
        check.to_tsv(),
        "codepoint\tcharacter\tunihan\tlibcangjie_cangjie3\tlibcangjie_cangjie5\n\
         U+4E1F\t丟\tmgi\thgi\tmgi,xxmgi\n\
         U+4E4D\t乍\tx\ths,ohs\tos\n\
         U+65E5\t日\tb\ta\ta\n"
    );
}
