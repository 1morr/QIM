//! 字根提示（plan〈輸入方案〉 `radical_hint`、§4.1 步驟 5）：字 → 倉頡三代 / 五代碼，只給提示，不進 lattice。

use std::collections::BTreeMap;

use qim_core::{
    CangjieCodes, CangjieVersion, Pack, PackError, Packs, RadicalHint, RadicalHintPack,
    RadicalHints, Ranking, Scheme,
};

fn codes(cangjie3: &[&str], cangjie5: &[&str]) -> CangjieCodes {
    let owned = |codes: &[&str]| codes.iter().map(|c| c.to_string()).collect();
    CangjieCodes {
        cangjie3: owned(cangjie3),
        cangjie5: owned(cangjie5),
    }
}

fn sample() -> RadicalHintPack {
    RadicalHintPack {
        id: "cangjie-hint".into(),
        hints: RadicalHints::new([
            ('門', codes(&["an"], &["an"])),
            ('口', codes(&["r"], &["r"])),
            ('丟', codes(&["hgi"], &["mgi", "xxmgi"])),
            ('乃', codes(&["nsh"], &[])),
        ]),
    }
}

fn scheme(radical_hint: Option<&str>) -> Scheme {
    Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        wildcard: None,
        radicals: BTreeMap::from([
            ('a', "日".into()),
            ('n', "弓".into()),
            ('r', "口".into()),
            ('h', "竹".into()),
            ('g', "土".into()),
            ('i', "戈".into()),
            ('m', "一".into()),
        ]),
        packs: Packs {
            radical_hint: radical_hint.map(Into::into),
            ..Packs::default()
        },
        ranking: Ranking::default(),
    }
}

/// `codes` 是（碼, 字根）。
fn hint(text: char, codes: &[(&str, &str)]) -> RadicalHint {
    RadicalHint {
        text,
        codes: codes.iter().map(|(c, _)| c.to_string()).collect(),
        radicals: codes.iter().map(|(_, r)| r.to_string()).collect(),
    }
}

#[test]
fn hint_lists_each_character_with_its_codes_of_the_chosen_version_and_their_radicals() {
    let hints = sample().hints;
    let s = scheme(None);
    assert_eq!(
        hints.hint(&s, "門丟", CangjieVersion::Cangjie5),
        [
            hint('門', &[("an", "日弓")]),
            // 方案沒有字根的鍵（`x`）顯示原字母。
            hint('丟', &[("mgi", "一土戈"), ("xxmgi", "xx一土戈")]),
        ]
    );
    assert_eq!(
        hints.hint(&s, "丟", CangjieVersion::Cangjie3),
        [hint('丟', &[("hgi", "竹土戈")])]
    );
}

#[test]
fn characters_without_codes_of_that_version_have_an_empty_hint() {
    let hints = sample().hints;
    assert_eq!(
        hints.hint(&scheme(None), "乃a", CangjieVersion::Cangjie5),
        [hint('乃', &[]), hint('a', &[])]
    );
}

#[test]
fn version_names_parse_and_others_are_an_error() {
    assert_eq!("cangjie5".parse(), Ok(CangjieVersion::Cangjie5));
    assert_eq!("cangjie3".parse(), Ok(CangjieVersion::Cangjie3));
    assert!("cangjie4".parse::<CangjieVersion>().is_err());
}

#[test]
fn cangjie5_is_the_default_version() {
    assert_eq!(CangjieVersion::default(), CangjieVersion::Cangjie5);
}

#[test]
fn radical_hint_pack_round_trips_through_bytes() {
    let pack = sample();
    let bytes = pack.to_bytes().unwrap();
    assert_eq!(RadicalHintPack::from_bytes(&bytes).unwrap(), pack);
}

#[test]
fn other_packs_are_not_radical_hint_packs() {
    assert_eq!(
        RadicalHintPack::from_bytes(b"QIMP\x05"),
        Err(PackError::NotAPack)
    );
}

/// 方案指名的是預設的字根提示資料包，可以換成另一份（plan〈資料包 manifest〉）；方案沒指名時不給提示。
#[test]
fn hints_are_only_given_to_a_scheme_that_wants_them() {
    assert_eq!(
        RadicalHints::for_scheme(sample(), &scheme(Some("cangjie-hint"))),
        Ok(sample().hints)
    );
    assert_eq!(
        RadicalHints::for_scheme(sample(), &scheme(Some("other"))),
        Ok(sample().hints)
    );
    assert_eq!(
        RadicalHints::for_scheme(sample(), &scheme(None)),
        Err(PackError::RadicalHintMismatch {
            expected: None,
            found: Some("cangjie-hint".into()),
        })
    );
}

#[test]
fn scheme_turns_a_code_into_its_radicals() {
    let s = scheme(None);
    assert_eq!(s.radicals_of("an"), "日弓");
    // 方案沒有字根的鍵顯示原字母。
    assert_eq!(s.radicals_of("anx"), "日弓x");
}
