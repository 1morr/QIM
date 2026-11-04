use qim_databuild::wikitext::plain_text;

/// 抽出的文字去掉空白行，方便比對。
fn text(wikitext: &str) -> String {
    plain_text(wikitext)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn plain_paragraph_stays() {
    assert_eq!(
        text("佢哋就合稱做歐羅區。\n\n第二段。"),
        "佢哋就合稱做歐羅區。\n第二段。"
    );
}

#[test]
fn internal_link_keeps_its_label_or_target() {
    assert_eq!(
        text("係[[歐洲聯盟]]入邊十二個[[國家 (政治)|國家]]嘅通貨"),
        "係歐洲聯盟入邊十二個國家嘅通貨"
    );
}

#[test]
fn file_category_and_interlanguage_links_are_removed_with_nested_links() {
    assert_eq!(
        text(
            "[[File:Euro.svg|thumb|依家嘅[[歐圓]]通行情況]]歐羅[[Category:通貨]][[分類:歐洲]][[en:Euro]]"
        ),
        "歐羅"
    );
}

#[test]
fn templates_are_removed_even_when_nested_or_multiline() {
    assert_eq!(
        text(
            "{{Infobox\n|名 = {{lang|en|Euro}}\n|國家 = 法國\n}}\n歐羅{{notetag|[[圓]]係通貨名}}（{{話|en|Euro}}）"
        ),
        "歐羅 （ ）"
    );
}

#[test]
fn removed_template_separates_the_text_around_it() {
    // 模板常是外文名或讀音，拿掉後兩邊的字不相鄰，不能黏成「佢喺出世」。
    assert_eq!(text("佢喺{{lang|en|Paris}}出世"), "佢喺 出世");
}

#[test]
fn tables_are_removed() {
    assert_eq!(
        text("前文\n{| class=\"wikitable\"\n! 國家\n|-\n| 法國\n多行嘅格\n|}\n後文"),
        "前文\n後文"
    );
}

#[test]
fn references_comments_and_dropped_tags_are_removed() {
    assert_eq!(
        text(
            "1歐羅＝100歐仙<ref>[http://example.com 圖片檔]</ref><ref name=\"a\" />（北話叫「歐分」）<!-- 註解 -->\
             <math>x^2</math>完"
        ),
        "1歐羅＝100歐仙（北話叫「歐分」）完"
    );
}

#[test]
fn other_tags_keep_their_content() {
    assert_eq!(
        text("Au<sup>1</sup>lo<span style=\"color:red\">紅色</span>嘅<br />下一行"),
        "Au1lo紅色嘅\n下一行"
    );
}

#[test]
fn bold_italic_and_external_links() {
    assert_eq!(
        text("'''歐羅'''同''斜體''，見[https://example.com 官方網站]同[http://example.com]"),
        "歐羅同斜體，見官方網站同"
    );
}

#[test]
fn headings_lists_and_indented_lines_are_dropped() {
    assert_eq!(
        text("==特點==\n* 列表\n# 編號\n: 縮排\n; 定義\n內文。"),
        "內文。"
    );
    // 開頭的空白多半是拿掉模板剩下的，照留。
    assert_eq!(text("{{Main|歐羅}} 內文。"), "內文。");
}

#[test]
fn language_conversion_keeps_plain_text_and_drops_variant_rules() {
    assert_eq!(
        text("用-{歐羅}-同-{zh-hans:欧元;zh-hant:歐元;}-"),
        "用歐羅同"
    );
}

#[test]
fn unclosed_markup_stays_as_literal_text_like_mediawiki_renders_it() {
    // 少了收尾的模板、連結不吞掉後文；殘留的括號在切句時當分隔。
    assert_eq!(
        text("壞咗{{模板\n\n下一段[[連結。{{完整}}尾"),
        "壞咗{{模板\n下一段[[連結。 尾"
    );
}
