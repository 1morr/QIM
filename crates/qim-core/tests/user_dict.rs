//! 使用者詞庫的匯出格式 `# qim-userdict v3`（plan §2.4）：對外的檔案格式，改了要升版本
//! （brief §5.3）；v1、v2 的檔一直要收。v3 在 M4 收尾前還沒定稿（M4 spec〈共同規則〉）。

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, Date, Engine, ImportError, Packs, Ranking, Scheme, SchemePack, Session,
    UserModel,
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

fn date(text: &str) -> Date {
    text.parse().unwrap()
}

/// 讀檔成一份新的個人模型，沒有日期的列（v1、v2）取 `today`；有警告就失敗。
fn import(text: &str, today: &str) -> Result<UserModel, ImportError> {
    let imported = UserModel::import(text, Some(date(today)))?;
    assert_eq!(imported.warnings, [], "{text}");
    Ok(imported.user)
}

/// 讀不進來的檔錯在第幾行。
fn line_of(text: &str) -> usize {
    UserModel::import(text, None).unwrap_err().line
}

/// 手寫的檔：裝置 `device-a` 在 2026-11-12 `mir` 分段選「一」「台」上屏一次（自動造詞），隔天手動加
/// 「口口」、觸發詞 `kk`。每列開頭是種類：`entry` 是（方案, 輸入碼, 詞）的計數與使用者詞，`bigram` 是
/// 個人 bigram（各方案共用，tick 是不帶方案的 `#@/tick`）；`last_used` 是最後使用的日期。
const ONE_AUTO_PHRASE_AND_ONE_MANUAL_WORD: &str = "\
# qim-userdict v3
#@/device\tdevice-a
#@/tick\t1
#@/tick/quick\t1
# kind\tscheme\tword\tcode\tcommits\tlast_commit_tick\tdecayed\tlast_tick\tlast_used\tlast_commit_date\tsource\ttrigger
entry\tquick\t台\tir\t0\t0\t1\t1\t2026-11-12\t\tlearned\t
entry\tquick\t一\tm\t0\t0\t1\t1\t2026-11-12\t\tlearned\t
entry\tquick\t一台\tmir\t1\t1\t1\t1\t2026-11-12\t2026-11-12\tauto\t
entry\tquick\t口口\trr\t0\t0\t0\t0\t2026-11-13\t\tmanual\tkk
# kind\tprevious\tword\tdecayed\tlast_tick\tlast_used
bigram\t一\t台\t1\t1\t2026-11-12
";

/// M1 票 14 定稿的 v1 檔（固定、不隨匯出改）：同上一份去掉個人 bigram、日期與裝置。
const V1_FILE: &str = "\
# qim-userdict v1
#@/tick/quick\t1
# scheme\tword\tcode\tcommits\tlast_commit_tick\tdecayed\tlast_tick\tsource\ttrigger
quick\t台\tir\t0\t0\t1\t1\tlearned\t
quick\t一\tm\t0\t0\t1\t1\tlearned\t
quick\t一台\tmir\t1\t1\t1\t1\tauto\t
quick\t口口\trr\t0\t0\t0\t0\tmanual\tkk
";

/// M3 票 23 定稿的 v2 檔（固定、不隨匯出改）：同 v1 加上個人 bigram。
const V2_FILE: &str = "\
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

#[test]
fn the_export_of_an_auto_phrase_and_a_manual_word_matches_the_documented_format() {
    let engine = engine();
    let mut user = UserModel::default();
    user.set_device("device-a").unwrap();
    user.set_today(date("2026-11-12"));
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
    user.set_today(date("2026-11-13"));
    user.add_word(&engine, "口口", Some("kk")).unwrap();

    assert_eq!(user.export(), ONE_AUTO_PHRASE_AND_ONE_MANUAL_WORD);
}

/// 換一天再用同一組（碼, 詞），那列的日期跟著變；沒用到的列不動。
#[test]
fn using_an_entry_on_a_later_day_moves_only_its_date() {
    let engine = engine();
    let mut user = import(ONE_AUTO_PHRASE_AND_ONE_MANUAL_WORD, "2027-01-01").unwrap();
    user.set_today(date("2027-03-04"));
    let mut session = Session::default();
    for c in "kk".chars() {
        session.handle(&engine, &mut user, qim_core::Key::Letter(c));
    }
    let committed = session.handle(&engine, &mut user, qim_core::Key::Space);
    assert_eq!(committed.commit.as_deref(), Some("口口"));
    let exported = user.export();
    assert!(
        exported
            .contains("entry\tquick\t口口\tkk\t1\t2\t1\t2\t2027-03-04\t2027-03-04\tlearned\t\n"),
        "{exported}"
    );
    assert!(
        exported.contains("entry\tquick\t口口\trr\t0\t0\t0\t0\t2026-11-13\t\tmanual\tkk\n"),
        "{exported}"
    );
}

#[test]
fn an_imported_file_works_like_the_learning_it_records() {
    let engine = engine();
    let user = import(ONE_AUTO_PHRASE_AND_ONE_MANUAL_WORD, "2030-01-01").unwrap();
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
    assert_eq!(user.device(), Some("device-a"));
    assert_eq!(user.today(), Some(date("2030-01-01")));
    let again = import(&user.export(), "2030-01-01").unwrap();
    assert_eq!(again, user);
}

#[test]
fn windows_line_endings_comments_and_blank_lines_are_accepted() {
    let text = ONE_AUTO_PHRASE_AND_ONE_MANUAL_WORD.replace('\n', "\r\n") + "\r\n# 註解\r\n";
    assert_eq!(
        import(&text, "2030-01-01").unwrap().export(),
        ONE_AUTO_PHRASE_AND_ONE_MANUAL_WORD
    );
}

/// 不認得的種類（之後的版本加的）略過並回警告、帶行號，其他列照收（M4 spec〈共同規則〉）。
#[test]
fn rows_of_an_unknown_kind_are_skipped_with_a_warning_and_the_rest_are_kept() {
    let text = ONE_AUTO_PHRASE_AND_ONE_MANUAL_WORD.replace(
        "bigram\t",
        "tombstone\tquick\t一\tm\t2026-11-12\nfuture\nbigram\t",
    );
    let imported = UserModel::import(&text, Some(date("2030-01-01"))).unwrap();
    let warned: Vec<usize> = imported.warnings.iter().map(|w| w.line).collect();
    assert_eq!(warned, [11, 12]);
    assert!(
        imported.warnings[0].to_string().contains("tombstone"),
        "{}",
        imported.warnings[0]
    );
    assert_eq!(imported.user.export(), ONE_AUTO_PHRASE_AND_ONE_MANUAL_WORD);
}

#[test]
fn a_file_that_is_not_userdict_v1_v2_or_v3_or_has_a_bad_row_is_an_error_with_its_line() {
    assert_eq!(line_of(""), 1);
    assert_eq!(line_of("# qim-userdict v4\n"), 1);
    assert_eq!(line_of("# Rime user dictionary\n"), 1);
    let v1 = "# qim-userdict v1\n";
    let v3 = "# qim-userdict v3\n";
    let bad_rows = [
        (v1, "quick\t一\tm\t0\t0\t1\t1\tlearned"),     // 少一欄
        (v1, "quick\t一\tm\tx\t0\t1\t1\tlearned\t"),   // 次數不是數字
        (v1, "quick\t一\tm\t0\t0\t-1\t1\tlearned\t"),  // 衰減值是負的
        (v1, "quick\t一\tm\t0\t0\t1\t1\tlexicon\t"),   // 沒有這種來源
        (v1, "quick\t一\tm\t0\t0\t1\t1\tlearned\tkk"), // 學到的不是使用者詞，沒有觸發詞
        (v1, "quick\t一\tm\t0\t0\t0\t0\tlearned\t"),   // 學到的卻沒有計數
        (v1, "quick\t\tm\t0\t0\t1\t1\tlearned\t"),     // 詞是空的
        (
            v1,
            "entry\tquick\t一\tm\t0\t0\t1\t1\t2026-11-12\t\tlearned\t",
        ), // v1 沒有種類欄
        (v3, "\tquick\t一\tm\t0\t0\t1\t1\t2026-11-12\tlearned\t"), // 種類是空的
        (v3, "entry\tquick\t一\tm\t0\t0\t1\t1\tlearned\t"), // 少了日期
        (v3, "entry\tquick\t一\tm\t0\t0\t1\t1\t2026-11-12\tlearned\t"), // 少了最後整串上屏的日期
        (
            v3,
            "entry\tquick\t一\tm\t1\t1\t1\t1\t2026-11-12\t2026-13-01\tlearned\t",
        ), // 最後整串上屏的日期不是真的日期
        (
            v3,
            "entry\tquick\t一\tm\t0\t0\t1\t1\t2026-11-12\t2026-11-12\tlearned\t",
        ), // 沒整串上屏過卻有最後整串上屏的日期
        (
            v3,
            "entry\tquick\t一\tm\t0\t0\t1\t1\t2026-02-30\t\tlearned\t",
        ), // 不是真的日期
        (
            v3,
            "entry\tquick\t一\tm\t0\t0\t1\t1\t2026/11/12\t\tlearned\t",
        ), // 日期的寫法
        (
            v3,
            "entry\tquick\t一\tm\t0\t0\t1\t1\t2026-11-12\t\tlexicon\t",
        ), // 已知種類照舊嚴格
    ];
    for (header, row) in bad_rows {
        assert_eq!(line_of(&format!("{header}{row}\n")), 2, "{row}");
    }
    let duplicate = "quick\t一\tm\t0\t0\t1\t1\tlearned\t\n";
    assert_eq!(line_of(&format!("{v1}{duplicate}{duplicate}")), 3);
    let after_tick = "#@/tick/quick\t1\nquick\t一\tm\t0\t0\t1\t2\tlearned\t\n";
    assert_eq!(
        line_of(&format!("{v1}{after_tick}")),
        3,
        "最後 tick 比方案的 tick 大"
    );
}

/// v1、v2 沒有日期：每列的最後使用日期取匯入那天（票 12 的 Comments），也沒有裝置；匯出成 v3。
#[test]
fn v1_and_v2_files_import_with_the_import_day_as_last_used_and_export_as_v3() {
    let expected = ONE_AUTO_PHRASE_AND_ONE_MANUAL_WORD
        .replace("#@/device\tdevice-a\n", "")
        .replace("2026-11-12", "2027-05-06")
        .replace("2026-11-13", "2027-05-06");
    let from_v2 = import(V2_FILE, "2027-05-06").unwrap();
    assert_eq!(from_v2.export(), expected);
    assert_eq!(from_v2.device(), None);

    let without_bigrams: String = expected
        .lines()
        .filter(|l| {
            !l.starts_with("bigram") && !l.contains("previous") && !l.starts_with("#@/tick\t")
        })
        .map(|l| format!("{l}\n"))
        .collect();
    assert_eq!(
        import(V1_FILE, "2027-05-06").unwrap().export(),
        without_bigrams
    );
}

/// 殼拿匯入的檔（可能是別台匯出的）或空白的個人模型取代學習資料時，今天與這台裝置的 id 留著：
/// 檔裡的裝置 id 是別台的，換過來的話之後各台分開的次數（M4 票 17）會記到別台那格。
#[test]
fn replacing_the_learning_keeps_today_and_this_device() {
    let mut user = UserModel::default();
    user.set_device("this-device").unwrap();
    user.set_today(date("2027-02-03"));
    let other = import(ONE_AUTO_PHRASE_AND_ONE_MANUAL_WORD, "2030-01-01").unwrap();
    user.replace_learning(other);
    assert_eq!(user.device(), Some("this-device"));
    assert_eq!(user.today(), Some(date("2027-02-03")));
    assert_eq!(
        user.export(),
        ONE_AUTO_PHRASE_AND_ONE_MANUAL_WORD.replace("device-a", "this-device")
    );

    user.replace_learning(UserModel::default());
    assert_eq!(user.device(), Some("this-device"));
    assert_eq!(user.today(), Some(date("2027-02-03")));
    assert!(!user.export().contains("\nentry\t"), "{}", user.export());
}

/// 殼沒給過日期時學到的條目沒有日期：日期欄留空，不寫假日期（例如 1970-01-01），之後的一年時限
/// 才不會把它們當成很久沒用（plan §2.4 殼的日期契約）。空的日期欄讀回來也是不知道。
#[test]
fn learning_before_the_shell_gives_a_date_writes_no_date() {
    let engine = engine();
    let mut user = UserModel::default();
    assert_eq!(user.today(), None);
    let mut session = Session::default();
    session.set_context("一");
    for c in "mir".chars() {
        session.handle(&engine, &mut user, qim_core::Key::Letter(c));
    }
    session.handle(&engine, &mut user, qim_core::Key::Space);
    user.add_word(&engine, "口口", Some("kk")).unwrap();
    let exported = user.export();
    assert!(!exported.contains("1970"), "{exported}");
    let rows: Vec<&str> = exported.lines().filter(|l| !l.starts_with('#')).collect();
    assert!(rows.len() >= 3, "{exported}");
    for row in &rows {
        let last_used = match row.split('\t').next() {
            Some("entry") => {
                assert_eq!(row.split('\t').nth(9), Some(""), "整串上屏的日期：{row}");
                row.split('\t').nth(8)
            }
            Some("bigram") => row.split('\t').nth(5),
            _ => None,
        };
        assert_eq!(last_used, Some(""), "{row}");
    }
    let again = UserModel::import(&exported, None).unwrap().user;
    assert_eq!(again, user);
    // 匯入時知道今天，v3 的空日期仍是不知道（只有沒有日期欄的 v1、v2 取匯入那天）。
    assert_eq!(import(&exported, "2026-11-12").unwrap().export(), exported);
    let dated = import(V2_FILE, "2026-11-12").unwrap().export();
    let entries = dated.lines().filter(|l| l.starts_with("entry\t"));
    assert!(
        entries
            .map(|l| l.split('\t').nth(8))
            .all(|d| d == Some("2026-11-12")),
        "{dated}"
    );
    let undated = UserModel::import(V2_FILE, None).unwrap().user.export();
    assert!(
        undated.contains("entry\tquick\t一台\tmir\t1\t1\t1\t1\t\t\tauto\t\n"),
        "{undated}"
    );
}

/// 裝置 id 由殼產生（網頁用 `crypto.randomUUID()`，照 librime 每次安裝一個 id）：英數、`-`、`_`，1–64 字。
#[test]
fn a_device_id_is_short_ascii_and_a_bad_one_in_a_file_is_an_error() {
    let mut user = UserModel::default();
    assert!(
        user.set_device("6f1c2d3e-4b5a-4c6d-8e7f-0a1b2c3d4e5f")
            .is_ok()
    );
    for bad in ["", "has space", "tab\t", "裝置", &"a".repeat(65)] {
        assert!(user.set_device(bad).is_err(), "{bad:?}");
    }
    assert_eq!(user.device(), Some("6f1c2d3e-4b5a-4c6d-8e7f-0a1b2c3d4e5f"));
    assert_eq!(line_of("# qim-userdict v3\n#@/device\tno way\n"), 2);
    assert_eq!(
        line_of("# qim-userdict v3\n#@/device\ta\n#@/device\tb\n"),
        3,
        "重複"
    );
}

#[test]
fn a_bad_bigram_row_is_an_error_with_its_line() {
    let v2 = "# qim-userdict v2\n";
    let v3 = "# qim-userdict v3\n";
    let bad_rows = [
        (v2, "\t一\t台\t1"),                      // 少一欄
        (v2, "\t一\t台\t1\t1\t"),                 // 多一欄
        (v2, "\t\t台\t1\t1"),                     // 前一個字詞是空的是記住的字串，台不是 email
        (v2, "\t一\t\t1\t1"),                     // 這一段是空的
        (v2, "\t一\t台\t0\t1"),                   // 衰減值要是正的
        (v2, "\t一\t台\tNaN\t1"),                 // 衰減值不是有限數
        (v2, "\t一\t台\t1\tx"),                   // tick 不是整數
        (v2, "\t一\\x\t台\t1\t1"),                // 沒有這種跳脫
        (v2, "\t一\t台\t1\t1\t2026-11-12"),       // v2 沒有日期
        (v3, "bigram\t一\t台\t1\t1"),             // 少了日期
        (v3, "bigram\t\t台\t1\t1\t2026-11-12"),   // 前一個字詞是空的
        (v3, "bigram\t一\t台\t0\t1\t2026-11-12"), // 衰減值要是正的
        (v3, "bigram\t一\t台\t1\t1\t2026-11-31"), // 不是真的日期
    ];
    for (header, row) in bad_rows {
        assert_eq!(line_of(&format!("{header}{row}\n")), 2, "{row:?}");
    }
    let row = "\t一\t台\t1\t1\n";
    assert_eq!(line_of(&format!("{v2}{row}{row}")), 3, "重複");
    assert_eq!(
        line_of(&format!("{v2}#@/tick\t0\n{row}")),
        3,
        "最後 tick 比 tick 大"
    );
    assert_eq!(line_of(&format!("{v2}#@/tick\tx\n")), 2, "tick 不是整數");
    assert_eq!(
        line_of(&format!("# qim-userdict v1\n{row}")),
        2,
        "v1 沒有個人 bigram"
    );
}

/// 個人 bigram 的前一個字詞與這一段照詞欄跳脫；沒有 tick 行時 tick 取最大的最後 tick。
#[test]
fn bigram_rows_unescape_and_without_a_tick_line_the_tick_is_the_latest() {
    let user = import("# qim-userdict v2\n\t九龍\\n\t3 樓\t2.5\t7\n", "2026-11-12").unwrap();
    let expected = "# qim-userdict v3\n#@/tick\t7\n# kind\tscheme\tword\tcode\tcommits\tlast_commit_tick\tdecayed\tlast_tick\tlast_used\tlast_commit_date\tsource\ttrigger\n# kind\tprevious\tword\tdecayed\tlast_tick\tlast_used\nbigram\t九龍\\n\t3 樓\t2.5\t7\t2026-11-12\n";
    assert_eq!(user.export(), expected);
    let v3 = expected.replace("#@/tick\t7\n", "");
    assert_eq!(import(&v3, "2030-01-01").unwrap().export(), expected);
}

/// v1 的詞欄照字面讀：反斜線不是跳脫。
#[test]
fn a_backslash_in_a_v1_word_is_literal() {
    let v1 = "# qim-userdict v1\nquick\tC:\\n\tkk\t0\t0\t0\t0\tmanual\tkk\n";
    let user = import(v1, "2026-11-12").unwrap();
    assert!(
        user.export().contains("entry\tquick\tC:\\\\n\tkk\t"),
        "{}",
        user.export()
    );
}

/// 常用語：碼是空的、沒有計數；詞欄的換行、Tab、反斜線寫成 `\n`、`\t`、`\\`。觸發詞學到的計數
/// 另是一列 `learned`。常用語那列的日期是存（或改）它的那天。
const TWO_PHRASES: &str = "\
# qim-userdict v3
#@/tick\t1
#@/tick/quick\t1
# kind\tscheme\tword\tcode\tcommits\tlast_commit_tick\tdecayed\tlast_tick\tlast_used\tlast_commit_date\tsource\ttrigger
entry\tquick\tC:\\\\ 與\\t\t\t0\t0\t0\t0\t2026-11-12\t\tphrase\t
entry\tquick\t九龍\\n3 樓\t\t0\t0\t0\t0\t2026-11-12\t\tphrase\tmir
entry\tquick\t九龍\\n3 樓\tmir\t1\t1\t1\t1\t2026-11-14\t2026-11-14\tlearned\t
";

/// 同一份常用語寫成 v2（M3 票 23 定稿，固定）。
const TWO_PHRASES_V2: &str = "\
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
    user.set_today(date("2026-11-12"));
    let mut session = Session::default();
    user.add_phrase(&engine, "九龍\n3 樓", Some("mir")).unwrap();
    user.add_phrase(&engine, "C:\\ 與\t", None).unwrap();
    user.set_today(date("2026-11-14"));
    for c in "mir".chars() {
        session.handle(&engine, &mut user, qim_core::Key::Letter(c));
    }
    let committed = session.handle(&engine, &mut user, qim_core::Key::Space);
    assert_eq!(committed.commit.as_deref(), Some("九龍\n3 樓"));
    assert_eq!(user.export(), TWO_PHRASES);
    assert_eq!(import(TWO_PHRASES, "2026-11-14"), Ok(user));

    let from_v2 = import(TWO_PHRASES_V2, "2026-11-12").unwrap();
    assert_eq!(
        from_v2.export(),
        TWO_PHRASES.replace("2026-11-14", "2026-11-12")
    );
}

#[test]
fn a_bad_phrase_row_or_escape_is_an_error_with_its_line() {
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
        let v3 = row
            .replacen("quick", "entry\tquick", 1)
            .replace("\tphrase", "\t2026-11-12\t\tphrase")
            .replace("\tlearned", "\t2026-11-12\t\tlearned");
        assert_eq!(line_of(&format!("# qim-userdict v3\n{v3}\n")), 2, "{v3}");
    }
    let v1_phrase = "# qim-userdict v1\nquick\t一\t\t0\t0\t0\t0\tphrase\t\n";
    assert_eq!(line_of(v1_phrase), 2, "v1 沒有常用語");
}

/// tick 行寫在列後面、又比列的最後 tick 小，也整份不收（照順序讀時列先過了檢查）。
#[test]
fn a_tick_line_after_rows_smaller_than_their_last_tick_is_an_error() {
    let cases = [
        (
            "# qim-userdict v2\n\t一\t台\t1\t5\n#@/tick\t3\n",
            "個人 bigram",
        ),
        (
            "# qim-userdict v2\nquick\t一\tm\t0\t0\t1\t5\tlearned\t\n#@/tick/quick\t3\n",
            "方案",
        ),
        (
            "# qim-userdict v3\nbigram\t一\t台\t1\t5\t2026-11-12\n#@/tick\t3\n",
            "v3 個人 bigram",
        ),
        (
            "# qim-userdict v3\nentry\tquick\t一\tm\t0\t0\t1\t5\t2026-11-12\t\tlearned\t\n#@/tick/quick\t3\n",
            "v3 方案",
        ),
    ];
    for (text, what) in cases {
        assert_eq!(line_of(text), 3, "{what}");
    }
}

/// v1 沒有共用的 tick：`#@/tick` 行和其他 `#` 開頭的行一樣是註解。
#[test]
fn a_shared_tick_line_in_v1_is_a_comment() {
    let text = V1_FILE.replace("#@/tick/quick\t1\n", "#@/tick/quick\t1\n#@/tick\tx\n");
    assert!(import(&text, "2026-11-12").is_ok());
}
