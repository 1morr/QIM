//! 常用語（plan §2.4）：使用者存的句子或段落，打觸發詞時是首選，也可以從清單選了上屏。

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, Engine, Key, Packs, Phrase, Ranking, Scheme, SchemePack, Session,
    Snapshot, UserModel,
};

/// `m`「一」、`ir`「台」、`r`「口」，沒有詞庫與語言模型。
fn engine() -> Engine {
    let scheme = Scheme {
        id: "quick".into(),
        name: "quick".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        wildcard: None,
        radicals: BTreeMap::new(),
        packs: Packs::default(),
        ranking: Ranking::default(),
    };
    let code_table = CodeTable::from_entries([("m", '一'), ("ir", '台'), ("r", '口')].map(
        |(code, text)| {
            let entry = CodeEntry {
                text,
                extension: false,
                simplified: false,
            };
            (code.to_string(), vec![entry])
        },
    ));
    Engine::new(SchemePack { scheme, code_table }, None, None).unwrap()
}

fn type_code(engine: &Engine, user: &mut UserModel, session: &mut Session, code: &str) -> Snapshot {
    let mut snapshot = Snapshot::default();
    for c in code.chars() {
        snapshot = session.handle(engine, user, Key::Letter(c));
    }
    snapshot
}

fn texts(snapshot: &Snapshot) -> Vec<&str> {
    snapshot
        .candidates
        .iter()
        .map(|c| c.text.as_str())
        .collect()
}

const ADDRESS: &str = "九龍彌敦道 1 號\n3 樓 A 室";

#[test]
fn a_phrase_with_a_line_break_is_the_top_for_its_trigger_and_commits_whole() {
    let engine = engine();
    let mut user = UserModel::default();
    let mut session = Session::default();
    user.add_phrase(&engine, ADDRESS, Some("mir")).unwrap();

    let shown = type_code(&engine, &mut user, &mut session, "mir");
    assert_eq!(texts(&shown).first(), Some(&ADDRESS), "{:?}", texts(&shown));
    let committed = session.handle(&engine, &mut user, Key::Space);
    assert_eq!(committed.commit.as_deref(), Some(ADDRESS));
}

#[test]
fn phrases_are_listed_by_text_with_their_triggers_and_a_removed_one_is_gone_everywhere() {
    let engine = engine();
    let mut user = UserModel::default();
    let mut session = Session::default();
    user.add_phrase(&engine, ADDRESS, Some("mir")).unwrap();
    user.add_phrase(&engine, "多謝晒！", None).unwrap();
    assert_eq!(
        user.phrases(&engine),
        [
            Phrase {
                text: "九龍彌敦道 1 號\n3 樓 A 室".into(),
                trigger: Some("mir".into()),
            },
            Phrase {
                text: "多謝晒！".into(),
                trigger: None,
            },
        ]
    );
    // 用過一次，學習計數也記了它。
    type_code(&engine, &mut user, &mut session, "mir");
    session.handle(&engine, &mut user, Key::Space);

    assert!(user.remove_phrase(&engine, ADDRESS));
    assert!(!user.remove_phrase(&engine, ADDRESS), "已經刪了");
    let shown = type_code(&engine, &mut user, &mut session, "mir");
    assert!(!texts(&shown).contains(&ADDRESS), "{:?}", texts(&shown));
    assert!(!user.export().contains("彌敦道"), "{}", user.export());
    assert_eq!(user.phrases(&engine).len(), 1);
}

#[test]
fn saving_a_phrase_again_replaces_its_trigger() {
    let engine = engine();
    let mut user = UserModel::default();
    let mut session = Session::default();
    user.add_phrase(&engine, ADDRESS, Some("mir")).unwrap();
    type_code(&engine, &mut user, &mut session, "mir");
    session.handle(&engine, &mut user, Key::Space);

    user.add_phrase(&engine, ADDRESS, Some("ir")).unwrap();
    let old = type_code(&engine, &mut user, &mut session, "mir");
    session.handle(&engine, &mut user, Key::Escape);
    assert!(!texts(&old).contains(&ADDRESS), "{:?}", texts(&old));
    assert!(
        !user.export().contains("\tmir\t"),
        "舊觸發詞學到的計數要刪掉：{}",
        user.export()
    );
    let new = type_code(&engine, &mut user, &mut session, "ir");
    assert_eq!(texts(&new).first(), Some(&ADDRESS));
}

/// 常用語不是由字的碼打出來的，不進連打：觸發詞接著打別的碼時不出現。
#[test]
fn a_phrase_is_not_part_of_continuous_typing() {
    let engine = engine();
    let mut user = UserModel::default();
    let mut session = Session::default();
    user.add_phrase(&engine, "台", Some("r")).unwrap();
    let shown = type_code(&engine, &mut user, &mut session, "mr");
    assert!(!texts(&shown).contains(&"一台"), "{:?}", texts(&shown));
}

/// 從清單選常用語走觸控鍵盤的 `text`：組字中先上屏組字再接它，文字照原樣。
#[test]
fn a_phrase_picked_from_the_list_commits_after_the_composition() {
    let engine = engine();
    let mut user = UserModel::default();
    let mut session = Session::default();
    type_code(&engine, &mut user, &mut session, "m");
    let committed = session.text(&engine, &mut user, ADDRESS);
    assert_eq!(committed.commit, Some(format!("一{ADDRESS}")));
}

#[test]
fn a_phrase_is_not_empty_has_no_other_control_characters_and_its_trigger_uses_scheme_keys() {
    let engine = engine();
    let mut user = UserModel::default();
    assert!(user.add_phrase(&engine, "", None).is_err());
    assert!(user.add_phrase(&engine, "一\r台", None).is_err());
    assert!(user.add_phrase(&engine, "一台", Some("z")).is_err());
    assert!(user.add_phrase(&engine, "一台", Some("")).is_err());
    assert_eq!(user, UserModel::default());
    user.add_phrase(&engine, "一\t台", None).unwrap();
}

/// 刪掉常用語後方案只剩 tick 行、沒有資料列：匯出再匯入要一模一樣。
#[test]
fn exporting_then_importing_keeps_a_tick_left_after_removing_the_only_phrase() {
    let engine = engine();
    let mut user = UserModel::default();
    let mut session = Session::default();
    user.add_phrase(&engine, ADDRESS, Some("mir")).unwrap();
    type_code(&engine, &mut user, &mut session, "mir");
    session.handle(&engine, &mut user, Key::Space);
    assert!(user.remove_phrase(&engine, ADDRESS));

    let exported = user.export();
    assert_eq!(
        UserModel::import(&exported, None).map(|i| i.user),
        Ok(user),
        "{exported}"
    );
}

/// 空模型上刪不到常用語，不該留下空的方案：模型不變，匯出再匯入也一樣。
#[test]
fn removing_a_missing_phrase_from_an_empty_model_changes_nothing() {
    let engine = engine();
    let mut user = UserModel::default();
    assert!(!user.remove_phrase(&engine, ADDRESS));
    assert_eq!(user, UserModel::default());
    assert_eq!(
        UserModel::import(&user.export(), None).map(|i| i.user),
        Ok(user)
    );
}
