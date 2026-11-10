# 01: 網頁拆模組，留出設定頁

Status: ready-for-agent
Blocked by: None (can start immediately)
讀: plan §3.1；spec〈共同規則〉；`.scratch/m1/issues/24-m1-close.md` Comments 的「M3」各條（`main.ts` 那條）

## What to build

M3 會往網頁加很多東西（上下文、聯想、常用語、詞庫匯入、資料包管理、雲端開關）。先把 `web/src/main.ts`（655 行）拆開，讓之後的票各自加模組，不再往同一個檔堆。不改行為。

- 候選列與展開面板抽成模組（M1 code-review 建議）；現有的開關選單（簡繁、無痕、重排、組字中單按 Shift、英文候選、數字後半形）收成一個設定區，新票加開關時只在這裡加一列。
- 網頁仍只透過綁定呼叫核心，不搬任何排序或組字邏輯進 TS（plan §1.3）。

## 驗收

- [ ] `main.ts` 只剩組裝與事件接線；候選列、面板、設定區各自一個模組。
- [ ] Playwright e2e 全過，測試不必改行為斷言（只動選擇器時在 Comments 說明）。
- [ ] 實際操作網頁亂按一輪，行為和拆之前相同。
- [ ] `cargo xtask check` 綠燈。

## Comments
