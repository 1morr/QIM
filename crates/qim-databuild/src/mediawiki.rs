//! 讀 MediaWiki 的 XML dump（`pages-articles.xml`），逐篇交出條目的 wikitext。
//! 格式見 <https://www.mediawiki.org/xml/export-0.11.xsd>。

use std::io::BufRead;

use quick_xml::Reader;
use quick_xml::escape::resolve_xml_entity;
use quick_xml::events::Event;

use crate::Error;

const WHAT: &str = "MediaWiki dump";

/// 只交出主名字空間（`<ns>0</ns>`）且不是重定向的頁面，依 dump 的順序。
pub fn articles(
    input: impl BufRead,
    mut article: impl FnMut(&str) -> Result<(), Error>,
) -> Result<(), Error> {
    let mut reader = Reader::from_reader(input);
    let mut buf = Vec::new();
    let mut path: Vec<String> = Vec::new();
    let mut page = Page::default();
    loop {
        let event = reader.read_event_into(&mut buf).map_err(|e| {
            Error::invalid(WHAT, format!("{e}（位置 {}）", reader.error_position()))
        })?;
        match event {
            Event::Start(e) => {
                let name = e.local_name().as_ref().to_string();
                if name == "page" {
                    page = Page::default();
                }
                if name == "redirect" {
                    page.redirect = true;
                }
                path.push(name);
            }
            Event::Empty(e) => match e.local_name().as_ref() {
                "redirect" => page.redirect = true,
                "text" if in_revision(&path) => page.text = Some(String::new()),
                _ => {}
            },
            Event::End(_) => {
                let name = path.pop().unwrap_or_default();
                if name == "page" && page.ns.trim() == "0" && !page.redirect {
                    article(page.text.as_deref().unwrap_or(""))?;
                }
            }
            Event::Text(e) => {
                let text = e.xml10_content();
                page.push(&path, &text);
            }
            Event::CData(e) => {
                let text = e.xml10_content();
                page.push(&path, &text);
            }
            Event::GeneralRef(e) => {
                let resolved = match e.resolve_char_ref() {
                    Ok(Some(ch)) => ch.to_string(),
                    Ok(None) => resolve_xml_entity(&e.xml10_content())
                        .ok_or_else(|| {
                            Error::invalid(WHAT, format!("不認得的實體 &{};", e.xml10_content()))
                        })?
                        .to_string(),
                    Err(err) => return Err(Error::invalid(WHAT, err.to_string())),
                };
                page.push(&path, &resolved);
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    if path.is_empty() {
        Ok(())
    } else {
        Err(Error::invalid(WHAT, "檔案在元素中間結束"))
    }
}

#[derive(Default)]
struct Page {
    ns: String,
    redirect: bool,
    /// 最後一個 revision 的 `<text>`；pages-articles 每頁只有一個。
    text: Option<String>,
}

impl Page {
    fn push(&mut self, path: &[String], s: &str) {
        match path.last().map(String::as_str) {
            Some("ns") if path.len() >= 2 && path[path.len() - 2] == "page" => self.ns.push_str(s),
            Some("text") if in_revision(&path[..path.len() - 1]) => {
                self.text.get_or_insert_with(String::new).push_str(s)
            }
            _ => {}
        }
    }
}

/// `path` 的最後一層是 `<revision>`。
fn in_revision(path: &[String]) -> bool {
    path.last().is_some_and(|n| n == "revision")
}
