//! 維基百科 dump → 繁體內文（plan §4.3）：bzip2 解壓、讀 XML、抽 wikitext 的內文、OpenCC `s2hk`。

use std::io::BufReader;

use bzip2::read::MultiBzDecoder;
use ferrous_opencc::OpenCC;
use ferrous_opencc::config::BuiltinConfig;

use crate::{Error, mediawiki, wikitext};

/// 逐篇交出轉成香港繁體的內文，依 dump 的順序。`dump` 是 `pages-articles.xml.bz2` 的內容。
pub fn articles(
    dump: &[u8],
    mut article: impl FnMut(&str) -> Result<(), Error>,
) -> Result<(), Error> {
    let s2hk = OpenCC::from_config(BuiltinConfig::S2hk)
        .map_err(|e| Error::invalid("OpenCC s2hk", e.to_string()))?;
    let xml = BufReader::new(MultiBzDecoder::new(dump));
    mediawiki::articles(xml, |text| {
        article(&s2hk.convert(&wikitext::plain_text(text)))
    })
}
