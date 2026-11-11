//! 繁體模式要排在繁體字之後的簡體字怎麼判定（M3 票 19、plan §4.1 步驟 4）。

use std::collections::BTreeMap;

use qim_databuild::{libcangjie, opencc, simplified};

/// libcangjie `table.txt` 的格式：字、簡體、漢字、Big5、HKSCS 旗標…、三代碼、五代碼、排序值。
/// 旗標照 libcangjie `346026b`，只有「床」的第二列是假的：同一個字分在兩列、旗標不同時，
/// 有一列在 Big5 就不是簡體字。碼是假的，判定不看碼。
const TABLE: &str = "\
床 床 1 1 0 0 1 0 0 0 0 id id NA 20006
床 床 1 0 1 0 0 0 0 0 0 idd idd NA 0
门 门 1 0 1 0 0 0 0 0 0 lls lls NA 0
門 门 1 1 0 0 1 0 0 0 0 an an NA 20000
农 农 1 0 1 0 0 0 0 0 0 bhv bhv NA 0
人 人 1 1 0 0 1 0 0 0 0 o o NA 20001
台 台 1 1 0 0 1 0 0 0 0 ir ir NA 20002
后 后 1 1 0 0 1 0 0 0 0 hmr hmr NA 20003
里 里 1 1 0 0 1 0 0 0 0 wg wg NA 20004
干 干 1 1 0 0 1 0 0 0 0 mj mj NA 20005
面 面 1 1 0 0 1 0 0 0 0 mwyl mwyl NA 20007
系 系 1 1 0 0 1 0 0 0 0 hvif hvif NA 20008
吓 吓 1 0 1 0 0 0 0 0 0 rmy rmy NA 0
温 温 1 0 1 0 0 0 0 0 0 eat eat NA 0
叶 叶 1 0 1 0 0 0 0 0 0 rj rj NA 0
麽 麽 1 0 1 0 0 0 0 0 0 ido ido NA 0
嘅 嘅 1 0 1 0 1 0 0 0 0 raiu raiu NA 0
够 够 1 0 1 0 0 0 0 0 0 nipr pnr NA 0
叙 叙 1 0 1 0 0 0 0 0 0 ode ode NA 0
𠀾 𠀾 1 0 0 0 0 0 0 0 0 mm mm NA 0
";

/// OpenCC STCharacters：簡體字 → 繁體字（簡轉繁時換成的字，第一個是預設）。
const ST: &str = "\
门\t門
农\t農
台\t臺 台 檯 颱
后\t後 后
里\t裏 裡 里
干\t幹 乾 干
面\t面 麪
系\t系 係 繫
床\t牀
吓\t嚇
温\t溫
叶\t葉 叶
麽\t麼
够\t夠
叙\t敘
𠀾\t𠁞
";

/// OpenCC TSCharacters：繁體字 → 簡體字；「麽」對到自己，是繁體字。
const TS: &str = "\
門\t门
農\t农
麽\t麽
";

/// OpenCC HKVariants：標準繁體字 → 香港寫法。
const HK_VARIANTS: &str = "溫\t温\n";

/// Unihan `kHKGlyph`（《常用字字形表》2000 年修訂本的序號，Unihan 18.0.0）：「够」收了、「夠」沒收；
/// 「叙」和「敘」同一條（1715），是列在一起的異體。
fn hk_glyphs() -> BTreeMap<char, u16> {
    BTreeMap::from([('够', 832), ('叙', 1715), ('敘', 1715), ('門', 4329)])
}

fn characters() -> String {
    let rows = libcangjie::parse(TABLE).unwrap();
    let dictionary = |name, text| opencc::parse(name, text).unwrap();
    simplified::characters(
        &rows,
        &simplified::References {
            st: &dictionary("ST", ST),
            ts: &dictionary("TS", TS),
            hk_variants: &dictionary("HK", HK_VARIANTS),
            hk_glyphs: &hk_glyphs(),
        },
    )
    .into_iter()
    .collect()
}

#[test]
fn hkscs_only_characters_that_opencc_always_converts_to_traditional_are_simplified() {
    assert_eq!(characters(), "农叙门");
}

/// 繁簡同形（人）與本身是正規繁體字、又是別的字的簡體寫法的字（台、后、里、干、面、系）都在 Big5；
/// 床（OpenCC 簡轉繁一律換成牀）也是。
#[test]
fn big5_characters_are_never_simplified() {
    let found = characters();
    for c in ['人', '台', '后', '里', '干', '面', '系', '門', '床'] {
        assert!(!found.contains(c), "{c} 被判成簡體字");
    }
}

#[test]
fn cantonese_hong_kong_and_self_converting_characters_are_kept() {
    let found = characters();
    // 吓：前處理保留的粵語用字；温：香港寫法；叶：簡轉繁可能留著它；麽：繁轉簡留著它。
    for c in ['吓', '温', '叶', '麽'] {
        assert!(!found.contains(c), "{c} 被判成簡體字");
    }
}

#[test]
fn characters_outside_the_continuous_set_are_not_marked() {
    assert!(!characters().contains('𠀾'));
}

/// 香港《常用字字形表》的標準字形不算簡體字：字表收了它、沒收它的繁體寫法（够／夠）。
/// 和繁體寫法同一條的異體（叙／敘）照算。
#[test]
fn hong_kong_standard_graphemes_are_not_simplified() {
    let found = characters();
    assert!(!found.contains('够'), "{found}");
    assert!(found.contains('叙'), "{found}");
}
