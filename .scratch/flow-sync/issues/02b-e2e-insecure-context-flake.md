# 02b: e2e 非安全環境測試在全套跑時紅

Status: ready-for-agent
Blocked by: 02
讀: `web/e2e/insecure-context.spec.ts`；`docs/development.md`〈網頁〉〈全部檢查〉；票 02 的 Comments（紅燈的紀錄）

協調者插的修復票：flow-sync 的票 02 跑全套 `cargo xtask check` 三次都停在 `e2e`，同一條測試連續紅，照協調流程算穩定紅，排在下一張票之前修。flow-sync spec〈範圍〉只列文件與腳本，這張票為了讓每張票的完成標準（全部檢查綠燈）做得到，例外動 `web/e2e/`；不改產品行為。

## What to build

全套 `cargo xtask check` 的 `e2e` 步驟穩定綠燈，之後的票（arch-diet 起）不再卡在這條。

票 02 看到的現象：324 passed，另有 1 failed 或「1 error was not a part of any test」，出在 `web/e2e/insecure-context.spec.ts` 的 `beforeEach` 註冊的 `context.route` handler：測試結束、context 關掉時 `route.fetch` 還沒回來。單獨跑這個檔 2 passed。

- 先分清是產品還是測試的問題：頁面在測試結束後還在發哪些請求（資料包、Worker、模型），那些請求本來就該有，還是產品漏了收尾。
- 先做出會紅的 repro（例如連跑、或和別的檔一起跑），寫進票的 Comments，修完同一個 repro 要綠。
- 原因說得出來才修；只拉長逾時或加重試而說不出原因的不收。

## 驗收

- [ ] Comments 寫明原因（產品或測試）與會紅的 repro 指令；修完同一個 repro 綠燈。
- [ ] 全套 `cargo xtask check` 連跑兩次都綠燈（要先 `cargo xtask data`），貼兩次的結尾輸出。
- [ ] 票 02 的 Comments 補一行：全套 check 的紅燈由 02b 修好。

## Comments
