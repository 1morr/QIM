use std::collections::BTreeMap;

use qim_core::{CodeEntry, CodeTable};
use qim_databuild::common_forms::CommonForms;

/// `(輸入碼, 字, 是否擴展區)` 組成的碼表。
fn table(entries: &[(&str, char, bool)]) -> CodeTable {
    let mut codes: BTreeMap<String, Vec<CodeEntry>> = BTreeMap::new();
    for &(code, text, extension) in entries {
        codes
            .entry(code.into())
            .or_default()
            .push(CodeEntry { text, extension });
    }
    CodeTable::from_entries(codes)
}

/// 連打字集：溫温、群羣同碼，戶户、為不同碼；説、爲只在擴展區。
fn sample() -> CodeTable {
    table(&[
        ("et", '溫', false),
        ("et", '温', false),
        ("sq", '群', false),
        ("sq", '羣', false),
        ("hs", '戶', false),
        ("is", '户', false),
        ("if", '為', false),
        ("bf", '爲', true),
        ("yu", '說', false),
        ("yu", '説', true),
    ])
}

/// 照 `pairs` 一個字換一個字，其餘不變：代替 OpenCC 的 `t2hk` 或 `t2tw`。
fn lookup(pairs: &[(char, char)]) -> impl Fn(char) -> char {
    let map: BTreeMap<char, char> = pairs.iter().copied().collect();
    move |c| map.get(&c).copied().unwrap_or(c)
}

fn forms(to_hong_kong: &[(char, char)], to_taiwan: &[(char, char)]) -> CommonForms {
    CommonForms::new(&sample(), lookup(to_hong_kong), lookup(to_taiwan)).unwrap()
}

#[test]
fn hong_kong_form_with_the_same_code_maps_back_to_the_common_form() {
    let forms = forms(&[('溫', '温')], &[]);
    assert_eq!(forms.pairs().collect::<Vec<_>>(), [('温', '溫')]);
    assert_eq!(forms.apply("温書"), "溫書");
}

#[test]
fn hong_kong_form_outside_the_continuous_set_maps_back_even_with_another_code() {
    // 不換回來，含「説」的句子在前處理會整句丟掉。
    assert_eq!(forms(&[('說', '説')], &[]).apply("佢説"), "佢說");
}

#[test]
fn opencc_standard_form_maps_to_the_taiwan_form_with_the_same_code() {
    // s2hk 的簡轉繁把「群」轉成 OpenCC 標準的「羣」；t2tw 把「羣」轉成「群」。
    assert_eq!(forms(&[], &[('羣', '群')]).apply("人羣"), "人群");
}

#[test]
fn pairs_with_different_codes_in_the_continuous_set_are_left_alone() {
    // 戶、户不同碼，打的時候不互搶。
    let forms = forms(&[('戶', '户')], &[]);
    assert_eq!(forms.pairs().count(), 0);
    assert_eq!(forms.apply("户口"), "户口");
}

#[test]
fn common_form_outside_the_continuous_set_is_never_a_target() {
    // OpenCC 的標準寫法「爲」不在連打字集，「為」不能換成它。
    assert_eq!(forms(&[('爲', '為')], &[]).apply("為"), "為");
}

#[test]
fn a_form_mapped_two_ways_fails() {
    let table = table(&[
        ("et", '溫', false),
        ("et", '温', false),
        ("et", '榲', false),
    ]);
    let error = CommonForms::new(&table, lookup(&[('溫', '温'), ('榲', '温')]), lookup(&[]))
        .unwrap_err()
        .to_string();
    assert!(error.contains("同時要換成"), "{error}");
}

#[test]
fn a_chain_of_forms_fails() {
    // 温 → 溫，溫又要換成榲：一次換不完。
    let table = table(&[
        ("et", '溫', false),
        ("et", '温', false),
        ("et", '榲', false),
    ]);
    let error = CommonForms::new(&table, lookup(&[('溫', '温')]), lookup(&[('溫', '榲')]))
        .unwrap_err()
        .to_string();
    assert!(error.contains("一次換不完"), "{error}");
}

#[test]
fn from_opencc_reads_the_dictionaries_the_corpus_conversion_uses() {
    // ferrous-opencc 內建的 t2hk、t2tw，就是前處理轉語料用的那一套。
    let forms = CommonForms::from_opencc(&sample()).unwrap();
    assert_eq!(forms.apply("温羣説"), "溫群說");
    assert_eq!(forms.apply("户為"), "户為");
}
