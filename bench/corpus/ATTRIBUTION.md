# Attribution

The sentences in this directory are derived from Wikipedia articles and are licensed
under the Creative Commons Attribution-ShareAlike 4.0 International License
(CC BY-SA 4.0), https://creativecommons.org/licenses/by-sa/4.0/. They are test data for the QIM
top-choice benchmark and are not shipped in any app.

Changes made by QIM: text was taken from each source as its section describes and
converted with OpenCC s2hk; it was split into clauses at punctuation and whitespace,
and clauses containing characters outside the QIM continuous character set were
dropped. Each file keeps up to 2000 distinct clauses from the held-out test split,
one per line, preceded by machine-generated Quick codes (one per character, separated
by spaces) and a tab.

## yue-wiki.tsv

- Source: Cantonese Wikipedia, https://zh-yue.wikipedia.org/, by its contributors (each article's page history lists its authors).
- Version: 20261101
- File: zh_yuewiki-20261101-pages-articles.xml.bz2 (sha256 e493dce7899884e250de0dcb208e560f6dff1e8b7970e7abdd015dc61da83126), https://dumps.wikimedia.org/zh_yuewiki/20261101/zh_yuewiki-20261101-pages-articles.xml.bz2
- License: CC-BY-SA-4.0, https://creativecommons.org/licenses/by-sa/4.0/
- Extraction: article text extracted from the wikitext (templates, tables, references, file links, headings and lists removed).

## zh-wiki.tsv

- Source: Chinese Wikipedia, https://zh.wikipedia.org/, by its contributors (each article's page history lists its authors).
- Version: 20220501
- File: zhwiki-20220501-pages-articles1.xml-p1p187712.bz2 (sha256 c7e1b65a7a36092ac35c2f2f2eaa2c12babf07caa223b46217f8ab6db7fea0aa), https://archive.org/download/zhwiki-20220501/zhwiki-20220501-pages-articles1.xml-p1p187712.bz2
- License: CC-BY-SA-4.0, https://creativecommons.org/licenses/by-sa/4.0/
- Extraction: article text extracted from the wikitext (templates, tables, references, file links, headings and lists removed).
