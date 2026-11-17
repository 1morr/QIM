//! 智慧標點（brief〈英文模式與欄位類型〉，M4 票 27）：英文模式連按兩下空白輸出句點、中文狀態的 `"` `'` 左右輪流。
//! 數字後半形在 `digit_separators.rs`。

mod common;

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, Engine, EnglishPack, EnglishWords, FieldKind, Key, Packs, Ranking,
    Scheme, SchemePack, Session, ShiftKey, Snapshot, UserModel,
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
            words: EnglishWords::new([("hello", 200), ("help", 180)]),
        })
}

/// 核心加一個假宿主：先照 `delete_before` 刪游標前的字，再接上 `commit`；核心沒吃下的鍵照宿主的預設動作打出來。
struct Ime {
    engine: Engine,
    user: UserModel,
    session: Session,
    /// 輸入框的內容，游標在最後。
    text: String,
}

impl Ime {
    fn new() -> Self {
        Self {
            engine: engine(),
            user: UserModel::default(),
            session: Session::default(),
            text: String::new(),
        }
    }

    /// 英文模式（觸控的地球鍵）。
    fn english() -> Self {
        let mut ime = Self::new();
        ime.press(Key::ToggleEnglish);
        ime
    }

    fn press(&mut self, key: Key) -> Snapshot {
        let snapshot = self
            .session
            .handle(&self.engine, &mut self.user, key.clone());
        self.apply(&snapshot);
        if !snapshot.handled {
            match key {
                Key::Letter(c) | Key::Punct(c) => self.text.push(c),
                Key::Digit(d) => self.text.push(char::from(b'0' + d)),
                Key::Space => self.text.push(' '),
                Key::Backspace => {
                    self.text.pop();
                }
                _ => {}
            }
        }
        snapshot
    }

    fn apply(&mut self, snapshot: &Snapshot) {
        for _ in 0..snapshot.delete_before {
            self.text.pop();
        }
        if let Some(commit) = &snapshot.commit {
            self.text.push_str(commit);
        }
    }

    /// 逐字打 `text`：字母、數字、標點、空白各轉成它的鍵。
    fn type_text(&mut self, text: &str) {
        for c in text.chars() {
            let key = match c {
                ' ' => Key::Space,
                c if c.is_ascii_digit() => Key::Digit(c.to_digit(10).unwrap() as u8),
                c if c.is_ascii_alphabetic() => Key::Letter(c),
                c => Key::Punct(c),
            };
            self.press(key);
        }
    }
}

/// 英文模式在字後連按兩下空白：第二下刪掉第一下的空白、上屏「. 」（Gboard、LatinIME，事實 M48、M52）。
#[test]
fn two_spaces_after_a_word_in_english_mode_become_a_period_and_a_space() {
    let mut ime = Ime::english();
    ime.type_text("hi ");
    let second = ime.press(Key::Space);
    assert_eq!(second.delete_before, 1);
    assert_eq!(second.commit.as_deref(), Some(". "));
    assert_eq!(ime.text, "hi. ");
}

/// 殼的時限到了（`expire_double_space`，照 LatinIME 1100 ms，事實 M52）之後的空白照打。
#[test]
fn a_second_space_after_the_time_limit_stays_a_space() {
    let mut ime = Ime::english();
    ime.type_text("hi ");
    ime.session.expire_double_space();
    ime.press(Key::Space);
    assert_eq!(ime.text, "hi  ");
}

/// 兩下之間夾了別的鍵（方向鍵移走了游標）照打。
#[test]
fn a_key_between_the_spaces_keeps_both_spaces() {
    let mut ime = Ime::english();
    ime.type_text("hi ");
    ime.press(Key::Left);
    ime.press(Key::Space);
    assert_eq!(ime.text, "hi  ");
}

/// 不送字元的修飾鍵不算夾在中間（同退格撤銷）。
#[test]
fn a_lone_modifier_between_the_spaces_still_makes_a_period() {
    let mut ime = Ime::english();
    ime.type_text("hi ");
    ime.press(Key::Modifier);
    ime.press(Key::Space);
    assert_eq!(ime.text, "hi. ");
}

/// 第一個空白前是句讀時照打（Gboard「前一字不是標點」、LatinIME 的字元表，事實 M48、M52）。
#[test]
fn two_spaces_after_punctuation_stay_spaces() {
    let mut ime = Ime::english();
    ime.type_text("hi,  ");
    assert_eq!(ime.text, "hi,  ");
}

/// 第三下空白照打：游標前是句點加空白。
#[test]
fn a_third_space_stays_a_space() {
    let mut ime = Ime::english();
    ime.type_text("hi   ");
    assert_eq!(ime.text, "hi.  ");
}

/// 右括號、引號、數字後照換（LatinIME 的字元表，事實 M52）。
#[test]
fn closing_marks_and_digits_also_take_the_period() {
    for typed in ["(hi)", "\"hi\"", "42"] {
        let mut ime = Ime::english();
        ime.type_text(&format!("{typed}  "));
        assert_eq!(ime.text, format!("{typed}. "));
    }
}

/// 換出句點後是句子開頭：句首自動大寫開著時要殼開 Shift（Gboard，事實 M48）。
#[test]
fn the_period_starts_a_sentence() {
    let mut ime = Ime::english();
    ime.session.set_options(qim_core::Options {
        auto_capitalize: true,
        ..ime.session.options()
    });
    ime.type_text("hi");
    assert!(!ime.press(Key::Space).auto_shift, "句中的空白");
    assert!(ime.press(Key::Space).auto_shift);
    assert_eq!(ime.text, "hi. ");
}

/// 緊接著退格不還原，只刪掉空白（照 Gboard，事實 M48；LatinIME 換回一個空白）。
#[test]
fn backspace_after_the_period_only_deletes_the_space() {
    let mut ime = Ime::english();
    ime.type_text("hi  ");
    let back = ime.press(Key::Backspace);
    assert!(!back.handled);
    assert_eq!(ime.text, "hi.");
}

/// 關掉開關後兩下空白照打。
#[test]
fn turning_the_option_off_keeps_both_spaces() {
    let mut ime = Ime::english();
    ime.session.set_options(qim_core::Options {
        double_space_period: false,
        ..ime.session.options()
    });
    ime.type_text("hi  ");
    assert_eq!(ime.text, "hi  ");
}

/// email、網址、密碼欄切到英文模式也不換（LatinIME 只在一般文字欄，事實 M52）。
#[test]
fn email_url_and_password_fields_keep_both_spaces() {
    for field in [FieldKind::Email, FieldKind::Url, FieldKind::Password] {
        let mut ime = Ime::new();
        ime.session.set_field(&ime.engine, &mut ime.user, field);
        ime.press(Key::ToggleEnglish);
        ime.press(Key::ToggleEnglish);
        assert!(ime.press(Key::Letter('h')).handled, "{field:?} 在英文模式");
        ime.type_text("i  ");
        assert_eq!(ime.text, "hi  ", "{field:?}");
    }
}

/// 點了建議（補一個空白）之後第一下空白被吃掉（Gboard，事實 M48），緊接著再按一下才換：同 LatinIME 點建議後
/// 打空白、再打空白（事實 M52）。
#[test]
fn after_picking_a_suggestion_the_eaten_space_counts_as_the_first() {
    let mut ime = Ime::english();
    ime.type_text("hel");
    let picked = ime.session.select(&ime.engine, &mut ime.user, 0);
    ime.apply(&picked);
    assert_eq!(ime.text, "hello ");
    ime.press(Key::Space);
    assert_eq!(ime.text, "hello ", "第一下吃掉");
    ime.press(Key::Space);
    assert_eq!(ime.text, "hello. ");
}

/// 只在英文模式：中文狀態（組字中空白選字、沒有組字時交還宿主）、英文直通、臨時英文都照打。
#[test]
fn chinese_pass_through_and_inline_english_keep_their_spaces() {
    let mut ime = Ime::new();
    ime.type_text("a  ");
    assert_eq!(ime.text, "日 ", "組字中空白選字，之後的空白交還宿主");
    ime.press(Key::Space);
    assert_eq!(ime.text, "日  ");

    let mut ime = Ime::new();
    ime.press(Key::ToggleAscii(ShiftKey::Right));
    ime.type_text("hi  ");
    assert_eq!(ime.text, "hi  ", "英文直通");

    let mut ime = Ime::new();
    ime.type_text("ab");
    ime.press(Key::ToggleAscii(ShiftKey::Left));
    ime.type_text("  ");
    ime.press(Key::Enter);
    assert_eq!(ime.text, "ab  ", "臨時英文");
}

/// 無痕時殼不給前文，照樣換：核心只記游標前是不是「字加空白」，不記文字。
#[test]
fn it_works_while_incognito() {
    let mut ime = Ime::english();
    ime.session.set_options(qim_core::Options {
        incognito: true,
        ..ime.session.options()
    });
    ime.type_text("hi  ");
    assert_eq!(ime.text, "hi. ");
}

/// 中文狀態的 `"` `'` 左右輪流（使用者定，照小狼毫用的 librime `pair`，事實 R12、M53）：第一下左、第二下右，兩個鍵各自
/// 輪流。
#[test]
fn quotes_alternate_between_opening_and_closing_in_chinese() {
    let mut ime = Ime::new();
    ime.type_text("\"'a '\"\"");
    assert_eq!(ime.text, "“‘日’”“");
}

/// 組字中按 `"` 先上屏整個組字再接引號；`'` 組字中照舊是分隔記號，沒有組字時才是引號。
#[test]
fn a_double_quote_while_composing_commits_first_and_a_single_quote_stays_the_separator() {
    let mut ime = Ime::new();
    ime.type_text("a\"");
    assert_eq!(ime.text, "日“");

    let mut ime = Ime::new();
    let separated = ime.press(Key::Letter('a'));
    assert!(separated.handled);
    assert_eq!(ime.press(Key::Punct('\'')).composition.input, "a'");
    assert_eq!(ime.text, "");
}

/// 換了欄位從左引號重來（照 fcitx5 在輸入框 reset 時清掉，事實 M53；librime 不重來）。
#[test]
fn quotes_start_over_in_a_new_field() {
    let mut ime = Ime::new();
    ime.type_text("\"");
    ime.session
        .set_field(&ime.engine, &mut ime.user, FieldKind::Text);
    ime.type_text("\"");
    assert_eq!(ime.text, "““");
}

/// 關掉開關，`"` `'` 照舊上屏半形（交還宿主）。
#[test]
fn turning_quote_pairing_off_keeps_ascii_quotes() {
    let mut ime = Ime::new();
    ime.session.set_options(qim_core::Options {
        paired_quotes: false,
        ..ime.session.options()
    });
    let quote = ime.press(Key::Punct('"'));
    assert!(!quote.handled);
    ime.type_text("'");
    assert_eq!(ime.text, "\"'");
}

/// 英文模式、英文直通、臨時英文的引號照原樣（Gboard 英文鍵盤是直引號，事實 M48）。
#[test]
fn english_keeps_straight_quotes() {
    let mut ime = Ime::english();
    ime.type_text("\"hi\"");
    assert_eq!(ime.text, "\"hi\"", "英文模式");

    let mut ime = Ime::new();
    ime.press(Key::ToggleAscii(ShiftKey::Right));
    ime.type_text("\"'");
    assert_eq!(ime.text, "\"'", "英文直通");

    let mut ime = Ime::new();
    ime.type_text("ab");
    ime.press(Key::ToggleAscii(ShiftKey::Left));
    ime.type_text("\"");
    ime.press(Key::Enter);
    assert_eq!(ime.text, "ab\"", "臨時英文");
}
