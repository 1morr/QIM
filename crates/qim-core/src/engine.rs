use std::cmp::Reverse;
use std::collections::HashSet;

use crate::lattice::{self, Lattice};
use crate::{CodeEntry, CodeTable, Lexicon, LexiconPack, Pack, PackError, Scheme, UserModel};

/// 不可變的方案與資料包，多個 [`crate::Session`] 共用。
#[derive(Debug, Clone)]
pub struct Engine {
    scheme: Scheme,
    /// 每個碼的字已依字頻重排（[`by_frequency`]），也就是依字的分數由高到低。
    code_table: CodeTable,
    /// 方案沒指名詞庫時是空的。
    lexicon: Lexicon,
    /// 字的 unigram 的分母取 ln：字頻總和加連打字集的字數（加一平滑）。
    char_log_total: f64,
    /// 詞的 unigram 的分母取 ln：每個詞（不論幾個碼）的詞頻總和。
    word_log_total: f64,
    /// 詞庫裡最長的碼的長度，lattice 往回找詞的上限。
    longest_word_code: usize,
}

/// lattice 的一條邊：輸入碼 `[start, end)` 對到的一個字或詞。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Edge {
    pub start: usize,
    pub text: String,
    /// 這一段的分數，已含 β、γ 與 `personal`（plan §2.3）。
    pub score: f64,
}

impl Engine {
    /// `lexicon` 要剛好是方案 [`crate::Packs::lexicon`] 指名的那份；方案沒指名時要是 `None`。
    pub fn new(pack: Pack, lexicon: Option<LexiconPack>) -> Result<Self, PackError> {
        let found = lexicon.as_ref().map(|l| l.id.clone());
        if pack.scheme.packs.lexicon != found {
            return Err(PackError::LexiconMismatch {
                expected: pack.scheme.packs.lexicon,
                found,
            });
        }
        let lexicon = lexicon.map(|l| l.lexicon).unwrap_or_default();
        let continuous: HashSet<char> = pack
            .code_table
            .iter()
            .flat_map(|(_, entries)| entries.iter().filter(|e| !e.extension).map(|e| e.text))
            .collect();
        let char_total: u64 = continuous.iter().map(|&c| lexicon.char_frequency(c)).sum();
        let mut seen = HashSet::new();
        let word_total: u64 = lexicon
            .iter()
            .flat_map(|(_, words)| words)
            .filter(|w| seen.insert(w.text.as_str()))
            .map(|w| w.frequency)
            .sum();
        let longest_word_code = lexicon
            .iter()
            .map(|(code, _)| code.len())
            .max()
            .unwrap_or(0);
        Ok(Self {
            code_table: by_frequency(&pack.code_table, &lexicon),
            scheme: pack.scheme,
            char_log_total: ((char_total + continuous.len() as u64) as f64).ln(),
            word_log_total: (word_total.max(1) as f64).ln(),
            longest_word_code,
            lexicon,
        })
    }

    /// 由資料包的位元組建立；讀檔是呼叫端的事（plan §1.3）。
    pub fn from_pack_bytes(pack: &[u8], lexicon: Option<&[u8]>) -> Result<Self, PackError> {
        let lexicon = lexicon.map(LexiconPack::from_bytes).transpose()?;
        Self::new(Pack::from_bytes(pack)?, lexicon)
    }

    pub fn scheme(&self) -> &Scheme {
        &self.scheme
    }

    /// 整串輸入碼在 `user` 之下的全部候選，從頭建 lattice；順序見 [`lattice::candidates`]。
    /// [`crate::Session`] 逐鍵補算，結果和這裡相同。
    pub fn candidates(&self, user: &UserModel, input: &str) -> Vec<String> {
        lattice::candidates(self, user, &Lattice::from_input(self, user, input))
            .into_iter()
            .map(|c| c.text)
            .collect()
    }

    pub(crate) fn beam(&self) -> usize {
        usize::from(self.scheme.ranking.beam)
    }

    /// 字的 ln 機率：字頻的 unigram（加一平滑）。
    fn char_log_prob(&self, c: char) -> f64 {
        ((self.lexicon.char_frequency(c) + 1) as f64).ln() - self.char_log_total
    }

    /// 詞的 ln 機率：詞頻的 unigram。
    fn word_log_prob(&self, frequency: u64) -> f64 {
        (frequency.max(1) as f64).ln() - self.word_log_total
    }

    /// 碼 `code` 的一段 `text` 的分數（plan §2.3）：ln 機率 `log_prob`、加 `bonus`（詞是 β，
    /// 字是 0）、扣 γ，再加 `personal`。
    fn segment_score(
        &self,
        user: &UserModel,
        code: &str,
        text: &str,
        log_prob: f64,
        bonus: f64,
    ) -> f64 {
        log_prob + bonus - self.scheme.ranking.segment_penalty
            + user.personal(&self.scheme.id, code, text, log_prob)
    }

    pub(crate) fn sentences(&self) -> usize {
        usize::from(self.scheme.ranking.sentences)
    }

    /// 碼表裡碼 `code` 的字；碼比方案的最長碼長時不是字的碼，是空的。
    fn char_entries(&self, code: &str) -> &[CodeEntry] {
        if code.len() <= usize::from(self.scheme.max_code_length) {
            self.code_table.lookup(code)
        } else {
            &[]
        }
    }

    /// 剛好是碼 `code` 的連打字集的字與詞。字取字頻最高的 `limit` 個（字序在 [`Engine::new`]
    /// 依字頻排過），另加 `user` 學過的：它們有 `personal`，靜態排名在後也可能進 beam。
    /// 詞全給，詞庫的順序由建置端決定，核心不靠它截斷。
    fn exact<'a>(
        &'a self,
        user: &'a UserModel,
        code: &'a str,
        start: usize,
        limit: usize,
    ) -> impl Iterator<Item = Edge> + 'a {
        let chars = self
            .char_entries(code)
            .iter()
            .filter(|e| !e.extension)
            .enumerate()
            .filter(move |&(i, e)| {
                i < limit
                    || user.has_learned(&self.scheme.id, code, e.text.encode_utf8(&mut [0; 4]))
            })
            .map(|(_, e)| e)
            .map(move |e| {
                let text = e.text.to_string();
                let score = self.segment_score(user, code, &text, self.char_log_prob(e.text), 0.0);
                Edge { start, text, score }
            });
        let words = self.lexicon.lookup(code).iter().map(move |w| {
            let log_prob = self.word_log_prob(w.frequency);
            Edge {
                start,
                text: w.text.clone(),
                score: self.segment_score(
                    user,
                    code,
                    &w.text,
                    log_prob,
                    self.scheme.ranking.word_bonus,
                ),
            }
        });
        chars.chain(words)
    }

    /// 結束在 `input` 結尾的 lattice 邊：每個起點的碼表字取字頻最高的 beam 個（沒學過的再多也
    /// 進不了 beam）與學過的，加上詞庫的詞。擴展區的字不進 lattice（plan §2.3）。
    pub(crate) fn edges_ending(&self, user: &UserModel, input: &str) -> Vec<Edge> {
        let end = input.len();
        let longest = self
            .longest_word_code
            .max(usize::from(self.scheme.max_code_length));
        (end.saturating_sub(longest)..end)
            .flat_map(|start| self.exact(user, &input[start..], start, self.beam()))
            .collect()
    }

    /// 碼 `code` 的全部連打字集的字與詞，依分數由高到低（同分照碼表、詞庫的順序）。
    pub(crate) fn span(&self, user: &UserModel, code: &str) -> Vec<Edge> {
        let mut edges: Vec<Edge> = self.exact(user, code, 0, usize::MAX).collect();
        edges.sort_by(|a, b| b.score.total_cmp(&a.score));
        edges
    }

    /// 碼 `code` 的擴展區的字，依碼表的順序。
    pub(crate) fn extension(&self, code: &str) -> impl Iterator<Item = char> {
        self.char_entries(code)
            .iter()
            .filter(|e| e.extension)
            .map(|e| e.text)
    }
}

/// 字序：字頻高的在前；字頻相同（含都沒有字頻）時照碼表原本的順序，也就是建置端排好的
/// 年級 → Big5（plan §4.1）。擴展區的字仍在連打字集之後。
fn by_frequency(code_table: &CodeTable, lexicon: &Lexicon) -> CodeTable {
    CodeTable::from_entries(code_table.iter().map(|(code, entries)| {
        let mut entries = entries.to_vec();
        entries.sort_by_key(|e| (e.extension, Reverse(lexicon.char_frequency(e.text))));
        (code.to_string(), entries)
    }))
}
