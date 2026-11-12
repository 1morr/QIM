//! 詞表匯入（M3 票 17，plan §2.4）：把別的輸入法匯出或自己整理的詞表加進使用者詞。收 Rime
//! `.dict.yaml`、Gboard 個人字典的 `dictionary.txt`、一行一詞的純文字；碼照手動加詞由字推出。

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, Engine, Packs, Ranking, Scheme, SchemePack, Session, Shortcuts,
    SkipReason, UserModel, WordListFormat, WordListReport,
};

/// `m`「一」、`ir`「台」、`r`「口」，沒有詞庫與語言模型（同 `user_dict.rs`）。
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

const RIME: &str = include_str!("fixtures/word-list/rime.dict.yaml");
const GBOARD: &str = include_str!("fixtures/word-list/gboard.txt");
const PLAIN: &str = include_str!("fixtures/word-list/plain.txt");

/// 照預設匯入：Gboard 的快捷鍵不設成觸發詞。
fn import(user: &mut UserModel, text: &str) -> WordListReport {
    user.import_word_list(&engine(), text, Shortcuts::Ignore)
        .unwrap()
}

/// 略過的列：（行號, 詞, 原因）。
fn skipped(report: &WordListReport) -> Vec<(usize, &str, &SkipReason)> {
    report
        .skipped
        .iter()
        .map(|s| (s.line, s.text.as_str(), &s.reason))
        .collect()
}

fn typed(user: &UserModel, code: &str) -> Vec<String> {
    engine().candidates(user, code)
}

#[test]
fn a_rime_dictionary_adds_its_words_with_codes_from_the_characters() {
    let mut user = UserModel::default();
    let report = import(&mut user, RIME);

    assert_eq!(report.format, WordListFormat::Rime);
    assert_eq!(report.added, 2);
    // 碼欄是別的方案的（粵拼），不看；碼由字推出：一台 `mir`、台口 `irr`。
    assert!(typed(&user, "mir").contains(&"一台".to_string()));
    assert!(typed(&user, "irr").contains(&"台口".to_string()));
    assert_eq!(
        skipped(&report),
        [(17, "口口\u{7}", &SkipReason::ControlCharacter)]
    );
}

/// 能當觸發詞的快捷鍵：（行號, 詞, 快捷鍵, 會搶走首選的候選）。
fn trigger_shortcuts(report: &WordListReport) -> Vec<(usize, &str, &str, Option<&str>)> {
    report
        .trigger_shortcuts
        .iter()
        .map(|t| {
            (
                t.line,
                t.text.as_str(),
                t.shortcut.as_str(),
                t.displaces.as_deref(),
            )
        })
        .collect()
}

#[test]
fn a_gboard_dictionary_adds_its_words_without_triggers_by_default() {
    let mut user = UserModel::default();
    let report = import(&mut user, GBOARD);

    assert_eq!(report.format, WordListFormat::Gboard);
    assert_eq!(report.added, 4);
    // 快捷鍵是在別的鍵盤設的，預設不當觸發詞：`kk`、`m` 照舊，詞由字的碼打得出。
    assert_ne!(typed(&user, "kk").first().map(String::as_str), Some("口口"));
    assert_eq!(typed(&user, "m").first().map(String::as_str), Some("一"));
    assert!(typed(&user, "rr").contains(&"口口".to_string()));
    assert!(typed(&user, "rir").contains(&"口台".to_string()));
    assert!(typed(&user, "irr").contains(&"台口".to_string()));
    assert!(typed(&user, "mir").contains(&"一台".to_string()));
    // 能當觸發詞的照樣列出，`m` 是「一」的碼，設了會搶走它的首選。
    assert_eq!(
        trigger_shortcuts(&report),
        [(2, "口口", "kk", None), (7, "口台", "m", Some("一"))]
    );
    // 快捷鍵 `IIRC` 不是速成的鍵，勾了也不能當觸發詞。
    assert_eq!(
        report
            .unused_shortcuts
            .iter()
            .map(|w| (w.line, w.text.as_str(), w.shortcut.as_str()))
            .collect::<Vec<_>>(),
        [(4, "台口", "IIRC")]
    );
    assert_eq!(
        skipped(&report),
        [
            (3, "STRAY", &SkipReason::NoCode),
            (
                6,
                "broken-row",
                &SkipReason::Malformed("Gboard 的列要是「快捷鍵<Tab>詞<Tab>語言」")
            ),
        ]
    );
}

#[test]
fn gboard_shortcuts_become_triggers_only_when_asked() {
    let engine = engine();
    let mut user = UserModel::default();
    let report = user
        .import_word_list(&engine, GBOARD, Shortcuts::AsTriggers)
        .unwrap();

    assert_eq!(report.added, 4);
    assert_eq!(typed(&user, "kk").first().map(String::as_str), Some("口口"));
    assert_eq!(typed(&user, "m").first().map(String::as_str), Some("口台"));
    assert!(typed(&user, "rr").contains(&"口口".to_string()));
    assert_eq!(
        trigger_shortcuts(&report),
        [(2, "口口", "kk", None), (7, "口台", "m", Some("一"))]
    );
}

#[test]
fn plain_text_takes_one_word_per_line_and_reports_what_it_skips() {
    let mut user = UserModel::default();
    // 固定檔要是 `\r\n` 換行（`.gitattributes` 設 `-text`，git 不轉成 `\n`）。
    assert!(PLAIN.contains("\r\n"));
    let report = import(&mut user, PLAIN);

    assert_eq!(report.format, WordListFormat::PlainText);
    assert_eq!(report.added, 3);
    // 開頭的 BOM 與 `\r\n` 不算進詞；Tab 之後的欄不看，前後的空白去掉。
    assert!(typed(&user, "mir").contains(&"一台".to_string()));
    assert!(typed(&user, "irrr").contains(&"台口口".to_string()));
    assert!(typed(&user, "irir").contains(&"台台".to_string()));
    assert_eq!(
        skipped(&report),
        [
            (5, "一台", &SkipReason::Duplicate { first_line: 1 }),
            (6, "口", &SkipReason::InLexicon),
            (7, "hello", &SkipReason::NoCode),
        ]
    );
}

#[test]
fn importing_adds_to_the_learning_instead_of_replacing_it() {
    let engine = engine();
    let mut user = UserModel::default();
    // 先學一次「口」、手動加「台口」。
    let mut session = Session::default();
    session.handle(&engine, &mut user, qim_core::Key::Letter('r'));
    session.handle(&engine, &mut user, qim_core::Key::Space);
    user.add_word(&engine, "台口", Some("tk")).unwrap();
    let before = user.export();

    let report = user
        .import_word_list(&engine, PLAIN, Shortcuts::Ignore)
        .unwrap();

    assert_eq!(report.added, 3);
    // 原本的學習資料都還在，加的是手動加的詞（來源 `manual`）。
    let after = user.export();
    for line in before.lines().filter(|l| !l.starts_with('#')) {
        assert!(after.contains(line), "少了：{line}\n{after}");
    }
    assert!(after.contains("entry\tquick\t一台\tmir\t0\t0\t0\t0\t\t\tmanual\t\n"));
    assert_eq!(typed(&user, "tk").first().map(String::as_str), Some("台口"));
}

#[test]
fn a_word_that_is_already_a_user_word_is_skipped_and_keeps_its_trigger() {
    let engine = engine();
    let mut user = UserModel::default();
    user.add_word(&engine, "一台", Some("yt")).unwrap();

    let report = user
        .import_word_list(&engine, "一台\n", Shortcuts::Ignore)
        .unwrap();

    assert_eq!(report.added, 0);
    assert_eq!(
        skipped(&report),
        [(1, "一台", &SkipReason::AlreadyUserWord)]
    );
    assert_eq!(typed(&user, "yt").first().map(String::as_str), Some("一台"));
}

#[test]
fn a_rime_header_may_name_the_text_column_in_a_flow_list() {
    let mut user = UserModel::default();
    let text = "---\nname: x\ncolumns: [code, text]  # 碼在前\n...\nyi tai\t一台\n";
    let report = import(&mut user, text);
    assert_eq!(report.added, 1);
    assert!(typed(&user, "mir").contains(&"一台".to_string()));

    // 列比 text 欄短：報行號。
    let report = import(
        &mut UserModel::default(),
        "---\ncolumns: [code, text]\n...\n只有碼\n",
    );
    assert_eq!(
        skipped(&report),
        [(
            4,
            "只有碼",
            &SkipReason::Malformed("少了 columns 裡的 text 欄")
        )]
    );
}

#[test]
fn a_rime_file_that_cannot_be_read_at_all_is_an_error_with_a_line_number() {
    let engine = engine();
    let mut user = UserModel::default();

    let unclosed = user
        .import_word_list(
            &engine,
            "# Rime dictionary\n---\nname: x\n一台\n",
            Shortcuts::Ignore,
        )
        .unwrap_err();
    assert_eq!(unclosed.line, 2);

    let no_text = user
        .import_word_list(
            &engine,
            "---\nname: x\ncolumns:\n  - code\n  - weight\n...\n一台\n",
            Shortcuts::Ignore,
        )
        .unwrap_err();
    assert_eq!(no_text.line, 3);
    assert_eq!(user, UserModel::default());
}

#[test]
fn checking_on_a_copy_leaves_the_model_untouched() {
    let engine = engine();
    let user = UserModel::default();
    let report = user
        .clone()
        .import_word_list(&engine, PLAIN, Shortcuts::Ignore)
        .unwrap();
    assert_eq!(report.added, 3);
    assert_eq!(user, UserModel::default());
}

#[test]
fn a_duplicate_points_at_a_line_that_was_added() {
    let mut user = UserModel::default();
    // 第 2 行沒有快捷鍵、推不出碼；第 3 行有快捷鍵才加得進去，第 4 行的重複指向它。
    let gboard = "# Gboard Dictionary version:1\n\thello\ten\nhh\thello\ten\nkk\thello\ten\n";
    let report = user
        .import_word_list(&engine(), gboard, Shortcuts::AsTriggers)
        .unwrap();
    assert_eq!(report.added, 1);
    assert_eq!(
        skipped(&report),
        [
            (2, "hello", &SkipReason::NoCode),
            (4, "hello", &SkipReason::Duplicate { first_line: 3 }),
        ]
    );
    assert_eq!(
        typed(&user, "hh").first().map(String::as_str),
        Some("hello")
    );
}

#[test]
fn an_empty_word_and_a_trailing_control_character_are_reported() {
    let mut user = UserModel::default();
    let report = import(&mut user, "\t碼\n台口\u{b}\n");
    assert_eq!(report.added, 0);
    assert_eq!(
        skipped(&report),
        [
            (1, "碼", &SkipReason::Malformed("詞是空的")),
            (2, "台口\u{b}", &SkipReason::ControlCharacter),
        ]
    );
}

#[test]
fn shortcuts_of_words_that_are_already_user_words_are_not_offered_again() {
    let engine = engine();
    let mut user = UserModel::default();
    user.import_word_list(&engine, GBOARD, Shortcuts::AsTriggers)
        .unwrap();

    // 再匯入同一份：詞都已是使用者詞，沒有快捷鍵可設，殼就不必再問。
    let report = import(&mut user, GBOARD);
    assert_eq!(report.added, 0);
    assert_eq!(trigger_shortcuts(&report), []);
}
