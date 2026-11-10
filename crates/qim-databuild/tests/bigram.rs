//! 字級 bigram 的訓練（plan §4.3）：小語料上手算 interpolated Kneser-Ney 與剪枝後的值。
//!
//! 語料「甲乙、甲乙、甲丙、乙丙」，連打字集 4 字（甲乙丙丁）：
//! - 字次數 甲 3、乙 3、丙 2，共 8；bigram 甲乙 2、甲丙 1、乙丙 1，n1 = 2、n2 = 1，D = 0.5。
//! - c(甲) = 3、N1+(甲•) = 2，λ(甲) = 1/3；c(乙) = 1、N1+(乙•) = 1，λ(乙) = 1/2。
//! - N1+(••) = 3；P_cont：甲 1/7、乙 2/7、丙 3/7、丁 1/7。

use qim_core::{LOG_PROB_STEP, LanguageModel, Pack};
use qim_databuild::bigram::{Counter, Summary, train};
use qim_databuild::common_forms::BothForms;

fn model(min_count: u64) -> (LanguageModel, Summary) {
    let mut counter = Counter::default();
    counter.add("甲乙\n甲乙\n甲丙\n".as_bytes(), 1).unwrap();
    counter.add("乙丙\n".as_bytes(), 1).unwrap();
    train(&counter, 4, min_count, &BothForms::default())
}

fn assert_close(actual: f64, expected_p: f64, tolerance: f64) {
    let expected = expected_p.ln();
    assert!(
        (actual - expected).abs() <= tolerance,
        "ln P {actual}（P {}），期望 P {}",
        actual.exp(),
        expected_p
    );
}

/// f32 存的 unigram 與退回權重的誤差。
const EXACT: f64 = 1e-6;
const QUANTIZED: f64 = LOG_PROB_STEP / 2.0;

#[test]
fn interpolated_kneser_ney_without_pruning() {
    let (model, summary) = model(1);
    assert_eq!(
        summary,
        Summary {
            characters: 3,
            bigrams: 3,
            min_count: 1,
            kept: 3,
            copied: 0,
            discount: 0.5,
        }
    );
    // 留下的：(2 − 0.5)/3 + 1/3 × 2/7；0.5/3 + 1/3 × 3/7；0.5/1 + 1/2 × 3/7。
    assert_close(
        model.log_prob(Some('甲'), '乙'),
        0.5 + 2.0 / 21.0,
        QUANTIZED,
    );
    assert_close(
        model.log_prob(Some('甲'), '丙'),
        1.0 / 6.0 + 1.0 / 7.0,
        QUANTIZED,
    );
    assert_close(
        model.log_prob(Some('乙'), '丙'),
        0.5 + 3.0 / 14.0,
        QUANTIZED,
    );
    // 沒出現過的：λ(甲) × P_cont(丁)。
    assert_close(model.log_prob(Some('甲'), '丁'), 1.0 / 21.0, EXACT);
    // 只在句尾出現的丙當前一字：沒有 bigram，α = 1，就是 P_cont。
    assert_close(model.log_prob(Some('丙'), '甲'), 1.0 / 7.0, EXACT);
    // 組字開頭：(c + 1)/(N + V)。
    assert_close(model.log_prob(None, '甲'), 4.0 / 12.0, EXACT);
    assert_close(model.log_prob(None, '丁'), 1.0 / 12.0, EXACT);
}

#[test]
fn pruning_drops_rare_pairs_and_renormalizes_the_backoff_weight() {
    let (model, summary) = model(2);
    assert_eq!(summary.kept, 1);
    // 甲乙留下，值不變。
    assert_close(
        model.log_prob(Some('甲'), '乙'),
        0.5 + 2.0 / 21.0,
        QUANTIZED,
    );
    // α(甲) = (1 − P(乙|甲)) / (1 − P_cont(乙)) = (1 − 25/42) / (5/7) = 17/30。
    let alpha = 17.0 / 30.0;
    assert_close(model.log_prob(Some('甲'), '丙'), alpha * 3.0 / 7.0, EXACT);
    // 乙的 bigram 全剪掉：α = 1。
    assert_close(model.log_prob(Some('乙'), '丙'), 3.0 / 7.0, EXACT);
}

#[test]
fn every_context_sums_to_one_after_pruning() {
    let (model, _) = model(2);
    for prev in ['甲', '乙', '丙', '丁'] {
        let total: f64 = ['甲', '乙', '丙', '丁']
            .iter()
            .map(|&c| model.log_prob(Some(prev), c).exp())
            .sum();
        // 留下的甲乙是量化過的值，加總只差它的誤差。
        assert!((total - 1.0).abs() < 0.05, "{prev}：{total}");
    }
}

#[test]
fn corpus_weight_multiplies_the_counts() {
    let mut once = Counter::default();
    once.add("甲乙\n".as_bytes(), 1).unwrap();
    once.add("甲乙\n".as_bytes(), 1).unwrap();
    let mut weighted = Counter::default();
    weighted.add("甲乙\n".as_bytes(), 2).unwrap();
    let none = BothForms::default();
    assert_eq!(train(&once, 4, 1, &none).0, train(&weighted, 4, 1, &none).0);
}

#[test]
fn training_is_deterministic() {
    let bytes = || {
        let mut counter = Counter::default();
        counter
            .add("一二三四五六七八九十\n十九八七六五四三二一\n".as_bytes(), 1)
            .unwrap();
        let (model, _) = train(&counter, 10, 1, &BothForms::default());
        qim_core::LanguageModelPack {
            id: "bigram".into(),
            model,
        }
        .to_bytes()
        .unwrap()
    };
    assert_eq!(bytes(), bytes());
}

/// 並存寫法「丙／丁」，代表寫法是丙。
fn with_both_forms(corpus: &str) -> (LanguageModel, Summary) {
    let mut counter = Counter::default();
    counter.add(corpus.as_bytes(), 1).unwrap();
    train(
        &counter,
        4,
        1,
        &BothForms::new(&[('丙', '丁')], &[]).unwrap(),
    )
}

#[test]
fn both_forms_get_the_probabilities_of_the_representative() {
    let (model, summary) = with_both_forms("甲乙\n甲乙\n甲丙\n乙丙\n丙甲\n");
    for prev in [None, Some('甲'), Some('乙')] {
        assert_eq!(
            model.log_prob(prev, '丁'),
            model.log_prob(prev, '丙'),
            "{prev:?}"
        );
    }
    assert_eq!(
        model.log_prob(Some('丁'), '甲'),
        model.log_prob(Some('丙'), '甲')
    );
    assert_eq!(
        model.log_prob(Some('丁'), '乙'),
        model.log_prob(Some('丙'), '乙')
    );
    // 甲丙、乙丙、丙甲各抄一對。
    assert_eq!((summary.kept, summary.copied), (4, 3));
}

#[test]
fn both_forms_are_counted_under_the_representative() {
    // 語料寫丁，和寫丙訓練出同一個模型。
    assert_eq!(
        with_both_forms("甲乙\n甲丁\n乙丙\n").0,
        with_both_forms("甲乙\n甲丙\n乙丙\n").0
    );
}
