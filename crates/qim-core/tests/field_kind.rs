//! 欄位類型（M4 票 26，brief〈英文模式與欄位類型〉）：殼在輸入框聚焦時給 `Session::set_field`。email、網址、
//! 密碼欄預設英文模式、離開時還原（照 Trime 的 `ascii_mode`，青簡對信箱、網址、密碼欄直通），不自動修正、不
//! 自動大寫（AOSP LatinIME 的 `InputAttributes`）；密碼欄另不出建議、照無痕不學不讀前文（brief〈隱私與安全〉）。
//! 數字欄的數字頁在殼與佈局，核心的中英不動。

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

/// email、網址、密碼欄一聚焦就是英文模式：字母當場上屏、不組字。
#[test]
fn email_url_and_password_fields_start_in_english_mode() {
    for field in [FieldKind::Email, FieldKind::Url, FieldKind::Password] {
        let mut ime = Ime::new();
        assert!(ime.field(field).ascii, "{field:?}");
        let typed = ime.press(Key::Letter('a'));
        assert_eq!(typed.commit.as_deref(), Some("a"), "{field:?}");
        assert!(!ime.session.composing(), "{field:?}");
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

/// 離開 email 欄回到一般欄時還原進去之前的中英（Trime 離開欄位時還原 `ascii_mode`），在 email 欄切過也一樣；
/// 原本就是英文模式的回來仍是英文。
#[test]
fn leaving_an_english_field_restores_the_mode_from_before() {
    let mut ime = Ime::new();
    ime.field(FieldKind::Email);
    assert!(!ime.field(FieldKind::Text).ascii, "回到中文");

    ime.field(FieldKind::Email);
    assert!(!ime.press(Key::ToggleEnglish).ascii, "在 email 欄切回中文");
    ime.field(FieldKind::Url);
    assert!(
        ime.press(Key::Letter('a')).commit.is_some(),
        "換到另一個網址欄又是英文"
    );
    assert!(
        !ime.field(FieldKind::Text).ascii,
        "回到進 email 欄之前的中文"
    );

    ime.press(Key::ToggleEnglish);
    ime.field(FieldKind::Password);
    assert!(ime.field(FieldKind::Number).ascii, "原本是英文模式");
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

/// email、網址欄自動修正開著也不修、自動大寫開著也不開 Shift（LatinIME：沒有 AUTO_CORRECT 旗標、email 與網址
/// 不大寫）；建議照出。
#[test]
fn email_and_url_fields_neither_autocorrect_nor_capitalize() {
    for field in [FieldKind::Email, FieldKind::Url] {
        let mut ime = Ime::new();
        ime.session.set_options(Options {
            autocorrect: true,
            auto_capitalize: true,
            ..Options::default()
        });
        ime.field(field);
        ime.session.set_context("");
        assert!(!ime.session.auto_shift(), "{field:?}");
        ime.type_text("hel");
        assert!(
            ime.suggestions().contains(&"hello".to_string()),
            "{field:?}"
        );
        let finished = ime.type_text("teh ");
        assert_eq!(finished.delete_before, 0, "{field:?}");
        assert_eq!(finished.commit.as_deref(), Some(" "), "{field:?}");
    }
    // 一般欄照修，對照組。
    let mut ime = Ime::new();
    ime.session.set_options(Options {
        autocorrect: true,
        ..Options::default()
    });
    ime.press(Key::ToggleEnglish);
    assert_eq!(ime.type_text("teh ").commit.as_deref(), Some("the "));
}

/// 密碼欄不出建議（LatinIME、Gboard 的密碼欄沒有建議列）、不學、不讀前文（brief〈隱私與安全〉），
/// 不必殼另開無痕。
#[test]
fn password_fields_show_no_suggestions_learn_nothing_and_read_no_context() {
    let mut ime = Ime::new();
    ime.field(FieldKind::Password);
    ime.session.set_context("my secret");
    assert_eq!(
        ime.session.inspect(&ime.engine, &ime.user).context,
        ContextState::Incognito
    );
    ime.type_text("hel");
    assert!(ime.suggestions().is_empty());
    ime.type_text("lohello ");
    ime.type_text("hellohello ");
    ime.field(FieldKind::Text);
    assert!(ime.user.english_user_words().is_empty());
    assert!(!ime.session.options().incognito, "使用者的無痕開關不動");
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

/// 綁定收字串：`text`、`email`、`url`、`number`、`password`。
#[test]
fn field_kinds_parse_from_their_names() {
    for (name, field) in [
        ("text", FieldKind::Text),
        ("email", FieldKind::Email),
        ("url", FieldKind::Url),
        ("number", FieldKind::Number),
        ("password", FieldKind::Password),
    ] {
        assert_eq!(name.parse::<FieldKind>(), Ok(field));
    }
    assert!("tel".parse::<FieldKind>().is_err());
}
