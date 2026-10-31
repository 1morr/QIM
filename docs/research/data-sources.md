# 碼表、詞庫、字頻、語料的來源與授權

查證日期：2026-10-30。範圍：QIM（開源、全平台速成輸入法，之後可能上 App Store / Play Store）階段 1 的資料盤點。

**免責**：本文只轉述各來源的授權條款與可查到的事實，不構成法律意見。凡涉及「這樣用是否符合條款」的判斷，一律標「需使用者判斷」。查不到的寫「未查證」。

**方法**：repo 內容是 shallow clone 到 session scratchpad（不在 QIM repo 內）後實際數行數、讀檔頭與 LICENSE；網頁事實用官方頁面或 repo 頁面。比對與統計（Unihan vs 碼表、速成推導）是本次自行用腳本算的，腳本與資料都不進 repo。

---

## 1. 摘要

1. **碼表（倉頡 / 速成）有可直接用的寬鬆來源**：libcangjie 的 `data/table.txt` 明文「置於 Public Domain」，77,575 列，同時含倉頡三代與五代碼、簡體對應、Big5 / HKSCS 旗標、Big5 排序值。Unicode Unihan 的 `kCangjie`（Unicode License v3，29,189 字）是另一條寬鬆來源，但實測它對應的是**三代**（與三代表 99.4% 一致，與五代只有 96.2%）。
2. **Rime 的 cangjie5 / quick5 與 fcitx5-table-extra 全是 copyleft**：rime-cangjie / rime-quick 為 LGPL-3.0，fcitx5-table-extra 為 GPL-3.0-or-later。cangjie5 碼表檔頭還寫 `License: GPL`，與 repo LICENSE（LGPL-3.0）不一致，且上游《五倉世紀》（chinesecj.com）的權利人未查證。
3. **速成碼可由倉頡碼機械推導**：Microsoft 官方文件與 rime-quick 的 schema 都是「倉頡首碼 + 尾碼」。規則：倉頡碼 ≥3 碼取首尾；1–2 碼保持原樣。我們比對 fcitx5 的 `quick5` 對 `cangjie5`（64,487 字）、`quick3` 對 `cangjie3`（17,567 字），0 例外，但這兩張表本身就是用這條規則產生的，所以不能證明真實產品沒有手工例外。
4. **速成的重碼極重，排序資料是核心需求**：fcitx5 的 quick5 全集 674 個碼、67,168 條，單碼最多 480 個候選；常用集（quick-classic）666 碼、17,568 條，最多 93 個。字頻與詞頻不是加分項。
5. **繁體詞頻的寬鬆來源很少**：rime-essay（八股文，442,688 筆含詞頻，LGPL-3.0）與 libchewing-data（`tsi.csv` 160,462 筆，LGPL-2.1-or-later）都是 LGPL；教育部國語辭典是 CC BY-ND 3.0 台灣，且只有詞條沒有詞頻。
6. **粵語**：rime-cantonese 是 CC BY 4.0（`maps` 檔另為 ODbL），但詞條部分參考了粵典、搜狗 / QQ / 百度用戶詞庫等，而粵典自己的授權是**非商業**（Non-Commercial Open Data License 1.0）；兩者關係未查證，是最需要使用者判斷的風險點。粵典另提供 public domain 的「字 → 讀音」詞表。
7. **語言模型**：librime-octagram 外掛是 BSD-3-Clause，但現成的繁體語法模型 `zh-hant-t-essay-bgw.gram`（41 MB）資料授權是 LGPL-3.0，且語料來源只寫「開放的互聯網語料」，無法追溯。KenLM 為 LGPL-2.1 / GPL-3.0 混合。
8. **英文**：SCOWL / ESDB（MIT-like 寬鬆）是最乾淨的詞表；wordfreq 資料為 CC BY-SA 4.0 且已停止更新（約 2021）；AOSP LatinIME 倉庫是 Apache-2.0，但其英文字典的內容來源未查證，且 AOSP 沒有中文 / 粵語主字典。
9. **簡繁轉換**：OpenCC 整個 repo（含字典）是 Apache-2.0。純 Rust（ferrous-opencc，Apache-2.0）與純 JS（opencc-js）可用；Swift（SwiftyOpenCC）與 Android（android-opencc）綁定都停在 2021 年。
10. **Emoji**：CLDR annotations 檔頭為 `SPDX-License-Identifier: Unicode-3.0`，`yue`、`zh_Hant`、`zh_Hant_HK` 各 3,950 個 `<annotation>` 元素，是目前唯一查到的、授權乾淨的粵語 / 繁中 emoji 標註。

---

## 2. 總表

「可上架？」欄只陳述條款對散布的要求，不下結論。縮寫：SA = ShareAlike，NC = NonCommercial，ND = NoDerivatives。

| 資料 | 來源 | 規模（實測 / 官方） | 格式 | 授權 | 備註（需使用者判斷者標 ★） |
|---|---|---|---|---|---|
| 倉頡三代 / 五代 + 屬性 | libcangjie `data/table.txt` | 77,575 列 | 空白分隔文字，15 欄 | Public Domain（README.table.md 明文）；程式碼 LGPL-3.0 | 資料部分來自 Unihan 與 Wan Leung Wong 舊版 libcangjie；PD 聲明與 Unihan 衍生部分的關係 ★ |
| 倉頡三代碼 | Unihan `kCangjie` | 29,189 字（Unicode 18.0.0） | Unihan TSV | Unicode License v3 | 要附版權與許可聲明；實測較接近三代 |
| 倉頡五代 + 擴展區 | rime-cangjie | `base` 21,470 字；`extended` 55,352 行 | Rime dict YAML（text TAB code） | LGPL-3.0（LICENSE）；`cangjie5.dict.yaml` 檔頭寫 `License: GPL`；AUTHORS 註明衍生自 ibus-table (LGPL) 與 朱邦復 (Public Domain) | 授權標示不一致、《五倉世紀》權利人未查證 ★ |
| 速成五代 | rime-quick | 無獨立表，由 cangjie5.base 於建置時推導；`supplement` 6,837 行 | Rime YAML | LGPL-3.0 | 靠 rime-cangjie 資料 |
| 倉頡 3 / 5、速成 3 / 5 / classic | fcitx5-table-extra | cangjie3 17,567；cangjie5 64,487；quick3 / quick-classic 17,567；quick5 64,487（字數） | fcitx table txt（`code char`） | GPL-3.0-or-later | copyleft 最強；僅作比對用最合適 ★ |
| 倉頡 + 港字 | data.gov.hk `HKSCS2016.json` | 約 5 千字（官方頁未列數字，未查證） | JSON：碼位、字、HKSCS 版本、倉頡碼、粵拼、來源 | DATA.GOV.HK Dataset Terms v1.2：商用 / 非商用皆免費，須標明來源與政府 IP | 條款含賠償（indemnify）條款 ★ |
| 常用字表（港） | 教育局《常用字字形表》 | 4,759（1990–2000 修訂本）/ 4,762（2007 重排本）字 | 字表（PDF 為主，未查證電子格式） | 未查證 | 字表本身的授權未查到 |
| 常用字表（台） | 教育部《常用國字標準字體表》 | 4,808 字 | 字表 | 未查證 | 同上；次常用 / 罕用字表未查證 |
| 字頻（簡體為主） | Jun Da 字頻表 | 約 10,000 簡體字 | 網頁表格 | 未查證 | 簡體語料，不適合直接當繁體 / 粵語排序 |
| 字詞頻（繁體） | 中研院平衡語料庫 4.0 | 19,247 篇、11,245,330 詞次、239,598 詞型、17,554,089 字次（1981–2007） | 線上檢索系統 | 未查證（頁面指向授權使用，聯絡信箱而已） | 無查到可下載、可再散布的詞頻檔 |
| 詞頻 + 八股文 | rime-essay `essay.txt` | 442,688 筆（單字 20,746；二字 169,405；三字 114,593；四字 116,359；五字以上 21,585） | `詞 TAB 頻` | LGPL-3.0 | 內容整理自新酷音 (LGPL)、OpenCC (Apache-2.0)、Android Pinyin IME (Apache-2.0)、萌典 (AUTHORS 寫 CC0 1.0) |
| 詞庫 + 注音 + 詞頻 | libchewing-data `tsi.csv` / `word.csv` | 160,462 / 26,105 行 | CSV | LGPL-2.1-or-later（檔頭 `dc:license`） | 繁體、台灣用字習慣；repo 已遷至 Codeberg |
| 詞條（台） | 教育部國語辭典（重編本 / 簡編本 / 小字典 / 成語典） | 重編本約 167,000 詞字；簡編本約 45,000 | 官方下載；g0v/moedict-data 為 JSON | CC BY-ND 3.0 台灣 | ND：官方限制「修改該著作」；教育部另有「使用說明」未取得 ★ |
| 粵拼詞庫 | rime-cantonese | chars 34,375；words 103,090；phrase 330,777 行（資料版本 2026.08.10） | Rime dict YAML | CC BY 4.0；`jyut6ping3.maps` 為 ODbL 1.0 | 詞條參考粵典、搜狗 / QQ / 百度用戶詞庫等，其授權關係未查證 ★ |
| 粵語詞頻 | rime-cantonese `essay-cantonese.txt` | 266,870 筆（頭幾名：我 12,831,921、係、你、唔、佢） | `詞 TAB 頻` | 同 repo（CC BY 4.0） | 頻次產生方式、語料來源未查證 ★ |
| 粵語詞典 | 粵典 words.hk | 筆數未查證 | 網站；CSV 下載說法未取得官方確認 | 非商業開放資料授權協議 1.0（NC） | 商用需另行許可 ★ |
| 粵語「字 → 讀音」詞表 | words.hk `faiman/analysis/wordslist` | 筆數未查證 | JSON / CSV | 頁面標「Data License: public domain」，希望署名 | 僅為字與讀音鍵值，不含釋義 |
| 粵語語料 | HKCanCor（fcbond/hkcancor） | 約 23 萬詞 / 約 20 萬字（兩份文獻口徑不同） | XML / 文字，原始編碼 Big5-HKSCS | CC BY | 口語、廣播稿，1997–1998，規模小 |
| 粵語語料 | 粵語維基 zh-yue | 154,445 篇；dump 129 MB（bz2，2026-10-01） | MediaWiki XML | CC BY-SA 4.0 | SA 對模型 / 衍生資料的影響 ★ |
| 繁中語料 | 中文維基 zh | 1,563,523 篇；dump 約 3.43 GB（bz2） | 同上 | CC BY-SA 4.0 | 同上；以普通話書面語為主 |
| 對白語料 | OpenSubtitles2024（含 yue、zh_TW） | 全集 26.6 GB；各語言筆數未取得 | Hugging Face 資料集 | ODC-By（資料集）；字幕原著作權不歸整理者 | 有 takedown 政策 ★ |
| 論壇語料 | AlienKevin/LIHKG（HF） | 14.4 GB；筆數級距 1M–10M | Hugging Face | 資料卡標 MIT（整理者所標） | 貼文著作權與站方條款未查證；內含個資疑慮 ★ |
| 語法模型 | lotem/rime-octagram-data `hant` 分支 | `bgw` 41,415,724 B；`bgc` 10,318,892 B | `.gram` 二進位 | LGPL-3.0 | 語料來源僅寫「開放的互聯網語料」 ★ |
| 語法模型工具 | lotem/librime-octagram | — | C++ 外掛 + `tools/build_grammar.cc` | BSD-3-Clause | 可自行以建置工具訓練 |
| n-gram 工具 | KenLM | — | C++ | GitHub 顯示 LGPL-2.1 與 GPL-3.0 兩組檔案 | 各檔適用範圍需看檔頭 ★ |
| 英文詞表 | SCOWL / ESDB | 以 size 級別區分（35–85） | 純文字 / SQLite | MIT-like（Kevin Atkinson）；>80 與澳式有額外聲明 | 見 §8 |
| 英文詞頻 | wordfreq | 40+ 語言；zh 含繁簡 | Python 套件 + 資料 | 程式 Apache-2.0；資料 CC BY-SA 4.0 | 已停更（約 2021）；不含粵語 |
| 英文 n-gram | Google Books Ngram | 英文 v1–v3 | TSV | CC BY 3.0 | 中文子集是否提供未查證 |
| 英文鍵盤字典 | AOSP LatinIME `dictionaries/` | 24 種語言 wordlist；無 zh / yue | `.combined.gz` | 倉庫 NOTICE：Apache-2.0 | 字典內容來源未查證 ★ |
| 簡繁轉換 | OpenCC | STPhrases 49,309；STCharacters 4,029；TSCharacters 5,062；TWPhrases 858 行（含各地區表） | 文字字典 + JSON 配置 | Apache-2.0 | 見 §9 |
| Emoji 標註 | Unicode CLDR `common/annotations` | yue / zh_Hant / zh_Hant_HK 各 3,950 個 annotation 元素 | LDML XML | Unicode-3.0（檔頭 SPDX） | 見 §10 |

---

## 3. 倉頡 / 速成碼表

### 3.1 候選來源

**libcangjie（freedesktop）**
- 位置：`https://gitlab.freedesktop.org/cangjie/libcangjie`（GitHub `Cangjians/libcangjie` 已標示遷移）。
- 程式碼：`COPYING` 為 LGPL-3.0。
- **資料** `data/table.txt`：`data/README.table.md` 寫道 "The data in the table.txt file is hereby placed under the Public Domain"。來源列為 Unihan、Unicode 規格、Wan Leung Wong 舊版 libcangjie 的資料、團隊自行考量；並承認「資料可能不完整甚至有錯」。
- 實測 77,575 列，欄位（README 定義）：字、簡體對應、是否漢字、是否在 Big5、是否在 HKSCS、是否在注音、是否日文漢字 / 平假名 / 片假名、是否標點、是否雜項符號、**倉頡三代碼（逗號分隔多碼）**、**倉頡五代碼**、短碼（標點用）、排序整數（Big5 字依 Big5 順序；有短碼者依作者判斷；其餘為 0）。
- 範例實測：`你 → onf`、`明 → ab`、`嘅 → rhpu, raiu`（兩個碼）、`日 → a`。
- 對 QIM 的意義：一份檔同時有三代、五代、HKSCS 旗標、繁簡對應，且是唯一明文 PD 的來源。要自行確認的是「Unihan 衍生」與「Wong 舊資料」兩段 PD 聲明是否足以涵蓋 ★。

**Unicode Unihan `kCangjie`**
- 版本：Unihan.zip（Unicode 18.0.0，檔頭 `© 2026 Unicode`）。`Unihan_DictionaryLikeData.txt` 中 kCangjie 29,189 字。
- UAX #38 說明：「cangjie input code … 納入 Christian Wittern 的 `cangjie-table.b5`」，狀態 Provisional。
- 授權：檔頭指向 `https://www.unicode.org/terms_of_use.html`；`copyright.html` §3.3 說 `/Public/` 下的 Data Files 受 Unicode License v3 約束（該頁未逐字點名 Unihan）。License v3 原文條件：授權通知須隨副本出現，或出現在相關文件中；不得以版權人名義背書；AS IS。
- **實測它是三代**：與 fcitx5 `cangjie3` 重疊 17,431 字中 17,327 字（99.4%）一致；與 rime `cangjie5.base` 重疊 20,910 字中 20,120 字（96.2%）一致。UAX #38 沒標版本，這是推論。
- 不含五代，也不含多碼變體（單字只有一個碼）。

**Rime：rime-cangjie / rime-quick**
- 授權：兩個 repo 的 `LICENSE` 都是 GNU LGPL v3（`https://github.com/rime/rime-cangjie/blob/master/LICENSE`、`…/rime-quick/blob/master/LICENSE`）。
- AUTHORS：`cangjie5.dict.yaml`「Derived from ibus-table by acevery (Yu Yuwei): (LGPL) and based on original work by 朱邦復 / Chu Bong-Foo: (Public Domain)」。`quick5.dict.yaml` 同。schema 檔作者 Gong Chen (LGPL)。
- 內容：`cangjie5.base.dict.yaml` 單字碼（21,470 不同字，碼長分佈 1 碼 32、2 碼 933、3 碼 3,144、4 碼 11,482、5 碼 8,345 條）；`cangjie5.extended.dict.yaml` 55,352 行（擴展區）；`stem` 882 行（詞彙取碼用的部件碼）。
- **不一致處**：`cangjie5.dict.yaml` 檔頭註解寫「單字碼表《五倉世紀》來自 www.chinesecj.com … 倉頡五代構詞碼碼表由惜緣兄製作 … License: GPL」，與 repo LICENSE（LGPL-3.0）不一致；《五倉世紀》本身的權利人與授權未查證 ★。
- rime-quick 沒有獨立的速成表：`quick5.dict.yaml` 匯入 `cangjie5.base` 與 `quick5.supplement`（6,837 行，擴展區 / 相容漢字補充），並在 schema 的 `speller/algebra` 以 `derive/^([^z])\w+(\w)$/$1$2/` 產生速成碼（取倉頡首尾）。
- 詞彙編碼（多字詞的取碼規則）見 `cangjie5.dict.yaml` 的 `encoder.rules`：二字詞 `AaAzBaBbBz` 等，說明為「取首碼、取尾碼、取首尾二碼、取首次尾三碼」。

**fcitx5-table-extra**
- `LICENSES/` 只有 `GPL-3.0-or-later.txt`；AUTHORS 列 You-Sheng Yang、Xuetian Weng。
- 表（`tables/*.txt`，格式為 `code char`，以 `&a 日` 這類行定義鍵碼提示）：`cangjie3` 17,567 字、`cangjie5` 64,487 字（68,667 行）、`cangjie-large` 70,867 行、`quick3` / `quick-classic` 17,567 字、`quick5` 64,487 字、`scj6`（SmartCangJie6）77,327 行、`easy-large` 129,855 行、`cantonhk`（港式粵拼）14,017 行。
- 用途：可交叉驗證其他來源，但 GPL-3.0-or-later 的條件最重，QIM 若不想帶 GPL，不要把它當資料來源 ★。

**Microsoft 速成**
- 官方文件（MicrosoftDocs/globalization `traditional-chinese-ime.md`）：「Quick is also known as Simplified ChangJie. It lets users enter a character's first and the last stroke used in the full ChangJie system and then pick a character from the conversion candidate window.」可作為取碼規則的官方出處。
- 碼表 / 詞庫檔的授權：未查證。這是 Windows 內建的專有 IME 資料（FOD 套件），沒查到公開再散布條款；不應當作可用來源。
- Wikipedia 另載：Microsoft 在 Windows 3.1x 起繁體版附 Quick，Office 2007 / Windows 7 起提供 Quick 與 New Quick。

**其他**
- 香港政府：data.gov.hk 的 HKSCS 資料集（數位政策辦公室）含碼位、字、首次收錄的 HKSCS 版本、**倉頡碼**、**粵語拼音**、來源。僅一個 `HKSCS2016.json`，無歷史版本。條款見 §4.3。「CCLI」（ccli.gov.hk）頁面只取得標題與連結，內容未查證。
- 倉頡三代、五代的原始著作權（朱邦復、倉頡平台）：AUTHORS 與 libcangjie README 皆稱 Public Domain，**我們沒有找到獨立的原始出處證實**，未查證 ★。

### 3.2 速成碼能否由倉頡碼推導

- **規則**：倉頡碼 ≥3 碼 → 首碼 + 尾碼；1–2 碼 → 原樣。依據：Microsoft Learn 的描述；rime-quick schema 的正規式；中文教學站（xn--0vqu8au0tro7d.com 與 pascal-man.com 講義）皆稱「速成取首尾碼」。
- **實測**（自行比對，資料為 fcitx5-table-extra）：
  - `quick5` 對 `cangjie5`（64,487 字）：0 例外。
  - `quick3`（含 `quick-classic`）對 `cangjie3`（17,567 字）：0 例外。
  - 局限：這些 fcitx 表本身是依此規則產生的，所以是「一致」而非「獨立驗證」。
- **已知的特殊情形**：
  - 單字多碼（如 `嘅 → rhpu, raiu`）：fcitx5 cangjie5 表的 64,487 字中有 3,921 字有多個五代碼，每個碼各自推導速成碼，可能得到不同的速成碼。
  - rime-quick 的正規式排除以 `z` 起頭的碼（`z` 是「重」，作為符號 / 反查前綴使用）；`x`（難）作輔助字形。
  - 1 碼字（`日 → a`）與 2 碼字照原碼；rime-quick 說明單碼字可以 `;` 或 `'` 與後字隔開。
  - rime-quick 用 `supplement`（6,837 行）補入倉頡五代基礎表沒有的擴展區 / 相容漢字。
- **未查證**：真實產品（Microsoft、macOS、Google）是否對個別字做手工例外、速成與倉頡在「簡易」（速成 3 碼字、3 碼以上字）處理上的歷史差異。
- **重碼規模**（自行統計 fcitx5 表）：
  | 表 | 不同碼 | 條目 | 最大候選數 | 平均候選數 |
  |---|---|---|---|---|
  | quick-classic（約 1.7 萬字） | 666 | 17,568 | 93 | 26.4 |
  | quick5（約 6.4 萬字） | 674 | 67,168 | 480 | 99.7 |
  碼的長度只有 1 碼（24 個）與 2 碼（約 640 個）。排序（字頻 / 詞頻 / 上下文）決定使用體驗。

---

## 4. 字集與字頻

### 4.1 常用字表

- **香港《常用字字形表》**（教育局，前教育署語文教育學院編）：1986 初版 4,719 字，1990–2000 修訂本 4,759 字，2007 重排本（附於《香港小學學習字詞表》）與 2012 獨立成書版 4,762 字。依據：台灣《常用國字標準字體表》（4,808 字）與《小學常用字表》（2,746 字）加以增刪（來源：zh.wikipedia「常用字字形表」）。字表本身授權、是否有官方電子檔：**未查證**。
- **台灣《常用國字標準字體表》**：4,808 字（教育部）；次常用、罕用字表的字數與授權：未查證。
- **HKSCS-2016**：data.gov.hk 資料集見 §3 / §4.3。Adobe 的 CJK Type Blog 記錄 HKSCS-2016 新增 24 字並更動 2 個映射。完整字數與 PDF 附件：未查證。
- 註：「字集」是事實性清單，其授權是否涵蓋「單純的字清單」需使用者判斷 ★。

### 4.2 字頻

- **Jun Da（MTSU）字頻表**：現代漢語字頻約 10,000 簡體字（2005 年資料，Hacking Chinese 描述），另有古漢語、小說、非小說分表與二元組表；**使用條款未查到**，且為簡體語料，用於繁體 / 粵語排序需轉換並自行校正 ★。
- **中研院現代漢語平衡語料庫 4.0（Sinica Corpus）**：19,247 篇、11,245,330 詞次、239,598 詞型、17,554,089 字次（1981–2007，台灣華語）。頁面（編碼損毀，閱讀不完整）似乎提到自 2010 年起可授權使用，並列有聯絡信箱；是否能下載、詞頻能否再散布：**未查證**。
- **rime-essay**：詞頻最完整的現成繁體資料（見 §5）；`的 4,822,928`、`了 1,512,483`、`我 1,191,912`。
- **粵語字詞頻**：`rime-cantonese/essay-cantonese.txt`（見 §6）。
- Wikipedia 統計：可自行從 dump 計算（CC BY-SA 4.0）；沒有找到現成、授權明確的繁體字頻表。

### 4.3 HKSCS 資料集的條款（data.gov.hk）

- 資料集：「Hong Kong Supplementary Character Set related information」（數位政策辦公室），單一檔 `HKSCS2016.json`，更新頻率「As and when necessary」。頁面自身未標獨立授權，連到通用條款。
- DATA.GOV.HK Dataset Terms of Use v1.2（2025-05-26）：
  - 使用「for both commercial and non-commercial purposes on a free-of-charge basis」。
  - 須「identify clearly the source of the Data」，並在所有副本（含數位副本、其他網站）承認政府與相關機構的 IP 擁有權。
  - 使用者須對政府與相關機構因其使用或散布而遭第三方侵權索賠所致的損失作賠償（indemnify）。
  - 資料 AS IS；條款可不經通知更改；條款未明確談修改 / 衍生作品。
  - 香港法律管轄。
- 需使用者判斷：indemnity 條款對一個開源、上架商店的 app 是否可接受 ★。

---

## 5. 繁體詞庫與詞頻

| 來源 | 規模（實測） | 授權 | 說明 |
|---|---|---|---|
| rime-essay（八股文） | 442,688 筆 | LGPL-3.0 | 字詞 + 整數頻次。AUTHORS：Gong Chen、Kunki Chiu、ksqsf (LGPL)；整理自 Chewing / 新酷音 (LGPL)、OpenCC (Apache-2.0)、Android Pinyin IME (Apache-2.0)、moedict.tw / 萌典 (CC0 1.0)。頭尾有 `〇`、`㐀` 等零頻條目 |
| libchewing-data | `tsi.csv` 160,462；`word.csv` 26,105；`mini.csv` 7,013 | LGPL-2.1-or-later（檔頭 `dc:rights Copyright (c) 2025 libchewing Core Team`） | 詞、頻、注音；`tsi.csv` 版本 2025.11.17；GitHub 上已標示遷移至 Codeberg |
| 教育部國語辭典 | 重編本約 167,000、簡編本約 45,000 | CC BY-ND 3.0 台灣 | 見下 |
| g0v/moedict-data | JSON（`dict-revised.json.xz` 等） | 內容同教育部授權；編輯著作權由 @kcwu 以 CC0 釋出 | 只有詞條與釋義，沒有頻次 |
| rime-ice / 霧凇 | `cn_dicts/` 6 個檔 | GPL-3.0（README 末段「GPL-3.0 (only) License」） | 簡體拼音詞庫，來源含華宇野風、THUOCL、騰訊詞向量；對繁體速成無直接價值 |
| 萬象（RIME-LMDG / rime_wanxiang） | `dicts_hant/` 另有繁體詞庫 | 資料 CC BY 4.0；其 `LICENSE-SCOPE.md` 稱原創程式碼「未授予獨立的複製、修改、再分發或商業使用許可」 | 以簡體拼音為主，繁體詞庫內容與品質未驗證 |
| libime（fcitx） | `data/` 只有 CMakeLists | LGPL-2.1-or-later | repo 不內附詞庫；詞庫在另一處下載，未查證 |
| jieba `extra_dict/dict.txt.big` | 584,429 行 | MIT（GitHub 顯示） | 抽樣 3 行為簡體（如「1号店」「一体同心」），繁體覆蓋與詞頻來源未查證；wordfreq 作者稱 jieba 詞表來源不明 |
| CKIP | `ckip-transformers`（GPL-3.0，程式） | 模型授權：未查證 | 分詞 / 詞性模型，不是詞表 |

### 教育部國語辭典授權（CC BY-ND 3.0 台灣）

- 教育部頁面（`https://depart.moe.edu.tw/ed2400/cp.aspx?n=D509C3C9F6F496FD&s=31218F193CB4FD6E`）：「本部採用「創用CC-姓名標示-禁止改作 臺灣3.0版授權條款」釋出《國語小字典》、《國語辭典簡編本》、《重編國語辭典修訂本》與《成語典》內文資料」；「本授權條款允許使用者重製、散布、傳輸著作（包括商業性利用），但不得修改該著作，使用時必須遵照「使用說明」之內容要求」。
- g0v/moedict-data README 轉述教育部的解釋：「「創用CC-姓名標示- 禁止改作 臺灣3.0版授權條款」之改作限制標的為文字資料本身，不限制格式轉換及後續應用」。這是 g0v 的轉述，原始公文未取得 ★。
- 影響：ND 對「抽取詞條作為輸入法詞庫」是否構成「修改該著作」需使用者判斷 ★。rime-essay 的 AUTHORS 把萌典標為 CC0 1.0，而 g0v 的 CC0 只涵蓋「轉換格式、重新編排的編輯著作權」，字典內文仍為教育部著作；兩種標示並存。

---

## 6. 粵語

### 6.1 rime-cantonese（CanCLID）

- 授權（README 原文）："The main part of this work is released under a Creative Commons Attribution 4.0 International License. `jyut6ping3.maps` is released under a Open Data Commons Open Database v1.0 License."（repo 內有 `LICENSE-CC-BY`、`LICENSE-ODbL`）。
- 實測：`jyut6ping3.chars.dict.yaml` 34,375 行（字 + 粵拼 + 讀音百分比）、`words` 103,090 行、`phrase` 330,777 行、`lettered` 1,324 行、`maps` 4,466 行；`essay-cantonese.txt` 266,870 筆（單字 20,549；二字 127,280；三字 65,412；四字 45,436；五字以上 8,193）。資料版本 `2026.08.10`。另有 `opencc/HKVariantsFull.txt`、`t2hkf.json`（港式異體轉換）。
- 上游 `CanCLID/rime-cantonese-upstream`：CC BY 4.0，8 個 CSV（`char.csv` 34,363；`word.csv` 93,689；`phrase_fragment.csv` 7,276；`fixed_expressions.csv` 938 等行）。README 列出的字音來源：LSHK 電腦用漢字粵語拼音表（主）、Unihan 12.0 kCantonese、粵語審音配詞字庫、《廣州話正音字典》；詞條來源：粵典、冚唪唥粵文、《實用廣州話分類詞典》、A Dictionary of Cantonese Slang、《廣州話詞典》、《地道廣州話用語》。
- 下游 `jyut6ping3.phrase.dict.yaml` 檔頭另列：LSHK Jyutping Word List（corpus.eduhk.hk）、words.hk、開放詞典 kaifangcidian、CC-Canto、Bolton & Hutton 的 Cantonese Slang，以及「收錄搜狗、QQ、百度輸入法用戶貢獻詞庫中嘅所有粵語詞條」。
- ★ **風險點**：repo 整體標 CC BY 4.0，但上列部分來源（粵典 NC、各輸入法用戶詞庫、紙本詞典）各有自己的條款。repo 是否已取得各來源許可、`wiki` 頁（資料來源說明；WebFetch 取得 404）內容，均未查證。
- 另：LDC 目錄上有 "Rime-Cantonese: A Normalized Cantonese Jyutping Lexicon"（LDC2022L01，約 130,000 筆，授權為 LDC User Agreement），是同一團隊的 2022 年快照，與 GitHub 版授權條件不同。
- 粵拼方案：LSHK 粵拼（`lshk.org/jyutping`）。該方案本身的授權未查證。

### 6.2 粵典 words.hk（香港辭書有限公司）

- 授權頁文字（取自 `https://words.hk/base/hoifong/`，粵語原文）：《非商業開放資料授權協議 1.0》；允許「複製、修改、借用、改寫、出版、翻譯、二次創作、發佈、再分發、演出、展示、或以其他方式使用」；「除咗特別批准，你唔可以用本產品做商業用途」；商業用途包括把本作品作為增值部分提供服務、收費出版、以及對非瑣碎使用本作品的網站收廣告費；例外含另簽協議、年收入低於當地收入中位數三倍的獨資經營者；任何發佈要附英文版權告示與免責聲明、要附特別鳴謝名單、從官網取得的需連回官網、不得向他人再授權。
- AlienKevin/wordshk_app（App Store 與 Play 上架的粵典 App）：Dictionary "Copyright (C) 2014-2022 Hong Kong Lexicography Limited"，程式碼 MIT。
- **例外**：`https://words.hk/faiman/analysis/wordslist/` 提供「字 → 讀音」的 JSON / CSV，頁面標「Data License: public domain」，「Credits to words.hk appreciated」。內容僅字與讀音，不含釋義。
- 影響：開源 + 免費 + 無廣告的使用是否算「非商業」，以及上架商店本身（App Store 會收費抽成與付費下載）是否構成「商業用途」需使用者判斷 ★。

### 6.3 其他粵語資料

| 來源 | 規模 | 授權 | 備註 |
|---|---|---|---|
| HKCanCor（Luke & Wong 2015；fcbond/hkcancor） | 約 23 萬詞（bond-lab 簡報）/ 約 20 萬字（PyCantonese LRECC 2022 論文） | repo 標 CC BY；論文稱「permissive license for both commercial and non-commercial usage」 | 口語 + 廣播，1997–1998，已分詞、標粵拼與詞性；原檔 Big5-HKSCS；僅音檔樣本公開 |
| 粵語維基 zh-yue | 154,445 篇；33,659,785 字詞（CirrusSearch 統計口徑，非嚴格詞數）；dump 129 MB | CC BY-SA 4.0（MediaWiki `rightsinfo`） | 書寫粵語與標準書面語混用，需清洗 |
| CantoDict | 筆數未查證 | 頁尾 "Copyright Adam Sheik 2003-2026"；條款頁未取得 | 未見下載選項，無法確認可用 |
| 粵語審音配詞字庫（中大人文電算研究中心） | 未查證 | 未查證 | 網站服務；rime-cantonese 上游以之為字音參考來源之一；未見批量授權 |
| 粵語口語用字表 | 未查到單一權威清單 | — | rime-cantonese 的 `char.csv` / `variant.csv`（431 行）與 words.hk 的詞表可作替代 |

---

## 7. n-gram 與語言模型資料

### 7.1 語料來源與授權

| 語料 | 規模 | 授權 / 條款 | 對模型與資料散布的條款要點 |
|---|---|---|---|
| 中文維基 / 粵語維基 | 1.56M / 154k 篇 | CC BY-SA 4.0 | 署名 + 相同方式分享。SA 是否傳遞到由文本統計出的 n-gram、模型 ★ |
| OpenSubtitles2024（含 `yue`、`zh_TW`） | 26.6 GB（全語言） | 資料集標 ODC-By；"The authors do not claim ownership of the subtitle content and maintain a takedown policy for copyright holders." | 字幕原著作權人不是整理者 ★ |
| LIHKG（HF `AlienKevin/LIHKG`） | 14.4 GB，100 萬–1,000 萬筆 | 資料卡標 MIT；貼文版權與站方條款未查證；資料卡未談同意或個資 | 授權標示來自爬取者，不是論壇或發文者；LIHKG 曾封鎖腳本存取（lilm 專案 2024-06 紀錄） ★ |
| Common Crawl / FineWeb 系列 | 英文 FineWeb 為主；粵語子集未查證 | ODC-By v1.0 + 須遵守 Common Crawl Terms of Use | 網頁內容的個別著作權不因整理而消失 ★ |
| HKCanCor | 約 20–23 萬字詞 | CC BY | 太小，只適合驗證或口語詞補強 |
| Common Voice Scripted Speech 粵語（25.0，2026-03） | 文字語料 28,727 句，已驗證 18,556 句 | 標 CC0-1.0；24.0 的資料說明另寫不得轉發資料集（2026-10-31 查，摘要層級） | 條款間的關係未查清 ★ |
| Tatoeba 粵語句子 | 句數未查 | 預設 CC BY 2.0 FR，部分 CC0（2026-10-31 查） | 須署名，文字的署名寫法見其 wiki；翻譯句不能標 CC0 |
| rime-essay / essay-cantonese | 見上 | LGPL-3.0 / CC BY 4.0 | 已是頻次表，可直接當 unigram 基底 |

### 7.2 現成模型與工具

- **librime-octagram**（`github.com/lotem/librime-octagram`）：外掛本體 `LICENSE` 為 BSD 風格（"Copyright (c) 2014, RIME Developers"，GitHub 標 BSD-3-Clause）；`tools/build_grammar.cc` 為建置工具。
- **rime-octagram-data**（`github.com/lotem/rime-octagram-data`）：`LICENSE` 為 LGPL-3.0。README 免責：「配方中的文件乃以開放的互聯網語料精煉而成，只供參考之用。全程自動處理，絕無人工干預」，未給語料清單。`hant` 分支提供 `zh-hant-t-essay-bgw.gram`（41,415,724 B）與 `zh-hant-t-essay-bgc.gram`（10,318,892 B）。
- **Rime 萬象語法模型**（amzxyz）：簡體為主，約 200 MB 級（一篇 Zhihu 說明稱「万象语言模型提交较大，有200M左右」），資料 CC BY 4.0。繁體可用性未驗證。
- **KenLM**（`github.com/kpu/kenlm`）：C++ 核心；Python module 由 Victor Chahuneau 貢獻；README 稱 "ARM support is reportedly working, at least on the iphone"。GitHub 顯示 LGPL-2.1 與 GPL-3.0 兩組授權檔（`COPYING`、`COPYING.3`、`COPYING.LESSER.3`、`LICENSE`），逐檔適用範圍需看原始碼檔頭 ★。query-only 程式可不用 Boost 編譯，估計（訓練）需要 Boost。Trie 結構記憶體最省。
- 小型神經 LM 的現成繁中 / 粵語預訓練權重：這次沒有查，**未查證**。

---

## 8. 英文

| 來源 | 授權原文要點 | 規模 / 狀態 |
|---|---|---|
| SCOWL / ESDB（`en-wl/wordlist` v2） | `Copyright` 檔：「Permission to use, copy, modify, distribute, and sell any part of the English Speller Database (ESDB, previously known as SCOWLv2), or word lists created from it, is hereby granted without fee, provided that the above copyright notice appears in all copies and that both the above copyright notice and this notice appear in supporting documentation.」另：澳式英語與 `AU` region 另有聲明；生成的詞表 size >80 適用 UKACD 聲明；若散布資料庫本身或非拼字字典的衍生物，可能適用多份版權（含 WordNet 可能）。 | size 35（最小）至 85；60 為預設拼字字典；主要資訊為「詞有多常見」，不是精確頻次。資料部分來自 COCA 3-gram（作者稱符合購買時簽署的 NDA 範圍）。 |
| wordfreq | README：程式 Apache-2.0；資料可依 CC BY-SA 4.0 再散布；Google Books Ngrams 與 SUBTLEX 另有署名要求；不得轉成 CSV（理由為無處署名）。 | 40+ 語言；`zh` 含繁簡、以普通話為主，無 `yue`；資料為約 2021 年快照，作者在 SUNSET.md 說明 LLM 生成文字汙染網路語料，不再更新。 |
| Google Books Ngram | 頁面：「licensed under a Creative Commons Attribution 3.0 Unported License」 | v3（2020-02-17）、v2（2012-07-01）、v1（2009）；英文已確認，中文子集未讀到（頁面超長，只讀了前 10 萬字）。 |
| AOSP LatinIME | 倉庫 NOTICE：Apache-2.0。`dictionaries/` 有 `en_US` / `en_GB` / `en` 等 `.combined.gz`（24 種語言，無中文）。 | 字典內容的原始來源與是否另有授權：**未查證** ★。 |
| Helium314/aosp-dictionaries（社群彙整，Codeberg） | 各語言列出授權；英文「experimental」字典 main_en_US 279,960 詞、main_en_GB 386,549 詞，標 CC BY 4.0；「legacy」en_US 160,715 詞（授權欄未標）。 | 另有 emoji 字典：中文 12,267、繁中 15,680、粵語 10,015 詞，授權欄為 "UNICODE LICENSE V3 (CLDR) and AGPL-3.0 (Signal)"，含 AGPL 成分，不宜直接採用 ★。 |

---

## 9. 簡繁轉換：OpenCC

- 授權：`LICENSE` 為 Apache License 2.0；README「License」節亦寫 Apache License 2.0，另列相依：darts-clone (BSD)、marisa-trie (BSD)、tclap (MIT)、rapidjson (MIT)、Google Test (BSD)、cppjieba (MIT)。字典 `data/dictionary/*.txt` 在同一 repo、無獨立授權檔，視為同一授權（未見例外聲明）。
- 版本與維護：NEWS.md 最新已發布為 1.4.2（1.4.3 標「尚未發佈」）；repo 最後提交 2026-10-29。新增 `GetAllConversions`、`ConvertWithAmbiguities` 為公開 API，NEWS 明說目標使用者就是輸入法引擎（librime 的 Simplifier 已用）。
- 字典規模（行數）：`STPhrases` 49,309、`STCharacters` 4,029、`TSCharacters` 5,062、`TSPhrases` 505、`TWPhrases` 858、`TWVariants` 54、`HKVariants` 78、`HKVariantsPhrases` 289、`JPShinjitaiCharacters` 552、`CJK_Compatibility_Ideographs` 2,045；另有實驗性小篆表 `SealCharacters` 11,362。
- 配置：`s2t`、`t2s`、`s2tw`、`s2twp`（台灣用語）、`s2hk`、`t2hk`、`tw2s`、`hk2s` 等 JSON。
- 綁定（README「Official / Recommended Ports」與「Other Ports」，加上我們查 registry / GitHub 的結果）：

| 語言 | 實作 | 授權 | 性質與維護 |
|---|---|---|---|
| C++ | OpenCC 本體 | Apache-2.0 | 官方；跨平台建置 |
| JS（純） | `opencc-js` 1.4.2 | MIT AND Apache-2.0（npm） | README 列為官方 / 推薦 |
| JS（資料） | `opencc-data` 1.4.2 | Apache-2.0 | 官方 |
| WASM | `opencc-wasm` | 未查證 | README 列為官方 / 推薦 |
| Rust | `ferrous-opencc` 0.4.0 | Apache-2.0 | **純 Rust**；crates.io 約 166k 下載；2026-04-18 更新 |
| Rust | `opencc-rust` 2.0.2 | Apache-2.0 | 綁 C++ 庫，需先編譯 OpenCC；2026-10-10 更新；官方只支援 Linux（Windows / FreeBSD 為 best effort） |
| Rust | `opencc-sys` 0.5.2+1.4.1、`opencc-rs` 0.6.2 | 未查證 | OpenCC 綁定，更新到 2026-08 |
| Rust | `zhconv` 0.4.2 | 未查證 | 結合 MediaWiki 與 OpenCC 規則 |
| Swift / iOS | `XQS6LB3A/SwiftyOpenCC` | MIT | 內含 OpenCC submodule；最後提交 2021-04-29；無 release；Package.swift 存在 |
| Android | `qichuan/android-opencc` | MIT | JNI 包 C++，詞典放在 assets，首次轉換時複製；最後 push 2021-05-25 |
| Java | `opencc4j` | 未查證 | 非官方 |
| Go（純） | `longbridge/opencc` | 未查證 | 非官方 |
| Dart | `opencc-dart` | 未查證 | 非官方 |

- 與 QIM 的關係：是否需要簡繁轉換、轉換方向（輸入繁體、輸出簡體）屬產品決策，未在此表態。純 Rust 版（ferrous-opencc）與 Rust 核心的組合最直接；其與官方字典版本的同步程度未查證。

---

## 10. Emoji 與符號

- **CLDR annotations**（`unicode-org/cldr` `common/annotations/`）：`yue.xml`、`zh_Hant.xml`、`zh_Hant_HK.xml`、`zh.xml` 各含 3,950 個 `<annotation>` 元素（一個 emoji 有關鍵字行與 `type="tts"` 短名各一行，所以 emoji 數約為一半）。例：`😀` 在 yue 是「微笑 | 笑臉 | 臉」。
- 授權：檔頭 `SPDX-License-Identifier: Unicode-3.0`；copyright.html §3.3 說 CLDR 的軟體目錄 `/Public/cldr/` 與 `github.com/unicode-org/` 屬 Unicode License v3。注意 repo 根目錄 `LICENSE` 談的是 UTS #35 規格文字的版權（個人 / 內部使用），並非資料授權。
- Helium314 彙整的 emoji 字典混有 Signal 的 AGPL-3.0 資料（見 §8），若只取 CLDR 請直接從 CLDR 產生。
- `emoji-test.txt`（Unicode UCD 一部分）的取得與授權：Unicode License v3 涵蓋 `/Public/` Data Files，**我們沒有逐檔確認**。
- 符號：rime-prelude 的 `symbols.yaml`、rime-cantonese 的 `symbols_cantonese.yaml` 存在但授權未逐一核對，未查證。

---

## 11. 授權條款要點（只陳述，不下結論）

| 條款 | 要點 | 出現在 |
|---|---|---|
| Unicode License v3 | 授權通知須隨副本或相關文件出現；不得以版權人名義背書；AS IS | Unihan、CLDR |
| Apache-2.0 | 保留版權與授權聲明、標示修改；附專利授權 | OpenCC、AOSP、ferrous-opencc |
| MIT / MIT-like | 保留版權與許可聲明 | SCOWL / ESDB、jieba、SwiftyOpenCC |
| CC BY 4.0 / 3.0 | 署名；標示是否修改 | rime-cantonese、HKCanCor（CC BY）、Google Books Ngram、萬象詞庫 |
| CC BY-SA 4.0 | 署名；衍生作品須以相同授權散布 | zh / yue 維基、wordfreq 資料 |
| CC BY-ND 3.0 台灣 | 可重製、散布、傳輸（含商業），不得修改該著作；教育部另有使用說明 | 國語辭典 |
| Non-Commercial Open Data License 1.0 | 不得商業使用（含增值服務、廣告網站），須附英文版權告示與鳴謝名單，不得再授權 | 粵典 |
| ODbL 1.0 | 資料庫衍生需相同方式分享（share-alike）、署名 | rime-cantonese `jyut6ping3.maps` |
| ODC-By 1.0 | 署名 | FineWeb、OpenSubtitles2024（資料集） |
| LGPL-3.0 / LGPL-2.1+ | 對函式庫的修改須以 LGPL 公開；使用者須能替換 / 重新連結受 LGPL 涵蓋的部分（細節見各版全文） | rime-cangjie、rime-quick、rime-essay、octagram-data、libchewing-data、libcangjie 程式碼 |
| GPL-3.0(-or-later) | 衍生作品整體須以 GPL 公開；GPLv3 另含使用者須能安裝修改版的條款（細節見全文） | fcitx5-table-extra、rime-ice、KenLM 部分檔案 |
| DATA.GOV.HK Terms v1.2 | 商用 / 非商用皆免費；署名與承認政府 IP；賠償條款 | HKSCS 資料集 |

LGPL / GPL 資料檔在 App Store、Play Store 散布時，是否構成條款所稱的「衍生作品」、碼表以資料檔還是編譯進程式分別怎麼算、商店的 DRM 與使用者替換權是否相容，**需使用者判斷（建議徵詢法律意見）** ★。本文不下結論。

---

## 12. 建議的組合（只列選項與取捨，不拍板）

### 碼表

| 選項 | 內容 | 優點 | 取捨 |
|---|---|---|---|
| T1 | libcangjie `table.txt`（PD）為主，速成碼由倉頡碼推導 | 授權最乾淨；同時有三代、五代、簡體對應、HKSCS 旗標；Big5 排序值可作次要排序 | PD 聲明的來源鏈需使用者確認 ★；五代碼覆蓋與正確性自述「可能有錯」；需自行建置 |
| T2 | T1 + Unihan `kCangjie` 交叉驗證 | Unicode License 寬鬆；可抓出 libcangjie 與 Unihan 的差異 | Unihan 偏三代；僅驗證用途 |
| T3 | Rime cangjie5 / quick5 | 五代、擴展區覆蓋最大且有社群維護 | LGPL-3.0 + 內部授權標示不一致；《五倉世紀》權利人未查證 ★ |
| T4 | fcitx5-table-extra | 格式簡單、版本齊（三、五代，classic） | GPL-3.0-or-later，最難與寬鬆授權並存 ★ |
| T5 | 自行輸入 / 重建 | 授權完全自主 | 成本最高；難以與既有習慣一致 |

字集（過濾「常用字」與非常用字）：HKSCS-2016（data.gov.hk 條款）、教育局字形表、教育部 4,808 字，是三種事實性清單；授權多為未查證，且屬事實性清單的性質需使用者判斷 ★。

### 詞頻 / 排序

| 選項 | 內容 | 優點 | 取捨 |
|---|---|---|---|
| F1 | rime-essay（LGPL-3.0） | 規模大（44 萬筆）、已有頻次、與 Rime 生態一致 | LGPL；頻次來源是多個上游合併，細節不清 |
| F2 | libchewing-data `tsi.csv`（LGPL-2.1+） | 繁體、台灣用語、含注音（可反查） | LGPL；偏台灣用語 |
| F3 | 自行從 zh / yue 維基計算 | 授權清楚（CC BY-SA） | SA 對輸出資料的影響需判斷 ★；書面語為主；需清洗與處理 |
| F4 | 只用 Public Domain / 寬鬆資料（libcangjie 的 Big5 順序 + 自行語料） | 授權最簡單 | 排序品質差，速成重碼嚴重（最多 480 候選） |

### 粵語

| 選項 | 內容 | 優點 | 取捨 |
|---|---|---|---|
| C1 | rime-cantonese（CC BY 4.0 + ODbL maps） | 最完整（約 10 萬詞、33 萬詞組）；有詞頻 | 上游來源授權關係未查證 ★；ODbL 為 SA |
| C2 | 粵典 PD 字讀音表 + HKCanCor（CC BY）+ 自算頻次 | 授權乾淨 | 規模小；缺詞組 |
| C3 | 先不做粵語詞庫，只做繁體 | 範圍最小 | 少了粵語特色 |

### 語言模型

| 選項 | 內容 | 取捨 |
|---|---|---|
| L1 | 自行以 KenLM 或 octagram 的 `build_grammar` 工具訓練（輸入為 F3 維基或其他授權清楚的語料） | 授權鏈可控；工具授權（KenLM LGPL/GPL 混合、octagram BSD）需確認 ★；繁體 / 粵語語料量有限 |
| L2 | 直接用 octagram-data 的 `zh-hant-t-essay-bgw.gram` | 現成、41 MB；LGPL-3.0；語料來源不透明 ★ |
| L3 | 先只用 unigram + bigram 頻次（無 LM） | 最簡；效果有限 |

### 簡繁與 emoji

- 簡繁：OpenCC 資料（Apache-2.0）搭配純 Rust 或 JS 實作，或自行以 OpenCC 字典產生轉換表。
- Emoji：直接從 CLDR yue / zh_Hant_HK annotations 產生（Unicode-3.0），避免使用混合 AGPL 資料的彙整版。

---

## 13. 來源

**碼表與倉頡 / 速成**
- libcangjie（freedesktop）：https://gitlab.freedesktop.org/cangjie/libcangjie ；`data/README.table.md`、`data/table.txt`（本次 clone 讀取）
- rime-cangjie：https://github.com/rime/rime-cangjie ；LICENSE、AUTHORS、`cangjie5.dict.yaml`
- rime-quick：https://github.com/rime/rime-quick ；`quick5.schema.yaml`
- fcitx5-table-extra：https://github.com/fcitx/fcitx5-table-extra
- Unicode Unihan / UAX #38：https://www.unicode.org/reports/tr38/ ；https://www.unicode.org/Public/UCD/latest/ucd/Unihan.zip
- Unicode License v3：https://www.unicode.org/license.txt ；https://www.unicode.org/copyright.html
- Microsoft 繁體中文 IME：https://github.com/MicrosoftDocs/globalization/blob/main/globalization/input/traditional-chinese-ime.md
- Wikipedia Simplified Cangjie：https://en.wikipedia.org/wiki/Simplified_Cangjie
- 倉頡講義：https://www.pascal-man.com/navigation/faq-java-browser/cj.pdf

**字集、字頻**
- 常用字字形表（維基）：https://zh.wikipedia.org/zh-hant/常用字字形表
- 教育部 4,808 字清單（個人 gist，僅作字數佐證）：https://gist.github.com/p208p2002/69ddc0197fad375ff8c87d95beb9b59c
- HKSCS 資料集：https://data.gov.hk/en-data/dataset/hk-dpo-dpo_hp-hong-kong-supplementary-character-set-related-information
- DATA.GOV.HK 條款：https://data.gov.hk/en/terms-and-conditions
- HKSCS-2016 與 Adobe-CNS1-7：https://ccjktype.fonts.adobe.com/2017/07/hkscs2016-ac17.html
- Jun Da 字頻：https://lingua.mtsu.edu/chinese-computing/statistics/ ；Hacking Chinese 摘要：https://www.hackingchinese.com/the-most-common-chinese-words-characters-and-components-for-language-learners-and-teachers
- Sinica Corpus 4.0 簡介：https://asbc.iis.sinica.edu.tw/index_readme.htm

**詞庫**
- rime-essay：https://github.com/rime/rime-essay
- libchewing-data：https://github.com/chewing/libchewing-data
- 教育部辭典授權頁：https://depart.moe.edu.tw/ed2400/cp.aspx?n=D509C3C9F6F496FD&s=31218F193CB4FD6E
- g0v/moedict-data：https://github.com/g0v/moedict-data
- rime-ice：https://github.com/iDvel/rime-ice
- 萬象：https://github.com/amzxyz/RIME-LMDG
- jieba：https://github.com/fxsjy/jieba
- ckip-transformers：https://github.com/ckiplab/ckip-transformers

**粵語**
- rime-cantonese：https://github.com/rime/rime-cantonese ；上游 https://github.com/CanCLID/rime-cantonese-upstream
- LDC2022L01：https://catalog.ldc.upenn.edu/LDC2022L01
- 粵典授權頁：https://words.hk/base/hoifong/ ；字讀音詞表：https://words.hk/faiman/analysis/wordslist/
- wordshk_app：https://github.com/AlienKevin/wordshk_app
- HKCanCor：https://github.com/fcbond/hkcancor ；PyCantonese 論文：https://aclanthology.org/2022.lrec-1.711.pdf
- CantoDict：https://www.cantonese.sheik.co.uk/dictionary/
- zh-yue 維基：https://zh-yue.wikipedia.org/ ；dump：https://dumps.wikimedia.org/zh_yuewiki/latest/

**語料與模型**
- OpenSubtitles2024：https://huggingface.co/datasets/Helsinki-NLP/OpenSubtitles2024
- LIHKG：https://huggingface.co/datasets/AlienKevin/LIHKG ；lilm：https://github.com/alphrc/lilm
- FineWeb：https://arxiv.org/html/2406.17557v2
- librime-octagram：https://github.com/lotem/librime-octagram ；rime-octagram-data：https://github.com/lotem/rime-octagram-data
- KenLM：https://github.com/kpu/kenlm

**英文**
- SCOWL / ESDB：https://github.com/en-wl/wordlist （`Copyright` 檔取自 v2 分支）
- wordfreq：https://github.com/rspeer/wordfreq ；SUNSET：https://github.com/rspeer/wordfreq/blob/master/SUNSET.md
- Google Books Ngram：https://storage.googleapis.com/books/ngrams/books/datasetsv3.html
- AOSP LatinIME：https://android.googlesource.com/platform/packages/inputmethods/LatinIME/
- Helium314/aosp-dictionaries：https://codeberg.org/Helium314/aosp-dictionaries

**簡繁、Emoji**
- OpenCC：https://github.com/BYVoid/OpenCC
- SwiftyOpenCC：https://github.com/XQS6LB3A/SwiftyOpenCC ；android-opencc：https://github.com/qichuan/android-opencc ；opencc-rust：https://github.com/magiclen/opencc-rust
- crates.io 查詢（opencc 關鍵字）：https://crates.io/api/v1/crates?q=opencc ；npm：https://registry.npmjs.org/opencc-js
- CLDR annotations：https://github.com/unicode-org/cldr/tree/main/common/annotations

---

## 14. 未查證清單

- 倉頡三代 / 五代原始著作權與「朱邦復 Public Domain」的獨立出處
- 《五倉世紀》（chinesecj.com）權利人與授權
- Microsoft 速成碼表的再散布條款
- 真實速成產品是否有手工例外
- 教育局字形表、教育部字表（常用 / 次常用 / 罕用）的授權與電子檔
- Jun Da 字頻、Sinica Corpus 的使用條款與可下載性
- rime-cantonese 對粵典、搜狗 / QQ / 百度詞庫、CC-Canto 等上游的授權處理；`essay-cantonese.txt` 頻次來源；其 wiki 頁
- CantoDict、粵語審音配詞字庫的條款
- LIHKG 站方條款與貼文版權
- octagram `.gram` 的訓練語料
- Common Crawl 粵語子集 / FineWeb-2 的 `yue` 規模
- 現成繁中 / 粵語小型神經 LM 權重
- AOSP 英文字典來源
- Google Books Ngram 中文子集
- CKIP 模型授權
- opencc-wasm、opencc-sys、opencc-rs、zhconv、opencc4j、Go / Dart 版的授權
- ferrous-opencc 與官方字典的同步程度
- `emoji-test.txt` 的逐檔授權、符號表授權
- HKSCS-2016 的確切字數與 CCLI 頁面內容
