# 07: tracer bullet：網頁打 `a` 上屏「日」

Status: done
Blocked by: 05
讀: plan §1.1、§1.3、§2.1、§3.1；brief §4.3；spec〈範圍〉〈共同規則〉

## What to build

使用者打開網頁 prototype，在輸入框按 `a`，組字顯示 `a`、候選列出現「日」；按空白，首選進入輸入框（真資料的首選是「曰」，見 Comments）。從資料包、核心、WebAssembly 綁定到網頁一路打通。

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

- [x] Playwright：打 `a`，讀回候選含「日」；按空白，讀回輸入框為首選「曰」、組字已清空（「日」改由票 08 的 `a` `2` 驗收）。
- [x] 截圖存證：`.scratch/m0/evidence/07-type-a.png`、`.scratch/m0/evidence/07-type-a-space.png`。
- [x] `qim-core` 單元測試：打 `a` 的首選是測試資料包裡 `a` 的第一個字；空白上屏首選並清空；沒有組字時空白不被攔截。
- [x] `cargo xtask check` 綠燈，`deps`（若票 02 已完成）不因新 crate 紅。

## Comments

- 2026-11-03 首選是「曰」：libcangjie 的 Big5 排序值「曰」23060、「日」23061，`a` 只有這兩字。M0 沒有字頻（spec〈範圍〉字序照 Big5），空白上屏「曰」；e2e 驗「候選含日、空白上屏首選」，「日」改由票 08 的數字鍵驗收，spec〈驗收〉同步改。
- 2026-11-03 `Snapshot` 加 `handled`（plan §2.1）：殼要知道鍵有沒有被吃下，才能在沒有組字時把空白交還宿主；同 librime `process_key` 的回傳值、fcitx5 的 `filterAndAccept`。
- 2026-11-03 `Snapshot` 怎麼傳給 JS：`qim-wasm` 的 `WebSnapshot`（JS 叫 `Snapshot`）是 `getter_with_clone` 的唯讀欄位類別，只有網頁用到的 `handled`、`input`、`candidates`、`commit`，其他欄位等用到再加。不用 `serde-wasm-bindgen` 或 JSON：plan §1.3 只准 `qim-wasm` 依賴 `wasm-bindgen`。網頁讀完欄位要 `free()`。
- 2026-11-03 按鍵：`qim-wasm` 每種鍵一個方法（`letter`、`space`），`wasm-bindgen` 不支援帶資料的 enum；網頁 `web/src/keys.ts` 的 `toKey` 只認 `a`–`z` 與空白，修飾鍵、IME 組字中的事件放行。核心另外不收方案字母表以外的字母（`z` 交還宿主）。
- 2026-11-03 建置：`cargo xtask web` 編綁定到 `web/src/wasm/`、複製 `build/packs/` 到 `web/public/packs/`。`check` 加 `wasm`（`--fast` 也跑，因為 `tsc` 要讀綁定的型別，pre-commit 因此要裝 `wasm-bindgen-cli`）與 `e2e`（`--fast` 跳過）。`wasm-bindgen-cli` 版本與 `Cargo.lock` 不同時擋下並印安裝指令；`rust-toolchain.toml` 加 `wasm32-unknown-unknown`。
- 2026-11-03 驗收輸出：`cargo xtask check` 各步驟通過，`qim-core` 的 `tests/session.rs` 6 passed，e2e `1 passed (720ms)`，最後一行 `check 通過`；`deps` 對新的 `qim-wasm` 綠燈。
- 2026-11-03 code-review 已處理：plan §1.2、§6 補 `web` 指令與 `wasm`、`e2e` 步驟；progress 補 `handled` 的偏差；`Ime` 改名 `WebSession`（JS 叫 `Session`，名詞表「輸入法」只指產品）；wasm `Snapshot` 拿掉沒用到的欄位；`check.rs` 的重複 `match` 改 `let … else`；補「有輸入碼但沒候選時空白不上屏」的核心測試。
- 2026-11-03 code-review 沒處理的發現：
  - `toKey` 的分支（修飾鍵、`isComposing`）沒有單元測試：`web/` 沒有單元測試工具，為一個函式加 vitest 不划算；票 08 的 e2e 會碰到更多鍵時再看。
  - 加一種鍵要改四處（`qim_core::Key`、`qim-wasm` 方法、`keys.ts`、`main.ts` 的 `press`）：綁定層不能傳帶資料的 enum，是 plan §1.3 薄綁定的代價。
  - 組字中按 Backspace、數字或 `z` 會直接改到輸入框而組字不清：Backspace、數字是票 08 的範圍。
  - 測試裡的 `Ime` 輔助結構與 `WebSession` 同形：測試不能依賴綁定 crate，保留。
- 2026-11-04 首選「曰」由票 08b 改掉：字序加 Unihan `kGradeLevel`，`a` 的首選是「日」，空白上屏「日」；見票 08b 的 Comments。
