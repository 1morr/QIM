# 03: 綁定層不放邏輯，按鍵只有一個入口

Status: ready-for-agent
Blocked by: 02
讀: plan §1.2、§1.3、§2.1、§2.6、§3.1；ADR 0002、0004；spec〈共同規則〉；`.scratch/m1/issues/24-m1-close.md` Comments 的「M2」各條（`qim-wasm` 放了邏輯、加一種鍵要改四處兩條）

## What to build

網頁與之後的原生殼呼叫同一套核心與 `qim-layout` 介面，`qim-wasm` 只做型別轉換。做完後加一種鍵只要改核心與網頁的按鍵對照，`qim-ffi`（票 07）照著包就好。用 `mattpocock-skills:codebase-design` 定介面。

- `perform` 把佈局動作轉成 `Key` 並切頁、`WebLayout` 記佈局版本與目前頁：收進 `qim-layout`（佈局的狀態與動作）與 `Session`（按鍵）；哪一邊放什麼寫在 `## Comments`。
- `WebSession.composing` 另存的組字狀態：改由 `Session` 或 `Snapshot` 提供。
- 按鍵入口：`qim-wasm` 現在一種鍵一個方法（`letter`、`punct`、`enter`、`left`…約 13 個），改成一個入口收 `Key`（或「種類 + 值」），`keys.ts`、`main.ts` 跟著改。
- `JsError` 轉換重複約 8 次：收成一處。
- 行為不變：Playwright e2e 全過；動到網頁，收尾照 M1 的習慣實際操作網頁亂按一輪（組字中夾各種鍵、快速連按、切換模式、觸控切頁）。

## 驗收

- [ ] `qim-wasm` 沒有分支決定按鍵行為、頁面或組字狀態的程式碼；移出去的邏輯在 `qim-core` 或 `qim-layout` 有單元測試。
- [ ] 網頁的按鍵都經同一個綁定入口。
- [ ] `cargo xtask bench --check` 綠燈且數字不變。
- [ ] `cargo xtask check` 綠燈（含 e2e）；亂按一輪找到的問題先寫成會紅的 Playwright 測試再修，記在 `## Comments`。
