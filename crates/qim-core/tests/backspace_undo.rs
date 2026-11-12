//! 上屏後緊接著退格撤銷這次的學習（M4 票 15，plan §2.4，照 librime `memory.cc` 的 `DiscardSession`）：
//! 上屏後第一個交還宿主的鍵是退格時，這次上屏學到的（整串上屏次數、衰減值、tick、自動造詞、個人
//! bigram、英文使用者詞）都回到上屏前；中間夾了別的鍵就不撤銷。退格照常交給宿主刪字。

mod common;

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, Date, Engine, Key, Limits, Options, Packs, Ranking, Scheme, SchemePack,
    Session, ShiftKey, Snapshot, UserModel,
};

/// `a` 日曰、`b` 月肉、`c` 丁：每個碼的首選是第一個字，第 2 個字字頻低。
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
    let table = [("a", "日曰"), ("b", "月肉"), ("c", "丁")];
    let code_table = CodeTable::from_entries(table.iter().map(|(code, chars)| {
        let entries = chars
            .chars()
            .map(|text| CodeEntry {
                text,
                extension: false,
                simplified: false,
            })
            .collect();
        (code.to_string(), entries)
    }));
    let frequency = [
        ('日', 1000),
        ('曰', 500),
        ('月', 1000),
        ('肉', 1),
        ('丁', 1),
    ];
    let model = common::unigram("model", &frequency, frequency.len());
    Engine::new(SchemePack { scheme, code_table }, None, Some(model)).unwrap()
}

struct Ime {
    engine: Engine,
    session: Session,
    user: UserModel,
}

impl Ime {
    /// 聯想關掉：上屏後的退格直接交還宿主。
    fn new() -> Self {
        Self::with_options(Options {
            association: false,
            ..Options::default()
        })
    }

    fn with_options(options: Options) -> Self {
        let mut user = UserModel::default();
        user.set_today("2026-11-12".parse::<Date>().unwrap());
        let mut session = Session::default();
        session.set_options(options);
        Self {
            engine: engine(),
            session,
            user,
        }
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

    /// 打 `code`，依序選文字是 `picks` 各項的候選（分段選字），回傳上屏的字。
    fn choose(&mut self, code: &str, picks: &[&str]) -> Option<String> {
        let mut shown = self.type_text(code);
        for text in picks {
            let at = shown.candidates.iter().position(|c| c.text == *text);
            let at = at.unwrap_or_else(|| panic!("這頁沒有「{text}」：{:?}", texts(&shown)));
            shown = self.press(Key::Digit(u8::try_from(at + 1).unwrap()));
        }
        shown.commit
    }

    /// 打 `code` 時的首選；不上屏。
    fn top_of(&mut self, code: &str) -> String {
        let shown = self.type_text(code);
        self.press(Key::Escape);
        texts(&shown)[0].clone()
    }
}

fn texts(snapshot: &Snapshot) -> Vec<String> {
    snapshot.candidates.iter().map(|c| c.text.clone()).collect()
}

/// 上屏後緊接著退格：這次上屏學到的整串上屏次數、衰減值、tick 與個人 bigram 都回到上屏前（匯出
/// 逐字相同），退格照常交還宿主刪字。
#[test]
fn backspace_right_after_a_commit_restores_the_learning_to_before_it() {
    let mut ime = Ime::new();
    ime.session.set_context("丁");
    ime.choose("a", &["曰"]);
    let before = ime.user.export();

    assert_eq!(ime.choose("a", &["曰"]).as_deref(), Some("曰"));
    assert_ne!(ime.user.export(), before, "上屏有學到");
    let backspace = ime.press(Key::Backspace);

    assert!(!backspace.handled, "退格交還宿主刪字");
    assert_eq!(ime.user.export(), before);
    assert_eq!(ime.top_of("a"), "日", "只選過一次，不成首選");
}

/// 上屏後中間夾了別的鍵（打了字又 Esc 放棄）才退格：不撤銷，選兩次的照樣成首選。
#[test]
fn a_key_between_the_commit_and_the_backspace_keeps_the_learning() {
    let mut ime = Ime::new();
    ime.choose("a", &["曰"]);
    ime.choose("a", &["曰"]);
    let learned = ime.user.export();

    ime.type_text("c");
    ime.press(Key::Escape);
    ime.press(Key::Backspace);
    assert_eq!(ime.user.export(), learned);

    ime.choose("a", &["曰"]);
    ime.press(Key::Space);
    assert!(!ime.press(Key::Backspace).handled);
    assert_eq!(ime.top_of("a"), "曰", "空白夾在中間");
}

/// 不送字元的修飾鍵（按住 Shift、Ctrl 再放開）不算夾在中間：之後的退格照樣撤銷。
#[test]
fn a_modifier_key_between_the_commit_and_the_backspace_does_not_count() {
    let mut ime = Ime::new();
    ime.choose("a", &["曰"]);
    let before = ime.user.export();

    ime.choose("a", &["曰"]);
    ime.press(Key::Modifier);
    ime.press(Key::Backspace);
    assert_eq!(ime.user.export(), before);
}

/// 分段選字上屏的自動造詞、每一段的衰減值與個人 bigram（前文接第一段、相鄰兩段、前文接整串）都撤銷；
/// 第二下退格只是刪字，不再撤銷更早的上屏。
#[test]
fn backspace_undoes_the_auto_phrase_and_personal_bigrams_of_the_commit() {
    let mut ime = Ime::new();
    ime.session.set_context("丁");
    ime.choose("c", &["丁"]);
    let before = ime.user.export();

    assert_eq!(ime.choose("ab", &["曰", "肉"]).as_deref(), Some("曰肉"));
    let learned = ime.user.export();
    assert!(learned.contains("\tauto\t"), "學到自動造詞：{learned}");
    assert!(learned.contains("bigram\t丁\t曰肉"), "{learned}");
    ime.press(Key::Backspace);
    assert_eq!(ime.user.export(), before);

    ime.press(Key::Backspace);
    assert_eq!(ime.user.export(), before, "更早的上屏不撤銷");
}

/// 觸控點候選（`Session::select`）也是一個動作：上屏後直接點聯想，緊接著退格只撤銷點聯想學到的個人
/// bigram，不連前一次上屏一起撤銷。
#[test]
fn backspace_after_a_tapped_association_undoes_only_that_commit() {
    let mut ime = Ime::with_options(Options::default());
    ime.choose("ab", &["日", "月"]);
    ime.press(Key::Escape);
    ime.type_text("a");
    let committed = ime.session.select(&ime.engine, &mut ime.user, 0);
    assert_eq!(committed.commit.as_deref(), Some("日"));
    assert_eq!(texts(&committed), ["月"], "聯想：個人 bigram 日 → 月");
    let before = ime.user.export();

    let tapped = ime.session.select(&ime.engine, &mut ime.user, 0);
    assert_eq!(tapped.commit.as_deref(), Some("月"));
    assert_ne!(ime.user.export(), before, "點聯想學到個人 bigram");
    assert!(!ime.press(Key::Backspace).handled);
    assert_eq!(ime.user.export(), before);
}

/// 觸控鍵盤打出的字（`Session::text`）夾在上屏與退格之間：不撤銷。
#[test]
fn text_from_the_touch_keyboard_between_the_commit_and_the_backspace_keeps_the_learning() {
    let mut ime = Ime::new();
    ime.choose("a", &["曰"]);
    ime.choose("a", &["曰"]);
    let learned = ime.user.export();

    ime.session.text(&ime.engine, &mut ime.user, "！");
    ime.press(Key::Backspace);
    assert_eq!(ime.user.export(), learned);
}

/// 上屏後出了聯想：第一下退格只收起聯想（不刪字，plan §2.1），不算夾在中間；第二下交還宿主刪字，
/// 這時撤銷（同 librime：被 predictor 吃下的退格不到 `OnUnhandledKey`）。
#[test]
fn the_backspace_that_only_closes_the_association_waits_for_the_next_key() {
    let mut ime = Ime::with_options(Options::default());
    ime.choose("ab", &["日", "月"]);
    ime.press(Key::Escape);
    let before = ime.user.export();

    ime.type_text("a");
    let committed = ime.press(Key::Space);
    assert_eq!(texts(&committed), ["月"], "聯想：個人 bigram 日 → 月");
    let closed = ime.press(Key::Backspace);
    assert!(closed.handled && closed.candidates.is_empty(), "只收起聯想");
    assert_ne!(ime.user.export(), before, "還沒撤銷");
    assert!(!ime.press(Key::Backspace).handled);
    assert_eq!(ime.user.export(), before);
}

/// 只收起聯想之後打了別的鍵：照樣是夾在中間，不撤銷。
#[test]
fn closing_the_association_then_typing_keeps_the_learning() {
    let mut ime = Ime::with_options(Options::default());
    ime.choose("ab", &["日", "月"]);
    ime.press(Key::Escape);
    ime.type_text("a");
    ime.press(Key::Space);
    let learned = ime.user.export();

    ime.press(Key::Backspace);
    ime.type_text("c");
    ime.press(Key::Escape);
    ime.press(Key::Backspace);
    assert_eq!(ime.user.export(), learned);
}

/// 上屏後換了欄位（殼呼叫 `clear_context`）才退格：退格不在剛上屏的字後面，不撤銷。
#[test]
fn backspace_after_leaving_the_field_keeps_the_learning() {
    let mut ime = Ime::new();
    ime.choose("a", &["曰"]);
    let learned = ime.user.export();

    ime.session.clear_context(&mut ime.user);
    ime.press(Key::Backspace);
    assert_eq!(ime.user.export(), learned);
}

/// 英文直通打完一串（空白或標點讓它打完，學成英文使用者詞）後緊接著退格：撤銷那個英文使用者詞，
/// 退格照常交還宿主刪掉那個空白或標點。
#[test]
fn backspace_right_after_the_space_or_punctuation_that_ends_an_english_word_undoes_it() {
    for ending in [" ", ","] {
        let mut ime = Ime::new();
        ime.press(Key::ToggleAscii(ShiftKey::Right));
        ime.type_text("abcd ");
        let before = ime.user.export();

        ime.type_text(&format!("rrrrr{ending}"));
        assert_eq!(
            ime.user.english_user_words(),
            ["abcd", "rrrrr"],
            "{ending:?}"
        );
        assert!(!ime.press(Key::Backspace).handled);
        assert_eq!(ime.user.export(), before, "{ending:?}");
    }
}

/// 撤銷後游標回到那串的結尾（退格刪掉的是讓它打完的字）：接著改字（刪一個字母、補一個）再打完，學的
/// 是改過的那串；打到一半的退格只是刪字。
#[test]
fn after_the_undo_the_corrected_word_is_learned_when_it_ends_again() {
    let mut ime = Ime::new();
    ime.press(Key::ToggleAscii(ShiftKey::Right));
    ime.type_text("rrrrr ");
    ime.press(Key::Backspace);
    ime.press(Key::Backspace);
    ime.type_text("s ");
    assert_eq!(ime.user.english_user_words(), ["rrrrs"]);

    ime.type_text("abcde");
    ime.press(Key::Backspace);
    ime.type_text(" ");
    assert_eq!(ime.user.english_user_words(), ["abcd", "rrrrs"]);
}

/// 中文上屏讓前面的英文串打完時，兩種學習是同一個動作：緊接著的退格一起撤銷。
#[test]
fn backspace_undoes_both_the_english_word_and_the_chinese_commit_that_ended_it() {
    let mut ime = Ime::new();
    let before = ime.user.export();

    ime.press(Key::ToggleAscii(ShiftKey::Right));
    ime.type_text("rrrrr");
    ime.press(Key::ToggleAscii(ShiftKey::Right));
    assert_eq!(ime.choose("a", &["曰"]).as_deref(), Some("曰"));
    assert_eq!(ime.user.english_user_words(), ["rrrrr"]);
    ime.press(Key::Backspace);
    assert_eq!(ime.user.export(), before);
}

/// 這次上屏讓條目到了上限、淘汰掉舊的：撤銷時淘汰掉的也放回來，整份和上屏前一樣。
#[test]
fn backspace_also_brings_back_the_entries_the_commit_evicted() {
    let mut ime = Ime::new();
    ime.user.set_limits(Limits {
        entries: 5,
        bigrams: 2,
    });
    ime.session.set_context("丁");
    for (code, text) in [
        ("a", "日"),
        ("a", "曰"),
        ("b", "月"),
        ("b", "肉"),
        ("c", "丁"),
    ] {
        ime.choose(code, &[text]);
    }
    let before = ime.user.export();
    let rows = |export: &str| export.lines().filter(|l| l.starts_with("entry\t")).count();
    assert_eq!(rows(&before), 5);

    ime.choose("ab", &["日月"]);
    let evicted = ime.user.export();
    assert_eq!(rows(&evicted), 5, "多出整串一列、淘汰一列：{evicted}");
    assert!(
        !evicted.contains("entry\tquick\t曰\t"),
        "最舊的「曰」被淘汰：{evicted}"
    );
    ime.press(Key::Backspace);
    assert_eq!(ime.user.export(), before);
}

/// 上屏後使用者手動加了詞（明確的操作）：之後的退格不撤銷，手動加的詞與上屏學到的都留著。
#[test]
fn a_word_added_by_hand_after_the_commit_is_not_undone() {
    let mut ime = Ime::new();
    ime.choose("a", &["曰"]);
    ime.user.add_word(&ime.engine, "曰", Some("yy")).unwrap();
    let added = ime.user.export();

    ime.press(Key::Backspace);
    assert_eq!(ime.user.export(), added);
}

/// 兩個輸入框各一個 `Session`、共用一份個人模型：一個輸入框上屏後，另一個輸入框的退格不撤銷它；無痕
/// 中的退格也不撤銷（無痕不動學習資料）。
#[test]
fn only_the_same_session_outside_incognito_undoes_its_commit() {
    let mut ime = Ime::new();
    ime.choose("a", &["曰"]);
    let learned = ime.user.export();

    let mut other = Session::default();
    other.handle(&ime.engine, &mut ime.user, Key::Backspace);
    assert_eq!(ime.user.export(), learned, "另一個輸入框的退格");

    ime.choose("a", &["曰"]);
    let learned = ime.user.export();
    ime.session.set_options(Options {
        incognito: true,
        ..ime.session.options()
    });
    ime.press(Key::Backspace);
    assert_eq!(ime.user.export(), learned, "無痕中的退格");
}

/// 殼計時（照 librime 的 3 秒，plan §2.4）：上屏後過了時限，殼呼叫 `expire_undo`，之後緊接著的退格只刪字、
/// 不撤銷；時限內照樣撤銷。另一個輸入框的 `expire_undo` 不影響這裡的紀錄。
#[test]
fn backspace_after_the_shell_expires_the_undo_keeps_the_learning() {
    let mut ime = Ime::new();
    ime.choose("a", &["曰"]);
    let before = ime.user.export();

    ime.choose("a", &["曰"]);
    let learned = ime.user.export();
    ime.session.expire_undo(&mut ime.user);
    assert!(!ime.press(Key::Backspace).handled, "退格照常交還宿主");
    assert_eq!(ime.user.export(), learned);

    let mut ime = Ime::new();
    ime.choose("a", &["曰"]);
    ime.choose("a", &["曰"]);
    Session::default().expire_undo(&mut ime.user);
    ime.press(Key::Backspace);
    assert_eq!(ime.user.export(), before, "別的輸入框的時限");
}

/// 英文直通打完一串的是方向鍵、Tab 這類不送字、會移走游標的鍵：之後的退格刪的不是那串旁邊的字，
/// 不撤銷（Enter 送出換行，緊接著的退格刪掉它，照樣撤銷）。
#[test]
fn an_english_word_ended_by_a_key_that_moves_the_cursor_is_not_undone() {
    for key in [Key::Left, Key::Other] {
        let mut ime = Ime::new();
        ime.press(Key::ToggleAscii(ShiftKey::Right));
        ime.type_text("rrrrr");
        ime.press(key.clone());
        ime.press(Key::Backspace);
        assert_eq!(ime.user.english_user_words(), ["rrrrr"], "{key:?}");
    }

    let mut ime = Ime::new();
    ime.press(Key::ToggleAscii(ShiftKey::Right));
    ime.type_text("rrrrr");
    ime.press(Key::Enter);
    ime.press(Key::Backspace);
    assert!(ime.user.english_user_words().is_empty(), "Enter 之後的退格");
}

/// 退格撤銷與停鍵結果快取（M4 票 03、15）：快取的鍵含路徑分，路徑分含學到的加分。上屏學到了，同一串碼
/// 同樣的前文也不用上屏前的結果；上屏後退格撤銷，學習資料回到上屏前，模型的輸入逐字相同，照快取排的
/// 就是模型會給的名次。
#[test]
fn the_rerank_cache_follows_learning_and_its_undo() {
    let rerank_ab = |ime: &mut Ime| {
        ime.session.set_reranker_id(Some("test"));
        ime.type_text("ab");
        let request = ime.session.rerank_request(&ime.engine).unwrap();
        let second = request.paths.iter().position(|p| p.text == "曰肉").unwrap();
        let reranked = ime
            .session
            .apply_rerank(&ime.engine, request.generation, &[second])
            .unwrap();
        assert_eq!(texts(&reranked)[1], "曰肉");
        (request, texts(&reranked))
    };

    // 學到了：前文同樣是不知道（換欄位），路徑分變了，不用快取。
    let mut learned = Ime::new();
    let (before, _) = rerank_ab(&mut learned);
    assert_eq!(learned.press(Key::Space).commit.as_deref(), Some("日月"));
    learned.session.clear_context(&mut learned.user);
    learned.type_text("ab");
    let after = learned.session.rerank_request(&learned.engine).unwrap();
    assert_eq!(after.context, before.context);
    assert_ne!(after.paths, before.paths, "學到的加分進了路徑分");
    assert!(
        !learned
            .session
            .inspect(&learned.engine, &learned.user)
            .rerank_cached
    );

    // 上屏後退格撤銷：學習資料回到上屏前，同一份要求用快取。
    let mut undone = Ime::new();
    let exported = undone.user.export();
    let (_, reranked) = rerank_ab(&mut undone);
    undone.press(Key::Space);
    assert!(!undone.press(Key::Backspace).handled);
    assert_eq!(undone.user.export(), exported);
    assert_eq!(texts(&undone.type_text("ab")), reranked);
    assert_eq!(undone.session.rerank_request(&undone.engine), None);
    assert!(
        undone
            .session
            .inspect(&undone.engine, &undone.user)
            .rerank_cached
    );
}
