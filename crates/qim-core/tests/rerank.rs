//! 重排（plan §2.1、§2.7、ADR 0005，M3 票 11c）：殼取 `rerank_request`（前文與 lattice 的前 9 條
//! 整句路徑），背景模型回傳路徑的名次，`apply_rerank` 只接受目前的 `generation`，把第一條和首選
//! 不同的路徑放到第 2 位：候選列有它就移上來，沒有就插入；首選不動。

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, ConversionTable, Engine, EnglishPack, EnglishWords, Key, Options, Pack,
    Packs, RERANK_CONTEXT, RERANK_PATHS, Ranking, Scheme, SchemePack, Session, ShiftKey,
    Simplifier, SimplifierPack, Snapshot, UserModel,
};

/// `a` 有 10 個字（比一頁多一個），`b` 有一個字；英文只有 `abba`。
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
    Engine::from_pack_bytes(&bytes, None, None)
        .unwrap()
        .with_english(EnglishPack {
            id: "english".into(),
            words: EnglishWords::new(["abba"]),
        })
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

    /// 簡體模式：「曰」轉成「日」、「晶」轉成「昌」，`ab` 的「曰月」與首選轉換後相同。
    fn simplified() -> Self {
        let table = ConversionTable::new([
            ("曰".to_string(), "日".to_string()),
            ("晶".to_string(), "昌".to_string()),
        ]);
        let simplifier = SimplifierPack {
            id: "opencc-hk2s".into(),
            simplifier: Simplifier::new(vec![], ConversionTable::new([]), vec![vec![table]]),
        };
        let mut ime = Self {
            engine: engine().with_simplifier(simplifier),
            ..Self::new()
        };
        ime.session.set_options(Options {
            simplified: true,
            ..Options::default()
        });
        ime
    }

    fn press(&mut self, key: Key) -> Snapshot {
        self.session.handle(&self.engine, &mut self.user, key)
    }

    fn type_code(&mut self, code: &str) -> Snapshot {
        let mut snapshot = Snapshot::default();
        for c in code.chars() {
            snapshot = self.press(Key::Letter(c));
        }
        snapshot
    }

    fn all(&self) -> Vec<String> {
        self.session
            .all_candidates()
            .into_iter()
            .map(|c| c.text)
            .collect()
    }

    /// 要求裡文字是 `text` 的路徑的序號。
    fn path(&self, text: &str) -> usize {
        let request = self.session.rerank_request(&self.engine).unwrap();
        request
            .paths
            .iter()
            .position(|p| p.text == text)
            .unwrap_or_else(|| panic!("要求裡沒有 {text}：{:?}", request.paths))
    }

    /// 取目前的要求，模型把 `texts` 這幾條路徑排在最前（依序），套用回傳的快照。
    fn rerank(&mut self, texts: &[&str]) -> Snapshot {
        let request = self.session.rerank_request(&self.engine).unwrap();
        let order: Vec<usize> = texts.iter().map(|t| self.path(t)).collect();
        self.session
            .apply_rerank(&self.engine, request.generation, &order)
            .unwrap()
    }

    /// 打 `code`、選全部候選裡的 `text`，回傳上屏的字。
    fn choose(&mut self, code: &str, text: &str) -> Option<String> {
        self.type_code(code);
        let at = self.all().iter().position(|t| t == text).unwrap();
        self.session.select(&self.engine, &mut self.user, at).commit
    }
}

fn texts(snapshot: &Snapshot) -> Vec<String> {
    snapshot.candidates.iter().map(|c| c.text.clone()).collect()
}

/// `ab` 的第一頁：方案的 3 條句子，接著只蓋住 `a` 的字。
const AB: [&str; 9] = ["日月", "曰月", "昌月", "日", "曰", "昌", "晶", "昍", "旦"];

#[test]
fn the_request_carries_the_best_nine_whole_paths_with_segments_and_scores() {
    let mut ime = Ime::new();
    let typed = ime.type_code("ab");
    let request = ime.session.rerank_request(&ime.engine).unwrap();
    assert_eq!(request.generation, typed.generation);
    let paths: Vec<&str> = request.paths.iter().map(|p| p.text.as_str()).collect();
    assert_eq!(
        paths,
        [
            "日月", "曰月", "昌月", "晶月", "昍月", "旦月", "早月", "旭月", "旨月"
        ]
    );
    assert_eq!(request.paths.len(), RERANK_PATHS);
    assert_eq!(request.paths[3].segments, ["晶", "月"]);
    assert!(request.paths.windows(2).all(|w| w[0].score >= w[1].score));
}

/// 候選列只有 3 條句子；模型挑中第 4 條時插到第 2 位，其餘照原順序往後接，候選多一個。
#[test]
fn a_path_missing_from_the_candidates_is_inserted_second() {
    let mut ime = Ime::new();
    assert_eq!(texts(&ime.type_code("ab")), AB);
    let before = ime.all().len();
    assert!(!ime.all().contains(&"晶月".to_string()));
    let asked = ime.session.rerank_request(&ime.engine).unwrap().generation;

    let reranked = ime.rerank(&["晶月"]);
    assert_eq!(
        texts(&reranked),
        ["日月", "晶月", "曰月", "昌月", "日", "曰", "昌", "晶", "昍"]
    );
    assert_eq!(ime.all().len(), before + 1);
    assert_ne!(reranked.generation, asked, "名次變了是候選變動");
}

#[test]
fn a_path_already_in_the_candidates_moves_up_to_second() {
    let mut ime = Ime::new();
    ime.type_code("ab");
    let before = ime.all().len();
    let reranked = ime.rerank(&["昌月"]);
    assert_eq!(
        texts(&reranked),
        ["日月", "昌月", "曰月", "日", "曰", "昌", "晶", "昍", "旦"]
    );
    assert_eq!(ime.all().len(), before, "移動不重複");
}

/// ADR 0005：首選不動。模型排第一的就是首選時，取下一條和首選不同的。
#[test]
fn when_the_best_path_is_the_top_candidate_the_next_one_goes_second() {
    let mut ime = Ime::new();
    ime.type_code("ab");
    let reranked = ime.rerank(&["日月", "旦月", "昌月"]);
    assert_eq!(texts(&reranked)[..3], ["日月", "旦月", "曰月"]);
}

/// 序號超出路徑數的、重複的不算；只剩首選或一個都不剩時名次不變。
#[test]
fn indices_outside_the_paths_or_repeated_are_ignored() {
    let mut ime = Ime::new();
    let typed = ime.type_code("ab");
    let request = ime.session.rerank_request(&ime.engine).unwrap();
    let reranked = ime
        .session
        .apply_rerank(&ime.engine, request.generation, &[40, 0, 0, 9, 5, 5])
        .unwrap();
    assert_eq!(texts(&reranked)[..3], ["日月", "旦月", "曰月"]);

    ime.press(Key::Escape);
    let typed_again = ime.type_code("ab");
    assert_ne!(typed_again.generation, typed.generation);
    let request = ime.session.rerank_request(&ime.engine).unwrap();
    let unchanged = ime
        .session
        .apply_rerank(&ime.engine, request.generation, &[0, 40, 0])
        .unwrap();
    assert_eq!(texts(&unchanged), AB);
    assert_eq!(unchanged.generation, request.generation, "名次沒變不作廢");
}

#[test]
fn a_result_for_an_old_generation_is_dropped() {
    let mut ime = Ime::new();
    ime.type_code("ab");
    let old = ime.session.rerank_request(&ime.engine).unwrap().generation;
    ime.press(Key::Backspace);
    ime.press(Key::Letter('b'));
    assert_eq!(ime.session.apply_rerank(&ime.engine, old, &[3]), None);
}

/// 套用後名次變了，同一份要求的另一個結果（另一層模型）照的是舊順序，要丟掉。
#[test]
fn a_result_that_changes_the_order_makes_the_request_stale() {
    let mut ime = Ime::new();
    ime.type_code("ab");
    let request = ime.session.rerank_request(&ime.engine).unwrap();
    let changed = ime
        .session
        .apply_rerank(&ime.engine, request.generation, &[3])
        .unwrap();
    assert_ne!(changed.generation, request.generation);
    assert_eq!(
        ime.session
            .apply_rerank(&ime.engine, request.generation, &[4]),
        None
    );
}

/// 選插入的候選：照那條路徑的分段上屏與學習，同選一條多段的整句（plan §2.4）：整串上屏一次，
/// 各段只加衰減值，不造詞。
#[test]
fn an_inserted_path_commits_and_learns_like_a_sentence() {
    let mut ime = Ime::new();
    ime.type_code("ab");
    ime.rerank(&["晶月"]);
    let done = ime.press(Key::Digit(2));
    assert_eq!(done.commit.as_deref(), Some("晶月"));
    assert!(done.composition.input.is_empty());

    let export = ime.user.export();
    let rows: Vec<Vec<&str>> = export
        .lines()
        .filter(|l| l.starts_with("entry\tquick\t"))
        .map(|l| l.split('\t').skip(1).take(4).collect())
        .collect();
    assert_eq!(
        rows,
        [
            ["quick", "晶", "a", "0"],
            ["quick", "晶月", "ab", "1"],
            ["quick", "月", "b", "0"],
        ],
        "{export}"
    );
}

/// 插入的候選經 `select` 選（點候選列）也一樣。
#[test]
fn an_inserted_path_can_be_selected_by_position() {
    let mut ime = Ime::new();
    ime.type_code("ab");
    ime.rerank(&["旭月"]);
    let done = ime.session.select(&ime.engine, &mut ime.user, 1);
    assert_eq!(done.commit.as_deref(), Some("旭月"));
}

/// 翻到後面的頁時先不套用：插入會把第一頁最後一個擠到下一頁，看得到的那頁會變（M1 票 23）。
/// 回到第一頁時才套用，算一次候選變動。
#[test]
fn a_result_arriving_on_a_later_page_waits_for_the_first_page() {
    let mut ime = Ime::new();
    ime.type_code("ab");
    let request = ime.session.rerank_request(&ime.engine).unwrap();
    let turned = ime.press(Key::PageDown);
    let page = texts(&turned);
    let result = ime
        .session
        .apply_rerank(&ime.engine, request.generation, &[3])
        .unwrap();
    assert_eq!(result.page, 1);
    assert_eq!(texts(&result), page);
    assert_eq!(result.generation, request.generation);

    let back = ime.press(Key::PageUp);
    assert_eq!(texts(&back)[..3], ["日月", "晶月", "曰月"]);
    assert_ne!(back.generation, request.generation);
}

/// 等著的結果在候選變了之後作廢：回到第一頁時不套用到新的候選列。
#[test]
fn a_waiting_result_is_dropped_when_the_candidates_change() {
    let mut ime = Ime::new();
    ime.type_code("ab");
    let request = ime.session.rerank_request(&ime.engine).unwrap();
    ime.press(Key::PageDown);
    ime.session
        .apply_rerank(&ime.engine, request.generation, &[3])
        .unwrap();
    ime.press(Key::Backspace);
    ime.press(Key::Letter('b'));
    ime.press(Key::PageDown);
    assert_eq!(texts(&ime.press(Key::PageUp)), AB);
}

/// 選了開頭一段後，路徑只蓋剩下的碼；選插入的路徑時接在選定的分段後面上屏，整串分兩次選定，
/// 照樣自動造詞（plan §2.4）。
#[test]
fn an_inserted_path_after_a_selected_beginning_commits_with_it() {
    let mut ime = Ime::new();
    ime.type_code("bab");
    let at = ime.all().iter().position(|t| t == "月").unwrap();
    ime.session.select(&ime.engine, &mut ime.user, at);
    let reranked = ime.rerank(&["晶月"]);
    assert_eq!(texts(&reranked)[1], "晶月");
    let done = ime.press(Key::Digit(2));
    assert_eq!(done.commit.as_deref(), Some("月晶月"));
    assert!(
        ime.user
            .export()
            .lines()
            .any(|l| l.starts_with("entry\tquick\t月晶月\tbab\t1\t")),
        "{}",
        ime.user.export()
    );
}

#[test]
fn there_is_nothing_to_rerank_without_a_composition() {
    let mut ime = Ime::new();
    assert_eq!(ime.session.rerank_request(&ime.engine), None);
    ime.type_code("ab");
    ime.press(Key::Space);
    assert_eq!(ime.session.rerank_request(&ime.engine), None);
}

/// 臨時英文沒有候選，不給要求。
#[test]
fn inline_english_has_nothing_to_rerank() {
    let mut ime = Ime::new();
    ime.type_code("ab");
    ime.press(Key::ToggleAscii(ShiftKey::Left));
    assert_eq!(ime.session.rerank_request(&ime.engine), None);
}

/// 移到某一段改字時候選是那一段單打的字詞，整句插不進去：不給要求（使用者決定），也不套用。
#[test]
fn a_focused_segment_is_not_reranked() {
    let mut ime = Ime::new();
    ime.type_code("ab");
    let request = ime.session.rerank_request(&ime.engine).unwrap();
    let focused = ime.press(Key::Left);
    assert_eq!(focused.composition.focus, 1);
    assert_eq!(ime.session.rerank_request(&ime.engine), None);
    assert_eq!(
        ime.session
            .apply_rerank(&ime.engine, request.generation, &[3]),
        None
    );
}

/// 前文是游標前的字接上選定的分段（路徑前面的字），只留最後 [`RERANK_CONTEXT`] 字（brief §8）。
#[test]
fn the_context_is_the_text_before_the_cursor_and_the_selected_segments() {
    let mut ime = Ime::new();
    ime.session.set_context("今天");
    ime.type_code("ba");
    assert_eq!(
        ime.session.rerank_request(&ime.engine).unwrap().context,
        "今天"
    );
    // `ba` 的候選是「月日」「月曰」「月昌」「月」：選只蓋住 `b` 的「月」，剩下 `a`。
    let at = ime.all().iter().position(|t| t == "月").unwrap();
    ime.session.select(&ime.engine, &mut ime.user, at);
    let request = ime.session.rerank_request(&ime.engine).unwrap();
    assert_eq!(request.context, "今天月");
    assert_eq!(request.paths[0].text, "日");

    ime.press(Key::Escape);
    let long = format!("開{}", "天".repeat(RERANK_CONTEXT));
    ime.session.set_context(&long);
    ime.type_code("ab");
    let context = ime.session.rerank_request(&ime.engine).unwrap().context;
    assert_eq!(context.chars().count(), RERANK_CONTEXT);
    assert!(!context.contains('開'));
}

/// 不知道前文時只有選定的分段。
#[test]
fn without_a_known_context_only_the_selected_segments_are_sent() {
    let mut ime = Ime::new();
    ime.type_code("ba");
    assert_eq!(ime.session.rerank_request(&ime.engine).unwrap().context, "");
}

/// 規則 4 的英文候選在第 2 位：重排的整句排在它後面（使用者決定：規則 3、4、5 優先）。
#[test]
fn the_english_candidate_keeps_second_and_the_reranked_path_goes_after_it() {
    let mut ime = Ime::new();
    let shown = texts(&ime.type_code("abba"));
    assert_eq!(shown[1], "abba");
    let path = ime.session.rerank_request(&ime.engine).unwrap().paths[5]
        .text
        .clone();
    let reranked = ime.rerank(&[&path]);
    assert_eq!(texts(&reranked)[..3], [shown[0].as_str(), "abba", &path]);
}

/// 規則 5 的記住的字串排在英文後面，重排的整句再排在它們後面。
#[test]
fn remembered_strings_keep_their_places_and_the_reranked_path_goes_after_them() {
    let mut ime = Ime::new();
    ime.press(Key::ToggleAscii(ShiftKey::Right));
    for c in "abba@example.com".chars() {
        let key = if c.is_ascii_alphabetic() {
            Key::Letter(c)
        } else {
            Key::Punct(c)
        };
        ime.press(key);
    }
    ime.press(Key::Space);
    ime.press(Key::ToggleAscii(ShiftKey::Right));

    let shown = texts(&ime.type_code("abba"));
    assert_eq!(shown[1..3], ["abba", "abba@example.com"]);
    let path = ime.session.rerank_request(&ime.engine).unwrap().paths[5]
        .text
        .clone();
    let reranked = ime.rerank(&[&path]);
    assert_eq!(
        texts(&reranked)[..4],
        [shown[0].as_str(), "abba", "abba@example.com", &path]
    );
}

/// 規則 3 排在第 2 位的使用者詞（這裡是自動造詞）不讓位，重排的整句排在它後面。
#[test]
fn a_used_word_keeps_second_and_the_reranked_path_goes_after_it() {
    let mut ime = Ime::new();
    // 分兩次選「昌」「月」：自動造詞 ab → 昌月，上屏 1 次。
    ime.type_code("ab");
    let at = ime.all().iter().position(|t| t == "昌").unwrap();
    ime.session.select(&ime.engine, &mut ime.user, at);
    assert_eq!(ime.press(Key::Space).commit.as_deref(), Some("昌月"));

    let shown = texts(&ime.type_code("ab"));
    assert_eq!(shown[..2], ["日月", "昌月"]);
    let reranked = ime.rerank(&["晶月"]);
    assert_eq!(texts(&reranked)[..3], ["日月", "昌月", "晶月"]);
}

/// 規則排定的候選不在第 2 位時（規則沒有作用），重排照樣到第 2 位。
#[test]
fn without_a_rule_on_second_the_path_goes_second() {
    let mut ime = Ime::new();
    assert_eq!(ime.choose("ab", "曰月").as_deref(), Some("曰月"));
    ime.type_code("ab");
    let reranked = ime.rerank(&["旨月"]);
    assert_eq!(texts(&reranked)[1], "旨月");
}

/// 前 n 條整串路徑（M3 票 07，模型階梯）：lattice 蓋滿沒選定的輸入碼的部分路徑，依分數由高到低、
/// 附分數；最多 beam 條。和候選列不同，多段路徑不受方案的 `sentences` 限制。
#[test]
fn paths_list_the_best_whole_paths_with_scores() {
    let mut ime = Ime::new();
    assert!(ime.session.paths(5).is_empty(), "沒有組字");
    ime.type_code("ab");
    let paths = ime.session.paths(3);
    let texts: Vec<&str> = paths.iter().map(|p| p.text.as_str()).collect();
    assert_eq!(texts, ["日月", "曰月", "昌月"]);
    assert!(paths.windows(2).all(|w| w[0].score >= w[1].score));
    // `a` 的 10 個字各接「月」：全部 10 條，候選列只有方案的 3 條句子。
    assert_eq!(ime.session.paths(100).len(), 10);
    let whole = ime.all().iter().filter(|c| c.chars().count() == 2).count();
    assert_eq!(whole, usize::from(Ranking::default().sentences));
}

/// 選了開頭一段之後，路徑只蓋剩下的輸入碼。
#[test]
fn paths_cover_only_the_input_left_after_a_selected_beginning() {
    let mut ime = Ime::new();
    ime.type_code("ab");
    let at = ime.all().iter().position(|t| t == "曰").unwrap();
    ime.session.select(&ime.engine, &mut ime.user, at);
    let texts: Vec<String> = ime.session.paths(5).into_iter().map(|p| p.text).collect();
    assert_eq!(texts, ["月"]);
}

/// 簡體模式比的是顯示的字：轉換後和首選相同的路徑跳過，和候選列裡的相同就不再插入。
#[test]
fn simplified_mode_compares_the_shown_text() {
    let mut ime = Ime::simplified();
    let shown = texts(&ime.type_code("ab"));
    assert_eq!(shown[..2], ["日月", "昌月"]);
    let reranked = ime.rerank(&["曰月", "旦月"]);
    assert_eq!(texts(&reranked)[..3], ["日月", "旦月", "昌月"]);

    ime.press(Key::Escape);
    let shown = texts(&ime.type_code("ab"));
    let unchanged = ime.rerank(&["晶月"]);
    assert_eq!(
        texts(&unchanged),
        shown,
        "「晶月」顯示成「昌月」，已在第 2 位"
    );
}
