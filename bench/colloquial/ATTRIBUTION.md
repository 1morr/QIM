# Attribution

The sentences in this directory come from several sources; each file keeps the
license of its source, given in its section below. They are test data for the QIM
top-choice benchmark and are not shipped in any app.

Changes made by QIM: text was taken from each source and converted with OpenCC as its
section describes; it was split into clauses at punctuation and whitespace,
and clauses containing characters outside the QIM continuous character set were
dropped. Each file keeps up to 2000 distinct clauses from the held-out test split,
one per line, preceded by machine-generated Quick codes (one per character, separated
by spaces) and a tab.

## hkcancor.tsv

- Source: Hong Kong Cantonese Corpus (HKCanCor), transcriptions, https://github.com/fcbond/hkcancor, created by Luke Kang Kwong; please cite K. K. Luke and May L.Y. Wong (2015) The Hong Kong Cantonese Corpus: Design and Uses, Journal of Chinese Linguistics Monograph Series 25, 312-333.
- Version: 39aeadf920e0b5ca93d0ad7792c59e740e7bdd65
- File: data/hkcancor-utf8.zip (sha256 09223963b8756254e15353cad843f8a4b0cbc4b9223dc8a8fa27fb1cf846057e), https://raw.githubusercontent.com/fcbond/hkcancor/39aeadf920e0b5ca93d0ad7792c59e740e7bdd65/data/hkcancor-utf8.zip
- License: CC-BY-4.0, https://creativecommons.org/licenses/by/4.0/
- Extraction: the words of each utterance joined in order, part-of-speech tags and Jyutping removed.
- Conversion: OpenCC t2hk (Traditional to Hong Kong variant forms; the source is already Traditional Chinese), then Hong Kong variant forms written in their common forms, and characters with two kept forms (such as 裏/裡) written in one representative form.

## common-voice-yue.tsv

- Source: Mozilla Common Voice sentence collection, Cantonese (yue and zh-HK), https://github.com/common-voice/common-voice, sentences contributed by Common Voice users.
- Version: ec845c798ead7144f2acc95fd8955e9d3995c7d0
- File: server/data/yue/sentence-collector.txt (sha256 645d6a544634db879cef9d38e13ce58b7cef91772ba4d2ae6f6f674dbeb02a8b), https://raw.githubusercontent.com/common-voice/common-voice/ec845c798ead7144f2acc95fd8955e9d3995c7d0/server/data/yue/sentence-collector.txt
- File: server/data/zh-HK/sentence-collector.txt (sha256 0c537e9d65c1f1de10d70a1e0989d929b557628d975bf480d8b3b6fcc6291f26), https://raw.githubusercontent.com/common-voice/common-voice/ec845c798ead7144f2acc95fd8955e9d3995c7d0/server/data/zh-HK/sentence-collector.txt
- License: CC0-1.0, https://creativecommons.org/publicdomain/zero/1.0/
- Extraction: one sentence per line, used as is.
- Conversion: OpenCC t2hk (Traditional to Hong Kong variant forms; the source is already Traditional Chinese), then Hong Kong variant forms written in their common forms, and characters with two kept forms (such as 裏/裡) written in one representative form.
