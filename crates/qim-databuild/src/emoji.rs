//! emoji 資料包（plan §4.4）：CLDR 的粵語、繁中標註建「字詞 → emoji」的對照表。
//!
//! 每個字詞的 emoji 依序是：短名（`type="tts"`）剛好是它的，再來是關鍵字有它的；兩種之內
//! 依標註檔的順序（粵語在前），檔內依檔裡的順序。CLDR 的 `cp` 去掉了 U+FE0F，照 Unicode 的
//! `emoji-test.txt` 換回完整寫法（fully-qualified），否則 ❤ 這類字會顯示成文字。關鍵字對到太多 emoji 的
//! 不收（[`MAX_KEYWORD_EMOJI`]），方框漢字整個不收（[`is_squared_ideograph`]）。

use std::collections::{HashMap, HashSet};

use qim_core::EmojiTable;
use quick_xml::escape::resolve_xml_entity;
use quick_xml::events::Event;
use quick_xml::{Reader, XmlVersion};

use crate::Error;

/// 關鍵字最多對到幾個 emoji 才收（M3 票 16）：CLDR 的關鍵字是給 emoji 選單搜尋的，「臉」「眼」
/// 「動物」這類分類詞對到幾十個 emoji，接在常用詞後面只會擠掉候選。門檻由
/// `scripts/experiments/emoji-keywords/` 依首選字基準的修正次數與抽查定。
pub const MAX_KEYWORD_EMOJI: usize = 6;

/// `sources.toml` 的 `[source.cldr]`。
pub const CLDR: &str = "cldr";
/// CLDR 的標註檔（名字、`[source.cldr]` 裡的路徑），依優先順序：粵語在前（brief §1.3 粵語字序以
/// iOS 為準）。
pub const CLDR_ANNOTATIONS: [(&str, &str); 2] = [
    ("yue", "common/annotations/yue.xml"),
    ("zh_Hant", "common/annotations/zh_Hant.xml"),
];
/// `sources.toml` 的 `[source.emoji_test]` 與它的檔。
pub const EMOJI_TEST: &str = "emoji_test";
pub const EMOJI_TEST_FILE: &str = "emoji-test.txt";

/// emoji 變體選擇符（U+FE0F）。
const VARIATION_SELECTOR: char = '\u{FE0F}';

/// 一份 CLDR 標註檔（`common/annotations/<語言>.xml`）。
pub struct Annotations<'a> {
    /// 錯誤訊息用的名字，例如 `yue`。
    pub name: &'a str,
    pub xml: &'a str,
}

/// 建出的對照表與統計。
#[derive(Debug)]
pub struct Built {
    pub table: EmojiTable,
    pub counts: Counts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    /// 各標註檔的 `<annotation>` 數（短名與關鍵字各算一個）。
    pub annotations: usize,
    /// 對到的 emoji 超過門檻、不收的關鍵字數（不重複）；短名不受門檻限制。
    pub broad: usize,
    /// 方框漢字的 emoji 數（不重複，[`is_squared_ideograph`]），整個不收。
    pub buttons: usize,
    /// 有字打不出、不收的字詞數（不重複）。
    pub untypable: usize,
    /// 收的字詞數。
    pub words: usize,
    /// 對照表裡不重複的 emoji 數。
    pub emoji: usize,
}

/// 一個 `<annotation>`：emoji（`cp`）、是不是短名、內容。
struct Annotation {
    cp: String,
    tts: bool,
    text: String,
}

/// `files` 依優先順序；`normalize` 把字詞換成詞庫的寫法（常見寫法），`typable` 判斷一個字打不打
/// 得出（有速成碼）。有字打不出的字詞不收：它永遠不會是候選。關鍵字對到的 emoji 超過
/// `max_keyword_emoji` 個時不收（[`MAX_KEYWORD_EMOJI`]）；短名一律收。
pub fn table(
    files: &[Annotations],
    emoji_test: &str,
    normalize: &dyn Fn(&str) -> String,
    typable: &dyn Fn(char) -> bool,
    max_keyword_emoji: usize,
) -> Result<Built, Error> {
    let qualified = fully_qualified(emoji_test);
    let parsed = files
        .iter()
        .map(|file| parse(file).map(|list| (file, list)))
        .collect::<Result<Vec<_>, _>>()?;
    let annotations = parsed.iter().map(|(_, list)| list.len()).sum();
    // 短名與關鍵字各一串（字詞, emoji），依檔的優先順序、檔內順序。
    let mut short_names: Vec<(String, String)> = Vec::new();
    let mut keywords: Vec<(String, String)> = Vec::new();
    let mut buttons: HashSet<&str> = HashSet::new();
    for a in parsed.iter().flat_map(|(_, list)| list) {
        if is_squared_ideograph(&a.cp) {
            buttons.insert(&a.cp);
            continue;
        }
        let emoji = qualified
            .get(&a.cp)
            .cloned()
            .unwrap_or_else(|| a.cp.clone());
        let (words, out): (Vec<&str>, _) = if a.tts {
            (vec![a.text.trim()], &mut short_names)
        } else {
            (a.text.split('|').map(str::trim).collect(), &mut keywords)
        };
        out.extend(
            words
                .into_iter()
                .filter(|w| !w.is_empty())
                .map(|w| (normalize(w), emoji.clone())),
        );
    }
    // 關鍵字對到幾個 emoji（各檔合計、不重複）；太多的是「臉」「動物」這類分類詞，不收。
    let mut shared: HashMap<&str, HashSet<&str>> = HashMap::new();
    for (word, emoji) in &keywords {
        shared.entry(word).or_default().insert(emoji);
    }
    let broad: HashSet<String> = shared
        .into_iter()
        .filter(|(_, emoji)| emoji.len() > max_keyword_emoji)
        .map(|(word, _)| word.to_string())
        .collect();
    // 先排短名，再排關鍵字；`EmojiTable::new` 依序接起同一個字詞的 emoji。
    let pairs = short_names.into_iter().chain(
        keywords
            .into_iter()
            .filter(|(word, _)| !broad.contains(word)),
    );
    let (kept, dropped): (Vec<_>, Vec<_>) = pairs
        .into_iter()
        .partition(|(word, _)| word.chars().all(typable));
    let mut untypable: Vec<&str> = dropped.iter().map(|(w, _)| w.as_str()).collect();
    untypable.sort_unstable();
    untypable.dedup();
    let table = EmojiTable::new(kept.into_iter().map(|(w, e)| (w, [e])));
    let mut emoji: Vec<&String> = table.iter().flat_map(|(_, e)| e).collect();
    emoji.sort_unstable();
    emoji.dedup();
    Ok(Built {
        counts: Counts {
            annotations,
            broad: broad.len(),
            buttons: buttons.len(),
            untypable: untypable.len(),
            words: table.len(),
            emoji: emoji.len(),
        },
        table,
    })
}

/// 方框漢字（🈶、🈚、🈷、㊗…，Unicode 的 Enclosed Ideographic Supplement 與 ㊗ ㊙）：標註只是框裡的字
/// 本身（「有」「無」「月」），接在這些常用字後面沒有意思、只擠掉候選，整個不收。
fn is_squared_ideograph(cp: &str) -> bool {
    cp.chars()
        .next()
        .is_some_and(|c| matches!(c, '\u{1F200}'..='\u{1F2FF}' | '\u{3297}' | '\u{3299}'))
}

/// `emoji-test.txt` 的 fully-qualified 列：去掉 U+FE0F 的寫法 → 完整寫法。
fn fully_qualified(emoji_test: &str) -> HashMap<String, String> {
    emoji_test
        .lines()
        .filter_map(|line| {
            let (points, rest) = line.split_once(';')?;
            let status = rest.split('#').next()?.trim();
            (status == "fully-qualified").then_some(points)
        })
        .filter_map(|points| {
            points
                .split_whitespace()
                .map(|hex| u32::from_str_radix(hex, 16).ok().and_then(char::from_u32))
                .collect::<Option<String>>()
        })
        .map(|full| (full.replace(VARIATION_SELECTOR, ""), full))
        .collect()
}

/// 依檔內順序讀出每個 `<annotation>`。
fn parse(file: &Annotations) -> Result<Vec<Annotation>, Error> {
    let what = format!("CLDR 標註 {}", file.name);
    let invalid = |reason: String| Error::invalid(what.clone(), reason);
    let mut reader = Reader::from_str(file.xml);
    let mut list = Vec::new();
    let mut open: Option<Annotation> = None;
    loop {
        match reader.read_event() {
            Err(e) => {
                return Err(invalid(format!("{e}（位置 {}）", reader.error_position())));
            }
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) if e.local_name().as_ref() == "annotation" => {
                let mut cp = None;
                let mut tts = false;
                for attr in e.attributes() {
                    let attr = attr.map_err(|e| invalid(e.to_string()))?;
                    let value = attr
                        .normalized_value(XmlVersion::Explicit1_0)
                        .map_err(|e| invalid(e.to_string()))?
                        .into_owned();
                    match attr.key.local_name().as_ref() {
                        "cp" => cp = Some(value),
                        "type" => tts = value == "tts",
                        _ => {}
                    }
                }
                let cp = cp.ok_or_else(|| invalid("<annotation> 沒有 cp".into()))?;
                open = Some(Annotation {
                    cp,
                    tts,
                    text: String::new(),
                });
            }
            Ok(Event::Text(t)) => {
                if let Some(a) = &mut open {
                    a.text.push_str(&t.xml10_content());
                }
            }
            Ok(Event::GeneralRef(r)) => {
                if let Some(a) = &mut open {
                    match r.resolve_char_ref().map_err(|e| invalid(e.to_string()))? {
                        Some(c) => a.text.push(c),
                        None => {
                            let name = r.xml10_content();
                            let resolved = resolve_xml_entity(&name)
                                .ok_or_else(|| invalid(format!("不認得的實體 &{name};")))?;
                            a.text.push_str(resolved);
                        }
                    }
                }
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == "annotation" => {
                list.extend(open.take());
            }
            Ok(_) => {}
        }
    }
    Ok(list)
}
