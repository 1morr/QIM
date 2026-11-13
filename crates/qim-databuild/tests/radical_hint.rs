//! 字根提示資料包的內容（plan〈速成碼表〉 步驟 5）：字 → 倉頡三代 / 五代碼。

use qim_core::{CangjieCodes, CangjieVersion};
use qim_databuild::{libcangjie, radical_hint};

/// 「乍」分在兩列（第二列只有三代碼）；「°」不是漢字。
const TABLE: &str = "\
乍 乍 1 1 0 0 1 0 0 0 0 hs os NA 0
乍 乍 1 1 0 0 1 0 0 0 0 os NA NA 0
丟 丢 1 1 0 0 1 0 0 0 0 hgi mgi,xxmgi NA 22813
° NA 0 0 0 0 0 0 0 0 1 yyycg yyycg NA 0
乃 乃 1 0 0 0 0 0 0 0 0 nsh NA NA 0
";

fn codes(cangjie3: &[&str], cangjie5: &[&str]) -> CangjieCodes {
    let owned = |codes: &[&str]| codes.iter().map(|c| c.to_string()).collect();
    CangjieCodes {
        cangjie3: owned(cangjie3),
        cangjie5: owned(cangjie5),
    }
}

#[test]
fn each_han_character_gets_its_codes_of_every_row_in_source_order() {
    let hints = radical_hint::hints(&libcangjie::parse(TABLE).unwrap());
    let all: Vec<(char, CangjieCodes)> = hints.iter().map(|(c, k)| (c, k.clone())).collect();
    assert_eq!(
        all,
        [
            ('丟', codes(&["hgi"], &["mgi", "xxmgi"])),
            ('乃', codes(&["nsh"], &[])),
            ('乍', codes(&["hs", "os"], &["os"])),
        ]
    );
    assert_eq!(hints.codes('乍', CangjieVersion::Cangjie5), ["os"]);
}
