use std::collections::BTreeMap;

use qim_core::Lexicon;
use qim_databuild::lexicon::{self, Counts, MAX_COMBINATIONS};
use qim_databuild::transcode::Transcoder;
use qim_databuild::{libcangjie, quick};

/// 日、門、口、弓的碼照抄 libcangjie `346026b`；丟三代 `hgi`、五代 `mgi`，速成 `hi` / `mi`，
/// 五代那個是轉碼用的主碼。甲（`a` / 主碼 `ab`）、乙（`c` / 主碼 `bc`）是假資料：
/// 甲乙換掉不同字的碼會拼出同一個碼。𠀀不在連打字集。
const TABLE: &str = "\
日 日 1 1 0 0 1 0 0 0 0 a a NA 23061
門 门 1 1 0 0 1 0 0 0 0 an an NA 20000
口 口 1 1 0 0 1 0 0 0 0 r r NA 20100
弓 弓 1 1 0 0 1 0 0 0 0 n n NA 20200
丟 丢 1 1 0 0 1 0 0 0 0 hgi mgi NA 22813
甲 甲 1 1 0 0 1 0 0 0 0 a ab NA 0
乙 乙 1 1 0 0 1 0 0 0 0 c bc NA 0
𠀀 𠀀 1 0 0 0 0 0 0 0 0 a NA NA 0
";

fn build(essay: &str) -> (Lexicon, Counts) {
    build_with(essay, BTreeMap::new())
}

fn build_with(essay: &str, char_frequency: BTreeMap<char, u64>) -> (Lexicon, Counts) {
    let rows = libcangjie::parse(TABLE).unwrap();
    let code_table = quick::code_table(&rows, &BTreeMap::new());
    lexicon::build(essay, &code_table, &Transcoder::new(&rows), char_frequency).unwrap()
}

fn texts(lexicon: &Lexicon, code: &str) -> Vec<String> {
    lexicon
        .lookup(code)
        .iter()
        .map(|w| w.text.clone())
        .collect()
}

/// 詞 → 它在詞庫裡的所有碼。
fn codes_of(lexicon: &Lexicon, word: &str) -> Vec<String> {
    lexicon
        .iter()
        .filter(|(_, words)| words.iter().any(|w| w.text == word))
        .map(|(code, _)| code.to_string())
        .collect()
}

#[test]
fn a_word_is_indexed_by_the_quick_codes_of_its_characters_joined() {
    let (lexicon, _) = build("門口\t9435\n");
    assert_eq!(texts(&lexicon, "anr"), ["門口"]);
    assert_eq!(lexicon.lookup("anr")[0].frequency, 9435);
}

#[test]
fn words_of_one_code_are_ordered_by_frequency_then_text() {
    let (lexicon, _) = build("日弓口\t500\n門口\t9435\n");
    assert_eq!(texts(&lexicon, "anr"), ["門口", "日弓口"]);
    // 同頻依碼位：日 U+65E5 < 門 U+9580。
    let (lexicon, _) = build("門口\t100\n日弓口\t100\n");
    assert_eq!(texts(&lexicon, "anr"), ["日弓口", "門口"]);
}

#[test]
fn single_characters_rare_words_and_words_outside_the_continuous_set_are_left_out() {
    let (lexicon, counts) = build("日\t71052\n門口\t9435\n日日\t99\n日𠀀\t500\n口口\t100\n");
    assert_eq!(texts(&lexicon, "aa"), Vec::<String>::new());
    assert_eq!(texts(&lexicon, "rr"), ["口口"]);
    assert_eq!(
        counts,
        Counts {
            entries: 5,
            single_characters: 1,
            below_threshold: 1,
            outside_continuous: 1,
            words: 2,
            capped: 0,
            codes: 2,
            indexed: 2,
        }
    );
}

#[test]
fn a_word_listed_twice_keeps_its_highest_frequency() {
    // rime-essay `054920d` 的「簽押」有 0 與 434 兩列。
    let (lexicon, counts) = build("門口\t0\n門口\t434\n門口\t120\n");
    assert_eq!(lexicon.lookup("anr").len(), 1);
    assert_eq!(lexicon.lookup("anr")[0].frequency, 434);
    assert_eq!(
        (counts.entries, counts.below_threshold, counts.words),
        (3, 0, 1)
    );
}

#[test]
fn a_word_with_multi_code_characters_gets_every_combination() {
    let (lexicon, counts) = build("丟丟\t200\n");
    assert_eq!(codes_of(&lexicon, "丟丟"), ["hihi", "himi", "mihi", "mimi"]);
    assert_eq!(counts.indexed, 4);
}

#[test]
fn combinations_beyond_the_limit_keep_those_closest_to_the_main_codes() {
    // 16 個組合：全用主碼 1 個、換掉一個字的碼 4 個、換掉兩個 6 個……
    let (lexicon, counts) = build("丟丟丟丟\t200\n");
    let codes = codes_of(&lexicon, "丟丟丟丟");
    assert_eq!(codes.len(), MAX_COMBINATIONS);
    for code in ["mimimimi", "himimimi", "mihimimi", "mimihimi", "mimimihi"] {
        assert!(codes.contains(&code.to_string()), "{code}：{codes:?}");
    }
    assert!(!codes.contains(&"hihihihi".to_string()), "{codes:?}");
    assert_eq!(counts.capped, 1);
}

#[test]
fn combinations_spelling_the_same_code_count_once_toward_the_limit() {
    // 甲乙甲乙 16 個組合：`ab`+`c` 與 `a`+`bc` 都是 `abc`，重複的不佔上限。
    let (lexicon, counts) = build("甲乙甲乙\t200\n");
    let codes = codes_of(&lexicon, "甲乙甲乙");
    assert_eq!(codes.len(), MAX_COMBINATIONS, "{codes:?}");
    assert!(codes.contains(&"abbcabbc".to_string()), "{codes:?}");
    assert_eq!(counts.indexed, MAX_COMBINATIONS);
    assert_eq!(counts.capped, 1);
}

#[test]
fn character_frequency_from_every_corpus_is_summed_into_the_lexicon() {
    let frequency = lexicon::char_frequency(&["日\t3\n口\t1\n", "日\t2\n"]).unwrap();
    assert_eq!(frequency, BTreeMap::from([('日', 5), ('口', 1)]));
    let (lexicon, _) = build_with("門口\t9435\n", frequency);
    assert_eq!(lexicon.char_frequency('日'), 5);
    assert_eq!(lexicon.char_frequency('門'), 0);
}

#[test]
fn malformed_lines_are_errors() {
    let rows = libcangjie::parse(TABLE).unwrap();
    let code_table = quick::code_table(&rows, &BTreeMap::new());
    let transcoder = Transcoder::new(&rows);
    for essay in ["門口\n", "門口\t9435\t1\n", "門口\tmany\n", "\t9\n"] {
        let err = lexicon::build(essay, &code_table, &transcoder, BTreeMap::new()).unwrap_err();
        assert!(err.to_string().contains("第 1 行"), "{essay:?}：{err}");
    }
    assert!(lexicon::char_frequency(&["日 3\n"]).is_err());
}
