//! 學習資料的匯出格式 `# qim-userdict v3`（plan §2.4〈匯出格式〉）：對外的檔案格式，改了要升版本
//! （brief §5.3）；v1、v2 的檔一直要收。殼存檔、換機、備份與跨裝置同步都用它（ADR 0007）。

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::sync::{self, Cell, Tombstone};
use super::{
    Bigrams, Cleared, Count, Decayed, PHRASE_CODE, SOURCE_NAMES, SchemeModel, Tally, UserModel,
    UserWord, WordSource,
};
use crate::Date;
use crate::english_user::{self, ENGLISH_COMPLETION_AFTER, EnglishUserWord, EnglishUserWords};

/// 匯出檔的第一行（plan §2.4）；格式改了要升版本（brief §5.3）。v3 每列開頭是種類、帶各台的次數、墓碑與
/// 最後使用的日期，檔頭有裝置 id；M4 收尾前還沒定稿，M4 的票可以往裡加種類與欄位（M4 spec〈共同規則〉）。
const HEADER: &str = "# qim-userdict v3";
/// M3 定稿的第一行：沒有種類欄，列的種類由空的欄分辨；詞欄有跳脫、可以有常用語，沒有日期。
const HEADER_V2: &str = "# qim-userdict v2";
/// M1 定稿的第一行：詞欄照字面讀、沒有常用語與個人 bigram。v1、v2 匯入一直要收（M4 spec〈共同規則〉）。
const HEADER_V1: &str = "# qim-userdict v1";

/// 匯入的檔是哪一版，由第一行決定。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Version {
    V1,
    V2,
    V3,
}
/// 方案 tick 的 metadata 行開頭，後接方案 id、Tab、tick；照 librime userdb 文字檔的
/// `#@/tick`（`docs/research/userdict-text-formats.md`）。
const TICK_KEY: &str = "#@/tick/";
/// 個人 bigram 的 tick 的 metadata 行開頭，後接 Tab、tick；個人 bigram 各方案共用，所以不帶方案。
const SHARED_TICK_KEY: &str = "#@/tick\t";
/// 英文使用者詞自己的 tick 的 metadata 行開頭（v3 起，M4 票 14），後接 Tab、tick。
const ENGLISH_TICK_KEY: &str = "#@/english_tick\t";
/// 匯出這個檔的裝置 id 的 metadata 行開頭（v3 起），後接 Tab、id；照 librime userdb 文字檔的 `#@/user_id`。
const DEVICE_KEY: &str = "#@/device\t";
/// 清空記號的 metadata 行開頭（v3 起，M4 票 17，使用者 2026-11-13 決定），後接 Tab 分隔的代數、清空的日期與
/// 裝置 id（不知道的留空）；清空過才寫。
const CLEARED_KEY: &str = "#@/cleared\t";
/// 裝置表的 metadata 行開頭（v3 起，M4 票 17），後接編號、Tab、id：各台次數欄用編號代替 id，檔才不會每列
/// 重複一串 UUID。id 空的是不知道哪台（v1、v2 匯入的、殼沒給裝置 id 時學的）。
const DEVICE_TABLE_KEY: &str = "#@/device/";
/// v3 的種類：（方案, 輸入碼, 詞）的學習計數與使用者詞。
const ENTRY: &str = "entry";
/// v3 的種類：個人 bigram。
const BIGRAM: &str = "bigram";
/// v3 的種類：英文使用者詞（M4 票 14）。
const ENGLISH: &str = "english";
/// 欄名註解行；匯入時和其他 `#` 開頭的行一樣略過。
const COLUMNS: &str = "# kind\tscheme\tword\tcode\tcounts\tdeleted\tdeleted_on\tversion\tdecayed\tlast_tick\tlast_commit_tick\tlast_used\tsource\ttrigger\tword_version\tword_last_used";
/// 個人 bigram 的欄名註解行，有個人 bigram 時寫在它們前面。
const BIGRAM_COLUMNS: &str =
    "# kind\tprevious\tword\tcounts\tdeleted\tdeleted_on\tversion\tdecayed\tlast_tick\tlast_used";
/// v3 的種類：刪掉的條目（墓碑表，M4 票 17）。只有鍵的雜湊，不存原文。
const DELETED: &str = "deleted";
/// 刪掉的條目的欄名註解行：`of` 是條目的種類（`entry`、`bigram`、`english`），`scheme` 只有 `entry` 有，
/// `key` 是鍵的雜湊（[`sync::digest`]）。
const DELETED_COLUMNS: &str = "# kind\tof\tscheme\tkey\tcounts\tdeleted\tdeleted_on";
/// 英文使用者詞的欄名註解行，有英文使用者詞時寫在它們前面。
const ENGLISH_COLUMNS: &str =
    "# kind\tword\tcounts\tdeleted\tdeleted_on\tversion\tdecayed\tlast_tick\tlast_used";
/// 日期欄讀不出來時的原因。
const BAD_DATE: &str = "日期要是 YYYY-MM-DD 的真實日期，不知道時留空";

/// v1、v2 的列分不出種類時的分法，與 v3 的種類欄對應；欄位是去掉種類欄（或空的欄）之後的。
enum Kind<'a> {
    Entry(&'a str),
    Bigram(&'a str),
    /// v3 的英文使用者詞。
    English(&'a str),
    /// v3 刪掉的條目（墓碑表）。
    Deleted(&'a str),
    /// v2 方案欄、前一個字詞欄都空的列（記住的字串）：匯入成已過門檻的英文使用者詞。
    V2String(&'a str),
}

/// 各台次數欄（`counts`）裡一格的寫法，依種類：
/// - `entry`：`編號:用過:整串上屏累計:不算的:最後整串上屏的日期`
/// - `bigram`：`編號:用過`
/// - `english`：`編號:打過的累計:不算的:最後打的日期`（英文每打一次用過與打過各加一，兩者一樣）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    Entry,
    Bigram,
    English,
}

impl UserModel {
    /// 匯出成 `# qim-userdict v3` 的 TSV（plan §2.4），給殼存檔、換機、備份或同步到別台。每列開頭是種類：
    /// 每個（方案, 輸入碼, 詞）一列 `entry`，依方案、輸入碼、詞的順序；接著個人 bigram 一列一組
    /// `bigram`，依前一個字詞、這一段的順序；再來英文使用者詞 `english`，依小寫的順序。刪掉的條目只剩各台的
    /// 次數與墓碑（ADR 0007）。殼沒給過日期時學到的日期留空（不寫假日期）。詞欄與前一個字詞經 [`escape`]。
    pub fn export(&self) -> String {
        let mut out = format!("{HEADER}\n");
        if let Some(device) = &self.device {
            out.push_str(&format!("{DEVICE_KEY}{device}\n"));
        }
        if let Some(cleared) = &self.cleared {
            out.push_str(&format!(
                "{CLEARED_KEY}{}\t{}\t{}\n",
                cleared.generation,
                OptionField(cleared.on),
                cleared.device
            ));
        }
        let devices = self.devices();
        for (n, id) in devices.values().zip(devices.keys()) {
            out.push_str(&format!("{DEVICE_TABLE_KEY}{n}\t{id}\n"));
        }
        if self.bigrams.tick > 0 {
            out.push_str(&format!("{SHARED_TICK_KEY}{}\n", self.bigrams.tick));
        }
        if self.english.tick > 0 {
            out.push_str(&format!("{ENGLISH_TICK_KEY}{}\n", self.english.tick));
        }
        for (scheme, model) in &self.schemes {
            out.push_str(&format!("{TICK_KEY}{scheme}\t{}\n", model.tick));
        }
        out.push_str(COLUMNS);
        out.push('\n');
        for (scheme, model) in &self.schemes {
            for (code, texts) in &model.tallies {
                for (text, tally) in texts {
                    let count = model.count(code, text);
                    let word = model.word(code, text);
                    let count = match count {
                        Some(c) => format!(
                            "{}\t{}\t{}\t{}\t{}",
                            c.version,
                            c.decayed,
                            c.last_tick,
                            OptionField(c.last_commit_tick),
                            OptionField(c.last_used),
                        ),
                        None => "\t\t\t\t".to_string(),
                    };
                    let word = match word {
                        Some(w) => {
                            let (_, source) = SOURCE_NAMES
                                .into_iter()
                                .find(|&(s, _)| s == Some(w.source))
                                .expect("每種來源都有名字");
                            format!(
                                "{source}\t{}\t{}\t{}",
                                w.trigger.as_deref().unwrap_or(""),
                                w.version,
                                OptionField(w.last_used),
                            )
                        }
                        None => "\t\t\t".to_string(),
                    };
                    out.push_str(&format!(
                        "{ENTRY}\t{scheme}\t{}\t{code}\t{}\t{count}\t{word}\n",
                        escape(text),
                        tally_fields(tally, Shape::Entry, &devices),
                    ));
                }
            }
            debug_assert!(
                model.entries.iter().all(|(code, texts)| texts
                    .keys()
                    .all(|text| model.tally(code, text).is_some()))
            );
            debug_assert!(
                model.words.iter().all(|(code, texts)| texts
                    .keys()
                    .all(|text| model.tally(code, text).is_some()))
            );
        }
        if !self.bigrams.tallies.is_empty() {
            out.push_str(BIGRAM_COLUMNS);
            out.push('\n');
        }
        for (previous, next) in &self.bigrams.tallies {
            for (text, tally) in next {
                let decayed = self.bigrams.entries.get(previous).and_then(|n| n.get(text));
                out.push_str(&format!(
                    "{BIGRAM}\t{}\t{}\t{}\t{}\n",
                    escape(previous),
                    escape(text),
                    tally_fields(tally, Shape::Bigram, &devices),
                    decayed_fields(decayed),
                ));
            }
        }
        if !self.english.tallies.is_empty() {
            out.push_str(ENGLISH_COLUMNS);
            out.push('\n');
        }
        for (key, tally) in &self.english.tallies {
            let word = self.english.words.get(key);
            out.push_str(&format!(
                "{ENGLISH}\t{}\t{}\t{}\n",
                word.map_or(key.as_str(), |w| w.text.as_str()),
                tally_fields(tally, Shape::English, &devices),
                decayed_fields(word.map(|w| &w.decayed)),
            ));
        }
        let mut deleted: Vec<(&str, &str, &str, &Tally)> = Vec::new();
        for (scheme, model) in &self.schemes {
            deleted.extend(
                model
                    .graveyard
                    .iter()
                    .map(|(k, t)| (ENTRY, scheme.as_str(), k.as_str(), t)),
            );
        }
        deleted.extend(
            self.bigrams
                .graveyard
                .iter()
                .map(|(k, t)| (BIGRAM, "", k.as_str(), t)),
        );
        deleted.extend(
            self.english
                .graveyard
                .iter()
                .map(|(k, t)| (ENGLISH, "", k.as_str(), t)),
        );
        if !deleted.is_empty() {
            out.push_str(DELETED_COLUMNS);
            out.push('\n');
        }
        for (of, scheme, key, tally) in deleted {
            let shape = Shape::of(of).expect("三種之一");
            out.push_str(&format!(
                "{DELETED}\t{of}\t{scheme}\t{key}\t{}\n",
                tally_fields(tally, shape, &devices)
            ));
        }
        out
    }

    /// 各台次數裡出現的裝置 id → 匯出時的編號，依 id 排（空的在前）。
    fn devices(&self) -> BTreeMap<&str, usize> {
        let tallies = self
            .schemes
            .values()
            .flat_map(|m| {
                m.tallies
                    .values()
                    .flat_map(BTreeMap::values)
                    .chain(m.graveyard.values())
            })
            .chain(self.bigrams.tallies.values().flat_map(BTreeMap::values))
            .chain(self.bigrams.graveyard.values())
            .chain(self.english.tallies.values())
            .chain(self.english.graveyard.values());
        let ids: BTreeSet<&str> = tallies
            .flat_map(|t| t.cells.keys().map(String::as_str))
            .collect();
        ids.into_iter().zip(0..).collect()
    }

    /// 讀 [`UserModel::export`] 的檔（v3，也收 v1、v2）成一份新的個人模型，今天是 `today`（殼還沒給
    /// 時是 `None`）。v1、v2 沒有日期與各台的次數：每列的最後使用日期取 `today`（匯入那天），次數記在不知道
    /// 哪台的那格（空的裝置 id，同一份舊檔在兩台匯入再同步也不會算兩次）。v3 的日期欄空的是不知道。裝置 id
    /// 是檔裡的（沒有時是 `None`）。殼拿它取代自己的學習資料用 [`UserModel::replace_learning`]（讀回自己存的
    /// 檔），合併別台的或備份用 [`UserModel::merge`]。
    ///
    /// 第一行不是三者之一、或已知種類的列有一列不合格式時整份不收，錯誤帶行號；v3 不認得的種類
    /// （之後的版本加的）略過，回警告帶行號，其他列照收。
    pub fn import(text: &str, today: Option<Date>) -> Result<Imported, ImportError> {
        let mut lines = text
            .lines()
            .map(|l| l.strip_suffix('\r').unwrap_or(l))
            .zip(1..);
        let version = match lines.next().map(|(l, _)| l) {
            Some(HEADER) => Version::V3,
            Some(HEADER_V2) => Version::V2,
            Some(HEADER_V1) => Version::V1,
            _ => {
                return Err(ImportError::at(
                    1,
                    format!("第一行要是「{HEADER}」「{HEADER_V2}」或「{HEADER_V1}」"),
                ));
            }
        };
        let mut user = UserModel {
            today,
            ..UserModel::default()
        };
        let mut warnings = Vec::new();
        let mut devices: Vec<String> = Vec::new();
        let mut ticks: BTreeMap<&str, u64> = BTreeMap::new();
        let mut shared_tick: Option<u64> = None;
        let mut english_tick: Option<u64> = None;
        for (line, number) in lines {
            let error = |reason: &str| ImportError::at(number, reason.to_string());
            // v1、v2 沒有裝置 id 與裝置表，這兩種行照其他 `#` 開頭的行是註解。
            if let Some(id) = line.strip_prefix(DEVICE_KEY)
                && version == Version::V3
            {
                if user.device.is_some() {
                    return Err(error("裝置 id 重複"));
                }
                if !is_device_id(id) {
                    return Err(error("裝置 id 只能用英數、-、_，1–64 字"));
                }
                user.device = Some(id.to_string());
                continue;
            }
            if let Some(fields) = line.strip_prefix(CLEARED_KEY)
                && version == Version::V3
            {
                if user.cleared.is_some() {
                    return Err(error("清空記號重複"));
                }
                user.cleared = Some(parse_cleared(fields).map_err(error)?);
                continue;
            }
            if let Some(entry) = line.strip_prefix(DEVICE_TABLE_KEY)
                && version == Version::V3
            {
                let (n, id) = entry
                    .split_once('\t')
                    .ok_or_else(|| error("裝置表的行少了 Tab"))?;
                if n.parse::<usize>().ok() != Some(devices.len()) {
                    return Err(error("裝置表的編號要從 0 起依序"));
                }
                if !(id.is_empty() || is_device_id(id)) {
                    return Err(error(
                        "裝置表的 id 只能用英數、-、_，1–64 字，或空的（不知道哪台）",
                    ));
                }
                if devices.iter().any(|d| d == id) {
                    return Err(error("裝置表的 id 重複"));
                }
                devices.push(id.to_string());
                continue;
            }
            // v1 沒有共用的 tick，這行照其他 `#` 開頭的行是註解。
            if let Some(tick) = line.strip_prefix(SHARED_TICK_KEY)
                && version >= Version::V2
            {
                let tick = tick.parse().map_err(|_| error("tick 不是整數"))?;
                // 讀到這裡時，個人 bigram 的 tick 是前面各列最大的最後 tick。
                if user.bigrams.tick > tick {
                    return Err(error("tick 比前面個人 bigram 列的最後 tick 小"));
                }
                shared_tick = Some(tick);
                continue;
            }
            if let Some(tick) = line.strip_prefix(ENGLISH_TICK_KEY)
                && version == Version::V3
            {
                let tick = tick.parse().map_err(|_| error("tick 不是整數"))?;
                // 讀到這裡時，英文使用者詞的 tick 是前面各列最大的最後 tick。
                if user.english.tick > tick {
                    return Err(error("tick 比前面英文使用者詞列的最後 tick 小"));
                }
                english_tick = Some(tick);
                continue;
            }
            if let Some(tick) = line.strip_prefix(TICK_KEY) {
                let (scheme, tick) = tick
                    .split_once('\t')
                    .ok_or_else(|| error("tick 行少了 Tab"))?;
                let tick = tick.parse().map_err(|_| error("tick 不是整數"))?;
                // 讀到這裡時，方案的 tick 是前面各列最大的最後 tick。
                if user.schemes.get(scheme).is_some_and(|m| m.tick > tick) {
                    return Err(error("tick 比前面這個方案的列的最後 tick 小"));
                }
                ticks.insert(scheme, tick);
                continue;
            }
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let kind = if version == Version::V3 {
                let (kind, fields) = line.split_once('\t').unwrap_or((line, ""));
                match kind {
                    ENTRY => Kind::Entry(fields),
                    BIGRAM => Kind::Bigram(fields),
                    ENGLISH => Kind::English(fields),
                    DELETED => Kind::Deleted(fields),
                    "" => return Err(error("種類欄不能是空的")),
                    other => {
                        warnings.push(ImportWarning {
                            line: number,
                            reason: format!("不認得的種類「{other}」，略過這列"),
                        });
                        continue;
                    }
                }
            } else if let Some(shared) = line.strip_prefix('\t') {
                // 方案欄是空的：個人 bigram 或記住的字串（v2 起）；前一個字詞欄也是空的是記住的字串。
                if version == Version::V1 {
                    return Err(error("方案不能是空的（v1 沒有個人 bigram 與記住的字串）"));
                }
                match shared.strip_prefix('\t') {
                    Some(string) => Kind::V2String(string),
                    None => Kind::Bigram(shared),
                }
            } else {
                Kind::Entry(line)
            };
            match kind {
                Kind::Deleted(fields) => {
                    let (of, scheme, key, tally) =
                        parse_deleted(fields, &devices).map_err(error)?;
                    let graveyard = match of {
                        Shape::Entry => &mut user.for_scheme_mut(scheme).graveyard,
                        Shape::Bigram => &mut user.bigrams.graveyard,
                        Shape::English => &mut user.english.graveyard,
                    };
                    if graveyard.insert(key.to_string(), tally).is_some() {
                        return Err(error("刪掉的條目重複"));
                    }
                }
                Kind::English(fields) => {
                    let (key, tally, word) =
                        parse_english(fields, &devices, draft(&user)).map_err(error)?;
                    insert_english(&mut user, key, tally, word, english_tick).map_err(error)?;
                }
                Kind::V2String(fields) => {
                    let (key, tally, word) = parse_string(fields, today).map_err(error)?;
                    insert_english(&mut user, key, tally, Some(word), shared_tick)
                        .map_err(error)?;
                }
                Kind::Bigram(fields) => {
                    let (previous, text, tally, decayed) =
                        parse_bigram(fields, version, today, &devices, draft(&user))
                            .map_err(error)?;
                    let last_tick = decayed.map_or(0, |d| d.last_tick);
                    if shared_tick.is_some_and(|t| last_tick > t) {
                        return Err(error("最後 tick 比個人 bigram 的 tick 大"));
                    }
                    let (previous, text) = (previous.into_owned(), text.into_owned());
                    if !tally.alive() {
                        // 刪掉的列（匯出時寫成雜湊，手寫的檔才會這樣）：埋進墓碑表。
                        user.bigrams
                            .graveyard
                            .insert(Bigrams::digest(&previous, &text), tally);
                        continue;
                    }
                    let next = user.bigrams.tallies.entry(previous.clone()).or_default();
                    if next.insert(text.clone(), tally).is_some() {
                        return Err(error("同前一個字詞、同一段的個人 bigram 重複"));
                    }
                    if let Some(decayed) = decayed {
                        user.bigrams
                            .entries
                            .entry(previous)
                            .or_default()
                            .insert(text, decayed);
                    }
                    user.bigrams.tick = user.bigrams.tick.max(last_tick);
                }
                Kind::Entry(fields) => {
                    let row = Row::parse(fields, version, today, &devices, draft(&user))
                        .map_err(error)?;
                    let last_tick = row
                        .count
                        .map_or(0, |c| c.last_tick.max(c.last_commit_tick.unwrap_or(0)));
                    if ticks.get(row.scheme).is_some_and(|&t| last_tick > t) {
                        return Err(error("最後 tick 比方案的 tick 大"));
                    }
                    let model = user.for_scheme_mut(row.scheme);
                    if model.tally(row.code, &row.text).is_some() {
                        return Err(error("同方案、同輸入碼、同詞的列重複"));
                    }
                    if !row.tally.alive() {
                        // 刪掉的列（匯出時寫成雜湊，手寫的檔才會這樣）：埋進墓碑表。
                        let key = SchemeModel::digest(row.code, &row.text);
                        model.graveyard.insert(key, row.tally);
                        continue;
                    }
                    model.insert_row(row.code, &row.text, row.tally, row.count, row.word);
                    // 沒有 tick 行時，方案的 tick 取最大的最後 tick（兩種最後 tick 都算，否則算期限時相減會小於 0）。
                    model.tick = model.tick.max(last_tick);
                }
            }
        }
        // 只剩 tick 行、沒有資料列的方案（資料列都刪光了）也要還原。
        for (scheme, &tick) in &ticks {
            user.for_scheme_mut(scheme).tick = tick;
        }
        // 沒有 tick 行時，個人 bigram 的 tick 取最大的最後 tick（同方案）。
        if let Some(tick) = shared_tick {
            user.bigrams.tick = tick;
        }
        // v2 的記住的字串用共用的 tick；換成英文使用者詞自己的 tick 時接著用它，衰減的先後不變。
        if let Some(tick) = english_tick
            .or(shared_tick.filter(|_| version == Version::V2 && !user.english.words.is_empty()))
        {
            user.english.tick = tick;
        }
        Ok(Imported { user, warnings })
    }
}

impl SchemeModel {
    /// 放進匯入的一列（不檢查，[`Row::parse`] 檢查過）。
    fn insert_row(
        &mut self,
        code: &str,
        text: &str,
        tally: Tally,
        count: Option<Count>,
        word: Option<UserWord>,
    ) {
        *self.tally_mut(code, text) = tally;
        if let Some(count) = count {
            self.entries
                .entry(code.to_string())
                .or_default()
                .insert(text.to_string(), count);
        }
        if let Some(word) = word {
            self.words_at(code).insert(text.to_string(), word);
        }
    }
}

/// [`UserModel::import`] 讀出來的：個人模型與略過的列。
#[derive(Debug, Clone, PartialEq)]
pub struct Imported {
    pub user: UserModel,
    /// 略過的列（v3 不認得的種類），依行號。
    pub warnings: Vec<ImportWarning>,
}

/// [`UserModel::import`] 略過的一列：第幾行（從 1 起）、為什麼。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportWarning {
    pub line: usize,
    pub reason: String,
}

impl fmt::Display for ImportWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "第 {} 行：{}", self.line, self.reason)
    }
}

/// 7ad01da（M4 票 16 之後的 main）的 v3 草稿的列沒有各台的次數，次數記在哪台：檔頭的裝置 id（網頁存檔時
/// 寫的就是這台），沒有時是不知道哪台的那格（協調者 2026-11-12 決定照收這版草稿，使用者在網頁累積的學習資料
/// 才讀得回來；更早的草稿不收）。
fn draft(user: &UserModel) -> &str {
    user.device.as_deref().unwrap_or("")
}

/// 裝置 id 只能用英數、`-`、`_`，1–64 字：放得進 TSV 的 metadata 行，UUID 也收得下。
pub(super) fn is_device_id(id: &str) -> bool {
    (1..=64).contains(&id.len())
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// 日期欄：空的是不知道（殼沒給過日期時學到的）。
fn parse_date(field: &str) -> Result<Option<Date>, &'static str> {
    if field.is_empty() {
        return Ok(None);
    }
    field.parse().map(Some).map_err(|_| BAD_DATE)
}

/// 匯出時可有可無的欄（日期、tick）：沒有時留空。
struct OptionField<T>(Option<T>);

impl<T: fmt::Display> fmt::Display for OptionField<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(value) => value.fmt(f),
            None => Ok(()),
        }
    }
}

/// 各台次數、墓碑三欄：`counts`、`deleted`（刪除時的合計次數，沒刪過留空）、`deleted_on`。
fn tally_fields(tally: &Tally, shape: Shape, devices: &BTreeMap<&str, usize>) -> String {
    let cells: Vec<String> = tally
        .cells
        .iter()
        .map(|(device, cell)| {
            let n = devices[device.as_str()];
            let date = OptionField(cell.last_commit);
            match shape {
                Shape::Entry => format!(
                    "{n}:{}:{}:{}:{date}",
                    cell.uses, cell.commits, cell.uncounted
                ),
                Shape::Bigram => format!("{n}:{}", cell.uses),
                Shape::English => format!("{n}:{}:{}:{date}", cell.commits, cell.uncounted),
            }
        })
        .collect();
    let (deleted, on) = match tally.deleted {
        Some(d) => (Some(d.uses), d.on),
        None => (None, None),
    };
    format!(
        "{}\t{}\t{}",
        cells.join(","),
        OptionField(deleted),
        OptionField(on)
    )
}

/// 衰減值四欄：`version`、`decayed`、`last_tick`、`last_used`；刪掉的（沒有值）都留空。
fn decayed_fields(decayed: Option<&Decayed>) -> String {
    match decayed {
        Some(d) => format!(
            "{}\t{}\t{}\t{}",
            d.version,
            d.value,
            d.last_tick,
            OptionField(d.last_used)
        ),
        None => "\t\t\t".to_string(),
    }
}

impl Shape {
    /// 種類欄的名字對到的形狀。
    fn of(kind: &str) -> Option<Self> {
        match kind {
            ENTRY => Some(Self::Entry),
            BIGRAM => Some(Self::Bigram),
            ENGLISH => Some(Self::English),
            _ => None,
        }
    }
}

/// 清空記號行去掉開頭之後：代數（正整數）、日期、裝置 id（空的或合格的 id）。
fn parse_cleared(fields: &str) -> Result<Cleared, &'static str> {
    let &[generation, on, device] = &fields.split('\t').collect::<Vec<_>>()[..] else {
        return Err("清空記號要有代數、日期、裝置 id 三欄，以 Tab 分隔");
    };
    let generation = integer(generation)?;
    if generation == 0 {
        return Err("清空記號的代數要是正整數");
    }
    if !(device.is_empty() || is_device_id(device)) {
        return Err("清空記號的裝置 id 只能用英數、-、_，1–64 字，或空的");
    }
    Ok(Cleared {
        generation,
        on: parse_date(on)?,
        device: device.to_string(),
    })
}

/// 刪掉的條目一列去掉種類欄之後：條目的種類、方案（只有 `entry` 有）、鍵的雜湊、各台次數與墓碑三欄（墓碑一定有）。
fn parse_deleted<'a>(
    line: &'a str,
    devices: &[String],
) -> Result<(Shape, &'a str, &'a str, Tally), &'static str> {
    let &[of, scheme, key, counts, deleted, deleted_on] = &line.split('\t').collect::<Vec<_>>()[..]
    else {
        return Err("deleted 要有 7 欄（含種類），以 Tab 分隔");
    };
    let shape = Shape::of(of).ok_or("刪掉的條目的種類（of）要是 entry、bigram 或 english")?;
    if (shape == Shape::Entry) == scheme.is_empty() {
        return Err("刪掉的條目只有 entry 有方案");
    }
    let hex = |b: u8| b.is_ascii_digit() || (b'a'..=b'f').contains(&b);
    if key.len() != 2 * sync::DIGEST_BYTES || !key.bytes().all(hex) {
        return Err("刪掉的條目的鍵要是 32 位小寫十六進位的雜湊");
    }
    let tally = parse_tally(counts, deleted, deleted_on, shape, devices)?;
    if tally.deleted.is_none() {
        return Err("刪掉的條目要有墓碑（deleted）");
    }
    Ok((shape, scheme, key, tally))
}

/// 非負整數欄。
fn integer(field: &str) -> Result<u64, &'static str> {
    field.parse().map_err(|_| "次數、版本與 tick 要是非負整數")
}

/// 讀各台次數、墓碑三欄（[`tally_fields`]）。`devices` 是檔頭的裝置表。
fn parse_tally(
    counts: &str,
    deleted: &str,
    deleted_on: &str,
    shape: Shape,
    devices: &[String],
) -> Result<Tally, &'static str> {
    if counts.is_empty() {
        return Err("各台的次數（counts）不能是空的");
    }
    let mut cells = BTreeMap::new();
    for cell in counts.split(',') {
        let parts: Vec<&str> = cell.split(':').collect();
        let (n, cell) = match (shape, &parts[..]) {
            (Shape::Entry, &[n, uses, commits, uncounted, date]) => (
                n,
                Cell {
                    uses: integer(uses)?,
                    commits: integer(commits)?,
                    uncounted: integer(uncounted)?,
                    last_commit: parse_date(date)?,
                },
            ),
            (Shape::Bigram, &[n, uses]) => (
                n,
                Cell {
                    uses: integer(uses)?,
                    ..Cell::default()
                },
            ),
            (Shape::English, &[n, count, uncounted, date]) => {
                let count = integer(count)?;
                (
                    n,
                    Cell {
                        uses: count,
                        commits: count,
                        uncounted: integer(uncounted)?,
                        last_commit: parse_date(date)?,
                    },
                )
            }
            (Shape::Entry, _) => {
                return Err("entry 的各台次數每格要是「編號:用過:整串上屏:不算的:日期」");
            }
            (Shape::Bigram, _) => return Err("bigram 的各台次數每格要是「編號:用過」"),
            (Shape::English, _) => {
                return Err("english 的各台次數每格要是「編號:打過:不算的:日期」");
            }
        };
        if cell.uncounted > cell.commits {
            return Err("不算的次數不能比累計的多");
        }
        let device = n
            .parse::<usize>()
            .ok()
            .and_then(|n| devices.get(n))
            .ok_or("各台次數的編號不在檔頭的裝置表裡")?;
        if cells.insert(device.clone(), cell).is_some() {
            return Err("同一列裡同一台的次數重複");
        }
    }
    let on = parse_date(deleted_on)?;
    let deleted = match deleted {
        "" if on.is_some() => return Err("沒刪過（deleted 空的）時刪除的日期要留空"),
        "" => None,
        uses => Some(Tombstone {
            uses: integer(uses)?,
            on,
        }),
    };
    let tally = Tally { cells, deleted };
    if tally.deleted.is_some_and(|d| d.uses > tally.uses()) {
        return Err("刪除時的合計（deleted）不能比各台用過的合計多");
    }
    Ok(tally)
}

/// 學到的值的版本：要是寫入時的合計次數，所以不比各台用過的合計多，也要在墓碑之後。
fn check_version(version: u64, tally: &Tally) -> Result<u64, &'static str> {
    if version > tally.uses() {
        return Err("版本不能比各台用過的合計多");
    }
    if !tally.admits(version) {
        return Err("版本要比刪除時的合計（deleted）大：刪除前的值不留");
    }
    Ok(version)
}

/// 讀衰減值四欄（[`decayed_fields`]）：都空的是沒有值（刪掉的）。
fn parse_decayed(
    [version, value, last_tick, last_used]: [&str; 4],
    tally: &Tally,
) -> Result<Option<Decayed>, &'static str> {
    if [version, value, last_tick, last_used]
        .iter()
        .all(|f| f.is_empty())
    {
        return Ok(None);
    }
    let value: f64 = value.parse().map_err(|_| "衰減值要是數字")?;
    if !value.is_finite() || value <= 0.0 {
        return Err("衰減值要是正的有限數");
    }
    Ok(Some(Decayed {
        value,
        last_tick: integer(last_tick)?,
        last_used: parse_date(last_used)?,
        version: check_version(integer(version)?, tally)?,
    }))
}

/// 匯出檔的一列 `entry`。
struct Row<'a> {
    scheme: &'a str,
    /// 已還原跳脫。
    text: Cow<'a, str>,
    code: &'a str,
    tally: Tally,
    count: Option<Count>,
    word: Option<UserWord>,
}

impl<'a> Row<'a> {
    /// `line` 是去掉種類欄之後的（v1、v2 沒有種類欄）。`draft` 是 7ad01da 草稿的列的次數要記在哪台
    /// （[`Draft`]）。
    fn parse(
        line: &'a str,
        version: Version,
        today: Option<Date>,
        devices: &[String],
        draft: &str,
    ) -> Result<Self, &'static str> {
        let fields: Vec<&str> = line.split('\t').collect();
        if version != Version::V3 {
            let &[
                scheme,
                text,
                code,
                commits,
                last_commit_tick,
                decayed,
                last_tick,
                source,
                trigger,
            ] = &fields[..]
            else {
                return Err("要有 9 欄，以 Tab 分隔");
            };
            let old = [
                scheme,
                text,
                code,
                commits,
                last_commit_tick,
                decayed,
                last_tick,
                source,
                trigger,
            ];
            let committed = integer(commits)? > 0;
            return Self::parse_old(
                old,
                version == Version::V1,
                "",
                today,
                today.filter(|_| committed),
            );
        }
        // M4 票 12–16 的 v3 草稿（main 7ad01da）：12 欄，沒有各台的次數與墓碑。
        if let &[
            scheme,
            text,
            code,
            commits,
            last_commit_tick,
            decayed,
            last_tick,
            last_used,
            last_commit_date,
            source,
            trigger,
        ] = &fields[..]
        {
            let last_commit = parse_date(last_commit_date)?;
            if last_commit.is_some() && integer(commits)? == 0 {
                return Err("沒有整串上屏過（commits 是 0）時最後整串上屏的日期要留空");
            }
            let old = [
                scheme,
                text,
                code,
                commits,
                last_commit_tick,
                decayed,
                last_tick,
                source,
                trigger,
            ];
            return Self::parse_old(old, false, draft, parse_date(last_used)?, last_commit);
        }
        let &[
            scheme,
            text,
            code,
            counts,
            deleted,
            deleted_on,
            count_version,
            decayed,
            last_tick,
            last_commit_tick,
            last_used,
            source,
            trigger,
            word_version,
            word_last_used,
        ] = &fields[..]
        else {
            return Err("entry 要有 16 欄（含種類），以 Tab 分隔");
        };
        if scheme.is_empty() || text.is_empty() {
            return Err("方案、詞不能是空的");
        }
        let text = unescape(text)?;
        let tally = parse_tally(counts, deleted, deleted_on, Shape::Entry, devices)?;
        let count_fields = [
            count_version,
            decayed,
            last_tick,
            last_commit_tick,
            last_used,
        ];
        let count = if count_fields.iter().all(|f| f.is_empty()) {
            None
        } else {
            let decayed: f64 = decayed.parse().map_err(|_| "衰減值要是數字")?;
            if !decayed.is_finite() || decayed <= 0.0 {
                return Err("學到的衰減值要是正的有限數");
            }
            Some(Count {
                decayed,
                last_tick: integer(last_tick)?,
                last_commit_tick: match last_commit_tick {
                    "" => None,
                    tick => Some(integer(tick)?),
                },
                last_used: parse_date(last_used)?,
                version: check_version(integer(count_version)?, &tally)?,
            })
        };
        let word = match source {
            "" if [trigger, word_version, word_last_used]
                .iter()
                .any(|f| !f.is_empty()) =>
            {
                return Err("沒有使用者詞（source 空的）時觸發詞與詞的欄要留空");
            }
            "" => None,
            source => {
                let (source, _) = SOURCE_NAMES
                    .into_iter()
                    .find(|&(_, name)| name == source)
                    .filter(|(s, _)| s.is_some())
                    .ok_or("來源要是 manual、auto、phrase 或空的")?;
                let source = source.expect("濾掉了 learned");
                let trigger = (!trigger.is_empty()).then(|| trigger.to_string());
                if source == WordSource::AutoPhrase && trigger.is_some() {
                    return Err("自動造詞沒有觸發詞");
                }
                Some(UserWord {
                    source,
                    trigger,
                    last_used: parse_date(word_last_used)?,
                    version: check_version(integer(word_version)?, &tally)?,
                })
            }
        };
        // 常用語的碼是空的；碼空的列是常用語，或刪掉的常用語（只剩次數與墓碑）。
        let phrase = word
            .as_ref()
            .is_some_and(|w| w.source == WordSource::Phrase);
        if phrase != (code == PHRASE_CODE) && !(code == PHRASE_CODE && word.is_none()) {
            return Err("常用語（phrase）的輸入碼是空的，其他列不能是空的");
        }
        if code == PHRASE_CODE && count.is_some() {
            return Err("常用語（phrase）沒有計數，觸發詞學到的另是一列");
        }
        Ok(Self {
            scheme,
            text,
            code,
            tally,
            count,
            word,
        })
    }

    /// 沒有各台次數的舊列（v1、v2 與 7ad01da 的 v3 草稿）的 9 欄：`方案  詞  輸入碼  整串上屏次數  最後整串上屏
    /// tick  衰減值  最後 tick  來源  觸發詞`。v1 的詞欄照字面讀，沒有常用語。次數記在 `device` 那格，用過的次數取
    /// 整串上屏次數（至少 1）；最後使用的日期是 `last_used`，最後整串上屏的日期是 `last_commit`。
    fn parse_old(
        [
            scheme,
            text,
            code,
            commits,
            last_commit_tick,
            decayed,
            last_tick,
            source,
            trigger,
        ]: [&'a str; 9],
        v1: bool,
        device: &str,
        last_used: Option<Date>,
        last_commit: Option<Date>,
    ) -> Result<Self, &'static str> {
        if scheme.is_empty() || text.is_empty() {
            return Err("方案、詞不能是空的");
        }
        let text = if v1 {
            Cow::Borrowed(text)
        } else {
            unescape(text)?
        };
        let decayed: f64 = decayed.parse().map_err(|_| "衰減值要是數字")?;
        if !decayed.is_finite() || decayed < 0.0 {
            return Err("衰減值要是非負的有限數");
        }
        let commits = integer(commits)?;
        let last_commit_tick = integer(last_commit_tick)?;
        let last_tick = integer(last_tick)?;
        let uses = commits.max(1);
        let tally = old_tally(device, uses, commits, last_commit);
        // 手動加了還沒用過的詞計數都寫 0，不建計數。
        let empty = commits == 0 && last_commit_tick == 0 && decayed == 0.0 && last_tick == 0;
        let count = (!empty).then_some(Count {
            decayed,
            last_tick,
            last_commit_tick: (commits > 0).then_some(last_commit_tick),
            last_used,
            version: uses,
        });
        let trigger = (!trigger.is_empty()).then(|| trigger.to_string());
        let (source, _) = SOURCE_NAMES
            .into_iter()
            .filter(|&(s, _)| !(v1 && s == Some(WordSource::Phrase)))
            .find(|&(_, name)| name == source)
            .ok_or(if v1 {
                "來源要是 manual、learned 或 auto"
            } else {
                "來源要是 manual、learned、auto 或 phrase"
            })?;
        let phrase = source == Some(WordSource::Phrase);
        if phrase != (code == PHRASE_CODE) {
            return Err("常用語（phrase）的輸入碼是空的，其他列不能是空的");
        }
        if phrase && count.is_some() {
            return Err("常用語（phrase）沒有計數，觸發詞學到的另是一列 learned");
        }
        let word = match source {
            None if trigger.is_some() => return Err("學到的（learned）不是使用者詞，沒有觸發詞"),
            None if decayed == 0.0 => return Err("學到的（learned）要有衰減值"),
            Some(WordSource::AutoPhrase) if trigger.is_some() => {
                return Err("自動造詞沒有觸發詞");
            }
            None => None,
            Some(source) => Some(UserWord {
                source,
                trigger,
                last_used,
                version: uses,
            }),
        };
        Ok(Self {
            scheme,
            text,
            code,
            tally,
            count,
            word,
        })
    }
}

/// 舊列（v1、v2 與 7ad01da 的 v3 草稿）的各台次數：只有一格，記在 `device`（v1、v2 不知道哪台，是空的 id；
/// 草稿是檔頭的裝置），沒有墓碑。
fn old_tally(device: &str, uses: u64, commits: u64, last_commit: Option<Date>) -> Tally {
    let cell = Cell {
        uses,
        commits,
        uncounted: 0,
        last_commit,
    };
    Tally {
        cells: BTreeMap::from([(device.to_string(), cell)]),
        deleted: None,
    }
}

/// 個人 bigram 的一列去掉種類欄（v2 是空的方案欄）之後。v3：前一個字詞、這一段、各台次數與墓碑三欄、
/// 衰減值四欄（刪掉的留空）；v2：前一個字詞、這一段、衰減值、最後 tick，日期取 `today`、次數記在不知道哪台的那格。
#[allow(clippy::type_complexity)] // 一列的四個部分，只在匯入用一次。
fn parse_bigram<'a>(
    line: &'a str,
    version: Version,
    today: Option<Date>,
    devices: &[String],
    draft: &str,
) -> Result<(Cow<'a, str>, Cow<'a, str>, Tally, Option<Decayed>), &'static str> {
    let fields: Vec<&str> = line.split('\t').collect();
    let positive = |value: &str| -> Result<f64, &'static str> {
        let value: f64 = value.parse().map_err(|_| "衰減值要是數字")?;
        if !value.is_finite() || value <= 0.0 {
            return Err("個人 bigram 的衰減值要是正的有限數");
        }
        Ok(value)
    };
    let (previous, text, tally, decayed) = match (version, &fields[..]) {
        // 7ad01da 的 v3 草稿：6 欄，沒有各台的次數。
        (Version::V3, &[previous, text, value, last_tick, last_used]) => {
            let decayed = Decayed {
                value: positive(value)?,
                last_tick: integer(last_tick)?,
                last_used: parse_date(last_used)?,
                version: 1,
            };
            (previous, text, old_tally(draft, 1, 0, None), Some(decayed))
        }
        (
            Version::V3,
            &[
                previous,
                text,
                counts,
                deleted,
                deleted_on,
                version,
                value,
                last_tick,
                last_used,
            ],
        ) => {
            let tally = parse_tally(counts, deleted, deleted_on, Shape::Bigram, devices)?;
            let decayed = parse_decayed([version, value, last_tick, last_used], &tally)?;
            (previous, text, tally, decayed)
        }
        (Version::V3, _) => return Err("bigram 要有 10 欄（含種類），以 Tab 分隔"),
        (_, &[previous, text, value, last_tick]) => {
            let decayed = Decayed {
                value: positive(value)?,
                last_tick: integer(last_tick)?,
                last_used: today,
                version: 1,
            };
            (previous, text, old_tally("", 1, 0, None), Some(decayed))
        }
        _ => return Err("個人 bigram（方案欄空的）要有 5 欄，以 Tab 分隔"),
    };
    if previous.is_empty() || text.is_empty() {
        return Err("個人 bigram 的前一個字詞、這一段不能是空的");
    }
    Ok((unescape(previous)?, unescape(text)?, tally, decayed))
}

/// 英文使用者詞的一列去掉種類欄之後（v3）：寫法、各台次數與墓碑三欄、衰減值四欄（刪掉的留空，寫法是小寫的鍵）。
/// 寫法要是打字時會學的串（[`english_user::to_learn`]），不跳脫（串裡沒有反斜線、換行與 Tab）。
fn parse_english(
    line: &str,
    devices: &[String],
    draft: &str,
) -> Result<(String, Tally, Option<EnglishUserWord>), &'static str> {
    // 7ad01da 的 v3 草稿：6 欄 `english  寫法  次數  衰減值  最後 tick  最後使用的日期`，沒有各台的次數。
    if let &[text, count, value, last_tick, last_used] = &line.split('\t').collect::<Vec<_>>()[..] {
        let count = integer(count)?;
        if count == 0 {
            return Err("次數要是正整數");
        }
        check_english(text)?;
        let last_used = parse_date(last_used)?;
        let value: f64 = value.parse().map_err(|_| "衰減值要是數字")?;
        if !value.is_finite() || value <= 0.0 {
            return Err("英文使用者詞的衰減值要是正的有限數");
        }
        let word = EnglishUserWord {
            text: text.to_string(),
            decayed: Decayed {
                value,
                last_tick: integer(last_tick)?,
                last_used,
                version: count,
            },
        };
        return Ok((
            english_user::key(text),
            old_tally(draft, count, count, last_used),
            Some(word),
        ));
    }
    let &[
        text,
        counts,
        deleted,
        deleted_on,
        version,
        value,
        last_tick,
        last_used,
    ] = &line.split('\t').collect::<Vec<_>>()[..]
    else {
        return Err("english 要有 9 欄（含種類），以 Tab 分隔");
    };
    check_english(text)?;
    let tally = parse_tally(counts, deleted, deleted_on, Shape::English, devices)?;
    let word = parse_decayed([version, value, last_tick, last_used], &tally)?.map(|decayed| {
        EnglishUserWord {
            text: text.to_string(),
            decayed,
        }
    });
    Ok((english_user::key(text), tally, word))
}

/// v2 的記住的字串（方案欄、前一個字詞欄空的列）去掉兩個空欄之後：字串、衰減值、最後 tick；最後使用
/// 的日期取 `today`。匯入成已過門檻的英文使用者詞（次數 [`ENGLISH_COMPLETION_AFTER`]，前綴補完也給），
/// 次數記在不知道哪台的那格。
fn parse_string(
    line: &str,
    today: Option<Date>,
) -> Result<(String, Tally, EnglishUserWord), &'static str> {
    let &[text, value, last_tick] = &line.split('\t').collect::<Vec<_>>()[..] else {
        return Err("記住的字串（方案欄、前一個字詞欄空的）要有 5 欄，以 Tab 分隔");
    };
    let text = unescape(text)?;
    check_english(&text)?;
    let value: f64 = value.parse().map_err(|_| "衰減值要是數字")?;
    if !value.is_finite() || value <= 0.0 {
        return Err("英文使用者詞的衰減值要是正的有限數");
    }
    let count = ENGLISH_COMPLETION_AFTER;
    let word = EnglishUserWord {
        text: text.to_string(),
        decayed: Decayed {
            value,
            last_tick: integer(last_tick)?,
            last_used: today,
            version: count,
        },
    };
    Ok((
        english_user::key(&text),
        old_tally("", count, count, today),
        word,
    ))
}

/// 英文使用者詞的寫法要是打字時會學的串。
fn check_english(text: &str) -> Result<(), &'static str> {
    if english_user::to_learn(text) != Some(text) {
        return Err(
            "英文使用者詞要是打字時會學的串（英文、數字與 . _ % + - @ '，夠長，頭尾不是 . 或 '）",
        );
    }
    Ok(())
}

/// 把匯入的一個英文使用者詞加進 `user`；最後 tick 比 tick 行（`tick`，沒有時不查）大、或不分大小寫
/// 重複時是錯誤。
fn insert_english(
    user: &mut UserModel,
    key: String,
    tally: Tally,
    word: Option<EnglishUserWord>,
    tick: Option<u64>,
) -> Result<(), &'static str> {
    if let Some(word) = &word
        && tick.is_some_and(|t| word.decayed.last_tick() > t)
    {
        return Err("最後 tick 比 tick 行的 tick 大");
    }
    if !tally.alive() {
        // 刪掉的詞（匯出時寫成雜湊，手寫的檔才會這樣）：埋進墓碑表。
        user.english
            .graveyard
            .insert(EnglishUserWords::digest(&key), tally);
        return Ok(());
    }
    if !user.english.insert(key, tally, word) {
        return Err("英文使用者詞重複（不分大小寫）");
    }
    Ok(())
}

/// 詞欄的跳脫（v2）：反斜線、換行、Tab 寫成 `\\`、`\n`、`\t`，照 PostgreSQL `COPY` 的文字格式與
/// Linear TSV；常用語可以含換行與 Tab，TSV 放不下原字。
fn escape(text: &str) -> Cow<'_, str> {
    if !text.contains(['\\', '\n', '\t']) {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len() + 2);
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    Cow::Owned(out)
}

/// [`escape`] 的反向；反斜線後不是 `\\`、`n`、`t` 時是錯的。
fn unescape(field: &str) -> Result<Cow<'_, str>, &'static str> {
    if !field.contains('\\') {
        return Ok(Cow::Borrowed(field));
    }
    let mut out = String::with_capacity(field.len());
    let mut chars = field.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        out.push(match chars.next() {
            Some('\\') => '\\',
            Some('n') => '\n',
            Some('t') => '\t',
            _ => return Err("詞欄的反斜線後面要是 \\、n 或 t"),
        });
    }
    Ok(Cow::Owned(out))
}

/// [`UserModel::import`] 不收的檔：第幾行（從 1 起）、為什麼。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportError {
    pub line: usize,
    pub reason: String,
}

impl ImportError {
    fn at(line: usize, reason: String) -> Self {
        Self { line, reason }
    }
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "第 {} 行：{}", self.line, self.reason)
    }
}

impl std::error::Error for ImportError {}
