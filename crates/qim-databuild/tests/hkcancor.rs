use std::collections::BTreeMap;

use qim_databuild::corpus::{Split, split_of};
use qim_databuild::hkcancor::{Tagged, TrainWords, tagged_utterances, utterances};
use qim_databuild::libcangjie;
use qim_databuild::transcode::Transcoder;

/// HKCanCor `utf8/FC-107_v2` 的開頭（fcbond/hkcancor `39aeadf`）。
const TRANSCRIPT: &str = "<info>
\t1-TN-107
\tINFO-END
</info>
\t<sent>
\t\t<sent_head>
\t\t\tA:
\t\t</sent_head>
\t\t<sent_tag>
\t\t\t陣間/t/zan6gaan1/
\t\t\t放學/v/fong3hok6/
\t\t\t去/v/heoi3/
\t\t\t邊/r/bin1/
\t\t\t啊/y/aa3/
\t\t\t？/w/VQ6/
\t\t</sent_tag>
\t\t<sent_tran>

\t\t</sent_tran>
\t</sent>
\t<sent>
\t\t<sent_head>
\t\t\tB:
\t\t</sent_head>
\t\t<sent_tag>
\t\t\t冇/v1/mou5/
\t\t\t啊/y1/aa3/
\t\t\t，/w/VQ2/
\t\t\tHong Kong U/n/hoeng1gong2ju1/
\t\t\t-/w/VQ3/
\t\t</sent_tag>
\t</sent>
";

#[test]
fn each_sentence_becomes_one_utterance_of_its_words() {
    assert_eq!(
        utterances(TRANSCRIPT),
        ["陣間放學去邊啊？", "冇啊，Hong Kong U-"]
    );
}

#[test]
fn transcript_without_sentences_has_no_utterances() {
    assert!(utterances("<info>\n</info>\n").is_empty());
}

#[test]
fn tagged_utterances_keep_each_word_and_its_tag() {
    let first = &tagged_utterances(TRANSCRIPT)[0];
    assert_eq!(
        first[0],
        Tagged {
            text: "陣間",
            tag: "t"
        }
    );
    assert_eq!(
        first[5],
        Tagged {
            text: "？",
            tag: "w"
        }
    );
    assert_eq!(tagged_utterances(TRANSCRIPT)[1][3].text, "Hong Kong U");
}

/// 甲乙丙丁戊己庚辛在連打字集，𠀀不在。碼是假的，只要轉得出碼。
const TABLE: &str = "\
甲 甲 1 1 0 0 1 0 0 0 0 a a NA 1
乙 乙 1 1 0 0 1 0 0 0 0 b b NA 2
丙 丙 1 1 0 0 1 0 0 0 0 c c NA 3
丁 丁 1 1 0 0 1 0 0 0 0 d d NA 4
戊 戊 1 1 0 0 1 0 0 0 0 e e NA 5
己 己 1 1 0 0 1 0 0 0 0 f f NA 6
庚 庚 1 1 0 0 1 0 0 0 0 g g NA 7
辛 辛 1 1 0 0 1 0 0 0 0 h h NA 8
𠀀 𠀀 1 0 0 0 0 0 0 0 0 a NA NA 0
";

fn transcoder() -> Transcoder {
    Transcoder::new(&libcangjie::parse(TABLE).unwrap())
}

/// 由兩個不同的兩字詞組成、落在 `split` 的句子，依字序列出。
fn clauses_in(split: Split) -> Vec<(String, String)> {
    let chars: Vec<char> = "甲乙丙丁戊己庚辛".chars().collect();
    let mut found = Vec::new();
    for a in &chars {
        for b in &chars {
            for c in &chars {
                for d in &chars {
                    let (x, y) = (format!("{a}{b}"), format!("{c}{d}"));
                    if x != y && split_of(&format!("{x}{y}")) == split {
                        found.push((x, y));
                    }
                }
            }
        }
    }
    found
}

fn clause_in(split: Split, nth: usize) -> (String, String) {
    clauses_in(split).swap_remove(nth)
}

fn word<'a>(text: &'a str, tag: &'a str) -> Tagged<'a> {
    Tagged { text, tag }
}

fn counted(sentences: &[Vec<Tagged>]) -> BTreeMap<String, u64> {
    let transcoder = transcoder();
    let mut words = TrainWords::new(&|s: &str| s.to_string(), &transcoder);
    for s in sentences {
        words.add(s);
    }
    words.counts.words
}

#[test]
fn only_words_of_training_clauses_are_counted() {
    let (x, y) = clause_in(Split::Train, 0);
    let (tx, ty) = clause_in(Split::Test, 0);
    let (dx, dy) = clause_in(Split::Dev, 0);
    let words = counted(&[
        vec![
            word(&x, "v"),
            word(&y, "n"),
            word("，", "w"),
            word(&tx, "v"),
            word(&ty, "n"),
        ],
        vec![word(&x, "v"), word(&y, "n")],
        vec![word(&dx, "v"), word(&dy, "d")],
    ]);
    assert_eq!(words, BTreeMap::from([(x, 2), (y, 2)]));
}

#[test]
fn names_single_characters_and_clauses_outside_the_continuous_set_are_left_out() {
    let (x, y) = clause_in(Split::Train, 1);
    let first: String = y.chars().take(1).collect();
    let second: String = y.chars().skip(1).collect();
    let words = counted(&[
        // 人名（nr）不收；單字詞不收，但仍是句子的一部分。
        vec![word(&x, "nr"), word(&first, "v"), word(&second, "y")],
        // 含連打字集以外的字：整句不進 train，詞也不數。
        vec![word(&x, "v"), word("𠀀甲", "n")],
    ]);
    assert!(words.is_empty(), "{words:?}");
}

#[test]
fn words_are_counted_in_converted_form_and_unaligned_sentences_are_skipped() {
    let (x, y) = clauses_in(Split::Train)
        .into_iter()
        .find(|(x, y)| x.starts_with('甲') && !y.contains('甲'))
        .unwrap();
    let transcoder = transcoder();
    // 轉換換字不改字數：詞照轉換後的寫法數，切分也看轉換後的句子。
    let raw_x = x.replace('甲', "𠀀");
    let convert = |s: &str| s.replace('𠀀', "甲");
    let mut words = TrainWords::new(&convert, &transcoder);
    words.add(&[word(&raw_x, "v"), word(&y, "n")]);
    // 轉換改了字數：對不上詞的位置，整句不數。
    let lengthen = |s: &str| s.replacen(&y, &format!("{y}乙"), 1);
    let mut skipped = TrainWords::new(&lengthen, &transcoder);
    skipped.add(&[word(&x, "v"), word(&y, "n")]);
    assert!(
        words.counts.words.contains_key(&x),
        "{:?}",
        words.counts.words
    );
    assert!(skipped.counts.words.is_empty());
    assert_eq!(skipped.counts.unaligned, 1);
}
