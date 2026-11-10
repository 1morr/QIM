//! 中文狀態的英文候選（M1 票 15、排位規則 4，plan §2.4、brief §4.3、§4.7）：整串輸入碼剛好是
//! 完整英文單字、又長過一個字的最長碼時，那個字排第 2 位；不容錯、不補全、不搶首選。

mod common;

use std::collections::{BTreeMap, BTreeSet};

use qim_core::{
    CodeEntry, CodeTable, Engine, EnglishPack, EnglishWords, Key, Lexicon, LexiconPack, Options,
    Pack, Packs, Ranking, Scheme, Session, ShiftKey, Snapshot, UserModel, Word,
};

/// 碼表的一列：碼、字（依字頻由高到低）。`he` 不是碼，`the` 的路徑只有 `th e` 與 `t h e`。
const TABLE: [(&str, &str); 8] = [
    ("t", "廿"),
    ("th", "甘"),
    ("h", "竹"),
    ("e", "水"),
    ("a", "日"),
    ("an", "門"),
    ("w", "田"),
    ("n", "弓"),
];

const CHARS: [(char, u64); 8] = [
    ('廿', 10),
    ('甘', 50),
    ('竹', 50),
    ('水', 100),
    ('日', 100),
    ('門', 50),
    ('田', 10),
    ('弓', 10),
];

/// 沒掛英文資料包的引擎。
fn base_engine() -> Engine {
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
            radical_hint: None,
        },
        ranking: Ranking::default(),
    };
    let code_table = CodeTable::from_entries(TABLE.iter().map(|(code, chars)| {
        let entries = chars
            .chars()
            .map(|text| CodeEntry {
                text,
                extension: false,
            })
            .collect();
        (code.to_string(), entries)
    }));
    let lexicon = LexiconPack {
        id: "lexicon".into(),
        // 分段的碼長過 2 碼只有詞：`weethe` 的首選路徑是「田水水」+「甘水」。
        lexicon: Lexicon::new(
            [(
                "wee".to_string(),
                vec![Word {
                    text: "田水水".into(),
                    frequency: 1000,
                }],
            )],
            CHARS,
        ),
    };
    let continuous: BTreeSet<char> = TABLE.iter().flat_map(|(_, c)| c.chars()).collect();
    let model = common::unigram("model", &CHARS, continuous.len());
    Engine::new(Pack { scheme, code_table }, Some(lexicon), Some(model)).unwrap()
}

fn engine() -> Engine {
    base_engine().with_english(english())
}

fn english() -> EnglishPack {
    EnglishPack {
        id: "english".into(),
        words: EnglishWords::new(["the", "an", "May", "may", "English", "NASA", "wee", "yes"]),
    }
}

struct Ime {
    engine: Engine,
    session: Session,
    user: UserModel,
}

impl Ime {
    fn new() -> Self {
        Self::with(engine())
    }

    fn with(engine: Engine) -> Self {
        Self {
            engine,
            session: Session::default(),
            user: UserModel::default(),
        }
    }

    fn press(&mut self, key: Key) -> Snapshot {
        self.session.handle(&self.engine, &mut self.user, key)
    }

    fn type_code(&mut self, code: &str) -> Snapshot {
        let mut snapshot = Snapshot::default();
        for c in code.chars() {
            snapshot = self.press(Key::Letter(c));
        }
        snapshot
    }

    /// 打 `code`，回傳第一頁的候選文字。
    fn candidates(&mut self, code: &str) -> Vec<String> {
        let shown = texts(&self.type_code(code));
        self.press(Key::Escape);
        shown
    }
}

fn texts(snapshot: &Snapshot) -> Vec<String> {
    snapshot.candidates.iter().map(|c| c.text.clone()).collect()
}

#[test]
fn a_complete_english_word_is_the_second_candidate() {
    let mut ime = Ime::new();
    let shown = ime.candidates("the");
    assert_eq!(shown[1], "the", "{shown:?}");
    assert_ne!(shown[0], "the");
}

#[test]
fn english_candidates_are_on_by_default() {
    assert!(Options::default().english_candidates);
}

#[test]
fn a_word_with_one_wrong_letter_or_a_prefix_has_no_english_candidate() {
    let mut ime = Ime::new();
    for code in ["thw", "th", "thee", "tha"] {
        let shown = ime.candidates(code);
        assert!(
            shown.iter().all(|t| !t.is_ascii()),
            "{code} 不是完整英文單字：{shown:?}"
        );
    }
}

#[test]
fn turning_the_option_off_hides_english_candidates() {
    let mut ime = Ime::new();
    ime.session.set_options(Options {
        english_candidates: false,
        ..Options::default()
    });
    let shown = ime.candidates("the");
    assert!(!shown.contains(&"the".to_string()), "{shown:?}");
}

#[test]
fn turning_the_option_off_mid_composition_takes_effect_on_refresh() {
    let mut ime = Ime::new();
    assert_eq!(texts(&ime.type_code("the"))[1], "the");
    ime.session.set_options(Options {
        english_candidates: false,
        ..Options::default()
    });
    let shown = texts(&ime.session.refresh(&ime.engine, &ime.user));
    assert!(!shown.contains(&"the".to_string()), "{shown:?}");
}

#[test]
fn an_engine_without_the_english_pack_has_no_english_candidates() {
    let mut ime = Ime::with(base_engine());
    let shown = ime.candidates("the");
    assert!(!shown.contains(&"the".to_string()), "{shown:?}");
}

#[test]
fn input_no_longer_than_one_character_code_has_no_english_candidate() {
    // 黑箱 T7：macOS 速成第 3 碼起才有英文候選，`an` 這種 2 碼單字只出中文。
    let mut ime = Ime::new();
    let shown = ime.candidates("an");
    assert!(!shown.contains(&"an".to_string()), "{shown:?}");
}

#[test]
fn pressing_two_commits_the_english_word() {
    let mut ime = Ime::new();
    ime.type_code("the");
    let snapshot = ime.press(Key::Digit(2));
    assert_eq!(snapshot.commit.as_deref(), Some("the"));
    assert_eq!(snapshot.commit_original.as_deref(), Some("the"));
    assert!(snapshot.composition.input.is_empty());
}

#[test]
fn choosing_the_english_word_is_not_learned_so_it_never_becomes_first() {
    let mut ime = Ime::new();
    for _ in 0..3 {
        ime.type_code("the");
        ime.press(Key::Digit(2));
    }
    let shown = ime.candidates("the");
    assert_eq!(shown[1], "the", "{shown:?}");
    assert_eq!(
        ime.user.export(),
        UserModel::default().export(),
        "英文候選和 Enter 一樣原樣上屏，不學"
    );
}

#[test]
fn space_still_commits_the_chinese_first_candidate() {
    let mut ime = Ime::new();
    let first = texts(&ime.type_code("the"))[0].clone();
    assert_eq!(ime.press(Key::Space).commit, Some(first));
}

#[test]
fn after_choosing_a_prefix_the_rest_can_be_an_english_word() {
    let mut ime = Ime::new();
    let shown = texts(&ime.type_code("athe"));
    let first_char = shown.iter().position(|t| t == "日").unwrap();
    let rest = ime.press(Key::Digit(u8::try_from(first_char).unwrap() + 1));
    assert_eq!(rest.composition.unselected_input(), "the");
    assert_eq!(texts(&rest)[1], "the", "{:?}", texts(&rest));
    let commit = ime.press(Key::Digit(2));
    assert_eq!(commit.commit.as_deref(), Some("日the"));
    assert_eq!(ime.user.export(), UserModel::default().export());
}

#[test]
fn a_focused_segment_lists_no_english_candidate() {
    // 分段的候選是那一段的字詞；英文候選只給整串（剩下的）輸入碼。
    let mut ime = Ime::new();
    let whole = ime.type_code("weethe");
    assert_eq!(
        whole.composition.segments[0].code, "wee",
        "{:?}",
        whole.composition
    );
    let mut snapshot = ime.press(Key::Left);
    loop {
        assert!(
            texts(&snapshot).iter().all(|t| !t.is_ascii()),
            "{:?}",
            texts(&snapshot)
        );
        if snapshot.composition.focus == 0 {
            break;
        }
        snapshot = ime.press(Key::Left);
    }
}

#[test]
fn a_used_user_word_keeps_second_and_the_english_word_goes_third() {
    let mut ime = Ime::new();
    // 選「廿」再選「竹水」上屏：自動造詞 the → 廿竹水，上屏 1 次（規則 3：至少第 2 位）。
    let shown = texts(&ime.type_code("the"));
    let nian = shown.iter().position(|t| t == "廿").unwrap();
    let rest = texts(&ime.press(Key::Digit(u8::try_from(nian).unwrap() + 1)));
    let zhu_shui = rest.iter().position(|t| t == "竹水").unwrap();
    let done = ime.press(Key::Digit(u8::try_from(zhu_shui).unwrap() + 1));
    assert_eq!(done.commit.as_deref(), Some("廿竹水"));

    let shown = ime.candidates("the");
    assert_eq!(shown[1], "廿竹水", "{shown:?}");
    assert_eq!(shown[2], "the", "{shown:?}");
}

#[test]
fn the_word_list_keys_by_lower_case_and_prefers_the_lower_case_spelling() {
    let words = english().words;
    assert_eq!(words.lookup("may"), Some("may"));
    assert_eq!(words.lookup("english"), Some("English"));
    assert_eq!(words.lookup("nasa"), Some("NASA"));
    assert_eq!(words.lookup("English"), None, "輸入碼只有小寫");
    assert_eq!(words.lookup("engl"), None, "不補全");
    assert_eq!(words.len(), 7);
}

#[test]
fn a_proper_noun_shows_with_its_capitals() {
    let mut ime = Ime::new();
    let shown = ime.candidates("nasa");
    assert_eq!(shown[1], "NASA", "{shown:?}");
}

#[test]
fn the_english_pack_round_trips_through_bytes() {
    let pack = english();
    assert_eq!(
        EnglishPack::from_bytes(&pack.to_bytes().unwrap()).unwrap(),
        pack
    );
}

#[test]
fn no_english_candidate_without_any_chinese_candidate() {
    // `y` 沒有字，`yes` 沒有任何中文候選：英文候選不搶首選，所以也不單獨出現。
    let mut ime = Ime::new();
    assert_eq!(ime.candidates("yes"), Vec::<String>::new());
}

#[test]
fn changing_the_option_without_refresh_takes_effect_on_the_next_key() {
    let mut ime = Ime::new();
    assert_eq!(texts(&ime.type_code("the"))[1], "the");
    ime.session.set_options(Options {
        english_candidates: false,
        ..Options::default()
    });
    // 組字中吃下不動作的鍵：先依新選項重列。
    let shown = texts(&ime.press(Key::Other));
    assert!(!shown.contains(&"the".to_string()), "{shown:?}");
}

#[test]
fn english_mode_lists_no_candidates_and_chinese_mode_still_has_the_english_candidate() {
    // 單按 Shift 切英文直通（M1 票 23）：英文狀態下沒有組字、也沒有任何候選，字母交還宿主。
    let mut ime = Ime::new();
    assert!(ime.press(Key::ToggleAscii(ShiftKey::Right)).ascii);
    for c in "the".chars() {
        let snapshot = ime.press(Key::Letter(c));
        assert!(!snapshot.handled);
        assert!(snapshot.candidates.is_empty(), "{:?}", texts(&snapshot));
    }
    assert!(!ime.press(Key::ToggleAscii(ShiftKey::Right)).ascii);
    assert_eq!(ime.candidates("the")[1], "the");
}

#[test]
fn switching_to_english_mid_composition_commits_the_chinese_first_candidate() {
    let mut ime = Ime::new();
    let first = texts(&ime.type_code("the"))[0].clone();
    let snapshot = ime.press(Key::ToggleAscii(ShiftKey::Right));
    assert_eq!(snapshot.commit, Some(first));
    assert!(snapshot.ascii);
}

/// 殼只換選項、沒呼叫 `refresh` 就選第 2 位：關掉英文候選後第 2 位是中文，不是換選項前的 `the`。
#[test]
fn selecting_after_turning_the_option_off_without_refresh_skips_the_english_word() {
    let off = Options {
        english_candidates: false,
        ..Options::default()
    };
    let mut expected = Ime::new();
    expected.session.set_options(off);
    expected.type_code("the");
    let expected = expected.press(Key::Digit(2));

    let mut ime = Ime::new();
    assert_eq!(texts(&ime.type_code("the"))[1], "the");
    ime.session.set_options(off);
    let shown = ime.session.select(&ime.engine, &mut ime.user, 1);
    assert_ne!(shown.commit.as_deref(), Some("the"));
    assert_eq!(shown.commit, expected.commit);
    assert_eq!(shown.composition, expected.composition);
}

/// 換選項後還沒重列時，候選列馬上會換：不給重排要求，換選項前要求的結果也作廢。
#[test]
fn reranking_waits_for_the_list_of_the_new_option() {
    let mut ime = Ime::new();
    ime.type_code("the");
    let request = ime.session.rerank_request().unwrap();
    ime.session.set_options(Options {
        english_candidates: false,
        ..Options::default()
    });
    assert_eq!(ime.session.rerank_request(), None);
    assert_eq!(
        ime.session
            .apply_rerank(&ime.engine, request.generation, &[2, 1]),
        None
    );
}

/// 換選項後還沒重列時，展開面板不給舊候選：面板的位置會被 `select` 拿去選新列表，給了會選錯字。
#[test]
fn the_expanded_list_is_empty_until_relisted_with_the_new_option() {
    let mut ime = Ime::new();
    ime.type_code("the");
    ime.session.set_options(Options {
        english_candidates: false,
        ..Options::default()
    });
    assert_eq!(ime.session.all_candidates(), Vec::new());
    let shown = ime.session.refresh(&ime.engine, &ime.user);
    let all: Vec<String> = ime
        .session
        .all_candidates()
        .into_iter()
        .map(|c| c.text)
        .collect();
    assert_eq!(all[..texts(&shown).len()], texts(&shown)[..]);
    assert!(!all.contains(&"the".to_string()), "{all:?}");
}
