//! 萬用鍵（brief §1.5、plan §2.5）：方案檔設的萬用鍵（速成是 `z`）在輸入碼裡代替任一個方案的鍵，
//! 是碼表與詞庫的比對模式，不是容錯（Apple 倉頡的 `Z`）。每個萬用鍵剛好代一碼，放在字的碼的
//! 任一位置都行；學習記的是實際對到的碼。

mod common;

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, Engine, Key, Lexicon, LexiconPack, Packs, Ranking, Scheme, SchemePack,
    SegmentSource, Session, Snapshot, UserModel, Word,
};

/// 碼表：`a` 日、`b` 月、`n` 弓、`r` 口，`ab` 明、`an` 門、`ar` 晶、`nr` 同、`br` 骨；
/// 詞庫：`anr` 門口、`abanar` 明門晶。
fn engine(wildcard: Option<char>) -> Engine {
    let mut radicals = BTreeMap::from([
        ('a', "日".to_string()),
        ('b', "月".to_string()),
        ('n', "弓".to_string()),
        ('r', "口".to_string()),
    ]);
    if let Some(w) = wildcard {
        radicals.insert(w, "＊".into());
    }
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        wildcard,
        radicals,
        packs: Packs {
            lexicon: Some("lexicon".into()),
            language_model: Some("unigram".into()),
            ..Packs::default()
        },
        ranking: Ranking::default(),
    };
    let table = [
        ("a", '日'),
        ("b", '月'),
        ("n", '弓'),
        ("r", '口'),
        ("ab", '明'),
        ("an", '門'),
        ("ar", '晶'),
        ("nr", '同'),
        ("br", '骨'),
    ];
    let code_table = CodeTable::from_entries(table.iter().map(|&(code, text)| {
        let entry = CodeEntry {
            text,
            extension: false,
            simplified: false,
        };
        (code.to_string(), vec![entry])
    }));
    let chars = [
        ('日', 100),
        ('月', 80),
        ('弓', 5),
        ('口', 300),
        ('明', 500),
        ('門', 200),
        ('晶', 12),
        ('同', 400),
        ('骨', 10),
    ];
    let words = BTreeMap::from([
        (
            "anr".to_string(),
            vec![Word {
                text: "門口".into(),
                frequency: 1000,
            }],
        ),
        (
            "abanar".to_string(),
            vec![Word {
                text: "明門晶".into(),
                frequency: 1000,
            }],
        ),
    ]);
    let lexicon = LexiconPack {
        id: "lexicon".into(),
        lexicon: Lexicon::new(words, chars),
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
    fn new(wildcard: Option<char>) -> Self {
        Self {
            engine: engine(wildcard),
            user: UserModel::default(),
            session: Session::default(),
        }
    }

    fn press(&mut self, key: Key) -> Snapshot {
        self.session.handle(&self.engine, &mut self.user, key)
    }

    fn type_keys(&mut self, typed: &str) -> Snapshot {
        let mut snapshot = Snapshot::default();
        for c in typed.chars() {
            snapshot = self.press(Key::for_input(c));
        }
        snapshot
    }

    /// 用數字鍵選第一頁的 `text`，回傳上屏的字。
    fn choose(&mut self, shown: &Snapshot, text: &str) -> Option<String> {
        let at = texts(shown).iter().position(|t| t == text)?;
        self.press(Key::Digit(u8::try_from(at + 1).ok()?)).commit
    }
}

fn texts(snapshot: &Snapshot) -> Vec<String> {
    snapshot.candidates.iter().map(|c| c.text.clone()).collect()
}

/// 蓋滿整串的單一字詞（不含多段路徑），依候選列的順序。
fn whole(engine: &Engine, input: &str, among: &[&str]) -> Vec<String> {
    engine
        .candidates(&UserModel::default(), input)
        .into_iter()
        .filter(|t| among.contains(&t.as_str()))
        .collect()
}

#[test]
fn a_wildcard_as_the_last_code_lists_every_character_with_that_first_code() {
    let engine = engine(Some('z'));
    let all = engine.candidates(&UserModel::default(), "az");
    assert_eq!(all[0], "明", "{all:?}");
    assert_eq!(
        whole(&engine, "az", &["明", "門", "晶", "同", "骨"]),
        ["明", "門", "晶"],
        "字頻高的在前，不是 a 開頭的不出：{all:?}"
    );
}

#[test]
fn a_wildcard_as_the_first_code_lists_every_character_with_that_last_code() {
    let engine = engine(Some('z'));
    let all = engine.candidates(&UserModel::default(), "zr");
    assert_eq!(all[0], "同", "{all:?}");
    assert_eq!(
        whole(&engine, "zr", &["明", "門", "晶", "同", "骨"]),
        ["同", "晶", "骨"],
        "{all:?}"
    );
}

#[test]
fn a_wildcard_alone_stands_for_any_one_letter_code() {
    let engine = engine(Some('z'));
    assert_eq!(
        whole(&engine, "z", &["日", "月", "弓", "口", "明"]),
        ["口", "日", "月", "弓"]
    );
}

/// 單獨的萬用鍵以外，一段的碼裡萬用鍵不能多於已知的碼：`zz` 不是兩碼的字，`zzr` 對不上
/// 三碼的「門口」（`anr`；拆成一碼字的路徑也拼不出它，「門」是兩碼）。
#[test]
fn a_longer_segment_needs_at_least_as_many_known_codes_as_wildcards() {
    let engine = engine(Some('z'));
    let all = engine.candidates(&UserModel::default(), "zz");
    assert!(
        !all.iter()
            .any(|t| ["明", "門", "晶", "同", "骨"].contains(&t.as_str())),
        "{all:?}"
    );
    let all = engine.candidates(&UserModel::default(), "zzr");
    assert!(!all.iter().any(|t| t == "門口"), "{all:?}");
    assert_eq!(engine.candidates(&UserModel::default(), "znr")[0], "門口");
}

#[test]
fn a_wildcard_matches_inside_the_code_of_a_word() {
    let engine = engine(Some('z'));
    for input in ["anz", "azr", "znr"] {
        let all = engine.candidates(&UserModel::default(), input);
        assert_eq!(all[0], "門口", "{input}: {all:?}");
    }
}

#[test]
fn each_wildcard_stands_for_exactly_one_code() {
    let engine = engine(Some('z'));
    // `az` 只有兩碼，對不上三碼的「門口」（`anr`）；`anz` 對得上。
    let all = engine.candidates(&UserModel::default(), "az");
    assert!(!all.iter().any(|t| t == "門口"), "{all:?}");
    assert_eq!(engine.candidates(&UserModel::default(), "anz")[0], "門口");
}

#[test]
fn the_wildcard_starts_a_composition_and_shows_its_radical() {
    let mut ime = Ime::new(Some('z'));
    let shown = ime.type_keys("za");
    assert!(shown.handled);
    let radicals: String = shown
        .composition
        .segments
        .iter()
        .map(|s| s.radicals.as_str())
        .collect();
    assert_eq!(radicals, "＊日");
}

#[test]
fn enter_commits_the_typed_wildcard_as_it_is() {
    let mut ime = Ime::new(Some('z'));
    ime.type_keys("az");
    assert_eq!(ime.press(Key::Enter).commit.as_deref(), Some("az"));
}

#[test]
fn learning_records_the_code_the_wildcard_matched() {
    let mut ime = Ime::new(Some('z'));
    let shown = ime.type_keys("az");
    assert_eq!(ime.choose(&shown, "晶").as_deref(), Some("晶"));
    ime.type_keys("anz");
    assert_eq!(ime.press(Key::Space).commit.as_deref(), Some("門口"));

    let exported = ime.user.export();
    assert!(exported.contains("\t晶\tar\t"), "{exported}");
    assert!(exported.contains("\t門口\tanr\t"), "{exported}");
    assert!(!exported.contains('z'), "學習不記萬用鍵：{exported}");
}

/// 個人機率照實際的碼查：用 `br` 學過的「骨」，打 `zr` 時排到字頻略高的「晶」前面。
#[test]
fn a_learned_character_ranks_higher_under_the_wildcard_too() {
    let mut ime = Ime::new(Some('z'));
    let among = ["同", "晶", "骨"];
    assert_eq!(whole(&ime.engine, "zr", &among), ["同", "晶", "骨"]);
    for _ in 0..10 {
        let shown = ime.type_keys("br");
        assert_eq!(ime.choose(&shown, "骨").as_deref(), Some("骨"));
    }
    let learned: Vec<String> = ime
        .engine
        .candidates(&ime.user, "zr")
        .into_iter()
        .filter(|t| among.contains(&t.as_str()))
        .collect();
    assert_eq!(learned, ["同", "骨", "晶"]);
}

#[test]
fn without_a_wildcard_in_the_scheme_z_is_swallowed_during_a_composition() {
    let mut ime = Ime::new(None);
    let before = texts(&ime.type_keys("a"));
    let shown = ime.press(Key::Letter('z'));
    assert!(shown.handled);
    assert_eq!(texts(&shown), before);
    assert_eq!(ime.press(Key::Enter).commit.as_deref(), Some("a"));

    let mut idle = Ime::new(None);
    assert!(!idle.press(Key::Letter('z')).handled, "沒有組字時交還宿主");
}

/// debug 面板（M4 票 01）標萬用鍵：有萬用鍵的段帶它實際對到的碼，其他段沒有。
#[test]
fn inspect_shows_the_code_each_wildcard_matched() {
    let mut ime = Ime::new(Some('z'));
    ime.type_keys("az");
    let inspection = ime.session.inspect(&ime.engine, &ime.user);
    let matched = |text: &str| {
        let candidate = inspection
            .candidates
            .iter()
            .find(|c| c.text == text)
            .unwrap_or_else(|| panic!("沒有 {text}"));
        candidate
            .segments
            .iter()
            .map(|s| (s.code.clone(), s.matched_code.clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(matched("晶"), [("az".into(), Some("ar".into()))]);
    assert_eq!(matched("明"), [("az".into(), Some("ab".into()))]);

    let mut plain = Ime::new(Some('z'));
    plain.type_keys("ar");
    let inspection = plain.session.inspect(&plain.engine, &plain.user);
    assert!(
        inspection
            .candidates
            .iter()
            .flat_map(|c| &c.segments)
            .all(|s| s.matched_code.is_none())
    );
}

/// 一段最多兩個萬用鍵：`azazar` 對得上六碼的「明門晶」，`zbznzr` 有三個就對不上。
#[test]
fn a_segment_has_at_most_two_wildcards() {
    let engine = engine(Some('z'));
    assert_eq!(
        engine.candidates(&UserModel::default(), "azazar")[0],
        "明門晶"
    );
    let all = engine.candidates(&UserModel::default(), "zbznzr");
    assert!(!all.iter().any(|t| t == "明門晶"), "{all:?}");
}

/// 分段選字後的自動造詞也記實際的碼：`az` 選「晶」、`ab` 選「明」，造的是 `arab` → 晶明。
#[test]
fn an_auto_phrase_records_the_codes_the_wildcards_matched() {
    let mut ime = Ime::new(Some('z'));
    let shown = ime.type_keys("azab");
    assert_eq!(ime.choose(&shown, "晶"), None, "選了開頭一段還沒上屏");
    let shown = ime.session.refresh(&ime.engine, &ime.user);
    assert_eq!(ime.choose(&shown, "明").as_deref(), Some("晶明"));

    let exported = ime.user.export();
    assert!(exported.contains("\t晶明\tarab\t"), "{exported}");
    assert!(!exported.contains('z'), "學習不記萬用鍵：{exported}");
}

/// 分隔記號加萬用鍵：詞照實際的碼看分隔落不落在字界。`an'z` 的分隔在「門」（`an`）「口」（`r`）
/// 之間，仍是一個詞；`a'nz` 的分隔在「門」的碼裡，詞不跨過它。
#[test]
fn a_separator_with_a_wildcard_splits_by_the_matched_codes() {
    let mut ime = Ime::new(Some('z'));
    let shown = ime.type_keys("an'z");
    assert_eq!(texts(&shown)[0], "門口", "{:?}", texts(&shown));
    assert_eq!(ime.press(Key::Space).commit.as_deref(), Some("門口"));

    let mut ime = Ime::new(Some('z'));
    let all = texts(&ime.type_keys("a'nz"));
    assert!(!all.iter().any(|t| t.contains('門')), "{all:?}");
}

/// 和學習時限（M4 票 13）一起成立：一年沒用的自動造詞只停用，在萬用鍵之下也不當一段使用者詞；
/// 用萬用鍵分段重選後照實際的碼 `arab` 重新啟用，學習資料裡仍沒有 `z`。
#[test]
fn an_expired_auto_phrase_stays_off_under_the_wildcard_until_chosen_again() {
    let date = |text: &str| text.parse::<qim_core::Date>().unwrap();
    let phrase_segments = |ime: &mut Ime| {
        ime.session.handle(&ime.engine, &mut ime.user, Key::Escape);
        ime.type_keys("azab");
        let inspection = ime.session.inspect(&ime.engine, &ime.user);
        let found = inspection
            .candidates
            .iter()
            .find(|c| c.text == "晶明")
            .map(|c| {
                c.segments
                    .iter()
                    .map(|s| (s.matched_code.clone(), s.source))
                    .collect::<Vec<_>>()
            });
        ime.press(Key::Escape);
        found
    };
    let row = |user: &UserModel| {
        user.export()
            .lines()
            .find(|l| l.starts_with("entry\tquick\t晶明\tarab\t"))
            .map(String::from)
    };
    let mut ime = Ime::new(Some('z'));
    ime.user.set_today(date("2026-01-01"));
    let shown = ime.type_keys("azab");
    ime.choose(&shown, "晶");
    let shown = ime.session.refresh(&ime.engine, &ime.user);
    assert_eq!(ime.choose(&shown, "明").as_deref(), Some("晶明"));
    assert_eq!(
        phrase_segments(&mut ime),
        Some(vec![(Some("arab".into()), SegmentSource::UserWord)]),
        "學過的自動造詞是一段使用者詞"
    );

    ime.user.set_today(date("2027-01-02"));
    // 停用後不是一段使用者詞：拆成兩字的路徑排不上候選列時整個不在，排得上時是碼表的字。
    let stopped = phrase_segments(&mut ime);
    assert!(
        stopped
            .iter()
            .flatten()
            .all(|(_, source)| *source != SegmentSource::UserWord),
        "停用的自動造詞不當一段使用者詞：{stopped:?}"
    );
    assert!(row(&ime.user).is_some(), "停用但不刪");

    let shown = ime.type_keys("azab");
    ime.choose(&shown, "晶");
    let shown = ime.session.refresh(&ime.engine, &ime.user);
    assert_eq!(ime.choose(&shown, "明").as_deref(), Some("晶明"));
    let row = row(&ime.user).unwrap();
    assert!(
        row.contains("2027-01-02"),
        "重選後照實際的碼更新日期：{row}"
    );
    assert!(!ime.user.export().contains('z'), "學習不記萬用鍵");
    assert_eq!(
        phrase_segments(&mut ime),
        Some(vec![(Some("arab".into()), SegmentSource::UserWord)])
    );
}
