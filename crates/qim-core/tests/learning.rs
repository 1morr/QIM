//! 學習與排位（plan §2.4、brief §4.5）經公開介面的行為：上屏的字詞記進 `UserModel`，
//! 之後的候選依它前移。

mod common;

use std::collections::{BTreeMap, BTreeSet};

use qim_core::{
    CodeEntry, CodeTable, Engine, Key, Lexicon, LexiconPack, Options, Packs, Ranking, Scheme,
    SchemePack, Session, Snapshot, UserModel, Word,
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
        wildcard: None,
        radicals: BTreeMap::new(),
        packs: Packs {
            lexicon: Some("lexicon".into()),
            language_model: Some("model".into()),
            ..Packs::default()
        },
        ranking: Ranking::default(),
    };
    let code_table = CodeTable::from_entries(data.table.iter().map(|(code, chars)| {
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
    // 字的分數由字頻造的 unigram 語言模型給，分母含碼表裡的每個字。
    let continuous: BTreeSet<char> = data.table.iter().flat_map(|(_, c)| c.chars()).collect();
    let model = common::unigram("model", &data.chars, continuous.len());
    Engine::new(
        SchemePack { scheme, code_table },
        Some(lexicon),
        Some(model),
    )
    .unwrap()
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

    /// 打 `code`，依序在當下的候選列選文字是 `picks` 各項的候選（分段選字），回傳最後的狀態。
    fn choose_segments(&mut self, user: &mut UserModel, code: &str, picks: &[&str]) -> Snapshot {
        let mut shown = self.type_code(user, code);
        for text in picks {
            let at = shown.candidates.iter().position(|c| c.text == *text);
            let at = at.unwrap_or_else(|| panic!("這頁沒有「{text}」：{:?}", texts(&shown)));
            shown = self.press(user, Key::Digit(u8::try_from(at + 1).unwrap()));
        }
        shown
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

    /// 打 `code` 第一頁的候選文字，回傳後清掉組字。
    fn candidates_of(&mut self, user: &mut UserModel, code: &str) -> Vec<String> {
        let shown = self.type_code(user, code);
        self.press(user, Key::Escape);
        texts(&shown)
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

/// `a`「日」字頻高、「曰」低，`b`「月」；`ab` 沒有自己的字，連打出「日月」「曰月」。
fn ri_yue() -> Data<'static> {
    Data {
        scheme: "quick",
        table: vec![("a", "日曰"), ("b", "月")],
        words: vec![],
        chars: vec![('日', 1000), ('曰', 10), ('月', 100)],
    }
}

/// 規則 2 只算整串輸入碼就是那個碼的上屏（使用者決定，plan §2.4）：連打句中用過的段只進
/// `personal`。
#[test]
fn a_character_used_in_sentences_does_not_count_towards_the_twice_chosen_rule() {
    let mut ime = Ime::new(&ri_yue());
    let mut user = UserModel::default();
    for _ in 0..3 {
        let committed = ime.choose_on_any_page(&mut user, "ab", "曰月");
        assert_eq!(committed.commit.as_deref(), Some("曰月"));
    }
    assert_eq!(ime.top_of(&mut user, "a"), "日");

    ime.choose(&mut user, "a", "曰");
    ime.choose(&mut user, "a", "曰");
    assert_eq!(ime.top_of(&mut user, "a"), "曰");
}

#[test]
fn using_a_twice_chosen_candidate_in_a_sentence_does_not_make_it_more_recent() {
    let mut ime = Ime::new(&ab());
    let mut user = UserModel::default();
    for text in ["乙", "乙", "丙", "丙"] {
        ime.choose(&mut user, "ab", text);
    }
    let committed = ime.choose_on_any_page(&mut user, "abc", "乙丁");
    assert_eq!(committed.commit.as_deref(), Some("乙丁"));
    assert_eq!(ime.top_of(&mut user, "ab"), "丙");
}

/// `mir`（一戈口）：詞庫的「一店」詞頻高，是首選；「歹口」（`mi` + `r`）其次；
/// 「一台」（`m` + `ir`）的「台」字頻低，排第 3。
fn yi_toi() -> Data<'static> {
    Data {
        scheme: "quick",
        table: vec![("m", "一"), ("mi", "歹"), ("ir", "台"), ("r", "口")],
        words: vec![("mir", "一店", 1_000_000)],
        chars: vec![
            ('一', 1000),
            ('歹', 200),
            ('台', 2),
            ('口', 1000),
            ('店', 300),
        ],
    }
}

/// 自動造詞（plan §2.4、brief §4.5）：分段選字上屏後學成使用者詞，下次排第 2，再選一次成首選。
#[test]
fn choosing_segment_by_segment_learns_a_word_that_is_second_then_top_after_one_more_choice() {
    let mut ime = Ime::new(&yi_toi());
    let mut user = UserModel::default();
    let shown = ime.type_code(&mut user, "mir");
    ime.press(&mut user, Key::Escape);
    let before = texts(&shown).iter().position(|t| t == "一台");
    assert!(
        before.is_none_or(|at| at > 1),
        "學之前「一台」就在前 2 位：{:?}",
        texts(&shown)
    );

    let selected = ime.choose(&mut user, "mir", "一");
    assert_eq!(selected.commit, None, "只選了開頭一段");
    let at = texts(&selected).iter().position(|t| t == "台");
    let at = at.unwrap_or_else(|| panic!("選了「一」之後沒有「台」：{:?}", texts(&selected)));
    let committed = ime.press(&mut user, Key::Digit(u8::try_from(at + 1).unwrap()));
    assert_eq!(committed.commit.as_deref(), Some("一台"));

    let shown = ime.type_code(&mut user, "mir");
    ime.press(&mut user, Key::Escape);
    assert_eq!(texts(&shown)[..2], ["一店", "一台"]);

    ime.choose(&mut user, "mir", "一台");
    assert_eq!(ime.top_of(&mut user, "mir"), "一台");
}

/// 分段選出、詞庫沒有、學之前連候選列都沒有的整串（M1 票 13 Comments 的 `abcd → XY`）：
/// 自動造詞後像詞一樣蓋滿整串，下次排第 2（規則 3），再選一次成首選（規則 2）。
#[test]
fn a_string_chosen_segment_by_segment_that_was_not_listed_is_second_then_top() {
    let data = Data {
        scheme: "quick",
        table: vec![("a", "日曰甲乙"), ("b", "月肉丙丁")],
        words: vec![],
        chars: vec![
            ('日', 1000),
            ('曰', 500),
            ('甲', 300),
            ('乙', 1),
            ('月', 1000),
            ('肉', 500),
            ('丙', 300),
            ('丁', 1),
        ],
    };
    let mut ime = Ime::new(&data);
    let mut user = UserModel::default();
    let shown = ime.candidates_of(&mut user, "ab");
    assert!(
        !shown.contains(&"乙丁".to_string()),
        "學之前「乙丁」就在候選列：{shown:?}"
    );

    let committed = ime.choose_segments(&mut user, "ab", &["乙", "丁"]);
    assert_eq!(committed.commit.as_deref(), Some("乙丁"));
    assert_eq!(ime.candidates_of(&mut user, "ab")[..2], ["日月", "乙丁"]);

    ime.choose(&mut user, "ab", "乙丁");
    assert_eq!(ime.top_of(&mut user, "ab"), "乙丁");
}

/// 規則 3：碼是整串、上屏過的使用者詞有幾個時，最近整串上屏的排第 2，其他的不往前移。
#[test]
fn of_two_used_user_words_the_most_recently_committed_is_second() {
    let data = Data {
        scheme: "quick",
        table: vec![("a", "日曰"), ("b", "月肉")],
        words: vec![],
        chars: vec![('日', 1000), ('曰', 500), ('月', 100), ('肉', 1)],
    };
    let mut ime = Ime::new(&data);
    let mut user = UserModel::default();
    ime.choose_segments(&mut user, "ab", &["日", "肉"]);
    ime.choose_segments(&mut user, "ab", &["曰", "月"]);

    let shown = ime.candidates_of(&mut user, "ab");
    assert_eq!(shown[..2], ["日月", "曰月"]);
    let older = shown.iter().position(|t| t == "日肉");
    assert!(older.is_some_and(|at| at > 1), "{shown:?}");
}

/// 規則 3 的「至少第 2 位」：本來就是首選的使用者詞釘在首選，不往後移到第 2。規則 2 不管它：
/// 上屏過 3 次，但最後一次在 10,000 次上屏之前。
#[test]
fn a_used_user_word_that_is_already_the_top_stays_the_top() {
    let data = Data {
        scheme: "quick",
        table: vec![("a", "日"), ("b", "月")],
        words: vec![],
        chars: vec![('日', 1000), ('月', 1000)],
    };
    let mut ime = Ime::new(&data);
    let mut user = UserModel::import(
        "# qim-userdict v2\n#@/tick/quick\t20000\nquick\t日月\tab\t3\t1\t1\t1\tauto\t\n",
        None,
    )
    .map(|i| i.user)
    .unwrap();
    assert_eq!(ime.candidates_of(&mut user, "ab"), ["日月", "日"]);
}

/// 手動加的使用者詞打觸發詞時是首選（排位規則 1，brief §4.5），即使觸發詞本來有更好的候選。
#[test]
fn a_manual_user_word_is_the_top_when_its_trigger_is_typed() {
    let mut ime = Ime::new(&yi_toi());
    let mut user = UserModel::default();
    assert_eq!(ime.top_of(&mut user, "mir"), "一店");

    user.add_word(&ime.engine, "台口", Some("mir")).unwrap();
    assert_eq!(ime.top_of(&mut user, "mir"), "台口");
    let committed = ime.choose(&mut user, "mir", "台口");
    assert_eq!(committed.commit.as_deref(), Some("台口"));
}

/// 使用者詞補上詞庫沒有的詞（brief §1.5）：用各字的碼接起來也打得出。`a` 有五個字、「丙」
/// 字頻最低，`aa` 連打的前 3 條路徑都輪不到「丙丙」。
#[test]
fn a_manual_user_word_can_be_typed_with_the_codes_of_its_characters() {
    let data = Data {
        scheme: "quick",
        table: vec![("a", "日曰甲乙丙")],
        words: vec![],
        chars: vec![
            ('日', 1000),
            ('曰', 500),
            ('甲', 300),
            ('乙', 200),
            ('丙', 1),
        ],
    };
    let mut ime = Ime::new(&data);
    let mut user = UserModel::default();
    let shown = ime.type_code(&mut user, "aa");
    ime.press(&mut user, Key::Escape);
    assert!(
        !texts(&shown).contains(&"丙丙".to_string()),
        "{:?}",
        texts(&shown)
    );

    user.add_word(&ime.engine, "丙丙", None).unwrap();
    let shown = ime.type_code(&mut user, "aa");
    assert!(
        texts(&shown).contains(&"丙丙".to_string()),
        "{:?}",
        texts(&shown)
    );
}

/// 字沒有碼的詞（英文、數字）只能靠觸發詞打；沒給觸發詞、觸發詞有方案以外的字、詞是空的或含
/// Tab、換行時加不進去。
#[test]
fn a_user_word_needs_a_trigger_made_of_scheme_keys_or_characters_that_have_codes() {
    let ime = Ime::new(&yi_toi());
    let mut user = UserModel::default();
    assert!(user.add_word(&ime.engine, "QIM 3", None).is_err());
    assert!(
        user.add_word(&ime.engine, "QIM 3", Some("qz")).is_err(),
        "z 不在速成的鍵裡"
    );
    assert!(user.add_word(&ime.engine, "QIM 3", Some("")).is_err());
    assert!(user.add_word(&ime.engine, "", Some("q")).is_err());
    assert!(user.add_word(&ime.engine, "一\t台", Some("q")).is_err());
    assert!(user.add_word(&ime.engine, "一\n台", Some("q")).is_err());
    assert_eq!(user, UserModel::default());

    user.add_word(&ime.engine, "QIM 3", Some("qim")).unwrap();
    let mut ime = ime;
    assert_eq!(ime.top_of(&mut user, "qim"), "QIM 3");
}

/// 「選兩次才成首選」（brief §4.5）：整串上屏一次的候選，就算學到的分數讓它勝過原本的首選，
/// 也只排第 2。`ab` 的「乙丙」「丙丙」是詞庫的詞，靜態機率都很低，學一次的個人分就夠勝出。
#[test]
fn a_candidate_committed_once_is_not_the_top_even_when_its_learned_score_is_highest() {
    let data = Data {
        scheme: "quick",
        table: vec![("a", "乙"), ("b", "丙"), ("y", "某")],
        words: vec![
            ("ab", "乙丙", 100),
            ("ab", "丙丙", 10),
            ("yy", "某某", 100_000_000),
        ],
        chars: vec![('乙', 10), ('丙', 5), ('某', 100_000_000)],
    };
    let mut ime = Ime::new(&data);
    let mut user = UserModel::default();
    assert_eq!(ime.top_of(&mut user, "ab"), "乙丙");

    ime.choose(&mut user, "ab", "丙丙");
    let shown = ime.type_code(&mut user, "ab");
    ime.press(&mut user, Key::Escape);
    assert_eq!(texts(&shown)[..2], ["乙丙", "丙丙"]);
    ime.choose(&mut user, "ab", "丙丙");
    assert_eq!(ime.top_of(&mut user, "ab"), "丙丙");
}

/// 規則 1 已把觸發詞釘在首選時，整串上屏一次的候選本來就當不了首選，規則 2 不再動它：
/// 不學習時的首選不會被多移一次、擠到它前面（M1 票 24 Comments 的「多移一次」）。
#[test]
fn a_trigger_on_top_leaves_the_once_committed_candidate_where_it_was() {
    let data = Data {
        scheme: "quick",
        table: vec![("a", "乙"), ("b", "丙"), ("y", "某")],
        words: vec![
            ("ab", "乙丙", 100),
            ("ab", "丙丙", 10),
            ("yy", "某某", 100_000_000),
        ],
        chars: vec![('乙', 10), ('丙', 5), ('某', 100_000_000)],
    };
    let mut ime = Ime::new(&data);
    let mut user = UserModel::default();
    ime.choose(&mut user, "ab", "丙丙");
    user.add_word(&ime.engine, "某乙", Some("ab")).unwrap();

    assert_eq!(
        ime.candidates_of(&mut user, "ab")[..3],
        ["某乙", "丙丙", "乙丙"]
    );
}

/// 規則 1 已把觸發詞釘在首選時，選兩次的候選照「至少首選」的放法退到第 2 位（plan §2.4 的
/// 「那些位置都排定了才移到最前面的空位」）：本來排第 3 的「丙」排到「甲」前面。
#[test]
fn a_trigger_on_top_puts_the_twice_chosen_candidate_second() {
    let mut ime = Ime::new(&ab());
    let mut user = UserModel::default();
    ime.choose(&mut user, "ab", "丙");
    ime.choose(&mut user, "ab", "丙");
    user.add_word(&ime.engine, "丁丁", Some("ab")).unwrap();
    assert_eq!(
        ime.candidates_of(&mut user, "ab")[..3],
        ["丁丁", "丙", "甲"]
    );
}

/// 本來就是首選的候選上屏一次後仍是首選：「剛好 1 次的不靠分數成首選」只擋靠學習升上來的。
#[test]
fn the_top_committed_once_stays_the_top() {
    let mut ime = Ime::new(&ab());
    let mut user = UserModel::default();
    ime.choose(&mut user, "ab", "甲");
    assert_eq!(ime.top_of(&mut user, "ab"), "甲");
}

/// 觸控鍵盤的文字鍵（`Session::text`，M1 票 20）組字中和標點相同：先上屏組字，照樣學。
#[test]
fn a_touch_text_key_learns_the_composition_like_punctuation() {
    let mut by_punct = UserModel::default();
    let mut ime = Ime::new(&men_kou());
    ime.type_code(&mut by_punct, "anr");
    let punct = ime.press(&mut by_punct, Key::Punct(','));

    let mut by_text = UserModel::default();
    let mut ime = Ime::new(&men_kou());
    ime.type_code(&mut by_text, "anr");
    let text = ime.session.text(&ime.engine, &mut by_text, "，");

    assert_eq!(text.commit, punct.commit);
    assert_ne!(by_text, UserModel::default());
    assert_eq!(by_text, by_punct);
}

/// 連打後移到第 2 段改選再上屏（M1 票 21 的組字中修改）也是分兩次以上選定：前面的分段照
/// 顯示的字固定，整串學成自動造詞。
#[test]
fn changing_a_later_segment_before_committing_also_learns_an_auto_phrase() {
    let data = Data {
        scheme: "quick",
        table: vec![("a", "日曰"), ("b", "月肉")],
        words: vec![],
        chars: vec![('日', 1000), ('曰', 500), ('月', 100), ('肉', 0)],
    };
    let mut ime = Ime::new(&data);
    let mut user = UserModel::default();
    ime.type_code(&mut user, "ab");
    let focused = ime.session.focus_segment(&ime.engine, &mut user, 1);
    let at = texts(&focused).iter().position(|t| t == "肉").unwrap();
    let committed = ime.press(&mut user, Key::Digit(u8::try_from(at + 1).unwrap()));
    assert_eq!(committed.commit.as_deref(), Some("日肉"));

    assert!(
        user.export()
            .contains("entry\tquick\t日肉\tab\t1\t1\t1\t1\t\tauto\t\n"),
        "{}",
        user.export()
    );
    let shown = ime.type_code(&mut user, "ab");
    assert_eq!(texts(&shown)[1], "日肉");
}
