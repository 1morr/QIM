# M1 網頁 prototype

由 plan §7.2 細化（2026-11-04）。拆票後以票為準；票與本檔衝突時改票或改本檔，並在 progress〈偏差與決定〉記一行。

## 目標

使用者任務書的階段 5：在網頁手感驗證引擎與佈局。速成連打、學習、英文候選、簡繁、觸控佈局都在網頁可用；三個 Rime 案例（`a` 日、`anr` 門口、`mir` 一台）首選正確；之後每項改動都有首選字基準的前後數字可比；競品的黑箱基線記錄在案。

## 範圍（做 / 不做）

### 做

- 資料：
  - 語料管線（plan §4.3）：粵語維基、中文維基、HKCanCor、Tatoeba 粵語句子（Common Voice 讀完條款可用才加）；`s2hk` → 切句 → 只留連打字集 → train / dev / test → 機械轉速成碼（多碼字取五代第一碼）。產出 `bench/corpus/`、`bench/colloquial/`（附 `ATTRIBUTION.md`）、訓練用切分與字頻。
  - 詞庫與字頻資料包（plan §4.2）：rime-essay `054920d`，門檻初值 100，丟掉含連打字集以外字的詞，多碼字組合數設上限；字序改成字頻 → 年級 → Big5（plan §4.1）。
  - 字級 bigram 資料包（plan §4.3）：interpolated Kneser-Ney，剪枝到 ≤ 8 MB。
  - 英文資料包（SCOWL size ≤ 60）、簡繁（OpenCC `fae2a0a` 經 `ferrous-opencc`）、字根提示資料包（plan §4.1 步驟 5）、粵語口語詞表 `data/lexicon/yue-colloquial.tsv`（CC0，使用者審過才進資料包）。
  - 每個新資料包都有自己的 `LICENSE`、`NOTICE`，`cargo xtask data --verify` 加對應檢查。
- 首選字基準（plan §5.2 全部）：`qim-bench`、`cargo xtask bench`（`--check`、`--update-baseline`、`--learning`）、`bench/fixed.tsv`、全部指標、依長度分組、翻盤清單、個人集（`bench/personal/rime-userdb.txt`）、每鍵延遲（原生與 WebAssembly）。
- 核心（plan §2）：lattice、Viterbi + beam、§2.3 打分（unigram → bigram，β、γ 由 dev 切分調出寫進方案檔）；prefix 候選選定後該段固定、其餘繼續組字；組字中各鍵的行為（`handled` 為假時組字是空的）；§2.4 學習與排位規則 1–4、自動造詞、使用者詞與觸發詞、無痕、`# qim-userdict v1` TSV 匯出匯入；`Options` 與 `set_options`；分段、`Left` / `Right` / `focus_segment`；`rerank_request` / `apply_rerank` 與 `generation`。
- 佈局（plan §2.6、§5.3）：`qim-layout`（格式、JSON Schema、兩級驗證、點擊判定、幾何）；格式定稿含佈局版本、淺色與深色、無障礙標籤；實作矩形、圓角、旋轉、上下層順序，橢圓與多邊形只留欄位；預設 Gboard 速成佈局由 `qim-databuild` 算出（手機直向與橫向）。
- 網頁（plan §3.1）：實體鍵盤模式補齊 brief §4.3 的鍵位；觸控模式（行動裝置 viewport）：字根主標籤與字母角標、長按出字母、第一排含數字、符號頁（打完不跳回）、一個自訂頁面示範、按鍵放大預覽、深色、橫向；候選展開面板；組字中顯示字根、字根提示快捷鍵；無痕與簡繁開關；使用者資料存 `localStorage`、匯出匯入；「回報」：匯出剛才那次組字成 `fixed.tsv` 的一行；重排的假實作（延遲後原樣回傳）。
- 黑箱基線（ADR 0006）：`docs/research/closed-source-imes.md` §4 的 T1–T18，對象 Gboard（Android 模擬器、iPhone）、iOS 內建速成、macOS 內建速成。

### 不做

- 個人 bigram、聯想、輸入框上下文、雲端與端上 LLM、資料包 manifest 與線上更新、詞庫匯入：M3。
- 空間模型、容錯、萬用鍵（`z` 在 M1 只吃下不動作）、英文模式的預測與自動修正、欄位類型、智慧標點、滑動輸入、撤銷學習：M4。
- 佈局編輯器、橢圓與多邊形的實作與點擊判定、單手佈局：M5。
- `qim-ffi` 與任何平台殼：M2 起。
- 學習資料在主 App 列出、搜尋、刪除：平台殼的主 App；網頁只做匯出匯入。
- CI 與遠端：建遠端要先問使用者（plan §6）。

## 共同規則

- 沿用 M0 的：依賴用 `cargo add` / `npm install` 取當時 stable，與 plan §1.4 不同時同一 commit 更新該表，版本相關 API 先查 context7 或官方文件；命名照 plan §0 與 `GLOSSARY.md`；建置產物與下載的來源不進 git；新增或改動開發指令的票同輪更新 `docs/development.md`；候選每頁 9 個；擴展區的字排在連打字集之後。
- 首選字基準那張票之後，改核心或資料包的票在 commit body 或票的 `## Comments` 附 `cargo xtask bench` 的前後對照：各集合的指標、依長度分組（有了之後）、翻盤清單（有了之後）。數字變差要在票內說明原因；`--update-baseline` 同一個 commit 寫理由（plan §5.2）。
- 固定題（`bench/fixed.tsv`）在案例通過的那張票加入，不先加會紅的題。
- 個人集只報數字與題號：內容不寫進 repo 的任何檔案、commit、票，也不送雲端（plan §5.2）。
- 動到網頁的票，收尾除了 Playwright 腳本，再實際操作網頁亂按一輪（組字中夾雜各種鍵、快速連按、切換模式）；找到的問題先寫成會紅的 Playwright 測試再修。
- M1 不新增流程閘門（progress、commit message 這類）；新的自動檢查只加在產品行為、資料驗證與基準上。
- 新資料來源釘進 `data/sources.toml`（版本或 dump 日期、每個檔的 sha256、授權、取用的檔案）；每個資料包的 `LICENSE`、`NOTICE` 只寫它用到的來源（plan §4）。來源下載很大時，`qim-databuild` 只抽需要的部分，抽法寫在 `sources.toml` 的註解。
- 需要使用者提供或決定的事（黑箱用的 iPhone、粵語詞表審閱、真實句子），到了就在該票停下問，不阻塞其他票。
- 票 09（M0 收尾）Comments 裡留給 M1 的發現，已分配到各票的〈M0 留下的〉；做完或決定不做都在票的 `## Comments` 交代。

## 驗收

- Playwright 實際打字並讀回畫面（貼結果、截圖存 `.scratch/m1/evidence/`）：
  - 實體鍵盤：三個 Rime 案例的首選；同一個碼選同一個非首選兩次後成首選；中文狀態打 `the` 第 2 位是 `the`；簡體模式 `anr` 首選「门口」；連打後移到中間分段改選、上屏結果符合重算。
  - 觸控（行動裝置 viewport）：點按鍵打出「門口」；長按讀回英文字母；橫向仍打得出「門口」；深色讀回按鍵顏色；自訂頁面可切換並輸入；重疊測試佈局點重疊處輸出上層按鍵。
- `cargo xtask bench --check` 綠燈；`bench/fixed.tsv` 含 `a 日`、`anr 門口`、`mir 一台` 與 3–5 句粵語口語；維基、口語、個人三個集合的指標分開列出。
- 每鍵 p95：原生 ≤ 10 ms、WebAssembly ≤ 50 ms（Mac 上量，含 20 碼以上的長輸入）。
- `cargo xtask data --verify` 綠燈：測試句原樣出現在訓練語料的句數為 0；詞的碼都由字的速成碼組成；bigram 資料包 ≤ 8 MB；同樣的來源建兩次 sha256 相同。
- `docs/research/blackbox-baseline.md` 記下版本、裝置與 T1–T18 每項結果；能在網頁做的項目在 QIM 上重跑，和競品並列。
- `cargo xtask check` 全套綠燈；progress.md 依〈寫法與封存〉封存 M1、M2 標為進行中之前，以上全部成立。

## 票序

| 票 | 被誰擋 | 交付 |
| --- | --- | --- |
| [01](issues/01-corpus-yue-wiki.md) | — | 粵語維基 → `bench/corpus/`：前處理、切分、轉碼、洩漏檢查 |
| [02](issues/02-bench-v0.md) | 01 | `qim-bench`、`cargo xtask bench`、`fixed.tsv`、`--check` / `--update-baseline`、第一份 baseline |
| [03](issues/03-corpus-zh-wiki-and-colloquial.md) | 01, 02 | 中文維基、HKCanCor、Tatoeba（Common Voice 讀條款）、`bench/colloquial/`、字頻 |
| [04](issues/04-bench-corrections-and-personal.md) | 02 | 修正次數、KSPC、跳動、依長度分組、翻盤清單、個人集 |
| [05](issues/05-bench-latency.md) | 02 | 每鍵 p50 / p95，原生與 WebAssembly |
| [06](issues/06-lexicon-and-char-frequency.md) | 03 | rime-essay 詞庫、字頻字序、整串查詞庫：網頁 `anr` 查到「門口」 |
| [07](issues/07-continuous.md) | 06 | lattice、Viterbi + beam、unigram + β / γ、prefix 選定；`anr` 門口 |
| [08](issues/08-blackbox-apple.md) | — | macOS、iOS 內建速成的 T1–T18 |
| [09](issues/09-blackbox-gboard.md) | — | Gboard（Android、iPhone）的 T1–T18 |
| [10](issues/10-composition-key-residue.md) | 07 | `handled` 為假時組字是空的；Enter、`z`、標點、大寫、方向鍵 |
| [11](issues/11-bigram-training.md) | 03, 07 | KN bigram ≤ 8 MB；`mir` 仍是一台 |
| [12](issues/12-bigram-tuning-and-report.md) | 04, 05, 08, 09, 11 | β、γ、口語權重；延遲預算；網頁「回報」；請使用者試打 |
| [13](issues/13-learning-counts-and-ranking.md) | 04, 07 | 學習計數、規則 2、`personal`、無痕、`set_options`、`bench --learning` |
| [14](issues/14-user-words-auto-phrase-export.md) | 13 | 使用者詞與觸發詞、自動造詞、`# qim-userdict v1`、`localStorage` |
| [15](issues/15-english-candidates.md) | 13 | SCOWL、規則 4：`the` 第 2 位 |
| [16](issues/16-simplified-chinese.md) | 13 | `ferrous-opencc`、簡體 `anr` 首選「门口」 |
| [17](issues/17-cantonese-colloquial-lexicon.md) | 12 | 粵語詞表（使用者審）、3–5 句口語固定題 |
| [17b](issues/17b-s2hk-keep-cantonese-chars.md) | 17 | 語料前處理保留「晒」「搵」等粵語用字（票 17 審閱時使用者決定） |
| [17c](issues/17c-lexicon-common-forms.md) | — | M1 封存後補（使用者試用）：詞庫也用常見寫法（羣組→群組）|
| [18](issues/18-layout-tracer-touch.md) | 07 | `qim-layout`、最小格式、點擊判定、Gboard 直向；觸控點出「門口」 |
| [19](issues/19-layout-format-final.md) | 18 | 格式定稿、JSON Schema、兩級驗證、旋轉與圓角、重疊佈局 |
| [20](issues/20-touch-keyboard-features.md) | 19 | 長按、符號頁、自訂頁、橫向、深色、放大預覽 |
| [21](issues/21-edit-segments-in-composition.md) | 10, 18 | 分段、← / →、`focus_segment`、觸控點分段 |
| [22](issues/22-radicals-and-radical-hint.md) | 07 | 組字中顯示字根、字根提示資料包與快捷鍵 |
| [23](issues/23-panel-keys-and-rerank-stub.md) | 20, 21 | 候選展開面板、實體鍵位補齊、重排假實作與 `generation` |
| [24](issues/24-m1-close.md) | 01–23 | 里程碑 code-review、重驗〈驗收〉、黑箱在 QIM 重跑、封存 M1 |
| [25](issues/25-shift-rime-default.md) | — | M1 封存後補（使用者試用）：單按 Shift 照 Rime，設定可改 |
| [25b](issues/25b-inline-ascii-space-joins.md) | 25 | M1 封存後補：留碼接英文時空白接進英文（舊版 Rime，使用者決定） |
| [26](issues/26-half-width-after-digits.md) | — | M1 封存後補：數字後的句點與逗號用半形，可關 |
| [27](issues/27-touch-layout-match-gboard.md) | — | M1 封存後補：觸控佈局對齊 Gboard 速成、數字列開關 |
| [28](issues/28-manual-segment-separator.md) | 27 | M1 封存後補：手動分隔鍵（實體 `'`、觸控組字中 ⇧ 變「'詞」，使用者決定） |

08、09 不依賴核心，只擋 12；要用使用者的 iPhone，照編號做時在 07 之後輪到。一個一個做時照編號；同時能做的票（例如 04、05，或 13 之後的 14–16）可以並行。
