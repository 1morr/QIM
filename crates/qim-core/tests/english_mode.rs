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
        Self::with_words(WORDS)
    }

    fn with_words(words: &[(&str, u8)]) -> Self {
        Self {
            engine: engine(words),
            session: Session::default(),
            user: UserModel::default(),
        }
    }

    fn press(&mut self, key: Key) -> Snapshot {
        self.session.handle(&self.engine, &mut self.user, key)
    }

    /// 只看補完：關掉英文容錯（建議列不列拼錯的修正）。
    fn completions_only(mut self) -> Self {
        self.session.set_options(Options {
            english_corrections: false,
            ..self.session.options()
        });
        self
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
    let mut ime = Ime::new().completions_only();
    ime.globe();
    assert_eq!(ime.suggestions("hel"), ["hello", "help", "held", "helmet"]);
    assert_eq!(ime.suggestions("p"), Vec::<String>::new(), "help 打完了");
    let mut ime = Ime::new().completions_only();
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

/// AOSP LatinIME 詞表（英文資料包 `aosp-en-us` 的來源）裡這些詞的 `f`，學過的加分和它合比時照真實的差距測。
const AOSP_WORDS: &[(&str, u8)] = &[
    ("held", 158),
    ("help", 149),
    ("helped", 142),
    ("helping", 128),
    ("helps", 124),
    ("hello", 120),
    ("helicopter", 120),
    ("helmet", 111),
    ("helium", 102),
    ("the", 222),
    ("to", 215),
    ("this", 200),
    ("that", 192),
    ("they", 174),
    ("tumors", 103),
];

/// 打過一次的詞典詞進建議列前 3 格（Gboard 打過一次就出在建議列，事實 M47）：helmet 原本排第 8。
#[test]
fn a_dictionary_word_typed_once_moves_into_the_first_three() {
    let mut ime = Ime::with_words(AOSP_WORDS);
    ime.globe();
    assert_eq!(ime.suggestions("hel")[7], "helmet");
    ime.type_text(" helmet ");
    let shown = ime.suggestions("hel");
    assert!(shown[..3].contains(&"helmet".to_string()), "{shown:?}");
}

/// 只打過一次的冷門詞不壓過常用詞：學過的加分和詞典頻率合比（照 LatinIME 的刻度，事實 M50），不是學過的一律排前。
#[test]
fn a_rare_word_typed_once_stays_behind_common_words() {
    let mut ime = Ime::with_words(AOSP_WORDS);
    ime.globe();
    ime.type_text("tumors ");
    let shown = ime.suggestions("t");
    assert_eq!(
        shown[..5],
        ["the", "to", "this", "that", "they"],
        "{shown:?}"
    );
}

/// 常打的詞往前：打得越多加分越多。
#[test]
fn words_typed_more_often_move_further_up() {
    let mut ime = Ime::with_words(AOSP_WORDS);
    ime.globe();
    ime.type_text("helium ");
    let once = ime.suggestions("hel");
    assert_ne!(once[0], "helium", "{once:?}");
    ime.type_text("ium helium helium helium ");
    let often = ime.suggestions("hel");
    assert_eq!(often[0], "helium", "{once:?} → {often:?}");
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
\t\tjean.test@example.com\t1\t1
";
    ime.user = UserModel::import(v2, None).unwrap().user;
    ime.globe();
    assert_eq!(ime.suggestions("je"), ["jean.test@example.com"]);
    assert_eq!(ime.suggestions("an.t"), ["jean.test@example.com"]);
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
    // 吃掉的那一下之後再按空白是連按兩下空白（`smart_punctuation.rs`），這裡只看補的空白。
    ime.session.set_options(Options {
        double_space_period: false,
        ..ime.session.options()
    });
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
    let mut ime = Ime::new().completions_only();
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

/// 觸控的標點鍵送文字（`Session::text`）：點了建議之後同樣先刪掉補的空白。
#[test]
fn punctuation_typed_as_text_after_a_pick_also_takes_the_place_of_its_space() {
    let mut ime = Ime::new();
    ime.globe();
    ime.type_text("hel");
    ime.select(0);
    let comma = ime.session.text(&ime.engine, &mut ime.user, ",");
    assert_eq!(
        (comma.delete_before, comma.commit.as_deref()),
        (1, Some(","))
    );
    let word = ime.session.text(&ime.engine, &mut ime.user, "A");
    assert_eq!((word.delete_before, word.commit.as_deref()), (0, Some("A")));
}

/// 殼給的前文不以正在打的串結尾（游標移走、宿主改了字）：建議收起，點不到舊建議，不會刪錯字。
#[test]
fn moving_the_cursor_away_drops_the_suggestions() {
    let mut ime = Ime::new();
    ime.session.set_context("");
    ime.globe();
    ime.type_text("hel");
    assert!(!ime.all().is_empty());
    ime.session.set_context("x");
    assert!(ime.all().is_empty());
    let picked = ime.select(0);
    assert_eq!((picked.delete_before, picked.commit), (0, None));
}

/// 點了建議之後游標移走（殼給的前文不以補的空白結尾）或換欄位：下一個空白、句讀照常上屏，不再吃掉或換掉。
#[test]
fn a_picked_space_is_forgotten_once_the_cursor_moves() {
    let mut ime = Ime::new();
    ime.session.set_context("");
    ime.globe();
    ime.type_text("hel");
    ime.select(0);
    ime.session.set_context("hello ");
    assert_eq!(
        ime.press(Key::Punct(',')).delete_before,
        1,
        "前文照舊：還是剛補的空白"
    );

    ime.type_text(" hel");
    ime.select(0);
    ime.session.set_context("elsewhere");
    assert_eq!(ime.press(Key::Space).commit.as_deref(), Some(" "));
    ime.type_text("hel");
    ime.select(0);
    ime.session.clear_context(&mut ime.user);
    assert_eq!(ime.press(Key::Punct(',')).delete_before, 0);
}

/// 換掉補的空白的句讀是串字元（`.`）時，接著打的照樣是經 QIM 打的串：有建議。
#[test]
fn typing_on_after_a_full_stop_that_took_the_picked_space_still_suggests() {
    let mut ime = Ime::new().completions_only();
    ime.globe();
    ime.type_text("hel");
    ime.select(0);
    ime.press(Key::Punct('.'));
    assert_eq!(ime.suggestions("he"), ["hello", "help", "held", "helmet"]);
}

// 自動修正（M4 票 25，brief〈英文模式與欄位類型〉）：預設關；開著時串打完（空白、句讀、Enter）那一下，原樣不是詞、
// 一處拼錯就讀得通的詞換成修正字。

/// 自動修正的測試詞典：`the` 遠比 `tea` 常見，`receive` 只有一個一處拼錯的鄰居。
const TYPOS: &[(&str, u8)] = &[
    ("the", 222),
    ("tea", 121),
    ("ten", 145),
    ("receive", 137),
    ("piece", 137),
    ("peace", 135),
    ("price", 134),
    ("you'd", 85),
    ("your", 190),
    ("final", 150),
    ("finally", 143),
    ("truly", 120),
    ("hello", 120),
    ("held", 158),
    ("help", 149),
    ("her", 176),
    ("don't", 185),
    ("font", 120),
    ("of", 214),
];

impl Ime {
    fn autocorrecting() -> Self {
        let mut ime = Self::with_words(TYPOS);
        ime.session.set_options(Options {
            autocorrect: true,
            ..Options::default()
        });
        ime.globe();
        ime
    }

    /// 打 `text`，回傳最後一個鍵的（`delete_before`, `commit`）。
    fn last(&mut self, text: &str) -> (usize, Option<String>) {
        let last = self.type_text(text).pop().expect("打了字");
        (last.delete_before, last.commit)
    }
}

/// 預設不修：`teh` 打空白照原樣上屏。
#[test]
fn autocorrect_is_off_by_default() {
    let mut ime = Ime::with_words(TYPOS);
    ime.globe();
    assert_eq!(ime.last("teh "), (0, Some(" ".into())));
}

/// 開著時打空白那一下：刪掉打的詞，上屏修正字接空白（照 Gboard，事實 M47）。
#[test]
fn autocorrect_replaces_a_misspelled_word_when_it_is_finished() {
    let mut ime = Ime::autocorrecting();
    assert_eq!(ime.last("teh "), (3, Some("the ".into())));
    assert_eq!(ime.last("recieve,"), (7, Some("receive,".into())));
}

/// 照 LatinIME（`Suggest.java`）不自動修的條件：原樣是詞、含數字、大寫超過一個（含全大寫）、email 字元與中間的
/// `.`、太短；句中首字大寫照修、保留大寫，詞尾的 `.` 留著（ADR 0008）。
#[test]
fn autocorrect_leaves_words_that_were_typed_with_care() {
    let mut ime = Ime::autocorrecting();
    for typed in ["ten ", "teh2 ", "TEH ", "TeH ", "teh@x ", "teh.com ", "fo "] {
        let (deleted, commit) = ime.last(typed);
        assert_eq!((deleted, commit.as_deref()), (0, Some(" ")), "{typed}");
    }
    assert_eq!(
        ime.last("Teh "),
        (3, Some("The ".into())),
        "句中首字大寫照修"
    );
    assert_eq!(ime.last("teh. "), (4, Some("the. ".into())), "修的是 teh");
}

/// 修正不夠明確時不修，留在建議列：最常見的不到次一個的 4 倍（piece、peace、price；`helo` 的 held 與 help）、有漏撇號的修正而最常見的不是它
/// （`youd`：your 比 you'd 常見）、只有多打一個字母才讀得通的（`truely`）。重複的字母少打一個照修（`finaly`）。
#[test]
fn autocorrect_only_takes_a_clear_single_fix() {
    let mut ime = Ime::autocorrecting();
    for typed in ["peice", "helo", "youd", "truely"] {
        ime.type_text(typed);
        assert!(!ime.all().is_empty(), "{typed} 有修正可選");
        let (deleted, commit) = ime.last(" ");
        assert_eq!((deleted, commit.as_deref()), (0, Some(" ")), "{typed}");
    }
    assert_eq!(ime.last("finaly "), (6, Some("finally ".into())));
}

/// 學過的串當成使用者要的字（LatinIME、Gboard 都把使用者歷史當有效詞）：自動修正關著時打過的 `recieve`，打開後
/// 照原樣。
#[test]
fn a_learned_string_is_not_autocorrected() {
    let mut ime = Ime::with_words(TYPOS);
    ime.globe();
    ime.type_text("recieve ");
    ime.session.set_options(Options {
        autocorrect: true,
        ..Options::default()
    });
    assert_eq!(ime.last("recieve "), (0, Some(" ".into())));
}

/// 英文容錯關掉時建議列只補完，自動修正也沒有對象。
#[test]
fn without_english_corrections_nothing_is_corrected() {
    let mut ime = Ime::autocorrecting();
    ime.session.set_options(Options {
        english_corrections: false,
        ..ime.session.options()
    });
    assert_eq!(ime.suggestions("recieve"), Vec::<String>::new());
    assert_eq!(ime.last(" "), (0, Some(" ".into())));
}

/// 英文容錯（預設開）：建議列也列一處拼錯的修正；要自動修正時那個字排第一，debug 面板讀得到。
#[test]
fn corrections_are_suggested_and_the_autocorrection_comes_first() {
    let mut ime = Ime::with_words(TYPOS);
    ime.globe();
    assert_eq!(ime.suggestions("recieve"), ["receive"]);
    let inspection = ime.session.inspect(&ime.engine, &ime.user);
    assert_eq!(
        inspection.candidates[0].kind,
        CandidateKind::EnglishCorrection
    );
    assert_eq!(inspection.autocorrection, None, "自動修正關著");

    let mut ime = Ime::autocorrecting();
    let listed = ime.suggestions("teh");
    assert_eq!(listed.first().map(String::as_str), Some("the"));
    assert!(listed.contains(&"ten".to_string()) && listed.contains(&"tea".to_string()));
    let inspection = ime.session.inspect(&ime.engine, &ime.user);
    assert_eq!(inspection.autocorrection.as_deref(), Some("the"));
}

/// 剛修正完緊接著退格：換回打的詞、觸發的空白一起拿掉（照 Gboard 的「Undo auto-correct on backspace」，事實 M47），
/// 之後這個 Session 不再修它；修正字學到的撤銷，換回的詞照樣在打完時學（M4 票 25）。
#[test]
fn backspace_right_after_an_autocorrection_reverts_it_without_learning_the_correction() {
    let mut ime = Ime::autocorrecting();
    assert_eq!(ime.last("recieve "), (7, Some("receive ".into())));
    assert_eq!(ime.user.english_user_words(), ["receive"]);
    let reverted = ime.press(Key::Backspace);
    assert!(reverted.handled);
    assert_eq!(
        (reverted.delete_before, reverted.commit.as_deref()),
        (8, Some("recieve"))
    );
    assert!(ime.user.english_user_words().is_empty(), "修正字不學");
    assert_eq!(ime.last(" "), (0, Some(" ".into())), "還原過的不再修");
    assert_eq!(ime.user.english_user_words(), ["recieve"]);

    // 句讀觸發的照 LatinIME 留著句讀；學不到的短詞（`teh`）也記得還原過。
    assert_eq!(ime.last("teh,"), (3, Some("the,".into())));
    let reverted = ime.press(Key::Backspace);
    assert_eq!(
        (reverted.delete_before, reverted.commit.as_deref()),
        (4, Some("teh,"))
    );
    assert_eq!(ime.last(" teh "), (0, Some(" ".into())));
}

/// 還原只在緊接著的退格：中間打了別的字，退格照常交還宿主。
#[test]
fn only_the_next_backspace_reverts() {
    let mut ime = Ime::autocorrecting();
    ime.type_text("teh x");
    assert!(!ime.press(Key::Backspace).handled);
    ime.type_text(" teh ");
    ime.session.set_context("somewhere else");
    assert!(!ime.press(Key::Backspace).handled, "游標移走了");
}

/// Enter 也觸發（照 Gboard，事實 M47）：換掉詞之後 Enter 照樣交還宿主，殼先上屏再交還。
#[test]
fn enter_autocorrects_then_goes_to_the_host() {
    let mut ime = Ime::autocorrecting();
    ime.type_text("recieve");
    let enter = ime.press(Key::Enter);
    assert!(!enter.handled);
    assert_eq!(
        (enter.delete_before, enter.commit.as_deref()),
        (7, Some("receive"))
    );
    assert_eq!(ime.user.english_user_words(), ["receive"]);
    assert_eq!(
        ime.suggestions("teh"),
        ["the", "ten", "tea"],
        "Enter 之後重新開始一串"
    );
}

/// 觸控的句讀鍵送文字（`text`），同樣讓詞打完、自動修正。
#[test]
fn punctuation_typed_as_text_also_autocorrects() {
    let mut ime = Ime::autocorrecting();
    ime.type_text("teh");
    let comma = ime.session.text(&ime.engine, &mut ime.user, "?");
    assert_eq!(
        (comma.delete_before, comma.commit.as_deref()),
        (3, Some("the?"))
    );
}

/// 不送字元的修飾鍵（觸控的 ⇧）不算夾在中間：修正之後按了它，退格照樣還原。
#[test]
fn a_modifier_between_does_not_stop_the_revert() {
    let mut ime = Ime::autocorrecting();
    ime.type_text("teh ");
    ime.press(Key::Modifier);
    let reverted = ime.press(Key::Backspace);
    assert_eq!(
        (reverted.delete_before, reverted.commit.as_deref()),
        (4, Some("teh"))
    );
}

/// 修正排在補完之間要再扣分：`hel` 的補完 held、help、hello 在前，一處拼錯的 her 雖然常見也排在它們後面。
#[test]
fn corrections_rank_behind_completions_unless_far_more_common() {
    let mut ime = Ime::with_words(TYPOS);
    ime.globe();
    assert_eq!(
        ime.suggestions("hel")[..4],
        ["held", "help", "hello", "her"]
    );
}

/// 漏打撇號不扣分：`dont` 修成 don't，不是 font。
#[test]
fn a_missing_apostrophe_is_the_cheapest_fix() {
    let mut ime = Ime::autocorrecting();
    assert_eq!(ime.last("dont "), (4, Some("don't ".into())));
}

/// 過了出現門檻的英文使用者詞也是修正的對象（Gboard 學到的字也會成為修正目標，事實 M47）。
#[test]
fn a_learned_string_past_the_threshold_is_a_correction_target() {
    let mut ime = Ime::autocorrecting();
    ime.type_text("qimzorb qimzorb ");
    assert_eq!(ime.last("qimzorv "), (7, Some("qimzorb ".into())));
}

/// 還原的時限同退格撤銷：殼呼叫 `expire_undo`（3 秒到了）之後，退格只刪字，修正字學到的照留。
#[test]
fn the_revert_expires_with_the_undo_window() {
    let mut ime = Ime::autocorrecting();
    ime.type_text("recieve ");
    ime.session.expire_undo(&mut ime.user);
    assert!(!ime.press(Key::Backspace).handled);
    assert_eq!(ime.user.english_user_words(), ["receive"]);
}

/// Enter 修的不還原：Enter 交還了宿主，核心不知道宿主有沒有換行，退格照常交還宿主。
#[test]
fn an_enter_autocorrection_is_not_reverted() {
    let mut ime = Ime::autocorrecting();
    ime.type_text("teh");
    ime.press(Key::Enter);
    assert!(!ime.press(Key::Backspace).handled);
}
