//! 每鍵延遲的百分位數（plan §5.2）：nearest-rank，取排序後第 ⌈p/100 × n⌉ 個。

use qim_bench::percentile;

#[test]
fn percentile_takes_the_nearest_rank() {
    let values: Vec<f64> = (1..=20).rev().map(f64::from).collect();
    assert_eq!(percentile(&values, 50), Some(10.0));
    assert_eq!(percentile(&values, 95), Some(19.0));
    assert_eq!(percentile(&values, 100), Some(20.0));
}

#[test]
fn percentile_of_one_value_is_that_value_and_of_none_is_none() {
    assert_eq!(percentile(&[0.25], 50), Some(0.25));
    assert_eq!(percentile(&[0.25], 95), Some(0.25));
    assert_eq!(percentile(&[], 50), None);
}

/// 綁定的計時腳本的輸出：runtime 一行，之後每個輸入一行；` | ` 後是選開頭一段的毫秒數。
#[test]
fn timing_output_has_keys_per_input_and_an_optional_select() {
    let timing = qim_bench::parse_timing("Node v26\n0.1 0.2\n0.3 0.4 0.5 | 0.9\n").unwrap();
    assert_eq!(timing.runtime, "Node v26");
    assert_eq!(timing.per_key_ms, vec![vec![0.1, 0.2], vec![0.3, 0.4, 0.5]]);
    assert_eq!(timing.select_ms, vec![None, Some(0.9)]);
    assert_eq!(timing.load_ms, None);
}

/// 第 1 行 ` | ` 後是載入的毫秒數（Node 的腳本有量，Swift 的沒有）。
#[test]
fn timing_output_may_carry_the_load_time_on_the_first_line() {
    let timing = qim_bench::parse_timing(
        "Node v26 | 87.5
0.1
",
    )
    .unwrap();
    assert_eq!(timing.runtime, "Node v26");
    assert_eq!(timing.load_ms, Some(87.5));
    let error = qim_bench::parse_timing(
        "Node v26 | x
0.1
",
    )
    .unwrap_err();
    assert!(error.contains("第 1 行"), "{error}");
}

#[test]
fn timing_output_that_is_not_milliseconds_is_an_error() {
    let error = qim_bench::parse_timing("Node v26\n0.1 x\n").unwrap_err();
    assert!(error.contains("第 2 行"), "{error}");
    assert!(qim_bench::parse_timing("").is_err());
}
