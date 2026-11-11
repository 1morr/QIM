//! 使用者詞庫的匯出格式 `# qim-userdict v2`（plan §2.4）：對外的檔案格式，改了要升版本
//! （brief §5.3）；v1 的檔一直要收。

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
/// 分段選字記下個人 bigram「一 → 台」：方案欄是空的（各方案共用），tick 是不帶方案的 `#@/tick`。
const ONE_AUTO_PHRASE_AND_ONE_MANUAL_WORD: &str = "\
# qim-userdict v2
#@/tick\t1
#@/tick/quick\t1
# scheme\tword\tcode\tcommits\tlast_commit_tick\tdecayed\tlast_tick\tsource\ttrigger
quick\t台\tir\t0\t0\t1\t1\tlearned\t
quick\t一\tm\t0\t0\t1\t1\tlearned\t
quick\t一台\tmir\t1\t1\t1\t1\tauto\t
quick\t口口\trr\t0\t0\t0\t0\tmanual\tkk
# \tprevious\tword\tdecayed\tlast_tick
\t一\t台\t1\t1
";

/// M1 票 14 定稿的 v1 檔（固定、不隨匯出改）：同上一份去掉個人 bigram。
const V1_FILE: &str = "\
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
fn a_file_that_is_not_userdict_v1_or_v2_or_has_a_bad_row_is_an_error_with_its_line() {
    let line_of = |text: &str| UserModel::import(text).unwrap_err().line;
    assert_eq!(line_of(""), 1);
    assert_eq!(line_of("# qim-userdict v3\n"), 1);
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

#[test]
fn a_v1_file_imports_the_same_learning_without_bigrams_and_exports_as_v2() {
    let expected = V1_FILE.replace("# qim-userdict v1", "# qim-userdict v2");
    assert_eq!(UserModel::import(V1_FILE).unwrap().export(), expected);
}

#[test]
fn a_bad_bigram_row_is_an_error_with_its_line() {
    let line_of = |text: &str| UserModel::import(text).unwrap_err().line;
    let header = "# qim-userdict v2\n";
    let bad_rows = [
        "\t一\t台\t1",       // 少一欄
        "\t一\t台\t1\t1\t",  // 多一欄
        "\t\t台\t1\t1",      // 前一個字詞是空的是記住的字串，台不是 email
        "\t一\t\t1\t1",      // 這一段是空的
        "\t一\t台\t0\t1",    // 衰減值要是正的
        "\t一\t台\tNaN\t1",  // 衰減值不是有限數
        "\t一\t台\t1\tx",    // tick 不是整數
        "\t一\\x\t台\t1\t1", // 沒有這種跳脫
    ];
    for row in bad_rows {
        assert_eq!(line_of(&format!("{header}{row}\n")), 2, "{row:?}");
    }
    let row = "\t一\t台\t1\t1\n";
    assert_eq!(line_of(&format!("{header}{row}{row}")), 3, "重複");
    assert_eq!(
        line_of(&format!("{header}#@/tick\t0\n{row}")),
        3,
        "最後 tick 比 tick 大"
    );
    assert_eq!(
        line_of(&format!("{header}#@/tick\tx\n")),
        2,
        "tick 不是整數"
    );
    assert_eq!(
        line_of(&format!("# qim-userdict v1\n{row}")),
        2,
        "v1 沒有個人 bigram"
    );
}

/// 個人 bigram 的前一個字詞與這一段照詞欄跳脫；沒有 tick 行時 tick 取最大的最後 tick。
#[test]
fn bigram_rows_unescape_and_without_a_tick_line_the_tick_is_the_latest() {
    let text = "# qim-userdict v2\n\t九龍\\n\t3 樓\t2.5\t7\n";
    let user = UserModel::import(text).unwrap();
    assert_eq!(
        user.export(),
        "# qim-userdict v2\n#@/tick\t7\n# scheme\tword\tcode\tcommits\tlast_commit_tick\tdecayed\tlast_tick\tsource\ttrigger\n# \tprevious\tword\tdecayed\tlast_tick\n\t九龍\\n\t3 樓\t2.5\t7\n"
    );
}

/// v1 的詞欄照字面讀：反斜線不是跳脫。
#[test]
fn a_backslash_in_a_v1_word_is_literal() {
    let v1 = "# qim-userdict v1\nquick\tC:\\n\tkk\t0\t0\t0\t0\tmanual\tkk\n";
    let user = UserModel::import(v1).unwrap();
    assert!(
        user.export().contains("quick\tC:\\\\n\tkk\t"),
        "{}",
        user.export()
    );
}

/// 常用語：碼是空的、沒有計數；詞欄的換行、Tab、反斜線寫成 `\n`、`\t`、`\\`。觸發詞學到的計數
/// 另是一列 `learned`。
const TWO_PHRASES: &str = "\
# qim-userdict v2
#@/tick\t1
#@/tick/quick\t1
# scheme\tword\tcode\tcommits\tlast_commit_tick\tdecayed\tlast_tick\tsource\ttrigger
quick\tC:\\\\ 與\\t\t\t0\t0\t0\t0\tphrase\t
quick\t九龍\\n3 樓\t\t0\t0\t0\t0\tphrase\tmir
quick\t九龍\\n3 樓\tmir\t1\t1\t1\t1\tlearned\t
";

#[test]
fn phrases_export_with_escaped_line_breaks_and_import_back() {
    let engine = engine();
    let mut user = UserModel::default();
    let mut session = Session::default();
    user.add_phrase(&engine, "九龍\n3 樓", Some("mir")).unwrap();
    user.add_phrase(&engine, "C:\\ 與\t", None).unwrap();
    for c in "mir".chars() {
        session.handle(&engine, &mut user, qim_core::Key::Letter(c));
    }
    let committed = session.handle(&engine, &mut user, qim_core::Key::Space);
    assert_eq!(committed.commit.as_deref(), Some("九龍\n3 樓"));
    assert_eq!(user.export(), TWO_PHRASES);

    let imported = UserModel::import(TWO_PHRASES).unwrap();
    assert_eq!(imported, user);
}

#[test]
fn a_bad_phrase_row_or_escape_is_an_error_with_its_line() {
    let line_of = |text: &str| UserModel::import(text).unwrap_err().line;
    let header = "# qim-userdict v2\n";
    let bad_rows = [
        "quick\t一\tm\t0\t0\t0\t0\tphrase\t",   // 常用語的碼要是空的
        "quick\t一\t\t1\t1\t1\t1\tphrase\t",    // 常用語沒有計數
        "quick\t一\t\t0\t0\t1\t1\tlearned\t",   // 其他列的碼不能是空的
        "quick\t一\\x\t\t0\t0\t0\t0\tphrase\t", // 沒有這種跳脫
        "quick\t一\\\t\t0\t0\t0\t0\tphrase\t",  // 結尾只有反斜線
    ];
    for row in bad_rows {
        assert_eq!(line_of(&format!("{header}{row}\n")), 2, "{row}");
    }
    let v1_phrase = "# qim-userdict v1\nquick\t一\t\t0\t0\t0\t0\tphrase\t\n";
    assert_eq!(line_of(v1_phrase), 2, "v1 沒有常用語");
}

/// tick 行寫在列後面、又比列的最後 tick 小，也整份不收（照順序讀時列先過了檢查）。
#[test]
fn a_tick_line_after_rows_smaller_than_their_last_tick_is_an_error() {
    let line_of = |text: &str| UserModel::import(text).unwrap_err().line;
    let header = "# qim-userdict v2\n";
    assert_eq!(
        line_of(&format!("{header}\t一\t台\t1\t5\n#@/tick\t3\n")),
        3,
        "個人 bigram"
    );
    assert_eq!(
        line_of(&format!(
            "{header}quick\t一\tm\t0\t0\t1\t5\tlearned\t\n#@/tick/quick\t3\n"
        )),
        3,
        "方案"
    );
}

/// v1 沒有共用的 tick：`#@/tick` 行和其他 `#` 開頭的行一樣是註解。
#[test]
fn a_shared_tick_line_in_v1_is_a_comment() {
    let text = V1_FILE.replace("#@/tick/quick\t1\n", "#@/tick/quick\t1\n#@/tick\tx\n");
    assert!(UserModel::import(&text).is_ok());
}
