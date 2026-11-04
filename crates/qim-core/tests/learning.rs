//! 學習與排位（plan §2.4、brief §4.5）經公開介面的行為：上屏的字詞記進 `UserModel`，
//! 之後的候選依它前移。

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, Engine, Key, Lexicon, LexiconPack, Options, Pack, Packs, Ranking, Scheme,
    Session, Snapshot, UserModel, Word,
};

/// 碼表的一列：碼、字（依建置時的字序）。
type Row<'a> = (&'a str, &'a str);

struct Data<'a> {
    scheme: &'a str,
    table: Vec<Row<'a>>,
    words: Vec<(&'a str, &'a str, u64)>,
    chars: Vec<(char, u64)>,
}

fn engine(data: &Data) -> Engine {
    let scheme = Scheme {
        id: data.scheme.into(),
        name: data.scheme.into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        radicals: BTreeMap::new(),
        packs: Packs {
            lexicon: Some("lexicon".into()),
        },
        ranking: Ranking::default(),
    };
    let code_table = CodeTable::from_entries(data.table.iter().map(|(code, chars)| {
        let entries = chars
            .chars()
            .map(|text| CodeEntry {
                text,
                extension: false,
            })
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
    Engine::new(Pack { scheme, code_table }, Some(lexicon)).unwrap()
}

/// 一個輸入框：方案與 `Session` 各自一份，`UserModel` 由呼叫端給，可以跨方案共用。
struct Ime {
    engine: Engine,
    session: Session,
}

impl Ime {
    fn new(data: &Data) -> Self {
        Self {
            engine: engine(data),
            session: Session::default(),
        }
    }

    fn press(&mut self, user: &mut UserModel, key: Key) -> Snapshot {
        self.session.handle(&self.engine, user, key)
    }

    fn type_code(&mut self, user: &mut UserModel, code: &str) -> Snapshot {
        let mut snapshot = Snapshot::default();
        for c in code.chars() {
            snapshot = self.press(user, Key::Letter(c));
        }
        snapshot
    }

    /// 打 `code`，按數字鍵選這頁文字是 `text` 的候選，回傳選後的狀態。
    fn choose(&mut self, user: &mut UserModel, code: &str, text: &str) -> Snapshot {
        let shown = self.type_code(user, code);
        let at = shown.candidates.iter().position(|c| c.text == text);
        let at = at.unwrap_or_else(|| panic!("`{code}` 這頁沒有「{text}」：{:?}", texts(&shown)));
        self.press(user, Key::Digit(u8::try_from(at + 1).unwrap()))
    }

    /// 打 `code`，翻頁找到文字是 `text` 的候選，按數字鍵選它。
    fn choose_on_any_page(&mut self, user: &mut UserModel, code: &str, text: &str) -> Snapshot {
        let mut shown = self.type_code(user, code);
        loop {
            if let Some(at) = shown.candidates.iter().position(|c| c.text == text) {
                return self.press(user, Key::Digit(u8::try_from(at + 1).unwrap()));
            }
            assert!(shown.has_more, "`{code}` 翻完都沒有「{text}」");
            shown = self.press(user, Key::PageDown);
        }
    }

    /// 打 `code` 的首選，回傳後清掉組字。
    fn top_of(&mut self, user: &mut UserModel, code: &str) -> String {
        let shown = self.type_code(user, code);
        self.press(user, Key::Escape);
        top(&shown).to_string()
    }
}

fn texts(snapshot: &Snapshot) -> Vec<String> {
    snapshot.candidates.iter().map(|c| c.text.clone()).collect()
}

fn top(snapshot: &Snapshot) -> &str {
    snapshot.candidates.first().map_or("", |c| c.text.as_str())
}

/// `ab` 的字：「甲」字頻高，是首選；「乙」「丙」在後。另有 `c`「丁」給別的上屏用。
fn ab() -> Data<'static> {
    Data {
        scheme: "quick",
        table: vec![("ab", "甲乙丙"), ("c", "丁")],
        words: vec![],
        chars: vec![('甲', 1000), ('乙', 10), ('丙', 5), ('丁', 1)],
    }
}

#[test]
fn choosing_the_same_candidate_twice_makes_it_the_top() {
    let mut ime = Ime::new(&ab());
    let mut user = UserModel::default();
    assert_eq!(ime.top_of(&mut user, "ab"), "甲");

    assert_eq!(
        ime.choose(&mut user, "ab", "丙").commit.as_deref(),
        Some("丙")
    );
    assert_eq!(ime.top_of(&mut user, "ab"), "甲", "選一次還不前移");
    assert_eq!(
        ime.choose(&mut user, "ab", "丙").commit.as_deref(),
        Some("丙")
    );
    assert_eq!(ime.top_of(&mut user, "ab"), "丙");
}

#[test]
fn choosing_twice_works_even_with_500_other_commits_in_between() {
    let mut ime = Ime::new(&ab());
    let mut user = UserModel::default();
    ime.choose(&mut user, "ab", "丙");
    for _ in 0..500 {
        assert_eq!(
            ime.choose(&mut user, "c", "丁").commit.as_deref(),
            Some("丁")
        );
    }
    ime.choose(&mut user, "ab", "丙");
    assert_eq!(ime.top_of(&mut user, "ab"), "丙");
}

#[test]
fn the_most_recently_chosen_of_two_twice_chosen_candidates_is_the_top() {
    let mut ime = Ime::new(&ab());
    let mut user = UserModel::default();
    for text in ["乙", "乙", "丙", "丙"] {
        ime.choose(&mut user, "ab", text);
    }
    assert_eq!(ime.top_of(&mut user, "ab"), "丙");
    ime.choose(&mut user, "ab", "乙");
    assert_eq!(ime.top_of(&mut user, "ab"), "乙");
}

#[test]
fn a_candidate_last_chosen_over_10000_commits_ago_is_no_longer_moved_to_the_top() {
    let mut ime = Ime::new(&ab());
    let mut user = UserModel::default();
    ime.choose(&mut user, "ab", "丙");
    ime.choose(&mut user, "ab", "丙");
    for _ in 0..10_000 {
        ime.choose(&mut user, "c", "丁");
    }
    assert_eq!(ime.top_of(&mut user, "ab"), "丙", "剛好 10,000 次還算");
    ime.choose(&mut user, "c", "丁");
    assert_eq!(ime.top_of(&mut user, "ab"), "甲");
}

/// `anr`（日弓口）與 `kbo`（大月人）：詞庫的「明叩」「有人」詞頻高、「門口」低；
/// 「某」字頻極高，把其他字與詞的機率壓到實際詞庫的大小（10⁻⁴ 以下）。
fn men_kou() -> Data<'static> {
    Data {
        scheme: "quick",
        table: vec![
            ("a", "日"),
            ("n", "弓"),
            ("r", "口"),
            ("an", "門"),
            ("ar", "問"),
            ("k", "大"),
            ("b", "月"),
            ("o", "人"),
            ("kb", "有"),
            ("y", "某"),
        ],
        words: vec![
            ("anr", "明叩", 5_000),
            ("anr", "門口", 10),
            ("kbo", "有人", 5_000),
            ("yy", "某某", 100_000_000),
        ],
        chars: vec![
            ('日', 900),
            ('弓', 20),
            ('口', 500),
            ('門', 300),
            ('問', 1200),
            ('大', 800),
            ('月', 400),
            ('人', 900),
            ('有', 700),
            ('某', 100_000_000),
        ],
    }
}

#[test]
fn a_learned_word_also_scores_in_longer_continuous_input() {
    let mut ime = Ime::new(&men_kou());
    let mut user = UserModel::default();
    let before = ime.top_of(&mut user, "anrkbo");
    assert!(!before.contains("門口"), "學之前首選就有「門口」：{before}");

    ime.choose(&mut user, "anr", "門口");
    let after = ime.top_of(&mut user, "anrkbo");
    assert!(after.contains("門口"), "學過「門口」，首選仍是 {after}");
}

#[test]
fn choosing_twice_in_incognito_mode_learns_nothing() {
    let mut ime = Ime::new(&ab());
    let mut user = UserModel::default();
    ime.session.set_options(Options {
        incognito: true,
        ..Options::default()
    });
    ime.choose(&mut user, "ab", "丙");
    ime.choose(&mut user, "ab", "丙");
    assert_eq!(ime.top_of(&mut user, "ab"), "甲");

    ime.session.set_options(Options::default());
    ime.choose(&mut user, "ab", "丙");
    assert_eq!(ime.top_of(&mut user, "ab"), "甲", "關掉無痕後只選了一次");
}

/// 字頻排在第 21 名以後的字（beam 20 之外），學過一次後在連打裡也加分。
#[test]
fn a_learned_character_beyond_the_beam_also_scores_in_longer_input() {
    // 「倀」起連續 25 個字，字頻都一樣，依碼表的順序排。
    let many: String = (0x5000..0x5019).filter_map(char::from_u32).collect();
    let last = many.chars().last().unwrap();
    let mut chars: Vec<(char, u64)> = many.chars().map(|c| (c, 1000)).collect();
    chars.push(('某', 100_000_000));
    let data = Data {
        scheme: "quick",
        table: vec![("ab", &many), ("c", "丁"), ("y", "某")],
        words: vec![],
        chars,
    };
    let mut ime = Ime::new(&data);
    let mut user = UserModel::default();
    assert_eq!(ime.top_of(&mut user, "abc"), "倀丁");

    let chosen = ime.choose_on_any_page(&mut user, "ab", &last.to_string());
    assert_eq!(chosen.commit, Some(last.to_string()));
    assert_eq!(ime.top_of(&mut user, "abc"), format!("{last}丁"));
}
