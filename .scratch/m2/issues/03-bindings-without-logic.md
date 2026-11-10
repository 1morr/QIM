# 03: 綁定層不放邏輯，按鍵只有一個入口

Status: done
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

- [x] `qim-wasm` 沒有分支決定按鍵行為、頁面或組字狀態的程式碼；移出去的邏輯在 `qim-core` 或 `qim-layout` 有單元測試。
- [x] 網頁的按鍵都經同一個綁定入口。
- [x] `cargo xtask bench --check` 綠燈且數字不變。
- [x] `cargo xtask check` 綠燈（含 e2e）；亂按一輪找到的問題先寫成會紅的 Playwright 測試再修，記在 `## Comments`。

## Comments

- 2026-11-10 哪一邊放什麼：
  - `qim-core`（按鍵）：`Key::parse(kind, value)` 收「種類 + 值」，綁定的 `press` 原樣轉交，加一種鍵只改核心與 `web/src/keys.ts`（同 librime 能從字串讀按鍵）；`Session::composing()` 取代 `WebSession.composing`；`Session::options()` 取代綁定另存的 `Options`；`ShiftSwitch: FromStr` 取代綁定的 `WebShiftSwitch`，`toggleAscii` 的左右也經 `Key::parse`，`WebShiftKey` 刪掉。
  - `qim-layout`（佈局的狀態與動作）：`Keyboard` 記佈局版本、目前頁、觸控 Shift，`tap` / `long_press` 做完切頁與 Shift 後回傳 `Output`（切頁、⇧、換鍵盤都是 `Modifier`），取代 `WebLayout` 的欄位與 `perform`。
  - 綁定留下的：`Output` 一對一轉 `Key`（中英鍵 → `ToggleAscii(Right)`、文字 → `Session::text`），plan §1.3 不准 `qim-layout` 依賴 `qim-core`，這段只能在綁定，`qim-ffi`（票 07）照抄；錯誤經 `js_error` 一處轉 `JsError`。
- 2026-11-10 行為差異（清理票的例外，前後對照）：
  - 載入佈局時的初始頁：舊版先把頁設成第 0 頁再選版本，選到的版本有同 id 的頁就停在那頁，`initialPage` 等於被忽略；現在照 plan §2.6 顯示 `initialPage`。內建佈局沒寫 `initialPage`、第一頁就是字母頁，網頁看不出差別。`keyboard.rs` 的夾具 `initialPage` 不在第一頁，把 `Keyboard::new` 改回舊選法時 5 個測試紅。
  - 綁定收到不認得的字串（`setShiftSwitch`、`press("toggleAscii", …)`）改成丟例外，舊版默默當 `Rime` / 右 Shift；網頁的 `shiftSwitch()` 先過濾，碰不到。
  - 觸控的 `Snapshot.action` 是 `Output::kind`：切頁、⇧、換鍵盤由 `switchPage`、`shift`、`nextKeyboard` 變成 `modifier`；網頁只看 `letter`、`punct`、`space`、`enter`、`backspace`，看不出差別。
- 2026-11-10 亂按一輪（Playwright 手動，桌面與 412×839 觸控）：組字中夾 ← → 數字選段、`-` `=` 翻頁、Tab、↓ ↑、退格、Esc、`'` 分隔、Cmd+A、CapsLock、Home；左右單按 Shift 進出臨時英文與英文直通；數字後 `.`；觸控組字中切符號頁再切回、⇧ 當分隔、Shift 開一次與快速點兩下鎖定、快速連點六鍵、組字中切簡體與數字列、組字中長按、轉向留在同頁。沒有找到問題，console 沒有錯誤。
- 2026-11-10 驗證：`cargo xtask bench --check` 通過、四個集合沒有翻盤、固定題 84/84，baseline 沒動；`cargo xtask check` 通過（e2e 147 passed）。
- 2026-11-10 code-review 沒處理的發現：
  - 新的錯誤訊息（`KeyError`、`KeyboardError`、`ShiftSwitch` 的 `Err`）是中文，全域偏好說字串常量用英文，但 repo 的錯誤訊息一貫中文、網頁直接顯示，照 repo。
  - `ShiftSwitch` 的名字是 kebab-case（網頁選單原有的值）、`Key` 的種類是 camelCase（同佈局動作名）；`ShiftSwitch` 的 `Err` 是 `String`（同 `Device`、`CangjieVersion`）。都照既有慣例，不改。
  - `Output::kind` 與 `Action::name` 是兩張相近的名字表：`Output` 少了切頁、⇧、換鍵盤，多了 `modifier`，不共用。
  - JS 端的 `Layout` 現在包的是有狀態的 `Keyboard`，改名要動網頁介面，留著。

