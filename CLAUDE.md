# QIM

全平台速成輸入法。文件是唯一的記憶，對話不是。

## 文件

- `docs/design-brief.md`：需求與決定。決定衝突時以它為準。
- `docs/research/facts-*.md`：查證過的外部事實，每條有編號與來源；索引在 brief〈事實索引〉。
- `docs/plan.md`：架構、依賴規則、測試策略（`.scratch/arch-diet/` 做完後縮成根目錄的 `ARCHITECTURE.md`）。
- `docs/roadmap.md`：里程碑的順序、狀態與閱讀指引；還沒開工的里程碑各有一小節，開工後縮成一行連到它的 spec；〈不排里程碑〉收沒人接的遺留。
- `docs/progress.md`：進行中里程碑的 session 紀錄。每個 session 的起點與終點。
- `docs/decisions.md`：進行中里程碑的決策紀錄，一條一行，標決定者；驗收後跟著封存。
- `.scratch/<里程碑>/spec.md`：該里程碑的目標、範圍、共同規則、驗收（Agent 驗、使用者試）與票序；由下面〈Skills〉的里程碑開工步驟建立。`issues/` 是票，執行單位；狀態在票的 `Status:` 行。
- `GLOSSARY.md`、`docs/adr/`：名詞表與架構決定，由 `/grill-with-docs` 按需建立；命名一律用名詞表的詞。

## 目錄

| 路徑 | 內容 |
| --- | --- |
| `crates/` | Rust crate；各 crate 的職責與依賴規則見 plan §1.2、§1.3 |
| `data/` | 方案檔、來源清單 `sources.toml`、自行整理的詞表 `lexicon/`、進資料包的授權全文 `licenses/`、停鍵層採用的模型 `neural-rerank/`；建置方式見 `docs/development.md`〈資料建置〉〈停鍵層的模型〉 |
| `bench/` | 首選字基準的題目與語料（plan §5.2）；`corpus/`、`colloquial/` 由 `cargo xtask data` 產生，`personal/` 只在本機 |
| `xtask/` | 閘門與開發指令（`cargo xtask check`），用法見 `docs/development.md` |
| `web/` | 網頁 prototype（Vite + TypeScript + Biome） |
| `docs/` | brief、plan、roadmap、progress、decisions、`development.md`、ADR、研究、已完成里程碑的封存 `archive/` |
| `.scratch/` | 里程碑的 spec 與票 |
| `.githooks/` | git hook，啟用方式見 `docs/development.md` |
| `.github/` | GitHub Actions 的 CI，跑什麼見 `docs/development.md`〈全部檢查〉 |
| `scripts/` | 腳本：文件檢查 `check_docs.py`（`cargo xtask check` 的 `docs` 步驟跑它，要 Python 3）、停鍵層模型的訓練在 `scripts/neural-rerank/`，設計實驗在 `scripts/experiments/` |

## 使用者確認

- brief、plan、ADR、`GLOSSARY.md` 裡的決定：草稿先給使用者看，確認後才寫入；訪談中也一樣。
- 做票時要推翻已定的決定（brief、plan、ADR、spec、`docs/decisions.md` 裡的）：停下來問使用者（協調者開的 session 問協調者，協調者照文件答不了的再問使用者）。同意後，同一 commit 改文件，並記一條決定。
- 問使用者決定（對話裡，或 progress.md 的 `等使用者：`）：每題寫成沒讀過票也看得懂的話，依序是背景一句、選項、你的建議、不決定會卡住什麼；票號、函式名、章節號後面跟一句它是什麼。
- 使用者回「照建議」，只算同意這一則訊息裡列出來的那幾題。
- 每個決定在 `docs/decisions.md` 佔一行並標**決定者**：`（使用者定）` 使用者自己定的、`（使用者同意）` agent 提議而使用者同意的、`（Agent 定）` 使用者沒看過的（協調者定的也算）。做票時在兩種做法之間選了一種，就是一個 `（Agent 定）` 的決定；`/1morr:checkpoint` 會把它們拿給使用者看。

## 每個 session 的流程

工作單位是**票**：`.scratch/<里程碑>/issues/NN-slug.md`，由里程碑開工的 session 產生（見〈Skills〉）。一個 session 只做一張票。

1. 讀 `docs/progress.md`、`docs/decisions.md` 裡這個里程碑的決定、該里程碑的 spec，再讀票；只讀票列出的 plan / brief 章節，其他按需讀。
2. `/implement <票路徑>`：tdd、定期 typecheck、結尾 `/code-review <開工時的 commit>`、commit。
3. 驗證：`docs/development.md`〈全部檢查〉加 test；貼指令輸出。
4. 影響架構的決定另寫 ADR；要推翻已定的決定照〈使用者確認〉。
5. 票的 `Status:` 改為 `done`、勾掉驗收條件、code-review 未處理的發現記到票的 `## Comments`；Comments 最後加一行 `Session 紀錄：<做了什麼，一句>`，做票時定的每個選擇各加一行 `決定：<一句>（<決定者>）`；commit。票的分支不改 progress.md 與 decisions.md：平行的票都改它們會互撞。
6. 合併的人（使用者自己開的 session 是它自己，`/1morr:ticket-coordinator` 開的是協調者）合併後在主分支把那兩種行抄進去（progress.md 一列 session 紀錄含下一步、decisions.md 每個決定一條）並 commit。收尾票不會平行，自己改這兩個檔。

完成標準：票上每個驗收條件打勾、全部檢查與 test 綠燈、Comments 有 `Session 紀錄：` 一行、已 commit。缺一就還沒完成。

**所有 session 的結尾**（做票、訪談、寫架構、里程碑開工都算）：告訴使用者下一步要在新 session 下的完整指令，寫 progress.md 的人把同一句寫進那一列的「下一步」。下一張票是 `/implement <票路徑>`，里程碑做完是 `/1morr:checkpoint <剛做完的里程碑>`（回顧上一段、和使用者對齊下一段；它結束時寫 `/to-tickets docs/roadmap.md 拆 <下一個里程碑> 的票`）；要使用者先決定才能繼續時，寫 `等使用者：<要決定什麼>`。

## 事實與查證

- 平台輸入法框架（macOS InputMethodKit、Windows TSF、Linux Fcitx5 / IBus、iOS 鍵盤擴充、Android InputMethodService）、碼表的來源與授權、外部引擎（如 Rime）的行為只信兩處：`docs/research/facts-*.md` 的〈事實〉段，以及現查（context7、官方文件）。沒寫的先查再用，查到的補進對應主題的檔，給編號、附來源。
- 程式碼、測試與註解要指向文件時，引 ADR 編號、事實編號或節的標題；brief 與 plan 的章節號只寫在文件與票裡，節才能搬動（`scripts/check_docs.py` 擋）。
- 現況以指令為準：`git log`、`git status`、測試輸出。progress.md 與 repo 不符時以 repo 為準並修 progress.md。

## 上下文紀律

- 讀大檔案、查文件、跑實驗交給子代理回摘要。
- brief 與 plan 只讀任務指到的章節。字元上限由 `scripts/check_docs.py` 強制：progress.md 20,000、brief 30,000；超過時搬走或刪掉內容，不靠縮短措辭湊數。
- 對話已長或一個任務碰了十幾個檔案：把進度與下一步寫進 progress.md，結束 session。

## 驗收輸入行為

輸入法的行為要在真的宿主程式裡打字並讀回畫面才算數：桌面平台用 `orca computer`，iOS 模擬器用 `orca emulator`，Android 用 `orca emulator-android`。單元測試綠燈不能取代這一步。

## Skills

- 需求與設計訪談：`/grill-with-docs`（邊問邊寫 `GLOSSARY.md` 與 ADR），結論寫回 brief。
- 里程碑開工（使用者在新 session 下 `/to-tickets docs/roadmap.md 拆 <里程碑> 的票`）、做收尾票 `NN-m<N>-close`：先讀 `docs/agents/milestone.md`，照它的步驟做。之後的里程碑留在 roadmap，開工時才有 spec。
- 做票：一張票一個 session，`/implement`；整批跑票：`/1morr:ticket-coordinator`。
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
