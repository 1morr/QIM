# Attribution

The sentences in this directory are derived from Wikipedia articles and are licensed
under the Creative Commons Attribution-ShareAlike 4.0 International License
(CC BY-SA 4.0), https://creativecommons.org/licenses/by-sa/4.0/. They are test data
for the QIM top-choice benchmark and are not shipped in any app.

Changes made by QIM: article text was extracted from the wikitext (templates, tables,
references, file links, headings and lists removed) and converted with OpenCC s2hk;
the text was split into clauses at punctuation and whitespace, and clauses containing
characters outside the QIM continuous character set were dropped. Each file keeps up to
2000 distinct clauses from the held-out test split, one per line, preceded by
machine-generated Quick codes (one per character, separated by spaces) and a tab.

## yue-wiki.tsv

- Source: Cantonese Wikipedia, https://zh-yue.wikipedia.org/, by its contributors (each article's page history lists its authors).
- Dump: 20261101, file zh_yuewiki-20261101-pages-articles.xml.bz2 (sha256 e493dce7899884e250de0dcb208e560f6dff1e8b7970e7abdd015dc61da83126), https://dumps.wikimedia.org/zh_yuewiki/20261101/zh_yuewiki-20261101-pages-articles.xml.bz2
- License: CC-BY-SA-4.0
