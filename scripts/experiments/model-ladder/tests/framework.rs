//! 框架的行為：重排只動第 2 位起、N 截斷、外部結果查表、前文截斷、配對檢定。
//! 碼表：`c` → 一到十（兩頁）、`a` → 日曰、`b` → 月。

use std::collections::BTreeMap;

use model_ladder::{
    Group, Identity, Orders, Query, Reranker, Set, Summary, mcnemar_exact, run_set, type_item,
    with_context_chars,
};
use qim_bench::{Item, Unit};
use qim_core::{CodeEntry, CodeTable, Engine, Packs, Ranking, Scheme, SchemePack};

fn engine() -> Engine {
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        radicals: BTreeMap::new(),
        packs: Packs::default(),
        ranking: Ranking::default(),
    };
    let table = [("a", "日曰"), ("b", "月"), ("c", "一二三四五六七八九十")];
    let code_table = CodeTable::from_entries(table.iter().map(|(code, chars)| {
        (
            code.to_string(),
            chars
                .chars()
                .map(|text| CodeEntry {
                    text,
                    extension: false,
                })
                .collect(),
        )
    }));
    Engine::new(SchemePack { scheme, code_table }, None, None).unwrap()
}

/// 一個字一個單位：`codes` 與 `text` 逐字對應。
fn item(line: usize, codes: &str, text: &str, context: Option<&str>) -> Item {
    Item {
        units: codes
            .split(' ')
            .zip(text.chars())
            .map(|(code, c)| Unit {
                code: code.into(),
                text: c.to_string(),
            })
            .collect(),
        line,
        label: line.to_string(),
        weight: 1,
        context: context.map(Into::into),
    }
}

/// 把指定的文字排第一的模型（只排看得到的：候選或路徑裡有才排）；記下看到的題。
struct Oracle {
    want: String,
    seen: Vec<Query>,
}

impl Reranker for Oracle {
    fn name(&self) -> &str {
        "oracle"
    }
    fn rerank(&mut self, q: &Query) -> Vec<String> {
        self.seen.push(q.clone());
        let visible =
            q.candidates.contains(&self.want) || q.paths.iter().any(|p| p.text == self.want);
        visible.then(|| self.want.clone()).into_iter().collect()
    }
}

fn oracle(want: &str) -> Oracle {
    Oracle {
        want: want.into(),
        seen: Vec::new(),
    }
}

#[test]
fn baseline_reports_the_rank_before_reranking() {
    let typed = type_item(&engine(), "s", &item(1, "c", "三", None), 9, &mut Identity);
    assert_eq!(typed.candidates.iter().position(|c| c == "三"), Some(2));
    assert_eq!(typed.shown_top2(), [Some("一"), Some("二")]);
}

#[test]
fn a_reranker_moves_the_answer_to_second_but_never_first() {
    let engine = engine();
    let set = Set {
        name: "s".into(),
        group: Group::Wiki,
        context_chars: 0,
        private: false,
        items: vec![item(1, "c", "三", None)],
    };
    let base = Summary::of(&run_set(&engine, &set, 9, &mut Identity).rows);
    let run = run_set(&engine, &set, 9, &mut oracle("三"));
    let row = &run.rows[0];
    assert_eq!((base.correct, base.top2, base.free_top), (0, 0, 0));
    assert_eq!(row.rank, Some(2), "名次是重排前的");
    assert!(!row.correct, "首選不動");
    assert!(row.top2 && row.free_top);
    // 修正次數照基準的模擬，不受重排影響。
    assert_eq!(row.corrections, base.corrections);
}

/// 模型可以挑候選列沒有、lattice 裡有的路徑：方案只放 3 條句子（一月、二月、三月），
/// 第 5 條「五月」不在候選列，排到第 2 名；N 不到 5 時看不到它。
#[test]
fn a_reranker_can_raise_a_path_that_is_not_on_the_candidate_list() {
    let engine = engine();
    let it = item(1, "c b", "五月", None);
    let base = type_item(&engine, "s", &it, 9, &mut Identity);
    assert!(!base.candidates.contains(&"五月".to_string()));
    assert_eq!(base.paths.iter().position(|p| p.text == "五月"), Some(4));
    let typed = type_item(&engine, "s", &it, 9, &mut oracle("五月"));
    assert_eq!(typed.shown_top2(), [Some("一月"), Some("五月")]);
    let short = type_item(&engine, "s", &it, 4, &mut oracle("五月"));
    assert_eq!(short.shown_top2()[1], Some("二月"));
}

#[test]
fn the_reranker_sees_the_first_page_n_paths_and_the_context() {
    let mut o = oracle("十");
    let typed = type_item(&engine(), "s", &item(1, "c", "十", Some("前文")), 3, &mut o);
    let q = &o.seen[0];
    assert_eq!(q.before, "前文");
    assert_eq!(q.input, "c");
    assert_eq!(q.candidates.len(), 9, "候選列第一頁");
    assert_eq!(q.paths.len(), 3, "路徑只留前 N 條");
    assert_eq!(
        typed.shown_top2(),
        [Some("一"), Some("二")],
        "十在第二頁、路徑也只看三條，挑不到"
    );
}

#[test]
fn orders_are_looked_up_by_set_and_line_and_missing_ones_are_counted() {
    let mut orders = Orders::parse(
        "ext",
        "{\"set\":\"s\",\"line\":1,\"ranked\":[\"一\",\"三\"]}\n",
    )
    .unwrap();
    let engine = engine();
    let hit = type_item(&engine, "s", &item(1, "c", "三", None), 9, &mut orders);
    let miss = type_item(&engine, "s", &item(2, "c", "三", None), 9, &mut orders);
    assert_eq!(
        hit.shown_top2(),
        [Some("一"), Some("三")],
        "排第一的是首選時取下一個"
    );
    assert_eq!(miss.shown_top2(), [Some("一"), Some("二")]);
    assert_eq!(orders.missing, 1);
}

#[test]
fn context_is_cut_to_its_last_chars_and_zero_drops_it() {
    let it = item(1, "a", "日", Some("一二三四五"));
    assert_eq!(
        with_context_chars(&it, 3).context.as_deref(),
        Some("三四五")
    );
    assert_eq!(
        with_context_chars(&it, 50).context.as_deref(),
        Some("一二三四五")
    );
    assert_eq!(with_context_chars(&it, 0).context, None);
}

#[test]
fn mcnemar_exact_matches_the_binomial_tail() {
    assert_eq!(mcnemar_exact(0, 0), 1.0);
    // 0 對 5：2 × 0.5^5。
    assert!((mcnemar_exact(0, 5) - 0.0625).abs() < 1e-12);
    // 1 對 9：2 × (1 + 10) / 1024。
    assert!((mcnemar_exact(9, 1) - 22.0 / 1024.0).abs() < 1e-12);
    assert_eq!(mcnemar_exact(5, 5), 1.0);
}
