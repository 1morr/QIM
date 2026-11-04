use qim_databuild::mediawiki;

/// 照 pages-articles 的結構縮短：一個條目、一個重定向、一個模板頁、一個用到實體的條目。
const DUMP: &str = r#"<mediawiki xmlns="http://www.mediawiki.org/xml/export-0.11/" version="0.11" xml:lang="yue">
  <siteinfo>
    <sitename>維基百科</sitename>
  </siteinfo>
  <page>
    <title>歐羅</title>
    <ns>0</ns>
    <id>1</id>
    <revision>
      <id>10</id>
      <text bytes="10" xml:space="preserve">'''歐羅'''係通貨。</text>
    </revision>
  </page>
  <page>
    <title>歐元</title>
    <ns>0</ns>
    <id>2</id>
    <redirect title="歐羅" />
    <revision>
      <text bytes="10" xml:space="preserve">#REDIRECT [[歐羅]]</text>
    </revision>
  </page>
  <page>
    <title>Template:話</title>
    <ns>10</ns>
    <id>3</id>
    <revision>
      <text bytes="10" xml:space="preserve">模板內容</text>
    </revision>
  </page>
  <page>
    <title>符號</title>
    <ns>0</ns>
    <id>4</id>
    <revision>
      <text bytes="10" xml:space="preserve">A &amp; B&lt;ref&gt;註&lt;/ref&gt;&#x6587;</text>
    </revision>
  </page>
  <page>
    <title>空頁</title>
    <ns>0</ns>
    <id>5</id>
    <revision>
      <text bytes="0" xml:space="preserve" />
    </revision>
  </page>
</mediawiki>
"#;

#[test]
fn yields_wikitext_of_main_namespace_articles_without_redirects() {
    let mut seen = Vec::new();
    mediawiki::articles(DUMP.as_bytes(), |text| {
        seen.push(text.to_string());
        Ok(())
    })
    .unwrap();
    assert_eq!(seen, ["'''歐羅'''係通貨。", "A & B<ref>註</ref>文", ""]);
}

#[test]
fn malformed_xml_is_an_error() {
    let broken = DUMP.replace(
        "</revision>\n  </page>\n  <page>\n    <title>歐元",
        "</page><title>歐元",
    );
    let result = mediawiki::articles(broken.as_bytes(), |_| Ok(()));
    assert!(result.is_err());
}
