# 07: tracer bullet：網頁打 `a` 上屏「日」

Status: ready-for-agent
Blocked by: 05
讀: plan §1.1、§1.3、§2.1、§3.1；brief §4.3；spec〈範圍〉〈共同規則〉

## What to build

使用者打開網頁 prototype，在輸入框按 `a`，組字顯示 `a`、候選列出現「日」；按空白，「日」進入輸入框。從資料包、核心、WebAssembly 綁定到網頁一路打通。

- `qim-core`：
  - `Engine` 由資料包位元組建立。
  - `Session::handle` 處理 `Letter` 與 `Space`，回傳 plan §2.1 的 `Snapshot`（欄位照 §2.1；本票用不到的給預設值）。
  - `UserModel`、`Options` 只建型別讓簽章成立。
  - 查碼：整串輸入碼查碼表，候選每頁 9 個（spec〈共同規則〉）。
  - 空白：有候選時上屏首選並清空組字；沒有組字時不攔截，交給宿主。
- `qim-wasm`：用 `wasm-bindgen`，只轉接、不放邏輯（plan §1.3）。`Snapshot` 怎麼傳給 JS 由本票決定，記在 `## Comments`。
- 建置：一個 xtask 指令編 WebAssembly（`wasm-bindgen-cli`）、把資料包複製到 `web/public/packs/`；寫進 `docs/development.md`。
- 網頁：輸入框、組字顯示、候選列；鍵盤事件轉成 `Key`，`Snapshot.commit` 插入輸入框游標處。
- Playwright：照 plan §3.1 用 `page.keyboard.type` 打字，讀回候選列與輸入框，截圖存證。e2e 的執行指令加進〈全部檢查〉。
- `qim-core` 的單元測試用測試內建的小資料包，不依賴下載的來源。

## 驗收

- [ ] Playwright：打 `a`，讀回候選含「日」；按空白，讀回輸入框為「日」、組字已清空。
- [ ] 截圖存證（貼路徑）。
- [ ] `qim-core` 單元測試：打 `a` 的首選是測試資料包裡 `a` 的第一個字；空白上屏首選並清空；沒有組字時空白不被攔截。
- [ ] `cargo xtask check` 綠燈，`deps`（若票 02 已完成）不因新 crate 紅。
