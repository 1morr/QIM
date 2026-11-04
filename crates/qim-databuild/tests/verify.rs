use std::collections::BTreeMap;

use qim_core::{
    CangjieCodes, CharModel, CodeEntry, CodeTable, ConversionTable, LanguageModel,
    LanguageModelPack, Lexicon, LexiconPack, Pack, RadicalHintPack, RadicalHints, Simplifier,
    SimplifierPack, Word,
};
use qim_databuild::libcangjie::{self, Row};
use qim_databuild::transcode::Transcoder;
use qim_databuild::verify::{self, CheckId, PackBytes, Report};
use qim_databuild::{bigram, build_pack, lexicon, radical_hint, scheme};

/// 取自 libcangjie `346026b`，涵蓋 plan §4.5 的抽查字與「門口」，加一個擴展區的字。
const TABLE: &str = "\
日 日 1 1 0 0 1 0 0 0 0 a a NA 23061
曰 曰 1 1 0 0 1 0 0 0 0 a a NA 23060
問 问 1 1 0 0 1 0 0 0 0 anr anr NA 20001
門 门 1 1 0 0 1 0 0 0 0 an an NA 20000
口 口 1 1 0 0 1 0 0 0 0 r r NA 20100
丟 丢 1 1 0 0 1 0 0 0 0 hgi mgi,xxmgi NA 22813
嘅 嘅 1 0 1 0 1 0 0 0 0 rhpu rhpu NA 0
° NA 0 0 0 0 0 0 0 0 1 yyycg yyycg NA 0
𠀀 𠀀 1 0 0 0 0 0 0 0 0 a NA NA 0
";

const SCHEME: &str = r#"
id = "t"
name = "測試"
version = "1"
alphabet = "abnr"
max_code_length = 2
[packs]
lexicon = "t-lexicon"
language_model = "t-bigram"
radical_hint = "t-hint"
[radicals]
a = "日"
b = "月"
n = "弓"
r = "口"
[ranking]
word_bonus = 0.0
segment_penalty = 0.0
beam = 20
sentences = 3
"#;

const ESSAY: &str = "門口\t9435\n丟丟\t200\n";

fn rows() -> Vec<Row> {
    libcangjie::parse(TABLE).unwrap()
}

/// 日有年級，排在 Big5 排序值較小的曰前面。
fn pack() -> Pack {
    build_pack(
        &rows(),
        &BTreeMap::from([('日', 1)]),
        scheme::parse(SCHEME).unwrap(),
    )
}

fn lexicon_with(char_frequency: BTreeMap<char, u64>) -> LexiconPack {
    let rows = rows();
    let (lexicon, _) = lexicon::build(
        ESSAY,
        "",
        &pack().code_table,
        &Transcoder::new(&rows),
        char_frequency,
    )
    .unwrap();
    LexiconPack {
        id: "t-lexicon".into(),
        lexicon,
    }
}

fn lexicon() -> LexiconPack {
    lexicon_with(BTreeMap::from([('日', 10), ('曰', 3)]))
}

/// 日比曰常見，`a` 的首選才是日。
fn language_model() -> LanguageModelPack {
    language_model_from("日曰日\n門口\n")
}

/// 由一行一句的語料訓練；單字的分數來自這裡的字次數，和詞庫的字頻同源。
fn language_model_from(corpus: &str) -> LanguageModelPack {
    let mut counter = bigram::Counter::default();
    counter.add(corpus.as_bytes(), 1).unwrap();
    LanguageModelPack {
        id: "t-bigram".into(),
        model: bigram::train(&counter, 8, 1).0,
    }
}

/// 只轉「門」的簡繁資料包。
fn simplifier() -> SimplifierPack {
    SimplifierPack {
        id: "opencc-hk2s".into(),
        simplifier: Simplifier::new(
            vec![],
            ConversionTable::new([]),
            vec![vec![ConversionTable::new([("門".into(), "门".into())])]],
        ),
    }
}

/// 由同一份 libcangjie 的列建的字根提示。
fn radical_hint() -> RadicalHintPack {
    radical_hint_from(radical_hint::hints(&rows()))
}

fn radical_hint_from(hints: RadicalHints) -> RadicalHintPack {
    RadicalHintPack {
        id: "t-hint".into(),
        hints,
    }
}

/// 改動字根提示後重建資料包。
fn edited_hint(edit: impl FnOnce(&mut BTreeMap<char, CangjieCodes>)) -> RadicalHintPack {
    let mut chars: BTreeMap<char, CangjieCodes> = radical_hint()
        .hints
        .iter()
        .map(|(c, codes)| (c, codes.clone()))
        .collect();
    edit(&mut chars);
    radical_hint_from(RadicalHints::new(chars))
}

fn run_with_hint(hint: &RadicalHintPack) -> Report {
    report(&pack(), &lexicon(), &language_model(), hint)
}

fn run_with(pack: &Pack, lexicon: &LexiconPack) -> Report {
    run_with_language_model(pack, lexicon, &language_model())
}

fn run_with_language_model(
    pack: &Pack,
    lexicon: &LexiconPack,
    language_model: &LanguageModelPack,
) -> Report {
    report(pack, lexicon, language_model, &radical_hint())
}

fn report(
    pack: &Pack,
    lexicon: &LexiconPack,
    language_model: &LanguageModelPack,
    radical_hint: &RadicalHintPack,
) -> Report {
    let quick = pack.to_bytes().unwrap();
    let lexicon = lexicon.to_bytes().unwrap();
    let language_model = language_model.to_bytes().unwrap();
    let radical_hint = radical_hint.to_bytes().unwrap();
    let simplifier = simplifier().to_bytes().unwrap();
    let bytes = PackBytes {
        quick: &quick,
        lexicon: &lexicon,
        language_model: &language_model,
        radical_hint: &radical_hint,
        simplifier: &simplifier,
    };
    verify::run(&rows(), bytes, bytes)
}

fn run(pack: &Pack) -> Report {
    run_with(pack, &lexicon())
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

/// 改動詞庫的碼索引，字頻照舊。
fn edited_lexicon(edit: impl FnOnce(&mut BTreeMap<String, Vec<Word>>)) -> LexiconPack {
    let mut pack = lexicon();
    let mut words: BTreeMap<String, Vec<Word>> = pack
        .lexicon
        .iter()
        .map(|(code, w)| (code.to_string(), w.to_vec()))
        .collect();
    edit(&mut words);
    pack.lexicon = Lexicon::new(words, [('日', 10), ('曰', 3)]);
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

fn word(text: &str) -> Word {
    Word {
        text: text.into(),
        frequency: 100,
    }
}

#[test]
fn packs_built_from_the_source_pass_every_check() {
    let report = run(&pack());
    assert_eq!(failed(&report), []);
    assert!(report.passed());
    assert_eq!((report.continuous, report.extension), (7, 1));
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
            CheckId::WordCodesFromCharacters,
            CheckId::MenKouForAnr,
            CheckId::RadicalHintCoversCharacters,
            CheckId::RadicalHintByVersion,
            CheckId::SimplifiedMenKou,
            CheckId::LanguageModelSize,
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
    let lexicon = lexicon().to_bytes().unwrap();
    let language_model = language_model().to_bytes().unwrap();
    let radical_hint = radical_hint_from(radical_hint::hints(&rows))
        .to_bytes()
        .unwrap();
    let simplifier = simplifier().to_bytes().unwrap();
    let both = PackBytes {
        quick: &bytes,
        lexicon: &lexicon,
        language_model: &language_model,
        radical_hint: &radical_hint,
        simplifier: &simplifier,
    };
    let report = verify::run(&rows, both, both);
    assert_eq!(failed(&report), []);
    assert_eq!((report.continuous, report.extension), (8, 1));
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
        // 詞庫的「丟丟」還用著丟的 mi。
        [
            CheckId::WenHasAr,
            CheckId::DiuHasHiAndMi,
            CheckId::WordCodesFromCharacters
        ]
    );
}

#[test]
fn another_character_before_ri_on_a_fails_the_top_candidate_check() {
    // 字頻相同時照碼表的順序，曰排到日前面就不對。
    let pack = edited(|t| {
        t.get_mut("a").unwrap().swap(0, 1);
    });
    let same = lexicon_with(BTreeMap::from([('日', 3), ('曰', 3)]));
    assert_eq!(
        failed(&run_with_language_model(
            &pack,
            &same,
            &language_model_from("日日日\n曰曰曰\n")
        )),
        [CheckId::RiIsTopForA]
    );
}

#[test]
fn a_higher_frequency_for_another_character_fails_the_top_candidate_check() {
    let yue_first = lexicon_with(BTreeMap::from([('日', 3), ('曰', 10)]));
    assert_eq!(
        failed(&run_with_language_model(
            &pack(),
            &yue_first,
            &language_model_from("日日日\n曰曰曰曰曰曰曰曰曰曰\n")
        )),
        [CheckId::RiIsTopForA]
    );
}

#[test]
fn word_code_not_made_of_its_characters_quick_codes_fails() {
    // 問的倉頡碼 anr 不是速成碼；anx 拼不出門口。
    for (code, text) in [("anra", "問日"), ("anx", "門口")] {
        let lexicon = edited_lexicon(|w| {
            w.entry(code.into()).or_default().push(word(text));
        });
        let report = run_with(&pack(), &lexicon);
        assert_eq!(
            failed(&report),
            [CheckId::WordCodesFromCharacters],
            "{code} {text}"
        );
        let detail = &report
            .checks
            .iter()
            .find(|c| c.id == CheckId::WordCodesFromCharacters)
            .unwrap()
            .detail;
        assert!(detail.contains(&format!("{code} {text}")), "{detail}");
    }
}

/// 簡繁資料包沒轉「門」：簡體模式的候選還是繁體。
#[test]
fn a_simplifier_pack_that_does_not_convert_fails_the_spot_check() {
    let quick = pack().to_bytes().unwrap();
    let lexicon = lexicon().to_bytes().unwrap();
    let model = language_model().to_bytes().unwrap();
    let mut identity = simplifier();
    identity.simplifier = Simplifier::new(vec![], ConversionTable::new([]), vec![]);
    let identity = identity.to_bytes().unwrap();
    let hint = radical_hint().to_bytes().unwrap();
    let bytes = PackBytes {
        quick: &quick,
        lexicon: &lexicon,
        language_model: &model,
        radical_hint: &hint,
        simplifier: &identity,
    };
    assert_eq!(
        failed(&verify::run(&rows(), bytes, bytes)),
        [CheckId::SimplifiedMenKou]
    );
}

#[test]
fn missing_men_kou_for_anr_fails() {
    let lexicon = edited_lexicon(|w| {
        w.remove("anr");
    });
    assert_eq!(
        failed(&run_with(&pack(), &lexicon)),
        [CheckId::MenKouForAnr]
    );
}

#[test]
fn different_bytes_from_the_second_build_fail_determinism() {
    let first = [
        pack().to_bytes().unwrap(),
        lexicon().to_bytes().unwrap(),
        language_model().to_bytes().unwrap(),
        radical_hint().to_bytes().unwrap(),
        simplifier().to_bytes().unwrap(),
    ];
    fn bytes(b: &[Vec<u8>; 5]) -> PackBytes<'_> {
        PackBytes {
            quick: &b[0],
            lexicon: &b[1],
            language_model: &b[2],
            radical_hint: &b[3],
            simplifier: &b[4],
        }
    }
    for changed in 0..5 {
        let mut second = first.clone();
        *second[changed].last_mut().unwrap() ^= 1;
        let report = verify::run(&rows(), bytes(&first), bytes(&second));
        assert_eq!(
            failed(&report),
            [CheckId::Deterministic],
            "改第 {changed} 份"
        );
    }
}

#[test]
fn undecodable_pack_fails_without_panicking() {
    let quick = pack().to_bytes().unwrap();
    let lexicon = lexicon().to_bytes().unwrap();
    let mut other_model = language_model();
    let model = language_model().to_bytes().unwrap();
    other_model.id = "other".into();
    let other_model = other_model.to_bytes().unwrap();
    let hint = radical_hint().to_bytes().unwrap();
    let mut other_hint = radical_hint();
    other_hint.id = "other".into();
    let other_hint = other_hint.to_bytes().unwrap();
    let simplifier = simplifier().to_bytes().unwrap();
    for (quick, lexicon, language_model, radical_hint, simplifier) in [
        (
            &b"garbage"[..],
            &lexicon[..],
            &model[..],
            &hint[..],
            &simplifier[..],
        ),
        (
            &quick[..],
            &b"garbage"[..],
            &model[..],
            &hint[..],
            &simplifier[..],
        ),
        (
            &quick[..],
            &lexicon[..],
            &b"garbage"[..],
            &hint[..],
            &simplifier[..],
        ),
        (
            &quick[..],
            &lexicon[..],
            &model[..],
            &b"garbage"[..],
            &simplifier[..],
        ),
        (
            &quick[..],
            &lexicon[..],
            &model[..],
            &hint[..],
            &b"garbage"[..],
        ),
        // 都解得開，但詞庫、語言模型或字根提示不是方案指名的那份。
        (
            &quick[..],
            &quick[..],
            &model[..],
            &hint[..],
            &simplifier[..],
        ),
        (
            &quick[..],
            &lexicon[..],
            &other_model[..],
            &hint[..],
            &simplifier[..],
        ),
        (
            &quick[..],
            &lexicon[..],
            &model[..],
            &other_hint[..],
            &simplifier[..],
        ),
    ] {
        let bytes = PackBytes {
            quick,
            lexicon,
            language_model,
            radical_hint,
            simplifier,
        };
        let report = verify::run(&rows(), bytes, bytes);
        assert_eq!(failed(&report), [CheckId::Decodes]);
    }
}

#[test]
fn language_model_over_the_size_limit_fails_the_size_check() {
    let model = |p: f64| CharModel {
        start: p.ln(),
        continuation: p.ln(),
        backoff: 0.0,
    };
    let mut big = language_model();
    // 每個字的 unigram 存 12 位元組，70 萬字就超過 8 MB。
    let chars = (0..=0x10FFFF)
        .filter_map(char::from_u32)
        .take(700_000)
        .map(|c| (c, model(1e-6)));
    big.model = LanguageModel::new(chars.chain([('日', model(0.5))]), model(1e-7), []);
    let report = run_with_language_model(&pack(), &lexicon(), &big);
    assert!(big.to_bytes().unwrap().len() > bigram::MAX_BYTES);
    assert_eq!(failed(&report), [CheckId::LanguageModelSize]);
    // 剛建好的小模型不超過。
    assert_eq!(failed(&run(&pack())), []);
}

#[test]
fn hint_without_a_character_of_the_code_table_fails_the_coverage_check() {
    let hint = edited_hint(|chars| {
        chars.remove(&'嘅');
    });
    assert_eq!(
        failed(&run_with_hint(&hint)),
        [CheckId::RadicalHintCoversCharacters]
    );
}

#[test]
fn hint_with_cangjie3_and_cangjie5_swapped_fails_the_version_check() {
    let hint = edited_hint(|chars| {
        let diu = chars.get_mut(&'丟').unwrap();
        std::mem::swap(&mut diu.cangjie3, &mut diu.cangjie5);
    });
    assert_eq!(
        failed(&run_with_hint(&hint)),
        [CheckId::RadicalHintByVersion]
    );
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
