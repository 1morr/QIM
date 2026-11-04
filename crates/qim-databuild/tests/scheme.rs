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
