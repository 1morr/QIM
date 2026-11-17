# 31b: 區網 http 開網頁時裝置 id 產生失敗

Status: done
Blocked by: None (can start immediately)
讀: `web/src/user-dict.ts` 的 `ensureDeviceId`；plan §2.4（裝置 id、同步）；spec〈共同規則〉

## What to build

收尾票 31 的使用者試用插入的修復票（使用者 2026-11-17 決定）。手機連同一個 Wi-Fi 用區網 IP（`http://192.168.x.x:5174/`）開網頁，頁面顯示「載入失敗：TypeError: crypto.randomUUID is not a function」，手機試用做不了；這是之前就有的問題。原因：`crypto.randomUUID()` 只在安全環境（HTTPS 或 localhost）提供，區網 http 不是安全環境；`ensureDeviceId` 第一次產生裝置 id 時呼叫它。

- 沒有 `crypto.randomUUID` 時改用 `crypto.getRandomValues`（非安全環境也有）組 RFC 4122 第 4 版 UUID（版本與變體位元照規範），有 `randomUUID` 時照舊用它。
- 查一遍 `web/` 其他只在安全環境才有的 API（例如 `navigator.clipboard`、Service Worker），有的話一起處理或在 Comments 列出不處理的理由。

## 驗收

- [x] 會紅的 repro：單元測試把 `crypto.randomUUID` 拿掉後產生裝置 id，修之前紅、修之後綠；產生的字串符合 UUID v4 格式（版本位是 4、變體位是 8–b）。
- [x] Playwright 或實跑：用非 localhost 的位址（例如本機區網 IP）開網頁，頁面正常載入、讀回畫面沒有「載入失敗」；附截圖或讀回的畫面文字。
- [x] `cargo xtask check` 綠燈。
- [ ] 修好後回到票 31，請使用者在手機上試用（票 31 的「使用者試用第二期」那條）。

## Comments

- repro：`web/test/device-id.test.ts` 把 `crypto` 換成只有 `getRandomValues` 的物件，修之前紅（`TypeError: source.randomUUID is not a function`，同手機上的錯），修之後綠；另有全 0xff 的固定例子鎖住版本位 4、變體位 `10`（`4fff`、`bfff`）。產生的邏輯抽成 `web/src/device-id.ts` 的 `newDeviceId`，`user-dict.ts` 的 `ensureDeviceId` 改呼叫它。
- e2e：`web/e2e/insecure-context.spec.ts` 讓頁面的網址是 `http://qim-lan.test`，請求由 Playwright 攔下轉給開發伺服器，頁面跑在真的非安全環境（`isSecureContext` 是 false）；把 `user-dict.ts` 暫時改回 `crypto.randomUUID()` 時紅（頁面沒到 `data-ready`），修之後綠。
- 實跑：開發伺服器加 `--host`，Playwright 開本機區網 IP（`http://192.168.x.x:25174/`）：`isSecureContext` false、`crypto.randomUUID` 與 `navigator.clipboard` 都是 undefined；頁面就緒、`qim.device` 是合格的 UUID v4、畫面沒有「載入失敗」；打 `anr ` 上屏「門口」；按「複製全部」狀態列讀回「這個網址不是 HTTPS 或 localhost，瀏覽器不給網頁寫剪貼簿：已選取回報清單，請自己複製」。截圖 `.scratch/m4/evidence/31b-lan-http-loaded.png`。console 兩條錯誤是沒有 favicon 與線上更新來源 404，原本就有、和本票無關。
- 其他只在安全環境才有的 API：`web/src`、`web/index.html` 只用到 `navigator.clipboard`（回報區的「複製全部」，非安全環境下按了會丟 TypeError），一起處理；沒有 Service Worker、CacheStorage、`crypto.subtle`、`navigator.storage`、`navigator.share`、`navigator.keyboard`；`web/public/` 沒有 Service Worker 或 manifest。生成的 wasm 綁定只用 `crypto.getRandomValues`（Rust `getrandom`），非安全環境也有。
- code-review（`8b784aa` 起，Standards／Spec 兩軸）：已處理：`mountReportPanel` 的註解補非安全環境的說明；iOS Safari 的 `select()` 不一定選得起來，改用 `setSelectionRange`；單元測試拿掉 100 個 id 不重複的斷言（在測 `getRandomValues` 本身）；plan §2.4（學習與排位）的裝置 id 補非安全環境的做法；截圖存進 evidence。未處理：UUID v4 的 regex 在單元測試與 e2e 各寫一份（兩邊跑在不同環境）；`DeviceIdCrypto` 介面只為了測試注入，照 `today.ts` 的 `localToday(now)` 慣例保留；註解引 RFC 4122（已被 RFC 9562 取代，位元排法相同，票也寫 4122）。
- 沒做：〈共同規則〉的「亂按一輪」只在區網位址打字與按複製，沒有再整輪亂按（本票只改裝置 id 的產生與複製按鈕）。手機上 `setSelectionRange` 選不選得起來，Chromium 驗不到，請使用者在票 31 試用時順便按一下「複製全部」。
- 第 4 條驗收（回到票 31 請使用者在手機試用）由協調者做，本 session 不勾。
- 全部檢查：`cargo xtask check` 通過（e2e 325 passed、2 skipped；本 worktree 先 `npm ci`、`cargo xtask data`）。跑 e2e 會重新產生票 26 的四張證據截圖（像素差異），沒有 commit。

Session 紀錄：區網 http 不是安全環境，裝置 id 沒有 `crypto.randomUUID` 時改用 `getRandomValues` 組 UUID v4，回報區的「複製全部」沒有 `navigator.clipboard` 時選取清單請使用者自己複製；補 repro 單元測試與非安全環境的 e2e。
決定：沒有 `navigator.clipboard` 時選取回報清單並在狀態列說明，不用已 deprecated 的 `document.execCommand("copy")`（Agent 定）
決定：非安全環境的 e2e 用 Playwright 攔截假主機名 `qim-lan.test` 轉給開發伺服器，不讓 e2e 的開發伺服器聽區網介面（Agent 定）
