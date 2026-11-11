//! 依句長分組（M3 票 25）與前文長度的分佈。

use model_ladder::{
    ContextStats, Group, LENGTH_BUCKETS, Row, Set, SetRun, Split, by_length, read_length_rows,
    write_rows,
};

fn row(line: usize, chars: usize, correct: bool, top2: bool) -> Row {
    Row {
        line,
        rank: None,
        path_rank: None,
        correct,
        top2,
        model_first: correct,
        correction_count: 1,
        finished: true,
        chars,
    }
}

/// 分組是 1–4、5–8、9–14、15 字以上；邊界的字數各落在它那組。
#[test]
fn rows_fall_into_length_buckets_at_the_boundaries() {
    let labels: Vec<&str> = LENGTH_BUCKETS.iter().map(|b| b.label).collect();
    assert_eq!(labels, ["1–4", "5–8", "9–14", "15+"]);
    let rows = [
        (1, true, true),
        (4, false, true),
        (5, true, true),
        (8, false, false),
        (9, false, true),
        (14, false, false),
        (15, true, true),
        (40, false, false),
    ];
    let cells = by_length(rows.iter().copied());
    let got: Vec<(usize, usize, usize)> =
        cells.iter().map(|c| (c.total, c.correct, c.top2)).collect();
    assert_eq!(got, [(2, 1, 2), (2, 1, 1), (2, 0, 1), (2, 1, 1)]);
    let all = model_ladder::LengthCell::sum(&cells);
    assert_eq!((all.total, all.correct, all.top2), (8, 3, 5));
}

/// 逐題結果檔多一欄字數，讀回來分得了組；舊的讀法（`compare`）不受影響。
#[test]
fn written_rows_carry_the_answer_length() {
    let dir = std::env::temp_dir().join(format!("model-ladder-by-length-{}", std::process::id()));
    let run = SetRun {
        set: Set {
            name: "s".into(),
            split: Split::Dev,
            group: Group::Colloquial,
            context_chars: 0,
            private: false,
            items: Vec::new(),
        },
        rows: vec![row(3, 6, true, true), row(7, 15, false, true)],
        path_chars: Vec::new(),
        path_counts: Vec::new(),
        rerank_ms: Vec::new(),
        unseen: 0,
    };
    write_rows(&dir, &[run]).unwrap();
    let rows = read_length_rows(&dir.join("s.tsv")).unwrap();
    assert_eq!(rows, [(6, true, true), (15, false, true)]);
    let compared = model_ladder::read_rows(&dir.join("s.tsv")).unwrap();
    assert_eq!(compared[&7], [false, true, false]);
    std::fs::remove_dir_all(dir).unwrap();
}

/// 前文長度的分佈：p50（最近秩）與到 50、200 字的比例（剛好 50 字算到）。
#[test]
fn context_stats_report_median_and_share_reaching_50_and_200() {
    let stats = ContextStats::of(&[250, 0, 10, 49, 50, 199, 200]);
    assert_eq!(stats.total, 7);
    assert_eq!(stats.p50, Some(50));
    assert_eq!(stats.at_least_50, 4);
    assert_eq!(stats.at_least_200, 2);
    assert_eq!(ContextStats::of(&[]).p50, None);
}
