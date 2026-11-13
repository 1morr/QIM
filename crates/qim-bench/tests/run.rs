//! 經 [`qim_bench::run`] 跑整條基準：暫存目錄擺出 repo 的佈局（資料包、`bench/`），
//! 數字都是照下面的小資料包手算的。
//!
//! `toy` 三句的模擬（plan〈首選字基準〉，`docs/development.md`〈首選字基準〉）：
//! - 第 1 行 `a 日`：`a` 空白，2 鍵、修正 0、跳動 0，答對。
//! - 第 2 行 `a 曰`：`a` 後首選「日」接不上，按 2 選「曰」，2 鍵、修正 1，答錯（首選「日」，錯 1 字）。
//! - 第 3 行 `a b 日月`：`ab` 的首選「明」接不上又沒有下一頁，Esc（修正 1）後一個字一個字打：
//!   `a` 空白、`b` 空白，共 7 鍵；逐鍵打時 `a` 的首選「日」被 `b` 改成「明」，跳動 1；答錯（錯 2 字）。
//!
//! 合計修正 2、按鍵 11、字數 4（KSPC 2.75）、跳動 1，答錯第 2、3 行。
//!
//! 每鍵延遲一律用假計時器，數字固定、不受機器負載影響：原生用 [`fast_native`]（在上限內），
//! 測延遲閘門的測試另給慢的 [`fixed_native`]；WebAssembly 用 [`slow_wasm`]：每鍵都遠超預算，它只報，
//! `--check` 與 baseline 照樣不受影響。

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use qim_bench::{Mode, Outcome, PERSONAL_SOURCE_HEADER, Personal, Timers, Timing, WasmTimer};
use qim_core::{
    CharModel, CodeEntry, CodeTable, EmojiPack, EmojiTable, Engine, EnglishPack, EnglishWords, Key,
    LanguageModel, LanguageModelPack, Lexicon, LexiconPack, NgramRow, Pack, Packs, Ranking, Scheme,
    SchemePack, Session, Snapshot, UserModel, Word, WordTrigram, WordTrigramPack,
};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

fn entry(text: char) -> CodeEntry {
    CodeEntry {
        text,
        extension: false,
        simplified: false,
    }
}

/// `a` → 日曰、`b` → 月、`ab` → 明、`c` → 一到十（兩頁）；`table` 可以換掉整張碼表造退步。
fn pack_bytes(table: &[(&str, &str)]) -> Vec<u8> {
    pack_bytes_with(table, None, None)
}

/// `lexicon`、`language_model` 是方案指名的詞庫、語言模型資料包 id。
fn pack_bytes_with(
    table: &[(&str, &str)],
    lexicon: Option<&str>,
    language_model: Option<&str>,
) -> Vec<u8> {
    scheme_bytes(table, lexicon, language_model, None)
}

/// 同 [`pack_bytes_with`]，另指名詞級 trigram 資料包 `word_trigram`。
fn scheme_bytes(
    table: &[(&str, &str)],
    lexicon: Option<&str>,
    language_model: Option<&str>,
    word_trigram: Option<&str>,
) -> Vec<u8> {
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        // 基準另量含萬用鍵的輸入，方案沒有萬用鍵時整個基準失敗；題目都不含 `z`，打分不受影響。
        wildcard: Some('z'),
        radicals: BTreeMap::new(),
        packs: Packs {
            lexicon: lexicon.map(Into::into),
            language_model: language_model.map(Into::into),
            word_trigram: word_trigram.map(Into::into),
            ..Packs::default()
        },
        ranking: Ranking::default(),
    };
    let code_table = CodeTable::from_entries(
        table
            .iter()
            .map(|(code, chars)| (code.to_string(), chars.chars().map(entry).collect())),
    );
    SchemePack { scheme, code_table }.to_bytes().unwrap()
}

const TABLE: &[(&str, &str)] = &[
    ("a", "日曰"),
    ("b", "月"),
    ("ab", "明"),
    ("c", "一二三四五六七八九十"),
];

/// 英文資料包：基準照網頁掛上（預設開英文候選）。`aba` 是英文字，但沒有題目打它。
fn english_bytes() -> Vec<u8> {
    EnglishPack {
        id: "scowl-en-us".into(),
        words: EnglishWords::new(["aba"]),
    }
    .to_bytes()
    .unwrap()
}

/// emoji 資料包：基準照網頁掛上（預設開 emoji 候選）。「月」有 emoji，沒有題目的正解是它。
fn emoji_bytes() -> Vec<u8> {
    EmojiPack {
        id: "cldr-emoji-yue-hant".into(),
        emoji: EmojiTable::new([("月", ["🌙"])]),
    }
    .to_bytes()
    .unwrap()
}

/// 固定題 `a 日`；語料 `toy` 三句，見檔頭。
fn repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    write(
        dir.path(),
        "build/packs/quick/quick.pack",
        &pack_bytes(TABLE),
    );
    write(dir.path(), qim_bench::ENGLISH_PACK, &english_bytes());
    write(dir.path(), qim_bench::EMOJI_PACK, &emoji_bytes());
    write(
        dir.path(),
        "bench/fixed.tsv",
        "# 輸入碼\t期望首選\t來源\na\t日\t測試\n".as_bytes(),
    );
    write(
        dir.path(),
        "bench/corpus/toy.tsv",
        "a\t日\na\t曰\na b\t日月\n".as_bytes(),
    );
    dir
}

fn run(root: &Path, mode: Mode) -> Result<Outcome, String> {
    qim_bench::run(root, mode, None, with_wasm(&slow_wasm))
}

/// 原生用 [`fast_native`]、WebAssembly 用 `wasm`。
fn with_wasm(wasm: WasmTimer) -> Timers {
    Timers {
        native: &fast_native,
        wasm,
    }
}

/// 假的原生計時器：每鍵 0.05 ms、選開頭一段 0.5 ms，都在每鍵延遲上限內。
fn fast_native(engine: &Engine, user: &UserModel, inputs: &[String]) -> Timing {
    fixed_native(0.05, 0.5)(engine, user, inputs)
}

/// 假的原生計時器：每鍵 `key_ms`、選開頭一段 `select_ms` 毫秒（[`fake_timing`]）。
fn fixed_native(key_ms: f64, select_ms: f64) -> impl Fn(&Engine, &UserModel, &[String]) -> Timing {
    move |engine, user, inputs| fake_timing(engine, user, inputs, |_| key_ms, select_ms)
}

/// 假的 WebAssembly 計時器：每個輸入的第 i 鍵（從 1 起）花 100 × i 毫秒（有學習資料時 200 × i），
/// 遠超每鍵 50 ms 的預算。選開頭一段花 1000 毫秒。用基準交給它的資料包與匯入的學習資料打，
/// 同 `web/bench/latency.ts`。
fn slow_wasm(
    packs: &[PathBuf],
    user_dict: Option<&str>,
    inputs: &[String],
) -> Result<Timing, String> {
    let engine = engine_from(packs);
    let per_key = if user_dict.is_some() { 200.0 } else { 100.0 };
    let user = user_dict.map_or_else(UserModel::default, |text| {
        UserModel::import(text, None).unwrap().user
    });
    Ok(Timing {
        load_ms: Some(12.5),
        ..fake_timing(&engine, &user, inputs, |i| per_key * i as f64, 1000.0)
    })
}

/// 不量時間的計時結果：在 `engine` 與學習資料 `user` 下逐鍵打每個輸入，第 i 鍵（從 1 起）記
/// `key_ms(i)` 毫秒；量哪些輸入的選開頭一段照原生的規則（[`qim_bench::lefts_to_first_segment`]），
/// 各記 `select_ms` 毫秒。
fn fake_timing(
    engine: &Engine,
    user: &UserModel,
    inputs: &[String],
    key_ms: impl Fn(usize) -> f64,
    select_ms: f64,
) -> Timing {
    let mut user = user.clone();
    Timing {
        runtime: "假".into(),
        per_key_ms: inputs
            .iter()
            .map(|input| (1..=input.chars().count()).map(&key_ms).collect())
            .collect(),
        select_ms: inputs
            .iter()
            .map(|input| {
                let mut session = Session::default();
                let mut last = Snapshot::default();
                for c in input.chars() {
                    last = session.handle(engine, &mut user, Key::for_input(c));
                }
                qim_bench::lefts_to_first_segment(&last).map(|_| select_ms)
            })
            .collect(),
        load_ms: None,
    }
}

/// 基準交給計時器的資料包（依檔頭魔數分辨，同 `web/bench/latency.ts`）建出的 `Engine`。
fn engine_from(packs: &[PathBuf]) -> Engine {
    let mut by_kind = BTreeMap::new();
    for path in packs {
        let bytes = fs::read(path).unwrap();
        by_kind.insert(bytes[..4].to_vec(), bytes);
    }
    let get = |magic: &[u8]| by_kind.get(magic).map(Vec::as_slice);
    let engine =
        Engine::from_pack_bytes(get(b"QIMP").unwrap(), get(b"QIML"), get(b"QIMB")).unwrap();
    let engine = match get(b"QIME") {
        Some(bytes) => engine.with_english(EnglishPack::from_bytes(bytes).unwrap()),
        None => engine,
    };
    match get(b"QIMJ") {
        Some(bytes) => engine.with_emoji(EmojiPack::from_bytes(bytes).unwrap()),
        None => engine,
    }
}

fn write(root: &Path, path: &str, bytes: &[u8]) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

#[test]
fn report_counts_fixed_passes_and_corpus_metrics() {
    let repo = repo();
    let outcome = run(repo.path(), Mode::Report).unwrap();

    assert_eq!(outcome.fixed.total(), 1);
    assert_eq!(outcome.fixed.wrong(), 0);

    let [toy] = outcome.sets.as_slice() else {
        panic!("應該只有一個語料集合：{:?}", outcome.sets);
    };
    assert_eq!(toy.name, "toy");
    assert_eq!(toy.total(), 3);
    assert_eq!(toy.correct(), 1);
    assert_eq!(toy.wrong(), 2);
    let m = toy.metrics(false);
    assert_eq!(m.char_accuracy, 0.25); // 1 − 3 ÷ 4
    assert!(outcome.violations.is_empty());
}

/// 固定題的輸入碼可以含分隔記號（網頁「回報」照打的記）：`a'b` 按的是 `a`、`'`、`b`，
/// 單字的碼不跨過分隔記號，所以首選是「日月」不是 `ab` 的「明」。
#[test]
fn a_fixed_item_with_a_separator_is_typed_with_the_separator_key() {
    let repo = repo();
    write(
        repo.path(),
        "bench/fixed.tsv",
        "a'b\t日月\t測試\n".as_bytes(),
    );
    let outcome = run(repo.path(), Mode::Report).unwrap();
    assert_eq!(outcome.fixed.total(), 1);
    assert_eq!(outcome.fixed.wrong(), 0, "{:?}", outcome.fixed);
}

fn check(repo: &TempDir) -> Vec<String> {
    run(repo.path(), Mode::Check).unwrap().violations
}

/// toy 這次的結果原樣當 baseline：答錯 2、修正 2、答錯第 2、3 行。
const TOY_BASELINE: &[u8] =
    br#"{"toy": {"wrong": 2, "correction_count": 2, "wrong_lines": "2-3"}}"#;

#[test]
fn simulation_counts_corrections_keystrokes_and_jitter() {
    let repo = repo();
    let outcome = run(repo.path(), Mode::Report).unwrap();
    let toy = &outcome.sets[0];
    let per_item: Vec<(usize, usize, usize, bool)> = toy
        .items
        .iter()
        .map(|i| (i.correction_count, i.keys, i.jitter, i.finished))
        .collect();
    // `a b` 日月：整串的候選是 明、日月、曰月…，選第 2 個「日月」（修正 1），共 3 鍵。
    assert_eq!(
        per_item,
        [(0, 2, 0, true), (1, 2, 0, true), (1, 3, 1, true)]
    );
    assert_eq!(toy.correction_count(), 2);
    let m = toy.metrics(false);
    assert_eq!(m.kspc, 7.0 / 4.0);
    assert_eq!(m.correction_count, 2.0 / 3.0);
    assert_eq!(m.jitter, 1.0 / 3.0);
}

#[test]
fn simulation_turns_the_page_when_the_first_page_does_not_connect() {
    let repo = repo();
    // `c` 的第 10 個候選「十」在第二頁：翻頁（修正 1）再按 1（修正 1），共 3 鍵。
    write(repo.path(), "bench/corpus/toy.tsv", "c\t十\n".as_bytes());
    let outcome = run(repo.path(), Mode::Report).unwrap();
    let item = &outcome.sets[0].items[0];
    assert_eq!(
        (item.correction_count, item.keys, item.finished),
        (2, 3, true)
    );
}

#[test]
fn simulation_selects_a_beginning_and_goes_on_composing_the_rest() {
    let repo = repo();
    // `cb` 整串的句子（一月、二月、三月）都接不上；第二頁的「十」只蓋住 `c`：翻頁、選「十」
    // （各算一次修正），「十」固定、`b` 繼續組字，首選「月」接得上就按空白。
    write(
        repo.path(),
        "bench/corpus/toy.tsv",
        "c b\t十月\n".as_bytes(),
    );
    let outcome = run(repo.path(), Mode::Report).unwrap();
    let item = &outcome.sets[0].items[0];
    assert_eq!(
        (item.correction_count, item.keys, item.finished),
        (2, 5, true)
    );
}

#[test]
fn an_item_that_cannot_be_typed_is_unfinished_and_fails_check() {
    let repo = repo();
    // 「目」不在碼表：`b` 的唯一候選「月」接不上，只有一個單位，無從重打。
    write(repo.path(), "bench/corpus/toy.tsv", "b\t目\n".as_bytes());
    write(
        repo.path(),
        "bench/baseline.json",
        br#"{"toy": {"wrong": 1, "correction_count": 9, "wrong_lines": "1"}}"#,
    );
    let outcome = run(repo.path(), Mode::Check).unwrap();
    assert_eq!(outcome.sets[0].unfinished(), 1);
    assert_eq!(outcome.violations.len(), 1, "{:?}", outcome.violations);
    assert!(
        outcome.violations[0].contains("打不出"),
        "{:?}",
        outcome.violations
    );
}

#[test]
fn length_groups_split_accuracy_and_corrections_by_target_length() {
    let repo = repo();
    let outcome = run(repo.path(), Mode::Report).unwrap();
    let groups = outcome.sets[0].length_groups();
    let one = groups[0].expect("有 1 字的題");
    assert_eq!(
        (one.total, one.sentence_accuracy, one.correction_count),
        (2, 0.5, 0.5)
    );
    let two = groups[1].expect("有 2 字的題");
    assert_eq!(
        (two.total, two.sentence_accuracy, two.correction_count),
        (1, 0.0, 1.0)
    );
    assert!(groups[2..].iter().all(Option::is_none));

    let report = outcome.report();
    let row = report
        .lines()
        .filter(|line| line.starts_with("| toy |"))
        .nth(1)
        .unwrap_or_else(|| panic!("沒有 toy 的分組列：{report}"));
    assert!(
        row.contains("| 50.00% · 0.50（2） | 0.00% · 1.00（1） | — |"),
        "{row}"
    );
}

#[test]
fn check_passes_when_counts_equal_baseline() {
    let repo = repo();
    write(repo.path(), "bench/baseline.json", TOY_BASELINE);
    assert_eq!(check(&repo), Vec::<String>::new());
}

#[test]
fn check_fails_when_wrong_count_exceeds_baseline() {
    let repo = repo();
    write(
        repo.path(),
        "bench/baseline.json",
        br#"{"toy": {"wrong": 1, "correction_count": 2, "wrong_lines": "2-3"}}"#,
    );
    let violations = check(&repo);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(violations[0].contains("答錯"), "{violations:?}");
}

#[test]
fn check_fails_when_corrections_exceed_baseline() {
    let repo = repo();
    write(
        repo.path(),
        "bench/baseline.json",
        br#"{"toy": {"wrong": 2, "correction_count": 1, "wrong_lines": "2-3"}}"#,
    );
    let violations = check(&repo);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(violations[0].contains("修正"), "{violations:?}");
}

#[test]
fn check_fails_when_a_pack_needs_more_corrections_with_the_same_wrong_count() {
    let repo = repo();
    write(repo.path(), "bench/baseline.json", TOY_BASELINE);
    // 「曰」移到第二頁：答錯數不變，第 2 行要多翻一頁。
    write(
        repo.path(),
        "build/packs/quick/quick.pack",
        &pack_bytes(&[("a", "日一二三四五六七八曰"), ("b", "月"), ("ab", "明")]),
    );
    let outcome = run(repo.path(), Mode::Check).unwrap();
    assert_eq!(outcome.sets[0].wrong(), 2);
    assert_eq!(outcome.violations.len(), 1, "{:?}", outcome.violations);
    assert!(
        outcome.violations[0].contains("修正 3 次"),
        "{:?}",
        outcome.violations
    );
}

#[test]
fn check_fails_when_a_pack_breaks_a_fixed_item() {
    let repo = repo();
    write(
        repo.path(),
        "bench/baseline.json",
        br#"{"toy": {"wrong": 3, "correction_count": 9, "wrong_lines": ""}}"#,
    );
    // 換一個 `a` 的字序相反的資料包：固定題 `a 日` 的首選變「曰」，語料仍在 baseline 內。
    write(
        repo.path(),
        "build/packs/quick/quick.pack",
        &pack_bytes(&[("a", "曰日"), ("b", "月"), ("ab", "明")]),
    );
    let violations = check(&repo);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(violations[0].contains("固定題"), "{violations:?}");
}

#[test]
fn check_fails_when_a_set_has_no_baseline() {
    let repo = repo();
    write(repo.path(), "bench/baseline.json", b"{}");
    let violations = check(&repo);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(violations[0].contains("toy"), "{violations:?}");
}

#[test]
fn check_fails_when_a_baseline_set_was_not_run() {
    let repo = repo();
    write(
        repo.path(),
        "bench/baseline.json",
        br#"{"toy": {"wrong": 2, "correction_count": 2, "wrong_lines": "2-3"},
             "gone": {"wrong": 0, "correction_count": 0, "wrong_lines": ""}}"#,
    );
    let violations = check(&repo);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(violations[0].contains("gone"), "{violations:?}");
}

#[test]
fn check_ignores_layout_of_baseline_and_comments_in_fixed() {
    let repo = repo();
    write(
        repo.path(),
        "bench/baseline.json",
        b"{\n\n      \"toy\"   :{\"wrong_lines\": \"2,3\", \"correction_count\" :2,\"wrong\":\n2 }\n}",
    );
    write(
        repo.path(),
        "bench/fixed.tsv",
        "# 換了說明\n\n# 輸入碼\t期望首選\t來源\na\t日\t另一個來源\n\n".as_bytes(),
    );
    assert_eq!(check(&repo), Vec::<String>::new());
}

#[test]
fn a_malformed_wrong_lines_is_an_error() {
    let repo = repo();
    write(
        repo.path(),
        "bench/baseline.json",
        br#"{"toy": {"wrong": 2, "correction_count": 2, "wrong_lines": "3-2"}}"#,
    );
    let error = run(repo.path(), Mode::Report).unwrap_err();
    assert!(error.contains("wrong_lines"), "{error}");
}

#[test]
fn update_baseline_rewrites_the_file_with_current_counts() {
    let repo = repo();
    write(
        repo.path(),
        "bench/baseline.json",
        br#"{"toy": {"wrong": 0, "correction_count": 0, "wrong_lines": ""},
             "gone": {"wrong": 0, "correction_count": 0, "wrong_lines": ""}}"#,
    );
    let outcome = run(repo.path(), Mode::UpdateBaseline).unwrap();
    assert!(outcome.violations.is_empty());
    let written = fs::read_to_string(repo.path().join("bench/baseline.json")).unwrap();
    assert_eq!(
        written,
        "{\n  \"toy\": {\n    \"wrong\": 2,\n    \"correction_count\": 2,\n    \"wrong_lines\": \"2-3\"\n  }\n}\n"
    );
    assert_eq!(check(&repo), Vec::<String>::new());
}

#[test]
fn report_lists_each_set_with_its_baseline() {
    let repo = repo();
    write(
        repo.path(),
        "bench/baseline.json",
        br#"{"toy": {"wrong": 5, "correction_count": 6, "wrong_lines": "2-3"}}"#,
    );
    let report = run(repo.path(), Mode::Report).unwrap().report();
    assert!(report.contains("固定題 1/1 通過"), "{report}");
    let toy = report
        .lines()
        .find(|line| line.starts_with("| toy |"))
        .unwrap_or_else(|| panic!("沒有 toy 那一列：{report}"));
    // 題數、整句首選正確、字正確率、答錯、baseline 答錯、修正/句、修正、baseline 修正、KSPC、跳動/句、打不出
    assert!(
        toy.contains("| 3 | 1（33.33%） | 25.00% | 2 | 5 | 0.67 | 2 | 6 | 1.75 | 0.33 | 0 |"),
        "{toy}"
    );
}

#[test]
fn report_runs_without_a_baseline_file() {
    let repo = repo();
    let report = run(repo.path(), Mode::Report).unwrap().report();
    assert!(report.contains("| toy |"), "{report}");
    assert!(report.contains("- toy：沒有 baseline"), "{report}");
}

#[test]
fn flips_list_an_item_that_went_from_right_to_wrong() {
    let repo = repo();
    // 刻意改壞 baseline：說第 2 行原本答對。
    write(
        repo.path(),
        "bench/baseline.json",
        br#"{"toy": {"wrong": 1, "correction_count": 2, "wrong_lines": "3"}}"#,
    );
    let outcome = run(repo.path(), Mode::Report).unwrap();
    let flips = outcome.flips("toy").unwrap();
    assert_eq!((flips.fixed, flips.broken), (vec![], vec![2]));
    let report = outcome.report();
    assert!(report.contains("- toy 由對變錯 1 題"), "{report}");
    assert!(
        report.contains("a（第 2 行）期望「曰」，首選「日」"),
        "{report}"
    );
}

#[test]
fn flips_show_both_directions_when_a_pack_swaps_two_answers() {
    let repo = repo();
    write(repo.path(), "bench/baseline.json", TOY_BASELINE);
    assert!(
        run(repo.path(), Mode::Report)
            .unwrap()
            .flips("toy")
            .unwrap()
            .is_empty()
    );
    // `a` 的字序相反：第 1 行由對變錯、第 2 行由錯變對，答錯數不變。
    write(
        repo.path(),
        "build/packs/quick/quick.pack",
        &pack_bytes(&[("a", "曰日"), ("b", "月"), ("ab", "明")]),
    );
    let outcome = run(repo.path(), Mode::Report).unwrap();
    assert_eq!(outcome.sets[0].wrong(), 2);
    let flips = outcome.flips("toy").unwrap();
    assert_eq!((flips.fixed, flips.broken), (vec![2], vec![1]));
    let report = outcome.report();
    assert!(report.contains("- toy 由錯變對 1 題"), "{report}");
    assert!(report.contains("- toy 由對變錯 1 題"), "{report}");
}

#[test]
fn check_ignores_the_order_of_sets_in_baseline() {
    let repo = repo();
    write(
        repo.path(),
        "bench/corpus/another.tsv",
        "b\t月\n".as_bytes(),
    );
    let toy = r#""toy": {"wrong": 2, "correction_count": 2, "wrong_lines": "2-3"}"#;
    let another = r#""another": {"wrong": 0, "correction_count": 0, "wrong_lines": ""}"#;
    write(
        repo.path(),
        "bench/baseline.json",
        format!("{{{toy}, {another}}}").as_bytes(),
    );
    assert_eq!(check(&repo), Vec::<String>::new());
    write(
        repo.path(),
        "bench/baseline.json",
        format!("{{{another}, {toy}}}").as_bytes(),
    );
    assert_eq!(check(&repo), Vec::<String>::new());
}

/// 報告裡指出資料包的那一行。
fn pack_line(repo: &TempDir) -> String {
    let report = run(repo.path(), Mode::Report).unwrap().report();
    report
        .lines()
        .find(|line| line.contains("資料包"))
        .unwrap_or_else(|| panic!("報告沒有指出資料包：{report}"))
        .to_string()
}

#[test]
fn a_missing_english_pack_fails_with_its_path() {
    let repo = repo();
    fs::remove_file(repo.path().join(qim_bench::ENGLISH_PACK)).unwrap();
    let error = run(repo.path(), Mode::Report).unwrap_err();
    assert!(error.contains(qim_bench::ENGLISH_PACK), "{error}");
}

#[test]
fn a_missing_emoji_pack_fails_with_its_path() {
    let repo = repo();
    fs::remove_file(repo.path().join(qim_bench::EMOJI_PACK)).unwrap();
    let error = run(repo.path(), Mode::Report).unwrap_err();
    assert!(error.contains(qim_bench::EMOJI_PACK), "{error}");
}

#[test]
fn report_names_the_pack_by_its_sha256_so_a_stale_pack_shows() {
    let repo = repo();
    let before = pack_line(&repo);
    assert_eq!(before, pack_line(&repo), "同一個資料包要印出一樣的指紋");
    write(
        repo.path(),
        "build/packs/quick/quick.pack",
        &pack_bytes(&[("a", "日曰"), ("b", "月")]),
    );
    assert_ne!(before, pack_line(&repo), "換了資料包，指紋要跟著變");
}

/// 詞庫資料包 `quick-lexicon`：`words` 是（碼, 詞）。
fn lexicon_bytes(words: &[(&str, &str)]) -> Vec<u8> {
    let lexicon = Lexicon::new(
        words.iter().map(|(code, text)| {
            let word = Word {
                text: text.to_string(),
                frequency: 100,
            };
            (code.to_string(), vec![word])
        }),
        [],
    );
    LexiconPack {
        id: "quick-lexicon".into(),
        lexicon,
    }
    .to_bytes()
    .unwrap()
}

#[test]
fn lexicon_named_by_the_scheme_is_loaded_next_to_the_pack_and_fingerprinted() {
    let repo = repo();
    write(
        repo.path(),
        "build/packs/quick/quick.pack",
        &pack_bytes_with(TABLE, Some("quick-lexicon"), None),
    );
    let err = run(repo.path(), Mode::Report).unwrap_err();
    assert!(
        err.contains("build/packs/quick-lexicon/quick-lexicon.pack"),
        "{err}"
    );

    const LEXICON: &str = "build/packs/quick-lexicon/quick-lexicon.pack";
    write(repo.path(), LEXICON, &lexicon_bytes(&[("ac", "日一")]));
    write(
        repo.path(),
        "bench/fixed.tsv",
        "a\t日\t測試\nac\t日一\t測試\n".as_bytes(),
    );
    let outcome = run(repo.path(), Mode::Report).unwrap();
    assert_eq!(outcome.fixed.failures.len(), 0, "{}", outcome.report());

    let before = pack_line(&repo);
    assert!(before.contains("quick-lexicon.pack"), "{before}");
    write(repo.path(), LEXICON, &lexicon_bytes(&[("ac", "日二")]));
    assert_ne!(before, pack_line(&repo), "換了詞庫，指紋要跟著變");
}

/// 語言模型資料包 `bigram`：只有「日 → 曰」一對，`a a` 的首選因此是「日曰」而不是「日日」。
fn language_model_bytes(next: char) -> Vec<u8> {
    let model = |p: f64| CharModel {
        start: p.ln(),
        continuation: p.ln(),
        backoff: 0.0,
    };
    LanguageModelPack {
        id: "bigram".into(),
        model: LanguageModel::new(
            [('日', model(0.5)), ('曰', model(0.1))],
            model(0.01),
            [('日', next, 0.9f64.ln())],
        ),
    }
    .to_bytes()
    .unwrap()
}

#[test]
fn language_model_named_by_the_scheme_is_loaded_next_to_the_pack_and_fingerprinted() {
    let repo = repo();
    write(
        repo.path(),
        "build/packs/quick/quick.pack",
        &pack_bytes_with(TABLE, None, Some("bigram")),
    );
    const LANGUAGE_MODEL: &str = "build/packs/bigram/bigram.pack";
    let err = run(repo.path(), Mode::Report).unwrap_err();
    assert!(err.contains(LANGUAGE_MODEL), "{err}");

    write(repo.path(), LANGUAGE_MODEL, &language_model_bytes('曰'));
    write(
        repo.path(),
        "bench/fixed.tsv",
        "a\t日\t測試\naa\t日曰\t測試\n".as_bytes(),
    );
    let outcome = run(repo.path(), Mode::Report).unwrap();
    assert_eq!(outcome.fixed.failures.len(), 0, "{}", outcome.report());

    let before = pack_line(&repo);
    assert!(before.contains("bigram.pack"), "{before}");
    write(repo.path(), LANGUAGE_MODEL, &language_model_bytes('日'));
    assert_ne!(before, pack_line(&repo), "換了語言模型，指紋要跟著變");
}

/// 詞級 trigram 資料包 `words`：「日」之後是 `next`。
fn word_trigram_bytes(next: char) -> Vec<u8> {
    WordTrigramPack {
        id: "words".into(),
        model: WordTrigram::new(
            vec!["日".into(), "曰".into()],
            vec![NgramRow {
                context: 0,
                backoff: 0.0,
                next: vec![(u32::from(next == '曰'), 0.95f64.ln())],
            }],
            vec![],
        ),
    }
    .to_bytes()
    .unwrap()
}

#[test]
fn word_trigram_named_by_the_scheme_is_loaded_next_to_the_pack_and_fingerprinted() {
    let repo = repo();
    write(
        repo.path(),
        "build/packs/quick/quick.pack",
        &scheme_bytes(TABLE, None, Some("bigram"), Some("words")),
    );
    write(
        repo.path(),
        "build/packs/bigram/bigram.pack",
        &language_model_bytes('曰'),
    );
    const WORDS: &str = "build/packs/words/words.pack";
    let err = run(repo.path(), Mode::Report).unwrap_err();
    assert!(err.contains(WORDS), "{err}");

    // 字級 bigram 說「日」之後是「曰」，詞級 trigram 說是「日」：詞級的先用。
    write(repo.path(), WORDS, &word_trigram_bytes('日'));
    write(
        repo.path(),
        "bench/fixed.tsv",
        "a\t日\t測試\naa\t日日\t測試\n".as_bytes(),
    );
    let outcome = run(repo.path(), Mode::Report).unwrap();
    assert_eq!(outcome.fixed.failures.len(), 0, "{}", outcome.report());

    let before = pack_line(&repo);
    assert!(before.contains("words.pack"), "{before}");
    write(repo.path(), WORDS, &word_trigram_bytes('曰'));
    assert_ne!(before, pack_line(&repo), "換了詞級 trigram，指紋要跟著變");
}

#[test]
fn report_gives_the_load_time_of_both_environments() {
    let report = run(repo().path(), Mode::Report).unwrap().report();
    let line = report
        .lines()
        .find(|line| line.starts_with("載入："))
        .unwrap_or_else(|| panic!("報告沒有載入時間：{report}"));
    assert!(line.contains("原生 ") && line.contains(" ms"), "{line}");
    assert!(line.contains("WebAssembly 12.5 ms"), "{line}");
}

/// 「有前文」的集合（plan〈首選字基準〉）：`bench/<目錄>/context/<名>.tsv`，一行「前文<TAB>逐字碼<TAB>句子」。
/// 語言模型只有「日 → 曰」，所以前文以「日」結尾時 `a` 的首選是「曰」。
fn repo_with_context() -> TempDir {
    let repo = repo();
    write(
        repo.path(),
        "build/packs/quick/quick.pack",
        &pack_bytes_with(TABLE, None, Some("bigram")),
    );
    write(
        repo.path(),
        "build/packs/bigram/bigram.pack",
        &language_model_bytes('曰'),
    );
    write(
        repo.path(),
        "bench/colloquial/context/talk.tsv",
        "今日\ta\t曰\nOK\tb\t月\n".as_bytes(),
    );
    repo
}

#[test]
fn context_sets_type_the_same_items_with_and_without_the_context() {
    let repo = repo_with_context();
    let outcome = run(repo.path(), Mode::Report).unwrap();
    let sets: Vec<(&str, usize, usize, usize)> = outcome
        .sets
        .iter()
        .map(|s| (s.name.as_str(), s.total(), s.wrong(), s.correction_count()))
        .collect();
    assert_eq!(
        sets,
        [
            ("toy", 3, 2, 2),
            ("talk·前文", 2, 0, 0),
            ("talk·前文·不給", 2, 1, 1),
        ]
    );
    let report = outcome.report();
    assert!(
        report.contains("| talk | 2 | 0 | 1 | 0 | 1 |\n| 合計 | 2 | 0 | 1 | 0 | 1 |"),
        "給與不給前文要並列：{report}"
    );

    // 兩種都進 baseline、都由 --check 擋。
    run(repo.path(), Mode::UpdateBaseline).unwrap();
    let baseline = fs::read_to_string(repo.path().join("bench/baseline.json")).unwrap();
    assert!(baseline.contains("talk·前文·不給"), "{baseline}");
    assert!(check(&repo).is_empty());
    write(
        repo.path(),
        "bench/colloquial/context/talk.tsv",
        "今日\ta\t日\nOK\tb\t月\n".as_bytes(),
    );
    let violations = check(&repo);
    assert!(
        violations.iter().any(|v| v.contains("talk·前文")),
        "{violations:?}"
    );
}

/// 學習模式也打「有前文」的集合，給前文。
#[test]
fn learning_mode_types_context_sets_with_the_context() {
    let repo = repo_with_context();
    let learning = qim_bench::run_learning(repo.path()).unwrap();
    let names: Vec<&str> = learning.sets.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["toy", "talk·前文"]);
    assert_eq!(learning.sets[1].corrections_off, 0);
}

#[test]
fn a_context_line_without_three_columns_is_an_error() {
    let repo = repo_with_context();
    write(
        repo.path(),
        "bench/colloquial/context/talk.tsv",
        "a\t曰\n".as_bytes(),
    );
    let error = run(repo.path(), Mode::Report).unwrap_err();
    assert!(
        error.contains("talk.tsv") && error.contains("第 1 行"),
        "{error}"
    );
}

#[test]
fn colloquial_sets_run_after_the_wiki_sets_and_need_a_baseline() {
    let repo = repo();
    write(
        repo.path(),
        "bench/colloquial/talk.tsv",
        "b\t月\na\t曰\n".as_bytes(),
    );
    let outcome = run(repo.path(), Mode::Report).unwrap();
    let names: Vec<&str> = outcome.sets.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["toy", "talk"]);
    assert_eq!(outcome.sets[1].wrong(), 1);

    write(repo.path(), "bench/baseline.json", TOY_BASELINE);
    let violations = check(&repo);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(violations[0].contains("talk"), "{violations:?}");
}

#[test]
fn the_same_set_name_in_two_directories_is_an_error() {
    let repo = repo();
    write(
        repo.path(),
        "bench/colloquial/toy.tsv",
        "a\t日\n".as_bytes(),
    );
    let error = run(repo.path(), Mode::Report).unwrap_err();
    assert!(error.contains("toy"), "{error}");
}

// 個人集：bench/personal/rime-userdb.tsv，一行「逐字碼<TAB>詞<TAB>上屏次數」。

/// 個人集兩題（題目檔第 2、3 行，第 1 行是記來源 sha256 的檔頭）：
/// 第 2 行「月」上屏 5 次（答對，0 修正），第 3 行「曰」上屏 1 次（答錯，修正 1）。
/// toy 的 baseline 照這次結果寫好，toy 沒有翻盤，報告裡不會因為 toy 印出「曰」。
fn repo_with_personal() -> TempDir {
    let repo = repo();
    write(repo.path(), "bench/baseline.json", TOY_BASELINE);
    transcode_personal(&repo, "使用者的學習資料");
    repo
}

/// 照 `cargo xtask data` 的做法寫出個人集題目檔：檔頭記 `userdb` 的 sha256。
fn transcode_personal(repo: &TempDir, userdb: &str) {
    write(
        repo.path(),
        "bench/personal/rime-userdb.txt",
        userdb.as_bytes(),
    );
    let sha256: String = Sha256::digest(userdb.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    write(
        repo.path(),
        "bench/personal/rime-userdb.tsv",
        format!("{PERSONAL_SOURCE_HEADER}{sha256}\nb\t月\t5\na\t曰\t1\n").as_bytes(),
    );
}

const PERSONAL_BASELINE: &[u8] =
    br#"{"personal": {"wrong": 1, "correction_count": 1, "wrong_lines": "3"}}"#;

#[test]
fn personal_reports_unweighted_and_weighted_numbers() {
    let repo = repo_with_personal();
    let outcome = run(repo.path(), Mode::Report).unwrap();
    let Personal::Ran(personal) = &outcome.personal else {
        panic!("個人集應該有跑：{:?}", outcome.personal);
    };
    assert_eq!(personal.metrics(false).sentence_accuracy, 0.5);
    assert_eq!(personal.metrics(true).sentence_accuracy, 5.0 / 6.0);
    assert_eq!(personal.metrics(true).correction_count, 1.0 / 6.0);

    let report = outcome.report();
    assert!(
        report.contains("| personal | 2 | 1（50.00%） |"),
        "{report}"
    );
    assert!(
        report.contains("| personal（加權） | 6 次 | 83.33% |"),
        "{report}"
    );
}

#[test]
fn personal_output_has_numbers_and_line_numbers_only() {
    let repo = repo_with_personal();
    // baseline 說第 2 行原本答錯、第 3 行原本答對：兩個方向都會列出。
    write(
        repo.path(),
        "bench/personal/baseline.json",
        br#"{"personal": {"wrong": 1, "correction_count": 1, "wrong_lines": "2"}}"#,
    );
    let report = run(repo.path(), Mode::Report).unwrap().report();
    assert!(
        report.contains("- personal 由錯變對 1 題：第 2 行"),
        "{report}"
    );
    assert!(
        report.contains("- personal 由對變錯 1 題：第 3 行"),
        "{report}"
    );
    for word in ["月", "曰"] {
        assert!(
            !report.contains(word),
            "報告不能帶個人集的內容「{word}」：{report}"
        );
    }
}

#[test]
fn personal_is_checked_against_its_own_baseline_file() {
    let repo = repo_with_personal();
    let violations = check(&repo);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0].contains("bench/personal/baseline.json"),
        "{violations:?}"
    );

    write(
        repo.path(),
        "bench/personal/baseline.json",
        PERSONAL_BASELINE,
    );
    assert_eq!(check(&repo), Vec::<String>::new());

    write(
        repo.path(),
        "bench/personal/baseline.json",
        br#"{"personal": {"wrong": 1, "correction_count": 0, "wrong_lines": "3"}}"#,
    );
    let violations = check(&repo);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0].contains("personal 修正 1 次"),
        "{violations:?}"
    );
}

#[test]
fn update_baseline_keeps_personal_out_of_the_repo_baseline() {
    let repo = repo_with_personal();
    run(repo.path(), Mode::UpdateBaseline).unwrap();
    let shared = fs::read_to_string(repo.path().join("bench/baseline.json")).unwrap();
    assert!(!shared.contains("personal"), "{shared}");
    let personal = fs::read_to_string(repo.path().join("bench/personal/baseline.json")).unwrap();
    assert_eq!(
        qim_bench::parse_baseline(&personal).unwrap(),
        qim_bench::parse_baseline(std::str::from_utf8(PERSONAL_BASELINE).unwrap()).unwrap()
    );
    assert_eq!(check(&repo), Vec::<String>::new());
}

#[test]
fn without_personal_files_bench_runs_and_says_so() {
    let repo = repo();
    write(repo.path(), "bench/baseline.json", TOY_BASELINE);
    // 別台機器留下的個人集 baseline 不影響：沒有題目就整個跳過。
    write(
        repo.path(),
        "bench/personal/baseline.json",
        PERSONAL_BASELINE,
    );
    let outcome = run(repo.path(), Mode::Check).unwrap();
    assert!(matches!(outcome.personal, Personal::Missing));
    assert_eq!(outcome.violations, Vec::<String>::new());
    assert!(
        outcome
            .report()
            .contains("個人集：沒有 bench/personal/rime-userdb.txt，跳過")
    );
}

#[test]
fn a_userdb_that_was_not_transcoded_yet_is_skipped_with_a_hint() {
    let repo = repo();
    write(repo.path(), "bench/personal/rime-userdb.txt", b"");
    let outcome = run(repo.path(), Mode::Report).unwrap();
    assert!(matches!(outcome.personal, Personal::NotTranscoded));
    assert!(outcome.report().contains("先跑 `cargo xtask data`"));
}

#[test]
fn a_corpus_cannot_take_the_personal_set_name() {
    let repo = repo();
    write(
        repo.path(),
        "bench/corpus/personal.tsv",
        "a\t日\n".as_bytes(),
    );
    let error = run(repo.path(), Mode::Report).unwrap_err();
    assert!(error.contains("personal"), "{error}");
}

#[test]
fn update_baseline_replaces_a_baseline_in_an_old_format() {
    let repo = repo();
    // 票 02 的格式只有答錯數：報告與 `--check` 讀不了，`--update-baseline` 要能直接改寫。
    write(
        repo.path(),
        "bench/baseline.json",
        br#"{"toy": {"wrong": 2}}"#,
    );
    assert!(run(repo.path(), Mode::Report).is_err());
    run(repo.path(), Mode::UpdateBaseline).unwrap();
    assert_eq!(check(&repo), Vec::<String>::new());
}

#[test]
fn a_userdb_changed_after_transcoding_is_skipped_instead_of_running_old_items() {
    let repo = repo_with_personal();
    write(
        repo.path(),
        "bench/personal/rime-userdb.txt",
        "換過的學習資料".as_bytes(),
    );
    let outcome = run(repo.path(), Mode::Report).unwrap();
    assert!(
        matches!(outcome.personal, Personal::NotTranscoded),
        "{:?}",
        outcome.personal
    );
    assert!(outcome.report().contains("先跑 `cargo xtask data`"));

    transcode_personal(&repo, "換過的學習資料");
    let outcome = run(repo.path(), Mode::Report).unwrap();
    assert!(
        matches!(outcome.personal, Personal::Ran(_)),
        "{:?}",
        outcome.personal
    );
}

#[test]
fn items_left_behind_without_a_userdb_are_not_run() {
    let repo = repo_with_personal();
    fs::remove_file(repo.path().join("bench/personal/rime-userdb.txt")).unwrap();
    let outcome = run(repo.path(), Mode::Report).unwrap();
    assert!(
        matches!(outcome.personal, Personal::Missing),
        "{:?}",
        outcome.personal
    );
}

/// `long` 一句 21 碼（`ab` 交替，一個字一個碼）：和固定題 `a`、toy 三句一起量延遲。
fn repo_with_long_input() -> TempDir {
    let repo = repo();
    let codes = ["a b"; 10].join(" ") + " a";
    write(
        repo.path(),
        "bench/corpus/long.tsv",
        format!("{codes}\t{}日\n", "日月".repeat(10)).as_bytes(),
    );
    repo
}

#[test]
fn latency_reports_native_and_wasm_percentiles_for_blank_and_learned_user_data() {
    let repo = repo_with_long_input();
    let outcome = run(repo.path(), Mode::Report).unwrap();
    let report = outcome.report();
    let rows: Vec<&str> = report
        .lines()
        .filter(|line| {
            ["| 全部 |", "| 第 20 碼起 |", "| 選開頭一段 |"]
                .iter()
                .any(|group| line.contains(group))
        })
        .collect();
    // 全部：固定題 `a`（1 鍵）、toy `a`、`a`、`ab`（4 鍵）、long（21 鍵），共 5 個輸入 26 鍵。
    // 假計時器的 26 個值排序後是 100 ms ×（1、1、1、1、1、2、2、3、4、…、21），p50 第 13 個是 8、p95 第 25 個是 20；
    // 學過的學習資料下每鍵加倍。第 20 碼起：只有 long 的第 20、21 鍵，p50 第 1 個、p95 第 2 個。
    // 選開頭一段：只有 long 打完有兩段以上，一次；學過的學習資料下 long 整句已造詞、只有一段，不量。
    // 原生是 [`fast_native`]：每鍵 50 µs、選開頭一段 500 µs，報告附 p95 上限。
    // 含萬用鍵的輸入（`z`、`z`、`z`、`zb`、`zbab…a`）鍵數相同；選開頭一段也一樣：空白時 long 有兩段以上，
    // 學過時萬用鍵照樣對上學過的整句、只有一段。上限同基準的輸入。
    assert_eq!(rows.len(), 24, "{report}");
    for (rows, inputs, user, selects, limits) in [
        (
            &rows[..6],
            "基準的輸入",
            "空白",
            "| 1 | 1 | 500.00 | 500.00",
            ["1000", "1000", "5000"],
        ),
        (
            &rows[6..12],
            "基準的輸入",
            "學過",
            "| 0 | 0 | — | —",
            ["3000", "3000", "10000"],
        ),
        (
            &rows[12..18],
            "含萬用鍵的輸入",
            "空白",
            "| 1 | 1 | 500.00 | 500.00",
            ["1000", "1000", "5000"],
        ),
        (
            &rows[18..],
            "含萬用鍵的輸入",
            "學過",
            "| 0 | 0 | — | —",
            ["3000", "3000", "10000"],
        ),
    ] {
        let expected = [
            format!(
                "| {inputs} | {user} | 原生（假） | 全部 | 5 | 26 | 50.00 | 50.00 | ≤ {} |",
                limits[0]
            ),
            format!(
                "| {inputs} | {user} | 原生（假） | 第 20 碼起 | 1 | 2 | 50.00 | 50.00 | ≤ {} |",
                limits[1]
            ),
            format!(
                "| {inputs} | {user} | 原生（假） | 選開頭一段 {selects} | ≤ {} |",
                limits[2]
            ),
        ];
        assert_eq!(rows[..3], expected, "{report}");
    }
    assert_eq!(
        rows[3],
        "| 基準的輸入 | 空白 | WebAssembly（假） | 全部 | 5 | 26 | 800000.00 | 2000000.00 | 只報 |"
    );
    assert_eq!(
        rows[4],
        "| 基準的輸入 | 空白 | WebAssembly（假） | 第 20 碼起 | 1 | 2 | 2000000.00 | 2100000.00 | 只報 |"
    );
    assert_eq!(
        rows[5],
        "| 基準的輸入 | 空白 | WebAssembly（假） | 選開頭一段 | 1 | 1 | 1000000.00 | 1000000.00 | 只報 |"
    );
    assert_eq!(
        rows[9],
        "| 基準的輸入 | 學過 | WebAssembly（假） | 全部 | 5 | 26 | 1600000.00 | 4000000.00 | 只報 |"
    );
    assert_eq!(
        rows[15],
        "| 含萬用鍵的輸入 | 空白 | WebAssembly（假） | 全部 | 5 | 26 | 800000.00 | 2000000.00 | 只報 |"
    );
}

/// 學過的學習資料：一個 `UserModel` 依序學完各語料集合與「有前文」的題，WebAssembly 拿到它的匯出；
/// 空白那次不給。
#[test]
fn the_learned_run_gives_wasm_the_user_model_that_learned_every_public_set() {
    let repo = repo_with_long_input();
    let dicts = std::cell::RefCell::new(Vec::new());
    qim_bench::run(
        repo.path(),
        Mode::Report,
        None,
        with_wasm(
            &|packs: &[PathBuf], user_dict: Option<&str>, inputs: &[String]| {
                dicts.borrow_mut().push(user_dict.map(String::from));
                slow_wasm(packs, user_dict, inputs)
            },
        ),
    )
    .unwrap();
    // 基準的輸入、含萬用鍵的輸入各量空白、學過一次。
    let dicts = dicts.into_inner();
    assert_eq!(dicts.len(), 4, "{dicts:?}");
    assert_eq!(dicts[0], None);
    assert_eq!(dicts[2], None);
    assert_eq!(dicts[1], dicts[3]);
    let dict = dicts[1].as_deref().unwrap();
    assert_ne!(
        UserModel::import(dict, None).unwrap().user,
        UserModel::default()
    );
    // toy 第 2 行選了「曰」上屏，學習計數有它。
    assert!(dict.contains('曰'), "{dict}");
}

#[test]
fn check_blocks_native_latency_only_and_update_baseline_ignores_latency() {
    let repo = repo_with_long_input();
    run(repo.path(), Mode::UpdateBaseline).unwrap();
    let written = fs::read_to_string(repo.path().join("bench/baseline.json")).unwrap();
    assert!(
        !written.contains("ms") && !written.contains("latency"),
        "{written}"
    );
    // WebAssembly 每鍵都遠超上限，原生在上限內：只報 WebAssembly，不擋。
    let outcome = run(repo.path(), Mode::Check).unwrap();
    assert!(
        outcome
            .latency
            .blank
            .wasm
            .per_key_ms
            .iter()
            .flatten()
            .all(|&ms| ms >= 100.0)
    );
    assert_eq!(outcome.latency.violations(), Vec::<String>::new());
    assert_eq!(outcome.violations, Vec::<String>::new());
}

/// 原生超過上限的那幾格由 `--check` 擋、報告註明；在上限內的不擋，只印結果時也不擋。
#[test]
fn check_blocks_the_native_latency_groups_over_their_limit() {
    let repo = repo_with_long_input();
    run(repo.path(), Mode::UpdateBaseline).unwrap();
    // 每鍵 2 ms：兩批輸入的空白「全部」「第 20 碼起」超過上限，學過的與選開頭一段（0.5 ms）都在內。
    let slow_native = fixed_native(2.0, 0.5);
    let timers = Timers {
        native: &slow_native,
        wasm: &slow_wasm,
    };
    let outcome = qim_bench::run(repo.path(), Mode::Check, None, timers).unwrap();
    let violations = &outcome.violations;
    assert_eq!(violations.len(), 4, "{violations:?}");
    let expected = [
        ("基準的輸入", "全部"),
        ("基準的輸入", "第 20 碼起"),
        ("含萬用鍵的輸入", "全部"),
        ("含萬用鍵的輸入", "第 20 碼起"),
    ];
    for (v, (inputs, group)) in violations.iter().zip(expected) {
        assert!(
            v.contains("每鍵延遲")
                && v.contains(inputs)
                && v.contains("空白")
                && v.contains(&format!("{group} 的 p95")),
            "{v}"
        );
    }
    assert_eq!(
        outcome.report().matches("，超過 |").count(),
        4,
        "{}",
        outcome.report()
    );
    let report = qim_bench::run(repo.path(), Mode::Report, None, timers).unwrap();
    assert_eq!(report.violations, Vec::<String>::new());
}

/// 含萬用鍵的輸入（M4 票 18b）由 `--check` 擋：只有它慢時紅、只註明它；它在上限內時不紅。
#[test]
fn check_blocks_the_wildcard_inputs_over_their_limit() {
    let repo = repo_with_long_input();
    run(repo.path(), Mode::UpdateBaseline).unwrap();
    // 假計時器：交給它的輸入開頭都是萬用鍵（含萬用鍵的那批）時每鍵 `wildcard_ms`，否則 0.05 ms。
    let native_with = |wildcard_ms: f64| {
        move |engine: &Engine, user: &UserModel, inputs: &[String]| {
            let wildcard = inputs.iter().all(|input| input.starts_with('z'));
            let key_ms = if wildcard { wildcard_ms } else { 0.05 };
            fake_timing(engine, user, inputs, |_| key_ms, 0.5)
        }
    };
    let slow = native_with(2.0);
    let outcome = qim_bench::run(
        repo.path(),
        Mode::Check,
        None,
        Timers {
            native: &slow,
            wasm: &slow_wasm,
        },
    )
    .unwrap();
    let violations = &outcome.violations;
    assert_eq!(violations.len(), 2, "{violations:?}");
    for (v, group) in violations.iter().zip(["全部", "第 20 碼起"]) {
        assert!(
            v.contains("含萬用鍵的輸入")
                && v.contains("空白")
                && v.contains(&format!("{group} 的 p95")),
            "{v}"
        );
    }
    let within = native_with(1.0);
    let outcome = qim_bench::run(
        repo.path(),
        Mode::Check,
        None,
        Timers {
            native: &within,
            wasm: &slow_wasm,
        },
    )
    .unwrap();
    assert_eq!(outcome.violations, Vec::<String>::new());
}

/// 方案沒有萬用鍵時量不了含萬用鍵的輸入：整個基準失敗，免得那組安靜地缺席。
#[test]
fn a_scheme_without_a_wildcard_is_an_error() {
    let repo = repo();
    let mut scheme = SchemePack::from_bytes(
        &fs::read(repo.path().join("build/packs/quick/quick.pack")).unwrap(),
    )
    .unwrap();
    scheme.scheme.wildcard = None;
    write(
        repo.path(),
        "build/packs/quick/quick.pack",
        &scheme.to_bytes().unwrap(),
    );
    let error = run(repo.path(), Mode::Report).unwrap_err();
    assert!(error.contains("萬用鍵"), "{error}");
}

/// CLI 用的真實原生計時（`native_timing_with`）：只驗量了哪些鍵、哪些輸入量選開頭一段與環境欄，
/// 不驗時間，所以不受機器負載影響。量選開頭一段的輸入和 [`slow_wasm`] 不同時 `run` 是錯誤，
/// 所以這裡也驗了原生在學過的學習資料下照同一條規則選。
#[test]
fn the_real_native_timer_times_every_key_and_the_same_selects_as_wasm() {
    let repo = repo_with_long_input();
    let timers = Timers {
        native: &qim_bench::native_timing_with,
        wasm: &slow_wasm,
    };
    let outcome = qim_bench::run(repo.path(), Mode::Report, None, timers).unwrap();
    let native = if cfg!(debug_assertions) {
        "原生（debug）"
    } else {
        "原生（release）"
    };
    let report = outcome.report();
    let runs = ["基準的輸入", "含萬用鍵的輸入"]
        .into_iter()
        .flat_map(|inputs| [(inputs, "空白", "1 | 1"), (inputs, "學過", "0 | 0")]);
    for (inputs, user, selects) in runs {
        for (group, counts) in [
            ("全部", "5 | 26"),
            ("第 20 碼起", "1 | 2"),
            ("選開頭一段", selects),
        ] {
            let row = format!("| {inputs} | {user} | {native} | {group} | {counts} | ");
            assert!(
                report.lines().any(|line| line.starts_with(&row)),
                "{row}\n{report}"
            );
        }
    }
}

#[test]
fn a_wasm_timer_that_misses_keys_is_an_error() {
    let repo = repo();
    let error = qim_bench::run(
        repo.path(),
        Mode::Report,
        None,
        with_wasm(&|_: &[PathBuf], _: Option<&str>, _: &[String]| {
            Ok(Timing {
                runtime: "假".into(),
                per_key_ms: vec![vec![1.0]],
                select_ms: vec![None],
                load_ms: None,
            })
        }),
    )
    .unwrap_err();
    assert!(error.contains("WebAssembly"), "{error}");
}

#[test]
fn a_wasm_timer_that_selects_on_other_inputs_than_native_is_an_error() {
    let repo = repo_with_long_input();
    let error = qim_bench::run(
        repo.path(),
        Mode::Report,
        None,
        with_wasm(
            &|packs: &[PathBuf], user_dict: Option<&str>, inputs: &[String]| {
                let mut timing = slow_wasm(packs, user_dict, inputs)?;
                assert_eq!(timing.select_ms.iter().flatten().count(), 1);
                timing.select_ms = vec![Some(1.0); inputs.len()];
                Ok(timing)
            },
        ),
    )
    .unwrap_err();
    assert!(error.contains("選開頭一段"), "{error}");
}

/// dev 模式（`--dev`）：題目換成 `build/corpus/<名>/dev.tsv`；toy 的 dev 只有一句 `a b 日月`，
/// 詞庫有 `ab` → 日月。β = 0 時同分，字「明」在詞前，首選錯；β > 0 時詞勝。
fn repo_with_dev() -> TempDir {
    let repo = repo();
    write(
        repo.path(),
        "build/packs/quick/quick.pack",
        &pack_bytes_with(TABLE, Some("quick-lexicon"), None),
    );
    write(
        repo.path(),
        "build/packs/quick-lexicon/quick-lexicon.pack",
        &lexicon_bytes(&[("ab", "日月")]),
    );
    write(
        repo.path(),
        "build/corpus/toy/dev.tsv",
        "a b\t日月\n".as_bytes(),
    );
    repo
}

fn dev(repo: &TempDir, overrides: &[&str]) -> Result<qim_bench::DevOutcome, String> {
    dev_with(repo, overrides, None)
}

/// `lexicon` 是 `--lexicon <id>`：換掉方案指名的詞庫。
fn dev_with(
    repo: &TempDir,
    overrides: &[&str],
    lexicon: Option<&str>,
) -> Result<qim_bench::DevOutcome, String> {
    let overrides: Vec<String> = overrides.iter().map(|s| s.to_string()).collect();
    qim_bench::run_dev(repo.path(), &overrides, lexicon)
}

/// 另一份詞庫資料包 `build/packs/other-lexicon/`，沒有 `ab` → 日月（粵典詞庫這類可選的詞庫，M3 票 24）。
fn with_other_lexicon(repo: &TempDir) {
    let pack = LexiconPack {
        id: "other-lexicon".into(),
        lexicon: Lexicon::new([], []),
    };
    write(
        repo.path(),
        "build/packs/other-lexicon/other-lexicon.pack",
        &pack.to_bytes().unwrap(),
    );
}

#[test]
fn another_lexicon_pack_replaces_the_one_the_scheme_names() {
    let repo = repo_with_dev();
    with_other_lexicon(&repo);
    // 方案指名的詞庫有日月，詞加分夠時答對；換成沒有日月的那份就答錯。
    let named = dev(&repo, &["word_bonus=1.5"]).unwrap();
    assert_eq!(named.sets[0].wrong(), 0);
    let other = dev_with(&repo, &["word_bonus=1.5"], Some("other-lexicon")).unwrap();
    assert_eq!(other.sets[0].wrong(), 1, "{}", other.report());
    let paths: Vec<&str> = other.packs.iter().map(|p| p.path.as_str()).collect();
    assert!(
        paths.contains(&"build/packs/other-lexicon/other-lexicon.pack"),
        "{paths:?}"
    );
    assert!(
        !paths.iter().any(|p| p.contains("quick-lexicon")),
        "{paths:?}"
    );

    let outcome = qim_bench::run(
        repo.path(),
        Mode::Report,
        Some("other-lexicon"),
        with_wasm(&slow_wasm),
    )
    .unwrap();
    assert!(
        outcome.report().contains("other-lexicon.pack"),
        "{}",
        outcome.report()
    );
}

#[test]
fn another_lexicon_cannot_update_the_baseline_and_must_exist() {
    let repo = repo_with_dev();
    with_other_lexicon(&repo);
    let err = qim_bench::run(
        repo.path(),
        Mode::UpdateBaseline,
        Some("other-lexicon"),
        with_wasm(&slow_wasm),
    )
    .unwrap_err();
    assert!(err.contains("--update-baseline"), "{err}");
    assert!(!repo.path().join(qim_bench::BASELINE).exists());
    let err = dev_with(&repo, &[], Some("missing")).unwrap_err();
    assert!(err.contains("build/packs/missing/missing.pack"), "{err}");
}

#[test]
fn dev_mode_scores_the_dev_split_of_each_set_instead_of_the_test_split() {
    let repo = repo_with_dev();
    let outcome = dev(&repo, &[]).unwrap();
    let [toy, _, _] = outcome.sets.as_slice() else {
        panic!("應該只有 toy 與它的單字、詞：{:?}", outcome.sets);
    };
    assert_eq!((toy.name.as_str(), toy.total(), toy.wrong()), ("toy", 1, 1));
    assert_eq!(outcome.fixed.total(), 1, "固定題照跑");
    assert!(
        outcome.report().contains("| toy | 1 |"),
        "{}",
        outcome.report()
    );
}

#[test]
fn dev_mode_overrides_ranking_parameters_without_touching_the_pack() {
    let repo = repo_with_dev();
    let outcome = dev(&repo, &["word_bonus=1.5", "segment_penalty=0.25"]).unwrap();
    assert_eq!(outcome.sets[0].wrong(), 0, "{}", outcome.report());
    assert_eq!(outcome.ranking.word_bonus, 1.5);
    assert_eq!(outcome.ranking.segment_penalty, 0.25);
    let report = outcome.report();
    assert!(report.contains("word_bonus = 1.5"), "{report}");
    // 方案檔的值沒變：不覆寫時照舊答錯。
    assert_eq!(dev(&repo, &[]).unwrap().sets[0].wrong(), 1);
}

#[test]
fn dev_mode_rejects_unknown_keys_and_bad_values() {
    let repo = repo_with_dev();
    for (bad, expected) in [
        ("bonus=1", "bonus"),
        ("word_bonus", "key=value"),
        ("word_bonus=x", "word_bonus"),
        ("beam=-1", "beam"),
    ] {
        let err = dev(&repo, &[bad]).unwrap_err();
        assert!(err.contains(expected), "{bad}：{err}");
    }
}

/// 蓋過的值照方案檔的規則驗（`Ranking::check`）：錯誤指出是哪個值、是 `--set` 給的，
/// 不說成資料包壞了。
#[test]
fn dev_mode_rejects_overrides_that_the_scheme_file_would_reject() {
    let repo = repo_with_dev();
    for (bad, field, value) in [
        ("beam=0", "beam", "0"),
        ("beam=NaN", "beam", "NaN"),
        ("word_bonus=NaN", "word_bonus", "NaN"),
        ("segment_penalty=inf", "segment_penalty", "inf"),
    ] {
        let err = dev(&repo, &[bad]).unwrap_err();
        assert!(
            err.contains("--set") && err.contains(field) && err.contains(value),
            "{bad}：{err}"
        );
    }
}

#[test]
fn dev_mode_needs_the_dev_file_of_every_set() {
    let repo = repo_with_dev();
    fs::remove_file(repo.path().join("build/corpus/toy/dev.tsv")).unwrap();
    let err = dev(&repo, &[]).unwrap_err();
    assert!(
        err.contains("build/corpus/toy/dev.tsv") && err.contains("cargo xtask data"),
        "{err}"
    );
}

#[test]
fn dev_mode_adds_the_short_inputs_taken_from_the_sentences_of_each_set() {
    let repo = repo_with_dev();
    write(
        repo.path(),
        "build/corpus/toy/dev.tsv",
        "a b\t曰月\na\t曰\na b a\t日月日\n".as_bytes(),
    );
    let outcome = dev(&repo, &[]).unwrap();
    let [_, chars, words] = outcome.sets.as_slice() else {
        panic!("toy 之後應有它的單字、詞：{:?}", outcome.sets);
    };
    // 單字：`a 曰` 兩次（首選是日，錯）、`b 月` 兩次、`a 日` 兩次（對）：不重複的 3 題，依次數加權。
    assert_eq!(chars.name, "toy·單字");
    assert_eq!((chars.total(), chars.wrong()), (3, 1));
    assert!((chars.metrics(true).sentence_accuracy - 4.0 / 6.0).abs() < 1e-9);
    // 詞：連續兩字以上、詞庫裡有的只有 `ab 日月`（曰月、月日、日月日都不是詞）；β = 0 時
    // 字「明」排在詞前，答錯。
    assert_eq!(words.name, "toy·詞");
    assert_eq!(
        (words.total(), words.wrong()),
        (1, 1),
        "{}",
        outcome.report()
    );
    assert_eq!(words.items[0].weight, 1);
    let report = outcome.report();
    assert!(report.contains("| toy·單字（加權） | 6 次 |"), "{report}");
    assert!(report.contains("| toy·詞（加權） | 1 次 |"), "{report}");
}

/// dev 模式也打 dev 切分的「有前文」題（`build/corpus/<名>/dev-context.tsv`），給前文與不給前文
/// 並列，接在所有集合之後；沒有這個檔的集合不打。語言模型只有「日 → 曰」。
#[test]
fn dev_mode_types_the_dev_context_items_with_and_without_the_context() {
    let repo = repo_with_dev();
    write(
        repo.path(),
        "build/packs/quick/quick.pack",
        &pack_bytes_with(TABLE, Some("quick-lexicon"), Some("bigram")),
    );
    write(
        repo.path(),
        "build/packs/bigram/bigram.pack",
        &language_model_bytes('曰'),
    );
    write(
        repo.path(),
        "build/corpus/toy/dev-context.tsv",
        "今日\ta\t曰\n".as_bytes(),
    );
    let outcome = dev(&repo, &[]).unwrap();
    let names: Vec<&str> = outcome.sets.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        ["toy", "toy·單字", "toy·詞", "toy·前文", "toy·前文·不給"]
    );
    assert_eq!((outcome.sets[3].wrong(), outcome.sets[4].wrong()), (0, 1));
    let report = outcome.report();
    assert!(report.contains("| toy | 1 | 0 | 1 | 0 | 1 |"), "{report}");
}

/// `a 曰` 打三次：學習關每次都要按 2 選「曰」（修正 3）；學習開時選過兩次後成首選（規則 2），
/// 第三次按空白就好（修正 2）。
#[test]
fn learning_mode_compares_corrections_with_and_without_learning() {
    let repo = repo();
    write(
        repo.path(),
        "bench/colloquial/again.tsv",
        "a\t曰\na\t曰\na\t曰\n".as_bytes(),
    );
    let learning = qim_bench::run_learning(repo.path()).unwrap();
    let sets: Vec<(&str, usize, usize, usize)> = learning
        .sets
        .iter()
        .map(|s| {
            (
                s.name.as_str(),
                s.total,
                s.corrections_off,
                s.corrections_on,
            )
        })
        .collect();
    // toy：`a 曰` 只有一次，學不到；`a b 日月` 卡住後逐字打，上屏的「日」本來就是首選。
    assert_eq!(sets, [("toy", 3, 2, 2), ("again", 3, 3, 2)]);

    let report = learning.report();
    assert!(
        report.contains("| again | 3 | 3 | 2 | 1.00 | 0.67 |"),
        "{report}"
    );
}

#[test]
fn learning_mode_reports_personal_as_numbers_only() {
    let repo = repo_with_personal();
    let learning = qim_bench::run_learning(repo.path()).unwrap();
    let personal = learning.sets.iter().find(|s| s.name == "personal").unwrap();
    assert!(personal.private);
    assert_eq!((personal.total, personal.corrections_off), (2, 1));
    let report = learning.report();
    for word in ["月", "曰"] {
        assert!(
            !report.contains(word),
            "報告不能帶個人集的內容「{word}」：{report}"
        );
    }
}

/// 「含打錯碼」的 toy（M4 票 08）：
/// - 第 1 行 `a`（沒打錯）：同一般的題，答對；原樣讀法也對，改錯的分母。
/// - 第 2 行打成 `b`：首選「月」答錯；模擬時「月」「🌙」都接不上又沒有下一頁，Esc（修正 1）照碼重打 `a`、空白，
///   共 4 鍵。
/// - 第 3 行 `ab` 打成 `ba`（兩碼對調、跨字）：`ba` 的候選都接不上，Esc（修正 1）照碼重打 `ab`，首選「明」
///   接不上、選第 2 個「日月」（修正 1）。
/// - 第 4 行碼全漏了：打不出組字，照碼重打 `a`（修正 1）、空白。
const TOY_TYPO: &str = "a\ta\t日\nb\ta\t日\nb a\ta b\t日月\n\ta\t日\n";

#[test]
fn typo_sets_type_the_typed_keys_first_and_retype_the_code_when_stuck() {
    let repo = repo();
    write(
        repo.path(),
        "bench/colloquial/typo/toy.tsv",
        TOY_TYPO.as_bytes(),
    );
    let outcome = run(repo.path(), Mode::Report).unwrap();
    let set = outcome
        .sets
        .iter()
        .find(|s| s.name == "toy·打錯")
        .expect("集合名是語料名接 ·打錯");
    let per_item: Vec<(bool, usize, usize, bool)> = set
        .items
        .iter()
        .map(|i| (i.correct, i.correction_count, i.keys, i.finished))
        .collect();
    assert_eq!(
        per_item,
        [
            (true, 0, 2, true),
            (false, 1, 4, true),
            (false, 2, 6, true),
            (false, 1, 2, true)
        ]
    );
    let stats = set.typo_stats().expect("打錯碼的集合有統計");
    assert_eq!(
        (
            stats.clean,
            stats.clean_correct,
            stats.mistyped,
            stats.mistyped_correct
        ),
        (1, 1, 3, 0)
    );
    // 核心沒有容錯（M4 票 09），原樣讀法就是實際的讀法：沒打錯而原樣正確的 1 題，改錯 0。
    assert_eq!((stats.literal_correct, stats.false_corrections), (1, 0));
    assert_eq!(stats.false_correction_rate(), 0.0);
    // 一般的集合沒有打錯碼的統計。
    assert!(outcome.sets[0].typo_stats().is_none());
    let report = outcome.report();
    assert!(report.contains("原樣正確卻被改錯"), "{report}");
    assert!(
        report.contains("| toy·打錯 | 4 | 1 | 1 | 3 | 0 | 0/1 |"),
        "{report}"
    );
}

#[test]
fn typo_sets_need_a_baseline_under_check() {
    let repo = repo();
    write(
        repo.path(),
        "bench/colloquial/typo/toy.tsv",
        TOY_TYPO.as_bytes(),
    );
    write(repo.path(), "bench/baseline.json", TOY_BASELINE);
    let violations = check(&repo);
    assert!(
        violations.iter().any(|v| v.contains("toy·打錯")),
        "{violations:?}"
    );
}

/// 兩個並排的字母鍵 a、b（各 50 × 50），觸控模擬經空間模型把點擊轉成按鍵機率。
const TWO_KEY_LAYOUT: &str = r#"{
  "format": "qim-layout/1",
  "variants": [{
    "when": { "device": "phone", "orientation": "portrait" },
    "canvas": { "width": 100, "height": 50 },
    "pages": [{ "id": "letters", "keys": [
      { "id": "a", "frame": { "x": 0, "y": 0, "w": 50, "h": 50 }, "tap": { "letter": "a" } },
      { "id": "b", "frame": { "x": 50, "y": 0, "w": 50, "h": 50 }, "tap": { "letter": "b" } }
    ]}]
  }]
}"#;

#[test]
fn touch_sets_turn_taps_into_letters_through_the_layout() {
    let repo = repo();
    write(
        repo.path(),
        qim_bench::TOUCH_LAYOUT,
        TWO_KEY_LAYOUT.as_bytes(),
    );
    // 第 1 行點中 a；第 2 行要打 a 卻點到 b（打錯）。
    write(
        repo.path(),
        "bench/colloquial/touch/toy.tsv",
        "10.0,25.0\ta\t日\n60.0,25.0\ta\t日\n".as_bytes(),
    );
    let outcome = run(repo.path(), Mode::Report).unwrap();
    let set = outcome.sets.iter().find(|s| s.name == "toy·觸控").unwrap();
    let stats = set.typo_stats().unwrap();
    assert_eq!(
        (stats.clean, stats.clean_correct, stats.mistyped),
        (1, 1, 1)
    );
    assert_eq!(set.wrong_lines().into_iter().collect::<Vec<_>>(), [2]);
}

#[test]
fn touch_sets_without_the_layout_point_at_cargo_xtask_data() {
    let repo = repo();
    write(
        repo.path(),
        "bench/colloquial/touch/toy.tsv",
        "10.0,25.0\ta\t日\n".as_bytes(),
    );
    let error = run(repo.path(), Mode::Report).unwrap_err();
    assert!(error.contains(qim_bench::TOUCH_LAYOUT), "{error}");
    assert!(error.contains("cargo xtask data"), "{error}");
}

/// dev 模式有打錯碼的調參題就打（M4 票 08），集合名同 test、接在最後，報告附打錯碼的表；沒有就不打。
#[test]
fn dev_mode_scores_typo_and_touch_items_when_present() {
    let repo = repo_with_dev();
    let without = dev(&repo, &[]).unwrap();
    assert!(without.sets.iter().all(|s| s.typo_stats().is_none()));
    write(
        repo.path(),
        "build/corpus/toy/dev-typo.tsv",
        "b a\ta b\t日月\n".as_bytes(),
    );
    write(
        repo.path(),
        qim_bench::TOUCH_LAYOUT,
        TWO_KEY_LAYOUT.as_bytes(),
    );
    write(
        repo.path(),
        "build/corpus/toy/dev-touch.tsv",
        "10.0,25.0 60.0,25.0\ta b\t日月\n".as_bytes(),
    );
    let outcome = dev(&repo, &[]).unwrap();
    let names: Vec<&str> = outcome.sets.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names[names.len() - 2..], ["toy·打錯", "toy·觸控"]);
    let typo = outcome.sets[names.len() - 2].typo_stats().unwrap();
    assert_eq!((typo.clean, typo.mistyped), (0, 1));
    // 兩個點擊都點中：沒打錯，和 dev 題一樣在方案檔的參數下答錯（`明` 排在 `日月` 前）。
    let touch = outcome.sets[names.len() - 1].typo_stats().unwrap();
    assert_eq!((touch.clean, touch.clean_correct), (1, 0));
    assert!(outcome.report().contains("原樣正確卻被改錯"));
}

/// 沒打錯的打錯碼題（實際按的鍵等於碼）照一般的題模擬：修正次數、按鍵數、打不打得出都一樣。
/// `a b 日目`：「目」不在碼表，選「日」（修正 1）後 `b` 卡住、只剩一個字，打不出；不能因為它在打錯碼的集合
/// 就多按一次 Esc 照碼重打。
#[test]
fn clean_typo_items_simulate_like_ordinary_items() {
    let repo = repo();
    let corpus = "a\t日\na\t曰\na b\t日月\nc b\t十月\na b\t日目\n";
    write(repo.path(), "bench/corpus/toy.tsv", corpus.as_bytes());
    let typo: String = corpus
        .lines()
        .map(|line| {
            let (codes, _) = line.split_once('\t').unwrap();
            format!("{codes}\t{line}\n")
        })
        .collect();
    write(
        repo.path(),
        "bench/colloquial/typo/toy.tsv",
        typo.as_bytes(),
    );
    let outcome = run(repo.path(), Mode::Report).unwrap();
    let summary = |name: &str| -> Vec<(bool, usize, usize, bool)> {
        let set = outcome.sets.iter().find(|s| s.name == name).unwrap();
        set.items
            .iter()
            .map(|i| (i.correct, i.correction_count, i.keys, i.finished))
            .collect()
    };
    assert_eq!(summary("toy·打錯"), summary("toy"));
    let stats = outcome
        .sets
        .iter()
        .find(|s| s.name == "toy·打錯")
        .unwrap()
        .typo_stats()
        .unwrap();
    assert_eq!(stats.mistyped, 0);
}

/// 打錯的第一輪不翻頁：要打 `a`（日）卻打成 `c`，`c` 的候選有兩頁，第一頁接不上就 Esc 照碼重打（修正 1），
/// 不先翻到第二頁（使用者看得到組字打錯了）。
#[test]
fn a_mistyped_composition_is_cleared_without_turning_pages() {
    let repo = repo();
    write(
        repo.path(),
        "bench/colloquial/typo/toy.tsv",
        "c\ta\t日\n".as_bytes(),
    );
    let outcome = run(repo.path(), Mode::Report).unwrap();
    let set = outcome.sets.iter().find(|s| s.name == "toy·打錯").unwrap();
    let item = &set.items[0];
    // c、Esc、a、空白。
    assert_eq!(
        (item.correction_count, item.keys, item.finished),
        (1, 4, true)
    );
}
