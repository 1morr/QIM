# 碼表、詞庫與語料的取材：授權鏈不清的不用，非商業的只進可選資料包

來源怎麼釘、各自取了什麼寫在 `data/sources.toml` 的註解，處理規則在 `crates/qim-databuild/src/`。

預設資料包只用授權鏈清楚、可以再散布的來源；每份資料包的 `LICENSE`、`NOTICE` 只寫它用到的來源（`ARCHITECTURE.md`〈資料包〉）。

- **粵語口語詞表自己整理**：`data/lexicon/yue-colloquial.tsv`，CC0，不從 rime-cantonese、粵典複製；使用者審過才進資料包。詞頻用 rime-essay 的尺度，分 3000、1000、300 三級。
- **rime-cantonese 不用**：授權鏈不明（`docs/research/facts-tables.md`）。HKCanCor 已分詞，直接取它 train 切分的詞；它的字頻本來就在詞庫資料包，授權不另加。
- **LIHKG、OpenSubtitles 不用**：權利來源不清。Tatoeba 不用的理由在 `data/sources.toml`。
- **粵典只進可選的詞庫資料包** `quick-lexicon-words-hk`，預設詞庫不收：授權照較嚴的非商業條款（`docs/research/facts-tables.md`），粵典的部分不得再授權、不得商業使用，商用改用 `quick-lexicon`。可選資料包的來源不能擋預設資料包：粵典的網址沒有版本，上游一改 sha256 就對不上，這時只略過這份。不改用 Wayback Machine 的存檔：最新只到 2025-04，存新的要在別人的網站留紀錄。
- **兩套 OpenCC 字典**：語料轉香港繁體與字形對照表用 ferrous-opencc 內建的字典，不用釘住的 OpenCC `fae2a0a`：兩者版本不同，用釘住的字典查出的對照表會和實際轉換對不上。簡繁資料包反過來用釘住的 `fae2a0a`、由 `qim-core` 自己轉換，因為 ferrous-opencc 只能用編進 crate 的字典、不能從資料包載入（理由也在 `data/sources.toml`）。換其中一套時兩邊的差異要重看（`docs/development.md`〈語料〉的 `s2hk-diff`、`lexicon-forms`）。

## Consequences

- 語料是 CC BY-SA 4.0，用它訓練的模型隨 App 發佈時是否要以相同授權散布，與 LGPL 資料並列 brief §11 的待決項。
- emoji 的 CLDR 關鍵字覆蓋不均（「笑」在繁中只對到 🤣🥰😼、粵語是 😆🤣😂，「哈哈」「開心」粵語沒有）。要補口語叫法時另加 repo 裡自行整理的詞表（同粵語口語詞表，CC0），等使用者回報。
