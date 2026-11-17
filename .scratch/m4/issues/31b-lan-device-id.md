# 31b: 區網 http 開網頁時裝置 id 產生失敗

Status: ready-for-agent
Blocked by: None (can start immediately)
讀: `web/src/user-dict.ts` 的 `ensureDeviceId`；plan §2.4（裝置 id、同步）；spec〈共同規則〉

## What to build

收尾票 31 的使用者試用插入的修復票（使用者 2026-11-17 決定）。手機連同一個 Wi-Fi 用區網 IP（`http://192.168.x.x:5174/`）開網頁，頁面顯示「載入失敗：TypeError: crypto.randomUUID is not a function」，手機試用做不了；這是之前就有的問題。原因：`crypto.randomUUID()` 只在安全環境（HTTPS 或 localhost）提供，區網 http 不是安全環境；`ensureDeviceId` 第一次產生裝置 id 時呼叫它。

- 沒有 `crypto.randomUUID` 時改用 `crypto.getRandomValues`（非安全環境也有）組 RFC 4122 第 4 版 UUID（版本與變體位元照規範），有 `randomUUID` 時照舊用它。
- 查一遍 `web/` 其他只在安全環境才有的 API（例如 `navigator.clipboard`、Service Worker），有的話一起處理或在 Comments 列出不處理的理由。

## 驗收

- [ ] 會紅的 repro：單元測試把 `crypto.randomUUID` 拿掉後產生裝置 id，修之前紅、修之後綠；產生的字串符合 UUID v4 格式（版本位是 4、變體位是 8–b）。
- [ ] Playwright 或實跑：用非 localhost 的位址（例如本機區網 IP）開網頁，頁面正常載入、讀回畫面沒有「載入失敗」；附截圖或讀回的畫面文字。
- [ ] `cargo xtask check` 綠燈。
- [ ] 修好後回到票 31，請使用者在手機上試用（票 31 的「使用者試用第二期」那條）。
