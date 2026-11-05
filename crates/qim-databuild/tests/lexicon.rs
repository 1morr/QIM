use std::collections::BTreeMap;

use qim_core::Lexicon;
use qim_databuild::common_forms::BothForms;
use qim_databuild::lexicon::{self, Counts, Forms, MAX_COMBINATIONS};
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
    try_build(essay, "", char_frequency).unwrap()
}

/// 加上粵語口語詞表（`data/lexicon/yue-colloquial.tsv` 的格式）。
fn build_colloquial(essay: &str, colloquial: &str) -> (Lexicon, Counts) {
    try_build(essay, colloquial, BTreeMap::new()).unwrap()
}

fn try_build(
    essay: &str,
    colloquial: &str,
    char_frequency: BTreeMap<char, u64>,
) -> Result<(Lexicon, Counts), qim_databuild::Error> {
    try_build_with_forms(
        essay,
        colloquial,
        char_frequency,
        &|s| s.to_string(),
        &BothForms::default(),
    )
}

fn try_build_with_forms(
    essay: &str,
    colloquial: &str,
    char_frequency: BTreeMap<char, u64>,
    normalize: &dyn Fn(&str) -> String,
    both: &BothForms,
) -> Result<(Lexicon, Counts), qim_databuild::Error> {
    let rows = libcangjie::parse(TABLE).unwrap();
    let code_table = quick::code_table(&rows, &BTreeMap::new());
    lexicon::build(
        essay,
        colloquial,
        &code_table,
        &Transcoder::new(&rows),
        char_frequency,
        &Forms { normalize, both },
    )
}

/// 假的寫法：「甲」是要換成「日」的寫法（像「羣」換成「群」）；「弓／乙」是並存寫法，代表寫法是弓。
fn normalize(text: &str) -> String {
    text.replace('甲', "日").replace('乙', "弓")
}

fn both() -> BothForms {
    BothForms::new(&[('弓', '乙')], &[]).unwrap()
}

fn build_with_forms(essay: &str, colloquial: &str) -> (Lexicon, Counts) {
    try_build_with_forms(
        essay,
        colloquial,
        BTreeMap::from([('弓', 7), ('乙', 3), ('日', 5)]),
        &normalize,
        &both(),
    )
    .unwrap()
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
            colloquial: 0,
            converted: 0,
            merged: 0,
            spellings: 0,
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
fn character_frequency_from_every_corpus_is_weighted_and_summed_into_the_lexicon() {
    // 第二份語料權重 10，和 bigram 的次數乘同一個權重（corpora.rs 的 `weight`）。
    let frequency = lexicon::char_frequency(&[("日\t3\n口\t1\n", 1), ("日\t2\n", 10)]).unwrap();
    assert_eq!(frequency, BTreeMap::from([('日', 23), ('口', 1)]));
    let (lexicon, _) = build_with("門口\t9435\n", frequency);
    assert_eq!(lexicon.char_frequency('日'), 23);
    assert_eq!(lexicon.char_frequency('門'), 0);
}

#[test]
fn malformed_lines_are_errors() {
    let rows = libcangjie::parse(TABLE).unwrap();
    let code_table = quick::code_table(&rows, &BTreeMap::new());
    let transcoder = Transcoder::new(&rows);
    for essay in ["門口\n", "門口\t9435\t1\n", "門口\tmany\n", "\t9\n"] {
        let forms = Forms {
            normalize: &|s| s.to_string(),
            both: &BothForms::default(),
        };
        let err = lexicon::build(essay, "", &code_table, &transcoder, BTreeMap::new(), &forms)
            .unwrap_err();
        assert!(err.to_string().contains("第 1 行"), "{essay:?}：{err}");
    }
    assert!(lexicon::char_frequency(&[("日 3\n", 1)]).is_err());
}

#[test]
fn colloquial_words_join_the_lexicon_with_their_listed_frequency() {
    // 日弓不在 essay、丟丟在 essay 但低於門檻：都由詞表補上，用詞表的詞頻。
    let (lexicon, counts) = build_colloquial(
        "門口\t9435\n丟丟\t20\n",
        "# 註解\n\n日弓\t300\n丟丟\t1000\n",
    );
    assert_eq!(texts(&lexicon, "an"), ["日弓"]);
    assert_eq!(lexicon.lookup("an")[0].frequency, 300);
    assert_eq!(lexicon.lookup("mimi")[0].frequency, 1000);
    assert_eq!(lexicon.lookup("anr")[0].frequency, 9435);
    assert_eq!(
        (counts.colloquial, counts.below_threshold, counts.words),
        (2, 0, 3)
    );
}

#[test]
fn colloquial_list_errors_name_the_file_and_line() {
    // 詞表是人工整理的：會被濾掉的列一律當錯，不默默丟掉。
    for (colloquial, reason) in [
        ("日弓\n", "詞<TAB>詞頻"),
        ("日弓\t300\t1\n", "詞<TAB>詞頻"),
        ("日弓\tmany\n", "整數"),
        ("日\t300\n", "兩字以上"),
        ("日弓\t99\n", "100"),
        ("日𠀀\t300\n", "連打字集"),
        ("日弓\t300\n日弓\t1000\n", "重複"),
        // essay 已收的詞列進詞表沒有作用，只會讓人以為詞表的詞頻算數。
        ("門口\t300\n", "rime-essay 已收"),
    ] {
        let line = colloquial.lines().count();
        let err = try_build("門口\t9435\n", colloquial, BTreeMap::new())
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("yue-colloquial.tsv") && err.contains(&format!("第 {line} 行")),
            "{colloquial:?}：{err}"
        );
        assert!(err.contains(reason), "{colloquial:?}：{err}");
    }
}

#[test]
fn words_are_written_in_the_corpus_forms_and_merged_by_summing() {
    // 甲口換成日口，和原本的日口撞在一起：詞頻相加（同一個詞列兩次才取最大）。
    let (lexicon, counts) = build_with_forms("甲口\t300\n日口\t200\n甲門\t150\n", "");
    assert_eq!(texts(&lexicon, "ar"), ["日口"]);
    assert_eq!(lexicon.lookup("ar")[0].frequency, 500);
    assert_eq!(texts(&lexicon, "aan"), ["日門"]);
    assert!(codes_of(&lexicon, "甲口").is_empty());
    assert_eq!((counts.converted, counts.merged, counts.words), (2, 1, 2));
}

#[test]
fn both_forms_are_indexed_under_their_own_codes_with_the_same_frequency() {
    // 弓口（`nr`）與乙口（`cr`、`bcr`）：essay 寫乙口，兩種拼法都收、詞頻一樣。
    let (lexicon, counts) = build_with_forms("乙口\t300\n弓口\t200\n", "");
    assert_eq!(texts(&lexicon, "nr"), ["弓口"]);
    assert_eq!(texts(&lexicon, "cr"), ["乙口"]);
    assert_eq!(lexicon.lookup("nr")[0].frequency, 500);
    assert_eq!(lexicon.lookup("cr")[0].frequency, 500);
    assert_eq!((counts.words, counts.spellings), (1, 1));
}

#[test]
fn both_forms_share_one_character_frequency() {
    let (lexicon, _) = build_with_forms("門口\t300\n", "");
    assert_eq!(lexicon.char_frequency('弓'), 10);
    assert_eq!(lexicon.char_frequency('乙'), 10);
    assert_eq!(lexicon.char_frequency('日'), 5);
}

#[test]
fn colloquial_words_go_through_the_same_forms() {
    // 詞表寫乙門：換成代表寫法查重，兩種拼法都收。
    let (lexicon, _) = build_with_forms("門口\t300\n", "乙門\t400\n");
    assert_eq!(texts(&lexicon, "nan"), ["弓門"]);
    assert_eq!(texts(&lexicon, "can"), ["乙門"]);
    let err = try_build_with_forms(
        "弓口\t300\n",
        "乙口\t400\n",
        BTreeMap::new(),
        &normalize,
        &both(),
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("rime-essay 已收"), "{err}");
}

#[test]
fn one_way_forms_add_the_other_spelling_but_never_the_reverse() {
    // 假的單向組：門的詞也可寫成日（像隨着 → 隨著），日的詞不反過來寫成門（像著名不能寫成着名）。
    let (lexicon, counts) = try_build_with_forms(
        "門口\t300\n日口\t200\n日弓\t150\n",
        "",
        BTreeMap::from([('門', 4), ('日', 9)]),
        &|s| s.to_string(),
        &BothForms::new(&[], &[('門', '日')]).unwrap(),
    )
    .unwrap();
    assert_eq!(texts(&lexicon, "anr"), ["門口"]);
    assert_eq!(lexicon.lookup("anr")[0].frequency, 300);
    // 門口寫成日口，和 essay 本來的日口是同一個詞：詞頻相加，只收一次。
    assert_eq!(texts(&lexicon, "ar"), ["日口"]);
    assert_eq!(lexicon.lookup("ar")[0].frequency, 500);
    assert!(codes_of(&lexicon, "門弓").is_empty());
    assert_eq!((counts.words, counts.spellings), (3, 0));
    // 字頻各算各的。
    assert_eq!(
        (lexicon.char_frequency('門'), lexicon.char_frequency('日')),
        (4, 9)
    );
}
