use std::collections::BTreeMap;

use qim_core::{CodeEntry, CodeTable, Pack};
use qim_databuild::libcangjie::{self, Row};
use qim_databuild::verify::{self, CheckId, Report};
use qim_databuild::{build_pack, scheme};

/// 取自 libcangjie `346026b`，涵蓋 plan §4.5 的抽查字，加一個擴展區的字。
const TABLE: &str = "\
日 日 1 1 0 0 1 0 0 0 0 a a NA 23061
問 问 1 1 0 0 1 0 0 0 0 anr anr NA 20001
丟 丢 1 1 0 0 1 0 0 0 0 hgi mgi,xxmgi NA 22813
嘅 嘅 1 0 1 0 1 0 0 0 0 rhpu rhpu NA 0
° NA 0 0 0 0 0 0 0 0 1 yyycg yyycg NA 0
𠀀 𠀀 1 0 0 0 0 0 0 0 0 a NA NA 0
";

const SCHEME: &str = r#"
id = "t"
name = "測試"
version = "1"
alphabet = "ab"
max_code_length = 2
[radicals]
a = "日"
b = "月"
"#;

fn rows() -> Vec<Row> {
    libcangjie::parse(TABLE).unwrap()
}

fn pack() -> Pack {
    build_pack(&rows(), &BTreeMap::new(), scheme::parse(SCHEME).unwrap())
}

fn run(pack: &Pack) -> Report {
    let bytes = pack.to_bytes().unwrap();
    verify::run(&rows(), &bytes, &bytes)
}

/// 改動碼表後重建資料包。
fn edited(edit: impl FnOnce(&mut BTreeMap<String, Vec<CodeEntry>>)) -> Pack {
    let mut pack = pack();
    let mut entries: BTreeMap<String, Vec<CodeEntry>> = pack
        .code_table
        .iter()
        .map(|(code, e)| (code.to_string(), e.to_vec()))
        .collect();
    edit(&mut entries);
    pack.code_table = CodeTable::from_entries(entries);
    pack
}

fn failed(report: &Report) -> Vec<CheckId> {
    report
        .checks
        .iter()
        .filter(|c| !c.passed)
        .map(|c| c.id)
        .collect()
}

const fn entry(text: char) -> CodeEntry {
    CodeEntry {
        text,
        extension: false,
    }
}

#[test]
fn pack_built_from_the_source_passes_every_check() {
    let report = run(&pack());
    assert_eq!(failed(&report), []);
    assert!(report.passed());
    assert_eq!((report.continuous, report.extension), (4, 1));
}

#[test]
fn report_lists_every_check_of_plan_4_5_except_unihan() {
    let ids: Vec<CheckId> = run(&pack()).checks.iter().map(|c| c.id).collect();
    assert_eq!(
        ids,
        [
            CheckId::Decodes,
            CheckId::NoMissingCharacters,
            CheckId::KeyLength,
            CheckId::NoCangjieCodeForWen,
            CheckId::WenHasAr,
            CheckId::DiuHasHiAndMi,
            CheckId::RiIsTopForA,
            CheckId::Deterministic,
        ]
    );
}

#[test]
fn flags_and_codes_on_different_rows_of_one_character_still_count_as_continuous() {
    // 有碼的列沒有旗標、有旗標的列沒有碼：以字為單位，它仍是有倉頡碼的連打字集的字。
    let table =
        format!("{TABLE}㐀 㐀 1 0 0 0 0 0 0 0 0 a NA NA 0\n㐀 㐀 1 0 1 0 0 0 0 0 0 NA NA NA 0\n");
    let rows = libcangjie::parse(&table).unwrap();
    let bytes = build_pack(&rows, &BTreeMap::new(), scheme::parse(SCHEME).unwrap())
        .to_bytes()
        .unwrap();
    let report = verify::run(&rows, &bytes, &bytes);
    assert_eq!(failed(&report), []);
    assert_eq!((report.continuous, report.extension), (5, 1));
}

#[test]
fn three_letter_key_fails_the_key_length_check() {
    let pack = edited(|t| {
        t.insert("abc".into(), vec![entry('日')]);
    });
    assert_eq!(failed(&run(&pack)), [CheckId::KeyLength]);
}

#[test]
fn dropped_character_fails_the_missing_characters_check() {
    let pack = edited(|t| {
        t.remove("ru");
    });
    assert_eq!(failed(&run(&pack)), [CheckId::NoMissingCharacters]);
}

#[test]
fn cangjie_code_leaking_into_the_table_fails() {
    let pack = edited(|t| {
        t.insert("anr".into(), vec![entry('問')]);
    });
    let failed = failed(&run(&pack));
    assert!(failed.contains(&CheckId::NoCangjieCodeForWen), "{failed:?}");
}

#[test]
fn missing_spot_check_codes_fail() {
    let pack = edited(|t| {
        t.get_mut("ar").unwrap().retain(|e| e.text != '問');
        t.insert("xx".into(), vec![entry('問')]);
        t.remove("mi");
    });
    assert_eq!(
        failed(&run(&pack)),
        [CheckId::WenHasAr, CheckId::DiuHasHiAndMi]
    );
}

#[test]
fn another_character_before_ri_on_a_fails_the_top_candidate_check() {
    let pack = edited(|t| {
        // `a` 是「日𠀀」，倒過來就是別的字在前。
        t.get_mut("a").unwrap().reverse();
    });
    assert_eq!(failed(&run(&pack)), [CheckId::RiIsTopForA]);
}

#[test]
fn different_bytes_from_the_second_build_fail_determinism() {
    let bytes = pack().to_bytes().unwrap();
    let mut other = bytes.clone();
    *other.last_mut().unwrap() ^= 1;
    let report = verify::run(&rows(), &bytes, &other);
    assert_eq!(failed(&report), [CheckId::Deterministic]);
}

#[test]
fn undecodable_pack_fails_without_panicking() {
    let report = verify::run(&rows(), b"garbage", b"garbage");
    assert!(failed(&report).contains(&CheckId::Decodes));
    assert!(!report.passed());
}

mod corpus {
    use qim_databuild::libcangjie;
    use qim_databuild::transcode::Transcoder;
    use qim_databuild::verify::{self, CheckId, CorpusDigests};

    use super::TABLE;

    const BENCH: &str = "ar a\t問日\nmi mi\t丟丟\n";
    const TRAIN: &str = "日丟\n問問日\n";
    const DEV: &str = "丟\n";

    fn failed(bench: &str, train: &str, dev: &str, second: [&str; 4]) -> Vec<CheckId> {
        let transcoder = Transcoder::new(&libcangjie::parse(TABLE).unwrap());
        let digests = |[bench, train, dev, chars]: [&str; 4]| CorpusDigests {
            bench: bench.into(),
            train: train.into(),
            dev: dev.into(),
            chars: chars.into(),
        };
        let first = digests(SAME);
        let second = digests(second);
        verify::corpus(
            "test",
            &transcoder,
            bench,
            train.as_bytes(),
            dev.as_bytes(),
            &first,
            &second,
        )
        .unwrap()
        .into_iter()
        .filter(|c| !c.passed)
        .map(|c| c.id)
        .collect()
    }

    const SAME: [&str; 4] = ["b", "t", "d", "c"];

    #[test]
    fn clean_corpus_passes_even_with_lines_in_another_order() {
        assert_eq!(failed(BENCH, TRAIN, DEV, SAME), []);
        assert_eq!(
            failed("mi mi\t丟丟\nar a\t問日\n", "問問日\n日丟\n", DEV, SAME),
            []
        );
    }

    /// 每個基準集對所有訓練切分的洩漏檢查：(集合名, 洩漏句數)。
    fn leaks(benches: &[(&str, &str)], trains: &[&str]) -> Vec<(bool, String)> {
        verify::leaks(benches, trains.iter().map(|t| t.as_bytes()))
            .unwrap()
            .into_iter()
            .map(|c| {
                assert_eq!(c.id, CheckId::CorpusNoLeak);
                (c.passed, c.detail)
            })
            .collect()
    }

    #[test]
    fn bench_sentence_in_its_own_train_is_a_leak() {
        let leaked = format!("{TRAIN}丟丟\n");
        assert_eq!(
            leaks(&[("a", BENCH)], &[&leaked]),
            [(false, "a test 句原樣出現在 train 1 句".into())]
        );
        assert_eq!(
            leaks(&[("a", BENCH)], &[TRAIN]),
            [(true, "a test 句原樣出現在 train 0 句".into())]
        );
    }

    #[test]
    fn bench_sentence_in_another_corpus_train_is_a_leak() {
        // bigram 用所有語料的 train 訓練，所以 a 的基準句出現在 b 的 train 也算洩漏。
        let other = "日日\n丟丟\n丟丟\n問日\n";
        assert_eq!(
            leaks(&[("a", BENCH), ("b", "a\t日\n")], &[TRAIN, other]),
            [
                (false, "a test 句原樣出現在 train 2 句".into()),
                (true, "b test 句原樣出現在 train 0 句".into()),
            ]
        );
    }

    #[test]
    fn wrong_code_or_untranscodable_sentence_fails() {
        let wrong_code = BENCH.replace("ar a", "an a");
        assert_eq!(
            failed(&wrong_code, TRAIN, DEV, SAME),
            [CheckId::CorpusTranscodes]
        );
        assert_eq!(
            failed(BENCH, "日𠀀\n", DEV, SAME),
            [CheckId::CorpusTranscodes]
        );
        assert_eq!(
            failed(BENCH, TRAIN, "嘅a\n", SAME),
            [CheckId::CorpusTranscodes]
        );
        assert_eq!(
            failed("問日\n", TRAIN, DEV, SAME),
            [CheckId::CorpusTranscodes]
        );
    }

    #[test]
    fn different_second_build_fails() {
        assert_eq!(
            failed(BENCH, TRAIN, DEV, ["b", "t", "x", "c"]),
            [CheckId::CorpusDeterministic]
        );
        assert_eq!(
            failed(BENCH, TRAIN, DEV, ["b", "t", "d", "x"]),
            [CheckId::CorpusDeterministic]
        );
    }
}
