//! 聯想（plan〈介面〉〈排序：同一把尺〉，M3 票 13）：上屏之後給接下來可能要打的字詞。
//!
//! 來源有四種，用連打的同一把尺排（plan〈排序：同一把尺〉）：分數是接上這個聯想的路徑比不接時多出多少。
//!
//! - 補完詞庫的詞：剛上屏的最後幾段接起來是詞庫裡某個更長的詞的開頭（上屏「門」，詞庫有「門口」，
//!   給「口」）。分數是那幾段換成整個詞的路徑分數減原本那幾段的：靜態機率照 lattice 的邊算，
//!   加上段數與詞庫的段的差（β、γ）。只從分段的邊界補，不拆開剛上屏的詞。
//! - 字級 bigram：留下的 bigram 裡接在最後一字後面的字，當新的一段。
//! - 以這些字開頭的詞庫的詞，當新的一段。
//! - 個人 bigram：剛上屏的最後一段後面學過的字詞，當新的一段。
//!
//! 新的一段的分數和連打裡的邊相同：首字 bigram 加詞內機率（詞庫的詞用詞頻換算、其他一字接一字用
//! 字級 bigram），詞庫的詞加 β，都扣 γ。同一串字有幾種算法時取高的；每個候選都加個人 bigram 的
//! `personal`（沒學過是 0）。

use std::collections::HashMap;

use crate::user::personal_bonus;
use crate::{Engine, PAGE_SIZE, UserModel};

/// 聯想一次最多給幾個：一頁，所以不必翻頁（librime-predict 的 `max_candidates` 也建議設成一頁）。
pub(crate) const ASSOCIATION_LIMIT: usize = PAGE_SIZE;

/// 補完詞庫的詞時最多看剛上屏的最後幾個字。
pub(crate) const ASSOCIATION_TAIL: usize = 8;

/// 一個聯想的分數：排序用的分數，與 ln 靜態機率的差（不含 β、γ，`personal` 要的，同 lattice 的邊）。
#[derive(Debug, Clone, Copy)]
struct Scored {
    score: f64,
    log_static: f64,
}

/// 接在 `segments`（剛上屏的各段，繁體原文，依上屏的順序）後面的聯想，好的在前，最多
/// [`ASSOCIATION_LIMIT`] 個。最後一段是個人 bigram 的前一個字詞。
pub(crate) fn associations(engine: &Engine, user: &UserModel, segments: &[String]) -> Vec<String> {
    let Some(last) = segments.last().and_then(|s| s.chars().next_back()) else {
        return Vec::new();
    };
    let ranking = &engine.scheme().ranking;
    let (beta, gamma) = (ranking.word_bonus, ranking.segment_penalty);
    let mut found: HashMap<String, Scored> = HashMap::new();
    let mut offer = |text: &str, scored: Scored| {
        if text.is_empty() {
            return;
        }
        found
            .entry(text.to_string())
            .and_modify(|best| {
                if scored.score > best.score {
                    *best = scored;
                }
            })
            .or_insert(scored);
    };

    // 補完詞庫的詞：最後 k 段接起來是 `piece`，換成以它開頭的詞。
    for k in 1..=segments.len() {
        let covered = &segments[segments.len() - k..];
        let piece: String = covered.concat();
        if piece.chars().count() > ASSOCIATION_TAIL {
            break;
        }
        let before = segments[..segments.len() - k]
            .last()
            .and_then(|s| s.chars().next_back());
        let mut prev = before;
        let mut log_covered = 0.0;
        let mut structure = beta - gamma;
        for segment in covered {
            let (log_static, is_word) = segment_static(engine, prev, segment);
            log_covered += log_static;
            structure -= if is_word { beta } else { 0.0 } - gamma;
            prev = segment.chars().next_back();
        }
        for (word, _) in top_words(engine, &piece) {
            let (log_word, _) = segment_static(engine, before, word);
            let log_static = log_word - log_covered;
            offer(
                &word[piece.len()..],
                Scored {
                    score: log_static + structure,
                    log_static,
                },
            );
        }
    }

    // 字級 bigram 接得上的字，與以其中最常接的字開頭的詞。
    let mut next: Vec<(char, f64)> = engine
        .language_model()
        .following(last)
        .filter(|&(c, _)| engine.is_continuous(c))
        .collect();
    next.sort_by(|a, b| b.1.total_cmp(&a.1));
    for (i, &(c, _)) in next.iter().enumerate() {
        let text = c.to_string();
        offer(&text, score_as_new_segment(engine, last, &text));
        if i < ASSOCIATION_LIMIT {
            for (word, _) in top_words(engine, &text) {
                offer(word, score_as_new_segment(engine, last, word));
            }
        }
    }

    // 個人 bigram 學過的。
    let following = segments.last().and_then(|w| user.bigrams_after(w));
    if let Some(following) = following {
        for (text, _) in following.iter() {
            offer(text, score_as_new_segment(engine, last, text));
        }
    }

    let mut ranked: Vec<(String, f64)> = found
        .into_iter()
        .map(|(text, s)| {
            let bigram = following.map_or(0.0, |f| f.probability(&text));
            let score = s.score + personal_bonus(s.log_static, 0.0, bigram);
            (text, score)
        })
        .collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    ranked
        .into_iter()
        .take(ASSOCIATION_LIMIT)
        .map(|(text, _)| text)
        .collect()
}

/// 一段 `text` 接在 `prev` 後面的 ln 靜態機率（同 lattice 的邊，plan〈排序：同一把尺〉）與它是不是詞庫的詞：
/// 詞庫的詞是首字 bigram 加詞頻換算的詞內機率；其他（單字、學過的字詞）詞內一字接一字用字級 bigram。
fn segment_static(engine: &Engine, prev: Option<char>, text: &str) -> (f64, bool) {
    let lm = engine.language_model();
    let mut chars = text.chars();
    let first = chars.next().expect("分段不是空的");
    let log_first = lm.log_prob(prev, first);
    let word = engine
        .word_log_prob(text)
        .filter(|_| text.chars().nth(1).is_some());
    match word {
        Some(log_prob) => (log_first + log_prob - lm.log_prob(None, first), true),
        None => {
            let (log_static, _) = chars.fold((log_first, first), |(sum, prev), c| {
                (sum + lm.log_prob(Some(prev), c), c)
            });
            (log_static, false)
        }
    }
}

/// `text` 當新的一段接在 `last` 後面的分數：靜態機率，詞庫的詞加 β，扣 γ。
fn score_as_new_segment(engine: &Engine, last: char, text: &str) -> Scored {
    let ranking = &engine.scheme().ranking;
    let (log_static, is_word) = segment_static(engine, Some(last), text);
    let bonus = if is_word { ranking.word_bonus } else { 0.0 };
    Scored {
        score: log_static + bonus - ranking.segment_penalty,
        log_static,
    }
}

/// 詞庫裡以 `prefix` 開頭的詞，詞頻最高的 [`ASSOCIATION_LIMIT`] 個：再多也排不進一頁。
fn top_words<'a>(engine: &'a Engine, prefix: &'a str) -> Vec<(&'a str, f64)> {
    let mut words: Vec<(&str, f64)> = engine.words_starting_with(prefix).collect();
    words.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    words.truncate(ASSOCIATION_LIMIT);
    words
}
