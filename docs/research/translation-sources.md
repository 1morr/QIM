# 譯詞的來源與授權

M4 票 20，2026-11-15 查。回答：繁體與粵語對英文的詞表，哪個能做成可選資料包（plan §4.6 的 manifest，新一種 `PackKind`）隨網頁與日後的商店 App 散布；查不到合用的，自己用 LLM 離線生成（青簡做法，`qingjian.md` §10）要花多少、生成結果是什麼授權、品質怎麼抽查。

譯詞要的是「可直接上屏的英文詞」（「發票」→ invoice），不是釋義句。鍵是 QIM 詞庫的詞（香港常見寫法），簡體模式查原文（spec〈細節〉）。

「能不能用」只照條款字面回答能否以資料包散布，不下法律結論；需要判斷的寫「需使用者判斷」。下載檔放在 session 的 scratchpad，不進 repo；數字除註明外是本機統計，腳本在 `scripts/experiments/gloss-coverage/`。

## 1. 現成的詞表

### 1.1 CC-CEDICT（MDBG）

- **授權**：CC BY-SA 4.0。下載檔檔頭逐字：「License: Creative Commons Attribution-ShareAlike 4.0 International License」「Referenced works: CEDICT - Copyright (C) 1997, 1998 Paul Andrew Denisowski」；`#! license=https://creativecommons.org/licenses/by-sa/4.0/`。[MDBG CC-CEDICT 頁](https://www.mdbg.net/chinese/dictionary?page=cc-cedict)：「you are allowed to use this data for both non-commercial and commercial purposes provided that you: mention where you got the data from (attribution) and that in case you improve / add to the data you will share these changes under the same license (share alike)」。舊的 3.0 時代與更早的 CEDICT 原版（非商業，維基百科轉述，一手文本已失傳）不用。
- **版本**：下載網址 `https://www.mdbg.net/chinese/export/cedict/cedict_1_0_ts_utf-8_mdbg.txt.gz`（另有 `.zip`）固定、內容每天更新，沒有日期化或 commit 網址；只能存一份並記檔頭的 `date=` 與 sha256。本次：`date=2026-11-14T02:25:17Z`、`entries=125413`，`.txt.gz` sha256 `82652957fa7c459ecc7b0d5e3f4cc2e3c0c96a29f950e7192baae31575469c94`。
- **格式**：一行一條 `繁 簡 [拼音] /義1/義2/…/`。英文是釋義：同一個詞多個義項、多半是片語或說明（「to eat; to consume」「CL:個|个[ge4]」「variant of …」），同一字不同讀音分成幾條、依拼音排列。
- **能不能用**：字面允許商用與再散布；署名、標明修改；改作物（抽詞、轉格式、合併）要以 BY-SA 4.0 或相容授權散布（4.0 legalcode §3(b)(1)）。

### 1.2 CC-Canto（Pleco，cantonese.org）

- **授權**：CC BY-SA 3.0（事實 T14；`cccanto-170202.zip` 檔頭「Distributed under a Creative Commons Attribution-Share Alike 3.0 license」，sha256 `0cc9e600a19c02405064d6d227d96563c09da1a0af1c7717508c0f231028af27`）。格式是 CC-CEDICT 式加粵拼，英文同樣是釋義。
- **和 CC-CEDICT 合成一包**：BY-SA 3.0 legalcode §4(b) 允許改作物以「(ii) a later version of this License with the same License Elements as this License」散布；[CC 的相容授權頁](https://creativecommons.org/share-your-work/licensing-considerations/compatible-licenses/)：「Your contributions to adaptations of BY-SA 3.0 materials may only be licensed under: BY-SA 3.0, or a later version of the BY-SA license.」所以兩份合併、去重、抽詞後整包用 BY-SA 4.0 可以，NOTICE 寫明 CC-Canto 那部分原為 3.0、依 §4(b)(ii) 改以 4.0 散布；反過來 4.0 不能退回 3.0。
- **能不能用**：同 §1.1，share-alike。

### 1.3 粵典

- 詞典本體（含英文釋義）：《非商業開放資料授權協議 1.0》、沒有全量下載（事實 T13）。
- 「細數據」頁另有 [`englishindex`](https://words.hk/faiman/analysis/)，標「Data License: public domain. Credits to words.hk appreciated.」頁面說它是搜尋用的索引：「built by mapping the words seen in each entry's English explanation to its Cantonese word」，英文詞正規化成美式拼法、`!` 開頭的是 Porter 詞幹，分數是 tf-idf、只留 > 40。實測 40,839 個英文鍵、值是 `詞:粵拼` 加分數。
- 方向是英→粵，英文是釋義裡的字或詞幹，不是譯詞（`invoice` 底下有「開單」，沒有「發票」）；又是從非商業的釋義反推的，和本體授權的關係頁面沒說明（同 T13 的 `wordslist`）。**不適合**。

### 1.4 其他

| 來源 | 授權（原文連結） | 能不能用 | 為什麼不當主要來源 |
| --- | --- | --- | --- |
| Unihan `kDefinition`（18.0.0，23,310 字） | Unicode-3.0（事實 T5；[UAX #38](https://www.unicode.org/reports/tr38/)：「An English definition for this ideograph」，Provisional） | 可，附聲明，無 share-alike；可釘 `Public/18.0.0/ucd/Unihan.zip` | 只有單字、是釋義（「票」= slip of paper or bamboo; ticket） |
| Wikidata 標籤 | CC0（[Wikidata:Copyright](https://www.wikidata.org/wiki/Wikidata:Copyright)：「All structured data from the main, Property, Lexeme, and EntitySchema namespaces is available under the Creative Commons CC0 License」） | 可；dump 可釘日期 `entities/<日期>/wikidata-<日期>-all.json.bz2`（約 104 GB） | 以人名、地名、機構等實體為主，日常詞少；`zh-hk`／`yue` 對 `en` 的配對數沒量 |
| 樂詞網雙語詞彙（國家教育研究院） | [政府網站資料開放宣告](https://terms.naer.edu.tw/mysite/about/2/)：「重製、改作、編輯、公開傳輸…開發各種產品或服務…然使用時，應註明出處」，另有「不得惡意變更」與第三方著作例外 | 字面可，無 share-alike；檔無版本號（`.ods`） | 台灣用語、專業名詞為主（雙語詞彙約 2 萬則，二手數字） |
| Cantonese Wordnet（[lmorgadodacosta/CantoneseWN](https://github.com/lmorgadodacosta/CantoneseWN)） | CC BY 4.0（README） | 可，無 share-alike；無 tag，可釘 commit | 繁體＋粵拼、約 1.6 萬 lemma、5,188 個 synset；英文要經 Princeton WordNet／OEWN（CC BY 4.0 ＋ WordNet License）的 synset 取，拿到的是 synset 的第一個 lemma，不一定是日常的譯詞 |
| Chinese Open Wordnet 1.1（[bond-lab/cow](https://github.com/bond-lab/cow)） | WordNet 式寬鬆授權（`docs/LICENSE`） | 字面可，但 2013 論文寫早期混入研究授權的 SEW 與 BY-SA 的 Wiktionary，來源待確認 | 簡體 |
| Wiktionary | CC BY-SA 4.0／GFDL（`cantonese-lexicon.md` §4） | share-alike，同 §1.1 | 要自己從 wiktextract 抽，釋義同樣是句子 |
| ECDICT | MIT，但 README 寫詞條來自來源不明的 EDictAZ.txt、爬蟲、網友貢獻 | 不建議 | 英→簡中，方向與字體都不對 |
| MUSE zh-en | CC BY-NC 4.0（repo `LICENSE`） | **不可**（非商業） | 簡體、內部翻譯工具產生 |
| 香港 DoJ 法律詞彙 | 版權政策原文沒取到；data.gov.hk 條款要署名、有彌償條款（事實 T24） | 未查清 | 法律領域 |

## 2. CC-CEDICT 能蓋多少

`scripts/experiments/gloss-coverage/coverage.py`：拿 rime-essay `054920d` 詞頻 ≥ 100 的兩字以上詞（詞庫門檻，251,622 詞，依詞頻排）去查，兩邊都把 HKVariants 與 QIM 的並存寫法折成同一寫法（近似詞庫的寫法處理）。「可直接上屏」：跳過 `CL:`、`variant of`、`see`、`surname` 這類義項，取第一個義項分號、逗號前那段，去括號與開頭的「to 」，剩 1–3 個英文詞。

| 範圍 | CC-CEDICT 有條目 | 可直接上屏 | 加 CC-Canto 後可直接上屏 |
| --- | --- | --- | --- |
| 詞前 5,000 | 71.3% | 68.4% | 70.7% |
| 詞前 20,000 | 61.5% | 57.7% | 59.5% |
| 詞前 50,000 | 55.0% | 50.1% | 52.2% |
| 全部 251,622 詞 | 37.6% | 30.5% | 32.3% |
| 粵語口語小詞表（196 詞） | 3.1% | 1.0% | 67.3% |
| 粵典詞頭、essay 沒有的（19,808 詞） | — | — | 29.6% |
| 單字前 3,000 | 99.8% | 88.5% | — |

- **單字**：條目都有，但讀音依拼音排、義項不依常用度，取第一個常是冷門義：「說」= persuade、「就」= in that case、「吧」= bar；要做單字得另外挑義項，規則挑不準。
- **抽查**（種子 2020，可直接上屏的詞，前 5,000 與 5,000–20,000 各抽 50，Agent 一人判，只看量級）：明顯錯 8 條（態度 = manner、做出 = put out、有一些 = somewhat、執業 = practitioner、大戶 = great family、培育 = train、前臺 = stage、填滿 = cram）；意思對但不像會上屏的 12 條（受傷 = sustain injuries、廣大 = vast or extensive、乘以 = multiplied with、領導 = lead…）。第一個義項常不是現代、香港最常用的那個，CC-CEDICT 的義項順序本來就不代表常用度。
- 合起來：詞前 2 萬約 59.5% × 80% ≈ 48% 有合用的譯詞；粵語口語詞靠 CC-Canto 才有，且 CC-Canto 偏書面（事實 T14）。

## 3. 自己用 LLM 離線生成

### 3.1 費用

定價 2026-11-15 查官方頁（[Anthropic](https://platform.claude.com/docs/en/about-claude/pricing)、[OpenAI](https://developers.openai.com/api/docs/pricing)、[DeepSeek](https://api-docs.deepseek.com/quick_start/pricing)、[Gemini](https://aistudio.google.com/docs/pricing)），單位：美元／百萬 token。

| 模型 | input | output | Batch（input／output） |
| --- | --- | --- | --- |
| Claude Haiku 5.5 | 0.10 | 0.50 | 0.05／0.25 |
| Claude Sonnet 5.5 | 2 | 10 | 1／5 |
| OpenAI gpt-6-luna | 0.10 | 0.50 | 0.05／0.25 |
| OpenAI gpt-6.1-sol | 2 | 10 | 1／5 |
| DeepSeek deepseek-flash | 0.30（離峰 0.15） | 1.20（離峰 0.60） | 沒有 Batch API，離峰半價 |
| Gemini 3.8 Flash | 0.75（2027 起 1.50） | 3.75（2027 起 7.50） | 半價 |

估算假設：一個請求 100 詞、system prompt 600 token 走快取、每詞 input 6 token、output 12 token（一行 JSON，可能偏低；改成 18 token 時費用約 ×1.45）；關掉 thinking（否則 output 多數倍）。Anthropic 的新 tokenizer 官方說同樣文字約多 30% token，繁中的實際 token 數沒量。

| 模型（Batch） | 5 萬詞 | 10 萬詞 | 25 萬詞 |
| --- | --- | --- | --- |
| Claude Haiku 5.5／gpt-6-luna | $0.17 | $0.33 | $0.83 |
| DeepSeek flash（離峰，非 Batch） | $0.41 | $0.81 | $2.03 |
| Claude Sonnet 5.5／gpt-6.1-sol | $3.31 | $6.63 | $16.57 |
| 旗艦（gpt-6-astra） | $16.65 | $33.30 | $83.25 |

另用強模型（Sonnet 5.5 Batch）裁判 2% 樣本：25 萬詞也不到 $0.5。費用不是決定因素，品質才是；幾個模型各跑一遍互相比對也只是乘幾倍。

### 3.2 生成結果的授權

- 四家條款都把輸出給使用者、都只禁止拿輸出訓練競爭的模型，做詞表不在其列：
  - [Anthropic Commercial Terms](https://www.anthropic.com/legal/commercial-terms)：「Customer (a) retains all rights to its Inputs, and (b) owns its Outputs … Anthropic hereby assigns to Customer its right, title and interest (if any) in and to Outputs.」
  - [OpenAI Services Agreement](https://openai.com/policies/services-agreement) 4.1：「Customer … (b) owns all Output. OpenAI hereby assigns to Customer all OpenAI's right, title, and interest, if any, in and to Output.」
  - [DeepSeek Terms of Use](https://cdn.deepseek.com/policies/en-US/deepseek-terms-of-use.html) 4.2 轉讓輸出權利；3.1 另要求公開散布時「clearly indicate that the output content is generated by artificial intelligence」。
  - [Gemini API Terms](https://ai.google.dev/gemini-api/terms)：「Google won't claim ownership over that content」，沒有轉讓字句。
- 美國著作權局 [Copyright and Artificial Intelligence, Part 2](https://www.copyright.gov/ai/Copyright-and-Artificial-Intelligence-Part-2-Copyrightability-Report.pdf)：「Copyright does not extend to purely AI-generated material」「prompts alone do not provide sufficient human control to make users of an AI system the authors of the output」。純生成的譯詞多半沒有著作權，以 CC0 釋出不矛盾。
- 要注意的兩點（需使用者判斷）：
  1. LLM 可能背出 CC-CEDICT、Wiktionary 的釋義。單詞對照（發票 = invoice）多半是事實；但若整串釋義照抄，就有 share-alike 的疑慮。抽查時順便比對 CC-CEDICT 的原句（只在本機比，不進資料包）。
  2. 鍵（詞）取自詞庫，詞庫的詞來自 LGPL-3.0 的 rime-essay（事實 T10）；譯詞包若帶整份鍵，就同時帶這份詞表。這和詞庫資料包是同一個問題（brief §5.1、§11 已列給使用者判斷）。

### 3.3 品質抽查

成熟做法：青簡的德語冠詞對照 Wiktionary 快照，一致率 98.4%；西語用機器翻譯，已知 13% 名詞仍是複數（`qingjian.md` §10）：能機械檢查的屬性要全量查。雙語詞典歸納的評估慣例是 Precision@1、多個正解任一命中即對（[Glavaš 等，EMNLP 2023 沿用 Lample 等 2018](https://aclanthology.org/2023.emnlp-main.595.pdf)）。LLM 當裁判與人的一致率約八成（[Zheng 等 2023，MT-Bench](https://arxiv.org/abs/2306.05685)，那是偏好比較，不是詞對照，所以裁判要先跟人校準）。

QIM 可用的方案：

1. **全量機械檢查**：1–3 個譯詞、ASCII、不照抄原詞、沒有空值、沒有「to 」開頭與分號。
2. **兩個模型交叉**：另一家的模型再生成一遍，第一個譯詞不同的詞全部交給強模型或人複核（比隨機抽樣更能抓錯）。
3. **分層抽樣**：詞頻前 1 萬、1 萬–5 萬、其餘、粵語口語詞四層，各抽 400；強模型當裁判（可用／可用但不自然／錯），另抽 100 詞由使用者盲評，量裁判與人的一致率。
4. **門檻**（建議，不是業界標準）：每層「第一個譯詞可用」的 Wilson 95% 下界 ≥ 95%（n=400 時觀察值約 97%），粵語口語層 ≥ 90%；沒過的那層換模型或加範例重跑再抽。
5. **紀錄**：模型與版本、prompt、日期、抽樣種子與判定寫進 `docs/research/`（凍結紀錄），NOTICE 標明 AI 生成。

## 4. 結論

| 方案 | 授權 | 費用 | 覆蓋與品質 |
| --- | --- | --- | --- |
| A. CC-CEDICT ＋ CC-Canto | 整包 CC BY-SA 4.0（share-alike），和詞庫資料包的語料字頻同一類 | 0 | 詞前 2 萬約 48% 有合用的譯詞；單字要另挑義項；粵語口語靠偏書面的 CC-Canto |
| B. LLM 離線生成 | 輸出歸 QIM，可標 CC0；鍵的授權同詞庫 | 25 萬詞 Batch：Haiku 約 $1、Sonnet 約 $17；裁判不到 $0.5 | 整份詞庫都有；品質要照 §3.3 量，沒有公開數字 |
| C. A 為主、缺的用 B 補 | 兩種授權在同一包，整包仍要 BY-SA | 同 B 但量少 | 留著 A 的錯義項 |

建議 B：A 的瓶頸在 CC-CEDICT 的格式（釋義句、義項不依常用度），一半以上的常用詞拿不到合用的譯詞，修這個要的人工比生成加抽查多；B 的費用即使用 Sonnet 級模型跑完整份詞庫也在 $20 內。要花錢，**由使用者決定**（spec〈共同規則〉）。

## 5. 未查證

- 樂詞網雙語詞彙各檔的筆數與欄位；樂詞網是否正式採用政府資料開放授權條款。
- Wikidata 有 `zh-hk`／`zh-hant`／`yue` 又有 `en` 標籤的 item 數。
- Cantonese Wordnet 的 commit、synset 對英文 lemma 的可用率。
- 粵典 `englishindex` 與詞典本體授權的關係（同 T13，沒去信）。
- OpenAI Services Agreement 禁止開發競爭模型那一條的全文（頁面截斷）；著作權局報告的發布日期。
- 各家 tokenizer 對繁中詞的實際 token 數；LLM 對粵語口語詞的正確率（沒有公開數字，要照 §3.3 自己量）。
