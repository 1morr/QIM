use std::collections::BTreeMap;

use qim_core::{
    CangjieCodes, CharModel, CodeEntry, CodeTable, ConversionTable, EnglishPack, EnglishWords,
    LanguageModel, LanguageModelPack, Lexicon, LexiconPack, RadicalHintPack, RadicalHints,
    SchemePack, Simplifier, SimplifierPack, Word,
};
use qim_databuild::common_forms::BothForms;
use qim_databuild::libcangjie::{self, Row};
use qim_databuild::transcode::Transcoder;
use qim_databuild::verify::{self, CheckId, PackBytes, Report};
use qim_databuild::{bigram, build_scheme_pack, lexicon, radical_hint, scheme};

/// 取自 libcangjie `346026b`，涵蓋 plan §4.5 的抽查字與「門口」，加一個擴展區的字；
/// 廿、竹、水讓 `the` 打得出中文（英文候選的抽查）。
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
廿 廿 1 1 0 0 1 0 0 0 0 t t NA 23000
竹 竹 1 1 0 0 1 0 0 0 0 h h NA 23001
水 水 1 1 0 0 1 0 0 0 0 e e NA 23002
";

const SCHEME: &str = r#"
id = "t"
name = "測試"
version = "1"
alphabet = "abehnrt"
max_code_length = 2
[packs]
lexicon = "t-lexicon"
language_model = "t-bigram"
radical_hint = "t-hint"
[radicals]
a = "日"
b = "月"
e = "水"
h = "竹"
n = "弓"
r = "口"
t = "廿"
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
fn pack() -> SchemePack {
    build_scheme_pack(
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
        &lexicon::Forms {
            normalize: &|s| s.to_string(),
            both: &BothForms::default(),
        },
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
        model: bigram::train(&counter, 8, 1, &BothForms::default()).0,
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

/// 有 `the` 的英文資料包。
fn english() -> EnglishPack {
    english_with(&["the", "an"])
}

fn english_with(words: &[&str]) -> EnglishPack {
    EnglishPack {
        id: "scowl-en-us".into(),
        words: EnglishWords::new(words.iter().copied()),
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

fn run_with(pack: &SchemePack, lexicon: &LexiconPack) -> Report {
    run_with_language_model(pack, lexicon, &language_model())
}

fn run_with_language_model(
    pack: &SchemePack,
    lexicon: &LexiconPack,
    language_model: &LanguageModelPack,
) -> Report {
    report(pack, lexicon, language_model, &radical_hint())
}

fn report(
    pack: &SchemePack,
    lexicon: &LexiconPack,
    language_model: &LanguageModelPack,
    radical_hint: &RadicalHintPack,
) -> Report {
    let quick = pack.to_bytes().unwrap();
    let lexicon = lexicon.to_bytes().unwrap();
    let language_model = language_model.to_bytes().unwrap();
    let radical_hint = radical_hint.to_bytes().unwrap();
    let simplifier = simplifier().to_bytes().unwrap();
    let english = english().to_bytes().unwrap();
    let bytes = PackBytes {
        quick: &quick,
        lexicon: &lexicon,
        language_model: &language_model,
        radical_hint: &radical_hint,
        simplifier: &simplifier,
        english: &english,
    };
    verify::run(&rows(), bytes, bytes)
}

fn run(pack: &SchemePack) -> Report {
    run_with(pack, &lexicon())
}

/// 改動碼表後重建資料包。
fn edited(edit: impl FnOnce(&mut BTreeMap<String, Vec<CodeEntry>>)) -> SchemePack {
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
    assert_eq!((report.continuous, report.extension), (10, 1));
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
            CheckId::EnglishThe,
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
    let bytes = build_scheme_pack(&rows, &BTreeMap::new(), scheme::parse(SCHEME).unwrap())
        .to_bytes()
        .unwrap();
    let lexicon = lexicon().to_bytes().unwrap();
    let language_model = language_model().to_bytes().unwrap();
    let radical_hint = radical_hint_from(radical_hint::hints(&rows))
        .to_bytes()
        .unwrap();
    let simplifier = simplifier().to_bytes().unwrap();
    let english = english().to_bytes().unwrap();
    let both = PackBytes {
        quick: &bytes,
        lexicon: &lexicon,
        language_model: &language_model,
        radical_hint: &radical_hint,
        simplifier: &simplifier,
        english: &english,
    };
    let report = verify::run(&rows, both, both);
    assert_eq!(failed(&report), []);
    assert_eq!((report.continuous, report.extension), (11, 1));
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
    let english = english().to_bytes().unwrap();
    let bytes = PackBytes {
        quick: &quick,
        lexicon: &lexicon,
        language_model: &model,
        radical_hint: &hint,
        simplifier: &identity,
        english: &english,
    };
    assert_eq!(
        failed(&verify::run(&rows(), bytes, bytes)),
        [CheckId::SimplifiedMenKou]
    );
}

/// 英文資料包沒有 `the`：中文狀態打 `the` 沒有英文候選。
#[test]
fn an_english_pack_without_the_fails_the_spot_check() {
    let quick = pack().to_bytes().unwrap();
    let lexicon = lexicon().to_bytes().unwrap();
    let model = language_model().to_bytes().unwrap();
    let hint = radical_hint().to_bytes().unwrap();
    let simplifier = simplifier().to_bytes().unwrap();
    let english = english_with(&["then"]).to_bytes().unwrap();
    let bytes = PackBytes {
        quick: &quick,
        lexicon: &lexicon,
        language_model: &model,
        radical_hint: &hint,
        simplifier: &simplifier,
        english: &english,
    };
    assert_eq!(
        failed(&verify::run(&rows(), bytes, bytes)),
        [CheckId::EnglishThe]
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
        english().to_bytes().unwrap(),
    ];
    fn bytes(b: &[Vec<u8>; 6]) -> PackBytes<'_> {
        PackBytes {
            quick: &b[0],
            lexicon: &b[1],
            language_model: &b[2],
            radical_hint: &b[3],
            simplifier: &b[4],
            english: &b[5],
        }
    }
    for changed in 0..6 {
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
    let english = english().to_bytes().unwrap();
    for (quick, lexicon, language_model, radical_hint, simplifier, english) in [
        (
            &b"garbage"[..],
            &lexicon[..],
            &model[..],
            &hint[..],
            &simplifier[..],
            &english[..],
        ),
        (
            &quick[..],
            &b"garbage"[..],
            &model[..],
            &hint[..],
            &simplifier[..],
            &english[..],
        ),
        (
            &quick[..],
            &lexicon[..],
            &b"garbage"[..],
            &hint[..],
            &simplifier[..],
            &english[..],
        ),
        (
            &quick[..],
            &lexicon[..],
            &model[..],
            &b"garbage"[..],
            &simplifier[..],
            &english[..],
        ),
        (
            &quick[..],
            &lexicon[..],
            &model[..],
            &hint[..],
            &b"garbage"[..],
            &english[..],
        ),
        (
            &quick[..],
            &lexicon[..],
            &model[..],
            &hint[..],
            &simplifier[..],
            &b"garbage"[..],
        ),
        // 都解得開，但詞庫、語言模型或字根提示不是方案指名的那份。
        (
            &quick[..],
            &quick[..],
            &model[..],
            &hint[..],
            &simplifier[..],
            &english[..],
        ),
        (
            &quick[..],
            &lexicon[..],
            &other_model[..],
            &hint[..],
            &simplifier[..],
            &english[..],
        ),
        (
            &quick[..],
            &lexicon[..],
            &model[..],
            &other_hint[..],
            &simplifier[..],
            &english[..],
        ),
    ] {
        let bytes = PackBytes {
            quick,
            lexicon,
            language_model,
            radical_hint,
            simplifier,
            english,
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
    /// 「有前文」的題：前文原樣，可含字集外的字。
    const CONTEXT: &str = "ATM，𠀀。\tar a\t問日\n";

    fn failed(bench: &str, train: &str, dev: &str, second: [&str; 5]) -> Vec<CheckId> {
        failed_with_context(bench, CONTEXT, train, dev, second)
    }

    fn failed_with_context(
        bench: &str,
        context: &str,
        train: &str,
        dev: &str,
        second: [&str; 5],
    ) -> Vec<CheckId> {
        let transcoder = Transcoder::new(&libcangjie::parse(TABLE).unwrap());
        let digests = |[bench, context_bench, train, dev, chars]: [&str; 5]| CorpusDigests {
            bench: bench.into(),
            context_bench: context_bench.into(),
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
            context,
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

    const SAME: [&str; 5] = ["b", "x", "t", "d", "c"];

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
    fn context_item_with_a_wrong_code_or_missing_its_context_fails() {
        let transcodes = |context: &str| failed_with_context(BENCH, context, TRAIN, DEV, SAME);
        assert_eq!(
            transcodes(&CONTEXT.replace("ar a", "an a")),
            [CheckId::CorpusTranscodes]
        );
        assert_eq!(transcodes("ar a\t問日\n"), [CheckId::CorpusTranscodes]);
        assert_eq!(
            transcodes("前文\t日𠀀\t日𠀀\n"),
            [CheckId::CorpusTranscodes]
        );
        assert_eq!(transcodes(""), []);
    }

    #[test]
    fn context_item_sentence_in_a_train_is_a_leak() {
        assert_eq!(
            leaks(&[("a 有前文", CONTEXT)], &["問日\n"]),
            [(false, "a 有前文 test 句原樣出現在 train 1 句".into())]
        );
    }

    #[test]
    fn different_second_build_fails() {
        assert_eq!(
            failed(BENCH, TRAIN, DEV, ["b", "x", "t", "y", "c"]),
            [CheckId::CorpusDeterministic]
        );
        assert_eq!(
            failed(BENCH, TRAIN, DEV, ["b", "x", "t", "d", "y"]),
            [CheckId::CorpusDeterministic]
        );
    }
}

mod kept_characters {
    use std::collections::BTreeMap;

    use qim_databuild::verify::{self, CheckId};

    fn check(frequency: &[(char, u64)]) -> (bool, String) {
        let frequency: BTreeMap<char, u64> = frequency.iter().copied().collect();
        let check = verify::kept_characters(&frequency, &['晒', '搵']);
        assert_eq!(check.id, CheckId::KeptCharacters);
        (check.passed, check.detail)
    }

    #[test]
    fn every_kept_character_counted_in_train_passes() {
        assert_eq!(
            check(&[('晒', 3), ('搵', 1), ('曬', 9)]),
            (true, "晒 3、搵 1".into())
        );
    }

    #[test]
    fn kept_character_missing_or_zero_in_train_fails() {
        assert_eq!(check(&[('晒', 3), ('揾', 5)]), (false, "晒 3、搵 0".into()));
        assert_eq!(check(&[('晒', 0), ('搵', 2)]), (false, "晒 0、搵 2".into()));
    }
}

mod common_forms {
    use std::collections::BTreeMap;

    use qim_core::{CharModel, CodeEntry, CodeTable, LanguageModel, Lexicon, Word};
    use qim_databuild::common_forms::CommonForms;
    use qim_databuild::verify::{self, CheckId};

    fn check(frequency: &[(char, u64)]) -> (bool, String) {
        let entry = |text| CodeEntry {
            text,
            extension: false,
        };
        let table = CodeTable::from_entries([
            ("et".to_string(), vec![entry('溫'), entry('温')]),
            ("sq".into(), vec![entry('群'), entry('羣')]),
        ]);
        let forms = CommonForms::from_opencc(&table).unwrap();
        let frequency: BTreeMap<char, u64> = frequency.iter().copied().collect();
        let check = verify::common_forms(&frequency, &forms);
        assert_eq!(check.id, CheckId::CommonForms);
        (check.passed, check.detail)
    }

    #[test]
    fn no_replaced_form_left_in_train_passes() {
        assert_eq!(
            check(&[('溫', 5), ('群', 2)]),
            (
                true,
                "對照表 2 組（温→溫、羣→群），換掉的寫法在 train 都沒有".into()
            )
        );
    }

    #[test]
    fn replaced_form_left_in_train_fails() {
        assert_eq!(
            check(&[('溫', 5), ('温', 3), ('羣', 1)]),
            (
                false,
                "對照表 2 組（温→溫、羣→群），換掉的寫法還在 train：温 3、羣 1".into()
            )
        );
    }

    /// 碼表有溫温、群羣（同碼）、雙向的並存寫法裏（`yv`）／裡（`lg`）與單向的着（`tu`）→ 著（`ta`）。
    fn forms() -> CommonForms {
        let entry = |text| CodeEntry {
            text,
            extension: false,
        };
        let table = CodeTable::from_entries([
            ("et".to_string(), vec![entry('溫'), entry('温')]),
            ("sq".into(), vec![entry('群'), entry('羣')]),
            ("yv".into(), vec![entry('裏')]),
            ("lg".into(), vec![entry('裡')]),
            ("tu".into(), vec![entry('着')]),
            ("ta".into(), vec![entry('著')]),
        ]);
        CommonForms::from_opencc(&table).unwrap()
    }

    #[test]
    fn other_both_form_left_in_train_fails() {
        // 單向的「著」本來就各算各的，不算留下。
        let frequency = BTreeMap::from([('裏', 5), ('裡', 2), ('着', 3), ('著', 4)]);
        let check = verify::common_forms(&frequency, &forms());
        assert!(!check.passed);
        assert!(
            check.detail.ends_with("還在 train：裡 2"),
            "{}",
            check.detail
        );
    }

    fn lexicon_check(words: &[(&str, &str, u64)], chars: &[(char, u64)]) -> (bool, String) {
        lexicon_check_with(words, chars, &model(&[]))
    }

    /// 裏、裡、這、隨的機率相同、「這裏」「這裡」的 bigram 相同的語言模型；`overrides` 蓋掉其中幾對。
    fn model(overrides: &[(char, char, f64)]) -> LanguageModel {
        let char_model = CharModel {
            start: -5.0,
            continuation: -6.0,
            backoff: -0.5,
        };
        let chars = ['這', '裏', '裡', '隨', '着', '著'].map(|c| (c, char_model));
        let bigrams = [('這', '裏', -1.0), ('這', '裡', -1.0)]
            .into_iter()
            .chain(overrides.iter().copied());
        LanguageModel::new(chars, char_model, bigrams)
    }

    fn lexicon_check_with(
        words: &[(&str, &str, u64)],
        chars: &[(char, u64)],
        model: &LanguageModel,
    ) -> (bool, String) {
        let mut index: BTreeMap<String, Vec<Word>> = BTreeMap::new();
        for &(code, text, frequency) in words {
            index.entry(code.into()).or_default().push(Word {
                text: text.into(),
                frequency,
            });
        }
        let lexicon = Lexicon::new(index, chars.iter().copied());
        let check = verify::lexicon_forms(&lexicon, model, &forms());
        assert_eq!(check.id, CheckId::LexiconForms);
        (check.passed, check.detail)
    }

    /// 都合規的詞庫：群組；這裏、這裡同頻；隨着與它的另一種寫法隨著（essay 也有隨著，詞頻較高）；著名。
    const GOOD: [(&str, &str, u64); 5] = [
        ("sqvm", "群組", 500),
        ("yyv", "這裏", 80),
        ("ylg", "這裡", 80),
        ("nlbtu", "隨着", 50),
        ("nlbta", "隨著", 70),
    ];

    #[test]
    fn lexicon_in_common_forms_with_every_spelling_passes() {
        let mut words = GOOD.to_vec();
        words.push(("tani", "著名", 900));
        assert_eq!(
            lexicon_check(&words, &[('裏', 9), ('裡', 9), ('着', 3), ('著', 7)]),
            (
                true,
                "對照表 2 組換掉的寫法與反向的錯字在詞庫都沒有；雙向並存寫法 1 組：含它的詞 2 個，\
                 每種寫法都收、詞頻相同，字頻與 bigram 相同；單向 1 組：含它的詞 1 個都另收另一種寫法"
                    .into()
            )
        );
    }

    #[test]
    fn replaced_form_or_reversed_one_way_word_in_the_lexicon_fails() {
        let (passed, detail) = lexicon_check(&[("sqvm", "羣組", 500)], &[]);
        assert!(!passed);
        assert!(detail.contains("不該有的寫法還在詞庫：羣組"), "{detail}");
        // 「著名」反過來生出「着名」。
        let (passed, detail) = lexicon_check(&[("tani", "著名", 900), ("tuni", "着名", 900)], &[]);
        assert!(!passed);
        assert!(detail.contains("不該有的寫法還在詞庫：着名"), "{detail}");
    }

    #[test]
    fn missing_spelling_or_unequal_frequency_fails() {
        let (passed, detail) = lexicon_check(&[("nlbtu", "隨着", 50)], &[]);
        assert!(!passed);
        assert!(detail.contains("缺：隨著"), "{detail}");
        let (passed, detail) = lexicon_check(&[("yyv", "這裏", 80)], &[]);
        assert!(!passed);
        assert!(detail.contains("缺：這裡"), "{detail}");
        let (passed, detail) = lexicon_check(
            &[
                ("yyv", "這裏", 80),
                ("ylg", "這裡", 70),
                ("nlbtu", "隨着", 50),
                ("nlbta", "隨著", 40),
            ],
            &[('裏', 9), ('裡', 8)],
        );
        assert!(!passed);
        assert!(
            detail.contains("詞頻不合：這裏 80／這裡 70、隨着 50／隨著 40；"),
            "{detail}"
        );
        assert!(detail.contains("字頻不同：裏 9、裡 8"), "{detail}");
    }

    #[test]
    fn two_way_forms_with_different_bigrams_fail() {
        // 「這裡」的 bigram 沒照抄「這裏」：bigram 傳了空的並存寫法（M1 票 17d 的接線變異）。
        let chars = [('裏', 9), ('裡', 9), ('着', 3), ('著', 7)];
        let model = model(&[('這', '裡', -3.0)]);
        let (passed, detail) = lexicon_check_with(&GOOD, &chars, &model);
        assert!(!passed);
        assert!(detail.contains("bigram 不同：這裏／這裡"), "{detail}");
        // 單向的「着／著」各算各的，不比。
        let model = self::model(&[('隨', '著', -3.0)]);
        let (passed, detail) = lexicon_check_with(&GOOD, &chars, &model);
        assert!(passed, "{detail}");
    }

    #[test]
    fn unrelated_words_and_one_way_frequencies_do_not_fail() {
        // 單向的「着／著」字頻各算各的；不含並存寫法的詞不必有別的寫法。
        let mut words = GOOD.to_vec();
        words.push(("ar", "日口", 3));
        let (passed, detail) =
            lexicon_check(&words, &[('裏', 4), ('裡', 4), ('着', 1), ('著', 99)]);
        assert!(passed, "{detail}");
    }
}
