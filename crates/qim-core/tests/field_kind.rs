//! 欄位類型（M4 票 26，brief〈英文模式與欄位類型〉）：殼在輸入框聚焦時給 `Session::set_field`。email、網址、
//! 密碼欄預設英文直通、離開時還原（照 Trime 的 `ascii_mode`，青簡對信箱、網址、密碼欄直通）；切到英文模式時
//! 不自動修正、不自動大寫（AOSP LatinIME 的 `InputAttributes`），密碼欄也不出建議；密碼欄照無痕不學不讀前文
//! （brief〈隱私與安全〉）。數字欄的數字頁在殼與佈局，核心的中英不動。

mod common;

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, ContextState, Engine, EnglishPack, EnglishWords, FieldKind, Key, Options,
    Packs, Ranking, Scheme, SchemePack, Session, Snapshot, UserModel,
};

/// 每個字母都是一個字的碼（`a` 日、`b` 月…），掛上一份小英文詞典。
fn engine() -> Engine {
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
            words: EnglishWords::new([("the", 250), ("hello", 200), ("help", 180)]),
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

    fn field(&mut self, field: FieldKind) -> Snapshot {
        self.session.set_field(&self.engine, &mut self.user, field)
    }

    fn press(&mut self, key: Key) -> Snapshot {
        self.session.handle(&self.engine, &mut self.user, key)
    }

    /// 逐字打 `text`，回傳最後一個鍵的結果。
    fn type_text(&mut self, text: &str) -> Snapshot {
        let mut last = None;
        for c in text.chars() {
            let key = match c {
                ' ' => Key::Space,
                c if c.is_ascii_digit() => Key::Digit(c.to_digit(10).unwrap() as u8),
                c if c.is_ascii_alphabetic() => Key::Letter(c),
                c => Key::Punct(c),
            };
            last = Some(self.press(key));
        }
        last.expect("打了字")
    }

    fn suggestions(&self) -> Vec<String> {
        self.session
            .all_candidates()
            .into_iter()
            .map(|c| c.text)
            .collect()
    }
}

/// email、網址、密碼欄一聚焦就是英文直通：鍵交還宿主、不組字、沒有建議。
#[test]
fn email_url_and_password_fields_start_in_english_pass_through() {
    for field in [FieldKind::Email, FieldKind::Url, FieldKind::Password] {
        let mut ime = Ime::new();
        assert!(ime.field(field).ascii, "{field:?}");
        let typed = ime.type_text("hel");
        assert!(!typed.handled, "{field:?}");
        assert!(!ime.session.composing(), "{field:?}");
        assert!(ime.suggestions().is_empty(), "{field:?}");
    }
}

/// 一般欄與數字欄不動中英：中文照樣組字。
#[test]
fn text_and_number_fields_keep_the_current_mode() {
    for field in [FieldKind::Text, FieldKind::Number] {
        let mut ime = Ime::new();
        assert!(!ime.field(field).ascii, "{field:?}");
        ime.press(Key::Letter('a'));
        assert!(ime.session.composing(), "{field:?}");
    }
}

/// 離開 email 欄回到一般欄時還原進去之前的中文（Trime 離開欄位時還原 `ascii_mode`）。
#[test]
fn leaving_an_english_field_restores_chinese() {
    let mut ime = Ime::new();
    ime.field(FieldKind::Email);
    assert!(!ime.field(FieldKind::Text).ascii);
    ime.press(Key::Letter('a'));
    assert!(ime.session.composing());
}

/// 原本就是英文模式的，離開密碼欄回到英文模式（字母核心吃下、當場上屏）。
#[test]
fn leaving_an_english_field_restores_english_mode() {
    let mut ime = Ime::new();
    ime.press(Key::ToggleEnglish);
    ime.field(FieldKind::Password);
    assert!(ime.field(FieldKind::Number).ascii);
    assert_eq!(ime.press(Key::Letter('a')).commit.as_deref(), Some("a"));
}

/// 在 email 欄切回中文的不留到下一個要英文的欄位，離開時仍還原進第一個欄位之前的。
#[test]
fn a_switch_inside_an_english_field_stays_in_that_field() {
    let mut ime = Ime::new();
    ime.press(Key::ToggleEnglish);
    ime.field(FieldKind::Email);
    assert!(!ime.press(Key::ToggleEnglish).ascii, "在 email 欄切回中文");
    assert!(ime.field(FieldKind::Url).ascii, "下一個網址欄又是英文");
    ime.field(FieldKind::Text);
    assert_eq!(
        ime.press(Key::Letter('a')).commit.as_deref(),
        Some("a"),
        "回到進 email 欄之前的英文模式"
    );
}

/// 組字中換欄位：組字留在上一個欄位，不帶過來。
#[test]
fn a_new_field_drops_the_composition() {
    let mut ime = Ime::new();
    ime.press(Key::Letter('a'));
    let snapshot = ime.field(FieldKind::Email);
    assert!(snapshot.composition.input.is_empty());
    assert!(snapshot.candidates.is_empty());
    assert_eq!(snapshot.commit, None);
}

/// 換欄位是打完了上一個欄位正在打的串：學成英文使用者詞（同離開欄位）。
#[test]
fn a_new_field_finishes_the_string_typed_in_the_last_one() {
    let mut ime = Ime::new();
    ime.field(FieldKind::Email);
    ime.type_text("me@example.com");
    ime.field(FieldKind::Password);
    assert_eq!(ime.user.english_user_words(), ["me@example.com"]);
}

/// email、網址欄切到英文模式時，自動修正開著也不修（LatinIME：沒有 AUTO_CORRECT 旗標）；一般欄照修。
#[test]
fn email_and_url_fields_do_not_autocorrect() {
    let autocorrecting = Options {
        autocorrect: true,
        ..Options::default()
    };
    for field in [FieldKind::Email, FieldKind::Url] {
        let mut ime = Ime::new();
        ime.session.set_options(autocorrecting);
        ime.field(field);
        ime.press(Key::ToggleEnglish);
        ime.press(Key::ToggleEnglish);
        let finished = ime.type_text("teh ");
        assert_eq!(finished.delete_before, 0, "{field:?}");
        assert_eq!(finished.commit.as_deref(), Some(" "), "{field:?}");
    }
    let mut ime = Ime::new();
    ime.session.set_options(autocorrecting);
    ime.press(Key::ToggleEnglish);
    assert_eq!(ime.type_text("teh ").commit.as_deref(), Some("the "));
}

/// email、網址欄切到英文模式時，句首自動大寫開著也不開 Shift（LatinIME、Gboard 在 email 與網址欄不大寫）。
#[test]
fn email_and_url_fields_do_not_capitalize() {
    for field in [FieldKind::Email, FieldKind::Url] {
        let mut ime = Ime::new();
        ime.session.set_options(Options {
            auto_capitalize: true,
            ..Options::default()
        });
        ime.field(field);
        ime.press(Key::ToggleEnglish);
        ime.press(Key::ToggleEnglish);
        ime.session.set_context("");
        assert!(!ime.session.auto_shift(), "{field:?}");
    }
}

/// 密碼欄切到英文模式也不出建議（LatinIME、Gboard 的密碼欄沒有建議列）。
#[test]
fn password_fields_show_no_suggestions_even_in_english_mode() {
    let mut ime = Ime::new();
    ime.field(FieldKind::Password);
    ime.press(Key::ToggleEnglish);
    ime.press(Key::ToggleEnglish);
    ime.type_text("hel");
    assert!(ime.suggestions().is_empty());
}

/// 密碼欄不讀前文（brief〈隱私與安全〉），殼不必另開無痕，使用者的無痕開關也不動。
#[test]
fn password_fields_read_no_context() {
    let mut ime = Ime::new();
    ime.field(FieldKind::Password);
    ime.session.set_context("my secret");
    assert_eq!(
        ime.session.inspect(&ime.engine, &ime.user).context,
        ContextState::Incognito
    );
    assert!(ime.session.private());
    assert!(!ime.session.options().incognito);
}

/// 密碼欄打的不學（brief〈隱私與安全〉）。
#[test]
fn password_fields_learn_nothing() {
    let mut ime = Ime::new();
    ime.field(FieldKind::Password);
    ime.type_text("hunter2hunter2 ");
    ime.field(FieldKind::Text);
    assert!(ime.user.english_user_words().is_empty());
}

/// debug 面板讀得到現在的欄位類型。
#[test]
fn inspection_shows_the_field_kind() {
    let mut ime = Ime::new();
    assert_eq!(
        ime.session.inspect(&ime.engine, &ime.user).field,
        FieldKind::Text
    );
    ime.field(FieldKind::Url);
    assert_eq!(
        ime.session.inspect(&ime.engine, &ime.user).field,
        FieldKind::Url
    );
}

/// 綁定收與給的名字：`text`、`email`、`url`、`number`、`password`。
#[test]
fn field_kinds_round_trip_through_their_names() {
    for field in [
        FieldKind::Text,
        FieldKind::Email,
        FieldKind::Url,
        FieldKind::Number,
        FieldKind::Password,
    ] {
        assert_eq!(field.name().parse::<FieldKind>(), Ok(field));
    }
    assert_eq!(FieldKind::Email.name(), "email");
    assert!("tel".parse::<FieldKind>().is_err());
}
