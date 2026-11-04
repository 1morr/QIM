//! 從 MediaWiki 的 wikitext 抽出內文（plan §4.3；CirrusSearch dump 不可用時的退路，見 progress）。
//!
//! 只求留下的內文是真的句子，不求完整：模板、表格、註腳、檔案與分類連結整個丟掉，
//! 標題、列表、縮排行也丟掉；留下的殘餘標記由切句當分隔或讓句子被丟掉（`corpus`）。
//! 沒收尾的標記照 MediaWiki 的顯示方式留成原文，不吞掉後文。

use std::ops::Range;

/// 整段內容都丟掉的標籤：註腳、公式、程式碼、圖庫等，裡面不是內文。
const DROPPED_TAGS: [&str; 14] = [
    "ref",
    "references",
    "math",
    "chem",
    "ce",
    "syntaxhighlight",
    "source",
    "pre",
    "code",
    "gallery",
    "imagemap",
    "timeline",
    "score",
    "graph",
];

/// 這些字元開頭的行是標題、列表、縮排、表格殘餘，不是段落。
const DROPPED_LINE_STARTS: [char; 9] = ['=', '*', '#', ':', ';', '|', '!', '{', '}'];

pub fn plain_text(wikitext: &str) -> String {
    let s = remove_comments(wikitext);
    let s = remove_tag_blocks(&s);
    // 模板常是外文名、讀音或資訊框，拿掉後留一個空白，兩邊的字不會黏成原文沒有的相鄰字。
    let s = remove_balanced(&s, "{{", "}}", " ");
    let s = remove_balanced(&s, "{|", "|}", "\n");
    let s = language_conversion(&s);
    let s = internal_links(&s);
    let s = external_links(&s);
    let s = tags(&s);
    let s = s.replace("'''", "").replace("''", "");
    s.lines()
        .map(str::trim)
        .filter(|l| !l.starts_with(DROPPED_LINE_STARTS))
        .flat_map(|l| [l, "\n"])
        .collect()
}

/// `<!-- … -->`；沒收尾的註解照 MediaWiki 吞到結尾。
fn remove_comments(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find("<!--") {
        out.push_str(&rest[..start]);
        rest = match rest[start..].find("-->") {
            Some(end) => &rest[start + end + 3..],
            None => "",
        };
    }
    out.push_str(rest);
    out
}

/// [`DROPPED_TAGS`] 的 `<tag …>…</tag>` 與 `<tag … />`，標籤名不分大小寫。
fn remove_tag_blocks(s: &str) -> String {
    // ASCII 轉小寫不改位元組位置，可以拿來找位置再切原字串。
    let lower = s.to_ascii_lowercase();
    let mut cut: Vec<Range<usize>> = Vec::new();
    let mut i = 0;
    while let Some(found) = lower[i..].find('<') {
        let start = i + found;
        i = start + 1;
        let Some(name) = DROPPED_TAGS.iter().find(|name| {
            lower[start + 1..].starts_with(*name)
                && lower[start + 1 + name.len()..].starts_with([' ', '>', '/', '\t', '\n'])
        }) else {
            continue;
        };
        let Some(open_end) = lower[start..].find('>').map(|e| start + e + 1) else {
            continue;
        };
        let end = if lower[..open_end].ends_with("/>") {
            open_end
        } else {
            let close = format!("</{name}>");
            match lower[open_end..].find(&close) {
                Some(e) => open_end + e + close.len(),
                None => open_end,
            }
        };
        cut.push(start..end);
        i = end;
    }
    // 註腳接在句子後面，拿掉後前後文本來就相連。
    without(s, &cut, "")
}

/// 成對、可巢狀的標記（模板 `{{ }}`、表格 `{| |}`）整個換成 `replacement`；沒配對的開頭留成原文。
fn remove_balanced(s: &str, open: &str, close: &str, replacement: &str) -> String {
    let bytes = s.as_bytes();
    let mut stack = Vec::new();
    let mut pairs: Vec<Range<usize>> = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i..].starts_with(open.as_bytes()) {
            stack.push(i);
            i += open.len();
        } else if bytes[i..].starts_with(close.as_bytes())
            && let Some(start) = stack.pop()
        {
            pairs.push(start..i + close.len());
            i += close.len();
        } else {
            i += 1;
        }
    }
    without(s, &pairs, replacement)
}

/// 繁簡規則挑哪個變體，依序找；`plain_text` 之後還會經 `s2hk`，所以簡體也比沒有好。
const VARIANTS: [&str; 7] = [
    "zh-hk", "zh-hant", "zh-tw", "zh-mo", "zh-hans", "zh-cn", "zh-sg",
];

/// `-{…}-` 繁簡轉換標記（<https://www.mediawiki.org/wiki/Writing_systems/Syntax>）：
/// 沒有規則的 `-{文字}-` 留文字；`zh-xx:文字` 的規則留 [`VARIANTS`] 最前面的那個；
/// 旗標 `R`（原樣）留原文；`H`、`T`、`-` 不顯示，和挑不到變體時一樣留一個空白，免得兩邊的字黏在一起。
fn language_conversion(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find("-{") {
        let Some(len) = rest[start + 2..].find("}-") else {
            break;
        };
        out.push_str(&rest[..start]);
        let inner = &rest[start + 2..start + 2 + len];
        out.push_str(
            conversion_text(inner)
                .filter(|t| !t.is_empty())
                .unwrap_or(" "),
        );
        rest = &rest[start + 2 + len + 2..];
    }
    out.push_str(rest);
    out
}

/// 一個 `-{inner}-` 顯示的文字；不顯示時是 `None`。
fn conversion_text(inner: &str) -> Option<&str> {
    let (flags, body) = match inner.split_once('|') {
        Some((flags, body))
            if flags.len() <= 4
                && flags
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c == '-' || c == ';') =>
        {
            (flags, body)
        }
        _ => ("", inner),
    };
    if flags.contains('R') {
        return Some(body);
    }
    if flags.contains(['H', 'T', '-']) {
        return None;
    }
    if !body.contains("zh") {
        return Some(body);
    }
    // 每條規則 `變體:文字`，單向規則 `原文=>變體:文字`。
    let rules: Vec<(&str, &str)> = body
        .split(';')
        .filter_map(|rule| {
            let rule = rule.rsplit_once("=>").map_or(rule, |(_, r)| r);
            let (variant, text) = rule.split_once(':')?;
            Some((variant.trim(), text.trim()))
        })
        .collect();
    VARIANTS
        .iter()
        .find_map(|v| rules.iter().find(|(variant, _)| variant == v))
        .map(|(_, text)| *text)
}

/// `[[目標|文字]]` 留文字、`[[目標]]` 留目標；目標含 `:` 的（檔案、分類、跨語言）整個丟掉。
fn internal_links(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find("[[") {
        let Some(len) = closing(&rest[start + 2..], "[[", "]]") else {
            // 沒收尾的 `[[` 留成原文，後面的連結照常處理。
            out.push_str(&rest[..start + 2]);
            rest = &rest[start + 2..];
            continue;
        };
        out.push_str(&rest[..start]);
        let inner = &rest[start + 2..start + 2 + len];
        let (target, label) = match inner.split_once('|') {
            Some((target, label)) => (target, label),
            None => (inner, inner),
        };
        if !target.contains(':') {
            out.push_str(&internal_links(label));
        }
        rest = &rest[start + 2 + len + 2..];
    }
    out.push_str(rest);
    out
}

/// `s` 開頭在一個 `open` 之內，回傳與它配對的 `close` 的位置。
fn closing(s: &str, open: &str, close: &str) -> Option<usize> {
    let mut depth = 0;
    let mut i = 0;
    while i < s.len() {
        if s[i..].starts_with(open) {
            depth += 1;
            i += open.len();
        } else if s[i..].starts_with(close) {
            if depth == 0 {
                return Some(i);
            }
            depth -= 1;
            i += close.len();
        } else {
            i += s[i..].chars().next().map_or(1, char::len_utf8);
        }
    }
    None
}

/// `[網址 文字]` 留文字，`[網址]` 丟掉；同一行內沒收尾的留成原文。
fn external_links(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find('[') {
        let after = &rest[start + 1..];
        let is_url = ["http://", "https://", "//", "ftp://"]
            .iter()
            .any(|scheme| after.starts_with(scheme));
        let end = after
            .find([']', '\n'])
            .filter(|&e| after[e..].starts_with(']'));
        let (true, Some(end)) = (is_url, end) else {
            out.push_str(&rest[..=start]);
            rest = after;
            continue;
        };
        out.push_str(&rest[..start]);
        if let Some((_, label)) = after[..end].split_once(' ') {
            out.push_str(label);
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

/// 其餘的 HTML 標籤只拿掉標籤、留內容；`<br>` 換成換行。
fn tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find('<') {
        let after = &rest[start + 1..];
        let name = after.strip_prefix('/').unwrap_or(after);
        let end = after
            .find(['>', '\n'])
            .filter(|&e| after[e..].starts_with('>'));
        let (true, Some(end)) = (name.starts_with(|c: char| c.is_ascii_alphabetic()), end) else {
            out.push_str(&rest[..=start]);
            rest = after;
            continue;
        };
        out.push_str(&rest[..start]);
        if name.get(..2).is_some_and(|n| n.eq_ignore_ascii_case("br")) {
            out.push('\n');
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

/// `cut` 的聯集每一段換成 `replacement`；`cut` 可以互相包含。
fn without(s: &str, cut: &[Range<usize>], replacement: &str) -> String {
    let mut cut = cut.to_vec();
    cut.sort_by_key(|r| r.start);
    let mut out = String::with_capacity(s.len());
    let mut kept_from = 0;
    for r in cut {
        if r.start >= kept_from {
            out.push_str(&s[kept_from..r.start]);
            out.push_str(replacement);
        }
        kept_from = kept_from.max(r.end);
    }
    out.push_str(&s[kept_from.min(s.len())..]);
    out
}
