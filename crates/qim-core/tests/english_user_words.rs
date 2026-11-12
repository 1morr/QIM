//! 英文使用者詞（M4 票 14，plan §2.1、§2.4，取代 M3 的記住的字串）：核心看得到送出去的英文串都學，
//! 4 個字元起；英文詞典沒有的串一年內打過 2 次，中文組字打完整串時它排第 2（規則 4），打過 3 次又是
//! 最常用的 20 條之一時，打它開頭的字母也給（前綴補完，規則 5）。無痕、學習關掉時不學。

mod common;

use std::collections::BTreeMap;

use qim_core::{
    CandidateKind, CodeEntry, CodeTable, ENGLISH_COMPLETION_LIMIT, ENGLISH_COMPLETION_POOL, Engine,
    EnglishPack, EnglishWords, Key, Limits, Options, Packs, Ranking, Scheme, SchemePack, Session,
    ShiftKey, Snapshot, UserModel,
};

const EMAIL: &str = "ivan.test@example.com";

/// 每個字母都是一個字的碼（`a` 日、`b` 月…），所以任何字母串都有中文候選。
fn engine() -> Engine {
    engine_with(None)
}

/// 同 [`engine`]，萬用鍵是 `wildcard`。
fn engine_with(wildcard: Option<char>) -> Engine {
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        wildcard,
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

/// 掛上一份只有 `words` 的英文詞典。
fn with_dictionary(words: &[&str]) -> Engine {
    engine().with_english(EnglishPack {
        id: "english".into(),
        words: EnglishWords::new(words.iter().copied()),
    })
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

    /// 英文直通打 `text` `times` 次。
    fn type_in_english_times(&mut self, text: &str, times: usize) {
        for _ in 0..times {
            self.type_in_english(text);
        }
    }

    /// 英文直通打 `text`，不按空白，停在英文直通。
    fn type_in_english_without_ending(&mut self, text: &str) {
        self.press(Key::ToggleAscii(ShiftKey::Right));
        self.type_text(text);
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

    /// 中文打 `code`，回傳 debug 面板看到的候選：文字、種類、排定它的規則；不上屏。
    fn inspected(&mut self, code: &str) -> Vec<(String, CandidateKind, Option<u8>)> {
        self.type_text(code);
        let all = self
            .session
            .inspect(&self.engine, &self.user)
            .candidates
            .into_iter()
            .map(|c| (c.text, c.kind, c.rule))
            .collect();
        self.press(Key::Escape);
        all
    }

    fn offers(&mut self, code: &str, text: &str) -> bool {
        self.candidates(code).iter().any(|t| t == text)
    }

    fn learned(&self) -> Vec<String> {
        self.user.english_user_words()
    }
}

/// 詞典沒有的串打過 2 次才出：打一次時中文組字打整串沒有它，打第二次後排第 2（規則 4），debug 面板標
/// 「英文使用者詞」。
#[test]
fn a_string_not_in_the_dictionary_shows_after_it_is_typed_twice() {
    let mut ime = Ime::new();
    ime.type_in_english("rrrrrr");
    assert_eq!(ime.learned(), ["rrrrrr"], "沒到門檻也學");
    assert!(!ime.offers("rrrrrr", "rrrrrr"));

    ime.type_in_english("rrrrrr");
    let shown = ime.inspected("rrrrrr");
    assert_eq!(
        shown[1],
        ("rrrrrr".into(), CandidateKind::EnglishUserWord, Some(4)),
        "{shown:?}"
    );
    ime.type_text("rrrrrr");
    assert_eq!(ime.press(Key::Digit(2)).commit.as_deref(), Some("rrrrrr"));
}

/// 4 個字元起才學：3 個字元的打幾次都不學。
#[test]
fn strings_shorter_than_four_characters_are_not_learned() {
    let mut ime = Ime::new();
    ime.type_in_english_times("rrr", 3);
    ime.type_in_english("abcd");
    assert_eq!(ime.learned(), ["abcd"]);
}

/// 數字、電話、卡號照學（不另認 email 這類字串）；組字裡打不出數字，所以不會出在中文組字。
#[test]
fn digit_strings_are_learned_too() {
    let mut ime = Ime::new();
    ime.type_in_english("12345678");
    ime.type_in_english("2345-6789");
    assert_eq!(ime.learned(), ["12345678", "2345-6789"]);
}

/// 句尾的句點與引號不算在串裡：`hello.` 學的是 `hello`。
#[test]
fn a_full_stop_or_quote_around_the_string_is_left_out() {
    let mut ime = Ime::new();
    ime.type_in_english("hello.");
    ime.type_in_english("'world'");
    ime.type_in_english("don't");
    assert_eq!(ime.learned(), ["don't", "hello", "world"]);
}

/// 只有方案打得出的字母組成的串才出在完整串（規則 4）：email 要靠前綴補完。
#[test]
fn only_strings_of_scheme_letters_show_as_the_whole_input() {
    let mut ime = Ime::new();
    ime.type_in_english_times("ab.cd", 2);
    assert!(!ime.offers("abcd", "ab.cd"));
}

/// 中文組字的前綴補完要打過 3 次：2 次時打開頭 3 碼沒有它，第 3 次後排第 2（規則 5），選了上屏整串。
#[test]
fn prefix_completion_needs_three_times() {
    let mut ime = Ime::new();
    ime.type_in_english_times(EMAIL, 2);
    assert!(!ime.offers("iva", EMAIL));

    ime.type_in_english(EMAIL);
    let shown = ime.inspected("iva");
    assert_eq!(
        shown[1],
        (EMAIL.into(), CandidateKind::EnglishUserWord, Some(5)),
        "{shown:?}"
    );
    ime.type_text("iva");
    assert_eq!(ime.press(Key::Digit(2)).commit.as_deref(), Some(EMAIL));
}

/// 前綴補完只給最常用的 [`ENGLISH_COMPLETION_POOL`] 條：誤觸率逐條累加。超出的照樣學、照樣出在完整串。
#[test]
fn prefix_completion_offers_only_the_most_used_twenty() {
    let mut ime = Ime::new();
    // 各條開頭 3 碼不同（`aaaword`、`bbbword`…），打開頭 3 碼只對得上一條。
    let word = |i: usize| format!("{}word", char::from(b'a' + i as u8).to_string().repeat(3));
    for i in 0..=ENGLISH_COMPLETION_POOL {
        // 第一個打 3 次，其他各 4 次：第一個最少用。
        let times = if i == 0 { 3 } else { 4 };
        ime.type_in_english_times(&word(i), times);
    }
    assert_eq!(ime.learned().len(), ENGLISH_COMPLETION_POOL + 1);
    assert!(!ime.offers("aaa", &word(0)), "最少用的不在 20 條內");
    assert!(ime.offers("bbb", &word(1)));
    assert!(ime.offers(&word(0), &word(0)), "完整串照樣出");
}

/// 英文詞典有的詞不進前綴補完、不佔 20 條的名額（協調者 2026-11-12 決定）：常打 `that`、`have`、`with`
/// 這類日常單字之後，常寄的 email 照樣在補完裡；詞典詞走規則 4 的完整串。
#[test]
fn dictionary_words_do_not_crowd_the_email_out_of_completion() {
    let mut ime = Ime::new();
    let mut dictionary: Vec<String> = vec!["that".into(), "have".into(), "with".into()];
    dictionary.extend(
        (0..ENGLISH_COMPLETION_POOL).map(|i| format!("word{}", char::from(b'a' + i as u8))),
    );
    let refs: Vec<&str> = dictionary.iter().map(String::as_str).collect();
    ime.engine = with_dictionary(&refs);
    ime.type_in_english_times(EMAIL, 3);
    for word in &dictionary {
        ime.type_in_english_times(word, 5);
    }
    assert_eq!(ime.candidates("iva")[1], EMAIL);
    assert!(!ime.offers("tha", "that"), "詞典詞不給前綴補完");
    assert_eq!(ime.candidates("that")[1], "that", "詞典詞照規則 4");
}

/// 1–2 碼是一個字的碼，第 2 位最常用，不給（同英文候選，速成 3 碼起）；之後比的是字串的英文字母，
/// 打不出的 `.`、`@` 跳過，所以打到網域也還對得上。
#[test]
fn completion_needs_more_letters_than_one_character_code_and_matches_the_letters_only() {
    let mut ime = Ime::new();
    ime.type_in_english_times(EMAIL, 3);
    for code in ["i", "iv"] {
        assert!(!ime.offers(code, EMAIL), "{code}");
    }
    for code in ["iva", "ivan", "ivantest", "ivantestex"] {
        assert_eq!(ime.candidates(code)[1], EMAIL, "{code}");
    }
    for code in ["ivb", "ivanb"] {
        assert!(!ime.offers(code, EMAIL), "{code}");
    }
}

/// 幾個都對得上時最近常打的在前，最多 [`ENGLISH_COMPLETION_LIMIT`] 個，依序從第 2 位起；首選照舊是中文。
#[test]
fn several_completions_follow_the_first_candidate_most_recent_first() {
    let mut ime = Ime::new();
    let emails = [
        "ivan.a@example.com",
        "ivan.b@example.com",
        "ivan.c@example.com",
        "ivan.d@example.com",
    ];
    for email in emails {
        ime.type_in_english_times(email, 3);
    }
    let shown = ime.candidates("ivan");
    assert_eq!(shown[0], "戈女日弓");
    assert_eq!(
        shown[1..=ENGLISH_COMPLETION_LIMIT],
        [
            "ivan.d@example.com",
            "ivan.c@example.com",
            "ivan.b@example.com"
        ]
    );
    assert!(!shown.contains(&emails[0].to_string()));
}

/// 選了前綴補完上屏，接著打的空白讓那串打完，同自己打一次：它成了最近常打的。
#[test]
fn choosing_a_completion_counts_as_typing_it() {
    let mut ime = Ime::new();
    ime.type_in_english_times("ivan.a@example.com", 3);
    ime.type_in_english_times("ivan.b@example.com", 3);
    ime.type_text("ivan");
    ime.press(Key::Digit(3));
    ime.press(Key::Space);
    assert_eq!(ime.candidates("ivan")[1], "ivan.a@example.com");
}

/// 詞典有的詞不受門檻：打一次也照詞典排第 2（種類是英文），寫法照詞典；學習不讓它多出一個候選。
#[test]
fn a_dictionary_word_is_not_held_back_by_the_threshold() {
    let mut ime = Ime::new();
    ime.engine = with_dictionary(&["receive"]);
    ime.type_in_english("Receive");
    let shown = ime.inspected("receive");
    assert_eq!(
        shown[1],
        ("receive".into(), CandidateKind::English, Some(4)),
        "{shown:?}"
    );
    assert_eq!(shown.iter().filter(|c| c.0 == "Receive").count(), 0);
}

/// 詞典有的詞只照「英文候選」由詞典出：關掉英文候選時，打過兩次也不靠學習冒出來。
#[test]
fn a_dictionary_word_does_not_show_through_learning_when_english_candidates_are_off() {
    let mut ime = Ime::new();
    ime.engine = with_dictionary(&["receive"]);
    ime.type_in_english_times("receive", 2);
    ime.session.set_options(Options {
        english_candidates: false,
        ..Options::default()
    });
    assert!(!ime.offers("receive", "receive"));
}

/// 英文候選（完整單字，規則 4）在第 2 位，前綴補完（規則 5）排在它後面。
#[test]
fn an_english_word_stays_second_and_the_completion_follows() {
    let mut ime = Ime::new();
    ime.engine = with_dictionary(&["test"]);
    ime.type_in_english_times("test.user@example.com", 3);
    let shown = ime.candidates("test");
    assert_eq!(shown[1..3], ["test", "test.user@example.com"]);
}

/// 方案打不出的字母（速成沒有 `z`）同數字、符號跳過：`zz98765@example.org` 打 `exa` 起就有。
#[test]
fn letters_the_scheme_cannot_type_are_skipped() {
    let mut ime = Ime::new();
    ime.type_in_english_times("zz98765@example.org", 3);
    assert_eq!(ime.candidates("exa")[1], "zz98765@example.org");
}

/// 英文使用者詞只比方案字母：方案設了萬用鍵時，前綴補完略過輸入碼裡的 `z`（萬用鍵以前在組字中
/// 吃下不動作），打 `lizzy` 照樣補出 `lizzy@example.com`（M4 票 11）。
#[test]
fn the_wildcard_is_skipped_when_matching_a_completion() {
    let email = "lizzy@example.com";
    let mut ime = Ime::new();
    ime.engine = engine_with(Some('z'));
    ime.type_in_english_times(email, 3);
    let shown = ime.candidates("lizzy");
    assert_eq!(shown.get(1).map(String::as_str), Some(email), "{shown:?}");
}

/// 含萬用鍵的輸入碼在中文組字是比對模式，不出整串的英文使用者詞（同英文詞典的候選，plan §4.4）：
/// 打過兩次的 `lizzy` 打 `lizzy` 時不出，`z` 照萬用鍵比對中文。
#[test]
fn an_input_with_the_wildcard_is_not_an_english_user_word() {
    let mut ime = Ime::new();
    ime.engine = engine_with(Some('z'));
    ime.type_in_english_times("lizzy", 2);
    let shown = ime.candidates("lizzy");
    assert!(!shown.iter().any(|c| c == "lizzy"), "{shown:?}");
    assert!(!shown.is_empty(), "萬用鍵照樣比對中文");
}

#[test]
fn incognito_does_not_learn() {
    let mut ime = Ime::with_options(Options {
        incognito: true,
        ..Options::default()
    });
    ime.type_in_english_times(EMAIL, 3);
    assert!(ime.learned().is_empty());
}

#[test]
fn learning_off_does_not_learn_but_still_offers_what_was_learned() {
    let mut ime = Ime::new();
    ime.type_in_english_times(EMAIL, 3);
    ime.session.set_options(Options {
        learning: false,
        ..Options::default()
    });
    ime.type_in_english("ivan@work.example.org");
    assert_eq!(ime.learned(), [EMAIL]);
    assert_eq!(ime.candidates("iva")[1], EMAIL);
}

#[test]
fn the_option_off_neither_learns_nor_offers() {
    let mut ime = Ime::new();
    ime.type_in_english_times(EMAIL, 3);
    ime.type_in_english_times("rrrrrr", 2);
    ime.session.set_options(Options {
        english_user_words: false,
        ..Options::default()
    });
    ime.type_in_english("ivan@work.example.org");
    assert_eq!(ime.learned(), [EMAIL, "rrrrrr"]);
    assert!(!ime.offers("iva", EMAIL));
    assert!(!ime.offers("rrrrrr", "rrrrrr"));
}

/// 學習的時限（M4 票 13）：一年沒打的停用、不刪，日期改回來又出現；隔一年再打，次數從 1 重算。
#[test]
fn a_word_not_typed_for_a_year_is_not_offered_but_kept_and_counts_again_from_one() {
    let mut ime = Ime::new();
    ime.user.set_today("2026-01-01".parse().unwrap());
    ime.type_in_english_times("rrrrrr", 2);
    ime.user.set_today("2027-01-02".parse().unwrap());
    assert!(!ime.offers("rrrrrr", "rrrrrr"));
    assert_eq!(ime.learned(), ["rrrrrr"]);
    ime.user.set_today("2027-01-01".parse().unwrap());
    assert!(ime.offers("rrrrrr", "rrrrrr"));

    ime.user.set_today("2027-01-02".parse().unwrap());
    ime.type_in_english("rrrrrr");
    assert!(!ime.offers("rrrrrr", "rrrrrr"), "隔一年從 1 重算");
    ime.type_in_english("rrrrrr");
    assert!(ime.offers("rrrrrr", "rrrrrr"));
}

/// 不分大小寫：只記一條、次數合計，留最後打的寫法。
#[test]
fn the_same_string_in_another_case_counts_as_the_same_word() {
    let mut ime = Ime::new();
    ime.type_in_english("Rrrrrr");
    ime.type_in_english("RRRRRR");
    assert_eq!(ime.learned(), ["RRRRRR"]);
    assert_eq!(ime.candidates("rrrrrr")[1], "RRRRRR");
}

#[test]
fn an_english_user_word_can_be_forgotten() {
    let mut ime = Ime::new();
    ime.type_in_english_times(EMAIL, 3);
    assert!(ime.user.forget_english_user_word(EMAIL));
    assert!(!ime.user.forget_english_user_word(EMAIL));
    assert!(ime.learned().is_empty());
    assert!(!ime.offers("iva", EMAIL));
    // 匯出檔只剩墓碑（M4 票 17）：鍵的雜湊、各台打過的次數（刪除時都記成不算的）與刪除時的合計，不留原文。
    let exported = ime.user.export();
    assert!(!exported.contains(EMAIL), "{exported}");
    let tombstone = exported
        .lines()
        .find(|l| l.starts_with("deleted\tenglish\t\t"))
        .map(|l| l.split('\t').skip(4).collect::<Vec<_>>());
    assert_eq!(tombstone, Some(vec!["0:3:3:", "3", ""]), "{exported}");
}

/// 撤銷學習（M4 票 16）：組字中清掉英文使用者詞的候選（規則 4 或前綴補完）就是忘掉那個英文使用者詞。
#[test]
fn forgetting_an_english_user_word_candidate_forgets_the_word() {
    let mut ime = Ime::new();
    ime.type_in_english_times("rrrrrr", 2);
    ime.type_in_english_times(EMAIL, 3);
    for (code, text) in [("rrrrrr", "rrrrrr"), ("iva", EMAIL)] {
        ime.type_text(code);
        let at = ime
            .session
            .all_candidates()
            .iter()
            .position(|c| c.text == text);
        let forgotten = ime.session.forget(&ime.engine, &mut ime.user, at.unwrap());
        assert!(!forgotten.candidates.iter().any(|c| c.text == text));
        ime.press(Key::Escape);
        assert!(!ime.offers(code, text));
    }
    assert!(ime.learned().is_empty());
}

/// 用自己的 tick：學英文使用者詞不推進個人 bigram 的 tick（`#@/tick`）。
#[test]
fn english_user_words_have_their_own_tick() {
    let mut ime = Ime::new();
    ime.type_in_english_times("rrrrrr", 2);
    let exported = ime.user.export();
    assert!(!exported.contains("#@/tick\t"), "{exported}");
    assert!(exported.contains("#@/english_tick\t2\n"), "{exported}");
}

/// 匯出 v3：種類 `english` 的列，含各台打過的次數（「編號:打過:不算的:最後打的日期」）；沒到門檻的照樣匯出
/// （同步才湊得到兩台各打一次）。
const TWO_WORDS: &str = "\
# qim-userdict v3
#@/device/0\t
#@/english_tick\t3
# kind\tscheme\tword\tcode\tcounts\tdeleted\tdeleted_on\tversion\tdecayed\tlast_tick\tlast_commit_tick\tlast_used\tsource\ttrigger\tword_version\tword_last_used
# kind\tword\tcounts\tdeleted\tdeleted_on\tversion\tdecayed\tlast_tick\tlast_used
english\tivan.test@example.com\t0:2:0:\t\t\t2\t1.990049833749168\t3\t
english\trrrrrr\t0:1:0:\t\t\t1\t1\t2\t
";

#[test]
fn english_user_words_export_and_import_with_their_counts() {
    let mut ime = Ime::new();
    ime.type_in_english(EMAIL);
    ime.type_in_english("rrrrrr");
    ime.type_in_english(EMAIL);
    assert_eq!(ime.user.export(), TWO_WORDS);

    let user = UserModel::import(TWO_WORDS, None).unwrap().user;
    assert_eq!(user.export(), TWO_WORDS);
    let mut ime = Ime { user, ..Ime::new() };
    ime.type_in_english("rrrrrr");
    assert!(ime.offers("rrrrrr", "rrrrrr"), "匯入的次數接著算");
}

/// v2（M3 票 23 定稿，固定）的記住的字串：方案欄、前一個字詞欄都空的列，匯入成已過門檻的英文使用者詞，
/// 打開頭 3 碼就有。
const TWO_EMAILS_V2: &str = "\
# qim-userdict v2
#@/tick\t3
# scheme\tword\tcode\tcommits\tlast_commit_tick\tdecayed\tlast_tick\tsource\ttrigger
# \t\tstring\tdecayed\tlast_tick
\t\tivan.test@example.com\t1.990049833749168\t3
\t\tivan@work.example.org\t1\t2
";

#[test]
fn remembered_strings_from_v2_import_as_english_user_words_past_the_threshold() {
    let user = UserModel::import(TWO_EMAILS_V2, None).unwrap().user;
    let mut ime = Ime { user, ..Ime::new() };
    assert_eq!(ime.learned(), [EMAIL, "ivan@work.example.org"]);
    assert_eq!(
        ime.candidates("ivan")[1..3],
        [EMAIL, "ivan@work.example.org"]
    );
    let exported = ime.user.export();
    assert!(
        exported.contains("english\tivan@work.example.org\t0:3:0:\t\t\t3\t1\t2\t\n"),
        "{exported}"
    );
    assert!(exported.contains("#@/english_tick\t3\n"), "{exported}");
}

#[test]
fn a_row_that_is_not_an_english_user_word_is_an_error_with_its_line() {
    let header = "# qim-userdict v3\n#@/device/0\t\n";
    let good = "english\tabcd\t0:1:0:\t\t\t1\t1\t1\t";
    assert!(UserModel::import(&format!("{header}{good}\n"), None).is_ok());
    for row in [
        good.replace("abcd", "台灣台灣"),
        good.replace("abcd", "abc"),
        good.replace("abcd", "ab cd"),
        good.replace("abcd", "abcd."),
        good.replace("0:1:0:", "0:1:2:"),   // 不算的比打過的多
        good.replace("0:1:0:", "0:1:0:0:"), // 英文的格子是「編號:打過:不算的:日期」
        good.replace("\t1\t1\t1\t", "\t1\t0\t1\t"), // 衰減值要是正的
        good.replace("\t1\t1\t1\t", "\t2\t1\t1\t"), // 版本比打過的多
        good.replace("\t1\t1\t1\t", "\t1\t1\t1"), // 少一欄
    ] {
        let error = UserModel::import(&format!("{header}{row}\n"), None)
            .map(|i| i.user)
            .unwrap_err();
        assert_eq!(error.line, 3, "{row:?}");
    }
    let rows = format!("{good}\n{}\n", good.replace("abcd", "ABCD"));
    assert_eq!(
        UserModel::import(&format!("{header}{rows}"), None)
            .map(|i| i.user)
            .unwrap_err()
            .line,
        4,
        "不分大小寫重複"
    );
    let row = "\t\tivan@localhost\t1\t1\n";
    assert!(
        UserModel::import(&format!("# qim-userdict v2\n{row}"), None).is_ok(),
        "v2 不再要求是 email"
    );
}

/// 上限淘汰（M4 票 13 的 `Eviction`）先刪沒到出現門檻的英文使用者詞，即使中文條目更舊；英文使用者詞和
/// （方案, 輸入碼, 詞）的列算同一個上限。
#[test]
fn eviction_drops_english_user_words_below_the_threshold_first() {
    let mut ime = Ime::new();
    ime.user.set_limits(Limits {
        entries: 5,
        ..Limits::default()
    });
    ime.user.set_today("2026-01-01".parse().unwrap());
    for code in ["a", "b", "c"] {
        ime.type_text(code);
        ime.press(Key::Space);
    }
    ime.user.set_today("2026-06-01".parse().unwrap());
    ime.type_in_english_times("aaaa", 2);
    ime.type_in_english("bbbb");
    assert_eq!(ime.learned(), ["aaaa", "bbbb"], "5 條，還沒到 1.2 倍");
    ime.type_in_english("cccc");
    assert_eq!(
        ime.learned(),
        ["aaaa", "cccc"],
        "到 6 條清到 5：最舊的沒到門檻的"
    );
    ime.type_in_english("dddd");
    assert_eq!(ime.learned(), ["aaaa", "dddd"]);
    let exported = ime.user.export();
    assert_eq!(exported.matches("entry\tquick\t").count(), 3, "{exported}");
}

/// 打完直接點別的欄位（殼呼叫 `clear_context`）也算打完，登入表單常這樣。
#[test]
fn leaving_the_field_ends_the_string() {
    let mut ime = Ime::new();
    ime.type_in_english_without_ending(EMAIL);
    assert!(ime.learned().is_empty(), "還在打");
    ime.session.clear_context(&mut ime.user);
    assert_eq!(ime.learned(), [EMAIL]);
}

#[test]
fn enter_and_tab_end_the_string() {
    for key in [Key::Enter, Key::Other] {
        let mut ime = Ime::new();
        ime.type_in_english_without_ending(EMAIL);
        ime.press(key.clone());
        assert_eq!(ime.learned(), [EMAIL], "{key:?}");
    }
}

/// 交還宿主的退格刪掉最後一個字元：打錯改過的照記改過的。
#[test]
fn backspace_removes_the_last_character_typed() {
    let mut ime = Ime::new();
    ime.type_in_english_without_ending("ivan.test@example.comm");
    ime.press(Key::Backspace);
    ime.press(Key::Space);
    assert_eq!(ime.learned(), [EMAIL]);
}

/// 殼給的前文不以正在打的那串結尾（游標移走、宿主改了字）：那串不一定是輸入框裡的樣子，不學。
#[test]
fn a_context_that_does_not_end_with_the_typed_string_drops_it() {
    let mut ime = Ime::new();
    ime.type_in_english_without_ending(EMAIL);
    ime.session.set_context(&format!("Hi {EMAIL}"));
    ime.press(Key::Space);
    assert_eq!(ime.learned(), [EMAIL]);

    let mut ime = Ime::new();
    ime.type_in_english_without_ending(EMAIL);
    ime.session.set_context("Hi ");
    ime.press(Key::Space);
    assert!(ime.learned().is_empty());
}

/// 中文組字裡單按左 Shift 接英文（臨時英文）打完再 Enter 上屏，也學；上屏中文接在後面同樣讓它打完。
#[test]
fn a_string_finished_in_temporary_english_is_learned() {
    let mut ime = Ime::new();
    ime.type_text("iv");
    ime.press(Key::ToggleAscii(ShiftKey::Left));
    ime.type_text("an.test@example.com");
    let done = ime.press(Key::Enter);
    assert_eq!(done.commit.as_deref(), Some(EMAIL));
    ime.type_text("a");
    ime.press(Key::Space);
    assert_eq!(ime.learned(), [EMAIL]);
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
    assert_eq!(ime.learned(), [EMAIL]);
}

/// 輸入框裡原有（貼上、autofill）的字接著打完的串只看得到後半段：開頭不是使用者經 QIM 打的，不學半段。
#[test]
fn a_string_started_before_what_the_user_typed_is_not_learned_in_part() {
    let mut ime = Ime::new();
    ime.press(Key::ToggleAscii(ShiftKey::Right));
    ime.session.set_context("ivan.");
    ime.type_text("test@example.com");
    ime.press(Key::Space);
    assert!(ime.learned().is_empty());
    ime.type_text(EMAIL);
    ime.press(Key::Space);
    assert_eq!(ime.learned(), [EMAIL]);
}

/// 退格刪到正在打的那串前面（刪的是之前的字），之後打的接在不知道是什麼的字後面，不學。
#[test]
fn backspacing_past_the_start_of_the_typed_string_stops_learning_it() {
    let mut ime = Ime::new();
    ime.press(Key::ToggleAscii(ShiftKey::Right));
    ime.type_text("x ");
    ime.press(Key::Backspace);
    ime.type_text("ivan@example.com");
    ime.press(Key::Space);
    assert!(ime.learned().is_empty());
}

/// 打超過上限後退格，留下的不是輸入框裡的樣子（超過的字沒接），不學。
#[test]
fn backspace_after_an_overlong_string_does_not_learn_the_truncated_rest() {
    let mut ime = Ime::new();
    let long = format!("{}@example.com", "a".repeat(241));
    assert_eq!(long.len(), 253);
    ime.press(Key::ToggleAscii(ShiftKey::Right));
    ime.type_text(&format!("{long}xyz"));
    ime.press(Key::Backspace);
    ime.press(Key::Space);
    assert!(ime.learned().is_empty());
}

/// 無痕或學習關掉時打的前半段，切回來後打完也不學：那串有一部分是不該學的時候打的。
#[test]
fn a_string_begun_while_not_learning_is_not_learned_after_turning_learning_back_on() {
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
        assert!(ime.learned().is_empty(), "{off:?}");
    }
}
