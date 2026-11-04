//! repo 裡手改的 `bench/fixed.tsv` 與 `bench/baseline.json` 格式要對：
//! `cargo test` 就抓到打錯的 tab 或欄名，不必等跑 `cargo xtask bench`（它要先建資料包）。

use std::fs;
use std::path::PathBuf;

use qim_bench::{BASELINE, Baseline, FIXED, parse_fixed};

fn repo_file(path: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("讀不到 {}：{e}", path.display()))
}

#[test]
fn fixed_items_parse() {
    let items = parse_fixed(&repo_file(FIXED)).unwrap();
    assert!(
        items
            .iter()
            .any(|item| item.input == "a" && item.expected == "日")
    );
}

#[test]
fn baseline_parses() {
    let baseline: Baseline = qim_bench::parse_baseline(&repo_file(BASELINE)).unwrap();
    assert!(!baseline.is_empty());
}
