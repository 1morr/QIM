//! 經 [`qim_bench::run`] 跑整條基準：暫存目錄擺出 repo 的佈局（資料包、`bench/`），
//! 數字都是照下面的小資料包手算的。
//!
//! `toy` 三句的模擬（plan §5.2，`docs/development.md`〈首選字基準〉）：
//! - 第 1 行 `a 日`：`a` 空白，2 鍵、修正 0、跳動 0，答對。
//! - 第 2 行 `a 曰`：`a` 後首選「日」接不上，按 2 選「曰」，2 鍵、修正 1，答錯（首選「日」，錯 1 字）。
//! - 第 3 行 `a b 日月`：`ab` 的首選「明」接不上又沒有下一頁，Esc（修正 1）後一個字一個字打：
//!   `a` 空白、`b` 空白，共 7 鍵；逐鍵打時 `a` 的首選「日」被 `b` 改成「明」，跳動 1；答錯（錯 2 字）。
//!
//! 合計修正 2、按鍵 11、字數 4（KSPC 2.75）、跳動 1，答錯第 2、3 行。
//!
//! WebAssembly 的每鍵延遲一律用 [`slow_wasm`]：每鍵都遠超預算，`--check` 與 baseline 照樣不受影響。

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use qim_bench::{Mode, Outcome, PERSONAL_SOURCE_HEADER, Personal, Timing};
use qim_core::{
    CharModel, CodeEntry, CodeTable, LanguageModel, LanguageModelPack, Lexicon, LexiconPack, Pack,
    Packs, Ranking, Scheme, Word,
};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

fn entry(text: char) -> CodeEntry {
    CodeEntry {
        text,
        extension: false,
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
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        radicals: BTreeMap::new(),
        packs: Packs {
            lexicon: lexicon.map(Into::into),
            language_model: language_model.map(Into::into),
        },
        ranking: Ranking::default(),
    };
    let code_table = CodeTable::from_entries(
        table
            .iter()
            .map(|(code, chars)| (code.to_string(), chars.chars().map(entry).collect())),
    );
    Pack { scheme, code_table }.to_bytes().unwrap()
}

const TABLE: &[(&str, &str)] = &[
    ("a", "日曰"),
    ("b", "月"),
    ("ab", "明"),
    ("c", "一二三四五六七八九十"),
];

/// 固定題 `a 日`；語料 `toy` 三句，見檔頭。
fn repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    write(
        dir.path(),
        "build/packs/quick/quick.pack",
        &pack_bytes(TABLE),
    );
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
    qim_bench::run(root, mode, &slow_wasm)
}

/// 假的 WebAssembly 計時器：每個輸入的第 i 鍵（從 1 起）花 100 × i 毫秒，遠超每鍵 50 ms 的預算。
fn slow_wasm(_packs: &[PathBuf], inputs: &[String]) -> Result<Timing, String> {
    Ok(Timing {
        runtime: "假".into(),
        per_key_ms: inputs
            .iter()
            .map(|input| {
                (1..=input.chars().count())
                    .map(|i| 100.0 * i as f64)
                    .collect()
            })
            .collect(),
    })
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
fn latency_reports_native_and_wasm_percentiles_with_a_long_input_group() {
    let repo = repo_with_long_input();
    let outcome = run(repo.path(), Mode::Report).unwrap();
    let report = outcome.report();
    let rows: Vec<&str> = report
        .lines()
        .filter(|line| line.contains("| 第 20 碼起 |") || line.contains("| 全部 |"))
        .collect();
    // 全部：固定題 `a`（1 鍵）、toy `a`、`a`、`ab`（4 鍵）、long（21 鍵），共 5 個輸入 26 鍵。
    // 假計時器的 26 個值排序後是 100 ms ×（1、1、1、1、1、2、2、3、4、…、21），p50 第 13 個是 8、p95 第 25 個是 20。
    // 第 20 碼起：只有 long 的第 20、21 鍵，p50 第 1 個、p95 第 2 個。
    assert_eq!(rows.len(), 4, "{report}");
    let native = if cfg!(debug_assertions) {
        "原生（debug）"
    } else {
        "原生（release）"
    };
    assert!(
        rows[0].starts_with(&format!("| {native} | 全部 | 5 | 26 | ")),
        "{report}"
    );
    assert!(
        rows[1].starts_with(&format!("| {native} | 第 20 碼起 | 1 | 2 | ")),
        "{report}"
    );
    assert_eq!(
        rows[2],
        "| WebAssembly（假） | 全部 | 5 | 26 | 800000.00 | 2000000.00 |"
    );
    assert_eq!(
        rows[3],
        "| WebAssembly（假） | 第 20 碼起 | 1 | 2 | 2000000.00 | 2100000.00 |"
    );
    assert!(report.contains("只報不擋"), "{report}");
}

#[test]
fn check_and_update_baseline_ignore_latency() {
    let repo = repo_with_long_input();
    run(repo.path(), Mode::UpdateBaseline).unwrap();
    let written = fs::read_to_string(repo.path().join("bench/baseline.json")).unwrap();
    assert!(
        !written.contains("ms") && !written.contains("latency"),
        "{written}"
    );
    let outcome = run(repo.path(), Mode::Check).unwrap();
    assert!(
        outcome
            .latency
            .wasm
            .per_key_ms
            .iter()
            .flatten()
            .all(|&ms| ms >= 100.0)
    );
    assert_eq!(outcome.violations, Vec::<String>::new());
}

#[test]
fn a_wasm_timer_that_misses_keys_is_an_error() {
    let repo = repo();
    let error = qim_bench::run(repo.path(), Mode::Report, &|_: &[PathBuf], _: &[String]| {
        Ok(Timing {
            runtime: "假".into(),
            per_key_ms: vec![vec![1.0]],
        })
    })
    .unwrap_err();
    assert!(error.contains("WebAssembly"), "{error}");
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
    let overrides: Vec<String> = overrides.iter().map(|s| s.to_string()).collect();
    qim_bench::run_dev(repo.path(), &overrides)
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
