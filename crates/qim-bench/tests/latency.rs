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

/// 每鍵延遲閘門：原生的 p95 照 `.scratch/m4/spec.md`〈每鍵延遲上限〉擋；WebAssembly 只報。
mod gate {
    use qim_bench::{Latency, Runs, Timing};

    /// 100 個 1 鍵的輸入花 `short` ms，加一個 21 鍵的輸入：前 19 鍵 `short`、第 20、21 鍵 `long`，
    /// 選開頭一段 `select` ms。「全部」的 p95 是第 115 個值，只看 `short`；「第 20 碼起」只看 `long`。
    fn timing(short: f64, long: f64, select: f64) -> Timing {
        let mut per_key_ms = vec![vec![short]; 100];
        per_key_ms.push([vec![short; 19], vec![long; 2]].concat());
        let mut select_ms = vec![None; 100];
        select_ms.push(Some(select));
        Timing {
            runtime: "假".into(),
            per_key_ms,
            select_ms,
            load_ms: None,
        }
    }

    /// WebAssembly 每鍵都遠超上限：它只報，不影響判斷。
    fn latency(blank: Timing, learned: Timing) -> Latency {
        let slow = || timing(100.0, 100.0, 100.0);
        Latency {
            machine: "假".into(),
            blank: Runs {
                native: blank,
                wasm: slow(),
            },
            learned: Runs {
                native: learned,
                wasm: slow(),
            },
        }
    }

    #[test]
    fn native_p95_within_every_limit_passes_even_when_wasm_is_slow() {
        let outcome = latency(timing(0.99, 1.0, 5.0), timing(2.99, 3.0, 10.0));
        assert_eq!(outcome.violations(), Vec::<String>::new());
    }

    #[test]
    fn native_p95_over_any_one_limit_is_a_violation_naming_it() {
        let blank_ok = || timing(1.0, 1.0, 5.0);
        let learned_ok = || timing(3.0, 3.0, 10.0);
        let cases = [
            (
                latency(timing(1.01, 1.0, 5.0), learned_ok()),
                "空白",
                "全部",
            ),
            (
                latency(timing(1.0, 1.01, 5.0), learned_ok()),
                "空白",
                "第 20 碼起",
            ),
            (
                latency(timing(1.0, 1.0, 5.01), learned_ok()),
                "空白",
                "選開頭一段",
            ),
            (latency(blank_ok(), timing(3.01, 3.0, 10.0)), "學過", "全部"),
            (
                latency(blank_ok(), timing(3.0, 3.01, 10.0)),
                "學過",
                "第 20 碼起",
            ),
            (
                latency(blank_ok(), timing(3.0, 3.0, 10.01)),
                "學過",
                "選開頭一段",
            ),
        ];
        for (outcome, user, group) in cases {
            let violations = outcome.violations();
            assert_eq!(violations.len(), 1, "{user} {group}：{violations:?}");
            let v = &violations[0];
            assert!(
                v.contains("每鍵延遲") && v.contains(user) && v.contains(group),
                "{v}"
            );
        }
    }
}
