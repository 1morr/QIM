use std::cmp::Reverse;
use std::collections::HashSet;

use crate::lattice::{self, Lattice};
use crate::user::personal_bonus;
use crate::{
    CodeEntry, CodeTable, LanguageModel, LanguageModelPack, Lexicon, LexiconPack, Pack, PackError,
    Scheme, UserModel, Word,
};

/// 不可變的方案與資料包，多個 [`crate::Session`] 共用。
#[derive(Debug, Clone)]
pub struct Engine {
    scheme: Scheme,
    /// 每個碼的字已依字頻重排（[`by_frequency`]）。
    code_table: CodeTable,
    /// 方案沒指名詞庫時是空的。
    lexicon: Lexicon,
    /// 方案沒指名語言模型時是 [`LanguageModel::default`]：每個字的分數都是 0。
    language_model: LanguageModel,
    /// 詞庫裡最長的碼的長度，lattice 往回找詞的上限。
    longest_word_code: usize,
    /// 詞的 unigram 的分母取 ln：每個詞（不論幾個碼）的詞頻總和。
    word_log_total: f64,
}

/// lattice 的一條邊：輸入碼 `[start, end)` 對到的一個字或詞。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Edge {
    pub start: usize,
    pub text: String,
    /// 第一個字；它的分數看前一段的最後一個字，見 [`Engine::edge_score`]。
    first: char,
    /// 不看前一段的部分（plan §2.3）：字是 −γ；詞是 β − γ 加上詞頻換算的詞內機率。
    inner: f64,
    /// 第一個字之後的 ln 機率：詞是詞頻換算的詞內機率，字是 0；`personal` 要的靜態機率用。
    rest: f64,
    /// 個人機率（plan §2.3 的 `personal`）；沒學過是 0。
    personal: f64,
}

impl Engine {
    /// `lexicon`、`language_model` 要剛好是方案 [`crate::Packs`] 指名的那份；方案沒指名時要是 `None`。
    pub fn new(
        pack: Pack,
        lexicon: Option<LexiconPack>,
        language_model: Option<LanguageModelPack>,
    ) -> Result<Self, PackError> {
        let found = lexicon.as_ref().map(|l| l.id.clone());
        if pack.scheme.packs.lexicon != found {
            return Err(PackError::LexiconMismatch {
                expected: pack.scheme.packs.lexicon,
                found,
            });
        }
        let found = language_model.as_ref().map(|m| m.id.clone());
        if pack.scheme.packs.language_model != found {
            return Err(PackError::LanguageModelMismatch {
                expected: pack.scheme.packs.language_model,
                found,
            });
        }
        let lexicon = lexicon.map(|l| l.lexicon).unwrap_or_default();
        let longest_word_code = lexicon
            .iter()
            .map(|(code, _)| code.len())
            .max()
            .unwrap_or(0);
        let mut seen = HashSet::new();
        let word_total: u64 = lexicon
            .iter()
            .flat_map(|(_, words)| words)
            .filter(|w| seen.insert(w.text.as_str()))
            .map(|w| w.frequency)
            .sum();
        Ok(Self {
            word_log_total: (word_total.max(1) as f64).ln(),
            code_table: by_frequency(&pack.code_table, &lexicon),
            scheme: pack.scheme,
            language_model: language_model.map(|m| m.model).unwrap_or_default(),
            longest_word_code,
            lexicon,
        })
    }

    /// 由資料包的位元組建立；讀檔是呼叫端的事（plan §1.3）。
    pub fn from_pack_bytes(
        pack: &[u8],
        lexicon: Option<&[u8]>,
        language_model: Option<&[u8]>,
    ) -> Result<Self, PackError> {
        let lexicon = lexicon.map(LexiconPack::from_bytes).transpose()?;
        let language_model = language_model
            .map(LanguageModelPack::from_bytes)
            .transpose()?;
        Self::new(Pack::from_bytes(pack)?, lexicon, language_model)
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

    /// 碼 `code` 的一個字的邊：第一個字就是它，不看前文的部分只有 −γ。
    fn char_edge(&self, user: &UserModel, code: &str, start: usize, c: char) -> Edge {
        let text = c.to_string();
        Edge {
            start,
            personal: user.personal_probability(&self.scheme.id, code, &text),
            text,
            first: c,
            inner: -self.scheme.ranking.segment_penalty,
            rest: 0.0,
        }
    }

    /// 一個詞的邊（plan §2.3）：`P(詞 | 前一字) ≈ P(第一字 | 前一字) × P_詞頻(詞) ÷ P(第一字)`，
    /// 第一個字之後由詞頻換算，所以詞頻仍分得出同碼的詞；再加 β − γ。
    fn word_edge(&self, user: &UserModel, code: &str, start: usize, word: &Word) -> Edge {
        let ranking = &self.scheme.ranking;
        let first = word.text.chars().next().expect("詞不是空的");
        let rest = (word.frequency.max(1) as f64).ln()
            - self.word_log_total
            - self.language_model.log_prob(None, first);
        Edge {
            start,
            personal: user.personal_probability(&self.scheme.id, code, &word.text),
            text: word.text.clone(),
            first,
            inner: rest + ranking.word_bonus - ranking.segment_penalty,
            rest,
        }
    }

    /// 一個使用者詞的邊（plan §2.4）：沒有詞頻，詞內由字級 bigram 一字接一字估，不加 β；
    /// 只算一段，所以比拆成單字的路徑少扣 γ。
    fn user_word_edge(&self, user: &UserModel, code: &str, start: usize, text: &str) -> Edge {
        let mut chars = text.chars();
        let first = chars.next().expect("使用者詞不是空的");
        let (rest, _) = chars.fold((0.0, first), |(sum, prev), c| {
            (sum + self.language_model.log_prob(Some(prev), c), c)
        });
        Edge {
            start,
            personal: user.personal_probability(&self.scheme.id, code, text),
            text: text.to_string(),
            first,
            inner: rest - self.scheme.ranking.segment_penalty,
            rest,
        }
    }

    /// `text` 各字的碼接起來；有字不在碼表裡時是 `None`。一個字有幾個碼時取它排得最前的
    /// 那個碼，同名次取短的、再取字典序前的。
    pub(crate) fn code_of(&self, text: &str) -> Option<String> {
        text.chars()
            .map(|c| {
                self.code_table
                    .iter()
                    .filter(|(code, _)| code.len() <= usize::from(self.scheme.max_code_length))
                    .filter_map(|(code, entries)| {
                        let rank = entries.iter().position(|e| e.text == c)?;
                        Some((rank, code.len(), code))
                    })
                    .min()
                    .map(|(_, _, code)| code)
            })
            .collect()
    }

    /// 碼表或詞庫裡有沒有碼 `code` 的 `text`；使用者詞不算。
    pub(crate) fn has_entry(&self, code: &str, text: &str) -> bool {
        let mut chars = text.chars();
        let is_char = match (chars.next(), chars.next()) {
            (Some(c), None) => self.char_entries(code).iter().any(|e| e.text == c),
            _ => false,
        };
        is_char || self.lexicon.lookup(code).iter().any(|w| w.text == text)
    }

    /// 邊接在最後一個字是 `prev` 的部分路徑後面的分數；`prev` 是 `None` 表示它是組字的開頭。
    /// 靜態機率接上前一字後才知道，`personal` 也在這裡算（plan §2.3）。
    pub(crate) fn edge_score(&self, prev: Option<char>, edge: &Edge) -> f64 {
        let first = self.language_model.log_prob(prev, edge.first);
        first + edge.inner + personal_bonus(first + edge.rest, edge.personal)
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

    /// 剛好是碼 `code` 的連打字集的字與詞，接著使用者詞。字取字頻最高的 `limit` 個（字序在
    /// [`Engine::new`] 排過），另加 `user` 學過的：它們有 `personal`，字頻排名在後也可能勝出。
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
            .map(move |(_, e)| self.char_edge(user, code, start, e.text));
        let words = self
            .lexicon
            .lookup(code)
            .iter()
            .map(move |w| self.word_edge(user, code, start, w));
        let user_words = user
            .words(&self.scheme.id, code)
            .map(move |text| self.user_word_edge(user, code, start, text));
        chars.chain(words).chain(user_words)
    }

    /// 結束在 `input` 結尾的 lattice 邊：每個起點的碼表字取字頻最高的 beam 個與學過的，
    /// 加上詞庫的詞與使用者詞。擴展區的字不進 lattice（plan §2.3）。
    pub(crate) fn edges_ending(&self, user: &UserModel, input: &str) -> Vec<Edge> {
        let end = input.len();
        let longest = self
            .longest_word_code
            .max(usize::from(self.scheme.max_code_length))
            .max(user.longest_word_code(&self.scheme.id));
        (end.saturating_sub(longest)..end)
            .flat_map(|start| self.exact(user, &input[start..], start, self.beam()))
            .collect()
    }

    /// 碼 `code` 的全部連打字集的字與詞與它們在組字開頭的分數，依分數由高到低（同分照碼表、
    /// 詞庫的順序）。
    pub(crate) fn span(&self, user: &UserModel, code: &str) -> Vec<(f64, String)> {
        let mut edges: Vec<(f64, String)> = self
            .exact(user, code, 0, usize::MAX)
            .map(|e| (self.edge_score(None, &e), e.text))
            .collect();
        edges.sort_by(|a, b| b.0.total_cmp(&a.0));
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
