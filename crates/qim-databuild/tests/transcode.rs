use qim_databuild::libcangjie;
use qim_databuild::transcode::Transcoder;

/// 日、問：一個碼。丟：三代 `hgi`、五代 `mgi,xxmgi`，速成 `hi`、`mi`、`xi`。
/// 乍：兩列，第一列五代 `os`。匠：只有三代碼。𠀀：擴展區。嘅：只有 HKSCS 旗標。
const TABLE: &str = "\
日 日 1 1 0 0 1 0 0 0 0 a a NA 23061
問 问 1 1 0 0 1 0 0 0 0 anr anr NA 20001
丟 丢 1 1 0 0 1 0 0 0 0 hgi mgi,xxmgi NA 22813
乍 乍 1 1 0 0 1 0 0 0 0 hs os NA 22969
乍 乍 1 1 0 0 1 0 0 0 0 os NA NA 0
匠 匠 1 1 0 0 1 0 0 0 0 shml NA NA 9000
嘅 嘅 1 0 1 0 1 0 0 0 0 raiu raiu NA 0
𠀀 𠀀 1 0 0 0 0 0 0 0 0 a NA NA 0
° NA 0 1 0 0 0 0 0 0 1 yyycg yyycg NA 0
";

fn transcoder() -> Transcoder {
    Transcoder::new(&libcangjie::parse(TABLE).unwrap())
}

#[test]
fn character_with_one_quick_code_gets_it() {
    let t = transcoder();
    assert_eq!(t.code('日'), Some("a"));
    assert_eq!(t.code('問'), Some("ar"));
    assert_eq!(t.code('嘅'), Some("ru"));
}

#[test]
fn character_with_several_quick_codes_gets_the_first_cangjie5_code() {
    let t = transcoder();
    assert_eq!(t.code('丟'), Some("mi"));
    assert_eq!(t.code('乍'), Some("os"));
}

#[test]
fn character_without_cangjie5_code_falls_back_to_cangjie3() {
    assert_eq!(transcoder().code('匠'), Some("sl"));
}

#[test]
fn text_becomes_one_code_per_character() {
    assert_eq!(transcoder().transcode("問日丟"), Ok(vec!["ar", "a", "mi"]));
}

#[test]
fn character_outside_the_continuous_set_fails() {
    let t = transcoder();
    assert_eq!(t.code('𠀀'), None);
    assert_eq!(t.transcode("日𠀀問"), Err('𠀀'));
    // 不是漢字的列不算，即使有 Big5 旗標。
    assert_eq!(t.transcode("°"), Err('°'));
    assert_eq!(t.transcode("日a"), Err('a'));
}
