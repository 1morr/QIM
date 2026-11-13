//! 維基百科 dump → 內文（plan〈字級 bigram 與詞級 trigram〉）：bzip2 解壓、讀 XML、抽 wikitext 的內文。

use std::io::{BufReader, Read};

use bzip2::read::MultiBzDecoder;

use crate::{Error, mediawiki, wikitext};

/// 逐篇交出內文，依 dump 的順序。`dump` 讀的是 `pages-articles*.xml.bz2`，邊讀邊解壓。
pub fn articles(
    dump: impl Read,
    mut article: impl FnMut(&str) -> Result<(), Error>,
) -> Result<(), Error> {
    let xml = BufReader::new(MultiBzDecoder::new(dump));
    mediawiki::articles(xml, |text| article(&wikitext::plain_text(text)))
}
