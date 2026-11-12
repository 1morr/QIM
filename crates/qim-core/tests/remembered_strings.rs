//! 記住的字串（M3 票 15，plan §2.1、§2.4）：使用者經 QIM 打過的 email 記進個人模型，之後在中文組字打
//! 它開頭的字母（長過一個字的最長碼），它就排在候選第 2 位；選了上屏整串。無痕、學習關掉時不記。

mod common;

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, Engine, EnglishPack, EnglishWords, Key, Options, Packs, REMEMBERED_LIMIT,
    REMEMBERED_STRINGS_MAX, Ranking, Scheme, SchemePack, Session, ShiftKey, Snapshot, UserModel,
};

const EMAIL: &str = "ivan.test@example.com";

/// 每個字母都是一個字的碼（`a` 日、`b` 月…），所以任何字母串都有中文候選。
fn engine() -> Engine {
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        radicals: BTreeMap::new(),
        packs: Packs {
            language_model: Some("model".into()),
            ..Packs::default()
        },
        ranking: Ranking::default(),
    };
    let chars: Vec<(char, char)> = "abcdefghijklmnopqrstuvwxy"
        .chars()
        .zip("日月金木水火土竹戈十大中一弓人心手口尸廿山女田難卜".chars())
        .collect();
    let code_table = CodeTable::from_entries(chars.iter().map(|&(code, text)| {
        (
            code.to_string(),
            vec![CodeEntry {
                text,
                extension: false,
                simplified: false,
            }],
        )
    }));
    let frequency: Vec<(char, u64)> = chars.iter().map(|&(_, c)| (c, 10)).collect();
    let model = common::unigram("model", &frequency, frequency.len());
    Engine::new(SchemePack { scheme, code_table }, None, Some(model)).unwrap()
}

struct Ime {
    engine: Engine,
    session: Session,
    user: UserModel,
}

impl Ime {
    fn new() -> Self {
        Self {
            engine: engine(),
            session: Session::default(),
            user: UserModel::default(),
        }
    }

    fn with_options(options: Options) -> Self {
        let mut ime = Self::new();
        ime.session.set_options(options);
        ime
    }

    fn press(&mut self, key: Key) -> Snapshot {
        self.session.handle(&self.engine, &mut self.user, key)
    }

    /// 實體鍵盤逐字打 `text`：字母、數字、標點、空白各轉成它的鍵。
    fn type_text(&mut self, text: &str) -> Snapshot {
        let mut snapshot = Snapshot::default();
        for c in text.chars() {
            let key = match c {
                ' ' => Key::Space,
                c if c.is_ascii_digit() => Key::Digit(c.to_digit(10).unwrap() as u8),
                c if c.is_ascii_alphabetic() => Key::Letter(c),
                c => Key::Punct(c),
            };
            snapshot = self.press(key);
        }
        snapshot
    }

    /// 英文直通（單按右 Shift）打 `text` 再按空白，再切回中文；網頁每個鍵前給前文，這裡只靠核心自己記。
    fn type_in_english(&mut self, text: &str) {
        self.press(Key::ToggleAscii(ShiftKey::Right));
        let snapshot = self.type_text(text);
        assert!(!snapshot.handled, "英文直通的鍵交還宿主");
        self.press(Key::Space);
        self.press(Key::ToggleAscii(ShiftKey::Right));
    }

    /// 中文打 `code`，回傳全部候選的文字；不上屏。
    fn candidates(&mut self, code: &str) -> Vec<String> {
        self.type_text(code);
        let all = self
            .session
            .all_candidates()
            .into_iter()
            .map(|c| c.text)
            .collect();
        self.press(Key::Escape);
        all
    }
}

#[test]
fn an_email_typed_in_english_comes_back_as_the_second_candidate_and_commits_whole() {
    let mut ime = Ime::new();
    ime.type_in_english(EMAIL);

    let shown = ime.candidates("iva");
    assert_eq!(shown.get(1).map(String::as_str), Some(EMAIL), "{shown:?}");

    ime.type_text("iva");
    let done = ime.press(Key::Digit(2));
    assert_eq!(done.commit.as_deref(), Some(EMAIL));
}

/// 學習的時限（M4 票 13）：一年沒打的記住的字串停用、不刪，日期改回來又出現。
#[test]
fn a_string_not_typed_for_a_year_is_not_offered_but_kept() {
    let mut ime = Ime::new();
    ime.user.set_today("2026-01-01".parse().unwrap());
    ime.type_in_english(EMAIL);
    ime.user.set_today("2027-01-02".parse().unwrap());
    let shown = ime.candidates("iva");
    assert!(!shown.iter().any(|t| t == EMAIL), "{shown:?}");
    assert_eq!(ime.user.remembered_strings(), [EMAIL]);
    ime.user.set_today("2027-01-01".parse().unwrap());
    assert!(ime.candidates("iva").iter().any(|t| t == EMAIL));
}

#[test]
fn incognito_does_not_remember() {
    let mut ime = Ime::with_options(Options {
        incognito: true,
        ..Options::default()
    });
    ime.type_in_english(EMAIL);
    assert!(ime.user.remembered_strings().is_empty());
    ime.session.set_options(Options::default());
    assert!(!ime.candidates("iva").contains(&EMAIL.to_string()));
}

#[test]
fn learning_off_does_not_remember_but_still_offers_what_was_remembered() {
    let mut ime = Ime::new();
    ime.type_in_english(EMAIL);
    ime.session.set_options(Options {
        learning: false,
        ..Options::default()
    });
    ime.type_in_english("ivan@work.example.org");
    assert_eq!(ime.user.remembered_strings(), vec![EMAIL.to_string()]);
    assert_eq!(ime.candidates("iva")[1], EMAIL);
}

#[test]
fn the_option_off_neither_remembers_nor_offers() {
    let mut ime = Ime::new();
    ime.type_in_english(EMAIL);
    ime.session.set_options(Options {
        remembered_strings: false,
        ..Options::default()
    });
    ime.type_in_english("ivan@work.example.org");
    assert_eq!(ime.user.remembered_strings(), vec![EMAIL.to_string()]);
    assert!(!ime.candidates("iva").contains(&EMAIL.to_string()));
}

/// 1–2 碼是一個字的碼，第 2 位最常用，不給（同英文候選，速成 3 碼起）；之後比的是字串的英文字母，
/// 打不出的 `.`、`@` 跳過，所以打到網域也還對得上。
#[test]
fn it_needs_more_letters_than_one_character_code_and_matches_the_letters_only() {
    let mut ime = Ime::new();
    ime.type_in_english(EMAIL);
    for code in ["i", "iv"] {
        assert!(!ime.candidates(code).contains(&EMAIL.to_string()), "{code}");
    }
    for code in ["iva", "ivan", "ivantest", "ivantestex"] {
        assert_eq!(ime.candidates(code)[1], EMAIL, "{code}");
    }
    for code in ["ivb", "ivanb"] {
        assert!(!ime.candidates(code).contains(&EMAIL.to_string()), "{code}");
    }
}

/// 幾個都對得上時最近常打的在前，最多 [`REMEMBERED_LIMIT`] 個，依序從第 2 位起；首選照舊是中文。
#[test]
fn several_matches_follow_the_first_candidate_most_recent_first() {
    let mut ime = Ime::new();
    let emails = [
        "ivan.a@example.com",
        "ivan.b@example.com",
        "ivan.c@example.com",
        "ivan.d@example.com",
    ];
    for email in emails {
        ime.type_in_english(email);
    }
    let shown = ime.candidates("ivan");
    assert_eq!(shown[0], "戈女日弓");
    assert_eq!(
        shown[1..=REMEMBERED_LIMIT],
        [
            "ivan.d@example.com",
            "ivan.c@example.com",
            "ivan.b@example.com"
        ]
    );
    assert!(!shown.contains(&emails[0].to_string()));
}

#[test]
fn typing_it_again_by_choosing_it_moves_it_first() {
    let mut ime = Ime::new();
    ime.type_in_english("ivan.a@example.com");
    ime.type_in_english("ivan.b@example.com");
    ime.type_text("ivan");
    ime.press(Key::Digit(3));
    // 選了上屏後接著打的空白讓那串打完，同自己打一次。
    ime.press(Key::Space);
    assert_eq!(ime.candidates("ivan")[1], "ivan.a@example.com");
}

impl Ime {
    /// 英文直通打 `text`，不按空白，停在英文直通。
    fn type_in_english_without_ending(&mut self, text: &str) {
        self.press(Key::ToggleAscii(ShiftKey::Right));
        self.type_text(text);
    }
}

/// 打完 email 直接點別的欄位（殼呼叫 `clear_context`）也算打完，登入表單常這樣。
#[test]
fn leaving_the_field_ends_the_email() {
    let mut ime = Ime::new();
    ime.type_in_english_without_ending(EMAIL);
    assert!(ime.user.remembered_strings().is_empty(), "還在打");
    ime.session.clear_context(&mut ime.user);
    assert_eq!(ime.user.remembered_strings(), vec![EMAIL.to_string()]);
}

#[test]
fn enter_and_tab_end_the_email_and_a_full_stop_after_it_is_left_out() {
    for (end, key) in [("", Key::Enter), ("", Key::Other), (".", Key::Space)] {
        let mut ime = Ime::new();
        ime.type_in_english_without_ending(&format!("{EMAIL}{end}"));
        ime.press(key);
        assert_eq!(
            ime.user.remembered_strings(),
            vec![EMAIL.to_string()],
            "{key:?}"
        );
    }
}

/// 交還宿主的退格刪掉最後一個字元：打錯改過的照記改過的。
#[test]
fn backspace_removes_the_last_character_typed() {
    let mut ime = Ime::new();
    ime.type_in_english_without_ending("ivan.test@example.comm");
    ime.press(Key::Backspace);
    ime.press(Key::Space);
    assert_eq!(ime.user.remembered_strings(), vec![EMAIL.to_string()]);
}

/// 殼給的前文不以正在打的那串結尾（游標移走、宿主改了字）：那串不一定是輸入框裡的樣子，不記。
#[test]
fn a_context_that_does_not_end_with_the_typed_string_drops_it() {
    let mut ime = Ime::new();
    ime.type_in_english_without_ending(EMAIL);
    ime.session.set_context(&format!("Hi {EMAIL}"));
    ime.press(Key::Space);
    assert_eq!(ime.user.remembered_strings(), vec![EMAIL.to_string()]);

    let mut ime = Ime::new();
    ime.type_in_english_without_ending(EMAIL);
    ime.session.set_context("Hi ");
    ime.press(Key::Space);
    assert!(ime.user.remembered_strings().is_empty());
}

/// 中文組字裡單按左 Shift 接英文（臨時英文）打完再 Enter 上屏，也記；上屏中文接在後面同樣讓它打完。
#[test]
fn an_email_finished_in_temporary_english_is_remembered() {
    let mut ime = Ime::new();
    ime.type_text("iv");
    ime.press(Key::ToggleAscii(ShiftKey::Left));
    ime.type_text("an.test@example.com");
    let done = ime.press(Key::Enter);
    assert_eq!(done.commit.as_deref(), Some(EMAIL));
    ime.type_text("a");
    ime.press(Key::Space);
    assert_eq!(ime.user.remembered_strings(), vec![EMAIL.to_string()]);
}

/// 觸控鍵盤的 `@`、`.` 是 `text` 動作，字母是交還宿主的鍵，合起來也是使用者打的。
#[test]
fn touch_text_keys_count_as_typed() {
    let mut ime = Ime::new();
    ime.press(Key::ToggleAscii(ShiftKey::Right));
    for part in ["ivan", ".", "test", "@", "example", ".", "com"] {
        if part.chars().all(|c| c.is_ascii_alphabetic()) {
            ime.type_text(part);
        } else {
            ime.session.text(&ime.engine, &mut ime.user, part);
        }
    }
    ime.press(Key::Space);
    assert_eq!(ime.user.remembered_strings(), vec![EMAIL.to_string()]);
}

/// 匯出 v3：記住的字串是種類 `string` 的列（各方案共用），衰減值與最後 tick 同個人 bigram，tick 是共用的
/// `#@/tick`；殼沒給日期，最後使用的日期欄是空的。
const TWO_EMAILS: &str = "\
# qim-userdict v3
#@/tick\t3
# kind\tscheme\tword\tcode\tcommits\tlast_commit_tick\tdecayed\tlast_tick\tlast_used\tlast_commit_date\tsource\ttrigger
# kind\tstring\tdecayed\tlast_tick\tlast_used
string\tivan.test@example.com\t1.990049833749168\t3\t
string\tivan@work.example.org\t1\t2\t
";

/// 同一份寫成 v2（M3 票 23 定稿，固定）：方案欄、前一個字詞欄都空的列。
const TWO_EMAILS_V2: &str = "\
# qim-userdict v2
#@/tick\t3
# scheme\tword\tcode\tcommits\tlast_commit_tick\tdecayed\tlast_tick\tsource\ttrigger
# \t\tstring\tdecayed\tlast_tick
\t\tivan.test@example.com\t1.990049833749168\t3
\t\tivan@work.example.org\t1\t2
";

#[test]
fn remembered_strings_export_and_import_as_shared_rows() {
    let mut ime = Ime::new();
    ime.type_in_english(EMAIL);
    ime.type_in_english("ivan@work.example.org");
    ime.type_in_english(EMAIL);
    assert_eq!(ime.user.export(), TWO_EMAILS);

    let user = UserModel::import(TWO_EMAILS, None).unwrap().user;
    assert_eq!(user.export(), TWO_EMAILS);
    let from_v2 = UserModel::import(TWO_EMAILS_V2, None).unwrap().user;
    assert_eq!(from_v2, user);
    let mut ime = Ime { user, ..Ime::new() };
    assert_eq!(
        ime.candidates("ivan")[1..3],
        [EMAIL, "ivan@work.example.org"]
    );
}

#[test]
fn a_row_that_is_not_an_email_is_an_error_with_its_line() {
    let header = "# qim-userdict v2\n";
    for row in [
        "\t\t台\t1\t1",
        "\t\tivan@localhost\t1\t1",
        "\t\tivan\\n@example.com\t1\t1",
    ] {
        let error = UserModel::import(&format!("{header}{row}\n"), None)
            .map(|i| i.user)
            .unwrap_err();
        assert_eq!(error.line, 2, "{row:?}");
    }
    let row = "\t\tivan@example.com\t1\t1\n";
    assert_eq!(
        UserModel::import(&format!("{header}{row}{row}"), None)
            .map(|i| i.user)
            .unwrap_err()
            .line,
        3,
        "重複"
    );
}

#[test]
fn a_remembered_string_can_be_forgotten() {
    let mut ime = Ime::new();
    ime.type_in_english(EMAIL);
    assert!(ime.user.forget_remembered_string(EMAIL));
    assert!(!ime.user.forget_remembered_string(EMAIL));
    assert!(ime.user.remembered_strings().is_empty());
    assert!(!ime.candidates("iva").contains(&EMAIL.to_string()));
    assert!(!ime.user.export().contains(EMAIL));
}

/// 英文候選（完整單字，規則 4）在第 2 位，記住的字串（開頭的字母，規則 5）排在它後面。
#[test]
fn an_english_word_stays_second_and_the_remembered_string_follows() {
    let mut ime = Ime::new();
    ime.engine = engine().with_english(EnglishPack {
        id: "english".into(),
        words: EnglishWords::new(["test"]),
    });
    ime.type_in_english("test.user@example.com");
    let shown = ime.candidates("test");
    assert_eq!(shown[1..3], ["test", "test.user@example.com"]);
}

/// 方案打不出的字母（速成沒有 `z`）同數字、符號跳過：`zz98765@example.org` 打 `exa` 起就有。
#[test]
fn letters_the_scheme_cannot_type_are_skipped() {
    let mut ime = Ime::new();
    ime.type_in_english("zz98765@example.org");
    assert_eq!(ime.candidates("exa")[1], "zz98765@example.org");
    ime.type_in_english("zeta.final@example.net");
    assert_eq!(ime.candidates("eta")[1], "zeta.final@example.net");
}

/// 輸入框裡原有（貼上、autofill）的字接著打完的 email 只看得到後半段：開頭不是使用者經 QIM 打的，不記
/// 半段。殼給的前文去掉正在打的那串後，最後一字還是 email 的字元時就是這樣。
#[test]
fn an_email_started_before_what_the_user_typed_is_not_remembered_in_part() {
    let mut ime = Ime::new();
    ime.press(Key::ToggleAscii(ShiftKey::Right));
    ime.session.set_context("ivan.");
    ime.type_text("test@example.com");
    ime.press(Key::Space);
    assert!(ime.user.remembered_strings().is_empty());
    // 之後照常記。
    ime.type_text(EMAIL);
    ime.press(Key::Space);
    assert_eq!(ime.user.remembered_strings(), vec![EMAIL.to_string()]);
}

/// 退格刪到正在打的那串前面（刪的是之前的字），之後打的接在不知道是什麼的字後面，不記。
#[test]
fn backspacing_past_the_start_of_the_typed_string_stops_remembering_it() {
    let mut ime = Ime::new();
    ime.press(Key::ToggleAscii(ShiftKey::Right));
    ime.type_text("x ");
    ime.press(Key::Backspace);
    ime.type_text("ivan@example.com");
    ime.press(Key::Space);
    assert!(ime.user.remembered_strings().is_empty());
}

/// 打超過上限後退格，留下的不是輸入框裡的樣子（超過的字沒接），不記。
#[test]
fn backspace_after_an_overlong_string_does_not_remember_the_truncated_rest() {
    let mut ime = Ime::new();
    let email = format!("{}@example.com", "a".repeat(241));
    assert_eq!(email.len(), 253);
    ime.press(Key::ToggleAscii(ShiftKey::Right));
    ime.type_text(&format!("{email}xyz"));
    ime.press(Key::Backspace);
    ime.press(Key::Space);
    assert!(ime.user.remembered_strings().is_empty());
}

/// 無痕或學習關掉時打的前半段，切回來後打完也不記：那串有一部分是不該記的時候打的。
#[test]
fn an_email_begun_while_not_learning_is_not_remembered_after_turning_learning_back_on() {
    for off in [
        Options {
            incognito: true,
            ..Options::default()
        },
        Options {
            learning: false,
            ..Options::default()
        },
    ] {
        let mut ime = Ime::with_options(off);
        ime.press(Key::ToggleAscii(ShiftKey::Right));
        ime.type_text("ivan.te");
        ime.session.set_options(Options::default());
        ime.type_text("st@example.com");
        ime.press(Key::Space);
        assert!(ime.user.remembered_strings().is_empty(), "{off:?}");
    }
}

/// email 不分大小寫（網域本來就不分，本地部分實務上也不分）：只記一條，留最後打的寫法。
#[test]
fn the_same_email_in_another_case_replaces_the_old_one() {
    let mut ime = Ime::new();
    ime.type_in_english("Ivan.Test@Example.com");
    ime.type_in_english(EMAIL);
    assert_eq!(ime.user.remembered_strings(), vec![EMAIL.to_string()]);
}

/// 最多記 [`REMEMBERED_STRINGS_MAX`] 條：每多記一條，中文打字撞上它開頭的機會就多一份。超過時淘汰最久
/// 沒用（打或選）的；用過的舊地址留著。
#[test]
fn beyond_the_limit_the_least_recently_used_email_is_dropped() {
    let mut ime = Ime::new();
    let email = |i: usize| format!("user{i}@example.com");
    for i in 0..REMEMBERED_STRINGS_MAX {
        ime.type_in_english(&email(i));
    }
    // 再打一次最舊的，它就是最近用過的。
    ime.type_in_english(&email(0));
    ime.type_in_english("new@example.com");
    let kept = ime.user.remembered_strings();
    assert_eq!(kept.len(), REMEMBERED_STRINGS_MAX);
    assert!(kept.contains(&email(0)));
    assert!(kept.contains(&"new@example.com".to_string()));
    assert!(!kept.contains(&email(1)), "最久沒用的是 user1");
}

/// 匯入的檔超過上限時整份不收（匯出的檔不會超過），錯誤帶行號。
#[test]
fn an_import_with_more_than_the_limit_is_an_error() {
    let mut text = String::from("# qim-userdict v2\n");
    for i in 0..=REMEMBERED_STRINGS_MAX {
        text.push_str(&format!("\t\tuser{i}@example.com\t1\t{}\n", i + 1));
    }
    let error = UserModel::import(&text, None).map(|i| i.user).unwrap_err();
    assert_eq!(error.line, REMEMBERED_STRINGS_MAX + 2);
}
