//! 使用者詞庫的匯出格式 `# qim-userdict v1`（plan §2.4）：對外的檔案格式，改了要升版本
//! （brief §5.3）。

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, Engine, Packs, Ranking, Scheme, SchemePack, Session, UserModel,
};

/// `m`「一」、`ir`「台」、`r`「口」，沒有詞庫與語言模型。
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
                simplified: false,
            };
            (code.to_string(), vec![entry])
        },
    ));
    Engine::new(SchemePack { scheme, code_table }, None, None).unwrap()
}

/// 手寫的檔，列依輸入碼排：`mir` 分段選「一」「台」上屏一次（自動造詞），再手動加「口口」、觸發詞 `kk`。
const ONE_AUTO_PHRASE_AND_ONE_MANUAL_WORD: &str = "\
# qim-userdict v1
#@/tick/quick\t1
# scheme\tword\tcode\tcommits\tlast_commit_tick\tdecayed\tlast_tick\tsource\ttrigger
quick\t台\tir\t0\t0\t1\t1\tlearned\t
quick\t一\tm\t0\t0\t1\t1\tlearned\t
quick\t一台\tmir\t1\t1\t1\t1\tauto\t
quick\t口口\trr\t0\t0\t0\t0\tmanual\tkk
";

#[test]
fn the_export_of_an_auto_phrase_and_a_manual_word_matches_the_documented_format() {
    let engine = engine();
    let mut user = UserModel::default();
    let mut session = Session::default();
    for c in "mir".chars() {
        session.handle(&engine, &mut user, qim_core::Key::Letter(c));
    }
    // `mir` 的候選：整串的「一台」在前，接著開頭一段的「一」；選「一」再選「台」。
    let at = |s: &qim_core::Snapshot, text: &str| {
        let at = s.candidates.iter().position(|c| c.text == text).unwrap();
        qim_core::Key::Digit(u8::try_from(at + 1).unwrap())
    };
    let shown = session.handle(&engine, &mut user, qim_core::Key::Other);
    let shown = session.handle(&engine, &mut user, at(&shown, "一"));
    let committed = session.handle(&engine, &mut user, at(&shown, "台"));
    assert_eq!(committed.commit.as_deref(), Some("一台"));
    user.add_word(&engine, "口口", Some("kk")).unwrap();

    assert_eq!(user.export(), ONE_AUTO_PHRASE_AND_ONE_MANUAL_WORD);
}

#[test]
fn an_imported_file_works_like_the_learning_it_records() {
    let engine = engine();
    let mut user = UserModel::import(ONE_AUTO_PHRASE_AND_ONE_MANUAL_WORD).unwrap();
    assert_eq!(
        engine.candidates(&user, "kk").first().map(String::as_str),
        Some("口口")
    );
    assert!(
        engine
            .candidates(&user, "mir")
            .contains(&"一台".to_string())
    );
    assert_eq!(user.export(), ONE_AUTO_PHRASE_AND_ONE_MANUAL_WORD);
    user = UserModel::import(&user.export()).unwrap();
    assert_eq!(user.export(), ONE_AUTO_PHRASE_AND_ONE_MANUAL_WORD);
}

#[test]
fn windows_line_endings_comments_and_blank_lines_are_accepted() {
    let text = ONE_AUTO_PHRASE_AND_ONE_MANUAL_WORD.replace('\n', "\r\n") + "\r\n# 註解\r\n";
    assert_eq!(
        UserModel::import(&text).unwrap().export(),
        ONE_AUTO_PHRASE_AND_ONE_MANUAL_WORD
    );
}

#[test]
fn a_file_that_is_not_userdict_v1_or_has_a_bad_row_is_an_error_with_its_line() {
    let line_of = |text: &str| UserModel::import(text).unwrap_err().line;
    assert_eq!(line_of(""), 1);
    assert_eq!(line_of("# qim-userdict v2\n"), 1);
    assert_eq!(line_of("# Rime user dictionary\n"), 1);
    let header = "# qim-userdict v1\n";
    let bad_rows = [
        "quick\t一\tm\t0\t0\t1\t1\tlearned",     // 少一欄
        "quick\t一\tm\tx\t0\t1\t1\tlearned\t",   // 次數不是數字
        "quick\t一\tm\t0\t0\t-1\t1\tlearned\t",  // 衰減值是負的
        "quick\t一\tm\t0\t0\t1\t1\tlexicon\t",   // 沒有這種來源
        "quick\t一\tm\t0\t0\t1\t1\tlearned\tkk", // 學到的不是使用者詞，沒有觸發詞
        "quick\t一\tm\t0\t0\t0\t0\tlearned\t",   // 學到的卻沒有計數
        "quick\t\tm\t0\t0\t1\t1\tlearned\t",     // 詞是空的
    ];
    for row in bad_rows {
        assert_eq!(line_of(&format!("{header}{row}\n")), 2, "{row}");
    }
    let duplicate = "quick\t一\tm\t0\t0\t1\t1\tlearned\t\n";
    assert_eq!(line_of(&format!("{header}{duplicate}{duplicate}")), 3);
    let after_tick = "#@/tick/quick\t1\nquick\t一\tm\t0\t0\t1\t2\tlearned\t\n";
    assert_eq!(
        line_of(&format!("{header}{after_tick}")),
        3,
        "最後 tick 比方案的 tick 大"
    );
}
