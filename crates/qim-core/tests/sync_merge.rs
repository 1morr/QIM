//! 跨裝置合併（M4 票 17，plan〈學習與排位〉〈跨裝置合併〉，ADR 0007）：各台分開記的次數合計時相加、刪除留墓碑
//! （只存雜湊）、刪除後再用會復活、tick 合併時換算。殼的同步是「各自匯出快照、匯入時合併」，這裡照做。
//! 交換律、結合律、冪等的性質測試在 `properties.rs`。

mod common;

use std::collections::{BTreeMap, BTreeSet};

use qim_core::{
    CodeEntry, CodeTable, Date, Engine, Key, Lexicon, LexiconPack, Merged, Packs, Ranking, Scheme,
    SchemePack, Session, Snapshot, UserModel,
};

/// `ab` 的字：「甲」字頻高是首選，「乙」「丙」在後；`c`「丁」、`d`「戊」給前文用。
fn engine() -> Engine {
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
    let table = [("ab", "甲乙丙"), ("c", "丁"), ("d", "戊")];
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
    let chars = [('甲', 1000), ('乙', 10), ('丙', 1), ('丁', 1), ('戊', 1)];
    let lexicon = LexiconPack {
        id: "lexicon".into(),
        lexicon: Lexicon::new(BTreeMap::new(), chars),
    };
    let continuous: BTreeSet<char> = table.iter().flat_map(|(_, c)| c.chars()).collect();
    let model = common::unigram("model", &chars, continuous.len());
    Engine::new(
        SchemePack { scheme, code_table },
        Some(lexicon),
        Some(model),
    )
    .unwrap()
}

fn date(text: &str) -> Date {
    text.parse().unwrap()
}

/// 一台裝置：自己的個人模型（裝置 id、今天）與 Session。
struct Device {
    engine: Engine,
    session: Session,
    user: UserModel,
}

impl Device {
    fn new(id: &str) -> Self {
        let mut user = UserModel::default();
        user.set_device(id).unwrap();
        user.set_today(date("2026-11-12"));
        Self {
            engine: engine(),
            session: Session::default(),
            user,
        }
    }

    fn press(&mut self, key: Key) -> Snapshot {
        self.session.handle(&self.engine, &mut self.user, key)
    }

    fn type_code(&mut self, code: &str) -> Snapshot {
        let mut shown = Snapshot::default();
        for c in code.chars() {
            shown = self.press(Key::Letter(c));
        }
        shown
    }

    /// 打 `code`、選文字是 `text` 的候選上屏。
    fn choose(&mut self, code: &str, text: &str) {
        let shown = self.type_code(code);
        let at = texts(&shown).iter().position(|t| t == text);
        let at = at.unwrap_or_else(|| panic!("沒有「{text}」：{:?}", texts(&shown)));
        let done = self.press(Key::Digit(u8::try_from(at + 1).unwrap()));
        assert_eq!(done.commit.as_deref(), Some(text));
    }

    /// 打 `code` 的首選（之後清掉組字）。
    fn top(&mut self, code: &str) -> String {
        let shown = self.type_code(code);
        self.press(Key::Escape);
        texts(&shown)[0].clone()
    }

    /// 組字 `code` 中清掉文字是 `text` 的候選學到的（撤銷學習）。
    fn forget(&mut self, code: &str, text: &str) {
        self.type_code(code);
        let at = self
            .session
            .all_candidates()
            .iter()
            .position(|c| c.text == text);
        let forgotten = self
            .session
            .forget(&self.engine, &mut self.user, at.unwrap());
        assert!(forgotten.handled);
        self.press(Key::Escape);
    }

    /// 匯入 `other` 匯出的快照、合併進來（殼的同步）。
    fn sync_from(&mut self, other: &Device) -> Merged {
        let snapshot = UserModel::import(&other.user.export(), self.user.today()).unwrap();
        assert_eq!(snapshot.warnings, []);
        self.user.merge(snapshot.user)
    }

    fn export(&self) -> String {
        self.user.export()
    }
}

fn texts(snapshot: &Snapshot) -> Vec<String> {
    snapshot.candidates.iter().map(|c| c.text.clone()).collect()
}

/// 兩台互相同步：`a` 先合併 `b` 的，`b` 再合併 `a` 合併後的。
fn sync_both(a: &mut Device, b: &mut Device) {
    a.sync_from(b);
    b.sync_from(a);
}

/// 匯出檔裡開頭是 `prefix` 的列拆成欄。
fn row(export: &str, prefix: &str) -> Option<Vec<String>> {
    let line = export.lines().find(|l| l.starts_with(prefix))?;
    Some(line.split('\t').map(String::from).collect())
}

/// 匯出檔裡刪掉的（方案, 輸入碼, 詞）列（只有雜湊）有幾列。
fn deleted_rows(export: &str) -> usize {
    export
        .lines()
        .filter(|l| l.starts_with("deleted\tentry\t"))
        .count()
}

#[test]
fn two_devices_choosing_a_candidate_once_each_make_it_the_top_after_syncing() {
    let (mut a, mut b) = (Device::new("phone"), Device::new("desktop"));
    a.choose("ab", "丙");
    b.choose("ab", "丙");
    assert_eq!(
        (a.top("ab"), b.top("ab")),
        ("甲".into(), "甲".into()),
        "各選一次還不前移"
    );

    sync_both(&mut a, &mut b);
    assert_eq!(a.top("ab"), "丙", "兩台各一次合計兩次");
    assert_eq!(b.top("ab"), "丙");
    // 各台一格：裝置表 0 是 desktop、1 是 phone。
    let counts = row(&a.export(), "entry\tquick\t丙\tab\t").unwrap()[4].clone();
    assert_eq!(counts, "0:1:1:0:2026-11-12,1:1:1:0:2026-11-12");
    assert_eq!(learning(&a), learning(&b), "兩邊讀回一致");
}

/// 學習資料：匯出檔去掉這台的裝置 id。
fn learning(device: &Device) -> String {
    device
        .export()
        .lines()
        .filter(|l| !l.starts_with("#@/device\t"))
        .map(|l| format!("{l}\n"))
        .collect()
}

#[test]
fn merging_ones_own_snapshot_or_the_same_snapshot_again_changes_nothing() {
    let (mut a, mut b) = (Device::new("phone"), Device::new("desktop"));
    a.choose("ab", "丙");
    b.choose("ab", "乙");
    let before = a.export();
    let own = UserModel::import(&before, a.user.today()).unwrap().user;
    assert_eq!(a.user.merge(own), Merged::default());
    assert_eq!(a.export(), before);

    let first = a.sync_from(&b);
    assert_eq!(first.added, 1, "{first:?}");
    let merged = a.export();
    assert_eq!(a.sync_from(&b), Merged::default());
    assert_eq!(a.export(), merged);
}

#[test]
fn a_deletion_reaches_the_other_device_without_keeping_the_text() {
    let (mut a, mut b) = (Device::new("phone"), Device::new("desktop"));
    a.choose("ab", "丙");
    a.choose("ab", "丙");
    b.sync_from(&a);
    assert_eq!(b.top("ab"), "丙");

    b.forget("ab", "丙");
    assert_eq!(b.top("ab"), "甲");
    let merged = a.sync_from(&b);
    assert_eq!(merged.deleted, 1, "{merged:?}");
    assert_eq!(a.top("ab"), "甲", "刪除傳過來");
    for export in [a.export(), b.export()] {
        assert!(
            row(&export, "entry\tquick\t丙\t").is_none(),
            "墓碑不留原文：{export}"
        );
        assert_eq!(deleted_rows(&export), 1, "{export}");
    }
    assert_eq!(learning(&a), learning(&b));

    // 舊的快照（刪除前的）再合併一次也不會讓它回來。
    let mut old = Device::new("phone");
    old.choose("ab", "丙");
    old.choose("ab", "丙");
    b.sync_from(&old);
    assert_eq!(b.top("ab"), "甲");
}

#[test]
fn a_deleted_candidate_comes_back_when_used_again_and_needs_two_choices_to_top() {
    let (mut a, mut b) = (Device::new("phone"), Device::new("desktop"));
    a.choose("ab", "丙");
    b.choose("ab", "丙");
    sync_both(&mut a, &mut b);
    b.forget("ab", "丙");
    a.sync_from(&b);

    a.choose("ab", "丙");
    assert!(
        !a.export().lines().any(|l| l.starts_with("deleted\t")),
        "復活後不在墓碑表"
    );
    assert_eq!(a.top("ab"), "甲", "刪除前兩台的次數不算");
    b.sync_from(&a);
    assert_eq!(b.top("ab"), "甲");
    b.choose("ab", "丙");
    assert_eq!(b.top("ab"), "丙", "刪除後兩台各選一次");
    a.sync_from(&b);
    assert_eq!(a.top("ab"), "丙");
    assert_eq!(learning(&a), learning(&b));
}

/// 沒看到刪除的那台之後又用過（刪除時沒見過的次數）：同步後存活，但刪除前的次數照樣不算。
#[test]
fn a_use_on_a_device_that_had_not_seen_the_deletion_revives_it() {
    let (mut a, mut b) = (Device::new("phone"), Device::new("desktop"));
    a.choose("ab", "丙");
    a.choose("ab", "丙");
    b.sync_from(&a);
    b.forget("ab", "丙");
    a.choose("ab", "丙");
    sync_both(&mut a, &mut b);
    assert!(
        row(&b.export(), "entry\tquick\t丙\tab\t").is_some(),
        "{}",
        b.export()
    );
    assert_eq!(b.top("ab"), "甲", "刪除後只選過一次");
    a.choose("ab", "丙");
    b.sync_from(&a);
    assert_eq!(b.top("ab"), "丙");
}

#[test]
fn forgetting_a_manual_word_clears_its_counts_on_both_devices_and_keeps_the_word() {
    let (mut a, mut b) = (Device::new("phone"), Device::new("desktop"));
    let engine = engine();
    a.user.add_word(&engine, "丙丁", Some("yy")).unwrap();
    a.choose("yy", "丙丁");
    b.sync_from(&a);
    assert!(row(&b.export(), "entry\tquick\t丙丁\tyy\t").is_some_and(|f| !f[7].is_empty()));

    b.forget("yy", "丙丁");
    a.sync_from(&b);
    for device in [&mut a, &mut b] {
        let export = device.export();
        let learned = row(&export, "entry\tquick\t丙丁\tyy\t");
        assert!(
            learned.is_none_or(|f| f[7].is_empty()),
            "計數清掉：{export}"
        );
        assert!(export.contains("\t丙丁\t"), "詞留著：{export}");
        assert_eq!(device.top("yy"), "丙丁", "觸發詞照樣是首選");
    }
}

#[test]
fn a_removed_phrase_is_removed_on_the_other_device_and_its_text_is_gone() {
    let (mut a, mut b) = (Device::new("phone"), Device::new("desktop"));
    let engine = engine();
    a.user
        .add_phrase(&engine, "九龍彌敦道 1 號", Some("kk"))
        .unwrap();
    b.sync_from(&a);
    assert_eq!(b.user.phrases(&engine).len(), 1);

    assert!(b.user.remove_phrase(&engine, "九龍彌敦道 1 號"));
    let merged = a.sync_from(&b);
    assert_eq!(merged.deleted, 1, "{merged:?}");
    assert!(a.user.phrases(&engine).is_empty());
    assert!(!a.export().contains("彌敦道"), "{}", a.export());

    // 再存一次（明確加入）就回來，同步到另一台也是。
    a.user
        .add_phrase(&engine, "九龍彌敦道 1 號", Some("kk"))
        .unwrap();
    b.sync_from(&a);
    assert_eq!(b.user.phrases(&engine).len(), 1);
}

/// 撤銷學習只清「現在的前一個字詞 → 它」那組個人 bigram（M4 票 16），墓碑也只記那一組：同步後別的
/// 前一個字詞後面學到的在兩台都還在。
#[test]
fn forgetting_clears_only_the_bigram_after_the_current_previous_word_on_every_device() {
    let (mut a, mut b) = (Device::new("phone"), Device::new("desktop"));
    for previous in ["丁", "戊"] {
        a.session.set_context(previous);
        a.choose("ab", "丙");
    }
    b.sync_from(&a);
    b.session.set_context("丁");
    b.forget("ab", "丙");
    a.sync_from(&b);
    for export in [a.export(), b.export()] {
        assert!(row(&export, "bigram\t丁\t丙\t").is_none(), "{export}");
        assert!(row(&export, "bigram\t戊\t丙\t").is_some(), "{export}");
    }
    assert_eq!(learning(&a), learning(&b));
}

/// 一年沒用的條目停用、不刪（M4 票 13）：合併到時鐘晚一年的裝置照樣留著，時鐘改回來就恢復作用。
#[test]
fn an_entry_unused_for_a_year_is_kept_disabled_by_a_merge() {
    let (mut a, mut b) = (Device::new("phone"), Device::new("desktop"));
    a.user.set_today(date("2026-01-01"));
    a.choose("ab", "丙");
    a.choose("ab", "丙");
    b.user.set_today(date("2027-06-01"));
    b.sync_from(&a);
    assert_eq!(b.top("ab"), "甲", "停用");
    assert!(row(&b.export(), "entry\tquick\t丙\tab\t").is_some(), "不刪");
    b.user.set_today(date("2026-06-01"));
    assert_eq!(b.top("ab"), "丙");
}

/// 一台過了一年才再整串上屏：那台自己的次數從 1 重算，別台一年內的照算（各台各自的時限）。
#[test]
fn each_device_counts_its_own_whole_commits_within_a_year() {
    let (mut a, mut b) = (Device::new("phone"), Device::new("desktop"));
    a.user.set_today(date("2026-01-01"));
    a.choose("ab", "丙");
    a.choose("ab", "丙");
    b.user.set_today(date("2027-03-01"));
    b.sync_from(&a);
    b.choose("ab", "丙");
    assert_eq!(b.top("ab"), "甲", "phone 的兩次過了一年，desktop 只有一次");
    b.choose("ab", "丙");
    assert_eq!(b.top("ab"), "丙");
}

/// 合併是使用者明確的操作：之前上屏的撤銷紀錄丟掉，接著的退格不撤銷（M4 票 15）。
#[test]
fn backspace_after_a_merge_does_not_undo_learning() {
    let (mut a, b) = (Device::new("phone"), Device::new("desktop"));
    a.choose("ab", "丙");
    a.sync_from(&b);
    let before = a.export();
    a.press(Key::Backspace);
    assert_eq!(a.export(), before);
}

#[test]
fn tombstones_are_collected_a_year_after_the_deletion() {
    let (mut a, mut b) = (Device::new("phone"), Device::new("desktop"));
    a.user.set_today(date("2026-01-01"));
    a.choose("ab", "丙");
    a.choose("ab", "丙");
    let stale = UserModel::import(&a.export(), None).unwrap().user;
    a.forget("ab", "丙");
    b.user.set_today(date("2026-12-31"));
    b.sync_from(&a);
    assert_eq!(deleted_rows(&b.export()), 1, "保留期內留著");

    b.user.set_today(date("2027-01-02"));
    b.sync_from(&Device::new("tablet"));
    assert_eq!(deleted_rows(&b.export()), 0, "過了一年回收：{}", b.export());
    // 回收後才回來的舊快照：刪除前用過的條目也過了一年，回來也是停用的。
    b.user.merge(stale);
    assert_eq!(b.top("ab"), "甲");
}

/// tick 只在本機有意義：合併 tick 小很多的別台時，換算成距「現在」的上屏次數，別台剛選兩次的照樣是首選
/// （排位規則 2 的 1 萬次期限）。
#[test]
fn merging_keeps_how_many_commits_ago_each_choice_was() {
    let mut a = Device::new("phone");
    let busy = "# qim-userdict v3\n#@/device\tphone\n#@/tick/quick\t50000\n";
    a.user = UserModel::import(busy, a.user.today()).unwrap().user;
    let mut b = Device::new("desktop");
    b.choose("ab", "丙");
    b.choose("ab", "丙");
    a.sync_from(&b);
    assert_eq!(a.top("ab"), "丙");
    assert!(
        a.export().contains("#@/tick/quick\t50000\n"),
        "{}",
        a.export()
    );
}

/// 兩台各自刪掉同一個詞，刪除前又各有對方沒見過的次數：合併後那個條目沒有任何值、只剩次數，不論合併的
/// 順序都只存雜湊（[`Merged`] 的結果和順序無關）。
#[test]
fn concurrent_deletions_stay_deleted_in_any_merge_order() {
    let (mut a, mut b, mut c) = (Device::new("a"), Device::new("b"), Device::new("c"));
    a.choose("ab", "丙");
    c.sync_from(&a);
    a.forget("ab", "丙");
    b.choose("ab", "丙");
    b.forget("ab", "丙");
    let snapshot = |d: &Device| UserModel::import(&d.export(), d.user.today()).unwrap().user;
    let mut left = snapshot(&a);
    left.merge(snapshot(&c));
    left.merge(snapshot(&b));
    let mut right = snapshot(&c);
    right.merge(snapshot(&b));
    let mut right_then_a = snapshot(&a);
    right_then_a.merge(right);
    let strip = |u: &UserModel| -> String {
        u.export()
            .lines()
            .filter(|l| !l.starts_with("#@/device\t"))
            .map(|l| format!("{l}\n"))
            .collect()
    };
    assert_eq!(strip(&left), strip(&right_then_a));
    assert!(!strip(&left).contains("丙"), "{}", strip(&left));
}

/// 常用語與手動加詞不受一年時限（一直有效），它們的墓碑不回收：刪掉的常用語過了一年後，沒同步的舊快照
/// 也不會帶著原文回來（M3 票 14 的保證）。
#[test]
fn a_removed_phrase_never_comes_back_from_an_old_snapshot() {
    let (mut a, mut b) = (Device::new("phone"), Device::new("desktop"));
    let engine = engine();
    a.user.set_today(date("2026-01-01"));
    a.user.add_phrase(&engine, "九龍彌敦道 1 號", None).unwrap();
    let stale = UserModel::import(&a.export(), None).unwrap().user;
    assert!(a.user.remove_phrase(&engine, "九龍彌敦道 1 號"));
    b.user.set_today(date("2026-01-02"));
    b.sync_from(&a);

    b.user.set_today(date("2028-01-01"));
    b.sync_from(&Device::new("tablet"));
    b.user.merge(stale);
    assert!(b.user.phrases(&engine).is_empty());
    assert!(!b.export().contains("彌敦道"), "{}", b.export());
}

// 清空學習資料要同步到別台（使用者 2026-11-13 決定）：清空留一個清空記號（代數），合併時代數大的那邊
// 才算數，代數小的（清空前的）整份不收；之後學的照收。

#[test]
fn after_clearing_an_old_snapshot_brings_nothing_back() {
    let (mut a, mut b) = (Device::new("phone"), Device::new("desktop"));
    a.choose("ab", "丙");
    b.choose("ab", "丙");
    sync_both(&mut a, &mut b);
    let old_b = UserModel::import(&b.export(), None).unwrap().user;
    a.user.clear();
    assert_eq!(a.top("ab"), "甲");
    let merged = a.user.merge(old_b);
    assert_eq!(merged, Merged::default(), "清空前的快照不收");
    assert_eq!(a.top("ab"), "甲");
    assert!(
        !a.export().lines().any(|l| l.starts_with("entry\t")),
        "{}",
        a.export()
    );
    assert!(
        a.export().contains("#@/cleared\t1\t2026-11-12\tphone\n"),
        "{}",
        a.export()
    );
}

#[test]
fn a_device_merging_a_cleared_snapshot_drops_what_it_learned_before_and_keeps_later_learning() {
    let (mut a, mut b) = (Device::new("phone"), Device::new("desktop"));
    a.choose("ab", "丙");
    b.sync_from(&a);
    // B 在 A 清空前又學了（A 沒看過），也一起清掉。
    b.choose("ab", "乙");
    b.choose("ab", "乙");
    a.user.clear();
    a.choose("ab", "丙");
    let merged = b.sync_from(&a);
    assert!(merged.deleted >= 2 && merged.added >= 1, "{merged:?}");
    assert_eq!(
        b.top("ab"),
        "甲",
        "清空前的學習清掉，清空後的一次還不成首選"
    );
    assert!(
        row(&b.export(), "entry\tquick\t乙\tab\t").is_none(),
        "{}",
        b.export()
    );
    // 收到清空之後學的照收。
    b.choose("ab", "丙");
    assert_eq!(b.top("ab"), "丙");
    a.sync_from(&b);
    assert_eq!(a.top("ab"), "丙");
    assert_eq!(learning(&a), learning(&b));
}

/// 兩台各自清空（同一代）：各自清空後學的都留著。
#[test]
fn clearing_on_both_devices_keeps_what_each_learned_after() {
    let (mut a, mut b) = (Device::new("phone"), Device::new("desktop"));
    a.choose("ab", "乙");
    b.choose("ab", "乙");
    a.user.clear();
    b.user.clear();
    a.choose("ab", "丙");
    b.choose("ab", "丙");
    sync_both(&mut a, &mut b);
    assert_eq!((a.top("ab"), b.top("ab")), ("丙".into(), "丙".into()));
    assert_eq!(learning(&a), learning(&b));
}

// 從備份還原（協調者 2026-11-13 決定）：和匯入（合併）分開的動作。還原是這台的新事件：留一個新的清空記號，
// 再把備份的條目當成之後學到的收進來，所以清空前做的備份也還原得回來，別台的舊資料同步後也不回來。

#[test]
fn restoring_a_backup_after_clearing_brings_its_learning_back() {
    let mut a = Device::new("phone");
    a.choose("ab", "丙");
    a.choose("ab", "丙");
    let backup = UserModel::import(&a.export(), None).unwrap().user;
    a.user.clear();
    assert_eq!(a.top("ab"), "甲");
    a.user.restore(backup);
    assert_eq!(a.top("ab"), "丙");
    assert!(
        a.export().contains("#@/cleared\t2\t2026-11-12\tphone\n"),
        "{}",
        a.export()
    );
}

#[test]
fn after_restoring_other_devices_old_data_does_not_come_back() {
    let (mut a, mut b) = (Device::new("phone"), Device::new("desktop"));
    a.choose("ab", "丙");
    a.choose("ab", "丙");
    let backup = UserModel::import(&a.export(), None).unwrap().user;
    b.choose("ab", "乙");
    b.choose("ab", "乙");
    a.user.restore(backup);
    a.sync_from(&b);
    assert_eq!(a.top("ab"), "丙", "別台還原前的不收");
    b.sync_from(&a);
    assert_eq!(b.top("ab"), "丙", "別台合併後換成還原的資料");
    assert!(
        row(&b.export(), "entry\tquick\t乙\tab\t").is_none(),
        "{}",
        b.export()
    );
    assert_eq!(learning(&a), learning(&b));
}

/// 備份來自代數較大的一代（別台清空後的備份）也還原得了：代數取兩者較大的加一。
#[test]
fn a_backup_from_a_newer_generation_restores_too() {
    let (mut a, mut b) = (Device::new("phone"), Device::new("desktop"));
    b.user.clear();
    b.user.clear();
    b.choose("ab", "丙");
    b.choose("ab", "丙");
    let backup = UserModel::import(&b.export(), None).unwrap().user;
    a.user.restore(backup);
    assert_eq!(a.top("ab"), "丙");
    assert!(a.export().contains("#@/cleared\t3\t"), "{}", a.export());
}
