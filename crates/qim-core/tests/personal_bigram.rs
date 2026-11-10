//! 個人 bigram 與學習總開關（plan §2.3、§2.4，M3 票 12）：上屏時記「前一個字詞 → 這一段」，
//! 之後同樣的前一個字詞後面這一段加分。前一個字詞是同一次組字的前一段，或前文：前文以上一次
//! 上屏的字詞結尾時是那個字詞，否則是前文的最後一字。只看文字、跨方案共用。

mod common;

use std::collections::{BTreeMap, BTreeSet};

use qim_core::{
    CodeEntry, CodeTable, Engine, Key, Lexicon, LexiconPack, Options, Packs, Ranking, Scheme,
    SchemePack, Session, Snapshot, UserModel, Word,
};

/// `ab` 是「甲」「乙」「丙」，甲的字頻遠高、乙丙相近；`c`「丁」、`d`「天」、`e`「日」給前文用；
/// `cd` 是詞「丁天」。
fn engine(scheme: &str) -> Engine {
    let table = [("ab", "甲乙丙"), ("c", "丁"), ("d", "天"), ("e", "日")];
    let chars = [
        ('甲', 1000),
        ('乙', 10),
        ('丙', 9),
        ('丁', 50),
        ('天', 50),
        ('日', 50),
    ];
    let code_table = CodeTable::from_entries(table.iter().map(|(code, chars)| {
        let entries = chars
            .chars()
            .map(|text| CodeEntry {
                text,
                extension: false,
                simplified: false,
            })
            .collect();
        (code.to_string(), entries)
    }));
    let words = BTreeMap::from([(
        "cd".to_string(),
        vec![Word {
            text: "丁天".into(),
            frequency: 100,
        }],
    )]);
    let lexicon = LexiconPack {
        id: "lexicon".into(),
        lexicon: Lexicon::new(words, chars),
    };
    let continuous: BTreeSet<char> = table.iter().flat_map(|(_, c)| c.chars()).collect();
    let model = common::unigram("model", &chars, continuous.len());
    let scheme = Scheme {
        id: scheme.into(),
        name: scheme.into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        radicals: BTreeMap::new(),
        packs: Packs {
            lexicon: Some("lexicon".into()),
            language_model: Some("model".into()),
            ..Packs::default()
        },
        ranking: Ranking::default(),
    };
    Engine::new(
        SchemePack { scheme, code_table },
        Some(lexicon),
        Some(model),
    )
    .unwrap()
}

fn type_code(engine: &Engine, session: &mut Session, user: &mut UserModel, code: &str) -> Snapshot {
    let mut snapshot = Snapshot::default();
    for c in code.chars() {
        snapshot = session.handle(engine, user, Key::Letter(c));
    }
    snapshot
}

/// 依序在當下的候選列選文字是 `picks` 各項的候選。
fn pick(
    engine: &Engine,
    session: &mut Session,
    user: &mut UserModel,
    mut shown: Snapshot,
    picks: &[&str],
) -> Snapshot {
    for text in picks {
        let at = shown.candidates.iter().position(|c| c.text == *text);
        let at = at.unwrap_or_else(|| panic!("這頁沒有「{text}」"));
        shown = session.handle(engine, user, Key::Digit(u8::try_from(at + 1).unwrap()));
    }
    shown
}

/// 新的 `Session` 給前文 `context`、打 `ab`，回「丙」在候選列的位置；清掉組字、不上屏。
fn rank_of_bing_after(engine: &Engine, user: &mut UserModel, context: &str) -> usize {
    let mut session = Session::default();
    session.set_context(context);
    let shown = type_code(engine, &mut session, user, "ab");
    session.handle(engine, user, Key::Escape);
    rank_of_bing(&shown)
}

/// 「丙」在這頁候選的位置。
fn rank_of_bing(shown: &Snapshot) -> usize {
    shown
        .candidates
        .iter()
        .position(|c| c.text == "丙")
        .expect("丙在第一頁")
}

/// 前文「丁」後面選過一次「丙」。
fn learned_ding_then_bing(engine: &Engine) -> UserModel {
    let mut user = UserModel::default();
    let mut session = Session::default();
    session.set_context("丁");
    let shown = type_code(engine, &mut session, &mut user, "ab");
    let committed = pick(engine, &mut session, &mut user, shown, &["丙"]);
    assert_eq!(committed.commit.as_deref(), Some("丙"));
    user
}

#[test]
fn after_choosing_y_after_x_y_ranks_higher_after_x() {
    let engine = engine("quick");
    let mut blank = UserModel::default();
    assert_eq!(rank_of_bing_after(&engine, &mut blank, "丁"), 2);

    let mut user = learned_ding_then_bing(&engine);
    let after_ding = rank_of_bing_after(&engine, &mut user, "丁");
    let after_ri = rank_of_bing_after(&engine, &mut user, "日");
    assert!(after_ding < 2, "前文「丁」後選過丙，丙仍排第 {after_ding}");
    assert!(
        after_ding < after_ri,
        "丙在「丁」後（第 {after_ding}）沒有比在「日」後（第 {after_ri}）前"
    );
}

/// 同一次組字裡選定的前一段也是前一個字詞。
#[test]
fn a_segment_selected_before_y_in_the_same_composition_is_its_previous_word() {
    let engine = engine("quick");
    let mut user = UserModel::default();
    let mut session = Session::default();
    let shown = type_code(&engine, &mut session, &mut user, "cab");
    let committed = pick(&engine, &mut session, &mut user, shown, &["丁", "丙"]);
    assert_eq!(committed.commit.as_deref(), Some("丁丙"));

    assert!(
        rank_of_bing_after(&engine, &mut user, "丁") < rank_of_bing_after(&engine, &mut user, "日")
    );
}

/// 前文以上一次上屏的詞結尾時，前一個字詞是那個詞，不只是它的最後一字。
#[test]
fn context_ending_with_the_last_committed_word_uses_that_word() {
    let engine = engine("quick");
    let mut user = UserModel::default();
    let mut session = Session::default();
    session.set_context("");
    type_code(&engine, &mut session, &mut user, "cd");
    let committed = session.handle(&engine, &mut user, Key::Space);
    assert_eq!(committed.commit.as_deref(), Some("丁天"));
    session.set_context("丁天");
    let shown = type_code(&engine, &mut session, &mut user, "ab");
    pick(&engine, &mut session, &mut user, shown, &["丙"]);

    // 前一個字詞是「丁天」：前文只有「天」的新 `Session` 接不上，同一個 `Session` 上屏「丁天」後接得上。
    let after_tian = rank_of_bing_after(&engine, &mut user, "天");
    session.set_context("");
    type_code(&engine, &mut session, &mut user, "cd");
    session.handle(&engine, &mut user, Key::Space);
    session.set_context("今日丁天");
    let shown = type_code(&engine, &mut session, &mut user, "ab");
    let after_ding_tian = rank_of_bing(&shown);
    assert!(
        after_ding_tian < after_tian,
        "接在「丁天」後第 {after_ding_tian}，接在「天」後第 {after_tian}"
    );
}

/// 個人 bigram 只看文字：在一個方案學到的，另一個方案同樣的字也用得到（plan §2.4）。
#[test]
fn a_bigram_learned_in_one_scheme_applies_in_another() {
    let quick = engine("quick");
    let cangjie = engine("cangjie");
    let mut user = learned_ding_then_bing(&quick);
    let mut blank = UserModel::default();
    assert!(
        rank_of_bing_after(&cangjie, &mut user, "丁")
            < rank_of_bing_after(&cangjie, &mut blank, "丁")
    );
}

/// 無痕模式不用前文（brief §8），個人 bigram 也不接前文。
#[test]
fn incognito_does_not_use_the_context_for_the_personal_bigram() {
    let engine = engine("quick");
    let mut user = learned_ding_then_bing(&engine);
    let mut session = Session::default();
    session.set_options(Options {
        incognito: true,
        ..Options::default()
    });
    session.set_context("丁");
    let shown = type_code(&engine, &mut session, &mut user, "ab");
    let rank = rank_of_bing(&shown);
    assert_eq!(
        rank,
        rank_of_bing_after(&engine, &mut UserModel::default(), "丁")
    );
}

/// 學習總開關（brief §4.5、§7）：關掉時不寫任何學習資料，已學的照用；和無痕不同，前文照用。
#[test]
fn with_learning_off_nothing_is_written_and_what_was_learned_still_applies() {
    let engine = engine("quick");
    let mut user = learned_ding_then_bing(&engine);
    let learned = user.clone();
    let on_rank = rank_of_bing_after(&engine, &mut user, "丁");

    let mut session = Session::default();
    session.set_options(Options {
        learning: false,
        ..Options::default()
    });
    session.set_context("丁");
    let shown = type_code(&engine, &mut session, &mut user, "ab");
    let off_rank = rank_of_bing(&shown);
    assert_eq!(off_rank, on_rank, "學習關掉後已學的不再生效");
    pick(&engine, &mut session, &mut user, shown, &["乙"]);
    // 分段選字（會自動造詞的那種）也不寫。
    session.set_context("日");
    let shown = type_code(&engine, &mut session, &mut user, "cab");
    pick(&engine, &mut session, &mut user, shown, &["丁", "乙"]);
    for _ in 0..2 {
        let shown = type_code(&engine, &mut session, &mut user, "ab");
        pick(&engine, &mut session, &mut user, shown, &["乙"]);
    }
    assert_eq!(user, learned);
    assert_eq!(
        engine.candidates(&user, "ab")[0],
        "甲",
        "學習關掉時選兩次不成首選"
    );
}

#[test]
fn learning_is_on_by_default() {
    assert!(Options::default().learning);
}

/// 組字中給的前文由下一次組字用（plan §2.1）：學到的前一個字詞是組字開頭時的前文，和打分時
/// 用的一樣。
#[test]
fn context_set_while_composing_does_not_change_what_this_composition_learns() {
    let engine = engine("quick");
    let mut user = UserModel::default();
    let mut session = Session::default();
    session.set_context("丁");
    let shown = type_code(&engine, &mut session, &mut user, "ab");
    session.set_context("日");
    pick(&engine, &mut session, &mut user, shown, &["丙"]);

    let mut blank = UserModel::default();
    assert!(
        rank_of_bing_after(&engine, &mut user, "丁")
            < rank_of_bing_after(&engine, &mut blank, "丁")
    );
    assert_eq!(
        rank_of_bing_after(&engine, &mut user, "日"),
        rank_of_bing_after(&engine, &mut blank, "日"),
        "學到的是「日 → 丙」"
    );
}
