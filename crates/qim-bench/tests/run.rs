//! 經 [`qim_bench::run`] 跑整條基準：暫存目錄擺出 repo 的佈局（資料包、`bench/`），
//! 數字都是照下面的小資料包手算的。

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use qim_bench::{Mode, run};
use qim_core::{CodeEntry, CodeTable, Pack, Scheme};
use tempfile::TempDir;

fn entry(text: char) -> CodeEntry {
    CodeEntry {
        text,
        extension: false,
    }
}

/// `a` → 日曰、`b` → 月、`ab` → 明；`table` 可以換掉整張碼表造退步。
fn pack_bytes(table: &[(&str, &str)]) -> Vec<u8> {
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        radicals: BTreeMap::new(),
    };
    let code_table = CodeTable::from_entries(
        table
            .iter()
            .map(|(code, chars)| (code.to_string(), chars.chars().map(entry).collect())),
    );
    Pack { scheme, code_table }.to_bytes().unwrap()
}

const TABLE: &[(&str, &str)] = &[("a", "日曰"), ("b", "月"), ("ab", "明")];

/// 固定題 `a 日`；語料 `toy` 三句：`a 日` 對，`b 目` 錯一字，`a b 日月` 得到「明」錯兩字。
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
        "a\t日\nb\t目\na b\t日月\n".as_bytes(),
    );
    dir
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

    assert_eq!(outcome.fixed.total, 1);
    assert_eq!(outcome.fixed.wrong(), 0);

    let [toy] = outcome.sets.as_slice() else {
        panic!("應該只有一個語料集合：{:?}", outcome.sets);
    };
    assert_eq!(toy.name, "toy");
    assert_eq!(toy.total, 3);
    assert_eq!(toy.correct, 1);
    assert_eq!(toy.wrong(), 2);
    assert_eq!(toy.chars, 4);
    assert_eq!(toy.char_errors, 3);
    assert!(outcome.violations.is_empty());
}

fn check(repo: &TempDir) -> Vec<String> {
    run(repo.path(), Mode::Check).unwrap().violations
}

#[test]
fn check_passes_when_wrong_count_equals_baseline() {
    let repo = repo();
    write(
        repo.path(),
        "bench/baseline.json",
        br#"{"toy": {"wrong": 2}}"#,
    );
    assert_eq!(check(&repo), Vec::<String>::new());
}

#[test]
fn check_fails_when_wrong_count_exceeds_baseline() {
    let repo = repo();
    write(
        repo.path(),
        "bench/baseline.json",
        br#"{"toy": {"wrong": 1}}"#,
    );
    let violations = check(&repo);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(violations[0].contains("toy"), "{violations:?}");
}

#[test]
fn check_fails_when_a_pack_breaks_a_fixed_item() {
    let repo = repo();
    write(
        repo.path(),
        "bench/baseline.json",
        br#"{"toy": {"wrong": 3}}"#,
    );
    // 換一個 `a` 的字序相反的資料包：固定題 `a 日` 的首選變「曰」，語料答錯數仍在 baseline 內。
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
        br#"{"toy": {"wrong": 2}, "gone": {"wrong": 0}}"#,
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
        b"{\n\n      \"toy\"   :{\"wrong\":\n2 }\n}",
    );
    write(
        repo.path(),
        "bench/fixed.tsv",
        "# 換了說明\n\n# 輸入碼\t期望首選\t來源\na\t日\t另一個來源\n\n".as_bytes(),
    );
    assert_eq!(check(&repo), Vec::<String>::new());
}

#[test]
fn update_baseline_rewrites_the_file_with_current_wrong_counts() {
    let repo = repo();
    write(
        repo.path(),
        "bench/baseline.json",
        br#"{"toy": {"wrong": 0}, "gone": {"wrong": 0}}"#,
    );
    let outcome = run(repo.path(), Mode::UpdateBaseline).unwrap();
    assert!(outcome.violations.is_empty());
    let written = fs::read_to_string(repo.path().join("bench/baseline.json")).unwrap();
    assert_eq!(written, "{\n  \"toy\": {\n    \"wrong\": 2\n  }\n}\n");
    assert_eq!(check(&repo), Vec::<String>::new());
}

#[test]
fn report_lists_each_set_with_its_baseline() {
    let repo = repo();
    write(
        repo.path(),
        "bench/baseline.json",
        br#"{"toy": {"wrong": 5}}"#,
    );
    let report = run(repo.path(), Mode::Report).unwrap().report();
    assert!(report.contains("固定題 1/1 通過"), "{report}");
    let toy = report
        .lines()
        .find(|line| line.starts_with("| toy |"))
        .unwrap_or_else(|| panic!("沒有 toy 那一列：{report}"));
    for cell in ["3", "1（33.33%）", "25.00%", "2", "5"] {
        assert!(toy.contains(&format!("| {cell} |")), "{toy}");
    }
}

#[test]
fn report_runs_without_a_baseline_file() {
    let repo = repo();
    let report = run(repo.path(), Mode::Report).unwrap().report();
    assert!(report.contains("| toy |"), "{report}");
}
