use std::io::Write;

use flate2::Compression;
use flate2::write::GzEncoder;
use qim_databuild::english;

/// 照 AOSP LatinIME `dictionaries/en_US_wordlist.combined.gz` 的寫法：檔頭一行，每個詞一行
/// ` word=…,f=…`，其下兩個空白開頭的是那個詞的 bigram 與 shortcut。
const HEADER: &str =
    "dictionary=main:en_us,locale=en_US,description=English (US),date=1414726260,version=54\n";

fn gz(text: &str) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(text.as_bytes()).unwrap();
    encoder.finish().unwrap()
}

/// `lines` 逐行接在檔頭後面。
fn build(lines: &[&str]) -> (qim_core::EnglishWords, english::Counts) {
    english::words(&gz(&format!("{HEADER}{}\n", lines.join("\n")))).unwrap()
}

#[test]
fn each_word_line_becomes_a_word_with_its_frequency_and_bigrams_are_skipped() {
    let (words, counts) = build(&[
        " word=the,f=222,flags=,originalFreq=222",
        "  bigram=new,f=1",
        " word=receive,f=137,flags=,originalFreq=137",
    ]);
    assert_eq!(words.lookup("the"), Some("the"));
    assert_eq!(words.frequency("the"), Some(222));
    assert_eq!(words.frequency("receive"), Some(137));
    assert_eq!(words.lookup("new"), None, "bigram 的詞不是詞表的詞");
    assert_eq!(counts.entries, 2);
    assert_eq!(counts.words, 2);
}

/// 英文模式也用這份詞表（ADR 0008），`z` 開頭、撇號、連字號的詞都收；`z` 只在中文組字比對時濾掉。
#[test]
fn words_with_z_apostrophes_and_hyphens_are_kept() {
    let (words, _) = build(&[
        " word=zero,f=126,flags=,originalFreq=126",
        " word=ain't,f=88,flags=nonword,originalFreq=88",
        " word=e-mail,f=90,flags=,originalFreq=90",
    ]);
    for w in ["zero", "ain't", "e-mail"] {
        assert_eq!(words.lookup(w), Some(w));
    }
}

/// LatinIME 不建議的詞不收：可能冒犯的、只當 shortcut 用的錯拼（`not_a_word`）、`f=0`。
#[test]
fn offensive_not_a_word_and_zero_frequency_entries_are_dropped() {
    let (words, counts) = build(&[
        " word=badword,f=120,flags=,originalFreq=120,possibly_offensive=true",
        " word=rude,f=100,flags=offensive,originalFreq=100",
        " word=acomodate,f=0,not_a_word=true",
        "  shortcut=accommodate,f=whitelist",
        " word=DOS,f=0,flags=,originalFreq=110",
        " word=nave,f=108,flags=,originalFreq=108,whitelist=have",
    ]);
    for w in ["badword", "rude", "acomodate", "dos", "accommodate"] {
        assert_eq!(words.lookup(w), None, "{w}");
    }
    assert_eq!(
        words.lookup("nave"),
        Some("nave"),
        "whitelist 是自動修正的對應，詞照收"
    );
    assert_eq!(counts.offensive, 2);
    assert_eq!(counts.not_a_word, 1);
    assert_eq!(counts.zero_frequency, 1);
    assert_eq!(counts.words, 1);
}

#[test]
fn spellings_merge_by_lower_case_keeping_the_lower_case_one_and_its_frequency() {
    let (words, counts) = build(&[
        " word=US,f=150,flags=abbreviation,originalFreq=150",
        " word=us,f=170,flags=,originalFreq=170",
        " word=iPad,f=86,flags=,originalFreq=86",
    ]);
    assert_eq!(words.lookup("us"), Some("us"));
    assert_eq!(words.frequency("us"), Some(170));
    assert_eq!(words.lookup("ipad"), Some("iPad"));
    assert_eq!(counts.entries, 3);
    assert_eq!(counts.words, 2);
}

#[test]
fn a_file_without_the_header_or_with_a_bad_frequency_is_an_error() {
    assert!(english::words(&gz(" word=the,f=222\n")).is_err());
    assert!(english::words(&gz(&format!("{HEADER} word=the,f=lots\n"))).is_err());
    assert!(english::words(&gz(&format!("{HEADER} word=the,f=256\n"))).is_err());
    assert!(english::words(&gz(&format!("{HEADER} f=1,word=\n"))).is_err());
    assert!(english::words(b"not gzip").is_err());
}
