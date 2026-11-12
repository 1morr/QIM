//! 退格撤銷（plan §2.4〈退格撤銷〉，M4 票 15）：一個動作學到的東西的撤銷紀錄，照 librime `memory.cc`
//! 的 `DiscardSession`（每次上屏開一個 transaction，緊接著的退格就 abort，連 tick 一起回滾）。每個被
//! 改到的條目記下這個動作第一次改它之前的值（之前沒有時是 `None`），撤銷時放回去；上限淘汰刪掉的條目
//! 也記，所以撤銷後整份學習資料和上屏前一樣。動作的界線由 [`crate::Session`] 決定。

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

use super::{Bigrams, Count, Decayed, SchemeModel, UserModel, UserWord};
use crate::english_user::{EnglishUserWord, EnglishUserWords};

/// 寫撤銷紀錄的是哪個 [`crate::Session`]：幾個 Session 共用一份個人模型時（一個輸入框一個），只撤銷
/// 同一個 Session 的上一個動作，別的輸入框的退格不撤銷這裡的上屏。每個新的 Session 拿一個新的號碼，
/// 複製出來的 Session 同號。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct UndoOwner(u64);

impl Default for UndoOwner {
    fn default() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

/// 一個動作的撤銷紀錄。
#[derive(Debug, Clone, Default)]
pub(crate) struct Undo {
    /// 寫這份紀錄的 Session；使用者明確的操作清掉紀錄後是 `None`，誰都不能撤銷。
    owner: Option<UndoOwner>,
    schemes: BTreeMap<String, SchemeUndo>,
    bigrams: Option<BigramsUndo>,
    english: Option<EnglishUndo>,
}

/// 一個方案改到的列：（碼, 詞）→ 改之前的計數與使用者詞。
#[derive(Debug, Clone)]
pub(super) struct SchemeUndo {
    /// 動作之前有沒有這個方案；沒有時撤銷整個刪掉，不留空的方案。
    existed: bool,
    tick: u64,
    rows: BTreeMap<(String, String), Row>,
}

/// 一列（碼, 詞）改之前的計數與使用者詞；`None` 是原本沒有。
#[derive(Debug, Clone)]
struct Row {
    count: Option<Count>,
    word: Option<UserWord>,
}

/// 改到的個人 bigram：（前一個字詞, 這一段）→ 改之前的衰減值。
#[derive(Debug, Clone)]
pub(super) struct BigramsUndo {
    tick: u64,
    pairs: BTreeMap<(String, String), Option<Decayed>>,
}

/// 改到的英文使用者詞：小寫的鍵 → 改之前的詞。
#[derive(Debug, Clone)]
pub(super) struct EnglishUndo {
    tick: u64,
    words: BTreeMap<String, Option<EnglishUserWord>>,
}

impl Undo {
    /// `owner` 要開始一個動作：之後學到的記在這份新的紀錄裡。
    pub(super) fn new(owner: UndoOwner) -> Self {
        Self {
            owner: Some(owner),
            ..Self::default()
        }
    }

    /// 這份紀錄是 `owner` 寫的。
    pub(super) fn is_by(&self, owner: UndoOwner) -> bool {
        self.owner == Some(owner)
    }

    /// 這個動作沒改到任何條目。
    pub(super) fn is_empty(&self) -> bool {
        self.schemes.is_empty() && self.bigrams.is_none() && self.english.is_none()
    }

    /// 方案 `scheme` 的紀錄；這個動作第一次改到它時記下它的 tick。
    pub(super) fn scheme(
        &mut self,
        schemes: &BTreeMap<String, SchemeModel>,
        scheme: &str,
    ) -> &mut SchemeUndo {
        self.schemes
            .entry(scheme.to_string())
            .or_insert_with(|| SchemeUndo {
                existed: schemes.contains_key(scheme),
                tick: schemes.get(scheme).map_or(0, |m| m.tick),
                rows: BTreeMap::new(),
            })
    }

    /// 方案 `scheme` 的（`code`, `text`）要刪了（上限淘汰）：這個動作第一次改它時記下它現在的值。
    pub(super) fn save_row(
        &mut self,
        schemes: &BTreeMap<String, SchemeModel>,
        scheme: &str,
        code: &str,
        text: &str,
    ) {
        if let Some(model) = schemes.get(scheme) {
            self.scheme(schemes, scheme).save(model, code, text);
        }
    }

    /// 個人 bigram 的 tick 要推進了：這個動作第一次改到時記下它。
    pub(super) fn save_bigram_tick(&mut self, bigrams: &Bigrams) -> &mut BigramsUndo {
        self.bigrams.get_or_insert_with(|| BigramsUndo {
            tick: bigrams.tick,
            pairs: BTreeMap::new(),
        })
    }

    /// 個人 bigram（`previous`, `text`）要改了：這個動作第一次改它時記下它現在的衰減值（與 tick）。
    pub(super) fn save_bigram(&mut self, bigrams: &Bigrams, previous: &str, text: &str) {
        self.save_bigram_tick(bigrams)
            .pairs
            .entry((previous.to_string(), text.to_string()))
            .or_insert_with(|| {
                bigrams
                    .entries
                    .get(previous)
                    .and_then(|n| n.get(text))
                    .copied()
            });
    }

    /// 鍵是 `key` 的英文使用者詞要改了：這個動作第一次改它時記下它現在的樣子（與 tick）。
    pub(super) fn save_english(&mut self, english: &EnglishUserWords, key: &str) {
        self.english
            .get_or_insert_with(|| EnglishUndo {
                tick: english.tick,
                words: BTreeMap::new(),
            })
            .words
            .entry(key.to_string())
            .or_insert_with(|| english.words.get(key).cloned());
    }

    /// 把記下的值放回 `user`：學習資料回到這個動作之前。
    pub(super) fn revert(self, user: &mut UserModel) {
        for (scheme, undo) in self.schemes {
            if !undo.existed {
                user.schemes.remove(&scheme);
                continue;
            }
            let model = user.schemes.entry(scheme).or_default();
            model.tick = undo.tick;
            for ((code, text), Row { count, word }) in undo.rows {
                put(&mut model.entries, code.clone(), text.clone(), count);
                put(&mut model.words, code, text, word);
            }
        }
        if let Some(undo) = self.bigrams {
            user.bigrams.tick = undo.tick;
            for ((previous, text), decayed) in undo.pairs {
                put(&mut user.bigrams.entries, previous, text, decayed);
            }
        }
        if let Some(undo) = self.english {
            user.english.tick = undo.tick;
            for (key, word) in undo.words {
                match word {
                    Some(word) => user.english.words.insert(key, word),
                    None => user.english.words.remove(&key),
                };
            }
        }
    }
}

impl SchemeUndo {
    /// （`code`, `text`）要改了：這個動作第一次改它時記下它現在的計數與使用者詞。
    pub(super) fn save(&mut self, model: &SchemeModel, code: &str, text: &str) {
        self.rows
            .entry((code.to_string(), text.to_string()))
            .or_insert_with(|| Row {
                count: model.count(code, text).copied(),
                word: model.word(code, text).cloned(),
            });
    }
}

/// 兩層的表裡放回一個值：`None` 是原本沒有，刪掉它（裡層刪光時連外層一起刪）。
fn put<V>(
    map: &mut BTreeMap<String, BTreeMap<String, V>>,
    outer: String,
    inner: String,
    value: Option<V>,
) {
    match value {
        Some(value) => {
            map.entry(outer).or_default().insert(inner, value);
        }
        None => {
            if let Some(next) = map.get_mut(&outer) {
                next.remove(&inner);
                if next.is_empty() {
                    map.remove(&outer);
                }
            }
        }
    }
}
