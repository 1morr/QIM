//! 聯想（plan §2.1、§2.3，M3 票 13）：上屏之後、還沒打下一個字時，候選列出現接下來可能要打的
//! 字詞；選了就上屏，接著出下一輪。按鍵照 macOS 速成：數字鍵或點選選、空白不選、Esc 收起，
//! 打字母直接開始新的組字。

use std::collections::BTreeMap;

use qim_core::{
    CharModel, CodeEntry, CodeTable, ConversionTable, Engine, Key, LanguageModel,
    LanguageModelPack, Lexicon, LexiconPack, Options, Packs, Ranking, Scheme, SchemePack, Session,
    ShiftKey, ShiftSwitch, Simplifier, SimplifierPack, Snapshot, UserModel, Word,
};

/// `an` 門、`r` 口、`ni` 外、`a` 日、`d` 本、`o` 人；詞庫有「門口」「日本」「日本人」。
/// 語言模型：門後面常接外（「門外」不在詞庫），其餘照字頻。
fn engine() -> Engine {
    let table = [
        ("an", '門'),
        ("r", '口'),
        ("ni", '外'),
        ("a", '日'),
        ("d", '本'),
        ("o", '人'),
    ];
    let code_table = CodeTable::from_entries(table.iter().map(|&(code, text)| {
        let entry = CodeEntry {
            text,
            extension: false,
            simplified: false,
        };
        (code.to_string(), vec![entry])
    }));
    let word = |text: &str, frequency| Word {
        text: text.into(),
        frequency,
    };
    let words = BTreeMap::from([
        ("anr".to_string(), vec![word("門口", 100)]),
        ("ad".to_string(), vec![word("日本", 200)]),
        ("ado".to_string(), vec![word("日本人", 50)]),
    ]);
    let chars = [
        ('門', 100),
        ('口', 100),
        ('外', 100),
        ('日', 300),
        ('本', 200),
        ('人', 300),
    ];
    let lexicon = LexiconPack {
        id: "lexicon".into(),
        lexicon: Lexicon::new(words, chars),
    };
    let total: f64 = chars.iter().map(|&(_, f)| f as f64).sum();
    let model = |f: u64| {
        let p = (f as f64 / total).ln();
        CharModel {
            start: p,
            continuation: p,
            backoff: 0.0,
        }
    };
    let language_model = LanguageModelPack {
        id: "model".into(),
        model: LanguageModel::new(
            chars.iter().map(|&(c, f)| (c, model(f))),
            model(1),
            [('門', '外', 0.5f64.ln())],
        ),
    };
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        radicals: BTreeMap::new(),
        packs: Packs {
            lexicon: Some("lexicon".into()),
            language_model: Some("model".into()),
            ..Packs::default()
        },
        ranking: Ranking::default(),
    };
    Engine::new(
        SchemePack { scheme, code_table },
        Some(lexicon),
        Some(language_model),
    )
    .unwrap()
}

struct Typing {
    engine: Engine,
    user: UserModel,
    session: Session,
}

impl Typing {
    fn new() -> Self {
        Self {
            engine: engine(),
            user: UserModel::default(),
            session: Session::default(),
        }
    }

    fn with_options(change: impl FnOnce(&mut Options)) -> Self {
        let mut typing = Self::new();
        let mut options = typing.session.options();
        change(&mut options);
        typing.session.set_options(options);
        typing
    }

    fn key(&mut self, key: Key) -> Snapshot {
        self.session.handle(&self.engine, &mut self.user, key)
    }

    /// 打 `code` 再按空白上屏首選。
    fn commit(&mut self, code: &str) -> Snapshot {
        for c in code.chars() {
            self.key(Key::Letter(c));
        }
        self.key(Key::Space)
    }

    /// 按這頁文字是 `text` 的候選的數字鍵。
    fn pick(&mut self, shown: &Snapshot, text: &str) -> Snapshot {
        let at = position(shown, text).unwrap_or_else(|| panic!("這頁沒有「{text}」：{shown:?}"));
        self.key(Key::Digit(u8::try_from(at + 1).unwrap()))
    }
}

fn texts(snapshot: &Snapshot) -> Vec<&str> {
    snapshot
        .candidates
        .iter()
        .map(|c| c.text.as_str())
        .collect()
}

fn position(snapshot: &Snapshot, text: &str) -> Option<usize> {
    snapshot.candidates.iter().position(|c| c.text == text)
}

#[test]
fn after_committing_men_the_bar_offers_kou() {
    let mut typing = Typing::new();
    let committed = typing.commit("an");
    assert_eq!(committed.commit.as_deref(), Some("門"));
    assert!(committed.composition.input.is_empty());
    assert!(texts(&committed).contains(&"口"), "{:?}", texts(&committed));
}

#[test]
fn choosing_an_association_commits_it_and_offers_the_next_round() {
    let mut typing = Typing::new();
    let committed = typing.commit("a");
    assert_eq!(committed.commit.as_deref(), Some("日"));
    let chosen = typing.pick(&committed, "本");
    assert_eq!(chosen.commit.as_deref(), Some("本"));
    // 「日本」接著補完成「日本人」。
    assert!(texts(&chosen).contains(&"人"), "{:?}", texts(&chosen));
    let again = typing.pick(&chosen, "人");
    assert_eq!(again.commit.as_deref(), Some("人"));
}

#[test]
fn typing_a_letter_drops_the_association_and_composes() {
    let mut typing = Typing::new();
    typing.commit("an");
    let typed = typing.key(Key::Letter('a'));
    assert!(typed.commit.is_none());
    assert_eq!(typed.composition.input, "a");
    assert_eq!(texts(&typed).first(), Some(&"日"));
}

#[test]
fn no_association_when_switched_off() {
    let mut typing = Typing::with_options(|o| o.association = false);
    let committed = typing.commit("an");
    assert_eq!(committed.commit.as_deref(), Some("門"));
    assert!(committed.candidates.is_empty());
}

#[test]
fn space_does_not_choose_an_association_and_passes_to_the_host() {
    let mut typing = Typing::new();
    typing.commit("an");
    let space = typing.key(Key::Space);
    assert!(!space.handled);
    assert!(space.commit.is_none());
    assert!(space.candidates.is_empty());
}

#[test]
fn escape_and_backspace_only_close_the_association() {
    for key in [Key::Escape, Key::Backspace] {
        let mut typing = Typing::new();
        typing.commit("an");
        let closed = typing.key(key);
        assert!(closed.handled, "{key:?}");
        assert!(closed.commit.is_none(), "{key:?}");
        assert!(closed.candidates.is_empty(), "{key:?}");
        // 第二下照常交還宿主。
        assert!(!typing.key(key).handled, "{key:?}");
    }
}

#[test]
fn a_digit_without_that_association_passes_to_the_host() {
    let mut typing = Typing::new();
    let committed = typing.commit("an");
    let beyond = u8::try_from(committed.candidates.len() + 1).unwrap();
    let digit = typing.key(Key::Digit(beyond));
    assert!(!digit.handled);
    assert!(digit.candidates.is_empty());
}

#[test]
fn punctuation_closes_the_association_and_starts_none() {
    let mut typing = Typing::new();
    typing.commit("an");
    let comma = typing.key(Key::Punct(','));
    assert_eq!(comma.commit.as_deref(), Some("，"));
    assert!(comma.candidates.is_empty());
}

#[test]
fn committing_the_raw_code_or_with_punctuation_gives_no_association() {
    let mut typing = Typing::new();
    typing.key(Key::Letter('a'));
    typing.key(Key::Letter('n'));
    let entered = typing.key(Key::Enter);
    assert_eq!(entered.commit.as_deref(), Some("an"));
    assert!(entered.candidates.is_empty());

    typing.key(Key::Letter('a'));
    typing.key(Key::Letter('n'));
    let punct = typing.key(Key::Punct('.'));
    assert_eq!(punct.commit.as_deref(), Some("門。"));
    assert!(punct.candidates.is_empty());
}

#[test]
fn clicking_an_association_chooses_it() {
    let mut typing = Typing::new();
    let committed = typing.commit("an");
    let at = position(&committed, "口").unwrap();
    let chosen = typing.session.select(&typing.engine, &mut typing.user, at);
    assert_eq!(chosen.commit.as_deref(), Some("口"));
}

#[test]
fn the_char_bigram_offers_following_characters() {
    let mut typing = Typing::new();
    let committed = typing.commit("an");
    // 「門外」不在詞庫，只有字級 bigram 給得出「外」。
    assert!(texts(&committed).contains(&"外"), "{:?}", texts(&committed));
}

#[test]
fn associations_are_not_sent_for_reranking() {
    let mut typing = Typing::new();
    typing.commit("an");
    assert!(typing.session.rerank_request().is_none());
    assert!(
        typing
            .session
            .apply_rerank(&typing.engine, 0, &[1, 0])
            .is_none()
    );
}

#[test]
fn turning_association_off_while_shown_closes_it_on_refresh() {
    let mut typing = Typing::new();
    typing.commit("an");
    let mut options = typing.session.options();
    options.association = false;
    typing.session.set_options(options);
    let refreshed = typing.session.refresh(&typing.engine, &typing.user);
    assert!(refreshed.candidates.is_empty());
    // 沒呼叫 refresh 時，下一個數字鍵也照沒有聯想交還宿主。
    let mut typing = Typing::new();
    typing.commit("an");
    typing.session.set_options(options);
    assert!(!typing.key(Key::Digit(1)).handled);
}

/// 上屏「日」、選聯想「本」再選「人」，學到個人 bigram「本 → 人」；之後單打「本」，聯想有「人」：
/// 詞庫沒有以「本」開頭的詞、字級 bigram 也沒有接在本後面的，只有個人 bigram 給得出。
fn ben_ren_after_learning(typing: &mut Typing) -> Snapshot {
    let committed = typing.commit("a");
    let ben = typing.pick(&committed, "本");
    typing.pick(&ben, "人");
    typing.key(Key::Escape);
    typing.commit("d")
}

#[test]
fn a_chosen_association_is_learned_as_a_personal_bigram() {
    let mut typing = Typing::new();
    let fresh = typing.commit("d");
    assert!(fresh.candidates.is_empty(), "{:?}", texts(&fresh));
    typing.key(Key::Escape);
    let learned = ben_ren_after_learning(&mut typing);
    assert_eq!(texts(&learned), ["人"]);
}

#[test]
fn a_word_typed_after_a_commit_is_offered_next_time() {
    let mut typing = Typing::new();
    typing.commit("an");
    // 前文以剛上屏的「門」結尾，打「日本」學到「門 → 日本」。
    typing.commit("ad");
    typing.key(Key::Escape);
    let again = typing.commit("an");
    assert!(texts(&again).contains(&"日本"), "{:?}", texts(&again));
}

#[test]
fn nothing_is_learned_from_associations_when_learning_is_off() {
    let mut typing = Typing::with_options(|o| o.learning = false);
    let learned = ben_ren_after_learning(&mut typing);
    assert!(learned.candidates.is_empty(), "{:?}", texts(&learned));
}

#[test]
fn incognito_gives_no_association() {
    // 剛上屏的字是下一個字的前文，無痕不讀前文（brief §8）。
    let mut typing = Typing::with_options(|o| o.incognito = true);
    let committed = typing.commit("an");
    assert_eq!(committed.commit.as_deref(), Some("門"));
    assert!(committed.candidates.is_empty(), "{:?}", texts(&committed));
    assert!(!typing.key(Key::Digit(1)).handled);
}

#[test]
fn clearing_the_context_closes_the_association() {
    let mut typing = Typing::new();
    typing.commit("an");
    typing.session.clear_context(&mut typing.user);
    assert!(typing.session.all_candidates().is_empty());
    assert!(!typing.key(Key::Digit(1)).handled);
}

#[test]
fn a_context_not_ending_with_the_commit_closes_the_association() {
    let mut typing = Typing::new();
    typing.commit("an");
    // 殼在上屏之後給的前文以剛上屏的字結尾：聯想留著。
    typing.session.set_context("今日門");
    assert!(!typing.session.all_candidates().is_empty());
    // 游標移走了。
    typing.session.set_context("今日");
    assert!(typing.session.all_candidates().is_empty());
    assert!(!typing.key(Key::Digit(1)).handled);
}

#[test]
fn a_modifier_keeps_the_association() {
    let mut typing = Typing::new();
    let committed = typing.commit("an");
    let modifier = typing.key(Key::Modifier);
    assert!(!modifier.handled);
    assert_eq!(texts(&modifier), texts(&committed));
}

#[test]
fn text_closes_the_association_and_starts_none() {
    let mut typing = Typing::new();
    typing.commit("an");
    let typed = typing.session.text(&typing.engine, &mut typing.user, "😀");
    assert_eq!(typed.commit.as_deref(), Some("😀"));
    assert!(typed.candidates.is_empty());
}

#[test]
fn switching_to_english_on_commit_gives_no_association() {
    let mut typing = Typing::with_options(|o| o.shift_switch = ShiftSwitch::CommitText);
    typing.key(Key::Letter('a'));
    typing.key(Key::Letter('n'));
    let shifted = typing.key(Key::ToggleAscii(ShiftKey::Right));
    assert_eq!(shifted.commit.as_deref(), Some("門"));
    assert!(shifted.candidates.is_empty());
}

#[test]
fn a_word_chosen_in_two_pieces_completes_across_them() {
    let mut typing = Typing::new();
    // `ad` 的首選是「日本」；移到第一段選「日」、再選「本」，分兩段上屏。
    typing.key(Key::Letter('a'));
    typing.key(Key::Letter('d'));
    typing.key(Key::Left);
    let ri = typing.key(Key::Left);
    let rest = typing.pick(&ri, "日");
    let committed = typing.pick(&rest, "本");
    assert_eq!(committed.commit.as_deref(), Some("日本"));
    // 「日」「本」兩段併成「日本人」，補「人」。
    assert!(texts(&committed).contains(&"人"), "{:?}", texts(&committed));
}

#[test]
fn choosing_after_association_is_switched_off_closes_it() {
    let mut typing = Typing::new();
    typing.commit("an");
    let mut options = typing.session.options();
    options.association = false;
    typing.session.set_options(options);
    let chosen = typing.session.select(&typing.engine, &mut typing.user, 0);
    assert!(!chosen.handled);
    assert!(chosen.commit.is_none());
    assert!(chosen.candidates.is_empty());
}

/// 簡繁資料包只把「本」轉成「夲」（測試用的對照，不是真的簡體）。
fn simplifier() -> SimplifierPack {
    let table = |entries: &[(&str, &str)]| {
        ConversionTable::new(entries.iter().map(|&(k, v)| (k.to_string(), v.to_string())))
    };
    SimplifierPack {
        id: "opencc-hk2s".into(),
        simplifier: Simplifier::new(vec![], table(&[]), vec![vec![table(&[("本", "夲")])]]),
    }
}

#[test]
fn simplified_mode_shows_and_commits_converted_associations_but_chains_on_the_original() {
    let mut typing = Typing::with_options(|o| o.simplified = true);
    typing.engine = engine().with_simplifier(simplifier());
    let committed = typing.commit("a");
    let at = position(&committed, "夲").expect("聯想轉換後顯示");
    assert_eq!(committed.candidates[at].original, "本");
    let chosen = typing.pick(&committed, "夲");
    assert_eq!(chosen.commit.as_deref(), Some("夲"));
    assert_eq!(chosen.commit_original.as_deref(), Some("本"));
    // 接著照原文「日本」補完。
    assert!(texts(&chosen).contains(&"人"), "{:?}", texts(&chosen));
}

#[test]
fn switching_to_simplified_while_shown_converts_the_associations() {
    let mut typing = Typing::new();
    typing.engine = engine().with_simplifier(simplifier());
    let committed = typing.commit("a");
    let at = position(&committed, "本").unwrap();
    let mut options = typing.session.options();
    options.simplified = true;
    typing.session.set_options(options);
    let refreshed = typing.session.refresh(&typing.engine, &typing.user);
    assert_eq!(refreshed.candidates[at].text, "夲");
}
