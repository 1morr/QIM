use std::collections::BTreeMap;

use qim_core::{CodeEntry, CodeTable};
use qim_databuild::common_forms::{BothForms, CommonForms, ONE_WAY_FORMS, TWO_WAY_FORMS};

/// `(輸入碼, 字, 是否擴展區)` 組成的碼表。
fn table(entries: &[(&str, char, bool)]) -> CodeTable {
    let mut codes: BTreeMap<String, Vec<CodeEntry>> = BTreeMap::new();
    for &(code, text, extension) in entries {
        codes.entry(code.into()).or_default().push(CodeEntry {
            text,
            extension,
            simplified: false,
        });
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
    CommonForms::new(
        &sample(),
        lookup(to_hong_kong),
        lookup(to_taiwan),
        BothForms::default(),
    )
    .unwrap()
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
    let error = CommonForms::new(
        &table,
        lookup(&[('溫', '温'), ('榲', '温')]),
        lookup(&[]),
        BothForms::default(),
    )
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
    let error = CommonForms::new(
        &table,
        lookup(&[('溫', '温')]),
        lookup(&[('溫', '榲')]),
        BothForms::default(),
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("一次換不完"), "{error}");
}

/// `sample()` 再加上 [`TWO_WAY_FORMS`] 的字，每個字各一個碼（兩種寫法不同碼）。
fn sample_with_both_forms() -> CodeTable {
    let mut entries: Vec<(String, char, bool)> = Vec::new();
    for (code, entries_at) in sample().iter() {
        for e in entries_at {
            entries.push((code.to_string(), e.text, e.extension));
        }
    }
    for (i, (representative, other)) in TWO_WAY_FORMS.iter().chain(&ONE_WAY_FORMS).enumerate() {
        entries.push((format!("x{i}"), *representative, false));
        entries.push((format!("y{i}"), *other, false));
    }
    let entries: Vec<(&str, char, bool)> = entries
        .iter()
        .map(|(c, t, e)| (c.as_str(), *t, *e))
        .collect();
    table(&entries)
}

#[test]
fn from_opencc_reads_the_dictionaries_the_corpus_conversion_uses() {
    // ferrous-opencc 內建的 t2hk、t2tw，就是前處理轉語料用的那一套；並存寫法是 TWO_WAY_FORMS、
    // ONE_WAY_FORMS。
    let forms = CommonForms::from_opencc(&sample_with_both_forms()).unwrap();
    assert_eq!(forms.apply("温羣説"), "溫群說");
    assert_eq!(forms.apply("為這裡的著名"), "為這裏的著名");
    assert_eq!(forms.both().two_way_pairs().count(), TWO_WAY_FORMS.len());
    assert_eq!(forms.both().one_way_pairs().count(), ONE_WAY_FORMS.len());
}

#[test]
fn both_form_pairs_missing_from_the_table_are_skipped() {
    // 小碼表只有「戶／户」這組的字。
    let forms = CommonForms::from_opencc(&sample()).unwrap();
    assert_eq!(
        forms.both().two_way_pairs().collect::<Vec<_>>(),
        [('戶', '户')]
    );
    assert_eq!(forms.apply("户口"), "戶口");
}

#[test]
fn a_both_form_outside_the_continuous_set_fails() {
    // 着在連打字集、著只在擴展區。
    let table = table(&[("tu", '着', false), ("ta", '著', true)]);
    let error = CommonForms::new(
        &table,
        lookup(&[]),
        lookup(&[]),
        BothForms::new(&[], &[('着', '著')]).unwrap(),
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("「著」不在連打字集"), "{error}");
}

/// 裏（`yv`）、裡（`lg`）雙向，代表寫法是裏；着（`tu`）→ 著（`ta`）單向。
fn with_zhe() -> CommonForms {
    let table = table(&[
        ("tu", '着', false),
        ("ta", '著', false),
        ("yv", '裏', false),
        ("lg", '裡', false),
        ("sq", '群', false),
        ("sq", '羣', false),
    ]);
    CommonForms::new(&table, lookup(&[]), lookup(&[('羣', '群')]), both()).unwrap()
}

fn both() -> BothForms {
    BothForms::new(&[('裏', '裡')], &[('着', '著')]).unwrap()
}

#[test]
fn both_forms_count_under_the_representative() {
    let forms = with_zhe();
    // 單向的不換：「著名」的著不能寫成着，語料裡兩字各算各的。
    assert_eq!(forms.apply("隨著人羣在這裡"), "隨著人群在這裏");
    assert_eq!(forms.apply("隨着"), "隨着");
    // 對照表的組不含並存寫法：`--verify` 用它查「換掉的寫法」。
    assert_eq!(forms.pairs().collect::<Vec<_>>(), [('羣', '群')]);
}

#[test]
fn spellings_cover_every_mix_of_both_forms() {
    let both = both();
    assert_eq!(both.spellings("群組"), ["群組"]);
    assert_eq!(both.spellings("這裏"), ["這裏", "這裡"]);
    assert_eq!(both.spellings("隨着"), ["隨着", "隨著"]);
    assert_eq!(both.spellings("着裏"), ["着裏", "着裡", "著裏", "著裡"]);
}

#[test]
fn reversed_one_way_words_are_corrected() {
    // rime-essay 原有的「顯着」改回「顯著」；「隨着」照舊。
    let both = BothForms::new(&[], &ONE_WAY_FORMS).unwrap();
    assert_eq!(both.correct("顯着"), "顯著");
    assert_eq!(both.correct("隨着"), "隨着");
}

#[test]
fn one_way_forms_are_never_spelled_backwards() {
    assert_eq!(both().spellings("著名"), ["著名"]);
    assert_eq!(both().spellings("這裡"), ["這裡"]);
}

#[test]
fn variants_t2hk_replaces_in_the_lexicon_are_spelled_both_ways() {
    // t2hk 把 rime-essay 的「鉤溼擡糉柺囪」換成右邊的寫法，兩字不同碼：打哪個碼都要出得了詞（票 17d）。
    let both = BothForms::new(&TWO_WAY_FORMS, &ONE_WAY_FORMS).unwrap();
    for (written, other) in [
        ("鈎子", "鉤子"),
        ("濕地", "溼地"),
        ("抬頭", "擡頭"),
        ("粽子", "糉子"),
        ("枴杖", "柺杖"),
        ("煙囱", "煙囪"),
    ] {
        assert_eq!(both.spellings(written), [written, other]);
    }
}

#[test]
fn a_one_way_word_whose_other_spelling_is_another_word_keeps_one_spelling() {
    // 「合着」寫成「合著」就是合寫的書，義項不同：不生出這個寫法。「刻着／刻著」是同一個詞，照生。
    let both = BothForms::new(&[], &ONE_WAY_FORMS).unwrap();
    assert_eq!(both.spellings("合着"), ["合着"]);
    assert_eq!(both.spellings("刻着"), ["刻着", "刻著"]);
}

#[test]
fn one_way_forms_are_put_back_as_written_only_when_the_conversion_keeps_the_length() {
    let both = BothForms::new(&[], &ONE_WAY_FORMS).unwrap();
    // OpenCC 換掉的「臺」放回去，其他換掉的字照轉換結果。
    assert_eq!(both.as_written("臺灣羣", "台灣群"), "臺灣群");
    // 字數變了就對不上位置：照轉換結果，不放回。
    assert_eq!(both.as_written("臺灣", "台灣省"), "台灣省");
}

#[test]
fn both_forms_share_the_summed_frequency() {
    // 單向的「着／著」各算各的。
    let mut frequency = BTreeMap::from([('裏', 70), ('裡', 30), ('着', 8), ('著', 9), ('群', 5)]);
    both().share_frequency(&mut frequency);
    assert_eq!(
        frequency,
        BTreeMap::from([('裏', 100), ('裡', 100), ('着', 8), ('著', 9), ('群', 5)])
    );
}

#[test]
fn a_character_in_two_both_form_pairs_fails() {
    let error = BothForms::new(&[('着', '著')], &[('著', '箸')])
        .unwrap_err()
        .to_string();
    assert!(error.contains("重複"), "{error}");
}

#[test]
fn a_both_form_that_is_also_in_the_table_fails() {
    // 羣→群在對照表，群又是並存寫法：換不出一致的寫法。
    let table = table(&[
        ("sq", '群', false),
        ("sq", '羣', false),
        ("xx", '帬', false),
    ]);
    let error = CommonForms::new(
        &table,
        lookup(&[]),
        lookup(&[('羣', '群')]),
        BothForms::new(&[('群', '帬')], &[]).unwrap(),
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("也在字形對照表"), "{error}");
}
