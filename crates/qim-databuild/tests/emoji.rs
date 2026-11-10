//! CLDR 標註建 emoji 對照表（M3 票 16，plan §4.4）。

use qim_databuild::emoji::{self, Annotations};

/// 摘自 CLDR `release-49` 的 `common/annotations/yue.xml`。
const YUE: &str = r#"<?xml version="1.0" encoding="UTF-8" ?>
<!DOCTYPE ldml SYSTEM "../../common/dtd/ldml.dtd">
<ldml>
	<identity>
		<version number="$Revision$"/>
		<language type="yue"/>
	</identity>
	<annotations>
		<annotation cp="😂">快樂 | 有開心眼淚嘅臉 | 眼淚 | 笑 | 臉</annotation>
		<annotation cp="😂" type="tts">有開心眼淚嘅臉</annotation>
		<annotation cp="🐱">寵物 | 臉 | 貓 | 貓臉</annotation>
		<annotation cp="🐱" type="tts">貓臉</annotation>
		<annotation cp="🐈">貓 | 寵物</annotation>
		<annotation cp="🐈" type="tts">貓</annotation>
		<annotation cp="❤">心 | 紅心</annotation>
		<annotation cp="❤" type="tts">紅心</annotation>
		<annotation cp="🙏">多謝 | 拜託</annotation>
		<annotation cp="🙏" type="tts">合十</annotation>
		<annotation cp="🈶">有 | 日文「有」按鈕</annotation>
		<annotation cp="🈶" type="tts">日文「有」按鈕</annotation>
		<annotation cp="㊗">祝 | 恭喜</annotation>
		<annotation cp="㊗" type="tts">日文「祝賀」按鈕</annotation>
	</annotations>
</ldml>
"#;

/// 摘自 CLDR `release-49` 的 `common/annotations/zh_Hant.xml`。
const ZH_HANT: &str = r#"<?xml version="1.0" encoding="UTF-8" ?>
<ldml>
	<annotations>
		<annotation cp="🤣">在地上打滾 | 笑 | 大笑</annotation>
		<annotation cp="🤣" type="tts">笑到在地上打滾</annotation>
		<annotation cp="🐱">動物 | 貓 | 貓頭</annotation>
		<annotation cp="🐱" type="tts">貓頭</annotation>
		<annotation cp="🪿">鵝 | goose</annotation>
		<annotation cp="🪿" type="tts">鵝</annotation>
	</annotations>
</ldml>
"#;

/// 摘自 Unicode 18.0 的 `emoji-test.txt`。
const EMOJI_TEST: &str = "\
# group: Smileys & Emotion
1F602                                                  ; fully-qualified     # 😂 E0.6 face with tears of joy
2764 FE0F                                              ; fully-qualified     # ❤️ E0.6 red heart
2764                                                   ; unqualified         # ❤ E0.6 red heart
";

fn build() -> emoji::Built {
    build_with_limit(usize::MAX)
}

/// 關鍵字最多對到 `limit` 個 emoji 才收。
fn build_with_limit(limit: usize) -> emoji::Built {
    emoji::table(
        &[
            Annotations {
                name: "yue",
                xml: YUE,
            },
            Annotations {
                name: "zh_Hant",
                xml: ZH_HANT,
            },
        ],
        EMOJI_TEST,
        &|word| word.to_string(),
        // 管線裡是「有速成碼」：英文字母與這裡假裝沒有碼的「鵝」打不出。
        &|c| !c.is_ascii() && c != '鵝',
        limit,
    )
    .unwrap()
}

#[test]
fn a_word_lists_the_emoji_named_by_it_first_then_those_with_it_as_a_keyword() {
    // 「貓」是 🐈 的短名，排第一；接著是關鍵字有「貓」的，粵語在繁中之前、各依檔內順序。
    let built = build();
    assert_eq!(built.table.lookup("貓"), ["🐈", "🐱"]);
    assert_eq!(built.table.lookup("笑"), ["😂", "🤣"]);
    assert_eq!(built.table.lookup("多謝"), ["🙏"]);
}

#[test]
fn an_emoji_gets_its_fully_qualified_form_back() {
    // CLDR 的 cp 去掉了 U+FE0F；沒有它 ❤ 會顯示成文字。
    assert_eq!(build().table.lookup("心"), ["\u{2764}\u{FE0F}"]);
}

#[test]
fn a_word_with_an_untypable_character_is_dropped() {
    let built = build();
    assert!(built.table.lookup("鵝").is_empty());
    assert_eq!(built.counts.untypable, 2, "「鵝」與「goose」");
    assert!(built.table.lookup("goose").is_empty());
}

#[test]
fn words_are_written_the_way_the_lexicon_writes_them() {
    let built = emoji::table(
        &[Annotations {
            name: "yue",
            xml: YUE,
        }],
        EMOJI_TEST,
        &|word| word.replace('謝', "谢"),
        &|_| true,
        usize::MAX,
    )
    .unwrap();
    assert_eq!(built.table.lookup("多谢"), ["🙏"]);
    assert!(built.table.lookup("多謝").is_empty());
}

#[test]
fn counts_report_annotations_words_and_emoji() {
    let counts = build().counts;
    assert_eq!(counts.annotations, 20);
    assert_eq!(counts.emoji, 6);
}

#[test]
fn malformed_xml_is_an_error_naming_the_file() {
    let error = emoji::table(
        &[Annotations {
            name: "yue",
            xml: "<ldml><annotation>沒有 cp</annotation></ldml>",
        }],
        EMOJI_TEST,
        &|w| w.to_string(),
        &|_| true,
        usize::MAX,
    )
    .unwrap_err();
    assert!(error.to_string().contains("yue"), "{error}");
}

#[test]
fn a_keyword_shared_by_more_emoji_than_the_limit_is_dropped_but_short_names_stay() {
    // 門檻 1：「臉」（😂、🐱）、「笑」（😂、🤣）、「貓」的關鍵字（🐱、🐈）都太泛，不收；
    // 「貓」仍有短名是它的 🐈，「多謝」只對到 🙏。
    let built = build_with_limit(1);
    assert!(built.table.lookup("臉").is_empty());
    assert!(built.table.lookup("笑").is_empty());
    assert_eq!(built.table.lookup("貓"), ["🐈"]);
    assert_eq!(built.table.lookup("多謝"), ["🙏"]);
    assert_eq!(built.counts.broad, 4, "臉、笑、貓、寵物");
}

#[test]
fn squared_ideograph_buttons_are_left_out() {
    // 🈶、㊗ 是框起來的漢字：「有」「祝」只是框裡的字，接在常用字後面只是雜訊。
    let built = build();
    assert!(built.table.lookup("有").is_empty());
    assert!(built.table.lookup("祝").is_empty());
    assert!(built.table.lookup("恭喜").is_empty());
    assert_eq!(built.counts.buttons, 2);
}
