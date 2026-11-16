//! 英文模式（ADR 0008，M4 票 24）：觸控的地球鍵切到的英文狀態。字母、數字、標點與空白核心吃下、逐字上屏，
//! 建議列是正在打的串的前綴補完（英文詞典加英文使用者詞，照使用頻率排）；點建議用 `delete_before` 換掉
//! 那個詞。實體鍵盤單按 Shift 照舊是英文直通。

mod common;

use std::collections::BTreeMap;

use qim_core::{
    CandidateKind, CodeEntry, CodeTable, Engine, EnglishPack, EnglishWords, Key, Options, Packs,
    Ranking, Scheme, SchemePack, Session, ShiftKey, Snapshot, UserModel,
};

/// 每個字母都是一個字的碼（`a` 日、`b` 月…），掛上一份小英文詞典（詞, 頻率）。
fn engine(words: &[(&str, u8)]) -> Engine {
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        wildcard: None,
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
    Engine::new(SchemePack { scheme, code_table }, None, Some(model))
        .unwrap()
        .with_english(EnglishPack {
            id: "english".into(),
            words: EnglishWords::new(words.iter().copied()),
        })
}

/// 測試用的詞典：`hello` 比 `help` 常見，`hel` 不是詞。
const WORDS: &[(&str, u8)] = &[
    ("hello", 200),
    ("help", 180),
    ("helmet", 120),
    ("held", 150),
    ("the", 250),
    ("them", 220),
    ("NASA", 100),
    ("zebra", 90),
];

struct Ime {
    engine: Engine,
    session: Session,
    user: UserModel,
}

impl Ime {
    fn new() -> Self {
        Self {
            engine: engine(WORDS),
            session: Session::default(),
            user: UserModel::default(),
        }
    }

    fn press(&mut self, key: Key) -> Snapshot {
        self.session.handle(&self.engine, &mut self.user, key)
    }

    /// 觸控的地球鍵。
    fn globe(&mut self) -> Snapshot {
        self.press(Key::ToggleEnglish)
    }

    /// 逐字打 `text`：字母、數字、標點、空白各轉成它的鍵；回傳每個鍵的結果。
    fn type_text(&mut self, text: &str) -> Vec<Snapshot> {
        text.chars()
            .map(|c| {
                let key = match c {
                    ' ' => Key::Space,
                    c if c.is_ascii_digit() => Key::Digit(c.to_digit(10).unwrap() as u8),
                    c if c.is_ascii_alphabetic() => Key::Letter(c),
                    c => Key::Punct(c),
                };
                self.press(key)
            })
            .collect()
    }

    /// 打 `text`，回傳最後一個鍵之後的建議列（全部，不分頁）。
    fn suggestions(&mut self, text: &str) -> Vec<String> {
        self.type_text(text);
        self.all()
    }

    fn all(&self) -> Vec<String> {
        self.session
            .all_candidates()
            .into_iter()
            .map(|c| c.text)
            .collect()
    }
}

/// 地球鍵進英文模式：`ascii` 為真、沒有組字；字母、數字、標點與空白都吃下、當場上屏。
#[test]
fn the_globe_key_enters_english_mode_where_every_character_is_committed_at_once() {
    let mut ime = Ime::new();
    let entered = ime.globe();
    assert!(entered.handled && entered.ascii);
    let typed = ime.type_text("Hi, 2u");
    let commits: Vec<_> = typed.iter().map(|s| s.commit.as_deref()).collect();
    assert_eq!(
        commits,
        [
            Some("H"),
            Some("i"),
            Some(","),
            Some(" "),
            Some("2"),
            Some("u")
        ]
    );
    assert!(typed.iter().all(|s| s.handled && s.ascii));
    assert!(typed.iter().all(|s| s.composition.input.is_empty()));
}

/// 實體鍵盤單按 Shift 照舊是英文直通：鍵交還宿主、沒有建議。英文直通或英文模式按任一種切換鍵都回中文。
#[test]
fn a_lone_shift_still_switches_to_pass_through_and_any_switch_key_returns_to_chinese() {
    let mut ime = Ime::new();
    ime.press(Key::ToggleAscii(ShiftKey::Right));
    let typed = ime.type_text("hel");
    assert!(typed.iter().all(|s| !s.handled && s.ascii));
    assert!(ime.all().is_empty());

    let back = ime.globe();
    assert!(back.handled && !back.ascii, "英文直通按地球鍵回中文");
    ime.globe();
    let back = ime.press(Key::ToggleAscii(ShiftKey::Left));
    assert!(!back.ascii, "英文模式按 Shift 回中文");
    assert!(
        ime.press(Key::Letter('a')).composition.input == "a",
        "回到中文組字"
    );
}

/// 組字中按地球鍵同單按右 Shift（預設先上屏整個組字），之後是英文模式。
#[test]
fn the_globe_key_while_composing_commits_the_composition_then_enters_english_mode() {
    let mut ime = Ime::new();
    ime.type_text("ab");
    let switched = ime.globe();
    assert_eq!(switched.commit.as_deref(), Some("日月"));
    assert!(switched.ascii && switched.composition.input.is_empty());
    assert_eq!(ime.press(Key::Letter('h')).commit.as_deref(), Some("h"));
}

/// `Key::parse` 讀得到地球鍵（網頁、Swift 經它送鍵）。
#[test]
fn the_globe_key_parses_from_its_kind() {
    assert_eq!(Key::parse("toggleEnglish", None), Ok(Key::ToggleEnglish));
    assert!(Key::parse("toggleEnglish", Some("x")).is_err());
}

/// 建議列是正在打的串的前綴補完：英文詞典的詞照頻率排；打的就是那個詞時不列它自己（選了也不變）。
#[test]
fn suggestions_complete_the_typed_word_from_the_dictionary_by_frequency() {
    let mut ime = Ime::new();
    ime.globe();
    assert_eq!(ime.suggestions("hel"), ["hello", "help", "held", "helmet"]);
    assert_eq!(ime.suggestions("p"), Vec::<String>::new(), "help 打完了");
    let mut ime = Ime::new();
    ime.globe();
    assert_eq!(ime.suggestions("the"), ["them"]);
    assert_eq!(
        ime.suggestions(" "),
        Vec::<String>::new(),
        "空白之後不預測下一個詞"
    );
}

/// 英文模式上屏的串學成英文使用者詞；詞典沒有的打過 2 次才出在建議列（出現門檻），debug 面板標「英文使用者詞」。
#[test]
fn a_string_not_in_the_dictionary_is_suggested_after_it_is_typed_twice() {
    let mut ime = Ime::new();
    ime.globe();
    ime.type_text("rrrrrr ");
    assert_eq!(ime.user.english_user_words(), ["rrrrrr"]);
    assert!(!ime.suggestions("rrr").contains(&"rrrrrr".to_string()));
    ime.type_text("rrr ");
    assert!(ime.suggestions("rrr").contains(&"rrrrrr".to_string()));
    let kinds: Vec<_> = ime
        .session
        .inspect(&ime.engine, &ime.user)
        .candidates
        .into_iter()
        .map(|c| (c.text, c.kind))
        .collect();
    assert_eq!(
        kinds,
        [("rrrrrr".to_string(), CandidateKind::EnglishUserWord)]
    );
}

/// 詞典有的詞不受門檻管：打過一次就排到沒打過的詞前面，學得多的在前。
#[test]
fn learned_dictionary_words_come_before_the_rest_and_the_most_used_first() {
    let mut ime = Ime::new();
    ime.globe();
    ime.type_text("helmet held held ");
    assert_eq!(ime.suggestions("hel"), ["held", "helmet", "hello", "help"]);
}

/// 英文使用者詞關掉時不學也不給，詞典照樣補完。
#[test]
fn with_english_user_words_off_only_the_dictionary_completes() {
    let mut ime = Ime::new();
    ime.globe();
    ime.type_text("helmet helmet rrrrrr rrrrrr ");
    ime.session.set_options(Options {
        english_user_words: false,
        ..Options::default()
    });
    assert_eq!(ime.suggestions("hel"), ["hello", "help", "held", "helmet"]);
    assert!(ime.suggestions(" rrr").is_empty());
}

/// 無痕、學習關掉時照樣追蹤正在打的串、照樣有建議（ADR 0008），只是不學。
#[test]
fn incognito_still_suggests_but_learns_nothing() {
    for options in [
        Options {
            incognito: true,
            ..Options::default()
        },
        Options {
            learning: false,
            ..Options::default()
        },
    ] {
        let mut ime = Ime::new();
        ime.session.set_options(options);
        ime.globe();
        assert_eq!(ime.suggestions("hel"), ["hello", "help", "held", "helmet"]);
        ime.type_text("met ");
        assert!(ime.user.english_user_words().is_empty(), "{options:?}");
    }
}

/// 比對不分大小寫，建議跟著打的大小寫：開頭大寫的建議開頭也大寫，全大寫的整個大寫；詞典的寫法照留。
#[test]
fn suggestions_follow_the_case_of_the_typed_word() {
    let mut ime = Ime::new();
    ime.globe();
    assert_eq!(ime.suggestions("Hel"), ["Hello", "Help", "Held", "Helmet"]);
    assert_eq!(ime.suggestions(" HEL"), ["HELLO", "HELP", "HELD", "HELMET"]);
    assert_eq!(ime.suggestions(" na"), ["NASA"]);
}

/// v2 匯入的記住的字串（已過門檻的英文使用者詞）打開頭的字母就在建議列；email 整串是一個詞。
#[test]
fn an_imported_email_is_suggested_from_its_first_letters() {
    let mut ime = Ime::new();
    let v2 = "\
# qim-userdict v2
#@/tick\t1
# scheme\tword\tcode\tcommits\tlast_commit_tick\tdecayed\tlast_tick\tsource\ttrigger
# \t\tstring\tdecayed\tlast_tick
\t\tivan.test@example.com\t1\t1
";
    ime.user = UserModel::import(v2, None).unwrap().user;
    ime.globe();
    assert_eq!(ime.suggestions("iv"), ["ivan.test@example.com"]);
    assert_eq!(ime.suggestions("an.t"), ["ivan.test@example.com"]);
}

impl Ime {
    fn select(&mut self, index: usize) -> Snapshot {
        self.session.select(&self.engine, &mut self.user, index)
    }
}

/// 點建議：殼先刪掉打的詞（`delete_before`，以字元計）再上屏建議與一個空白（照 Gboard，事實 M47），詞打完、學起來。
#[test]
fn picking_a_suggestion_replaces_the_typed_word_and_adds_a_space() {
    let mut ime = Ime::new();
    ime.globe();
    ime.type_text("'Hel");
    let picked = ime.select(1);
    assert!(picked.handled);
    assert_eq!(picked.delete_before, 3, "開頭的 ' 不算在詞裡、不刪");
    assert_eq!(picked.commit.as_deref(), Some("Help "));
    assert!(picked.candidates.is_empty());
    assert_eq!(ime.user.english_user_words(), ["Help"]);
}

/// 點建議補的空白之後：打句讀或右括號先刪掉那個空白再上屏；緊接著的空白被吃掉（照 Gboard，事實 M48）。
#[test]
fn punctuation_right_after_a_picked_suggestion_takes_the_place_of_its_space() {
    let mut ime = Ime::new();
    ime.globe();
    ime.type_text("hel");
    ime.select(0);
    let comma = ime.press(Key::Punct(','));
    assert_eq!(
        (comma.delete_before, comma.commit.as_deref()),
        (1, Some(","))
    );
    let space = ime.press(Key::Space);
    assert_eq!(
        (space.delete_before, space.commit.as_deref()),
        (0, Some(" "))
    );

    ime.type_text("hel");
    ime.select(0);
    let space = ime.press(Key::Space);
    assert!(
        space.handled && space.commit.is_none(),
        "補的空白之後第一下空白吃掉"
    );
    let space = ime.press(Key::Space);
    assert_eq!(space.commit.as_deref(), Some(" "));

    ime.type_text("hel");
    ime.select(0);
    let at = ime.press(Key::Punct('('));
    assert_eq!(
        (at.delete_before, at.commit.as_deref()),
        (0, Some("(")),
        "左括號不換"
    );
}

/// 點建議之後緊接著退格：宿主刪掉補的空白、學到的撤銷（退格撤銷），正在打的串回到選的那個詞，再打空白學它。
#[test]
fn backspace_right_after_a_pick_undoes_its_learning_and_resumes_the_word() {
    let mut ime = Ime::new();
    ime.globe();
    ime.type_text("hel");
    ime.select(3);
    assert_eq!(ime.user.english_user_words(), ["helmet"]);
    assert!(!ime.press(Key::Backspace).handled);
    assert!(ime.user.english_user_words().is_empty());
    assert_eq!(ime.suggestions("s"), Vec::<String>::new(), "串是 helmets");
    ime.press(Key::Space);
    assert_eq!(ime.user.english_user_words(), ["helmets"]);
}

/// 句首自動大寫（brief〈設定與同步〉，預設關）：開著時英文模式在句子開頭（空的欄位、`.` `?` `!` 後接空白、
/// 換行之後，照 Gboard 實測，事實 M48）要殼自動開 Shift（`auto_shift`），使用者可以點掉；打了字母就關。
#[test]
fn auto_capitalize_asks_the_keyboard_to_shift_at_the_start_of_a_sentence() {
    let mut ime = Ime::new();
    ime.session.set_context("");
    assert!(!ime.globe().auto_shift, "預設關");
    ime.globe();

    ime.session.set_options(Options {
        auto_capitalize: true,
        ..Options::default()
    });
    assert!(!ime.press(Key::Letter('a')).auto_shift, "中文不自動大寫");
    ime.press(Key::Escape);
    ime.session.set_context("");
    assert!(ime.globe().auto_shift, "空的欄位");
    let shifts: Vec<bool> = ime
        .type_text("Ok. Then? yes!no ")
        .iter()
        .map(|s| s.auto_shift)
        .collect();
    // O k . ␣ T h e n ? ␣ y e s ! n o ␣
    assert_eq!(
        shifts,
        [
            false, false, false, true, false, false, false, false, false, true, false, false,
            false, false, false, false, false
        ]
    );
    ime.session.set_context("first line\n");
    assert!(ime.press(Key::Modifier).auto_shift, "換行之後");
    ime.session.set_context("said so");
    assert!(!ime.press(Key::Modifier).auto_shift);
    ime.session.clear_context(&mut ime.user);
    assert!(!ime.press(Key::Modifier).auto_shift, "不知道前文時不大寫");
}
