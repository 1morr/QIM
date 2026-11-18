# QIM

全平台速成輸入法。文件是唯一的記憶，對話不是。

## 文件

- `docs/design-brief.md`：需求與決定。決定衝突時以它為準。
- `docs/research/`：一層、不分資料夾，檔名三種：`facts-<主題>.md` 是查證過的外部事實（每條有編號與來源，索引在 brief〈事實索引〉）；`<主題>.md` 是會更新的主題整理；`<主題>-<YYYY-MM-DD>.md` 是寫完就凍結的調查或實驗紀錄。
- `ARCHITECTURE.md`（根目錄）：執行形態、模組地圖、依賴規則、跨模組的資料與契約、平台殼、主要依賴、測試策略、風險與回寫點。
- `docs/roadmap.md`：里程碑的順序、狀態與閱讀指引；還沒開工的里程碑各有一小節，開工後縮成一行連到它的 spec；〈不排里程碑〉收沒人接的遺留。
- `docs/progress.md`：進行中里程碑裡不是做票的 session（訪談、規劃、開工、驗收）的紀錄。
- **下一步**不靠記：`python3 scripts/next_step.py` 從 roadmap、票的狀態與 progress.md 最後一列算出等使用者決定的事、現在要下的指令、後面排著什麼。使用者問接下來做什麼，就跑它。
- `docs/decisions.md`：進行中里程碑裡不是做票時定的決定，一條一行，標決定者；驗收後跟著封存。
- `.scratch/<里程碑>/spec.md`：該里程碑的目標、範圍、共同規則、驗收（Agent 驗、使用者試）與票序；由下面〈Skills〉的里程碑開工步驟建立。`issues/` 是票，執行單位，狀態在票的 `Status:` 行。做票的 session 紀錄與決定留在票上，`python3 scripts/ticket_notes.py <里程碑的目錄名>` 印出整個里程碑的。
- `CHANGELOG.md`：使用者拿到什麼、改了什麼，依功能記。做票時寫在票的 `變更：` 行，驗收票組進 `[Unreleased]`。
- `docs/development.md`：開發指令的單一來源；`docs/agents/`：給工程 skill 讀的票與里程碑慣例；`docs/archive/`：已驗收里程碑的紀錄。
- 上面這些檔名與位置是固定的，別的 skill 靠它們找檔。`docs/` 第一層每多一個檔或資料夾，就在這一段加一行說明它是什麼（`scripts/check_docs.py` 擋）；不再維護的搬進 `docs/legacy/`。
- `docs/design/`：畫面的設計稿，使用者確認過的定稿（〈UI〉）。
- `GLOSSARY.md`、`docs/adr/`：名詞表與架構決定，由 `/grill-with-docs` 按需建立；命名一律用名詞表的詞。

## 目錄

| 路徑 | 內容 |
| --- | --- |
| `crates/` | Rust crate；各 crate 的職責與依賴規則見 `ARCHITECTURE.md`〈模組地圖〉〈依賴規則〉 |
| `data/` | 方案檔、來源清單 `sources.toml`、自行整理的詞表 `lexicon/`、進資料包的授權全文 `licenses/`、停鍵層採用的模型 `neural-rerank/`；建置方式見 `docs/development.md`〈資料建置〉〈停鍵層的模型〉 |
| `bench/` | 首選字基準的題目與語料（`docs/development.md`〈首選字基準〉）；`corpus/`、`colloquial/` 由 `cargo xtask data` 產生，`personal/` 只在本機 |
| `xtask/` | 閘門與開發指令（`cargo xtask check`），用法見 `docs/development.md` |
| `web/` | 網頁 prototype（Vite + TypeScript + Biome） |
| `docs/` | brief、roadmap、progress、decisions、`development.md`、ADR、研究、已完成里程碑的封存 `archive/` |
| `.scratch/` | 里程碑的 spec 與票 |
| `.githooks/` | git hook，啟用方式見 `docs/development.md` |
| `.github/` | GitHub Actions 的 CI，跑什麼見 `docs/development.md`〈全部檢查〉 |
| `scripts/` | 腳本：文件檢查 `check_docs.py`（`cargo xtask check` 的 `docs` 步驟跑它，要 Python 3）、下一步 `next_step.py`、票上紀錄 `ticket_notes.py`、封存比對 `docs_moved_lines.py`、停鍵層模型的訓練在 `scripts/neural-rerank/`，設計實驗在 `scripts/experiments/` |

## 使用者確認

- brief、`ARCHITECTURE.md`、ADR、`GLOSSARY.md` 裡的決定：草稿先給使用者看，確認後才寫入；訪談中也一樣。
- 做票時要推翻已定的決定（brief、`ARCHITECTURE.md`、ADR、spec、`docs/decisions.md` 裡的）：停下來問使用者（協調者開的 session 問協調者，協調者照文件答不了的再問使用者）。同意後，同一 commit 改文件，並記一條決定。
- 問使用者決定（對話裡，或留給下一個 session 的 `等使用者：`）：每題寫成沒讀過票也看得懂的話，依序是背景一句、選項、你的建議、不決定會卡住什麼；票號、函式名、章節號後面跟一句它是什麼。
- 使用者回「照建議」，只算同意這一則訊息裡列出來的那幾題。
- 每個決定佔一行並標**決定者**，做票時定的記在票上（見〈每個 session 的流程〉），其餘記在 `docs/decisions.md`：`（使用者定）` 使用者自己定的、`（使用者同意）` agent 提議而使用者同意的、`（Agent 定）` 使用者沒看過的。做票時在兩種做法之間選了一種，就是一個 `（Agent 定）` 的決定；協調者（`/1morr:ticket-coordinator`）代答的題也是，不算使用者的；`/1morr:checkpoint` 會把它們拿給使用者看。

## 每個 session 的流程

工作單位是**票**：`.scratch/<里程碑>/issues/NN-slug.md`，由里程碑開工的 session 產生（見〈Skills〉）。一個 session 只做一張票，在自己的分支上做。

1. 讀 `docs/progress.md`、`docs/decisions.md`、該里程碑的 spec，跑 `python3 scripts/ticket_notes.py <里程碑的目錄名>` 看先前的票定過什麼，再讀票；只讀票列出的 `ARCHITECTURE.md` / brief 章節，其他按需讀。
2. `/implement <票路徑>`：tdd、定期 typecheck、結尾從開工時的 commit 起跑 `mattpocock-skills:code-review`、commit。
3. 驗證：`docs/development.md`〈全部檢查〉加 test；貼指令輸出。
4. 影響架構的決定另寫 ADR；要推翻已定的決定照〈使用者確認〉。
5. 票的 `Status:` 改為 `done`、勾掉驗收條件、code-review 未處理的發現記到票的 `## Comments`；Comments 最後加一行 `Session 紀錄：<YYYY-MM-DD> <做了什麼，一句>`，做票時定的每個選擇各加一行 `決定：<一句>（<決定者>）`，使用者看得到的每個改動各加一行 `變更：<分類>｜<功能>｜<使用者看得懂的一句>`（分類是 Added、Changed、Deprecated、Removed、Fixed、Security 之一；`<功能>` 沿用 `CHANGELOG.md` 與 `GLOSSARY.md` 已有的名字；沒有這種改動寫 `變更：無`）；commit。票就是這些紀錄唯一的來源：做票的分支把 progress.md、decisions.md 與 `CHANGELOG.md` 留給不平行的 session，平行的票才不會互撞。
6. 合併由**開這個 session 的人**做：使用者自己開的 session，做完在主分支 `git merge --no-ff <票分支>`（訊息 `merge: ticket NN <slug>`）；`/1morr:ticket-coordinator` 開的 session 只在自己的分支 commit，合併與 push 留給協調者。
7. 事實編號這類全專案遞增的號，票在分支上先用當時的下一號；合併時撞號，合併的人把後合併的那張改成下一個空號，連同引用它的地方。
8. 驗收票被其他所有票擋、不會平行：它自己改 progress.md 與 decisions.md，封存時把各票的紀錄組進封存檔與 `CHANGELOG.md`（`docs/agents/milestone.md`〈驗收票〉）。

完成標準：票上每個驗收條件打勾、全部檢查與 test 綠燈、Comments 有帶日期的 `Session 紀錄：` 與至少一行 `變更：`、已 commit。缺一就還沒完成。

**所有 session 的結尾**（做票、訪談、寫架構、里程碑開工都算），依序：

1. 不是做票的 session 在 progress.md 加一列，「下一步」寫下一個 session 要下的完整指令：里程碑做完是 `/1morr:checkpoint <剛做完的里程碑>`（回顧上一段、和使用者對齊下一段；它結束時寫 `/to-tickets docs/roadmap.md 拆 <下一個里程碑> 的票`），拆完票是第一張票的 `/implement <票路徑>`。
2. 要使用者先決定才能繼續：做票的 session 在票的 `## Comments` 加一行 `等使用者：<要決定什麼>`，票留著不改 `done`；其他 session 把它寫進 progress.md 的「下一步」。使用者回答後，接手的 session 在那一行下面記 `決定：<一句>（使用者定）`，那一題就結了。
3. commit 之後跑 `python3 scripts/next_step.py`，照它的結果告訴使用者下一步：等使用者決定的每一題用你的話講（寫法照〈使用者確認〉），下一條指令照它印的放進 code block；它說的和你剛做的對不上時，說明差在哪。協調者開的 session 只回報做完了，下一步由協調者報。

## 事實與查證

- 平台輸入法框架（macOS InputMethodKit、Windows TSF、Linux Fcitx5 / IBus、iOS 鍵盤擴充、Android InputMethodService）、碼表的來源與授權、外部引擎（如 Rime）的行為只信兩處：`docs/research/facts-*.md` 的〈事實〉段，以及現查（context7、官方文件）。沒寫的先查再用，查到的補進對應主題的檔，給編號、附來源；新開的檔加進 brief〈事實索引〉。
- 程式碼、測試與註解要指向文件時，引 ADR 編號、事實編號或節的標題；brief 的章節號只寫在文件與票裡，節才能搬動（`scripts/check_docs.py` 擋）。
- 現況以指令為準：`git log`、`git status`、測試輸出。progress.md 與 repo 不符時以 repo 為準並修 progress.md。

## 上下文紀律

- 讀大檔案、查文件、跑實驗交給子代理回摘要。
- brief 只讀任務指到的章節。字元上限由 `scripts/check_docs.py` 強制：本檔 8,000、`ARCHITECTURE.md` 15,000、progress.md 20,000、brief 30,000；超過時搬走或刪掉內容，不靠縮短措辭湊數。
- 對話已長或一個任務碰了十幾個檔案：把進度與下一步寫下來，結束 session。做票時寫在票的 `## Comments`，其他 session 寫進 progress.md。

## 驗收輸入行為

輸入法的行為要在真的宿主程式裡打字並讀回畫面才算數：桌面平台用 `orca computer`，iOS 模擬器用 `orca emulator`，Android 用 `orca emulator-android`。單元測試綠燈不能取代這一步。

## 公開 repo

commit、文件、票、fixture、log 摘錄與截圖：

- 不含個人資訊：使用者名稱、email、手機號、帳號名稱。
- cookie、token、簽名網址、API key 的值寫成 `***`。
- 本機路徑用代稱：`<資料目錄>`、`<暫存目錄>`、`<repo>`。

## UI

- UI 改動實跑驗證，附截圖或讀回的畫面文字。
- 排 roadmap、里程碑開工、做設計稿或對比版、做會新增畫面或改版面的票之前，先讀 `docs/agents/ui.md`：畫面要先有使用者確認過的設計稿才實作。

## 視覺化

這個專案用 `visual-explainer` 把圖和要使用者選的事做成一頁，用在四處：寫架構的草稿、里程碑開工的計劃、開工時票對程式碼找到的問題、要畫出來才看得懂的二選一。到了這四處先讀 `docs/agents/visual.md`。

## Skills

- 需求與設計訪談：`/grill-with-docs`（邊問邊寫 `GLOSSARY.md` 與 ADR），結論寫回 brief。
- 里程碑開工（使用者在新 session 下 `/to-tickets docs/roadmap.md 拆 <里程碑> 的票`）、做驗收票 `NN-<里程碑>-acceptance`：先讀 `docs/agents/milestone.md`，照它的步驟做。之後的里程碑留在 roadmap，開工時才有 spec。
- 做票：一張票一個 session，`/implement`；整批跑票：`/1morr:ticket-coordinator`。
- 里程碑之間：`/1morr:checkpoint`；想看懂某樣東西現在怎麼運作：`/1morr:explain`；里程碑、票、決定的總覽頁：`/1morr:dashboard`。
- 核心（碼表查詢、候選、排序）的新功能與修 bug：`mattpocock-skills:tdd`。
- 設計核心與各平台殼的邊界：`mattpocock-skills:codebase-design`。
- 網頁的 UI（`web/` 的原型頁、觸控鍵盤、設定與面板、`web/editor/` 佈局編輯器）的設計、改版與打磨：`impeccable:impeccable`；觸控鍵盤要照 Gboard 的行為時，行為以 Gboard 實測為準，impeccable 管外觀與互動細節。
- 用實驗回答設計問題（平台框架行為、按鍵延遲實測）：`mattpocock-skills:prototype`，決定寫回 brief，量到的數字寫進 `docs/research/`。實驗腳本留在 `scripts/experiments/`，換版本時要重量。
- 查證外部事實：`mattpocock-skills:research`，輸出放 `docs/research/`，結論寫成對應 `facts-*.md` 的一條事實。
- 每張票與里程碑收尾：`mattpocock-skills:code-review`（Spec 軸讀票，Standards 軸讀本檔）；難 bug 或效能退化：`mattpocock-skills:diagnosing-bugs`。
- 改本檔或 progress.md 的流程段：`mattpocock-skills:writing-for-agents`。

## Agent skills

### Issue tracker

Local markdown：票以檔案存在 `.scratch/<里程碑>/issues/`。See `docs/agents/issue-tracker.md`.

### Triage labels

五個預設角色標籤，字串與角色名相同。See `docs/agents/triage-labels.md`.

### Domain docs

Single-context：根目錄 `GLOSSARY.md` + `docs/adr/`。See `docs/agents/domain.md`.
