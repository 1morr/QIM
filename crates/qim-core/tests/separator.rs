//! 手動分隔鍵（brief §4.3、plan §2.3）：組字中按 `'` 在目前位置插入分隔記號，這裡一定要斷字：
//! 單字的碼不跨過它，詞庫的詞只在它落在詞內兩字的碼之間時跨過它（照 Rime 速成，M1 票 28b）。

mod common;

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, Engine, Key, Lexicon, LexiconPack, Packs, Ranking, SEPARATOR, Scheme,
    SchemePack, Session, ShiftKey, Snapshot, UserModel, Word,
};

/// `mir`（一戈口）：「一台」是 `m` + `ir`，「歹口」是 `mi` + `r`，「一戈口」是 `m` + `i` + `r`。
/// 「一」「台」字頻高，所以不分隔時首選是「一台」；詞庫也有「一台」。
///
/// 另有語料實驗的真實例子（M1 票 28b，碼照速成碼表）：`vivdjg` 是「絃樂室」（`vi` + `vd` + `jg`），
/// 也是「幾好塞」；「幾」「好」字頻高、「幾好」是詞，靠「絃樂室」的詞頻才是首選。
/// 「門口」是 `an` + `r`（Rime 三案例）。「甲乙」是 `pqs`：「甲」有 `p`、`pq` 兩個碼，「乙」有 `qs`、`s`，兩種切法字界不同。
fn engine() -> Engine {
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        radicals: BTreeMap::from([('m', "一".into()), ('i', "戈".into()), ('r', "口".into())]),
        packs: Packs {
            lexicon: Some("lexicon".into()),
            language_model: Some("unigram".into()),
            ..Packs::default()
        },
        ranking: Ranking::default(),
    };
    let table = [
        ("m", "一"),
        ("i", "戈"),
        ("r", "口"),
        ("mi", "歹"),
        ("ir", "台"),
        ("an", "門"),
        ("v", "女"),
        ("d", "木"),
        ("vi", "幾絃"),
        ("vd", "好樂"),
        ("jg", "塞室"),
        ("p", "甲"),
        ("pq", "甲"),
        ("qs", "乙"),
        ("s", "乙"),
    ];
    let code_table = CodeTable::from_entries(table.iter().map(|(code, chars)| {
        let entries = chars
            .chars()
            .map(|text| CodeEntry {
                text,
                extension: false,
            })
            .collect();
        (code.to_string(), entries)
    }));
    let chars = [
        ('一', 1000),
        ('台', 500),
        ('口', 300),
        ('歹', 20),
        ('戈', 5),
        ('幾', 900),
        ('好', 900),
        ('塞', 50),
        ('絃', 5),
        ('樂', 100),
        ('室', 100),
        ('門', 200),
        ('女', 10),
        ('木', 10),
        ('甲', 10),
        ('乙', 10),
    ];
    let words = [
        ("mir", "一台", 100),
        ("anr", "門口", 1000),
        ("vivd", "幾好", 1000),
        ("vivdjg", "絃樂室", 1000),
        ("pqs", "甲乙", 1000),
    ];
    let mut by_code: BTreeMap<String, Vec<Word>> = BTreeMap::new();
    for (code, text, frequency) in words {
        by_code.entry(code.into()).or_default().push(Word {
            text: text.into(),
            frequency,
        });
    }
    let lexicon = LexiconPack {
        id: "lexicon".into(),
        lexicon: Lexicon::new(by_code, chars),
    };
    let language_model = common::unigram("unigram", &chars, chars.len());
    Engine::new(
        SchemePack { scheme, code_table },
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
    fn new() -> Self {
        Self {
            engine: engine(),
            user: UserModel::default(),
            session: Session::default(),
        }
    }

    fn press(&mut self, key: Key) -> Snapshot {
        self.session.handle(&self.engine, &mut self.user, key)
    }

    /// 逐鍵打 `typed`：方案的字母是 `Letter`，分隔記號是 `Punct`（實體鍵盤的 `'`）。
    fn type_keys(&mut self, typed: &str) -> Snapshot {
        let mut snapshot = Snapshot::default();
        for c in typed.chars() {
            snapshot = self.press(Key::for_input(c));
        }
        snapshot
    }
}

fn texts(snapshot: &Snapshot) -> Vec<String> {
    snapshot.candidates.iter().map(|c| c.text.clone()).collect()
}

#[test]
fn without_a_separator_mir_tops_yi_toi() {
    let mut ime = Ime::new();
    assert_eq!(texts(&ime.type_keys("mir"))[0], "一台");
}

/// `m`｜`ir` 的字界不在分隔記號的位置：詞「一台」與「一」「台」兩段都不出現。
#[test]
fn nothing_crosses_a_separator_that_is_not_on_a_boundary_between_characters() {
    let mut ime = Ime::new();
    let shown = ime.type_keys("mi'r");
    let all = texts(&shown);
    assert_eq!(all[0], "歹口", "{all:?}");
    assert!(
        !all.iter().any(|t| t == "一台"),
        "一台 跨過分隔記號：{all:?}"
    );
    assert_eq!(ime.press(Key::Space).commit.as_deref(), Some("歹口"));
}

#[test]
fn a_word_crosses_a_separator_on_the_boundary_between_its_characters() {
    let mut plain = Ime::new();
    assert_eq!(texts(&plain.type_keys("vivdjg"))[0], "絃樂室");
    // 這組測試資料只有 `m`、`i`、`r` 有字根，其餘字母與分隔記號照原字元顯示。
    for (typed, radicals) in [
        ("vi'vdjg", "v戈'vdjg"),
        ("vivd'jg", "v戈vd'jg"),
        ("vi'vd'jg", "v戈'vd'jg"),
    ] {
        let mut ime = Ime::new();
        let shown = ime.type_keys(typed);
        assert_eq!(texts(&shown)[0], "絃樂室", "{typed}");
        assert_eq!(
            segments(&shown),
            [(typed, radicals, "絃樂室", false)],
            "{typed}"
        );
        assert_eq!(ime.press(Key::Space).commit.as_deref(), Some("絃樂室"));
    }
}

/// 同 Gboard 速成（黑箱 T18）首選仍是門口；分隔落在「門」「口」之間，所以是一個詞、一段。
#[test]
fn an_r_tops_the_word_mun_hau_across_the_separator() {
    let mut ime = Ime::new();
    let shown = ime.type_keys("an'r");
    assert_eq!(texts(&shown)[0], "門口");
    assert_eq!(segments(&shown), [("an'r", "an'口", "門口", false)]);
}

#[test]
fn a_word_does_not_cross_a_separator_inside_the_code_of_one_of_its_characters() {
    let mut ime = Ime::new();
    let all = texts(&ime.type_keys("v'ivdjg"));
    assert!(
        !all.iter().any(|t| t.contains("絃樂室") || t.contains('幾')),
        "{all:?}"
    );
    let mut ime = Ime::new();
    let all = texts(&ime.type_keys("vivdj'g"));
    assert!(
        !all.iter().any(|t| t.contains("絃樂室") || t.contains('塞')),
        "{all:?}"
    );
}

#[test]
fn a_single_character_does_not_cross_a_separator() {
    let mut ime = Ime::new();
    let all = texts(&ime.type_keys("v'd"));
    assert_eq!(all[0], "女木", "{all:?}");
    assert!(!all.iter().any(|t| t == "好" || t == "樂"), "{all:?}");
}

/// 字有幾個碼時，任一種切法的字界落在分隔記號上就可以跨：詞「甲乙」是一段，不是「甲」「乙」兩段。
#[test]
fn a_word_crosses_when_any_split_of_its_code_puts_a_boundary_on_the_separator() {
    for typed in ["p'qs", "pq's"] {
        let mut ime = Ime::new();
        let shown = ime.type_keys(typed);
        assert_eq!(segments(&shown), [(typed, typed, "甲乙", false)], "{typed}");
    }
}

#[test]
fn backspace_after_a_word_across_a_separator_deletes_the_letters_then_the_separator() {
    let mut ime = Ime::new();
    ime.type_keys("vi'vd");
    let shown = ime.press(Key::Backspace);
    assert_eq!(shown.composition.input, "vi'v");
    let shown = ime.press(Key::Backspace);
    assert_eq!(shown.composition.input, "vi'");
    let shown = ime.press(Key::Backspace);
    assert_eq!(shown.composition.input, "vi");
}

/// 跨過分隔記號的詞記的碼不含分隔記號，同其他分段（plan §2.4）。
#[test]
fn learning_a_word_across_a_separator_records_its_code_without_it() {
    let mut ime = Ime::new();
    let shown = ime.type_keys("vi'vd");
    assert_eq!(segments(&shown), [("vi'vd", "v戈'vd", "幾好", false)]);
    assert_eq!(ime.press(Key::Space).commit.as_deref(), Some("幾好"));
    let exported = ime.user.export();
    let rows: Vec<(&str, &str)> = exported
        .lines()
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| {
            let mut columns = l.split('\t').skip(1);
            Some((columns.next()?, columns.next()?))
        })
        .collect();
    assert_eq!(rows, [("幾好", "vivd")], "{exported}");
}

/// 各分段的（碼, 字根, 字詞, 是否已選定）。
fn segments(snapshot: &Snapshot) -> Vec<(&str, &str, &str, bool)> {
    snapshot
        .composition
        .segments
        .iter()
        .map(|s| {
            (
                s.code.as_str(),
                s.radicals.as_str(),
                s.text.as_str(),
                s.selected,
            )
        })
        .collect()
}

#[test]
fn the_separator_stays_in_the_composition_at_the_end_of_the_segment_before_it() {
    let mut ime = Ime::new();
    let shown = ime.type_keys("mi'r");
    assert_eq!(shown.composition.input, "mi'r");
    assert_eq!(
        segments(&shown),
        [("mi'", "一戈'", "歹", false), ("r", "口", "口", false)]
    );
}

#[test]
fn backspace_deletes_the_separator_first() {
    let mut ime = Ime::new();
    ime.type_keys("mi'r");
    let shown = ime.press(Key::Backspace);
    assert_eq!(shown.composition.input, "mi'");
    // 結尾的分隔記號算在最後一段裡：候選是 `mi` 的，蓋住整串。
    assert_eq!(texts(&shown)[0], "歹");
    let shown = ime.press(Key::Backspace);
    assert_eq!(shown.composition.input, "mi");
    assert!(shown.handled);
}

#[test]
fn a_trailing_separator_lists_the_candidates_of_the_code_before_it() {
    let mut ime = Ime::new();
    let separated = texts(&ime.type_keys("mi'"));
    let mut plain = Ime::new();
    assert_eq!(separated, texts(&plain.type_keys("mi")));
    assert_eq!(ime.press(Key::Space).commit.as_deref(), Some("歹"));
}

#[test]
fn without_composition_the_separator_key_goes_to_the_host() {
    let mut ime = Ime::new();
    let shown = ime.press(Key::Punct(SEPARATOR));
    assert!(!shown.handled);
    assert_eq!(shown.commit, None);
    assert_eq!(shown.composition.input, "");
}

#[test]
fn a_second_separator_in_a_row_is_swallowed() {
    let mut ime = Ime::new();
    let shown = ime.type_keys("mi''r");
    assert!(shown.handled);
    assert_eq!(shown.composition.input, "mi'r");
}

#[test]
fn choosing_the_beginning_before_a_separator_takes_the_separator_with_it() {
    let mut ime = Ime::new();
    let shown = ime.type_keys("mi'r");
    let at = texts(&shown).iter().position(|t| t == "歹").unwrap();
    let selected = ime.press(Key::Digit(u8::try_from(at + 1).unwrap()));
    assert_eq!(
        segments(&selected),
        [("mi'", "一戈'", "歹", true), ("r", "口", "口", false)]
    );
    assert_eq!(texts(&selected)[0], "口");
    // 剩下的刪光了，選定的分段連同分隔記號放回來重選。
    let back = ime.press(Key::Backspace);
    assert_eq!(back.composition.input, "mi'");
    assert!(back.composition.segments.iter().all(|s| !s.selected));
}

#[test]
fn enter_commits_the_input_with_its_separators() {
    let mut ime = Ime::new();
    ime.type_keys("mi'r");
    assert_eq!(ime.press(Key::Enter).commit.as_deref(), Some("mi'r"));
}

#[test]
fn a_focused_segment_ending_in_a_separator_lists_the_words_of_its_code() {
    let mut ime = Ime::new();
    ime.type_keys("mi'r");
    ime.press(Key::Left);
    let focused = ime.press(Key::Left);
    assert_eq!(focused.composition.focus, 0);
    assert_eq!(texts(&focused)[0], "歹");
    let chosen = ime.press(Key::Digit(1));
    assert_eq!(texts(&chosen)[0], "口");
    assert_eq!(ime.press(Key::Space).commit.as_deref(), Some("歹口"));
}

#[test]
fn learning_records_the_codes_without_separators() {
    let mut ime = Ime::new();
    for _ in 0..2 {
        ime.type_keys("mi'r");
        assert_eq!(ime.press(Key::Space).commit.as_deref(), Some("歹口"));
    }
    let exported = ime.user.export();
    assert!(!exported.contains(SEPARATOR), "{exported}");
    // 整串記成 `mir` → 歹口：選兩次後不分隔也是首選（排位規則 2）。
    assert_eq!(texts(&ime.type_keys("mir"))[0], "歹口");
}

#[test]
fn choosing_segments_split_by_a_separator_adds_an_auto_phrase_without_it() {
    let mut ime = Ime::new();
    let shown = ime.type_keys("mi'r");
    let at = texts(&shown).iter().position(|t| t == "歹").unwrap();
    ime.press(Key::Digit(u8::try_from(at + 1).unwrap()));
    assert_eq!(ime.press(Key::Space).commit.as_deref(), Some("歹口"));
    let exported = ime.user.export();
    let auto: Vec<&str> = exported
        .lines()
        .filter(|l| l.split('\t').nth(7) == Some("auto"))
        .collect();
    assert_eq!(auto.len(), 1, "{exported}");
    assert_eq!(auto[0].split('\t').nth(2), Some("mir"));
}

#[test]
fn a_candidate_chosen_twice_with_separators_tops_the_same_separated_input() {
    let mut ime = Ime::new();
    for _ in 0..2 {
        let shown = ime.type_keys("mi'r");
        let at = texts(&shown).iter().position(|t| t == "一戈口").unwrap();
        ime.session.select(&ime.engine, &mut ime.user, at);
    }
    assert_eq!(texts(&ime.type_keys("mi'r"))[0], "一戈口");
}

#[test]
fn inline_english_with_separators_goes_back_to_the_same_composition() {
    let mut ime = Ime::new();
    ime.type_keys("mi'r");
    let english = ime.press(Key::ToggleAscii(ShiftKey::Left));
    assert!(english.ascii);
    assert_eq!(english.composition.input, "mi'r");
    let back = ime.press(Key::ToggleAscii(ShiftKey::Left));
    assert_eq!(back.commit, None);
    assert!(!back.ascii);
    assert_eq!(texts(&back)[0], "歹口");
}

#[test]
fn separators_typed_in_inline_english_split_the_chinese_composition_on_return() {
    let mut ime = Ime::new();
    ime.type_keys("mi");
    ime.press(Key::ToggleAscii(ShiftKey::Left));
    ime.type_keys("'r");
    let back = ime.press(Key::ToggleAscii(ShiftKey::Left));
    assert_eq!(back.commit, None);
    assert_eq!(back.composition.input, "mi'r");
    assert_eq!(texts(&back)[0], "歹口");
}

/// 臨時英文裡連打兩個 `'` 時中文組字裝不下（組字中不能連著兩個分隔記號），按回留碼的
/// Shift 上屏英文，同英文含其他標點時。
#[test]
fn inline_english_with_two_separators_in_a_row_is_committed_on_return() {
    let mut ime = Ime::new();
    ime.type_keys("mi");
    ime.press(Key::ToggleAscii(ShiftKey::Left));
    ime.type_keys("''r");
    let back = ime.press(Key::ToggleAscii(ShiftKey::Left));
    assert_eq!(back.commit.as_deref(), Some("mi''r"));
    assert_eq!(back.composition.input, "");
}

/// QIM 沒有輸入碼內的游標：有目前分段時按 `'` 和打字母一樣回到整串，接在輸入碼最後。
#[test]
fn a_separator_with_a_focused_segment_goes_to_the_end_like_a_letter() {
    let mut ime = Ime::new();
    ime.type_keys("mi'r");
    let focused = ime.press(Key::Left);
    assert_eq!(focused.composition.focus, 1);
    let shown = ime.press(Key::Punct(SEPARATOR));
    assert_eq!(shown.composition.input, "mi'r'");
    assert_eq!(shown.composition.focus, shown.composition.segments.len());
}
