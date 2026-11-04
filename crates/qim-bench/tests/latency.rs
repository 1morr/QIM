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
