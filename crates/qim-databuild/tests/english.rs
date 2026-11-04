use qim_databuild::english::{self, Affixes};

/// 摘自 ESDB `rel-2026.02.25` 的 `en_US.aff`：字首 `U`、字尾 `S`、`D`、`G`、`M`，禁忌詞旗標 `!`。
const AFF: &str = "\
SET UTF-8
NOSUGGEST !
ONLYINCOMPOUND c
COMPOUNDMIN 1
COMPOUNDRULE 2
COMPOUNDRULE n*1t
COMPOUNDRULE n*mp

PFX U Y 1
PFX U   0     un         .

SFX D Y 4
SFX D   0     d          e
SFX D   y     ied        [^aeiou]y
SFX D   0     ed         [^ey]
SFX D   0     ed         [aeiou]y

SFX G Y 2
SFX G   e     ing        e
SFX G   0     ing        [^e]

SFX S Y 4
SFX S   y     ies        [^aeiou]y
SFX S   0     s          [aeiou]y
SFX S   0     es         [sxzh]
SFX S   0     s          [^sxzhy]

SFX M Y 1
SFX M   0     's         .

SFX H N 1
SFX H   0     th         .
";

const QUICK_ALPHABET: &str = "abcdefghijklmnopqrstuvwxy";

fn expanded(dic: &str) -> Vec<String> {
    let affixes = Affixes::parse(AFF).unwrap();
    english::expand(&affixes, dic)
        .unwrap()
        .words
        .into_iter()
        .collect()
}

#[test]
fn a_stem_expands_to_each_matching_suffix_rule() {
    let words = expanded("2\ncarry/DGS\nbake/DG\n");
    for w in [
        "carry", "carried", "carrying", "carries", "bake", "baked", "baking",
    ] {
        assert!(words.contains(&w.to_string()), "少了 {w}：{words:?}");
    }
    for w in ["carryed", "bakeing", "bakeed"] {
        assert!(!words.contains(&w.to_string()), "條件不合的 {w}：{words:?}");
    }
}

#[test]
fn cross_product_prefixes_combine_with_suffixes() {
    let words = expanded("1\ndo/UG\n");
    assert_eq!(words, ["do", "doing", "undo", "undoing"]);
}

#[test]
fn a_suffix_without_cross_product_does_not_combine_with_a_prefix() {
    let words = expanded("1\nfour/UH\n");
    assert_eq!(words, ["four", "fourth", "unfour"]);
}

#[test]
fn nosuggest_and_only_in_compound_stems_are_dropped_with_all_their_forms() {
    let expansion = english::expand(
        &Affixes::parse(AFF).unwrap(),
        "3\ndamn/DG!\n1th/c\nwalk/D\n",
    )
    .unwrap();
    assert_eq!(
        expansion.words.into_iter().collect::<Vec<_>>(),
        ["walk", "walked"]
    );
    assert_eq!(expansion.taboo, 1);
}

#[test]
fn only_words_typed_with_the_scheme_letters_are_kept_and_keyed_by_lower_case() {
    let expansion = english::expand(
        &Affixes::parse(AFF).unwrap(),
        "6\nthe\nzoo/S\nEnglish/M\nMay\nmay\ncafé\n",
    )
    .unwrap();
    let (words, untypable) = english::typable(&expansion.words, QUICK_ALPHABET);
    let listed: Vec<&str> = words.iter().collect();
    assert_eq!(listed, ["English", "may", "the"]);
    // zoo、zoos、English's、café 打不出（`z` 不在速成字母表、撇號、重音）。
    assert_eq!(untypable, 4);
}

#[test]
fn an_unknown_flag_or_a_continuation_class_is_an_error() {
    let affixes = Affixes::parse(AFF).unwrap();
    assert!(english::expand(&affixes, "1\nwalk/Q\n").is_err());
    assert!(Affixes::parse("SFX X Y 1\nSFX X 0 s/S .\n").is_err());
    assert!(
        Affixes::parse("SFX X Y 2\nSFX X 0 s .\n").is_err(),
        "規則數不符"
    );
}

#[test]
fn compound_rule_flags_are_ignored() {
    // 序數的複合規則只組數字（`21st`），速成打不出；詞幹本身照收。
    let words = expanded("3\n0/nm\n1st/p\nwalk/D\n");
    assert_eq!(words, ["0", "1st", "walk", "walked"]);
}

#[test]
fn a_directive_that_changes_the_word_set_is_an_error() {
    for directive in english::UNSUPPORTED {
        let aff = format!("{AFF}\n{directive} x\n");
        assert!(Affixes::parse(&aff).is_err(), "{directive} 要讓建置失敗");
    }
    // 只影響建議的指令照略過。
    assert!(
        Affixes::parse(&format!(
            "{AFF}\nTRY esianrtolcdugmphbyfvkwz\nREP 1\nREP a ei\n"
        ))
        .is_ok()
    );
}
