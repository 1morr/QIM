//! 連打（plan §2.2、§2.3）經公開介面的行為：輸入碼切成字與詞的 lattice，用同一把尺打分。

mod common;

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, Engine, Key, Lexicon, LexiconPack, Pack, Packs, Ranking, Scheme, Session,
    Snapshot, UserModel, Word,
};

/// 碼表的一列：碼、字（依建置時的字序），`+` 之後的字是擴展區。
type Row<'a> = (&'a str, &'a str);

struct Data<'a> {
    table: Vec<Row<'a>>,
    words: Vec<(&'a str, &'a str, u64)>,
    chars: Vec<(char, u64)>,
    ranking: Ranking,
}

fn engine(data: &Data) -> Engine {
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        radicals: BTreeMap::new(),
        packs: Packs {
            lexicon: Some("lexicon".into()),
            language_model: Some("unigram".into()),
        },
        ranking: data.ranking,
    };
    let code_table = CodeTable::from_entries(data.table.iter().map(|(code, chars)| {
        let (continuous, extension) = chars.split_once('+').unwrap_or((chars, ""));
        let entries = continuous
            .chars()
            .map(|text| (text, false))
            .chain(extension.chars().map(|text| (text, true)))
            .map(|(text, extension)| CodeEntry { text, extension })
            .collect();
        (code.to_string(), entries)
    }));
    let mut words: BTreeMap<String, Vec<Word>> = BTreeMap::new();
    for &(code, text, frequency) in &data.words {
        words.entry(code.into()).or_default().push(Word {
            text: text.into(),
            frequency,
        });
    }
    let lexicon = LexiconPack {
        id: "lexicon".into(),
        lexicon: Lexicon::new(words, data.chars.iter().copied()),
    };
    let continuous = code_table
        .iter()
        .flat_map(|(_, entries)| entries.iter().filter(|e| !e.extension).map(|e| e.text))
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let language_model = common::unigram("unigram", &data.chars, continuous);
    Engine::new(
        Pack { scheme, code_table },
        Some(lexicon),
        Some(language_model),
    )
    .unwrap()
}

struct Ime {
    engine: Engine,
    user: UserModel,
    session: Session,
}

impl Ime {
    fn new(data: &Data) -> Self {
        Self {
            engine: engine(data),
            user: UserModel::default(),
            session: Session::default(),
        }
    }

    fn press(&mut self, key: Key) -> Snapshot {
        self.session.handle(&self.engine, &mut self.user, key)
    }

    /// 按數字鍵選 `shown` 這頁上文字是 `text` 的候選。
    fn select(&mut self, shown: &Snapshot, text: &str) -> Snapshot {
        let at = shown.candidates.iter().position(|c| c.text == text);
        let at = at.unwrap_or_else(|| panic!("這頁沒有「{text}」：{:?}", texts(shown)));
        self.press(Key::Digit(u8::try_from(at + 1).unwrap()))
    }

    fn focus_segment(&mut self, segment: usize) -> Snapshot {
        self.session
            .focus_segment(&self.engine, &mut self.user, segment)
    }

    fn type_code(&mut self, code: &str) -> Snapshot {
        let mut snapshot = Snapshot::default();
        for c in code.chars() {
            snapshot = self.press(Key::Letter(c));
        }
        snapshot
    }
}

fn texts(snapshot: &Snapshot) -> Vec<String> {
    snapshot.candidates.iter().map(|c| c.text.clone()).collect()
}

fn top(snapshot: &Snapshot) -> &str {
    snapshot.candidates.first().map_or("", |c| c.text.as_str())
}

#[test]
fn letters_with_no_character_of_their_own_compose_one_character_each() {
    let data = Data {
        table: vec![("a", "日"), ("b", "月")],
        words: vec![],
        chars: vec![],
        ranking: Ranking::default(),
    };
    let mut ime = Ime::new(&data);
    let snapshot = ime.type_code("ab");
    assert_eq!(top(&snapshot), "日月");
    assert_eq!(ime.press(Key::Space).commit.as_deref(), Some("日月"));
}

/// `anr`（日弓口）的小資料：「問」的速成碼是 `ar`、字頻高；「門口」是詞（brief §12.2 的案例 2）。
fn men_kou() -> Data<'static> {
    Data {
        table: vec![
            ("a", "日曰"),
            ("n", "弓"),
            ("r", "口"),
            ("an", "門"),
            ("ar", "問"),
            ("nr", "弔"),
        ],
        words: vec![("anr", "門口", 9435), ("anr", "明叩", 3)],
        chars: vec![
            ('日', 900),
            ('曰', 10),
            ('弓', 20),
            ('口', 500),
            ('門', 300),
            ('問', 1200),
            ('弔', 5),
        ],
        ranking: Ranking::default(),
    }
}

#[test]
fn anr_tops_men_kou_over_characters_that_split_the_code() {
    let mut ime = Ime::new(&men_kou());
    assert_eq!(top(&ime.type_code("anr")), "門口");
}

#[test]
fn candidates_cover_the_whole_input_first_then_the_beginning_longest_first() {
    let mut ime = Ime::new(&men_kou());
    let all = texts(&ime.type_code("anr"));
    let men = all.iter().position(|t| t == "門").unwrap();
    let ri = all.iter().position(|t| t == "日").unwrap();
    let ming_kou = all.iter().position(|t| t == "明叩").unwrap();
    assert!(ming_kou < men && men < ri, "{all:?}");
}

#[test]
fn selecting_a_beginning_fixes_that_segment_and_composes_the_rest_again() {
    let mut ime = Ime::new(&men_kou());
    let typed = ime.type_code("anr");
    let selected = ime.select(&typed, "門");
    assert_eq!(selected.commit, None);
    assert_eq!(selected.composition.input, "anr");
    let segments: Vec<(&str, &str, bool)> = selected
        .composition
        .segments
        .iter()
        .map(|s| (s.code.as_str(), s.text.as_str(), s.selected))
        .collect();
    assert_eq!(segments, [("an", "門", true), ("r", "口", false)]);
    assert_eq!(selected.composition.focus, 2);
    assert_eq!(texts(&selected), ["口"]);

    assert_eq!(ime.press(Key::Space).commit.as_deref(), Some("門口"));
}

#[test]
fn letters_typed_after_selecting_a_beginning_extend_the_rest() {
    let mut ime = Ime::new(&men_kou());
    ime.type_code("an");
    // 「門」是 `an` 的首選，選第一個就把整串選完上屏；所以先多打一碼再選開頭。
    let typed = ime.type_code("r");
    ime.select(&typed, "門");
    let more = ime.type_code("a");
    assert_eq!(more.composition.input, "anra");
    assert_eq!(top(&more), "口日");
    assert_eq!(ime.press(Key::Space).commit.as_deref(), Some("門口日"));
}

#[test]
fn backspace_after_the_rest_is_gone_puts_the_selected_segment_back() {
    let mut ime = Ime::new(&men_kou());
    let typed = ime.type_code("anr");
    ime.select(&typed, "門");

    let back = ime.press(Key::Backspace);
    assert_eq!(back.composition.input, "an");
    assert!(back.composition.segments.iter().all(|s| !s.selected));
    assert_eq!(top(&back), "門");
}

#[test]
fn escape_drops_selected_segments_too() {
    let mut ime = Ime::new(&men_kou());
    let typed = ime.type_code("anr");
    ime.select(&typed, "門");
    let cleared = ime.press(Key::Escape);
    assert_eq!(cleared.composition.input, "");
    assert!(cleared.composition.segments.is_empty());
    assert_eq!(ime.press(Key::Space).commit, None);
}

#[test]
fn extension_characters_stay_out_of_paths_and_only_follow_an_exact_code() {
    let data = Data {
        table: vec![("a", "日+𠀀"), ("b", "月")],
        words: vec![],
        chars: vec![('日', 1), ('月', 1), ('𠀀', 1_000_000)],
        ranking: Ranking::default(),
    };
    let mut ime = Ime::new(&data);
    assert_eq!(texts(&ime.type_code("a")), ["日", "𠀀"]);
    let two = ime.type_code("b");
    assert!(
        texts(&two).iter().all(|t| !t.contains('𠀀')),
        "{:?}",
        texts(&two)
    );
    assert_eq!(top(&two), "日月");
}

#[test]
fn word_bonus_and_segment_penalty_from_the_scheme_decide_between_a_word_and_characters() {
    let mut data = Data {
        table: vec![("a", "日"), ("b", "月")],
        words: vec![("ab", "明月", 1), ("ab", "日夜", 1000)],
        chars: vec![('日', 1000), ('月', 1000)],
        ranking: Ranking::default(),
    };
    // β、γ 為 0：「日夜」佔詞頻的絕大部分，分數接近 0，勝過兩個字。
    assert_eq!(top(&Ime::new(&data).type_code("ab")), "日夜");
    // β 為負：兩個字勝；再加上很大的 γ：一段的詞扣得少，又勝回來。
    data.ranking.word_bonus = -10.0;
    assert_eq!(top(&Ime::new(&data).type_code("ab")), "日月");
    data.ranking.segment_penalty = 10.0;
    assert_eq!(top(&Ime::new(&data).type_code("ab")), "日夜");
}

#[test]
fn input_no_path_can_cover_is_one_unselected_segment_without_text() {
    // `d` 沒有字：沒有路徑蓋得滿 `ad`，候選只剩開頭的 `a`，目前分段仍指得到。
    let mut ime = Ime::new(&men_kou());
    let snapshot = ime.type_code("ad");
    let segments: Vec<(&str, &str, bool)> = snapshot
        .composition
        .segments
        .iter()
        .map(|s| (s.code.as_str(), s.text.as_str(), s.selected))
        .collect();
    assert_eq!(segments, [("ad", "", false)]);
    assert_eq!(snapshot.composition.focus, 1);
    assert_eq!(top(&snapshot), "日");

    // 空白選了開頭的「日」：固定下來，`d` 自己成一段。
    let selected = ime.press(Key::Space);
    assert_eq!(selected.commit, None);
    let segments: Vec<(&str, &str, bool)> = selected
        .composition
        .segments
        .iter()
        .map(|s| (s.code.as_str(), s.text.as_str(), s.selected))
        .collect();
    assert_eq!(segments, [("a", "日", true), ("d", "", false)]);
    assert_eq!(selected.composition.focus, 2);
    assert!(selected.candidates.is_empty());
}

// 組字中的其他鍵（brief §4.3）：都吃下，組字不留給宿主。

#[test]
fn enter_commits_the_input_code_as_typed() {
    let mut ime = Ime::new(&men_kou());
    ime.type_code("anr");
    let entered = ime.press(Key::Enter);
    assert!(entered.handled);
    assert_eq!(entered.commit.as_deref(), Some("anr"));
    assert_eq!(entered.composition.input, "");
    assert!(entered.candidates.is_empty());
}

#[test]
fn enter_after_selecting_a_beginning_commits_its_text_and_the_rest_as_typed() {
    // librime express_editor 的 Return（`Editor::CommitRawInput`）：已選定的段上屏字，其餘上屏原碼。
    let mut ime = Ime::new(&men_kou());
    let typed = ime.type_code("anr");
    ime.select(&typed, "門");
    assert_eq!(ime.press(Key::Enter).commit.as_deref(), Some("門r"));
}

#[test]
fn z_toggle_ascii_and_other_keys_are_swallowed_and_change_nothing() {
    for key in [Key::Letter('z'), Key::ToggleAscii, Key::Other] {
        let mut ime = Ime::new(&men_kou());
        let typed = ime.type_code("anr");
        let pressed = ime.press(key);
        assert!(pressed.handled, "{key:?}");
        assert_eq!(pressed.commit, None, "{key:?}");
        assert_eq!(pressed.composition, typed.composition, "{key:?}");
        assert_eq!(texts(&pressed), texts(&typed), "{key:?}");
    }
}

#[test]
fn punctuation_commits_the_composition_then_its_full_width_form() {
    let mut ime = Ime::new(&men_kou());
    ime.type_code("anr");
    let pressed = ime.press(Key::Punct(','));
    assert!(pressed.handled);
    assert_eq!(pressed.commit.as_deref(), Some("門口，"));
    assert_eq!(pressed.composition.input, "");
    assert!(pressed.candidates.is_empty());
}

#[test]
fn punctuation_commits_selected_segments_and_the_top_of_the_rest() {
    let mut ime = Ime::new(&men_kou());
    let typed = ime.type_code("anra");
    ime.select(&typed, "門");
    assert_eq!(
        ime.press(Key::Punct('.')).commit.as_deref(),
        Some("門口日。")
    );
}

#[test]
fn punctuation_commits_the_top_beginnings_in_turn_and_code_with_no_character_as_typed() {
    // 和一直按空白相同：`ad` 的首選只蓋住 `a`，選了「日」之後 `d` 沒有字，原樣上屏。
    let mut ime = Ime::new(&men_kou());
    ime.type_code("ad");
    assert_eq!(ime.press(Key::Punct('?')).commit.as_deref(), Some("日d？"));
}

#[test]
fn punctuation_without_a_full_width_form_follows_the_composition_as_typed() {
    let mut ime = Ime::new(&men_kou());
    ime.type_code("anr");
    assert_eq!(ime.press(Key::Punct('@')).commit.as_deref(), Some("門口@"));
}

#[test]
fn an_uppercase_letter_commits_the_composition_then_itself() {
    // librime express_editor 的 `DirectCommit`：不是方案的字元先上屏組字，再照打那個字元。
    let mut ime = Ime::new(&men_kou());
    ime.type_code("anr");
    let pressed = ime.press(Key::Letter('A'));
    assert!(pressed.handled);
    assert_eq!(pressed.commit.as_deref(), Some("門口A"));
    assert_eq!(pressed.composition.input, "");
}

#[test]
fn without_composition_full_width_punctuation_is_committed_and_other_keys_go_to_the_host() {
    let mut ime = Ime::new(&men_kou());
    let comma = ime.press(Key::Punct(','));
    assert!(comma.handled);
    assert_eq!(comma.commit.as_deref(), Some("，"));

    for key in [
        Key::Punct('@'),
        Key::Letter('A'),
        Key::Letter('z'),
        Key::Enter,
        Key::Left,
        Key::Right,
        Key::ToggleAscii,
        Key::Other,
    ] {
        let pressed = ime.press(key);
        assert!(!pressed.handled, "{key:?}");
        assert_eq!(pressed.commit, None, "{key:?}");
    }
}

// 組字中修改（plan §2.1、brief §4.3 的 ← / →、§4.4 點組字的某一段）。

/// `abcd`：「日朋水」勝過「日月林」；單打 `cd` 時「林」勝過「木水」。
fn peng() -> Data<'static> {
    Data {
        table: vec![
            ("a", "日"),
            ("b", "月"),
            ("c", "木"),
            ("d", "水"),
            ("bc", "朋"),
            ("cd", "林"),
        ],
        words: vec![],
        chars: vec![
            ('日', 100),
            ('月', 10),
            ('木', 10),
            ('水', 50),
            ('朋', 1000),
            ('林', 50),
        ],
        ranking: Ranking::default(),
    }
}

fn segments(snapshot: &Snapshot) -> Vec<(&str, &str, bool)> {
    snapshot
        .composition
        .segments
        .iter()
        .map(|s| (s.code.as_str(), s.text.as_str(), s.selected))
        .collect()
}

#[test]
fn left_and_right_move_between_segments_and_list_the_words_of_the_focused_one() {
    let mut ime = Ime::new(&peng());
    let typed = ime.type_code("abcd");
    assert_eq!(
        segments(&typed),
        [("a", "日", false), ("bc", "朋", false), ("d", "水", false)]
    );
    // 沒移過時目前分段在最後一段之後：候選列整串。
    assert_eq!(typed.composition.focus, 3);
    assert_eq!(top(&typed), "日朋水");

    let last = ime.press(Key::Left);
    assert!(last.handled);
    assert_eq!(last.commit, None);
    assert_eq!(last.composition.focus, 2);
    assert_eq!(texts(&last), ["水"]);

    let middle = ime.press(Key::Left);
    assert_eq!(middle.composition.focus, 1);
    assert_eq!(texts(&middle), ["朋", "月木", "月"]);
    assert_eq!(segments(&middle), segments(&typed));

    ime.press(Key::Left);
    let first = ime.press(Key::Left);
    assert_eq!(first.composition.focus, 0, "最前面再按 ← 不動");
    assert_eq!(texts(&first), ["日"]);

    for _ in 0..3 {
        ime.press(Key::Right);
    }
    let back = ime.press(Key::Right);
    assert_eq!(back.composition.focus, 3, "最後面再按 → 不動");
    assert_eq!(texts(&back), texts(&typed));
}

#[test]
fn choosing_in_a_middle_segment_fixes_it_and_those_before_and_composes_the_rest_again() {
    let mut ime = Ime::new(&peng());
    ime.type_code("abcd");
    ime.press(Key::Left);
    let middle = ime.press(Key::Left);
    // 「朋」（bc）改成只蓋住 `b` 的「月」：`c` 和後面的 `d` 一起重算，單打 `cd` 是「林」。
    let chosen = ime.select(&middle, "月");
    assert_eq!(chosen.commit, None);
    assert_eq!(
        segments(&chosen),
        [("a", "日", true), ("b", "月", true), ("cd", "林", false)]
    );
    assert_eq!(chosen.composition.focus, 3);
    assert_eq!(texts(&chosen), texts(&Ime::new(&peng()).type_code("cd")));

    assert_eq!(ime.press(Key::Space).commit.as_deref(), Some("日月林"));
}

#[test]
fn choosing_the_last_segment_when_the_rest_is_fixed_commits_everything() {
    let mut ime = Ime::new(&peng());
    ime.type_code("abcd");
    let last = ime.press(Key::Left);
    assert_eq!(ime.select(&last, "水").commit.as_deref(), Some("日朋水"));
}

#[test]
fn space_on_a_focused_segment_commits_the_whole_composition() {
    let mut ime = Ime::new(&peng());
    ime.type_code("abcd");
    ime.press(Key::Left);
    ime.press(Key::Left);
    let pressed = ime.press(Key::Space);
    assert_eq!(pressed.commit.as_deref(), Some("日朋水"));
    assert_eq!(pressed.composition.input, "");
}

#[test]
fn rechoosing_a_selected_segment_keeps_the_selected_ones_after_it() {
    let mut ime = Ime::new(&peng());
    let typed = ime.type_code("abcd");
    // 依序選「日」「朋」，`d` 還沒選。
    let first = ime.select(&typed, "日");
    let second = ime.select(&first, "朋");
    assert_eq!(
        segments(&second),
        [("a", "日", true), ("bc", "朋", true), ("d", "水", false)]
    );
    let focused = ime.focus_segment(0);
    assert_eq!(focused.composition.focus, 0);
    assert_eq!(texts(&focused), ["日"]);
    let back = ime.focus_segment(1);
    let rechosen = ime.select(&back, "月木");
    assert_eq!(
        segments(&rechosen),
        [
            ("a", "日", true),
            ("b", "月", true),
            ("c", "木", true),
            ("d", "水", false)
        ]
    );
}

#[test]
fn rechoosing_the_beginning_of_a_selected_segment_composes_everything_after_it_again() {
    let mut ime = Ime::new(&peng());
    let typed = ime.type_code("abcd");
    let first = ime.select(&typed, "日");
    ime.select(&first, "朋");
    let focused = ime.focus_segment(1);
    let chosen = ime.select(&focused, "月");
    assert_eq!(
        segments(&chosen),
        [("a", "日", true), ("b", "月", true), ("cd", "林", false)]
    );
}

#[test]
fn focus_segment_jumps_directly_and_ignores_segments_out_of_range() {
    let mut ime = Ime::new(&peng());
    let typed = ime.type_code("abcd");
    let jumped = ime.focus_segment(1);
    assert!(jumped.handled);
    assert_eq!(jumped.composition.focus, 1);
    assert_eq!(texts(&jumped), ["朋", "月木", "月"]);
    assert_eq!(ime.focus_segment(9).composition.focus, 1);
    let whole = ime.focus_segment(3);
    assert_eq!(whole.composition.focus, 3);
    assert_eq!(texts(&whole), texts(&typed));
}

#[test]
fn typing_or_deleting_on_a_focused_segment_returns_to_the_whole_input() {
    let mut ime = Ime::new(&peng());
    ime.type_code("abc");
    ime.focus_segment(0);
    let typed = ime.type_code("d");
    assert_eq!(typed.composition.focus, 3);
    assert_eq!(top(&typed), "日朋水");

    ime.focus_segment(0);
    let deleted = ime.press(Key::Backspace);
    assert_eq!(deleted.composition.input, "abc");
    assert_eq!(
        deleted.composition.focus,
        deleted.composition.segments.len()
    );
    assert_eq!(texts(&deleted), texts(&Ime::new(&peng()).type_code("abc")));
}

#[test]
fn without_composition_focus_segment_and_select_do_nothing() {
    let mut ime = Ime::new(&peng());
    assert!(!ime.focus_segment(0).handled);
    assert!(!ime.session.select(&ime.engine, &mut ime.user, 0).handled);
}
