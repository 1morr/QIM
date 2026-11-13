//! 停鍵結果快取（plan〈模型分層〉，M4 票 03）：同一份重排要求的結果留著，退格回到算過的那串碼時直接套用，
//! 殼不必再等停鍵、跑模型（青簡做法）。

use super::ScoredPath;

/// 快取最多留幾份結果（plan〈模型分層〉）：速成每字最多兩碼，32 份蓋得住一句 16 字的每個前綴；超過時丟
/// 最久沒用的。一份約 2 KB（前文 50 字、9 條路徑），滿了約 64 KB。
pub const RERANK_CACHE_LIMIT: usize = 32;

/// 快取的鍵：模型的輸入本身，前文（含選定的分段）與各路徑的文字、分段、路徑分。輸入碼、選項、學習、
/// 資料包變了，路徑或路徑分跟著變，鍵就不同；模型換了整份清掉（[`super::Session::set_reranker_id`]）。
#[derive(Debug, Clone, PartialEq)]
pub(super) struct RerankKey {
    pub(super) context: String,
    pub(super) paths: Vec<ScoredPath>,
}

/// 最近用的在後面。
#[derive(Debug, Clone, Default)]
pub(super) struct RerankCache {
    entries: Vec<(RerankKey, Vec<usize>)>,
}

impl RerankCache {
    pub(super) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn position(&self, key: &RerankKey) -> Option<usize> {
        self.entries.iter().position(|(k, _)| k == key)
    }

    pub(super) fn contains(&self, key: &RerankKey) -> bool {
        self.position(key).is_some()
    }

    /// `key` 的結果；用到的移到最後（最近用的）。
    pub(super) fn get(&mut self, key: &RerankKey) -> Option<&[usize]> {
        let at = self.position(key)?;
        let entry = self.entries.remove(at);
        self.entries.push(entry);
        self.entries.last().map(|(_, order)| order.as_slice())
    }

    /// 記下 `key` 的結果；已有的換掉，滿了丟最久沒用的。
    pub(super) fn insert(&mut self, key: RerankKey, order: &[usize]) {
        self.entries.retain(|(k, _)| *k != key);
        if self.entries.len() == RERANK_CACHE_LIMIT {
            self.entries.remove(0);
        }
        self.entries.push((key, order.to_vec()));
    }

    pub(super) fn clear(&mut self) {
        self.entries.clear();
    }
}
