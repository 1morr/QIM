//! 跨裝置合併（plan §2.4〈跨裝置合併〉，M4 票 17，ADR 0007）：每個條目各台分開記的次數（G-Counter）、
//! 刪除的墓碑，與學到的值在合併時誰留下。
//!
//! 一個條目（學習計數與使用者詞的（方案, 輸入碼, 詞）、個人 bigram 的一組、英文使用者詞）分兩部分：
//! - [`Tally`]：各台的次數與墓碑，合併時每台的格子各取較大的、墓碑取較大的；結果和合併的順序無關、
//!   重複合併不變（交換律、結合律、冪等）。
//! - 學到的值（衰減值、tick、觸發詞…）：每次寫入時記下當時的合計次數當版本（[`Tally::uses`]），合併時
//!   版本大的留下（同版本依固定的順序挑一個），版本不大於墓碑的（刪除前寫的）丟掉。合計次數只增不減，
//!   後來寫的版本一定比較大，所以刪除後再用寫的值一定蓋過別台刪除前的。

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use super::{
    Bigrams, Count, Decayed, SchemeModel, UserModel, UserWord, expired, put_in, remove_in,
};
use crate::Date;
use crate::english_user::{EnglishUserWord, EnglishUserWords};

/// 一個條目各台分開記的次數與墓碑。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Tally {
    /// 裝置 id → 那台記的次數；只有那台會加自己的格子。殼沒給裝置 id 時（與 v1、v2 匯入的資料）記在空字串
    /// 那格（不知道哪台）。
    pub(crate) cells: BTreeMap<String, Cell>,
    /// 刪除的墓碑；沒刪過是 `None`。
    pub(crate) deleted: Option<Tombstone>,
}

/// 一台記的次數。每個欄位只增不減（撤銷學習以外，ADR 0007），所以合併時逐欄取大就是那台最新的樣子。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Cell {
    /// 用過幾次：每次學到（整串或句中的一段、英文打完一次）與明確加入（手動加詞、常用語）各加一。墓碑
    /// 比的是它的合計，學到的值的版本也是它的合計。
    pub(crate) uses: u64,
    /// 整串上屏（英文：打完）的累計次數，不重算。
    pub(crate) commits: u64,
    /// `commits` 裡不算的：一年沒整串上屏後重算時與刪除時，把當時的累計記在這裡。
    pub(crate) uncounted: u64,
    /// 這台最後一次整串上屏的日期；殼沒給日期時是 `None`。
    pub(crate) last_commit: Option<Date>,
}

/// 刪除的墓碑：刪除時的合計次數與日期。合計次數大於它才存活（刪除後再用就復活）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Tombstone {
    pub(crate) uses: u64,
    /// 刪除的日期，保留期從這天算；`None` 是不回收：殼沒給日期，或刪的是不受一年時限的手動加詞、常用語。
    pub(crate) on: Option<Date>,
}

/// 合併時取較大的墓碑：合計次數大的；同次數時不回收的（沒有日期）最大，再來是日期晚的。是全序，所以取大
/// 和順序無關。
impl Ord for Tombstone {
    fn cmp(&self, other: &Self) -> Ordering {
        let key = |t: &Self| (t.uses, t.on.is_none(), t.on);
        key(self).cmp(&key(other))
    }
}

impl PartialOrd for Tombstone {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Cell {
    /// `today` 時算數的整串上屏次數：這台最後一次整串上屏距今超過一年時是 0。
    fn counted(&self, today: Option<Date>) -> u64 {
        if expired(self.last_commit, today) {
            0
        } else {
            self.commits - self.uncounted
        }
    }

    fn join(&mut self, other: &Cell) {
        self.uses = self.uses.max(other.uses);
        self.commits = self.commits.max(other.commits);
        self.uncounted = self.uncounted.max(other.uncounted);
        self.last_commit = self.last_commit.max(other.last_commit);
    }
}

impl Tally {
    /// 各台用過的次數合計。
    pub(crate) fn uses(&self) -> u64 {
        self.cells.values().map(|c| c.uses).sum()
    }

    /// 沒刪過，或刪除後又用過。
    pub(crate) fn alive(&self) -> bool {
        self.admits(self.uses())
    }

    /// 版本 `version` 寫的值在墓碑之後（還算數）。
    pub(crate) fn admits(&self, version: u64) -> bool {
        self.deleted.is_none_or(|d| version > d.uses)
    }

    /// `today` 時算數的整串上屏次數，各台相加：兩台各選一次就是兩次。
    pub(crate) fn commits_on(&self, today: Option<Date>) -> u64 {
        self.cells.values().map(|c| c.counted(today)).sum()
    }

    /// 裝置 `device` 用了一次（學到或明確加入）：自己那格加一，回傳新的合計（寫入的值的版本）。
    pub(crate) fn touch(&mut self, device: &str) -> u64 {
        self.cells.entry(device.to_string()).or_default().uses += 1;
        self.uses()
    }

    /// 裝置 `device` 在 `today` 整串上屏（英文：打完）一次：自己那格最後一次距今超過一年時從 1 重算。
    pub(crate) fn commit(&mut self, device: &str, today: Option<Date>) {
        let cell = self.cells.entry(device.to_string()).or_default();
        if expired(cell.last_commit, today) {
            cell.uncounted = cell.commits;
        }
        cell.commits += 1;
        cell.last_commit = today;
    }

    /// 在 `today` 刪除：留墓碑記下現在的合計次數，各台的整串上屏次數到此為止都不算（再選就重新數）。
    pub(crate) fn delete(&mut self, today: Option<Date>) {
        for cell in self.cells.values_mut() {
            cell.uncounted = cell.commits;
        }
        let deleted = Tombstone {
            uses: self.uses(),
            on: today,
        };
        self.deleted = self.deleted.max(Some(deleted));
    }

    /// 合併別台的：每台的格子逐欄取大，墓碑取較大的（[`Tombstone`] 的順序）。
    pub(crate) fn join(&mut self, other: &Tally) {
        for (device, cell) in &other.cells {
            self.cells.entry(device.clone()).or_default().join(cell);
        }
        self.deleted = self.deleted.max(other.deleted);
    }

    /// 回收過了保留期（刪除的日期距 `today` 超過一年，同學習的時限）的墓碑。還死著的整個條目丟掉，回傳
    /// `false`；已經復活的只丟墓碑。保留期和時限同一年，回收後才回來的舊快照，裡面刪除前用過的條目最後
    /// 使用的日期也過了一年，回來也是停用的（ADR 0007）。
    pub(crate) fn collect(&mut self, today: Option<Date>) -> bool {
        match self.deleted {
            Some(d) if expired(d.on, today) => {
                if !self.alive() {
                    return false;
                }
                self.deleted = None;
                true
            }
            _ => true,
        }
    }
}

/// 刪掉的條目在墓碑表裡的鍵：條目的鍵（種類與各欄，以 NUL 隔開）的 SHA-256 前 16 位元組，小寫十六進位。
/// 墓碑只存它、不存原文（協調者 2026-11-12 替使用者決定，維持 M3 票 14「匯出檔不留刪掉的字」）：同步時拿
/// 對方的鍵算同樣的雜湊來比。常見的短詞可以拿字典算雜湊比對出「刪過哪個詞」，地址、email 這類長串猜不到
/// （ADR 0007）。
pub(crate) fn digest(parts: &[&str]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            hasher.update([0]);
        }
        hasher.update(part.as_bytes());
    }
    hasher.finalize()[..DIGEST_BYTES]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// [`digest`] 取幾個位元組：128 位元，條目上限（幾萬）內撞到的機率可以不計。
pub(crate) const DIGEST_BYTES: usize = 16;

/// 墓碑表（刪掉的條目，鍵是 [`digest`]）裡取出 `key` 的那筆：只在表不是空的時才算雜湊。
pub(crate) fn exhume(
    graveyard: &mut BTreeMap<String, Tally>,
    key: impl FnOnce() -> String,
) -> Option<Tally> {
    if graveyard.is_empty() {
        return None;
    }
    graveyard.remove(&key())
}

/// 有版本的值（學到的值）：寫入時的合計次數。
pub(crate) trait Versioned {
    fn version(&self) -> u64;
    /// 合併時的先後：版本大的在後（留下）；同版本時依其餘欄位的固定順序，結果和合併的順序無關。
    fn order(&self, other: &Self) -> Ordering;
}

/// 合併兩邊的值：版本不大於墓碑的丟掉，其餘留 [`Versioned::order`] 最大的。
pub(crate) fn pick<T: Versioned>(mine: Option<T>, theirs: Option<T>, tally: &Tally) -> Option<T> {
    let admitted = |v: &Option<T>| v.as_ref().is_some_and(|v| tally.admits(v.version()));
    match (admitted(&mine), admitted(&theirs)) {
        (true, true) => {
            let (mine, theirs) = (mine?, theirs?);
            Some(if theirs.order(&mine) == Ordering::Greater {
                theirs
            } else {
                mine
            })
        }
        (true, false) => mine,
        (false, true) => theirs,
        (false, false) => None,
    }
}

/// 合併兩份 tick 不同的資料時，兩邊各要往後挪多少：合併後的 tick 取兩邊較大的，每個值距「現在」的
/// 上屏次數（衰減、規則 2 的期限用）照舊。tick 只在本機有意義，不同台的 tick 不能直接比（ADR 0007）。
pub(crate) fn align(mine: u64, theirs: u64) -> (u64, u64, u64) {
    let tick = mine.max(theirs);
    (tick, tick - mine, tick - theirs)
}

/// 合併一個條目的結果，給殼說明會合併幾筆（[`super::Merged`]）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Change {
    Unchanged,
    Added,
    Updated,
    Deleted,
}

impl Change {
    /// 合併前後：`before`、`after` 是條目有沒有作用中的值，`same` 是合併前後一樣。
    pub(crate) fn of(before: bool, after: bool, same: bool) -> Self {
        match (before, after) {
            _ if same => Self::Unchanged,
            (false, true) => Self::Added,
            (true, false) => Self::Deleted,
            (true, true) => Self::Updated,
            (false, false) => Self::Unchanged,
        }
    }
}

/// 合併別台的學習資料改了幾個條目（[`UserModel::merge`]），給殼在匯入前說明。條目是（方案, 輸入碼, 詞）、
/// 個人 bigram 的一組或英文使用者詞。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Merged {
    /// 這台原本沒有（或刪掉了）、合併後有的。
    pub added: usize,
    /// 兩邊都有、合併後值變了的（次數、日期、觸發詞…）。
    pub updated: usize,
    /// 這台原本有、別台刪掉了的（墓碑傳過來）。
    pub deleted: usize,
}

impl Merged {
    fn record(&mut self, change: Change) {
        match change {
            Change::Unchanged => {}
            Change::Added => self.added += 1,
            Change::Updated => self.updated += 1,
            Change::Deleted => self.deleted += 1,
        }
    }
}

impl UserModel {
    /// 把 `other`（[`UserModel::import`] 讀出來的別台或自己的快照、備份）合併進來（M4 票 17，ADR 0007）：每個
    /// 條目各台的次數逐台取大、相加是合計（兩台各選一次就是兩次）；刪除留的墓碑傳過來，合計次數大於刪除時的
    /// 合計才存活；學到的值留版本大的。結果和合併的順序無關、重複合併不變。tick 只在本機有意義，合併時對齊
    /// 到兩邊較大的（[`align`]）。`other` 的今天、裝置 id 與上限不用。
    ///
    /// 合併後回收過了保留期的墓碑（[`Tally::collect`]，照這台的今天），超過上限時照常淘汰。撤銷紀錄丟掉
    /// （使用者明確的操作）。
    pub fn merge(&mut self, other: UserModel) -> Merged {
        self.discard_undo();
        let mut merged = Merged::default();
        // 清空記號（使用者 2026-11-13 決定）：代數小的那邊是清空前的，整份不收；這台的較小時先清掉自己的
        // （記成刪除），再收對方的。同代數逐條合併，清空記號取大的。
        match self.generation().cmp(&other.generation()) {
            Ordering::Greater => return merged,
            Ordering::Less => {
                merged.deleted = self.values();
                self.replace_data(UserModel {
                    cleared: other.cleared.clone(),
                    ..UserModel::default()
                });
            }
            Ordering::Equal => self.cleared = self.cleared.clone().max(other.cleared.clone()),
        }
        let UserModel {
            schemes,
            bigrams,
            english,
            ..
        } = other;
        for (scheme, theirs) in schemes {
            self.schemes
                .entry(scheme)
                .or_default()
                .merge(theirs, &mut merged);
        }
        self.bigrams.merge(bigrams, &mut merged);
        self.english.merge(english, &mut merged);
        self.collect_tombstones();
        self.enforce_limits();
        self.discard_undo();
        merged
    }

    /// 有作用中的值的條目數（清空前合併到較新的清空記號時，記成刪除了幾筆）。
    fn values(&self) -> usize {
        let rows = self.schemes.values().map(|m| {
            let counted = m
                .entries
                .iter()
                .flat_map(|(c, t)| t.keys().map(move |t| (c, t)));
            let words = m
                .words
                .iter()
                .flat_map(|(c, t)| t.keys().map(move |t| (c, t)));
            counted.chain(words).collect::<BTreeSet<_>>().len()
        });
        let pairs = self.bigrams.entries.values().map(BTreeMap::len);
        rows.sum::<usize>() + pairs.sum::<usize>() + self.english.words.len()
    }

    /// 回收過了保留期的墓碑：墓碑表裡的整筆丟掉，還在的條目只丟墓碑（[`Tally::collect`]）。
    fn collect_tombstones(&mut self) {
        let today = self.today;
        let expired_tomb = |tally: &Tally| tally.deleted.is_some_and(|d| expired(d.on, today));
        for model in self.schemes.values_mut() {
            model.graveyard.retain(|_, tally| !expired_tomb(tally));
            let mut gone = Vec::new();
            for (code, texts) in &mut model.tallies {
                for (text, tally) in texts.iter_mut() {
                    if !tally.collect(today) {
                        gone.push((code.clone(), text.clone()));
                    }
                }
            }
            for (code, text) in gone {
                model.remove_row(&code, &text);
            }
        }
        self.bigrams
            .graveyard
            .retain(|_, tally| !expired_tomb(tally));
        self.bigrams.retain_tallies(|tally| tally.collect(today));
        let english = &mut self.english;
        english.graveyard.retain(|_, tally| !expired_tomb(tally));
        let gone: Vec<String> = english
            .tallies
            .iter_mut()
            .filter_map(|(key, tally)| (!tally.collect(today)).then(|| key.clone()))
            .collect();
        for key in gone {
            english.remove(&key);
        }
    }
}

/// 合併後的條目放哪裡：死了（合計次數不大於墓碑），或沒有任何值又有墓碑（已經沒有作用、可能也不知道原文）的
/// 放墓碑表、只存雜湊；其餘照原文放。只看合併的結果，和合併的順序無關。
fn buried(tally: &Tally, has_value: bool) -> bool {
    !tally.alive() || !has_value && tally.deleted.is_some()
}

impl SchemeModel {
    /// 合併別台的同一個方案（[`UserModel::merge`]）。
    fn merge(&mut self, theirs: SchemeModel, merged: &mut Merged) {
        let (tick, mine_by, theirs_by) = align(self.tick, theirs.tick);
        if mine_by > 0 {
            for count in self.entries.values_mut().flat_map(BTreeMap::values_mut) {
                *count = count.shifted(mine_by);
            }
        }
        self.tick = tick;
        let SchemeModel {
            mut entries,
            mut words,
            tallies,
            graveyard,
            ..
        } = theirs;
        for (code, texts) in tallies {
            for (text, their_tally) in texts {
                let their_count =
                    remove_in(&mut entries, &code, &text).map(|c| c.shifted(theirs_by));
                let their_word = remove_in(&mut words, &code, &text);
                let mine = self
                    .tally(&code, &text)
                    .cloned()
                    .or_else(|| exhume(&mut self.graveyard, || SchemeModel::digest(&code, &text)));
                self.merge_row(
                    &code,
                    &text,
                    mine,
                    &their_tally,
                    their_count,
                    their_word,
                    merged,
                );
            }
        }
        if graveyard.is_empty() {
            return;
        }
        let named: BTreeMap<String, (String, String)> = self
            .tallies
            .iter()
            .flat_map(|(code, texts)| texts.keys().map(move |text| (code, text)))
            .map(|(code, text)| {
                (
                    SchemeModel::digest(code, text),
                    (code.clone(), text.clone()),
                )
            })
            .collect();
        for (key, their_tally) in graveyard {
            match named.get(&key) {
                Some((code, text)) => {
                    let mine = self.tally(code, text).cloned();
                    self.merge_row(code, text, mine, &their_tally, None, None, merged);
                }
                None => self.graveyard.entry(key).or_default().join(&their_tally),
            }
        }
    }

    /// 合併一列：這台的各台次數 `mine`（墓碑表裡的已經取出來了）與學到的值，加上別台的。結果照 [`buried`] 放。
    #[allow(clippy::too_many_arguments)] // 一列的兩邊各三部分，只在合併用。
    fn merge_row(
        &mut self,
        code: &str,
        text: &str,
        mine: Option<Tally>,
        theirs: &Tally,
        their_count: Option<Count>,
        their_word: Option<UserWord>,
        merged: &mut Merged,
    ) {
        let my_count = self.count(code, text).copied();
        let my_word = self.word(code, text).cloned();
        let mut tally = mine.clone().unwrap_or_default();
        tally.join(theirs);
        let count = pick(my_count, their_count, &tally);
        let word = pick(my_word.clone(), their_word, &tally);
        let same = mine.as_ref() == Some(&tally) && my_count == count && my_word == word;
        let has_value = count.is_some() || word.is_some();
        merged.record(Change::of(
            my_count.is_some() || my_word.is_some(),
            has_value,
            same,
        ));
        put_in(&mut self.entries, code, text, count);
        put_in(&mut self.words, code, text, word);
        if buried(&tally, has_value) {
            remove_in(&mut self.tallies, code, text);
            self.graveyard
                .insert(SchemeModel::digest(code, text), tally);
        } else {
            put_in(&mut self.tallies, code, text, Some(tally));
        }
    }
}

impl Bigrams {
    /// 合併別台的個人 bigram（[`UserModel::merge`]）。
    fn merge(&mut self, theirs: Bigrams, merged: &mut Merged) {
        let (tick, mine_by, theirs_by) = align(self.tick, theirs.tick);
        if mine_by > 0 {
            for decayed in self.entries.values_mut().flat_map(BTreeMap::values_mut) {
                *decayed = decayed.shifted(mine_by);
            }
        }
        self.tick = tick;
        let Bigrams {
            mut entries,
            tallies,
            graveyard,
            ..
        } = theirs;
        for (previous, next) in tallies {
            for (text, their_tally) in next {
                let theirs =
                    remove_in(&mut entries, &previous, &text).map(|d| d.shifted(theirs_by));
                let mine = self
                    .tally(&previous, &text)
                    .cloned()
                    .or_else(|| exhume(&mut self.graveyard, || Bigrams::digest(&previous, &text)));
                self.merge_pair(&previous, &text, mine, &their_tally, theirs, merged);
            }
        }
        if graveyard.is_empty() {
            return;
        }
        let named: BTreeMap<String, (String, String)> = self
            .tallies
            .iter()
            .flat_map(|(previous, next)| next.keys().map(move |text| (previous, text)))
            .map(|(p, t)| (Bigrams::digest(p, t), (p.clone(), t.clone())))
            .collect();
        for (key, their_tally) in graveyard {
            match named.get(&key) {
                Some((previous, text)) => {
                    let mine = self.tally(previous, text).cloned();
                    self.merge_pair(previous, text, mine, &their_tally, None, merged);
                }
                None => self.graveyard.entry(key).or_default().join(&their_tally),
            }
        }
    }

    /// 合併一組，同 [`SchemeModel::merge_row`]。
    fn merge_pair(
        &mut self,
        previous: &str,
        text: &str,
        mine: Option<Tally>,
        theirs: &Tally,
        their_decayed: Option<Decayed>,
        merged: &mut Merged,
    ) {
        let my_decayed = self
            .entries
            .get(previous)
            .and_then(|n| n.get(text))
            .copied();
        let mut tally = mine.clone().unwrap_or_default();
        tally.join(theirs);
        let decayed = pick(my_decayed, their_decayed, &tally);
        let same = mine.as_ref() == Some(&tally) && my_decayed == decayed;
        merged.record(Change::of(my_decayed.is_some(), decayed.is_some(), same));
        put_in(&mut self.entries, previous, text, decayed);
        if buried(&tally, decayed.is_some()) {
            remove_in(&mut self.tallies, previous, text);
            self.graveyard
                .insert(Bigrams::digest(previous, text), tally);
        } else {
            put_in(&mut self.tallies, previous, text, Some(tally));
        }
    }

    /// 只留 `keep` 為真的各台次數（回收墓碑）；丟掉的連衰減值一起丟。
    fn retain_tallies(&mut self, mut keep: impl FnMut(&mut Tally) -> bool) {
        let mut gone = Vec::new();
        for (previous, next) in &mut self.tallies {
            for (text, tally) in next.iter_mut() {
                if !keep(tally) {
                    gone.push((previous.clone(), text.clone()));
                }
            }
        }
        for (previous, text) in gone {
            remove_in(&mut self.tallies, &previous, &text);
            remove_in(&mut self.entries, &previous, &text);
        }
    }
}

impl EnglishUserWords {
    /// 合併別台的英文使用者詞（[`UserModel::merge`]）。
    fn merge(&mut self, theirs: EnglishUserWords, merged: &mut Merged) {
        let (tick, mine_by, theirs_by) = align(self.tick, theirs.tick);
        if mine_by > 0 {
            for word in self.words.values_mut() {
                word.decayed = word.decayed.shifted(mine_by);
            }
        }
        self.tick = tick;
        let EnglishUserWords {
            mut words,
            tallies,
            graveyard,
            ..
        } = theirs;
        for (key, their_tally) in tallies {
            let theirs = words.remove(&key).map(|mut w| {
                w.decayed = w.decayed.shifted(theirs_by);
                w
            });
            let mine = self
                .tallies
                .get(&key)
                .cloned()
                .or_else(|| exhume(&mut self.graveyard, || EnglishUserWords::digest(&key)));
            self.merge_word(&key, mine, &their_tally, theirs, merged);
        }
        if graveyard.is_empty() {
            return;
        }
        let named: BTreeMap<String, String> = self
            .tallies
            .keys()
            .map(|key| (EnglishUserWords::digest(key), key.clone()))
            .collect();
        for (digest, their_tally) in graveyard {
            match named.get(&digest) {
                Some(key) => {
                    let mine = self.tallies.get(key).cloned();
                    self.merge_word(key, mine, &their_tally, None, merged);
                }
                None => self.graveyard.entry(digest).or_default().join(&their_tally),
            }
        }
    }

    /// 合併一個詞，同 [`SchemeModel::merge_row`]。
    fn merge_word(
        &mut self,
        key: &str,
        mine: Option<Tally>,
        theirs: &Tally,
        their_word: Option<EnglishUserWord>,
        merged: &mut Merged,
    ) {
        let my_word = self.words.get(key).cloned();
        let mut tally = mine.clone().unwrap_or_default();
        tally.join(theirs);
        let word = pick(my_word.clone(), their_word, &tally);
        let same = mine.as_ref() == Some(&tally) && my_word == word;
        merged.record(Change::of(my_word.is_some(), word.is_some(), same));
        let has_value = word.is_some();
        match word {
            Some(word) => self.words.insert(key.to_string(), word),
            None => self.words.remove(key),
        };
        if buried(&tally, has_value) {
            self.tallies.remove(key);
            self.graveyard.insert(EnglishUserWords::digest(key), tally);
        } else {
            self.tallies.insert(key.to_string(), tally);
        }
    }
}
