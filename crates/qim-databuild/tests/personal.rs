//! 個人集：Rime 的 `.userdb.txt`（`docs/research/facts-rime.md`）轉成基準的題目（plan〈首選字基準〉）。

use qim_databuild::libcangjie;
use qim_databuild::personal::{Counts, from_userdb};
use qim_databuild::transcode::Transcoder;

/// 日 `a`、問 `ar`、匠 `sl`；𠀀 在擴展區，不在連打字集。
const TABLE: &str = "\
日 日 1 1 0 0 1 0 0 0 0 a a NA 23061
問 问 1 1 0 0 1 0 0 0 0 anr anr NA 20001
匠 匠 1 1 0 0 1 0 0 0 0 shml NA NA 9000
𠀀 𠀀 1 0 0 0 0 0 0 0 0 a NA NA 0
";

fn transcoder() -> Transcoder {
    Transcoder::new(&libcangjie::parse(TABLE).unwrap())
}

/// 檔頭照 librime `user_db.cc` 的寫法；輸入碼是倉頡全碼，每個音節後一個空白。
const HEADER: &str = "\
# Rime user dictionary
#@/db_name\tquick5.userdb
#@/db_type\tuserdb
#@/rime_version\t1.13.1
#@/tick\t42
#@/user_id\tsome-id
";

#[test]
fn words_used_at_least_once_get_quick_codes_from_the_transcoder() {
    let text = format!("{HEADER}a anr \t日問\tc=3 d=0.9 t=10\nshml \t匠\tc=1 d=1 t=11\n");
    let (tsv, counts) = from_userdb(&text, &transcoder()).unwrap();
    // 不用 Rime 記下的倉頡全碼，碼由轉碼器從詞轉出。
    assert_eq!(tsv, "a ar\t日問\t3\nsl\t匠\t1\n");
    assert_eq!(
        counts,
        Counts {
            words: 2,
            unused: 0,
            untranscodable: 0,
            merged: 0,
        }
    );
}

#[test]
fn unused_deleted_and_untranscodable_words_are_dropped_and_counted() {
    let text = format!(
        "{HEADER}a \t日\tc=0 d=0.1 t=1\nanr \t問\tc=-2 d=0 t=2\na \t𠀀\tc=4 d=1 t=3\nshml \t匠\tc=2 d=1 t=4\n"
    );
    let (tsv, counts) = from_userdb(&text, &transcoder()).unwrap();
    assert_eq!(tsv, "sl\t匠\t2\n");
    assert_eq!(
        counts,
        Counts {
            words: 1,
            unused: 2,
            untranscodable: 1,
            merged: 0,
        }
    );
}

#[test]
fn the_same_word_under_two_codes_becomes_one_item_with_summed_counts() {
    let text =
        format!("{HEADER}anr \t問\tc=2 d=1 t=1\na \t日\tc=1 d=1 t=2\nanb \t問\tc=5 d=1 t=3\n");
    let (tsv, counts) = from_userdb(&text, &transcoder()).unwrap();
    assert_eq!(tsv, "ar\t問\t7\na\t日\t1\n");
    assert_eq!(counts.merged, 1);
}

#[test]
fn malformed_lines_name_the_line_but_not_the_content() {
    for (line, what) in [
        ("a \t日問\n", "欄"),
        ("a \t日問\td=1 t=1\n", "c="),
        ("a \t日問\tc=x d=1 t=1\n", "c="),
    ] {
        let error = from_userdb(&format!("{HEADER}{line}"), &transcoder()).unwrap_err();
        assert!(error.contains("第 7 行"), "{error}");
        assert!(error.contains(what), "{error}");
        assert!(
            !error.contains("日問"),
            "個人集的內容不能出現在訊息裡：{error}"
        );
    }
}
