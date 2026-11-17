# 02b: e2e 非安全環境測試在全套跑時紅

Status: done
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

- [x] Comments 寫明原因（產品或測試）與會紅的 repro 指令；修完同一個 repro 綠燈。
- [x] 全套 `cargo xtask check` 連跑兩次都綠燈（要先 `cargo xtask data`），貼兩次的結尾輸出。
- [x] 票 02 的 Comments 補一行：全套 check 的紅燈由 02b 修好。

## Comments

原因：測試的問題，不是產品漏了收尾。輸入框 `data-ready` 之後頁面照設計還在背景載入：停鍵層 Worker（`src/rerank-worker.ts`）載入自己的 `qim_wasm.js`、`qim_wasm_bg.wasm`、Vite 的 `env.mjs`（`src/main.ts`「打字不等它」），以及線上更新檢查 `/pack-updates/index.json`（`src/pack-settings.ts` 的 `void runUpdates(false)`）。這幾條請求都經過 `beforeEach` 的 `context.route` 轉發；測試先結束時 `route.fetch` 還沒回來，context 關掉就丟「route.fetch: Test ended.」，記成當時那條測試的錯，或「1 error was not a part of any test」。在 handler 插樁記錄，失敗的都是上面這幾條。

會紅的 repro（要先 `cargo xtask data`、`cargo xtask web`）：`cd web && npx playwright test e2e/insecure-context.spec.ts --repeat-each 20 --workers 8`。修前 40 次紅 13 次。

修法：`test.afterEach` 呼叫 `context.unrouteAll({ behavior: "ignoreErrors" })`，只在測試結束後才不理會轉發到一半的請求丟的錯，測試期間的錯照樣丟出來。先試過 `behavior: "wait"`，仍偶發「Route is already handled!」：Playwright 1.64 先等已開始的 handler，再更新攔截規則；更新規則的當下剛進 handler 的請求會被自動放行（`removeHandler` → `continue({ isFallback: true })`），handler 之後再 fulfill 就丟錯。修後同一個 repro 連跑 5 輪共 200 次全綠。

其他用 `route.fetch` 的 e2e（`storage.spec.ts` 截斷模型資料包、`pack-update.spec.ts` 改 sha256）都等到回應處理完的畫面才結束測試，不會有同樣的問題，沒改。

全套 `cargo xtask check`（已跑過 `cargo xtask data`）連跑兩次，兩次的結尾：

```
  325 passed (1.2m)
check 通過
```

code-review：Standards、Spec 兩軸都沒有要改的；Spec 軸提的票 02 補一行已補上。

Session 紀錄：2026-11-18 找出 insecure-context e2e 在測試結束後轉發背景請求而紅，afterEach 拿掉轉發並不理會之後的錯，全套 check 連兩次綠
決定：判定是測試的問題，在測試結束前 `unrouteAll` 收掉轉發，不改產品、不讓測試多等 Worker 與更新檢查跑完（Agent 定）
決定：`unrouteAll` 用 `ignoreErrors` 不用 `wait`：`wait` 在拿掉轉發的當下仍會讓剛進 handler 的請求丟「Route is already handled!」（Agent 定）
變更：無
