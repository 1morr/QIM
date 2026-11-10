use std::path::Path;

use qim_databuild::scheme;

const MINIMAL: &str = r#"
id = "t"
name = "測試"
version = "1"
alphabet = "ab"
max_code_length = 2

[radicals]
a = "日"
b = "月"

[ranking]
word_bonus = 1.5
segment_penalty = 0.5
beam = 8
sentences = 2
"#;

#[test]
fn scheme_file_becomes_core_scheme() {
    let s = scheme::parse(MINIMAL).unwrap();
    assert_eq!(s.id, "t");
    assert_eq!(s.alphabet, "ab");
    assert_eq!(s.max_code_length, 2);
    assert_eq!(s.radicals[&'b'], "月");
}

#[test]
fn every_key_of_the_alphabet_needs_exactly_one_radical() {
    let missing = MINIMAL.replace("b = \"月\"\n", "");
    assert!(scheme::parse(&missing).is_err());
    let extra = MINIMAL.replace("b = \"月\"", "b = \"月\"\nc = \"金\"");
    assert!(scheme::parse(&extra).is_err());
    let long_key = MINIMAL.replace("b = \"月\"", "b = \"月\"\nab = \"金\"");
    assert!(scheme::parse(&long_key).is_err());
}

#[test]
fn packs_name_the_radical_hint_pack() {
    let named = MINIMAL.replace(
        "[radicals]",
        "[packs]\nradical_hint = \"t-hint\"\n\n[radicals]",
    );
    let s = scheme::parse(&named).unwrap();
    assert_eq!(s.packs.radical_hint.as_deref(), Some("t-hint"));
    assert_eq!(scheme::parse(MINIMAL).unwrap().packs.radical_hint, None);
}

#[test]
fn unknown_field_is_an_error() {
    let typo = MINIMAL.replace("max_code_length", "max_code_len");
    assert!(scheme::parse(&typo).is_err());
}

#[test]
fn quick_scheme_in_repo_has_the_25_quick_radicals() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/schemes/quick.toml");
    let s = scheme::parse(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(s.id, "quick");
    assert_eq!(s.max_code_length, 2);
    let radicals: String = s
        .alphabet
        .chars()
        .map(|k| s.radicals[&k].as_str())
        .collect();
    assert_eq!(
        radicals,
        "日月金木水火土竹戈十大中一弓人心手口尸廿山女田難卜"
    );
}

#[test]
fn packs_table_names_the_lexicon_pack() {
    assert_eq!(scheme::parse(MINIMAL).unwrap().packs.lexicon, None);
    let with = format!("{MINIMAL}\n[packs]\nlexicon = \"t-lexicon\"\n");
    assert_eq!(
        scheme::parse(&with).unwrap().packs.lexicon.as_deref(),
        Some("t-lexicon")
    );
    let typo = format!("{MINIMAL}\n[packs]\nlexicons = \"t-lexicon\"\n");
    assert!(scheme::parse(&typo).is_err());
}

#[test]
fn packs_table_names_the_language_model_pack() {
    assert_eq!(scheme::parse(MINIMAL).unwrap().packs.language_model, None);
    let with = format!("{MINIMAL}\n[packs]\nlanguage_model = \"t-bigram\"\n");
    assert_eq!(
        scheme::parse(&with)
            .unwrap()
            .packs
            .language_model
            .as_deref(),
        Some("t-bigram")
    );
}

#[test]
fn ranking_table_sets_word_bonus_segment_penalty_and_beam() {
    let r = scheme::parse(MINIMAL).unwrap().ranking;
    assert_eq!(
        (r.word_bonus, r.segment_penalty, r.beam, r.sentences),
        (1.5, 0.5, 8, 2)
    );
}

#[test]
fn ranking_table_is_required_with_finite_scores_and_a_beam_of_at_least_one() {
    let start = MINIMAL.find("[ranking]").unwrap();
    assert!(scheme::parse(&MINIMAL[..start]).is_err());
    assert!(scheme::parse(&MINIMAL.replace("sentences = 2\n", "")).is_err());
    // 驗證在 `Ranking::check`，與 `bench --set` 共用；錯誤指出是哪個欄位。
    let named = |text: String, field: &str| {
        let error = scheme::parse(&text).unwrap_err().to_string();
        assert!(error.contains(&format!("ranking.{field}")), "{error}");
    };
    named(MINIMAL.replace("1.5", "nan"), "word_bonus");
    named(MINIMAL.replace("0.5", "-inf"), "segment_penalty");
    assert!(scheme::parse(&MINIMAL.replace("sentences = 2", "sentences = 0")).is_ok());
    named(MINIMAL.replace("beam = 8", "beam = 0"), "beam");
    assert!(scheme::parse(&MINIMAL.replace("beam = 8", "beam = 1")).is_ok());
}

#[test]
fn alphabet_is_ascii_letters() {
    let wide = MINIMAL
        .replace("alphabet = \"ab\"", "alphabet = \"aｂ\"")
        .replace("b = \"月\"", "ｂ = \"月\"");
    assert!(scheme::parse(&wide).is_err());
    let digit = MINIMAL
        .replace("alphabet = \"ab\"", "alphabet = \"a1\"")
        .replace("b = \"月\"", "1 = \"月\"");
    assert!(scheme::parse(&digit).is_err());
}
