//! OpenCC 字典的讀法、反轉與 `hk2s` 的組法（plan §4.4）。

use qim_databuild::opencc::{self, Dictionary};

fn entries(d: &Dictionary) -> Vec<(&str, Vec<&str>)> {
    d.entries
        .iter()
        .map(|(k, v)| (k.as_str(), v.iter().map(String::as_str).collect()))
        .collect()
}

#[test]
fn comments_and_blank_lines_are_skipped() {
    let text = "# File: X\n\n乾\t干 乾\n# @tofu-risk: rare\n㑮\t㑮 𫝈\n";
    let d = opencc::parse("X", text).unwrap();
    assert_eq!(
        entries(&d),
        [("乾", vec!["干", "乾"]), ("㑮", vec!["㑮", "𫝈"])]
    );
}

/// 只取第一個值：`@tofu-risk` 的罕用簡體字是第二個值，保留原字。
#[test]
fn a_table_takes_the_first_value() {
    let d = opencc::parse("X", "㑮\t㑮 𫝈\n乾\t干 乾\n").unwrap();
    let s = qim_core::Simplifier::new(
        vec![],
        opencc::table(&Dictionary {
            entries: vec![],
            reverse_preferences: Default::default(),
        }),
        vec![vec![opencc::table(&d)]],
    );
    assert_eq!(s.convert("㑮乾"), "㑮干");
}

#[test]
fn malformed_lines_fail() {
    assert!(opencc::parse("X", "乾 干\n").is_err(), "少了 TAB");
    assert!(opencc::parse("X", "乾\t\n").is_err(), "值是空的");
    assert!(opencc::parse("X", "乾\t干\n乾\t乾\n").is_err(), "鍵重複");
    assert!(opencc::parse("X", "乾\t干\n").is_ok());
}

/// 反轉照 OpenCC `Dict.swap`：值 → 依檔案順序的鍵，`@reverse-prefer` 的鍵移到第一個。
#[test]
fn reversing_honours_preferences() {
    let text = "纔\t才\n# @reverse-prefer: 才\n才\t才\n僞\t偽\n嫺\t嫻 嫺\n";
    let d = opencc::parse("HKVariants.txt", text).unwrap();
    let r = opencc::reverse("HKVariants.txt", &d).unwrap();
    assert_eq!(
        entries(&r),
        [
            ("才", vec!["才", "纔"]),
            ("偽", vec!["僞"]),
            ("嫻", vec!["嫺"]),
            ("嫺", vec!["嫺"]),
        ]
    );
}

#[test]
fn a_preference_without_a_mapping_fails() {
    let d = opencc::parse("X", "# @reverse-prefer: 才 纔\n才\t才\n").unwrap();
    assert!(opencc::reverse("X", &d).is_err());
}

/// `hk2s`：香港異體字先轉回標準繁體（「綫」→「線」），再轉簡體；分段的詞保住「乾隆」。
#[test]
fn hk2s_chains_the_dictionaries() {
    let read = |file: &str| -> Result<String, qim_databuild::Error> {
        Ok(match file {
            opencc::COMPATIBILITY => "\u{F900}\t豈\n",
            opencc::TS_PHRASES => "乾隆\t乾隆\n",
            opencc::TS_CHARACTERS => "門\t门\n線\t线\n乾\t干 乾\n豈\t岂\n",
            opencc::HK_VARIANTS => "線\t綫\n",
            opencc::HK_VARIANTS_REV_PHRASES => "",
            other => panic!("沒有 {other}"),
        }
        .to_string())
    };
    let s = opencc::hk2s(read).unwrap();
    assert_eq!(s.convert("門口綫乾乾隆\u{F900}"), "门口线干乾隆岂");
}
