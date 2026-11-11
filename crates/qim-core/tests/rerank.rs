//! 重排（plan §2.1、§2.7、ADR 0005）：殼取 `rerank_request`，背景模型回傳名次，`apply_rerank`
//! 只接受目前的 `generation`，重排過的候選最高到第 2 位。

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, Engine, Key, Pack, Packs, Ranking, Scheme, SchemePack, Session, Snapshot,
    UserModel,
};

/// `a` 有 10 個字（比一頁多一個），`b` 有一個字。
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
    let entries = |chars: &str| {
        chars
            .chars()
            .map(|text| CodeEntry {
                text,
                extension: false,
                simplified: false,
            })
            .collect()
    };
    let code_table = CodeTable::from_entries([
        ("a".to_string(), entries("日曰昌晶昍旦早旭旨旬")),
        ("b".to_string(), entries("月")),
    ]);
    let bytes = SchemePack { scheme, code_table }.to_bytes().unwrap();
    Engine::from_pack_bytes(&bytes, None, None).unwrap()
}

struct Ime {
    engine: Engine,
    user: UserModel,
    session: Session,
}

impl Ime {
    fn new() -> Self {
        Self {
            engine: engine(),
            user: UserModel::default(),
            session: Session::default(),
        }
    }

    fn press(&mut self, key: Key) -> Snapshot {
        self.session.handle(&self.engine, &mut self.user, key)
    }
}

fn texts(snapshot: &Snapshot) -> Vec<String> {
    snapshot.candidates.iter().map(|c| c.text.clone()).collect()
}

#[test]
fn a_result_for_an_old_generation_is_dropped() {
    let mut ime = Ime::new();
    let old = ime.press(Key::Letter('a')).generation;
    ime.press(Key::Backspace);
    ime.press(Key::Letter('a'));
    assert_eq!(ime.session.apply_rerank(&ime.engine, old, &[1, 0]), None);
}

/// ADR 0005：慢的模型不能動首選，最高到第 2 位；其餘照它給的名次，它沒提到的照原順序接在後面。
#[test]
fn raising_the_fifth_to_the_front_stops_at_second_and_keeps_the_top() {
    let mut ime = Ime::new();
    let typed = ime.press(Key::Letter('a'));
    let request = ime.session.rerank_request().unwrap();
    assert_eq!(request.generation, typed.generation);
    let reranked = ime
        .session
        .apply_rerank(&ime.engine, request.generation, &[4, 0, 1, 2, 3])
        .unwrap();
    assert_eq!(
        texts(&reranked),
        ["日", "昍", "曰", "昌", "晶", "旦", "早", "旭", "旨"]
    );
    // 選字照重排後的順序。
    assert_eq!(ime.press(Key::Digit(2)).commit.as_deref(), Some("昍"));
}

/// 套用後名次變了，同一份要求的另一個結果（另一層模型）照的是舊順序，要丟掉；名次沒變時不作廢。
#[test]
fn only_a_result_that_changes_the_order_makes_the_request_stale() {
    let mut ime = Ime::new();
    ime.press(Key::Letter('a'));
    let request = ime.session.rerank_request().unwrap();
    let same = ime
        .session
        .apply_rerank(&ime.engine, request.generation, &[0, 1, 2])
        .unwrap();
    assert_eq!(same.generation, request.generation);
    let changed = ime
        .session
        .apply_rerank(&ime.engine, request.generation, &[2])
        .unwrap();
    assert_ne!(changed.generation, request.generation);
    assert_eq!(
        ime.session
            .apply_rerank(&ime.engine, request.generation, &[3]),
        None
    );
}

/// 翻頁不是候選變動；翻頁中收到的結果照樣套用，只動第一頁，看得到的這頁不變（M1 票 23）。
#[test]
fn a_result_arriving_on_the_second_page_applies_to_the_first_page_only() {
    let mut ime = Ime::new();
    ime.press(Key::Letter('a'));
    let request = ime.session.rerank_request().unwrap();
    let turned = ime.press(Key::PageDown);
    assert_eq!(turned.generation, request.generation);
    let reranked = ime
        .session
        .apply_rerank(&ime.engine, request.generation, &[4])
        .unwrap();
    assert_eq!(reranked.page, 1);
    assert_eq!(texts(&reranked), ["旬"]);
    assert_eq!(texts(&ime.press(Key::PageUp))[..2], ["日", "昍"]);
}

#[test]
fn positions_outside_the_request_or_repeated_are_ignored() {
    let mut ime = Ime::new();
    ime.press(Key::Letter('a'));
    let request = ime.session.rerank_request().unwrap();
    assert_eq!(request.candidates.len(), 9);
    // 第 10 個（`9`）不在要求裡。
    let reranked = ime
        .session
        .apply_rerank(&ime.engine, request.generation, &[9, 2, 2, 40])
        .unwrap();
    assert_eq!(texts(&reranked)[..3], ["日", "昌", "曰"]);
}

#[test]
fn there_is_nothing_to_rerank_without_a_composition() {
    let mut ime = Ime::new();
    assert_eq!(ime.session.rerank_request(), None);
    ime.press(Key::Letter('a'));
    ime.press(Key::Space);
    assert_eq!(ime.session.rerank_request(), None);
}

/// 上下文是候選前面的字詞：連打時是選定的分段。
#[test]
fn the_context_is_the_selected_segments_before_the_candidates() {
    let mut ime = Ime::new();
    ime.press(Key::Letter('b'));
    ime.press(Key::Letter('a'));
    let request = ime.session.rerank_request().unwrap();
    assert_eq!(request.context, "");
    // `ba` 的候選是「月日」「月曰」「月昌」「月」：選只蓋住 `b` 的「月」，剩下 `a`。
    let selected = ime.press(Key::Digit(4));
    assert_eq!(selected.composition.input, "ba");
    let request = ime.session.rerank_request().unwrap();
    assert_eq!(request.context, "月");
    assert_eq!(request.candidates[0].text, "日");
}

/// 移到某一段時，候選是那一段的，上下文是它前面的分段的原文。
#[test]
fn on_a_focused_segment_the_context_is_the_segments_before_it() {
    let mut ime = Ime::new();
    ime.press(Key::Letter('b'));
    ime.press(Key::Letter('a'));
    let focused = ime.press(Key::Left);
    assert_eq!(focused.composition.focus, 1);
    let request = ime.session.rerank_request().unwrap();
    assert_eq!(request.context, "月");
    assert_eq!(request.candidates[0].text, "日");
}

/// 前 n 條整串路徑（M3 票 07，模型階梯）：lattice 蓋滿沒選定的輸入碼的部分路徑，依分數由高到低、
/// 附分數；最多 beam 條。和候選列不同，多段路徑不受方案的 `sentences` 限制。
#[test]
fn paths_list_the_best_whole_paths_with_scores() {
    let mut ime = Ime::new();
    assert!(ime.session.paths(5).is_empty(), "沒有組字");
    ime.press(Key::Letter('a'));
    ime.press(Key::Letter('b'));
    let paths = ime.session.paths(3);
    let texts: Vec<&str> = paths.iter().map(|p| p.text.as_str()).collect();
    assert_eq!(texts, ["日月", "曰月", "昌月"]);
    assert!(paths.windows(2).all(|w| w[0].score >= w[1].score));
    // `a` 的 10 個字各接「月」：全部 10 條，候選列只有方案的 3 條句子。
    assert_eq!(ime.session.paths(100).len(), 10);
    let whole = ime
        .session
        .all_candidates()
        .iter()
        .filter(|c| c.text.chars().count() == 2)
        .count();
    assert_eq!(whole, usize::from(Ranking::default().sentences));
}

/// 選了開頭一段之後，路徑只蓋剩下的輸入碼。
#[test]
fn paths_cover_only_the_input_left_after_a_selected_beginning() {
    let mut ime = Ime::new();
    ime.press(Key::Letter('a'));
    ime.press(Key::Letter('b'));
    let at = ime
        .session
        .all_candidates()
        .iter()
        .position(|c| c.text == "曰")
        .unwrap();
    ime.session.select(&ime.engine, &mut ime.user, at);
    let texts: Vec<String> = ime.session.paths(5).into_iter().map(|p| p.text).collect();
    assert_eq!(texts, ["月"]);
}
