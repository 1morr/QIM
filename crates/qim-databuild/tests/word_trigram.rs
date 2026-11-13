//! 詞級 trigram 的訓練（`qim_databuild::word_trigram`，plan〈字級 bigram 與詞級 trigram〉）：剪枝後機率加總仍為 1、α 夾在 1
//! 以下、各語料各算折扣、用現行打分切段、句首也是一段。

use std::io::BufRead;

use qim_core::{CLAUSE_START, CharModel, LanguageModel, Lexicon, Word};
use qim_databuild::word_trigram::{
    self, Count, CurrentScoring, LARGE, Lookup, Pruning, Table, discount_of, merge,
};
use rustc_hash::FxHashMap;

fn counts(pairs: &[(u64, u32, u64)]) -> FxHashMap<(u64, u32), Count> {
    pairs
        .iter()
        .map(|&(ctx, next, n)| {
            (
                (ctx, next),
                Count {
                    weighted: n,
                    // 同字級 bigram 的 D：這組 n1 = 2、n2 = 0。
                    discount: 1.0,
                },
            )
        })
        .collect()
}

/// 低一階是 4 個後一段平均分；同一個前文下 4 個的機率（留下的直接查，沒留下的乘 α）加起來是 1。
fn total(table: &Table, ctx: u64) -> f64 {
    (0..4)
        .map(|next| match table.lookup(ctx, next) {
            Lookup::Hit(lp) => lp.exp(),
            Lookup::Backoff(ln_alpha) => (ln_alpha + 0.25f64.ln()).exp(),
            Lookup::Unseen => 0.25,
        })
        .sum()
}

#[test]
fn probabilities_still_sum_to_one_after_pruning() {
    let data = counts(&[(7, 0, 9), (7, 1, 3), (7, 2, 1), (8, 3, 1)]);
    for min_count in [1, 2, 4] {
        let (table, stats) = Table::build(&data, min_count, |_, _| 0.25);
        // 機率量化成 1/12 nat 的級距，加總差幾個百分點內。
        assert!((total(&table, 7) - 1.0).abs() < 0.05, "門檻 {min_count}");
        assert_eq!(stats.types, 4);
    }
    let (table, _) = Table::build(&data, 2, |_, _| 0.25);
    assert_eq!(
        table.lookup(8, 3),
        Lookup::Unseen,
        "前文 8 沒有留下任何一個"
    );
    assert!(matches!(table.lookup(7, 2), Lookup::Backoff(_)));
}

#[test]
fn backoff_weight_is_clamped_at_one() {
    // 低一階幾乎全給了留下的那一個（罕用的前文常這樣）：重算的 α 會超過 1，夾成 1。
    let data = counts(&[(1, 0, 3), (1, 1, 2), (1, 2, 2)]);
    let (table, stats) = Table::build(&data, 3, |_, next| if next == 0 { 0.99 } else { 0.005 });
    assert_eq!(stats.clamped, 1);
    assert_eq!(table.lookup(1, 1), Lookup::Backoff(0.0));
}

#[test]
fn per_corpus_discounts_scale_with_the_weight() {
    // 維基：甲乙 兩次、乙丙 一次 → n1 = 1、n2 = 1，D = 1/3。口語：甲乙 一次 → D = 1。
    let mut total: FxHashMap<(char, char), Count> = FxHashMap::default();
    merge(
        &mut total,
        FxHashMap::from_iter([(('甲', '乙'), 2), (('乙', '丙'), 1)]),
        1,
    );
    merge(&mut total, FxHashMap::from_iter([(('甲', '乙'), 1)]), 64);
    let ab = total[&('甲', '乙')];
    assert_eq!(ab.weighted, 2 + 64);
    assert!((ab.discount - (1.0 / 3.0 + 64.0)).abs() < 1e-9);
    assert!((total[&('乙', '丙')].discount - 1.0 / 3.0).abs() < 1e-9);
}

#[test]
fn discount_follows_the_counts_of_counts() {
    assert!((discount_of([1, 1, 2, 5].into_iter()) - 0.5).abs() < 1e-9);
    assert!((discount_of([3, 4].into_iter()) - 0.5).abs() < 1e-9);
}

/// 「門」「口」「問」三字，詞庫有「門口」。
fn language_model() -> LanguageModel {
    let model = |p: f64| CharModel {
        start: p.ln(),
        continuation: p.ln(),
        backoff: 0.0,
    };
    LanguageModel::new(
        [
            ('門', model(0.3)),
            ('口', model(0.3)),
            ('問', model(0.3)),
            (CLAUSE_START, model(0.01)),
        ],
        model(0.001),
        [],
    )
}

fn lexicon() -> Lexicon {
    Lexicon::new(
        [(
            "anr".to_string(),
            vec![Word {
                text: "門口".into(),
                frequency: 100,
            }],
        )],
        [],
    )
}

#[test]
fn sentences_split_into_the_words_the_current_scoring_prefers() {
    let model = language_model();
    let scoring = CurrentScoring::new(&model, &lexicon(), 1.0, 1.0);
    let line = "問門口";
    let segments: Vec<&str> = scoring
        .segment(line)
        .into_iter()
        .map(|(i, j)| &line[i..j])
        .collect();
    assert_eq!(segments, ["問", "門口"]);
    // 不是字也不是詞庫的詞的段沒有分數。
    assert_eq!(scoring.log_prob(Some('問'), "口門"), None);
}

fn reader(text: &str) -> Box<dyn BufRead + '_> {
    Box::new(text.as_bytes())
}

#[test]
fn training_keeps_the_frequent_word_trigrams_and_only_the_words_they_use() {
    let model = language_model();
    let scoring = CurrentScoring::new(&model, &lexicon(), 1.0, 1.0);
    let wiki = "問門口\n".repeat(10) + "口問\n";
    let counts =
        word_trigram::count(vec![(reader(&wiki), 1), (reader("門口問\n"), 64)], &scoring).unwrap();
    // 句首是編號 0，其餘照第一次出現的順序。
    assert_eq!(&*counts.vocabulary[0], "\u{2}");
    assert_eq!(counts.vocabulary.len(), 4);
    let pruning = Pruning {
        min_bigram: 8,
        min_trigram: 8,
    };
    let (trained, summary) = word_trigram::train(&scoring, &counts, "t", pruning);
    assert_eq!(summary.segments, 4);
    // 「口問」只在維基出現一次：含「口」的 n-gram 都沒留下，詞表就沒有它。
    assert_eq!(trained.len(), 3);
    assert_eq!(trained.id("口"), None);
    let lower = -10.0;
    let after_start = trained.log_prob(&["\u{2}"], "問", lower);
    assert!(after_start > -1.0, "句首之後的「問」：{after_start}");
    let trigram = trained.log_prob(&["\u{2}", "問"], "門口", lower);
    assert!(trigram > -0.5, "句首、問之後的門口：{trigram}");
    // 沒見過的前文照低一階。
    assert_eq!(trained.log_prob(&["日"], "問", lower), lower);
    // 放寬版留得比預算版多。
    let (_, large) = word_trigram::train(&scoring, &counts, "t-large", LARGE);
    assert!(large.bigram.kept >= summary.bigram.kept);
}
