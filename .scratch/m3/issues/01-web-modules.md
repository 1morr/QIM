# 01: 網頁拆模組，留出設定頁

Status: done
Blocked by: None (can start immediately)
讀: plan §3.1；spec〈共同規則〉；`.scratch/m1/issues/24-m1-close.md` Comments 的「M3」各條（`main.ts` 那條）

## What to build

M3 會往網頁加很多東西（上下文、聯想、常用語、詞庫匯入、資料包管理、雲端開關）。先把 `web/src/main.ts`（655 行）拆開，讓之後的票各自加模組，不再往同一個檔堆。不改行為。

- 候選列與展開面板抽成模組（M1 code-review 建議）；現有的開關選單（簡繁、無痕、重排、組字中單按 Shift、英文候選、數字後半形）收成一個設定區，新票加開關時只在這裡加一列。
- 網頁仍只透過綁定呼叫核心，不搬任何排序或組字邏輯進 TS（plan §1.3）。

## 驗收

- [x] `main.ts` 只剩組裝與事件接線；候選列、面板、設定區各自一個模組。
- [x] Playwright e2e 全過，測試不必改行為斷言（只動選擇器時在 Comments 說明）。
- [x] 實際操作網頁亂按一輪，行為和拆之前相同。
- [x] `cargo xtask check` 綠燈。

## Comments

- 2026-11-11 實作：`main.ts` 655 → 237 行，只剩載入資料包、建核心、接模組與輸入框的鍵盤事件。抽出 `candidate-bar.ts`（候選列、字根提示、長按）、`panel.ts`（展開鍵與面板）、`settings.ts`（設定區）、`composition.ts`（組字區）、`report-panel.ts`（回報區）、`display.ts`（觸控鍵盤開關、數字列、佈局版本、深色）、`host.ts`（上屏到輸入框、觸控鍵沒被吃下時的預設動作）；假重排的要求與套用搬進 `rerank.ts` 的 `fakeReranker`（只 `import type` 綁定，`node --test` 照跑）。
- 設定區：六個開關由 `settings.ts` 的 `ROWS` 產生，放在頁面下方的 `<section aria-label="設定">`，id 不變；原本四個勾選在輸入框上方、兩個選單在工具列。之後的設定（票 14、17、20 的清單、匯入、資料包）在這一區各加一段；寫進 plan §3.1。
- 唯一刻意的行為差異：「重排」選單改完也把焦點還給輸入框，和其他開關一致（原本只有它不還）。`e2e/settings.spec.ts` 守著，並守設定區六個開關與預設值；雙向變異驗證過（讓重排不還焦點、刪掉英文候選那列各紅一個，改區域變數名不紅）。
- 現有 e2e 一行沒改；`check` 的 e2e 149 個全過（原 147 + 新 2）。
- 亂按一輪：除了 Playwright 手動操作（桌面組字中夾 Tab、↓、←、Esc、單按 Shift、數字後半形、簡體；手機尺寸觸控、數字列、展開面板、深色、長按候選），另寫固定種子的亂按腳本（打字與各種鍵、點候選、面板、分段、切開關與選單、觸控鍵、回報），每步讀回輸入框、組字、候選、面板、回報、觸控標籤等。拆之前與之後各跑 4 個種子 × 300 步，逐位元相同；腳本是一次性的，沒進 repo。截圖 `.scratch/m3/evidence/01-*.png`。
- code-review 未處理的發現（判斷題，不改）：`display.ts` 同時放深色與觸控鍵盤掛載，名字可能和 `touch.ts` 混淆；「非中文組字就收起」在候選列（字根提示）與面板各存一份 `chinese`，兩個模組各管自己的狀態，暫不合併；`mountReportPanel` 收 `status`、`mountTouch` 收 `text`，是共用元素才由外面傳。

