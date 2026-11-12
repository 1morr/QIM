//! 撤銷學習（M4 票 16，plan §2.4〈撤銷學習〉）：清掉一個候選的學習紀錄，詞庫原有的詞回到不學習時的
//! 位置、自動造詞不再出，再選可以重新學；不做永久屏蔽。

mod common;

use std::collections::{BTreeMap, BTreeSet};

use qim_core::{
    CodeEntry, CodeTable, Engine, Key, Lexicon, LexiconPack, Options, Packs, Ranking, Scheme,
    SchemePack, Session, Snapshot, UserModel, Word,
};

/// 碼表的一列：碼、字（依建置時的字序）。
type Row<'a> = (&'a str, &'a str);

struct Data<'a> {
    table: Vec<Row<'a>>,
    words: Vec<(&'a str, &'a str, u64)>,
    chars: Vec<(char, u64)>,
}

fn engine(data: &Data) -> Engine {
    let scheme = Scheme {
        id: "quick".into(),
        name: "quick".into(),
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
    let continuous: BTreeSet<char> = data.table.iter().flat_map(|(_, c)| c.chars()).collect();
    let model = common::unigram("model", &data.chars, continuous.len());
    Engine::new(
        SchemePack { scheme, code_table },
        Some(lexicon),
        Some(model),
    )
    .unwrap()
}

struct Ime {
    engine: Engine,
    session: Session,
    user: UserModel,
}

impl Ime {
    fn new(data: &Data) -> Self {
        Self {
            engine: engine(data),
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

    /// 在當下的候選列依序選文字是 `picks` 各項的候選。
    fn pick(&mut self, mut shown: Snapshot, picks: &[&str]) -> Snapshot {
        for text in picks {
            let at = position(&shown, text)
                .unwrap_or_else(|| panic!("這頁沒有「{text}」：{:?}", texts(&shown)));
            shown = self.press(Key::Digit(u8::try_from(at + 1).unwrap()));
        }
        shown
    }

    /// 打 `code`，依序選 `picks`（一項是整串、幾項是分段選字）。
    fn choose(&mut self, code: &str, picks: &[&str]) -> Snapshot {
        let shown = self.type_code(code);
        self.pick(shown, picks)
    }

    /// 打 `code` 第一頁的候選文字，回傳後清掉組字。
    fn candidates_of(&mut self, code: &str) -> Vec<String> {
        let shown = self.type_code(code);
        self.press(Key::Escape);
        texts(&shown)
    }

    /// 組字中清掉文字是 `text` 的候選的學習紀錄。
    fn forget(&mut self, text: &str) -> Snapshot {
        let all = self.session.all_candidates();
        let at = all.iter().position(|c| c.text == text);
        let at = at.unwrap_or_else(|| panic!("沒有「{text}」"));
        self.session.forget(&self.engine, &mut self.user, at)
    }

    /// debug 面板看到的 `text` 的學習加分。
    fn learning_of(&self, text: &str) -> Option<f64> {
        let inspection = self.session.inspect(&self.engine, &self.user);
        let found = inspection.candidates.into_iter().find(|c| c.text == text);
        found.and_then(|c| c.learning)
    }
}

fn texts(snapshot: &Snapshot) -> Vec<String> {
    snapshot.candidates.iter().map(|c| c.text.clone()).collect()
}

fn position(snapshot: &Snapshot, text: &str) -> Option<usize> {
    snapshot.candidates.iter().position(|c| c.text == text)
}

/// `ab` 的字：「甲」字頻高，是首選；「乙」「丙」在後。另有 `c`「丁」給前文用。
/// 匯出檔裡開頭是 `prefix` 的列拆成欄；沒有時是 `None`。
fn row(export: &str, prefix: &str) -> Option<Vec<String>> {
    let line = export.lines().find(|l| l.starts_with(prefix))?;
    Some(line.split('\t').map(String::from).collect())
}

/// 匯出檔裡刪掉的條目（墓碑表，M4 票 17）有幾列是 `of`（`entry`、`bigram`、`english`）的：同步時把刪除傳到
/// 別台，只存鍵的雜湊，不留原文。
fn deleted(export: &str, of: &str) -> usize {
    let prefix = format!("deleted\t{of}\t");
    export.lines().filter(|l| l.starts_with(&prefix)).count()
}

/// 開頭是 `prefix` 的 `entry` 列有學到的計數（`version` 到 `last_used` 五欄）。
fn has_counts(export: &str, prefix: &str) -> bool {
    row(export, prefix).is_some_and(|f| !f[7].is_empty())
}

fn ab() -> Data<'static> {
    Data {
        table: vec![("ab", "甲乙丙"), ("c", "丁")],
        words: vec![],
        chars: vec![('甲', 1000), ('乙', 10), ('丙', 5), ('丁', 1)],
    }
}

/// 學成首選的字清掉後回到不學習時的位置，現在前文下的學習加分（含接在前一個字詞後面的個人 bigram）歸零；
/// 再選兩次又成首選。
#[test]
fn a_learned_top_returns_to_its_unlearned_place_and_can_be_learned_again() {
    let mut ime = Ime::new(&ab());
    let unlearned = ime.candidates_of("ab");
    assert_eq!(unlearned, ["甲", "乙", "丙"]);
    // 前文「丁」，之後每次組字接剛上屏的「丙」：學到「丁 → 丙」「丙 → 丙」兩組個人 bigram。
    ime.choose("c", &["丁"]);
    ime.choose("ab", &["丙"]);
    ime.choose("ab", &["丙"]);
    let shown = ime.type_code("ab");
    assert_eq!(texts(&shown)[0], "丙");
    assert!(ime.learning_of("丙").is_some_and(|l| l > 0.0));

    let forgotten = ime.forget("丙");
    assert!(forgotten.handled);
    assert_eq!(texts(&forgotten), unlearned, "清掉後候選列跟著重排");
    assert_eq!(ime.learning_of("丙"), Some(0.0));
    ime.press(Key::Escape);
    assert_eq!(ime.candidates_of("ab"), unlearned);
    let export = ime.user.export();
    assert!(row(&export, "entry\tquick\t丙\t").is_none(), "{export}");
    assert!(
        row(&export, "bigram\t丙\t丙\t").is_none(),
        "現在前文那一組清掉：{export}"
    );
    assert_eq!(
        (deleted(&export, "entry"), deleted(&export, "bigram")),
        (1, 1),
        "{export}"
    );
    assert!(
        row(&export, "bigram\t丁\t丙\t").is_some_and(|f| f[4].is_empty() && !f[7].is_empty()),
        "別的前文那一組留著：{export}"
    );

    ime.choose("ab", &["丙"]);
    assert_eq!(ime.candidates_of("ab")[0], "甲", "清掉後選一次還不前移");
    ime.choose("ab", &["丙"]);
    assert_eq!(ime.candidates_of("ab")[0], "丙");
}

/// 分段選出、詞庫沒有的整串（自動造詞）：清掉後不再出；再分段選一次又學成使用者詞、排第 2，再選一次成首選。
#[test]
fn a_forgotten_auto_phrase_is_gone_and_can_be_learned_again() {
    let data = Data {
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
    ime.choose("ab", &["乙", "丁"]);
    assert_eq!(ime.candidates_of("ab")[..2], ["日月", "乙丁"]);

    ime.type_code("ab");
    let forgotten = ime.forget("乙丁");
    assert!(!texts(&forgotten).contains(&"乙丁".into()), "{forgotten:?}");
    ime.press(Key::Escape);
    assert!(!ime.candidates_of("ab").contains(&"乙丁".into()));
    let export = ime.user.export();
    assert!(!export.contains("乙丁"), "{export}");
    assert!(
        row(&export, "bigram\t乙\t丁\t").is_none(),
        "造詞裡的接續：{export}"
    );
    assert_eq!(
        (deleted(&export, "entry"), deleted(&export, "bigram")),
        (1, 1),
        "{export}"
    );

    ime.choose("ab", &["乙", "丁"]);
    assert_eq!(ime.candidates_of("ab")[..2], ["日月", "乙丁"]);
    ime.choose("ab", &["乙丁"]);
    assert_eq!(ime.candidates_of("ab")[0], "乙丁");
}

/// 句子候選只清它自己那一列（同 librime 只刪那個詞條）：各段是各自的學習，不跟著清。
#[test]
fn forgetting_a_sentence_keeps_what_its_segments_learned() {
    let data = Data {
        table: vec![("a", "日曰"), ("b", "月肉")],
        words: vec![],
        chars: vec![('日', 1000), ('曰', 500), ('月', 1000), ('肉', 500)],
    };
    let mut ime = Ime::new(&data);
    ime.choose("a", &["曰"]);
    ime.choose("ab", &["曰", "月"]);
    ime.type_code("ab");
    ime.forget("曰月");
    let export = ime.user.export();
    assert!(row(&export, "entry\tquick\t曰月\t").is_none(), "{export}");
    assert_eq!(deleted(&export, "entry"), 1, "{export}");
    assert!(has_counts(&export, "entry\tquick\t曰\ta\t"), "{export}");
}

/// 手動加的詞與常用語只清學到的計數，詞留著：打觸發詞仍是首選。
#[test]
fn a_manual_word_keeps_the_word_and_loses_only_its_counts() {
    let mut ime = Ime::new(&ab());
    ime.user.add_word(&ime.engine, "乙丁", Some("yy")).unwrap();
    ime.choose("yy", &["乙丁"]);
    assert!(has_counts(&ime.user.export(), "entry\tquick\t乙丁\tyy\t"));

    ime.type_code("yy");
    ime.forget("乙丁");
    ime.press(Key::Escape);
    let export = ime.user.export();
    assert!(!has_counts(&export, "entry\tquick\t乙丁\tyy\t"), "{export}");
    assert!(export.contains("\tmanual\tyy\t"), "{export}");
    assert_eq!(ime.candidates_of("yy")[0], "乙丁");
}

/// 沒有組字、序號超出範圍時不動作；沒有組字時鍵交還宿主（Shift+Delete 照宿主的意思）。
#[test]
fn forgetting_without_a_composition_or_out_of_range_does_nothing() {
    let mut ime = Ime::new(&ab());
    ime.choose("ab", &["丙"]);
    ime.choose("ab", &["丙"]);
    let before = ime.user.export();

    let snapshot = ime.session.forget(&ime.engine, &mut ime.user, 0);
    assert!(!snapshot.handled);
    ime.type_code("ab");
    let snapshot = ime.session.forget(&ime.engine, &mut ime.user, 99);
    assert!(snapshot.handled);
    assert_eq!(texts(&snapshot)[0], "丙");
    assert_eq!(ime.user.export(), before);
}

/// 無痕、學習關掉時也清得掉：清掉是使用者明確要的刪除，不是學習。
#[test]
fn forgetting_works_in_incognito() {
    let mut ime = Ime::new(&ab());
    ime.choose("ab", &["丙"]);
    ime.choose("ab", &["丙"]);
    ime.session.set_options(Options {
        incognito: true,
        ..Options::default()
    });
    ime.type_code("ab");
    let forgotten = ime.forget("丙");
    assert_eq!(texts(&forgotten), ["甲", "乙", "丙"]);
}

/// 自動造詞選到成首選（規則 2）再清：上屏它時學到的段間個人 bigram（乙 → 丁）一起清，同樣的字不會以句子
/// 候選回到前面（字頻高到各段學到的分數有作用時，不清的話「乙丁」回到第 2 位）。
#[test]
fn an_auto_phrase_chosen_to_the_top_does_not_come_back_as_a_sentence() {
    let data = Data {
        table: vec![("a", "日曰甲乙"), ("b", "月肉丙丁")],
        words: vec![],
        chars: vec![
            ('日', 1000),
            ('曰', 500),
            ('甲', 300),
            ('乙', 900),
            ('月', 1000),
            ('肉', 500),
            ('丙', 300),
            ('丁', 900),
        ],
    };
    let mut ime = Ime::new(&data);
    let unlearned = ime.candidates_of("ab");
    assert!(!unlearned.contains(&"乙丁".into()), "{unlearned:?}");
    ime.choose("ab", &["乙", "丁"]);
    ime.choose("ab", &["乙丁"]);
    ime.choose("ab", &["乙丁"]);
    assert_eq!(ime.candidates_of("ab")[0], "乙丁");

    ime.type_code("ab");
    assert_eq!(texts(&ime.forget("乙丁")), unlearned);
}

/// 目前分段的候選清掉後，重算的分段切法變了時回到整串，不悄悄換成另一段的候選。
#[test]
fn the_focused_segment_is_kept_only_while_its_code_stays_the_same() {
    let data = Data {
        table: vec![("a", "日曰甲乙"), ("b", "月肉丙丁")],
        words: vec![],
        chars: vec![
            ('日', 1000),
            ('曰', 500),
            ('甲', 300),
            ('乙', 900),
            ('月', 1000),
            ('肉', 500),
            ('丙', 300),
            ('丁', 900),
        ],
    };
    let mut ime = Ime::new(&data);
    ime.choose("ab", &["乙", "丁"]);
    for _ in 0..20 {
        ime.choose("ab", &["乙丁"]);
    }
    // 學過之後「乙丁」是一段 `ab`；← 移到它，清掉後首選路徑變回 `a` `b` 兩段。
    let shown = ime.type_code("ab");
    assert_eq!(shown.composition.segments.len(), 1);
    let focused = ime.press(Key::Left);
    assert_eq!(focused.composition.focus, 0);
    let forgotten = ime.forget("乙丁");
    assert_eq!(forgotten.composition.segments.len(), 2);
    assert_eq!(forgotten.composition.focus, 2, "回到整串");
    // 前幾次以句子路徑上屏時「乙」「丁」各自學到的計數不清（各段是各自的學習），所以它還在後面。
    assert_ne!(texts(&forgotten)[0], "乙丁");

    // 切法沒變時留在原來那一段：`a` 段的「乙」學成首選後清掉，還在 `a` 段。
    ime.press(Key::Escape);
    ime.choose("a", &["乙"]);
    ime.choose("a", &["乙"]);
    ime.type_code("ab");
    ime.press(Key::Left);
    let focused = ime.press(Key::Left);
    assert_eq!(focused.composition.segments[0].code, "a");
    assert_eq!(focused.composition.focus, 0);
    let forgotten = ime.forget("乙");
    assert_eq!(forgotten.composition.focus, 0);
    assert_eq!(texts(&forgotten)[0], "日");
}

/// 刪學習資料從嚴（協調者 2026-11-12 決定）：清掉「的」只清現在前文後面那一組個人 bigram，別的前文後面學到的
/// 加分還在；它自己的計數照清。
#[test]
fn forgetting_a_common_character_keeps_its_bigrams_after_other_words() {
    let data = Data {
        table: vec![("ab", "甲的"), ("c", "丁"), ("d", "戊")],
        words: vec![],
        chars: vec![('甲', 1000), ('的', 10), ('丁', 1), ('戊', 1)],
    };
    let mut ime = Ime::new(&data);
    for (code, word) in [("c", "丁"), ("d", "戊")] {
        ime.choose(code, &[word]);
        ime.choose("ab", &["的"]);
    }
    ime.choose("c", &["丁"]);
    ime.type_code("ab");
    assert!(ime.learning_of("的").is_some_and(|l| l > 0.0));
    ime.forget("的");
    assert_eq!(ime.learning_of("的"), Some(0.0), "丁 → 的 清掉");
    ime.press(Key::Escape);

    ime.choose("d", &["戊"]);
    ime.type_code("ab");
    assert!(
        ime.learning_of("的").is_some_and(|l| l > 0.0),
        "戊 → 的 的加分還在"
    );
    let export = ime.user.export();
    assert!(row(&export, "entry\tquick\t的\t").is_none(), "{export}");
    assert_eq!(deleted(&export, "entry"), 1, "{export}");
}
