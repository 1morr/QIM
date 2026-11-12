use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet};

use crate::lattice::{self, Lattice, Listing, Preceding};
use crate::lexicon::WordsByText;
use crate::pack::Pack;
use crate::scheme::{SEPARATOR, without_separators};
use crate::session::SegmentSource;
use crate::user::{SchemeModel, personal_bonus};
use crate::{
    CLAUSE_START, CodeEntry, CodeTable, EmojiPack, EmojiTable, EnglishPack, EnglishWords,
    LanguageModel, LanguageModelPack, Lexicon, LexiconPack, Manifest, PackError, PackKind, Scheme,
    SchemePack, Segment, Simplifier, SimplifierPack, UserModel, Word, WordTrigram, WordTrigramPack,
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
    /// 簡體模式的轉換；沒掛上簡繁資料包時是 `None`，簡體模式照原樣輸出。
    simplifier: Option<Simplifier>,
    /// 逐字轉回繁體（[`Simplifier::traditional_chars`]，一對多取語言模型 unigram 最高的）；沒掛上
    /// 簡繁資料包時是空的。
    traditional_chars: HashMap<char, char>,
    /// 英文候選的詞表；沒掛上英文資料包時是空的，沒有英文候選。
    english: EnglishWords,
    /// emoji 候選的對照表；沒掛上 emoji 資料包時是空的，沒有 emoji 候選。
    emoji: EmojiTable,
    /// 詞庫的詞依文字排序和它的 ln unigram（詞頻 ÷ 總和），聯想用（[`Engine::words_starting_with`]）。
    words_by_text: WordsByText,
    /// 連打字集的字（碼表裡不是擴展區的字）：聯想只給連打打得出的字。
    continuous_chars: HashSet<char>,
    /// 詞級 trigram（[`Engine::with_word_trigram`]）：有掛上時 lattice 的靜態機率改用它，沒有統計的
    /// 地方退回字級 bigram；`None` 只用字級 bigram（plan §2.3）。
    word_trigram: Option<WordTrigram>,
}

/// 一段前面的兩段在詞級 trigram 詞表的編號：`[前前一段, 前一段]`，沒有或不在詞表是 `None`。
pub(crate) type WordHistory = [Option<u32>; 2];

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
    /// 碼表標成簡體字的字（[`CodeEntry::simplified`]）；詞不是。
    simplified: bool,
    /// 在詞級 trigram 詞表的編號；沒掛上或不在詞表是 `None`。
    word: Option<u32>,
    /// 碼表的字、詞庫的詞或使用者詞（debug 面板，[`Engine::inspect_path`]）。
    source: SegmentSource,
}

/// 一條路徑的一段的分數與來源（[`Engine::inspect_path`]）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ScoredSegment {
    pub source: SegmentSource,
    /// 這一段接在前一段後面的分數（[`Engine::edge_score`]），含學習加分。
    pub score: f64,
    /// 其中 `personal` 的部分（plan §2.3）：學到的加分。
    pub learning: f64,
    /// 有這一段的學習計數。
    pub learned: bool,
    /// 有它接在前一個字詞後面的個人 bigram。
    pub personal_bigram: bool,
}

/// [`Engine::span`] 的一項：剛好是一段輸入碼的一個字或詞。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Spanned {
    /// 在組字開頭的分數。
    pub score: f64,
    pub text: String,
    /// 碼表標成簡體字的字：繁體模式排在同一段的其他字詞之後（brief §4.1）。
    pub simplified: bool,
}

impl Engine {
    /// 方案 [`crate::Packs`] 指名詞庫、語言模型時要給，沒指名時要是 `None`。方案指名的是預設的那份，
    /// 殼可以換成同種的另一份（設定頁的切換，plan §4.6），所以不核對 id。
    pub fn new(
        scheme_pack: SchemePack,
        lexicon: Option<LexiconPack>,
        language_model: Option<LanguageModelPack>,
    ) -> Result<Self, PackError> {
        // 資料包可以從檔案匯入、不一定經 `qim-databuild`，載入時再驗一次。
        scheme_pack
            .scheme
            .ranking
            .check()
            .map_err(|e| PackError::Malformed(e.to_string()))?;
        let found = lexicon.as_ref().map(|l| l.id.clone());
        if scheme_pack.scheme.packs.lexicon.is_some() != found.is_some() {
            return Err(PackError::LexiconMismatch {
                expected: scheme_pack.scheme.packs.lexicon,
                found,
            });
        }
        let found = language_model.as_ref().map(|m| m.id.clone());
        if scheme_pack.scheme.packs.language_model.is_some() != found.is_some() {
            return Err(PackError::LanguageModelMismatch {
                expected: scheme_pack.scheme.packs.language_model,
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
        let word_log_total = (word_total.max(1) as f64).ln();
        // 同文字留碼的字典序最前的那個，和上面的詞頻總和取的相同。
        let words_by_text = WordsByText::new(&lexicon, |w| {
            (w.frequency.max(1) as f64).ln() - word_log_total
        });
        let continuous_chars = scheme_pack
            .code_table
            .iter()
            .flat_map(|(_, entries)| entries)
            .filter(|e| !e.extension)
            .map(|e| e.text)
            .collect();
        Ok(Self {
            word_log_total,
            words_by_text,
            continuous_chars,
            code_table: by_frequency(&scheme_pack.code_table, &lexicon),
            scheme: scheme_pack.scheme,
            language_model: language_model.map(|m| m.model).unwrap_or_default(),
            longest_word_code,
            lexicon,
            simplifier: None,
            traditional_chars: HashMap::new(),
            english: EnglishWords::default(),
            emoji: EmojiTable::default(),
            word_trigram: None,
        })
    }

    /// 掛上詞級 trigram 資料包（plan §2.3、§4.3）：lattice 的邊與整串的字詞的靜態機率改用它，
    /// β、γ、`personal` 照舊；聯想仍用字級 bigram。方案指名詞級 trigram 時 [`Engine::open`] 一定要給。
    #[must_use]
    pub fn with_word_trigram(mut self, pack: WordTrigramPack) -> Self {
        self.word_trigram = Some(pack.model);
        self
    }

    /// 有沒有掛上詞級 trigram：有才要算 [`Engine::edge_score`] 的前面兩段。
    pub(crate) fn has_word_trigram(&self) -> bool {
        self.word_trigram.is_some()
    }

    /// 字詞 `text` 在詞級 trigram 詞表的編號；沒掛上或不在詞表是 `None`。
    pub(crate) fn word_id(&self, text: &str) -> Option<u32> {
        self.word_trigram.as_ref()?.id(text)
    }

    /// 掛上簡繁資料包，簡體模式（[`crate::Options::simplified`]）才有作用。
    pub fn with_simplifier(mut self, pack: SimplifierPack) -> Self {
        let model = &self.language_model;
        self.traditional_chars = pack.simplifier.traditional_chars(|c| model.unigram(c));
        self.simplifier = Some(pack.simplifier);
        self
    }

    /// 掛上英文資料包，英文候選（[`crate::Options::english_candidates`]）才有作用。
    pub fn with_english(mut self, pack: EnglishPack) -> Self {
        self.english = pack.words;
        self
    }

    /// 掛上 emoji 資料包，emoji 候選（[`crate::Options::emoji_candidates`]）才有作用。
    pub fn with_emoji(mut self, pack: EmojiPack) -> Self {
        self.emoji = pack.emoji;
        self
    }

    /// 字詞 `word`（繁體原文）的 emoji，好的在前。
    pub(crate) fn emoji(&self, word: &str) -> &[String] {
        self.emoji.lookup(word)
    }

    /// 整串輸入碼 `input` 的英文候選（排位規則 4，plan §2.4）：剛好是詞表裡的完整單字、又長過
    /// 方案一個字的最長碼時才有。不容錯、不補全；1–2 碼的英文字（`an`、`it`）和單字的碼撞得太常，
    /// macOS 速成也是第 3 碼起才出英文（黑箱 T7）。
    /// 輸入碼有萬用鍵時沒有：萬用鍵在中文組字是比對模式，不當字母（plan §4.4）。
    pub(crate) fn english_candidate(&self, input: &str) -> Option<&str> {
        (input.len() > usize::from(self.scheme.max_code_length) && !self.scheme.has_wildcard(input))
            .then(|| self.english.lookup(input))
            .flatten()
    }

    /// 簡體模式下輸出的文字；沒掛上簡繁資料包時照原樣。
    pub(crate) fn simplify(&self, text: &str) -> String {
        match &self.simplifier {
            Some(simplifier) => simplifier.convert(text),
            None => text.to_string(),
        }
    }

    /// 簡體字逐字轉回繁體（[`Engine::traditional_char`]）。
    pub(crate) fn traditional(&self, text: &str) -> String {
        text.chars().map(|c| self.traditional_char(c)).collect()
    }

    /// 簡體字轉回繁體（[`Engine::traditional_chars`]），其他字照原樣；沒掛上簡繁資料包時照原樣。
    pub(crate) fn traditional_char(&self, c: char) -> char {
        self.traditional_chars.get(&c).copied().unwrap_or(c)
    }

    /// 由資料包的位元組建立；讀檔是呼叫端的事（plan §1.3）。
    pub fn from_pack_bytes(
        scheme_pack: &[u8],
        lexicon: Option<&[u8]>,
        language_model: Option<&[u8]>,
    ) -> Result<Self, PackError> {
        let lexicon = lexicon.map(LexiconPack::from_bytes).transpose()?;
        let language_model = language_model
            .map(LanguageModelPack::from_bytes)
            .transpose()?;
        Self::new(
            SchemePack::from_bytes(scheme_pack)?,
            lexicon,
            language_model,
        )
    }

    /// 殼載入資料包的入口（plan §4.6）：每份照它的 manifest 經 [`Pack::open`] 核對，依種類放到該放
    /// 的位置，順序不拘。方案資料包一定要有，詞庫、語言模型照方案要不要（[`Engine::new`]），詞級
    /// trigram 也照方案要不要（只核對有沒有、不核對 id），簡繁、英文、emoji 可有可無；同一種兩份、
    /// 或給了字根提示、停鍵層的模型時拒絕。
    pub fn open(packs: &[(&Manifest, &[u8])]) -> Result<Self, PackError> {
        let mut by_kind = BTreeMap::new();
        for &(manifest, bytes) in packs {
            if matches!(manifest.kind, PackKind::RadicalHint | PackKind::Model) {
                return Err(PackError::NotForEngine(manifest.kind));
            }
            if by_kind.insert(manifest.kind, (manifest, bytes)).is_some() {
                return Err(PackError::DuplicateKind(manifest.kind));
            }
        }
        fn open<P: Pack>(file: Option<&(&Manifest, &[u8])>) -> Result<Option<P>, PackError> {
            file.map(|(manifest, bytes)| P::open(manifest, bytes))
                .transpose()
        }
        let scheme =
            open(by_kind.get(&PackKind::Scheme))?.ok_or(PackError::Missing(PackKind::Scheme))?;
        let mut engine = Self::new(
            scheme,
            open(by_kind.get(&PackKind::Lexicon))?,
            open(by_kind.get(&PackKind::LanguageModel))?,
        )?;
        let word_trigram: Option<WordTrigramPack> = open(by_kind.get(&PackKind::WordTrigram))?;
        let expected = engine.scheme.packs.word_trigram.clone();
        if expected.is_some() != word_trigram.is_some() {
            return Err(PackError::WordTrigramMismatch {
                expected,
                found: word_trigram.map(|p| p.id),
            });
        }
        if let Some(pack) = word_trigram {
            engine = engine.with_word_trigram(pack);
        }
        if let Some(pack) = open(by_kind.get(&PackKind::Simplifier))? {
            engine = engine.with_simplifier(pack);
        }
        if let Some(pack) = open(by_kind.get(&PackKind::English))? {
            engine = engine.with_english(pack);
        }
        if let Some(pack) = open(by_kind.get(&PackKind::Emoji))? {
            engine = engine.with_emoji(pack);
        }
        Ok(engine)
    }

    pub fn scheme(&self) -> &Scheme {
        &self.scheme
    }

    /// 整串輸入碼在 `user` 之下、預設選項（繁體模式、簡體字排後）的全部候選，從頭建 lattice；
    /// 順序見 [`lattice::candidates`]。不含英文候選、emoji 與記住的字串：它們是 [`crate::Session`] 的選項。
    /// [`crate::Session`] 逐鍵補算，結果和這裡相同。
    pub fn candidates(&self, user: &UserModel, input: &str) -> Vec<String> {
        let lattice = Lattice::from_input(self, user, Preceding::default(), input);
        let listing = Listing {
            english_candidates: false,
            emoji_candidates: false,
            remembered_strings: false,
            simplified_last: true,
        };
        lattice::candidates(self, user, &lattice, listing)
            .into_iter()
            .map(|c| c.text)
            .collect()
    }

    pub(crate) fn beam(&self) -> usize {
        usize::from(self.scheme.ranking.beam)
    }

    /// 碼 `code` 的一個字的邊：第一個字就是它，不看前文的部分只有 −γ。
    fn char_edge(&self, user: &SchemeModel, code: &str, start: usize, entry: &CodeEntry) -> Edge {
        let text = entry.text.to_string();
        Edge {
            start,
            personal: user.personal_probability(code, &text),
            word: self.word_id(&text),
            text,
            first: entry.text,
            inner: -self.scheme.ranking.segment_penalty,
            rest: 0.0,
            simplified: entry.simplified,
            source: SegmentSource::CodeTable,
        }
    }

    /// 一個詞的邊（plan §2.3）：`P(詞 | 前一字) ≈ P(第一字 | 前一字) × P_詞頻(詞) ÷ P(第一字)`，
    /// 第一個字之後由詞頻換算，所以詞頻仍分得出同碼的詞；再加 β − γ。
    fn word_edge(&self, user: &SchemeModel, code: &str, start: usize, word: &Word) -> Edge {
        let ranking = &self.scheme.ranking;
        let first = word.text.chars().next().expect("詞不是空的");
        let rest = (word.frequency.max(1) as f64).ln()
            - self.word_log_total
            - self.language_model.log_prob(None, first);
        Edge {
            start,
            personal: user.personal_probability(code, &word.text),
            word: self.word_id(&word.text),
            text: word.text.clone(),
            first,
            inner: rest + ranking.word_bonus - ranking.segment_penalty,
            rest,
            simplified: false,
            source: SegmentSource::Lexicon,
        }
    }

    /// 一個使用者詞的邊（plan §2.4）：沒有詞頻，詞內由字級 bigram 一字接一字估，不加 β；
    /// 只算一段，所以比拆成單字的路徑少扣 γ。
    fn user_word_edge(&self, user: &SchemeModel, code: &str, start: usize, text: &str) -> Edge {
        let mut chars = text.chars();
        let first = chars.next().expect("使用者詞不是空的");
        let (rest, _) = chars.fold((0.0, first), |(sum, prev), c| {
            (sum + self.language_model.log_prob(Some(prev), c), c)
        });
        Edge {
            start,
            personal: user.personal_probability(code, text),
            word: self.word_id(text),
            text: text.to_string(),
            first,
            inner: rest - self.scheme.ranking.segment_penalty,
            rest,
            simplified: false,
            source: SegmentSource::UserWord,
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

    /// 邊接在最後一個字是 `prev` 的部分路徑後面的分數；組字開頭接前文最後一字或句首（`CLAUSE_START`），
    /// `prev` 是 `None` 表示沒有統計可接、用 unigram（同 `Preceding::character`）。
    /// 靜態機率接上前一字後才知道，`personal` 也在這裡算（plan §2.3）；`bigram` 是這條邊接在
    /// 前一個字詞後面的個人 bigram（[`crate::user::Following::probability`]），沒有是 0。
    /// 掛上詞級 trigram 時，靜態機率改用它（退回現行打分），`before` 是前面兩段的編號。
    pub(crate) fn edge_score(
        &self,
        prev: Option<char>,
        before: WordHistory,
        edge: &Edge,
        bigram: f64,
    ) -> f64 {
        let (unlearned, learning) = self.edge_score_parts(prev, before, edge, bigram);
        unlearned + learning
    }

    /// [`Engine::edge_score`] 分成沒學習時的分數與 `personal`（學到的加分）兩部分。
    #[inline]
    fn edge_score_parts(
        &self,
        prev: Option<char>,
        before: WordHistory,
        edge: &Edge,
        bigram: f64,
    ) -> (f64, f64) {
        let first = self.language_model.log_prob(prev, edge.first);
        let Some(model) = &self.word_trigram else {
            let learning = personal_bonus(first + edge.rest, edge.personal, bigram);
            return (first + edge.inner, learning);
        };
        let [u, v] = before;
        let static_log_prob = model.score(u, v, edge.word, first + edge.rest);
        // `inner` 是詞內機率加 β、γ：減掉詞內機率剩 β、γ。
        (
            static_log_prob + (edge.inner - edge.rest),
            personal_bonus(static_log_prob, edge.personal, bigram),
        )
    }

    /// 接在 `before` 後面的一串分段（候選的各段，碼與原文）每段的分數與來源，同 lattice 逐段接前一段
    /// 打分（[`crate::lattice::Lattice::push`]），所以總和就是那條路徑的路徑分（debug 面板，
    /// [`crate::Session::inspect`]）。有一段不是碼表、詞庫或使用者詞裡這個碼的字詞（擴展區的字、
    /// 觸發詞）時是 `None`：它不在 lattice 裡，沒有路徑分。
    pub(crate) fn inspect_path(
        &self,
        user: &UserModel,
        before: &Preceding,
        segments: &[Segment],
    ) -> Option<Vec<ScoredSegment>> {
        let scheme_user = user.for_scheme(&self.scheme.id);
        // 每段前面的一段：組字前面的那一段，接著各段。
        let history: Vec<Option<u32>> = std::iter::once(before.segment())
            .chain(segments.iter().map(|s| Some(s.text.clone())))
            .map(|segment| segment.and_then(|s| self.word_id(&s)))
            .collect();
        segments
            .iter()
            .enumerate()
            .map(|(i, segment)| {
                let (prev, previous_word) = match i.checked_sub(1).map(|p| &segments[p]) {
                    Some(p) => (p.text.chars().next_back(), Some(p.text.as_str())),
                    None => (before.character, before.word.as_deref()),
                };
                let words = [i.checked_sub(1).and_then(|p| history[p]), history[i]];
                let bigram = previous_word
                    .and_then(|w| user.bigrams_after(w))
                    .map_or(0.0, |f| f.probability(&segment.text));
                let code = segment.code.trim_end_matches(SEPARATOR);
                self.exact(scheme_user, code, 0, usize::MAX)
                    .into_iter()
                    .filter(|e| e.text == segment.text)
                    .map(|e| {
                        let (unlearned, learning) = self.edge_score_parts(prev, words, &e, bigram);
                        ScoredSegment {
                            source: e.source,
                            score: unlearned + learning,
                            learning,
                            learned: e.personal > 0.0,
                            personal_bigram: bigram > 0.0,
                        }
                    })
                    .max_by(|a, b| a.score.total_cmp(&b.score))
            })
            .collect()
    }

    /// 碼 `code` 的字詞 `text` 從哪裡來（不在 lattice 裡的候選，[`Engine::inspect_path`] 是 `None` 時）：
    /// 碼表的字（含擴展區）、詞庫的詞，其餘是使用者詞（觸發詞）。
    pub(crate) fn source_of(&self, code: &str, text: &str) -> SegmentSource {
        let code = without_separators(code);
        let mut chars = text.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) if self.char_entries(&code).iter().any(|e| e.text == c) => {
                SegmentSource::CodeTable
            }
            _ if self.lexicon.lookup(&code).iter().any(|w| w.text == text) => {
                SegmentSource::Lexicon
            }
            _ => SegmentSource::UserWord,
        }
    }

    /// 組字接在前文後面時，開頭的字要接的前一字（plan §2.3）；`last` 是前文的最後一字（繁體，
    /// 簡體模式由呼叫端轉回），空的前文是 `None`。bigram 只在句內訓練（句子以不是字母也不是數字
    /// 的字元分開），所以照切句的規則：前文以字結尾時是那個字；以標點、空白或符號結尾，或是空的
    /// （輸入框的開頭）時是句首 [`CLAUSE_START`]。語言模型裡沒有它的統計（英文、數字、罕用字，
    /// 或資料包沒有句首）時是 `None`，照不知道前文打。
    pub(crate) fn context_char(&self, last: Option<char>) -> Option<char> {
        let last = last.filter(|c| c.is_alphanumeric()).unwrap_or(CLAUSE_START);
        self.language_model.has_statistics(last).then_some(last)
    }

    /// 詞庫裡以 `prefix` 開頭、比它長的詞與它的 ln unigram，依文字排序（聯想用，plan §2.3）。
    pub(crate) fn words_starting_with<'a>(
        &'a self,
        prefix: &'a str,
    ) -> impl Iterator<Item = (&'a str, f64)> + 'a {
        self.words_by_text.starting_with(prefix)
    }

    /// 詞庫的詞 `text` 的 ln unigram（詞頻 ÷ 每個詞的詞頻總和）；不是詞庫的詞時是 `None`。
    pub(crate) fn word_log_prob(&self, text: &str) -> Option<f64> {
        self.words_by_text.log_prob(text)
    }

    pub(crate) fn language_model(&self) -> &LanguageModel {
        &self.language_model
    }

    /// `c` 在連打字集裡：碼表有它、不是擴展區的字。
    pub(crate) fn is_continuous(&self, c: char) -> bool {
        self.continuous_chars.contains(&c)
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

    /// 剛好是輸入碼 `input`（整串輸入碼的一段）的連打字集的字與詞，接著使用者詞。字取字頻最高的
    /// `limit` 個（字序在 [`Engine::new`] 排過），另加 `user` 學過的：它們有 `personal`，字頻排名
    /// 在後也可能勝出。詞全給，詞庫的順序由建置端決定，核心不靠它截斷。
    ///
    /// `input` 中間可以有分隔記號（[`SEPARATOR`]，不在頭尾）：這裡一定要斷字，所以沒有字，只有
    /// 每個分隔記號都剛好落在兩字的碼之間的詞與使用者詞（[`Engine::splits_at`]），照 Rime 速成
    /// （M1 票 28b，brief §12.2）。查碼表、詞庫與學習計數的碼都去掉分隔記號。
    fn exact(&self, user: &SchemeModel, input: &str, start: usize, limit: usize) -> Vec<Edge> {
        debug_assert!(!input.starts_with(SEPARATOR) && !input.ends_with(SEPARATOR));
        let cuts = separator_cuts(input);
        if self.scheme.has_wildcard(input) {
            return self.wildcard_edges(user, &without_separators(input), &cuts, start, limit);
        }
        if cuts.is_empty() {
            return self.exact_code(user, input, start, limit).collect();
        }
        let code = without_separators(input);
        let words = self
            .lexicon
            .lookup(&code)
            .iter()
            .filter(|w| self.splits_at(&w.text, &code, &cuts))
            .map(|w| self.word_edge(user, &code, start, w));
        let user_words = user
            .words(&code)
            .filter(|text| self.splits_at(text, &code, &cuts))
            .map(|text| self.user_word_edge(user, &code, start, text));
        words.chain(user_words).collect()
    }

    /// 沒有分隔記號的碼 `code` 的字、詞與使用者詞，見 [`Engine::exact`]。
    fn exact_code<'a>(
        &'a self,
        user: &'a SchemeModel,
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
                i < limit || user.has_learned(code, e.text.encode_utf8(&mut [0; 4]))
            })
            .map(move |(_, e)| self.char_edge(user, code, start, e));
        let words = self
            .lexicon
            .lookup(code)
            .iter()
            .map(move |w| self.word_edge(user, code, start, w));
        let user_words = user
            .words(code)
            .map(move |text| self.user_word_edge(user, code, start, text));
        chars.chain(words).chain(user_words)
    }

    /// 有萬用鍵的碼 `pattern`（已去掉分隔記號，`cuts` 是分隔記號的位置，見 [`Engine::exact`]）的
    /// 字、詞與使用者詞（brief §1.5、plan §2.5）：每個萬用鍵剛好代方案的一個鍵，對得上的實際的碼
    /// 由 [`Engine::matching_codes`] 列出。字跨各個碼依字頻取最高的 `limit` 個（同 [`by_frequency`]）
    /// 與學過的；同一個字詞對到幾個碼時只留第一個。學習計數與個人機率照實際的碼查。
    ///
    /// 單獨一個萬用鍵是任一個一碼的字；其餘一段的碼裡萬用鍵不能多於已知的碼，免得 `zz`、`azzz`
    /// 這類碼對上大半個碼表或詞庫、每鍵延遲倍增。兩種規則在 dev 切分上比過（M4 票 11 Comments）：
    /// 單獨的萬用鍵也不收時，一碼字換成 `z` 的題有一成多打不出，首碼、尾碼的題沒有分別。
    fn wildcard_edges(
        &self,
        user: &SchemeModel,
        pattern: &str,
        cuts: &[usize],
        start: usize,
        limit: usize,
    ) -> Vec<Edge> {
        let wildcards = pattern
            .chars()
            .filter(|&c| Some(c) == self.scheme.wildcard)
            .count();
        if pattern.len() > 1 && wildcards * 2 > pattern.len() {
            return Vec::new();
        }
        let codes = self.matching_codes(user, pattern);
        let mut seen: HashSet<String> = HashSet::new();
        let mut edges = Vec::new();
        if cuts.is_empty() {
            let mut chars: Vec<(&str, &CodeEntry)> = codes
                .iter()
                .flat_map(|code| {
                    self.char_entries(code)
                        .iter()
                        .filter(|e| !e.extension)
                        .map(move |e| (code.as_str(), e))
                })
                .collect();
            // 穩定排序：同字頻照碼的字典序、碼內的字序。
            chars.sort_by_key(|(_, e)| Reverse(self.lexicon.char_frequency(e.text)));
            let mut rank = 0;
            for (code, e) in chars {
                let text = e.text.to_string();
                if !seen.insert(text.clone()) {
                    continue;
                }
                if rank < limit || user.has_learned(code, &text) {
                    edges.push(self.char_edge(user, code, start, e));
                }
                rank += 1;
            }
        }
        for code in &codes {
            let fits = |text: &str| cuts.is_empty() || self.splits_at(text, code, cuts);
            for w in self.lexicon.lookup(code) {
                if fits(&w.text) && seen.insert(w.text.clone()) {
                    edges.push(self.word_edge(user, code, start, w));
                }
            }
            for text in user.words(code) {
                if fits(text) && seen.insert(text.to_string()) {
                    edges.push(self.user_word_edge(user, code, start, text));
                }
            }
        }
        edges
    }

    /// 有萬用鍵的碼 `pattern`（不含分隔記號）對得上的實際的碼，依字典序：萬用鍵的位置換成方案的
    /// 每個鍵，碼表、詞庫與使用者詞都沒有以它開頭的碼時就不往下展開。
    fn matching_codes(&self, user: &SchemeModel, pattern: &str) -> Vec<String> {
        let exists = |prefix: &str| {
            self.code_table.has_prefix(prefix)
                || self.lexicon.has_code_prefix(prefix)
                || user.has_word_code_prefix(prefix)
        };
        let mut codes = Vec::new();
        let mut todo = vec![String::new()];
        while let Some(prefix) = todo.pop() {
            let Some(next) = pattern[prefix.len()..].chars().next() else {
                codes.push(prefix);
                continue;
            };
            let keys: Vec<char> = if Some(next) == self.scheme.wildcard {
                // 反過來推進堆疊，取出時依字母表的順序。
                self.scheme.alphabet.chars().rev().collect()
            } else {
                vec![next]
            };
            for key in keys {
                let mut longer = prefix.clone();
                longer.push(key);
                if exists(&longer) {
                    todo.push(longer);
                }
            }
        }
        codes.sort();
        codes
    }

    /// 學習記的碼（plan §2.4）：輸入碼 `code`（已去掉分隔記號）有萬用鍵時換成 `text` 實際的碼——
    /// 碼表、詞庫或使用者詞裡對得上的第一個；都對不上時是 `None`。沒有萬用鍵時照原樣。
    pub(crate) fn learned_code(&self, user: &UserModel, code: &str, text: &str) -> Option<String> {
        if !self.scheme.has_wildcard(code) {
            return Some(code.to_string());
        }
        let user = user.for_scheme(&self.scheme.id);
        self.matching_codes(user, code)
            .into_iter()
            .find(|c| self.has_entry(c, text) || user.words(c).any(|t| t == text))
    }

    /// `text` 能不能一字一碼地拆開 `code`，使 `cuts` 的每個位置（去掉分隔記號後的碼裡）都是
    /// 兩字的碼之間：字的碼照碼表，一個字有幾個碼時任一種拆法都算。碼不是由字的碼組成的使用者詞
    /// （只有觸發詞的）拆不開，不跨分隔記號。
    fn splits_at(&self, text: &str, code: &str, cuts: &[usize]) -> bool {
        let chars: Vec<char> = text.chars().collect();
        let longest = usize::from(self.scheme.max_code_length);
        // 待走的（已對齊的字數, 已用掉的碼長）。
        let mut todo = vec![(0, 0)];
        let mut seen = HashSet::new();
        while let Some((aligned, used)) = todo.pop() {
            let Some(&c) = chars.get(aligned) else {
                if used == code.len() {
                    return true;
                }
                continue;
            };
            for end in used + 1..=(used + longest).min(code.len()) {
                let cut_inside = cuts.iter().any(|&cut| used < cut && cut < end);
                if !cut_inside
                    && self
                        .char_entries(&code[used..end])
                        .iter()
                        .any(|e| e.text == c)
                    && seen.insert((aligned + 1, end))
                {
                    todo.push((aligned + 1, end));
                }
            }
        }
        false
    }

    /// 結束在 `input` 結尾的 lattice 邊：每個起點的碼表字取字頻最高的 beam 個與學過的，
    /// 加上詞庫的詞與使用者詞。擴展區的字不進 lattice（plan §2.3）。邊不從分隔記號開始；
    /// 往回找的上限算的是字母數，分隔記號不算。
    pub(crate) fn edges_ending(&self, user: &UserModel, input: &str) -> Vec<Edge> {
        let user = user.for_scheme(&self.scheme.id);
        let longest = self
            .longest_word_code
            .max(usize::from(self.scheme.max_code_length))
            .max(user.longest_word_code());
        let mut starts: Vec<usize> = input
            .char_indices()
            .rev()
            .filter(|&(_, c)| c != SEPARATOR)
            .take(longest)
            .map(|(i, _)| i)
            .collect();
        // 同分時照邊的順序決定名次（[`Lattice::push`]），起點由前到後。
        starts.reverse();
        starts
            .into_iter()
            .flat_map(|start| self.exact(user, &input[start..], start, self.beam()))
            .collect()
    }

    /// 輸入碼 `input` 的全部連打字集的字與詞與它們接在 `before` 後面的分數，依分數由高到低
    /// （同分照碼表、詞庫的順序）；`input` 中間的分隔記號見 [`Engine::exact`]。
    pub(crate) fn span(&self, user: &UserModel, before: &Preceding, input: &str) -> Vec<Spanned> {
        let following = before.word.as_deref().and_then(|w| user.bigrams_after(w));
        let history: WordHistory = [
            None,
            before.segment().and_then(|segment| self.word_id(&segment)),
        ];
        let scheme_user = user.for_scheme(&self.scheme.id);
        let mut edges: Vec<Spanned> = self
            .exact(scheme_user, input, 0, usize::MAX)
            .into_iter()
            .map(|e| Spanned {
                score: self.edge_score(
                    before.character,
                    history,
                    &e,
                    following.map_or(0.0, |f| f.probability(&e.text)),
                ),
                simplified: e.simplified,
                text: e.text,
            })
            .collect();
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

/// 每個分隔記號在去掉分隔記號後的碼裡的位置：`vi'vd'jg` 是 2、4。
fn separator_cuts(input: &str) -> Vec<usize> {
    input
        .chars()
        .enumerate()
        .filter(|&(_, c)| c == SEPARATOR)
        .enumerate()
        .map(|(seen, (at, _))| at - seen)
        .collect()
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
