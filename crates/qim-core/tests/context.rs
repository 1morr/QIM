//! 上下文（plan §2.1、§2.3）：組字開頭的字接前一個字的 bigram。前一個字是選定的分段的最後一字，
//! 沒有選定的分段時看 `Session::set_context` 給的前文：以字結尾時是那個字，以標點、空白結尾
//! 或是空的時是句首（[`CLAUSE_START`]）；沒給前文與無痕（brief §8）時照舊用 unigram。

use std::collections::BTreeMap;

use qim_core::{
    CLAUSE_START, CharModel, CodeEntry, CodeTable, ConversionTable, Engine, Key, LanguageModel,
    LanguageModelPack, Options, Packs, Ranking, Scheme, SchemePack, Session, Simplifier,
    SimplifierPack, UserModel,
};

/// `a` 是「甲」「乙」「丁」，單打時甲的字頻高；`b` 是「天」，「天乙」是常見的接續，
/// 丁多半當句首。簡體模式用的：`c`「後」（简「后」）後面接乙；`d`「乾」、`e`「幹」都簡化成「干」，
/// 「乾」的 unigram 高、後面接甲，「幹」後面接乙。
fn engine() -> Engine {
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        radicals: BTreeMap::new(),
        packs: Packs {
            language_model: Some("bigram".into()),
            ..Packs::default()
        },
        ranking: Ranking::default(),
    };
    let entry = |text| CodeEntry {
        text,
        extension: false,
        simplified: false,
    };
    let code_table = CodeTable::from_entries([
        ("a".to_string(), vec![entry('甲'), entry('乙'), entry('丁')]),
        ("b".to_string(), vec![entry('天')]),
        ("c".to_string(), vec![entry('後')]),
        ("d".to_string(), vec![entry('乾')]),
        ("e".to_string(), vec![entry('幹')]),
    ]);
    // 接續機率和 unigram 反過來：前一字沒有統計時誤用退回式，首選就成了乙。
    let model = |start: f64, continuation: f64| CharModel {
        start: start.ln(),
        continuation: continuation.ln(),
        backoff: 0.0,
    };
    let language_model = LanguageModel::new(
        [
            ('甲', model(0.5, 0.1)),
            ('乙', model(0.1, 0.5)),
            ('天', model(0.4, 0.4)),
            ('丁', model(0.05, 0.05)),
            ('後', model(0.1, 0.1)),
            ('乾', model(0.1, 0.1)),
            ('幹', model(0.05, 0.05)),
            (CLAUSE_START, model(1e-6, 1e-6)),
        ],
        model(1e-6, 1e-6),
        [
            ('天', '乙', 0.9f64.ln()),
            ('天', '甲', 0.01f64.ln()),
            (CLAUSE_START, '丁', 0.9f64.ln()),
            ('後', '乙', 0.9f64.ln()),
            ('乾', '甲', 0.9f64.ln()),
            ('幹', '乙', 0.9f64.ln()),
        ],
    );
    Engine::new(
        SchemePack { scheme, code_table },
        None,
        Some(LanguageModelPack {
            id: "bigram".into(),
            model: language_model,
        }),
    )
    .unwrap()
}

/// 給前文 `before`（`None` 是不呼叫 `set_context`）、照 `options` 打 `input`，回首選。
fn top_after(before: Option<&str>, options: Options, input: &str) -> String {
    let engine = engine();
    let mut user = UserModel::default();
    let mut session = Session::default();
    session.set_options(options);
    if let Some(before) = before {
        session.set_context(before);
    }
    let mut snapshot = None;
    for c in input.chars() {
        snapshot = Some(session.handle(&engine, &mut user, Key::Letter(c)));
    }
    snapshot.unwrap().candidates[0].text.clone()
}

fn top(before: Option<&str>, input: &str) -> String {
    top_after(before, Options::default(), input)
}

#[test]
fn the_same_code_tops_differently_after_different_context() {
    assert_eq!(top(None, "a"), "甲");
    assert_eq!(top(Some("今天"), "a"), "乙");
}

/// bigram 只在句內訓練，接不過標點（跨標點接上一句的最後一字，「有前文」的基準變差）；
/// 標點與空白之後是一句的開頭。空的前文是輸入框的開頭，也是句首。
#[test]
fn context_ending_in_punctuation_or_whitespace_starts_a_clause() {
    assert_eq!(top(Some("今天，"), "a"), "丁");
    assert_eq!(top(Some("今天。 \n"), "a"), "丁");
    assert_eq!(top(Some("OK "), "a"), "丁");
    assert_eq!(top(Some(""), "a"), "丁");
}

/// 英文、數字與語言模型沒有統計的字接不上 bigram：照沒有前文打。
#[test]
fn context_ending_in_a_character_without_statistics_is_like_no_context() {
    assert_eq!(top(Some("天 OK"), "a"), "甲");
    assert_eq!(top(Some("天 3"), "a"), "甲");
    assert_eq!(top(Some("天丙"), "a"), "甲");
}

#[test]
fn incognito_ignores_the_context() {
    let incognito = Options {
        incognito: true,
        ..Options::default()
    };
    assert_eq!(top_after(Some("今天"), incognito, "a"), "甲");
}

/// 選了開頭一段後，剩下輸入碼的第一個字接選定分段的最後一字（plan §2.3），不再用 unigram。
#[test]
fn after_selecting_a_leading_segment_the_rest_follows_its_last_character() {
    let engine = engine();
    let mut user = UserModel::default();
    let mut session = Session::default();
    session.handle(&engine, &mut user, Key::Letter('b'));
    session.handle(&engine, &mut user, Key::Letter('a'));
    let all = session.all_candidates();
    let leading = all.iter().position(|c| c.text == "天").unwrap();
    let snapshot = session.select(&engine, &mut user, leading);
    assert_eq!(snapshot.composition.selected_text(), "天");
    assert_eq!(snapshot.candidates[0].text, "乙");
}

/// 組字中給的前文由下一次組字用。
#[test]
fn context_set_while_composing_is_used_by_the_next_composition() {
    let engine = engine();
    let mut user = UserModel::default();
    let mut session = Session::default();
    session.handle(&engine, &mut user, Key::Letter('a'));
    session.set_context("天");
    let snapshot = session.handle(&engine, &mut user, Key::Escape);
    assert!(snapshot.candidates.is_empty());
    let snapshot = session.handle(&engine, &mut user, Key::Letter('a'));
    assert_eq!(snapshot.candidates[0].text, "乙");
}

/// 殼沒給前文時，核心自己上屏的字就是前文（同 librime 的 commit history，plan §2.1）。
#[test]
fn without_context_from_the_host_the_next_composition_follows_the_last_commit() {
    let engine = engine();
    let mut user = UserModel::default();
    let mut session = Session::default();
    session.handle(&engine, &mut user, Key::Letter('b'));
    let snapshot = session.handle(&engine, &mut user, Key::Space);
    assert_eq!(snapshot.commit.as_deref(), Some("天"));
    let snapshot = session.handle(&engine, &mut user, Key::Letter('a'));
    assert_eq!(snapshot.candidates[0].text, "乙");
}

/// 交還宿主的可見字元與空白接在前文後面：「天」之後打了空白，下一個字是句首。
#[test]
fn a_space_passed_to_the_host_extends_the_context() {
    let engine = engine();
    let mut user = UserModel::default();
    let mut session = Session::default();
    session.set_context("天");
    assert!(!session.handle(&engine, &mut user, Key::Space).handled);
    let snapshot = session.handle(&engine, &mut user, Key::Letter('a'));
    assert_eq!(snapshot.candidates[0].text, "丁");
}

/// 交還宿主的退格、Enter、方向鍵改了游標前的字或移走游標：前文不知道了，照沒有前文打，
/// 等殼再給。
#[test]
fn keys_passed_to_the_host_that_edit_or_move_forget_the_context() {
    let engine = engine();
    for key in [Key::Backspace, Key::Enter, Key::Left, Key::Other] {
        let mut user = UserModel::default();
        let mut session = Session::default();
        session.set_context("今天");
        assert!(!session.handle(&engine, &mut user, key.clone()).handled);
        let snapshot = session.handle(&engine, &mut user, Key::Letter('a'));
        assert_eq!(snapshot.candidates[0].text, "甲", "{key:?} 後");
    }
}

/// 不送字元的修飾鍵沒改輸入框，前文照舊。
#[test]
fn a_modifier_key_keeps_the_context() {
    let engine = engine();
    let mut user = UserModel::default();
    let mut session = Session::default();
    session.set_context("今天");
    session.handle(&engine, &mut user, Key::Modifier);
    let snapshot = session.handle(&engine, &mut user, Key::Letter('a'));
    assert_eq!(snapshot.candidates[0].text, "乙");
}

/// 無痕打開時丟掉殼給過的前文（brief §8：不讀上下文），關掉後也不會回來。
#[test]
fn turning_incognito_on_forgets_the_context() {
    let engine = engine();
    let mut user = UserModel::default();
    let mut session = Session::default();
    session.set_context("今天");
    let incognito = Options {
        incognito: true,
        ..Options::default()
    };
    session.set_options(incognito);
    session.set_options(Options::default());
    let snapshot = session.handle(&engine, &mut user, Key::Letter('a'));
    assert_eq!(snapshot.candidates[0].text, "甲");
}

fn incognito() -> Options {
    Options {
        incognito: true,
        ..Options::default()
    }
}

/// 無痕期間上屏的字不接進前文（brief §8：不讀上下文、不送雲端）：關掉無痕後、殼還沒給前文時，
/// 組字開頭的字照不知道前文打，不接無痕時上屏的「天」。
#[test]
fn commits_made_in_incognito_never_become_context() {
    let engine = engine();
    let mut user = UserModel::default();
    let mut session = Session::default();
    session.set_options(incognito());
    session.handle(&engine, &mut user, Key::Letter('b'));
    assert_eq!(
        session
            .handle(&engine, &mut user, Key::Space)
            .commit
            .as_deref(),
        Some("天")
    );
    session.set_options(Options::default());
    let snapshot = session.handle(&engine, &mut user, Key::Letter('a'));
    assert_eq!(snapshot.candidates[0].text, "甲");
}

/// 無痕時殼給的前文也不收，關掉後不會冒出來。
#[test]
fn context_given_in_incognito_is_ignored() {
    let engine = engine();
    let mut user = UserModel::default();
    let mut session = Session::default();
    session.set_options(incognito());
    session.set_context("今天");
    session.set_options(Options::default());
    let snapshot = session.handle(&engine, &mut user, Key::Letter('a'));
    assert_eq!(snapshot.candidates[0].text, "甲");
}

/// 關掉無痕之後上屏的字照常是前文（不含無痕期間的）。
#[test]
fn commits_after_leaving_incognito_are_context_again() {
    let engine = engine();
    let mut user = UserModel::default();
    let mut session = Session::default();
    session.set_options(incognito());
    session.set_options(Options::default());
    session.handle(&engine, &mut user, Key::Letter('b'));
    session.handle(&engine, &mut user, Key::Space);
    let snapshot = session.handle(&engine, &mut user, Key::Letter('a'));
    assert_eq!(snapshot.candidates[0].text, "乙");
}

/// 換欄位或失焦時殼呼叫 `clear_context`：上一個欄位上屏的字不再是前文。殼給新的前文時整份取代，
/// 自己記的也不留。
#[test]
fn clearing_or_replacing_the_context_drops_the_commits_recorded_so_far() {
    let engine = engine();
    let commit_heaven = |session: &mut Session, user: &mut UserModel| {
        session.handle(&engine, user, Key::Letter('b'));
        session.handle(&engine, user, Key::Space);
    };
    let mut user = UserModel::default();
    let mut session = Session::default();
    commit_heaven(&mut session, &mut user);
    session.clear_context(&mut user);
    let snapshot = session.handle(&engine, &mut user, Key::Letter('a'));
    assert_eq!(snapshot.candidates[0].text, "甲");

    let mut session = Session::default();
    commit_heaven(&mut session, &mut user);
    session.set_context("。");
    let snapshot = session.handle(&engine, &mut user, Key::Letter('a'));
    assert_eq!(snapshot.candidates[0].text, "丁");
}

fn simplifier() -> SimplifierPack {
    let table = ConversionTable::new(
        [("後", "后"), ("乾", "干"), ("幹", "干")]
            .iter()
            .map(|&(k, v)| (k.to_string(), v.to_string())),
    );
    SimplifierPack {
        id: "opencc-hk2s".into(),
        simplifier: Simplifier::new(vec![], ConversionTable::new([]), vec![vec![table]]),
    }
}

fn simplified() -> Options {
    Options {
        simplified: true,
        ..Options::default()
    }
}

/// 掛上簡繁資料包，照 `options` 給前文 `before`、打 `input`，回首選。
fn top_with_simplifier(before: &str, options: Options, input: &str) -> String {
    let engine = engine().with_simplifier(simplifier());
    let mut user = UserModel::default();
    let mut session = Session::default();
    session.set_options(options);
    session.set_context(before);
    let mut snapshot = None;
    for c in input.chars() {
        snapshot = Some(session.handle(&engine, &mut user, Key::Letter(c)));
    }
    snapshot.unwrap().candidates[0].text.clone()
}

/// 簡體模式下殼給的前文是簡體字，語言模型是繁體：最後一字轉回繁體再接 bigram（同停鍵層的
/// 前文，plan §2.7），首選和繁體模式下對應的繁體前文相同。一對多的取 unigram 最高的（干 → 乾）。
#[test]
fn simplified_context_is_turned_back_into_traditional() {
    assert_eq!(top_with_simplifier("後", Options::default(), "a"), "乙");
    assert_eq!(top_with_simplifier("后", simplified(), "a"), "乙");
    assert_eq!(top_with_simplifier("乾", Options::default(), "a"), "甲");
    assert_eq!(top_with_simplifier("干", simplified(), "a"), "甲");
    // 一串碼：開頭的字接前文，整句同樣跟著變。
    assert_eq!(top_with_simplifier("後", Options::default(), "ab"), "乙天");
    assert_eq!(top_with_simplifier("后", simplified(), "ab"), "乙天");
}

/// 繁體模式的前文照原樣：簡體字沒有統計，照沒有前文打。
#[test]
fn traditional_mode_does_not_convert_the_context() {
    assert_eq!(top_with_simplifier("后", Options::default(), "a"), "甲");
}

/// 前文以剛上屏的那一段（簡體模式顯示的字）結尾時，前一字是那段原文的最後一字：上屏「幹」
/// 顯示「干」，接的是「幹」，不是逐字轉回的「乾」。
#[test]
fn context_ending_with_the_last_commit_follows_its_original() {
    let engine = engine().with_simplifier(simplifier());
    let mut user = UserModel::default();
    let mut session = Session::default();
    session.set_options(simplified());
    session.set_context("");
    session.handle(&engine, &mut user, Key::Letter('e'));
    let snapshot = session.handle(&engine, &mut user, Key::Space);
    assert_eq!(snapshot.commit.as_deref(), Some("干"));
    session.set_context("干");
    let snapshot = session.handle(&engine, &mut user, Key::Letter('a'));
    assert_eq!(snapshot.candidates[0].text, "乙");
}
