# QIM 實作計劃

> 狀態：M0–M4 完成（`.scratch/m0/spec.md`–`.scratch/m4/spec.md`，封存 `docs/archive/M0.md`–`docs/archive/M4.md`）；M5 未開工（排在 gates、flow-sync、arch-diet 之後）；還沒開工的里程碑只有目標、範圍與完成標準。本檔只寫現況：決定改了就直接改寫，不留「原寫」「取代原」「某日改」；舊版與理由記在 progress〈偏差與決定〉，影響架構的寫 ADR。里程碑開工時，它在〈里程碑任務〉的那一節細化成 `.scratch/<里程碑>/spec.md`，這裡縮成一行連結；拆票後以票為準。需求與決定見 `docs/design-brief.md`，名詞見 `GLOSSARY.md`。

## 0. 工程慣例

已搬到 `ARCHITECTURE.md`〈識別符〉與〈主要依賴〉。

## 1. 架構總覽

已搬到 `ARCHITECTURE.md`。

### 1.1 執行形態

已搬到 `ARCHITECTURE.md`〈執行形態〉。

### 1.2 目錄結構

已搬到 `ARCHITECTURE.md`〈模組地圖〉。

### 1.3 依賴規則【決定】

已搬到 `ARCHITECTURE.md`〈依賴規則〉；鍵盤擴充不做 Schema 驗證的打算搬到 roadmap〈平台里程碑（M7–M11）〉的 M8。

### 1.4 主要依賴

已搬到 `ARCHITECTURE.md`〈主要依賴〉。

## 2. 核心

已搬到 `ARCHITECTURE.md`〈跨模組的資料與契約〉；單一行為由 `crates/qim-core/tests/` 依功能分檔說明，取捨的理由在 ADR 草稿（`.scratch/arch-diet/adr-drafts/`，票 06 編號）。逐段去處見 `.scratch/arch-diet/issues/02-core.md` 的對照表。

### 2.1 介面

已搬到 `ARCHITECTURE.md`〈核心對殼〉；前文、欄位類型、聯想、重排、按鍵機率的理由在 ADR 草稿。

### 2.2 資料流

已刪：由 `crates/qim-core/src/lattice.rs`、`session.rs` 說明；已知限制（簡繁逐段轉換）搬到 roadmap〈不排里程碑〉。

### 2.3 排序：同一把尺

已搬到 `ARCHITECTURE.md`〈候選的排位順序〉；參數的取捨在 ADR 草稿 `ranking-parameters.md`。

### 2.4 學習與排位（brief §4.5）

已搬到 `ARCHITECTURE.md`〈候選的排位順序〉〈學習資料的格式〉；格式、時限、常用語的理由在 ADR 草稿。

### 2.5 輸入方案（brief §6）

已搬到 ADR 草稿 `new-schemes-share-models.md`；方案檔的欄位由 `crates/qim-core/src/scheme.rs` 與 `data/schemes/quick.toml` 說明。

### 2.6 佈局：畫布模型（ADR 0004）

已搬到 `ARCHITECTURE.md`〈佈局格式〉；格式由 `crates/qim-layout/schema/qim-layout-1.schema.json` 與 `crates/qim-layout/tests/` 說明。

### 2.7 模型分層（ADR 0005）

已搬到 ADR 0005、`docs/research/model-ladder.md` 與 `docs/development.md`〈停鍵層的模型〉；殼的重排契約在 `ARCHITECTURE.md`〈核心對殼〉。

## 3. 平台殼

已搬到 `ARCHITECTURE.md`〈平台殼〉；只和某個平台有關的打算搬到 roadmap〈平台里程碑（M7–M11）〉。

### 3.1 網頁 prototype（M0–M4）

各殼都要守的搬到 `ARCHITECTURE.md`〈平台殼〉，在手機上試的步驟搬到 `docs/development.md`〈網頁〉；網頁自己的行為由 `web/src/` 與 `web/e2e/` 說明。

### 3.2 macOS（M7）

已搬到 roadmap〈平台里程碑（M7–M11）〉的 M7。

### 3.3 之後的平台

已搬到 roadmap〈平台里程碑（M7–M11）〉。

## 4. 碼表與資料管線

`data/sources.toml` 列出每個來源：URL、釘住的版本（git 來源填 commit）與每個檔的 sha256、授權、取用的檔案。`cargo xtask data` 依序執行下列步驟，產物放 `build/packs/`，每個資料包一個目錄，旁邊放它的 `LICENSE` 與 `NOTICE`（brief §5.1）；不同資料包不共用授權檔，一個資料包用到幾個來源，`LICENSE` 就逐一寫進各來源的授權（照抄上游聲明、上游釘住的授權檔整份，或上游授權檔沒有版本化網址時用 `data/licenses/` 裡存的全文）、`NOTICE` 逐一列出來源與取用的內容。

資料包是 postcard 整份解開進記憶體，不改 mmap 友善格式（M2 票 09）：iPhone 18 Pro Max 的鍵盤擴充上限 177 MB，六份資料包加 `Engine` 載入後 79 MB（詞庫 41.5 MB、字根提示 14.3 MB），放進 M3 的模型（roadmap〈M3 的模型階梯實驗〉）後仍在記憶體警告線（約 142 MB）以下。低階機的上限量出來放不下時，先把詞庫改成 mmap 友善的格式（例如 `fst` 存碼、詞放成一整塊字串加位移索引），見 §8 與 `docs/research/ios-feasibility.md`〈記憶體上限〉。

### 4.1 速成碼表

1. 取 libcangjie `data/table.txt`（freedesktop GitLab，`346026b`）。
2. 只取 Big5 或 HKSCS 旗標為 1 的漢字當連打字集；其他漢字標成擴展區（只在單字查詢時出現）。
3. 由倉頡三代與五代碼推導速成碼，取聯集（brief §4.2）。
4. 字的排序先用字頻；再來是 Unihan `kGradeLevel`，有年級的字依年級排前面；都沒有的退回 libcangjie 的 Big5 排序值。資料包裡的碼表只排到年級 → Big5，字頻在詞庫資料包（§4.2），`Engine` 載入兩份資料包時才依字頻重排：速成資料包不含語料衍生的資料，授權只有 libcangjie、Unihan 與判定簡體字用的 OpenCC（M1 票 06、M3 票 19）。
   - 簡體字（brief §4.1，M3 票 19，`simplified.rs`）：碼表的每個字多一個標記，不影響字序；繁體模式排在後面是核心的事（§2.3）。只標連打字集裡 HKSCS 旗標為 1、Big5 不是的字，再要 OpenCC（`fae2a0a`，同簡繁資料包）認定它只是簡體寫法：`STCharacters` 簡轉繁一定換掉它（值裡沒有它自己：「叶 → 葉 叶」不算）、`TSCharacters` 不把它對到自己（「麽」不算）、不是 `HKVariants` 的香港寫法（温、户、囱）、不是香港《常用字字形表》的標準字形（Unihan `kHKGlyph` 有它，`STCharacters` 給的繁體寫法卻沒有一個和它同一個序號：够；和繁體寫法同序號的叙、携、烟是同一條列的異體，照算）、不是前處理保留的粵語用字（`CANTONESE_KEPT` 的吓、响）。現在 278 字，全表在票 19 Comments。只看「OpenCC 簡體那邊有它」不行：台（臺）、后（後）、里（裡）這類正規繁體字也在簡體那邊，Big5 就有 59 字 `STCharacters` 一律換掉（峰、床、群）；Big5 的字一律不標。
5. 另產一份字根提示資料包（字 → 倉頡三代 / 五代碼，id `cangjie-hint`），只給字根提示用：每個漢字合併各列的碼、各代依列序去重；授權只有 libcangjie。核心的 `RadicalHints` 不在 `Engine` 裡，倉頡碼結構上進不了 lattice（ADR 0003）；殼查候選每個字的碼，用方案的 `[radicals]` 換成字根顯示（M1 票 22）。

### 4.2 詞庫

- rime-essay（`054920d`）：只收兩字以上的詞；先設最低詞頻門檻，初值 100，與 rime-quick 舊版相同，由基準調整；再丟掉含連打字集以外字的詞；同一個詞列了幾次取最大詞頻。詞的寫法和語料一致（brief §4.1，M1 票 17c）：rime-essay 是 OpenCC 標準寫法（「羣組」「觀衆」「喫飯」），每個詞照粵語語料的轉法經 `t2hk` 再套字形對照表（§4.3），門檻與字集看換完的詞；口語詞表也經同一套轉換。換完撞成同一個詞的合併：詞頻相同的是 essay 把同一份計數列在兩種寫法下（「巔峯／巔峰」都是 9448，14 組），只算一次；不同的是各自的計數（「才是／纔是」，261 組），相加（M1 票 17d）。並存寫法（§4.3）的詞每種寫法都收（「這裏」「這裡」、「隨着」「隨著」、「臺灣」「台灣」各在自己的碼上），碼不同，不互搶：雙向的詞頻相同；單向的只從前一個字生出後一個字的寫法，生出的詞撞上 essay 本來就有的詞時照上面的規則合併（「刻着」與 essay 的「刻著」是同一個詞），「著名」不生出「着名」、「台州」不生出「臺州」。生出的寫法是別的詞時不生出、也不併（`ONE_WAY_DISTINCT`：「合着」≠「合著」合寫的書、「所着」穿的 ≠「所著」寫的、「舊着」≠「舊著」舊作；M1 票 17d 查 essay 會被併的「X着」7 個，「顯着」是錯字照票 17c 併進「顯著」，其餘「刻着／張着／親着」同義照併）。已知限制（使用者決定詞頻共用，2026-11-05，M1 票 17c）：另一種寫法生出的詞和同碼的詞搶，例如「到底幾時」打成「到底綫時」（「底綫」共用「底線」的詞頻 6616，壓過「到底｜幾時」）；不修，靠學習分。改成照字頻比例打折時，打少見寫法的碼反而出不了詞（「開啓」→「開船」），沒採用。每個字的速成碼展開成詞的碼，一個詞最多 8 個組合，依換掉主碼（轉碼用的五代第一碼）的字數由少到多取（M1 票 06：門檻以上的詞 99.4% 不超過 4 個組合）。詞庫是獨立資料包（方案檔 `[packs] lexicon`），`LICENSE` 寫 LGPL-3.0 全文與字頻用到的語料授權。
- 粵語口語詞表：`data/lexicon/yue-colloquial.tsv`，CC0，自行整理（不從 rime-cantonese、粵典複製），使用者審過才進資料包（M1 票 17：197 詞，2026-11-04 審過；票 17c 拿掉「車仔麵」：essay 的「車仔麪」換成常見寫法後已收，現在 196 詞）。格式同 `essay.txt`，一行「詞<TAB>詞頻」，`#` 開頭是註解；詞頻同 rime-essay 的尺度，分 3000、1000、300 三級。只收 essay 沒有或低於門檻的兩字以上口語詞，essay 低於門檻的詞改用詞表的詞頻。詞表是人工整理的，單字、詞頻低於門檻、含連打字集以外的字、重複、essay 已收的詞都讓建置失敗，不默默濾掉。授權在詞庫資料包的 `LICENSE`、`NOTICE` 另列一段（CC0 1.0，`NOTICE` 記檔案 sha256）。
- HKCanCor 的詞（M3 票 18）：HKCanCor 已分詞，取 train 切分裡兩字以上的詞與次數（`hkcancor.rs` 的 `TrainWords`）。切句、切分與丟句同語料管線、看轉換後的話輪，所以基準題（test）與 dev 句子裡的詞不數；人名（詞性 `nr`）不收；轉換改了字數的話輪對不上詞的位置，整句略過（現在 0 個）。詞照語料的轉換寫，再經同一套寫法轉換；只補 essay（達門檻）與口語詞表都沒收、出現 ≥ 2 次（`CORPUS_MIN_COUNT`）的詞，詞頻是次數乘 300（`CORPUS_WORD_FREQUENCY`：兩邊都有的詞，essay 詞頻是次數的 310–3,770 倍（四分位），取下四分位；dev 掃 300、1,000、3,000 差不多，1,000 時只出現 3–5 次的詞壓過 essay 詞頻一兩千的常用詞，個人集多錯 4 題），essay 低於門檻的改用這個詞頻；機器抽的，含連打字集以外的字的詞默默略過。現在補 383 詞。首選字基準的 hkcancor 集與這份詞表同源（題目是 test 切分、詞取 train，但同一批對話、用詞接近），增益以無關的 common-voice-yue 為準（修正 3964 → 3906，票 18）。授權不另加：HKCanCor 的字頻本來就在詞庫資料包，`NOTICE` 另寫取了詞。rime-cantonese 授權鏈不明，不用；粵典只進下一條的可選詞庫（`docs/research/facts-tables.md`）。
- 粵典的詞頭（M3 票 24）：另建一份可選的詞庫資料包 `quick-lexicon-words-hk`，是預設詞庫的詞再補粵典的詞頭，預設詞庫不收粵典的詞；網頁內建兩份、預設用 `quick-lexicon`，設定區切換（§4.6，不改核心）。來源是粵典「細數據」頁的 `wordslist`（詞頭）與 `existingwordcount`（詞在粵文庫的次數），`words_hk.rs` 讀：去掉結尾的 `*`（含異體字的記號）、去重，單字不收。網址沒有版本、內容隨詞典更新：`sources.toml` 的 version 填下載日期、sha256 釘那天的內容，上游一改，沒有快取的建置 sha256 就對不上：這時（或下載失敗）只略過粵典詞庫、刪掉上次建出的那份，其他資料包照建、結束碼仍是 0，警告與重釘步驟印在 stderr，`--verify` 列出略過的那份、不算失敗；網頁把它當可以缺的內建資料包，沒有時照樣載入（`web/src/packs.ts` 的 `OPTIONAL_IDS`）。可選資料包的來源不能擋預設資料包；不改用 Wayback Machine 的存檔（最新只到 2025-04，存新的要在別人的網站留紀錄）。詞照同一套寫法轉換；只補前面各條都沒收的詞，而且詞的任一種並存寫法已收就算收了（粵典的「開臺」不收：它生出的「開台」essay 已有，兩者撞成同一個詞時詞頻會相加、改到預設詞庫的詞；HKCanCor 的詞不這樣比：撞上的只有台北、台灣、電視台，改了 zh-wiki 多錯 1 題、common-voice-yue 修正多 1，預設詞庫維持原樣）；含連打字集以外的字（英文、數字、標點）的默默略過。詞頻是粵文庫次數乘 50（`HEADWORD_FREQUENCY`），至少門檻 100（粵文庫沒見過的詞頭也收）；dev 切分掃 0、25、50、100、200、400 與上限 300／1,000／3,000，50、不設上限最好，見票 24 Comments。`existingwordcount` 是沒分詞的原始出現次數，短詞偏高，只當粗略排序。現在補 20,556 詞（兩字以上的詞頭 57,860 個，含英文、數字或標點的 1,819 個略過；`docs/research/facts-tables.md` 的 56,037 只數全是漢字的）。授權照較嚴的非商業條款（`docs/research/facts-tables.md`）：`LICENSE` 多一段 `data/licenses/words-hk-NCODL-1.0.txt`（授權頁的英文版權告示、免責聲明、授權全文與〈關於我哋〉的特別鳴謝，授權頁沒有版本化網址，全文存 repo），`NOTICE` 寫官網連結、§1(b)(iii) 的單一網址告示（兩個檔各用自己的網址）與「粵典的部分不得再授權、不得商業使用，商用改用 `quick-lexicon`」；manifest 的 `license` 多 `LicenseRef-words-hk-NCODL-1.0`。首選字基準用 `cargo xtask bench --lexicon quick-lexicon-words-hk` 換詞庫（`docs/development.md`〈首選字基準〉）。
- 字頻：由 §4.3 各語料的 train 切分統計（`build/corpus/<名>/chars.tsv`，M1 票 03），與詞頻放同一個資料包；各語料的次數乘上和 bigram 同一個語料權重再相加（§4.3，M1 票 12）。雙向並存寫法的兩個字共用兩者的和。

### 4.3 字級 bigram 與詞級 trigram

- 語料：中文維基與粵語維基。用 Wikimedia 的 `pages-articles` dump 自己抽內文（`qim-databuild` 的 `wikitext` 模組：模板、表格、註腳、檔案連結、標題與列表整個丟掉；繁簡轉換標記取繁體那個變體），dump 日期與 sha256 釘在 `sources.toml`。CirrusSearch 內容 dump 雖是純文字，但只留最近約 5 週，釘了很快就下載不到（`docs/research/facts-tables.md`）。
- 中文維基只取 Internet Archive 存檔的 `zhwiki-20220501` 第一個分片（page id 1–187712，約 204 MB），分片內每篇都用：dumps.wikimedia.org 對大站只留約 7 個月，archive.org 是唯一能長期重現的取法（`docs/research/facts-tables.md`）。
- 口語語料：維基偏書面語，另加 HKCanCor 與 Common Voice 的粵語句子（GitHub 上 Sentence Collector 的句子檔，CC0；brief §5.2）。量遠小於維基，訓練時乘上權重，權重由口語集的開發切分調出。Tatoeba 沒有版本化的下載、要逐句署名，M1 不用；LIHKG、OpenSubtitles 權利來源不清，不用。
- 前處理：轉成香港繁體 → 在標點、空白、符號處切句 → 含連打字集以外字元（英文字母、數字、擴展區的字）的句子整句丟掉，不拿掉個別字，免得拼出原文沒有的相鄰字。
- 轉香港繁體（M1 票 17b，`corpora.rs` 的 `Conversion`）：中文維基混有簡體，用 OpenCC `s2hk`；粵語維基、HKCanCor、Common Voice 原文已是繁體，只用 `t2hk` 換香港寫法。`s2hk` 把繁體裡也是簡體的字再轉一次（「晒」→「曬」、「吓」→「嚇」、「郁」→「鬱」、「布」→「佈」），粵語維基換錯約 6.8 萬字；`t2hk` 不碰這些字，但留下粵語維基約 0.005% 的簡體字。字形用常見寫法，不用香港寫法（brief §4.1，使用者決定）：兩種轉完都照字形對照表（`common_forms.rs`）換回常見寫法。對照表用轉語料的同一套 OpenCC（ferrous-opencc 內建的字典）逐字查碼表裡的每個字：`t2hk` 給香港寫法，反過來是香港 → 常見（「説→說」「温→溫」「揾→搵」）；`t2tw` 給 OpenCC 標準 → 台灣常見（「羣→群」「峯→峰」，`s2hk` 的簡轉繁出的是前者）。不用釘住的 OpenCC `fae2a0a` 字典：它和 ferrous-opencc 內建的版本不同，查出的表會和實際轉換對不上；只收常見寫法在連打字集、而且香港寫法不在連打字集（不換回來，含它的句子整句丟掉）或兩字同碼的組，現在 34 組。不同碼的不收進對照表，免得把「吃」換成「喫」、「才」換成「纔」；但 `t2hk` 自己會把不同碼的字換掉（「臺→台」「菸→煙」「鉤→鈎」「溼→濕」，essay 與語料都是），其中兩種都常見的另列為並存寫法（下一點：雙向的「户」換成代表寫法「戶」，單向的「臺」「菸」照原文放回）。essay 被 `t2hk` 換掉、不同碼的字全表在票 17d Comments（`GAPS=1` 跑 `scripts/experiments/lexicon-forms/`）。統計見票 17b Comments，實驗在 `scripts/experiments/s2hk-diff/`。
- 並存寫法（M1 票 17c，使用者 2026-11-05 決定，`common_forms.rs`）：不同碼、兩種寫法都常見的字兩種都收、打哪個碼出哪個。不改核心、不改資料包格式，照抄在建置端做。實驗在 `scripts/experiments/lexicon-forms/`。
  - 雙向（`TWO_WAY_FORMS`，兩種寫法同義）17 組：線綫、啟啓、麵麪、床牀、戶户、敘敍、臥卧、裏裡、污汙、泄洩、枱檯（票 17c），鈎鉤、濕溼、抬擡、粽糉、枴柺、囱囪（票 17d，`t2hk` 換掉 essay 的寫法，代表寫法是它換成的那個；「囪」使用者 2026-11-05 決定），前一個是代表寫法。計數歸到代表寫法：前處理照對照表把另一種換成代表寫法（基準題目也寫代表寫法），字頻、詞頻與 bigram 都在代表寫法上數；建資料包時把代表寫法的 unigram、含它的 bigram（前後字各換或不換）與詞照抄給另一種寫法，兩種寫法的機率相同。bigram 另抄 10,121 對（剪枝後的 0.7%，票 17d 後）。
  - 單向（`ONE_WAY_FORMS`）：「着→著」「臺→台」「菸→煙」。「着」的用法都能寫成「著」，「著」的 zhù 義項（著名、著作、土著，rime-essay 有 103 詞）不能寫成「着」；「臺灣」「香菸」可寫成「台灣」「香煙」，「台州」「煙霧」不能寫成「臺州」「菸霧」（票 17d）。只在詞庫照抄（「隨着」另收「隨著」、「臺灣」另收「台灣」）；語料照原文，`t2hk` 換掉的「臺」「菸」前處理放回去（`BothForms::as_written`），兩字的字頻與 bigram 各算各的。義項不同的例外見 §4.2。17c 的 11 組雙向在 rime-essay 裡查過，另一種寫法的詞都同義。
  - 罕用的「蔘／參」「痹／痺」等不動。
  - 已知限制（M1 票 17d，使用者 2026-11-05 決定不修）：簡體模式打單向生出的寫法（`nbta`「隨著」）出「随著」，不是「随着」。簡繁轉換照 OpenCC `hk2s`，沒有「著→着」；OpenCC 的 `tw2s` 有，但它的保護詞只 17 個，「土著」「原著」會變成「土着」（brief〈查證過的事實〉）。要修得讓簡繁資料包帶一份 essay 的詞表（多 LGPL、依賴詞庫），不做；打「隨着」的碼出「随着」。
- 切分：依句子的 sha256 分 train 90%、dev 5%、test 5%，同一句永遠在同一個切分；test 取雜湊最小的 2,000 個不重複句進基準。
- 每份語料都切 train / dev / test：test 給首選字基準，不參與訓練。
- 訓練：字級 interpolated Kneser-Ney bigram（M1 票 11，`qim-databuild` 的 `bigram` 模組）：
  - 只數同一句內的相鄰字，不加句尾記號：不知道前文時組字多半不在句首，開頭的字用字頻 unigram `(c + 1) ÷ (N + V)`。
  - 句首（M3 票 04）：每句的第一字接在句首記號 `qim_core::CLAUSE_START`（U+0002）後面，算成語言模型裡一個前一字的一列，同樣的 KN 式子、`D` 與接續機率用句內的；它的次數不算進字次數、`N1+(•w)` 與 `D`，所以其他字的機率和沒有它時相同。前文以標點結尾時組字開頭的字接它（§2.3）。資料包格式不變：句首只是一個多出來的字與它的一列，舊的核心讀到也只是一個打不出的字。
  - 單一折扣 `D = n1 ÷ (n1 + 2 n2)`；退回用接續機率 `(N1+(•w) + 1) ÷ (N1+(••) + V)`，`V` 是連打字集的字數。`n1`、`n2` 與 `D` 都在乘上語料權重後的次數上算：口語字對次數至少 64，進不了 `n1`、`n2`，口語前一字的退回權重近 0，口語幾乎不平滑；dev 上撐得住（M1 票 24 code-review）。M3 票 08 比照標準 KN 重訓（各語料在原始次數上算 `D`、次數依權重相加）：字級 bigram 沒有好處（test 口語首選 −0.1 點、修正 +2.0%），照舊；高一階的 trigram、詞級 n-gram 用現行做法比標準 KN 口語首選低 1.6–3.5 點（dev），所以下面的詞級 trigram 用標準 KN（`scripts/experiments/model-ladder/` README〈同步層〉）。
  - 剪枝：只留乘上語料權重後次數 ≥ 2 的相鄰字（2.9 MB；不剪約 4.6 MB），剪掉後重算每個前一字的退回權重，讓機率加總仍為 1。口語權重 64，口語的相鄰字都留下，剪掉的只有維基只出現一次的；不剪在 dev 上口語答錯只少約 1%（M1 票 12）。資料包 ≤ 8 MB（Gboard 文獻的 5–10 MB 預算，`docs/research/facts-products.md`）。
  - 資料包存 ARPA 式的退回形式：留下的 bigram 機率量化成 `u8`（每級 1/12 nat），unigram 與退回權重存 `f32`。
  - 各語料的次數乘上權重再相加（`crates/qim-databuild/src/corpora.rs`），維基 1、口語 64：口語 train 只有維基的約 1/127 句。M1 票 12 在 dev 切分上掃 1–256，口語集修正次數隨權重一路降、維基集在 32 以上開始退，64 之後口語只再好一點；權重屬於共用的語言模型，不寫在方案檔（§2.5）。
- 詞級 trigram（M3 票 11b，`qim-databuild` 的 `word_trigram.rs`；做法與 dev / test 數字在 `scripts/experiments/model-ladder/` README〈同步層〉，票 08 量過）：
  - 段：用現行打分（預設詞庫的詞與詞頻、字級 bigram、方案的 β、γ）把四份 train 切分的每句切成段（文字已知的 Viterbi，前一字固定是句中的前一字），和解碼時的切法一致；每句前面接句首當一段。換了詞庫、bigram 或 β、γ 要重建，`cargo xtask data` 每次都重切。
  - 機率：interpolated KN，折扣照標準 KN：每份語料在自己的原始次數上算 `D_k = n1 ÷ (n1 + 2 n2)`，一種 n-gram 扣 `Σ D_k × w_k`（出現在語料 k 的才算，權重同字級 bigram）。trigram 退回詞級 bigram（剪枝、量化後的那份），詞級 bigram 退回現行打分。剪枝只留乘上權重後次數 ≥ 門檻的，重算 α 讓機率加總為 1，α 夾在 1 以下（罕用的前文分母近 0，α 會爆大，票 08）。機率 `u8` 量化同字級 bigram、α 存 `f32`；詞表照第一次出現的順序編號（常見的段在前，差值的 varint 小；照文字排序重編時預算版大 0.35 MB、加上字級 bigram 超過 8 MB），只留留下的表用到的段。
  - 兩份都建、都進資料包清單：預算版 `zh-hant-word-trigram`（門檻 bigram 8、trigram 8，4.82 MB（4.60 MiB），加上字級 bigram 7.84 MB ≤ 8 MB，`--verify` 擋），方案指名的預設；放寬版 `zh-hant-word-trigram-large`（2、4，14.56 MB（13.89 MiB）），設定頁可以切換。
  - iPhone 18 Pro Max（M3 票 11b，`docs/research/ios-feasibility.md`〈詞級 trigram 的記憶體〉）：核心加六份資料包載入後 79.4 MB，再載入放寬版 footprint 留下 +23.4 MB、峰值 +71.3 MB（到 150.7 MB，那個程序收到兩次記憶體警告），超過 M3 的 60 MB 上限，所以預設留預算版；預算版留下 +5.6 MB、峰值 +20.4 MB。峰值比留下的多 48 MB，推論是解開途中的暫存（檔案位元組、`qim-ffi` 複製進 Rust 的那份、差值格式的中間陣列，沒逐項量）；要改用放寬版得先把解開改成就地轉換、再在 iPhone 量一次。
  - 和預設詞庫綁在一起：詞表的「段」是用方案指名的預設詞庫（`quick-lexicon`）與當時的打分參數（字級 bigram、β、γ）切出來的。改預設詞庫、調 β、γ 或重訓字級 bigram 時，詞級 trigram 要一起重建（`cargo xtask data` 一起建，所以本機永遠對得上）；線上更新（§4.7）各資料包各自發版，更新詞庫或語言模型時詞級 trigram 也要同一批重建、一起發，不然切法對不上。對不上不會打錯：模型沒見過的段照退回式乘 α 或整個退回現行打分，只是首選的進步變小。設定區換成粵典詞庫時就是這樣：口語集首選 37.8% → 45.4%（+7.5 點），比預設詞庫的 +8.2 點小 0.6 點（M3 票 11b Comments）。manifest 要不要記訓練用的詞庫 id 與版本、讓殼或建置端擋掉對不上的組合，留給之後（同票）。
  - 建置：四份語料切段、數一次，兩種門檻各建一份，這台 Mac 多約 30 秒、峰值約 5 GB（`cargo xtask data` 約 75 秒）。
- 授權：語料是 CC BY-SA 4.0，這份模型隨 App 發佈是否要以相同授權散布，與 LGPL 資料並列 brief §11 的待決項，由使用者判斷。詞級 trigram 另用 rime-essay（LGPL-3.0）的詞與詞頻切段，`LICENSE`、`NOTICE` 列四份語料、rime-essay 與粵語口語詞表（切段用的預設詞庫的來源）。

### 4.4 英文、簡繁與 emoji

- 英文：AOSP LatinIME（`127336e`）的 `dictionaries/en_US_wordlist.combined.gz`，就是 LatinIME 建美式英文主字典的詞表（160,715 詞、逐詞頻率 `f` 0–255 與 bigram，Apache-2.0），建成英文資料包 `aosp-en-us`（M4 票 22，取代 ESDB 的 `scowl-en-us`）。不用 aosp-dictionaries 的 experimental 詞典：它把這份詞表整個併進去、多數頻率照抄，另加授權沒寫版本的 Leipzig 語料詞頻（事實 T39，協調者 2026-11-15 定）。`qim-databuild` 收每個詞與它的 `f`（`english.rs`），LatinIME 不建議的不收：可能冒犯的（`possibly_offensive`、`flags=offensive`）、只當 shortcut 的錯拼（`not_a_word`，如 `acomodate`）、`f=0`（如 `DOS`）；bigram 與 shortcut 不收（下一個詞留到 M5，ADR 0008）。`z`、撇號、連字號的詞都收，英文模式要用；中文組字的英文候選（§2.4 規則 4）只比對整串輸入碼，輸入碼有萬用鍵 `z` 時本來就不出（M4 票 11），撇號、連字號打不出，所以 `z` 只在中文組字的比對時濾掉。同一個小寫拼法只留一種大小寫，頻率取留下的那種（§2.4 規則 4）。頻率給英文模式的建議列排序（M4 票 24）。googlesource 只給 base64，`data/sources.toml` 的檔標 `encoding = "base64"`，下載後先解開再核對 sha256。
- emoji：CLDR（`release-49`）`common/annotations/` 的 `yue.xml`、`zh_Hant.xml` 建成 emoji 資料包 `cldr-emoji-yue-hant`（M3 票 16，`emoji.rs`）。每個 emoji 的短名（`type="tts"`）與每個關鍵字（` | ` 分隔）各是一個字詞。短名一律收；關鍵字是給 emoji 選單搜尋的、收得很寬（粵語標註裡 😄 的關鍵字有「嘴」「眼」「打開」「臉」），只在它對到的 emoji 不超過 6 個（兩份標註合計、不重複，`MAX_KEYWORD_EMOJI`）時收，「臉」「動物」這類分類詞自然濾掉；門檻由抽查定（「笑」剛好對到 5 個、要留著，「打開」「眼」「嘴」後面不再接笑臉），首選字基準在 4–不限之間幾乎不變，見 `docs/research/emoji-candidates.md` §3（M3 票 16）。一個字詞的 emoji 依序是短名是它的、再來關鍵字有它的，兩種之內粵語在繁中前（brief §1.3 粵語字序以 iOS 為準）、檔內照檔的順序。字詞照詞庫的寫法換（`t2hk` 再換回常見寫法，§4.2），有字沒有速成碼的不收。CLDR 的 `cp` 拿掉了 U+FE0F，照 Unicode 18.0 的 `emoji-test.txt` 換回 fully-qualified 的寫法，否則 ❤ 這類字顯示成文字。膚色與組合序列（`annotationsDerived/`）不收，免得候選塞滿同一個 emoji 的變體；方框漢字（🈶🈚🈷 等 U+1F200–1F2FF 與 ㊗ ㊙）也不收，它們的標註只是框裡的字，全收時打「有」「無」「月」這些常用字第 2 位就是它。CLDR 的關鍵字覆蓋不均（「笑」在繁中只對到 🤣🥰😼、粵語是 😆🤣😂，「哈哈」「開心」粵語沒有），要補口語叫法時另加 repo 裡自行整理的詞表（同粵語口語詞表，CC0），等使用者回報。
- 簡繁：OpenCC（`fae2a0a`）的字典建成簡繁資料包 `opencc-hk2s`，`qim-core` 照 OpenCC 的規則轉換（`simplifier.rs`）：照 `hk2s.json` 先正規化 CJK 相容表意文字、用 TSPhrases 分段，再經 HKVariantsRevPhrases + HKVariantsRev、TSPhrases + TSCharacters 兩組，每組第一張有匹配的表勝出、表內取最長的鍵。`@tofu-risk` 的罕用簡體字不用、保留原字（多數字型顯示不出）。不用 `ferrous-opencc`：它的公開 API 只能用編進 crate 的字典（版本不是 `fae2a0a`），不能從資料包載入（M1 票 16 Comments）。

### 4.5 驗證（`cargo xtask data --verify`，M0 起每次改資料管線或來源都跑）

- 不掉字：連打字集的字數等於來源中旗標符合、且有倉頡碼的字數。
- 速成碼表的每個鍵長度都是 1 或 2；不存在長度 3 以上的鍵（ADR 0003 的守門測試）。
- 抽查：`anr` 查不到「問」；「問」的速成碼含 `ar`；「丟」同時有 `hi` 與 `mi`；`a` 的首選是「日」；`anr` 的候選有「門口」；簡體模式 `anr` 的第一頁有「门口」、沒有「門口」；中文狀態打 `the` 第 2 個候選是 `the`；`hk` 的「笑」後面緊接 emoji 資料包裡它的第一個 emoji。「日」「門口」經 `Engine` 查，含字頻排序；「门口」、`the` 與 `hk` 經 `Session` 打。
- 詞庫：每個詞的碼都由詞裡每個字的一個速成碼接成；報告列詞數、被門檻與字集濾掉的數量、HKCanCor 補上的詞數與對不上而略過的話輪數。
- Unihan `kCangjie` 交叉比對，差異清單輸出成報表，不擋建置。
- bigram 資料包 ≤ 8 MB（§4.3）；詞級 trigram 預算版加上 bigram ≤ 8 MB（M3 票 11b，spec〈採用門檻〉的預算版）。
- 前處理保留的粵語用字（`corpora.rs` 的 `CANTONESE_KEPT`：晒、搵、吓、郁、喂、涌、咸、响、仆）在 train 的字頻都大於 0；字形對照表換掉的香港寫法與雙向並存寫法的另一種在 train 都不出現（§4.3，M1 票 17b）；詞庫沒有對照表換掉的寫法與「着名」這類反向的錯字，雙向並存寫法的詞每種寫法都在、詞頻相同、兩字字頻相同，含單向前一個字的詞都另收後一個字的寫法（§4.2，M1 票 17c）；雙向兩字的 unigram、詞庫裡含它的相鄰字每種寫法的 bigram 都相同（M1 票 17d）。
- 簡體字（M3 票 19）：繁簡同形與本身是正規繁體字的人、日、台、后、里、干、面、系、云、松、只、范、谷、斗、制、准、几都沒標成簡體字，報告列簡體字的字數；繁體模式 `ls` 的「门」排在碼表裡 `ls` 每個沒標簡體字的連打字集的字之後（經 `Engine` 查）。
- 字根提示：碼表的每個字（含擴展區）都有至少一個倉頡碼；抽查「丟」三代 `hgi`、五代先列 `mgi`，「問」兩代都是 `anr`。
- 英文：報告列詞表的詞數、可能冒犯的、只當 shortcut 的錯拼與 `f=0` 而不收的數量與收的詞數；抽查英文資料包有 `zero` 與它的頻率（`z` 開頭的詞留給英文模式）。
- emoji：報告列 CLDR 標註數、太泛而不收的關鍵字數、方框漢字數、有字打不出而不收的字詞數、收的字詞數與 emoji 數。
- 決定性：同樣的來源建出 sha256 相同的資料包（速成、詞庫、粵典詞庫、bigram、詞級 trigram 預算版與放寬版、字根提示、簡繁、英文與 emoji 十份；第二次建置把詞級 trigram 也重切重訓）。
- 粵典詞庫（M3 票 24）：收了預設詞庫的每個詞條（同碼、同詞頻），也多了詞；報告列粵典的詞頭數與補上的詞數。粵典的來源對不上而略過時不做這項，報告列「略過 quick-lexicon-words-hk」與原因，不算失敗（§4.2）。
- 方案指名：建出的詞庫、語言模型、詞級 trigram、字根提示的 id 是方案 `[packs]` 指名的那份（核心接受別份，§4.6，所以由建置端擋）。
- manifest（§4.6）：每份資料包旁的 `manifest.json` 讀得回，照殼的路（`Pack::open`）讀回寫出的檔核對得過。

### 4.6 資料包 manifest（M3 票 20）

每份資料包是一個目錄 `<id>/`：資料包檔 `<id>.pack`、`manifest.json`、`LICENSE`、`NOTICE`。manifest 是對外格式，`qim-databuild` 寫出（`crates/qim-core/src/manifest.rs` 的 `Manifest`）。慣例查證見 `docs/research/pack-manifest.md`（2026-11-11）。

```json
{
  "manifest_version": 1,
  "id": "quick-lexicon",
  "kind": "lexicon",
  "version": "1.0.0",
  "min_core_version": "0.0.0",
  "size": 5201789,
  "sha256": "25d53b1d…",
  "license": "LGPL-3.0 AND CC-BY-SA-4.0 AND CC-BY-4.0 AND CC0-1.0"
}
```

| 欄位 | 內容 | 採用的慣例 |
| --- | --- | --- |
| `manifest_version` | manifest 格式的版本，現在 1；欄位的意思改了才加，加欄位不加 | WebExtension `manifest.json` 的 `manifest_version`、Cargo index 的 `v` |
| `id` | 等於資料包裡的 id；小寫英數與 `-`，資料包檔名是 `<id>.pack` | Cargo `name` 的字元限制 |
| `kind` | `scheme`、`lexicon`、`language-model`、`word-trigram`（M3 票 11b 加）、`radical-hint`、`simplifier`、`english`、`emoji`、`model`（停鍵層的神經模型，M3 票 11d 加），一種對應一個 `Pack` 型別；加種類不加 `manifest_version`，舊核心讀到不認得的種類拒絕載入 | — |
| `version` | semver；同一個 id 的新版較大。內容改了就加，現在由 `pipeline.rs` 的 `build_packs` 每份各寫一個，全是 `1.0.0` | Cargo `vers`、npm、VS Code |
| `min_core_version` | 讀得了它的最舊核心（`qim-core` 的 crate 版本），`MAJOR.MINOR.PATCH`，單一下限、不寫範圍；建置端寫建它的核心版本 | Cargo `rust-version`、Firefox `strict_min_version`、LibreOffice minimal-version |
| `size` | 資料包檔的位元組數 | Flatpak `extra-data` 的 `size` |
| `sha256` | 資料包檔整份的 sha256，64 位小寫十六進位 | Cargo `cksum`、Homebrew、Flatpak；GitHub Releases asset 的 `digest`（`sha256:<hex>`）只當交叉檢查 |
| `license` | SPDX 運算式：各來源 `sources.toml` 的 `license` 依序以 ` AND ` 接起、不重複；SPDX 沒有的用 `LicenseRef-`。全文在同目錄的 `LICENSE` | npm `license` |

- 不認得的欄位略過（給之後加欄位）；少欄位、`manifest_version` 不是 1、`id`、`version` 或 `sha256` 不合格式時拒絕（`InvalidManifest`，不報成檔案損壞）。
- 載入：殼交一組 manifest 加位元組給 `Engine::open`（依各自的種類放好，同種兩份、給了字根提示或模型、沒有方案資料包時拒絕），字根提示經 `RadicalHintPack::open`，模型經 `ModelPack::open`（核心只核對、解開，權重原樣交給 `qim-neural`，§2.7），不知道種類時用 `Manifest::verify`（匯入前檢查）；底下都是 `Pack::open(manifest, bytes)`，依序核對欄位格式、`min_core_version ≤ CORE_VERSION`、種類、大小、sha256，解開後再核對 id；不合時回 `PackError`（`CoreTooOld`、`ChecksumMismatch`…），訊息說明原因。`Pack::from_bytes` 不核對，給建置端與測試讀剛建好的檔。網頁經 `qim-wasm` 的 `Pack`，模型資料包另交給 Web Worker 的 `Reranker`（§3.1）；`qim-ffi` 還是只收位元組，M7 起的平台殼接資料包管理時改成同一條路。
- 不照研究建議把 manifest 塞進 `.pack` 檔頭：那要改資料包格式、每份 sha256 都換，而且檔內放不了自己的 sha256；匯入改成 manifest 與 `.pack` 一起選。放上 GitHub Releases 時改名成不撞名的 asset（§4.7），安裝都走 manifest 加位元組這條路（brief §5.3）。
- 使用中的資料包：方案 `[packs]` 指名的是預設；同一種可以裝好幾份，殼選一份交給核心（設定頁的切換）。所以 `Engine::new` 與 `RadicalHints::for_scheme` 只核對方案要不要那種資料包、不核對 id（推翻 M1 的「要剛好是指名的那份」）；建出的預設那份是不是指名的由 `--verify` 擋（§4.5）。
- 網頁（plan §3.1）：內建的從 `/packs/<id>/` fetch，匯入的存 IndexedDB（`idb-keyval`，庫 `qim-packs`、表 `packs`、鍵是 id），每種用哪份存 `localStorage` 的 `qim.packs.active`。`localStorage` 每個網域約 5 MB、只能存字串，詞庫一份就 5 MB，所以用 IndexedDB。和內建同 id 的匯入取代內建那份，刪掉就回到內建的；內建的不能刪。匯入後那一種改用它（同 Firefox 裝字典後即可用）。換資料包時核心換掉 `Engine`，學習資料、選項與組字留著。匯入、刪除先在記憶體換、核心收下了才寫進 IndexedDB，切換也是核心收下才算，不收就退回原本的樣子；和已有的資料包同 id 但種類不同的匯入直接拒絕。載入時選的或匯入的資料包核心不收（例如 IndexedDB 裡的檔壞了），這次只用內建的、在狀態列說明，IndexedDB 裡的不刪。IndexedDB 打不開（瀏覽器封鎖網站資料）時也只用內建的、狀態列說明，這次匯入或下載的只留在記憶體（M4 票 02）。
- e2e 匯入的小詞庫 `web/e2e/fixtures/e2e-lexicon/` 進 git，`crates/qim-core/tests/web_fixture.rs` 比對它和現在的格式一致；格式改了照測試訊息重產。

### 4.7 線上更新（M3 票 21）

資料包不等 App 發版就能換新版（brief §5.3）。來源是 GitHub Releases（`1morr/QIM`，使用者 2026-11-11 決定）的 tag `packs` 那個 release：上面放每份資料包的檔與一份更新索引 `index.json`，殼讀索引、比出裝著的資料包有沒有新版、下載、照 manifest 核對後換上。格式與規則在核心（`crates/qim-core/src/pack_index.rs` 的 `PackIndex`），殼只做 HTTP 與儲存。照研究（`docs/research/pack-manifest.md` 第 6、7 點）：外部索引記每個檔的 sha256，同 Cargo index、Homebrew、Flatpak；檔案本身不帶索引資訊。

release 的 asset（`cargo xtask pack-release` 由 `build/packs/` 產生到 `build/pack-release/`）：

| asset | 內容 |
| --- | --- |
| `index.json` | 更新索引，見下 |
| `<id>.pack` | 資料包檔 |
| `<id>.manifest.json` | 同目錄的 `manifest.json`；網頁的「從檔案匯入」也認這個檔名 |
| `<id>.LICENSE`、`<id>.NOTICE` | 同目錄的 `LICENSE`、`NOTICE`（asset 名字不能重複，各份的 `LICENSE` 會撞名，所以加 id 前綴；不打包成 zip：`.pack` 本來就是單檔，殼也不必多一個解壓縮依賴） |

```json
{
  "index_version": 1,
  "packs": [
    { "manifest_version": 1, "id": "quick-lexicon", "kind": "lexicon", "version": "1.1.0", "min_core_version": "0.0.0",
      "size": 5201789, "sha256": "25d53b1d…", "license": "LGPL-3.0 AND …", "url": "quick-lexicon.pack" }
  ]
}
```

- `index_version`：索引格式的版本，規則同 `manifest_version`（§4.6）。`packs` 每項是一份 manifest 的欄位攤平、多一個 `url`，所以一項原樣就是一份合格的 manifest（核心略過不認得的欄位）。`url` 是相對於 `index.json` 網址的相對參照（RFC 3986），release 上就是 `<id>.pack`；同一份索引放本機伺服器或 GitHub 都能用。
- 讀索引（`PackIndex::from_json`）時核對：`index_version` 是 1、每項的 manifest 欄位合格式（同 §4.6）、id 不重複；不合回 `PackError::InvalidIndex`。
- 要下載的（`PackIndex::updates`）：裝著的每份資料包，索引裡同 id、同種類、版本（semver）較大、`min_core_version` 不比這個核心新的那項。沒裝的不裝：線上更新只換新版（同 Firefox 的字典更新）。下載後照 §4.6 經 `Pack::open` 核對大小與 sha256，不合就不換、照用舊版。GitHub asset 的 `digest` 不用：索引自己有 sha256。
- 發新版要加 `version`：`cargo xtask pack-release --previous <上一版的 index.json>` 拿上一版索引比（`PackIndex::check_successor`），同 id 的 sha256 變了而版本沒變大、或版本變小時失敗，release 目錄不動。這補上 §4.6「`version` 靠人加」沒有的檢查；不給 `--previous` 時不比（第一次發佈），發佈步驟見 `docs/development.md`〈線上更新的 release〉。
- 固定用 tag `packs` 的 release、換新版時換掉它的 asset（同 Neovim 在 GitHub 上的 `nightly` 滾動 tag），不用 `releases/latest`：App 也會在同一個 repo 發 release（brief §10 的 APK），`latest` 會指到 App 的那個。換 asset 時先傳資料包、最後傳 `index.json`；中間讀到舊索引的殼下載的檔核對不過、這次不換，下次再試。
- 網頁（§3.1）：設定區「資料包」一段的「線上更新」開關（預設開，存 `localStorage` 的 `qim.packs.update`，關掉存 `off`）與「檢查更新」鍵。開著時每次開網頁在背景檢查一次（`web/src/pack-update.ts`），不擋載入與打字；失敗、斷網或來源不回應時照用現有的，狀態列說明（brief §8）。索引 15 秒、每份資料包 120 秒讀不完算失敗，「檢查更新」鍵才能再按。關掉就不送任何請求，「檢查更新」也停用。下載的新版經 `PackLibrary.install` 存 IndexedDB（同匯入那條路），來源欄寫「線上更新」；它取代同 id 的那份，但不改每種用哪份。下載期間那份被刪掉或換掉的不裝回來。
- 網頁讀不了 GitHub 的 release：github.com 的轉址與 release-assets.githubusercontent.com 都不帶 `Access-Control-Allow-Origin`（2026-11-11 用 curl 帶 `Origin` 標頭實測）。所以網頁一律向自己的 `/pack-updates/<檔名>` 要，由 Vite 開發伺服器轉發（`web/vite.config.ts`），來源預設 `https://github.com/1morr/QIM/releases/download/packs/`，環境變數 `QIM_PACK_UPDATE_URL` 可換。`vite build` 出的靜態頁沒有轉發，也就沒有線上更新。平台殼（M7 起）沒有 CORS，直接讀 release。
- e2e 用 Playwright 起的本機靜態伺服器（`web/e2e/static-server.ts`）扮演 release，放 `web/e2e/fixtures/pack-release/`：`qim-databuild` 的 `tests/release.rs` 照 `cargo xtask pack-release` 的路（`release::write`）寫出，不手寫索引。
- `pack-release` 收 `build/packs/` 的每一份，停鍵層的模型資料包 `zh-hant-neural-rerank`（M3 票 11d）也一起進索引；殼裝著它時照樣線上更新。
- 還沒在 GitHub 建 release：票 21 時使用者不在、只授權推 `main`，M3 只驗本機流程；真的建 release 前先問使用者。
- 同一個 id 的內建版與下載版：每次列出時取版本（semver，核心的 `Manifest::is_newer_than`）較高的那份；同版用內建的：內建的隨 App 發佈、已知可用，同版號照理內容相同，不同就是發佈出錯，以 App 帶的為準。比下去的下載版不刪、只是不列也不交給核心；之後 App 內建版又比它舊（例如回退）時它會回來。所以 App 內建了更新的版本時，使用者不會卡在舊的下載版（`web/src/pack-choice.ts` 的 `listPacks`）。從檔案匯入的不比版本，照 §4.6 一律取代內建的：那是使用者自己選的。

## 5. 測試策略

已搬到 `ARCHITECTURE.md`〈測試策略〉。

### 5.1 核心

已搬到 `ARCHITECTURE.md`〈測試策略〉。

### 5.2 首選字基準【決定】

組成與門檻已搬到 `ARCHITECTURE.md`〈測試策略〉；檔案格式、指標與量法在 `docs/development.md`〈首選字基準〉與〈資料建置〉的〈語料〉〈個人集〉。

### 5.3 佈局

已搬到 `ARCHITECTURE.md`〈測試策略〉；iOS 預設佈局要不要放地球鍵搬到 roadmap〈平台里程碑（M7–M11）〉的 M8。

### 5.4 平台殼

已搬到 `ARCHITECTURE.md`〈測試策略〉。

## 6. 開發環境與閘門

已搬到 `docs/development.md`〈全部檢查〉〈閘門〉〈安裝〉。

## 7. 里程碑任務

搬到 `docs/roadmap.md`（2026-11-13）；這一節的編號留著，其他節的編號不變。

## 8. 風險與回寫點

| 風險 | 影響 | 何時確認 | 回寫 |
| --- | --- | --- | --- |
| 調 bigram 的參數後 `mir` 不再選「一台」（M1 票 11 換上 bigram 後仍選對，票 12 調參後也仍選對，`bench/fixed.tsv` 擋） | M1 驗收 | 改打分或資料包的票 | 改用詞級 bigram 或其他資料，記偏差；必要時改 brief §5.2 |
| 口語語料太小（M3 票 09 使用者決定先用現有語料訓練神經模型；M1 票 03：HKCanCor 加 Common Voice 約 5.6 萬句 train、2,630 句 test；票 12：權重加到 256 口語集仍在進步，資料量是瓶頸） | 口語集代表性、bigram 品質 | M1 票 12 之後由協調者問使用者 | 先靠個人集；再不足時請使用者決定其他來源（例如自存一份 Tatoeba），寫回 brief §5.2 |
| Wikimedia 的 XML dump 預告 2027-01 停用，粵語維基的釘選可能下載不到 | 乾淨建置 | 2027-01 前 | 改用 archive.org 的存檔或新的 MediaWiki Content File Exports，重釘 `sources.toml`（`docs/research/facts-tables.md`） |
| 使用者的 Rime 學習資料遲遲沒到，個人集是空的 | M1 的品質判斷只剩公開語料 | M1 票 04 開工時 | 停下問使用者；其他票照常 |
| iPhone 版 Gboard 沒有速成 | 黑箱基線少一個對象 | M1 票 09 開頭 | 只測 Android 版，註明 |
| 空間模型讓正確輸入被錯字蓋過 | M4 範圍 | M4 基準 | 改成只在無候選時啟動，或只用於觸控 |
| 小型神經模型放不進鍵盤擴充或太慢（M3 票 11 採用 1.4M、不採用 23M；M2 票 11：candle 在擴充裡能跑；1.4M 級幾乎不佔；23M 級 q8 約 +47 MB、f32 約 +95 MB，q8 上下文 200 字 p95 55 ms，以前綴快取為前提；票 09 確認上限 177 MB、核心加六份資料包 79 MB，23M q8 合計約 126 MB、f32 約 174 MB 太貼上限，M3 上限定為峰值增量 ≤ 60 MB（roadmap〈M3 的模型階梯實驗〉）；M3 票 09 實量 9 條 × 18 字：23M q8 +75 MB、p95 84 ms 超標，1.4M q8 +2 MB、11 ms；旗艦機，`docs/research/facts-platforms.md`） | M3 範圍、M8 | M2、roadmap〈M3 的模型階梯實驗〉 | 只在桌面與 Android 用，或不做，寫回 brief §1.5 |
| 維基語料授權（CC BY-SA）對模型的要求 | 發佈 | 上架前 | brief §11 的待決項，由使用者判斷 |
| iOS 鍵盤擴充放不下核心與資料包（M2 票 09：iPhone 18 Pro Max 放得下，79 MB／177 MB，格式不改；低階機沒量，§4） | M3 的模型大小、M8 | M2；低階機在 M8 | 資料包改 mmap 友善格式（例如 `fst`），改 §1.4、§4 |
| `jsonschema` 讓程式碼變大（M1 票 19：網頁 WebAssembly 由 0.42 MB 變 4.6 MB，gzip 1.2 MB，多半是它帶的 regex 與大數） | iOS 鍵盤擴充的記憶體（brief §9） | M2 | 鍵盤擴充只讀存檔時驗過的佈局，Schema 驗證留在主 App 與編輯器（`qim-layout` 加 feature 關掉），改 `ARCHITECTURE.md`〈依賴規則〉〈主要依賴〉（M2 票 09 已決定；做法寫在 roadmap〈平台里程碑（M7–M11）〉的 M8，實作在 M8） |
| 免費佈建不能跑鍵盤擴充（M2 票 06：能跑，App Group 擴充只能讀；brief §11 已決） | M2、M8 | M2 開頭 | 請使用者決定提前申請帳號，或實驗改在模擬器做並註明限制 |
| 延遲超過預算（M2：iPhone 18 Pro Max 鍵盤擴充每鍵 p95 64 µs、選開頭一段 0.79 ms，brief §9；低階機沒量） | 所有平台 | 每次 `bench` | 調 beam 或剪枝 bigram，記 baseline |
| LLM 重排提升太小或太慢（M3 票 10、11：Foundation Models 口語前 2 名只 +1.3 點、停頓後第一個請求 14.6% 超過 2 s；雲端 flash +10.4 點但比 1.4M 只多 2.0 點，都不做；票 25 在新基線重量，flash 比基線 +7.9、比 1.4M +2.4，仍不做；brief §1.5 已寫回） | M3 範圍 | roadmap〈M3 的模型階梯實驗〉 的實驗 | 票 25 對新基線重量：flash 比 1.4M 只多 2.4 點（要 3.0），維持不做；重評條件見票 22 |
| iOS 鍵盤擴充不能用 Foundation Models（M2 票 10：能用，不需完整取用；但停頓後第一個請求偶爾 6–15 s，`docs/research/facts-platforms.md`） | M8 的端上 LLM | M2 | iOS 只用雲端 LLM（需完整取用權限） |
| 瀏覽器呼叫雲端 LLM 被 CORS 擋 | 雲端重評後的網頁接入 | 重評時 | 用 Vite 開發伺服器轉發，只影響網頁測試頁；線上更新讀 GitHub release 遇到同一個問題，已這樣做（§4.7） |
| 內嵌網頁編輯器在手機主 App 手感不佳 | M8、M9 | M8 開工時 | 只把手機上手感差的操作改原生；要全改原生時記偏差 |
| iOS 鍵盤擴充收不到實體鍵事件（目前只有 2017 年的第三方說法，未查證） | M8 的實體鍵盤 | M8 開工時 | iOS 不做實體鍵盤模式，設定頁註明，寫回 brief §1.5 |
| 網頁驗不到的平台差異（真實 App 行為、讀上下文、手機延遲） | M7–M11 | 各平台實打 | 修平台殼；要動核心時記偏差並重跑首選字基準 |
