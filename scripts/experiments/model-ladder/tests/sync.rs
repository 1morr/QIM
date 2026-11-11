//! 同步層模型（`model_ladder::sync`）：剪枝後機率加總仍為 1、α 夾在 1 以下、各語料各算折扣、句首的
//! trigram、用現行打分切詞；各模型查到留下的 n-gram 才改分數，沒見過的前文照現行打分。

use std::io::Write as _;

use model_ladder::sync::{
    CharTrigram, Count, CurrentScoring, Discounting, KnBigram, Lookup, MixedBigram, Table,
    TrainCorpus, Vocabulary, WordCounts, WordNgram, char_trigrams, count, discount_of,
};
use qim_core::{CLAUSE_START, CharModel, LanguageModel, Lexicon, SegmentModel, SegmentQuery, Word};
use rustc_hash::FxHashMap;

fn counts(pairs: &[(u64, u32, u64)]) -> FxHashMap<(u64, u32), Count> {
    pairs
        .iter()
        .map(|&(ctx, next, n)| {
            (
                (ctx, next),
                Count {
                    weighted: n,
                    per_corpus: 0.0,
                },
            )
        })
        .collect()
}

/// 低一階是 4 個字平均分；同一個前文下 4 個字的機率（留下的直接查，沒留下的乘 α）加起來是 1。
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
        let (table, stats) = Table::build(&data, Discounting::Weighted, min_count, |_, _| 0.25);
        // 機率量化成 1/12 nat 的級距，加總差幾個百分點內。
        assert!((total(&table, 7) - 1.0).abs() < 0.05, "門檻 {min_count}");
        assert_eq!(stats.types, 4);
    }
    let (table, _) = Table::build(&data, Discounting::Weighted, 2, |_, _| 0.25);
    assert_eq!(
        table.lookup(8, 3),
        Lookup::Unseen,
        "前文 8 沒有留下任何一個"
    );
    assert!(matches!(table.lookup(7, 2), Lookup::Backoff(_)));
}

#[test]
fn per_corpus_discounts_scale_with_the_weight() {
    let dir = std::env::temp_dir().join(format!("model-ladder-sync-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let write = |name: &str, text: &str| {
        let path = dir.join(name);
        std::fs::File::create(&path)
            .unwrap()
            .write_all(text.as_bytes())
            .unwrap();
        path
    };
    // 維基：甲乙 兩次、乙丙 一次 → n1 = 1、n2 = 1，D = 1/3。口語：甲乙 一次 → D = 1。
    let corpora = [
        TrainCorpus {
            name: "wiki".into(),
            path: write("wiki.txt", "甲乙\n甲乙\n乙丙\n"),
            weight: 1,
            colloquial: false,
        },
        TrainCorpus {
            name: "talk".into(),
            path: write("talk.txt", "甲乙\n"),
            weight: 64,
            colloquial: true,
        },
    ];
    let pairs = count(
        &corpora,
        |line: &str, emit: &mut dyn FnMut((char, char))| {
            let chars: Vec<char> = line.chars().collect();
            for w in chars.windows(2) {
                emit((w[0], w[1]));
            }
        },
    )
    .unwrap();
    let ab = pairs[&('甲', '乙')];
    assert_eq!(ab.weighted, 2 + 64);
    assert!((ab.per_corpus - (1.0 / 3.0 + 64.0)).abs() < 1e-9);
    assert!((pairs[&('乙', '丙')].per_corpus - 1.0 / 3.0).abs() < 1e-9);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn discount_follows_the_counts_of_counts() {
    assert!((discount_of([1, 1, 2, 5].into_iter()) - 0.5).abs() < 1e-9);
    assert!((discount_of([3, 4].into_iter()) - 0.5).abs() < 1e-9);
}

#[test]
fn the_first_trigram_of_a_clause_starts_at_the_clause_start() {
    let mut seen = Vec::new();
    char_trigrams("甲乙丙", &mut |key| seen.push(key));
    let pair = |a: char, b: char| (u64::from(u32::from(a)) << 21) | u64::from(u32::from(b));
    assert_eq!(
        seen,
        vec![
            (pair(CLAUSE_START, '甲'), u32::from('乙')),
            (pair('甲', '乙'), u32::from('丙')),
        ]
    );
}

fn scoring(word_frequency: u64) -> CurrentScoring {
    let model = |p: f64| CharModel {
        start: p.ln(),
        continuation: p.ln(),
        backoff: 0.0,
    };
    let base = LanguageModel::new(
        [('門', model(0.1)), ('口', model(0.1)), ('問', model(0.1))],
        model(0.01),
        [],
    );
    let lexicon = Lexicon::new(
        [(
            "anr".to_string(),
            vec![Word {
                text: "門口".into(),
                frequency: word_frequency,
            }],
        )],
        [],
    );
    CurrentScoring::new(base, &lexicon, 1.0, 1.0)
}

#[test]
fn clauses_are_segmented_with_the_current_scoring() {
    // 詞庫只有一個詞，詞頻比例是 1，詞內機率 1 ÷ 0.1：整個詞比拆成兩字好。
    assert_eq!(scoring(10).segment("問門口"), vec![(0, 3), (3, 9)]);
    // 拆不開的字照字切；一個字不在模型裡也照切。
    assert_eq!(scoring(10).segment("口門"), vec![(0, 3), (3, 6)]);
}

/// 低一階幾乎全給了留下的那個（罕用字的 bigram 列常這樣）、又剪掉別的：重算的 α 會爆大，
/// 沒留下的 trigram 反而比低一階高得多。α 夾在 1 以下。
#[test]
fn backoff_never_raises_an_unseen_event_above_the_lower_order() {
    let mut data = counts(&[(7, 0, 5), (7, 1, 1), (7, 2, 1)]);
    for c in data.values_mut() {
        c.per_corpus = 0.5;
    }
    let lower = |_: u64, next: u32| if next == 0 { 0.999_999 } else { 1e-7 };
    let (table, _) = Table::build(&data, Discounting::PerCorpus, 2, lower);
    match table.lookup(7, 1) {
        Lookup::Backoff(ln_alpha) => assert!(ln_alpha <= 0.0, "ln α = {ln_alpha}"),
        other => panic!("{other:?}"),
    }
}

fn query<'a>(before: &'a [&'a str], text: &'a str) -> SegmentQuery<'a> {
    SegmentQuery {
        before,
        text,
        current_log_prob: -5.0,
    }
}

fn pair(a: char, b: char) -> u64 {
    (u64::from(u32::from(a)) << 21) | u64::from(u32::from(b))
}

#[test]
fn a_char_trigram_changes_the_score_only_after_two_known_chars() {
    let base = scoring(10).base;
    let data = counts(&[(pair('門', '口'), u32::from('問'), 9)]);
    let (model, _) = CharTrigram::train(&base, &data, Discounting::Weighted, 1);
    // 前兩字跨過段界也算：「門」「口」各一段。
    let hit = model.log_prob(&query(&["門", "口"], "問"));
    let (Lookup::Hit(lp), low) = model.explain('門', '口', '問') else {
        panic!("門口問 要留下");
    };
    assert!((hit - (-5.0 - low + lp)).abs() < 1e-9);
    assert!(hit > -5.0);
    // 只有一個字、沒有前文、前文沒有統計：照現行。
    assert_eq!(model.log_prob(&query(&["口"], "問")), -5.0);
    assert_eq!(model.log_prob(&query(&[], "問")), -5.0);
    assert_eq!(model.log_prob(&query(&["問", "門"], "口")), -5.0);
}

#[test]
fn a_word_ngram_backs_off_to_the_current_scoring() {
    let scoring = scoring(10);
    let mut vocabulary = Vocabulary::new();
    let ask = vocabulary.intern("問");
    let door = vocabulary.intern("門口");
    let mouth = vocabulary.intern("口");
    let mut counts = WordCounts {
        vocabulary,
        ..WordCounts::default()
    };
    for (next, n) in [(door, 9), (mouth, 1)] {
        counts.bigrams.insert(
            (u64::from(ask), next),
            Count {
                weighted: n,
                per_corpus: 0.0,
            },
        );
    }
    let (model, _, _) = WordNgram::train(&scoring, &counts, Discounting::Weighted, 2, None);
    // 「問 門口」留下：分數換掉；「問 口」剪掉：乘 α（≤ 1）接回現行打分。
    assert_ne!(model.log_prob(&query(&["問"], "門口")), -5.0);
    let backed_off = model.log_prob(&query(&["問"], "口"));
    assert!(backed_off <= -5.0);
    // 表裡沒有的段也一樣乘 α；前一段不在表裡、沒有前一段時照現行。
    assert_eq!(model.log_prob(&query(&["問"], "門")), backed_off);
    assert_eq!(model.log_prob(&query(&["門"], "口")), -5.0);
    assert_eq!(model.log_prob(&query(&[], "門口")), -5.0);
}

#[test]
fn bigram_retrains_keep_the_current_score_without_a_previous_char() {
    let base = scoring(10).base;
    let data = counts(&[(u64::from(u32::from('門')), u32::from('口'), 9)]);
    let (model, _) = KnBigram::train(&base, &data, 3, Discounting::Weighted, 1);
    assert_eq!(model.log_prob(&query(&[], "口")), -5.0);
    assert_ne!(model.log_prob(&query(&["門"], "口")), -5.0);
    // 兩份都是現行的模型時，內插回到現行的分數。
    let mixed = MixedBigram::new(base.clone(), base.clone(), base, 0.3);
    assert!((mixed.log_prob(&query(&["門"], "口")) - -5.0).abs() < 1e-9);
}
