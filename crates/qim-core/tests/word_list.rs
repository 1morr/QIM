//! 詞表匯入（M3 票 17，plan §2.4）：把別的輸入法匯出或自己整理的詞表加進使用者詞。收 Rime
//! `.dict.yaml`、Gboard 個人字典的 `dictionary.txt`、一行一詞的純文字；碼照手動加詞由字推出。

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, Engine, Packs, Ranking, Scheme, SchemePack, Session, Skip, UserModel,
    WordListFormat, WordListReport,
};

/// `m`「一」、`ir`「台」、`r`「口」，沒有詞庫與語言模型（同 `user_dict.rs`）。
fn engine() -> Engine {
    let scheme = Scheme {
        id: "quick".into(),
        name: "quick".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        radicals: BTreeMap::new(),
        packs: Packs::default(),
        ranking: Ranking::default(),
    };
    let code_table = CodeTable::from_entries([("m", '一'), ("ir", '台'), ("r", '口')].map(
        |(code, text)| {
            let entry = CodeEntry {
                text,
                extension: false,
            };
            (code.to_string(), vec![entry])
        },
    ));
    Engine::new(SchemePack { scheme, code_table }, None, None).unwrap()
}

const RIME: &str = include_str!("fixtures/word-list/rime.dict.yaml");
const GBOARD: &str = include_str!("fixtures/word-list/gboard.txt");
const PLAIN: &str = include_str!("fixtures/word-list/plain.txt");

fn import(user: &mut UserModel, text: &str) -> WordListReport {
    user.import_word_list(&engine(), text).unwrap()
}

/// 略過的列：（行號, 詞, 原因）。
fn skipped(report: &WordListReport) -> Vec<(usize, &str, &Skip)> {
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
        [(17, "口口\u{7}", &Skip::ControlCharacter)]
    );
}

#[test]
fn a_gboard_dictionary_keeps_shortcuts_that_are_scheme_keys_as_triggers() {
    let mut user = UserModel::default();
    let report = import(&mut user, GBOARD);

    assert_eq!(report.format, WordListFormat::Gboard);
    assert_eq!(report.added, 3);
    assert_eq!(typed(&user, "kk").first().map(String::as_str), Some("口口"));
    assert!(typed(&user, "rr").contains(&"口口".to_string()));
    // 快捷鍵 `IIRC` 不是速成的鍵：詞照樣加，只是沒有觸發詞。
    assert!(typed(&user, "irr").contains(&"台口".to_string()));
    assert_eq!(
        report
            .without_trigger
            .iter()
            .map(|w| (w.line, w.text.as_str(), w.trigger.as_str()))
            .collect::<Vec<_>>(),
        [(4, "台口", "IIRC")]
    );
    assert!(typed(&user, "mir").contains(&"一台".to_string()));
    assert_eq!(
        skipped(&report),
        [
            (3, "STRAY", &Skip::NoCode),
            (
                6,
                "broken-row",
                &Skip::Malformed("Gboard 的列要是「快捷鍵<Tab>詞<Tab>語言」")
            ),
        ]
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
            (5, "一台", &Skip::Duplicate { first_line: 1 }),
            (6, "口", &Skip::InLexicon),
            (7, "hello", &Skip::NoCode),
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

    let report = user.import_word_list(&engine, PLAIN).unwrap();

    assert_eq!(report.added, 3);
    // 原本的學習資料都還在，加的是手動加的詞（來源 `manual`）。
    let after = user.export();
    for line in before.lines().filter(|l| !l.starts_with('#')) {
        assert!(after.contains(line), "少了：{line}\n{after}");
    }
    assert!(after.contains("quick\t一台\tmir\t0\t0\t0\t0\tmanual\t\n"));
    assert_eq!(typed(&user, "tk").first().map(String::as_str), Some("台口"));
}

#[test]
fn a_word_that_is_already_a_user_word_is_skipped_and_keeps_its_trigger() {
    let engine = engine();
    let mut user = UserModel::default();
    user.add_word(&engine, "一台", Some("yt")).unwrap();

    let report = user.import_word_list(&engine, "一台\n").unwrap();

    assert_eq!(report.added, 0);
    assert_eq!(skipped(&report), [(1, "一台", &Skip::AlreadyUserWord)]);
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
        [(4, "只有碼", &Skip::Malformed("少了 columns 裡的 text 欄"))]
    );
}

#[test]
fn a_rime_file_that_cannot_be_read_at_all_is_an_error_with_a_line_number() {
    let engine = engine();
    let mut user = UserModel::default();

    let unclosed = user
        .import_word_list(&engine, "# Rime dictionary\n---\nname: x\n一台\n")
        .unwrap_err();
    assert_eq!(unclosed.line, 2);

    let no_text = user
        .import_word_list(
            &engine,
            "---\nname: x\ncolumns:\n  - code\n  - weight\n...\n一台\n",
        )
        .unwrap_err();
    assert_eq!(no_text.line, 3);
    assert_eq!(user, UserModel::default());
}

#[test]
fn checking_on_a_copy_leaves_the_model_untouched() {
    let engine = engine();
    let user = UserModel::default();
    let report = user.clone().import_word_list(&engine, PLAIN).unwrap();
    assert_eq!(report.added, 3);
    assert_eq!(user, UserModel::default());
}
