# 18: 佈局 tracer：觸控點出「門口」

Status: done
Blocked by: 07
讀: plan §1.2、§1.3、§2.6、§3.1、§5.3；brief §4.4、§4.6；ADR 0004；spec〈共同規則〉；`.scratch/m0/issues/09-m0-close.md` 的 Comments（「M1 票 7」那條）

## What to build

網頁有觸控模式：行動裝置 viewport 下顯示一份由資料算出的 Gboard 速成直向鍵盤，點按鍵打出「門口」。從佈局資料、`qim-layout` 的點擊判定、綁定到網頁一路打通；格式只做到這條路需要的欄位，其餘在票 19 定稿。

- 建 `crates/qim-layout`；plan §1.3 表與 `xtask` 的 `RULES` 同步，並補〈M0 留下的〉的同步測試。
- 格式最小子集（照 plan §2.6 的欄位名，不另取名）：`format`、一個 `variant`（手機直向）、`canvas`、`pages` 的一頁、按鍵的 `id`、`frame`、`z`、`tap`、`hint`；label 省略時用方案的字根。
- 點擊判定：依 `z` 由上往下，落在矩形內的第一個；都沒有時取最近的按鍵（plan §2.6）。網頁只畫，不自己判定。
- 預設 Gboard 速成佈局由 `qim-databuild` 依排列算出（plan §2.6），輸出到建置產物，網頁載入。
- `qim-wasm` 加佈局的綁定；網頁加觸控模式切換，點擊位置交給 `qim-layout` 判定後送 `Key`。

## M0 留下的

票 09 Comments「M1 票 7」：`qim-layout` 進來時 plan §1.3 表要改，順便補「不可以」欄與 `FORBIDDEN_PATHS` 的同步測試。

## 驗收

- [x] Playwright（行動裝置 viewport，`tap`）：依序點 `a`、`n`、`r` 的位置，讀回首選「門口」，點空白讀回輸入框「門口」；截圖。
- [x] `qim-layout` 單元測試：上層優先、點在空白處取最近的鍵。
- [x] `deps` 對 `qim-layout` 綠燈；plan §1.3 表與 `RULES` 的「可以」「不可以」兩欄都有同步測試。
- [x] 〈M0 留下的〉已處理。

## Comments

- 2026-11-04 plan §1.3 表的改法：`qim-databuild` 要用 `qim-layout` 的型別寫出預設佈局，「可以依賴」加 `qim-layout`，和 `qim-bench` 拆成兩列；`qim-core` 的「不可以」由「任何綁定或平台 crate」改成可檢查的「任何 workspace crate（含綁定與平台 crate）」。
- 2026-11-04 「不可以」欄的同步測試（`xtask/src/deps.rs` 的 `plan_forbidden_column_matches_rules`）：欄以「；」分子句，每句要是反引號包住的禁用路徑或 crate、`被 … 依賴`、`任何 workspace crate`、`同上`、`同上的 I/O` 之一，看不懂的子句也紅；禁用路徑與 `FORBIDDEN_PATHS`、`no_io` 雙向相等，crate 的禁止只單向比（表是白名單）。變異：少一個路徑、拿掉 `同上的 I/O`、禁止它其實依賴的 crate、`被 qim-wasm 依賴`、`任何 workspace crate`、看不懂的子句都紅；改括號說明、空白、表頭不紅。
- 2026-11-04 格式子集：`format`、`variants[].when`、`canvas`、`pages[].id`、`keys[]` 的 `id`、`frame`、`z`（預設 0）、`label`、`hint`、`tap`；動作 `{ "letter": "a" }` 與 `"space"`、`"backspace"`、`"enter"`（serde 的外部標記）。`deny_unknown_fields`，票 19 加欄位時一起放寬或補。識別符照 plan §0 用 `LayoutVariant`。
- 2026-11-04 點擊判定的細節（plan §2.6 沒寫的）：矩形左上邊算在內、右下邊不算；同一個 `z` 時陣列裡後面的在上（後畫的蓋住先畫的）；最近的鍵一樣近時取上層的。
- 2026-11-04 Gboard 直向：畫布 390 × 216，一排 10 鍵寬、高 54，每鍵四周留 3 / 5 的縫（縫上由點擊判定取最近的鍵）；第三排左邊 1.5 鍵寬留給之後的分隔鍵，第四排空白 8.5 加 Enter 1.5。`z` 鍵照 Gboard 放著，方案沒有它的字根所以標籤是字母。寫到 `build/layouts/gboard-quick.json`，`cargo xtask web` 複製到 `web/public/layouts/`。
- 2026-11-04 綁定：`Layout.hit(x, y)` 回按鍵位置（`qim_layout::Page::hit` 本身就回 `keys` 的位置），`Session.tap(layout, index)` 在 `qim-wasm` 把佈局動作轉成 `qim_core::Key`，所以沒加逐鍵的 `enter()`（票 10 也在改 Enter）。核心沒吃下的鍵網頁照輸入框的預設動作處理（插字母、空格、換行，刪游標前一字）。佈局版本的選法留給票 19，綁定先取第一個版本的第一頁。
- 2026-11-04 網頁：`(pointer: coarse)` 時預設開觸控鍵盤，「觸控鍵盤」按鈕切換；開著時輸入框 `inputmode="none"`。亂按（Pixel 7 viewport，兩個種子各 400 步：隨機點鍵與縫、夾實體鍵、切換模式，加 20 次並發連點）找到兩個問題，各先寫成會紅的 Playwright 測試再修：候選出現時鍵盤往下跳 62 px，改成固定在畫面底部；手機寬度下候選被擠成直排，改單列可左右滑（brief §4.4）。修好後亂按沒有 console 錯誤。
- 2026-11-04 bench 前後對照：沒改核心與資料包，`cargo xtask data` 前後兩份資料包的 sha256 相同（quick `b515cedf128b`、quick-lexicon `4403437674d5`），各集合答錯與修正次數與 baseline 相同、沒有翻盤，固定題 11/11。
- 2026-11-04 驗收輸出：`cargo xtask check` 印 `==> fmt` … `==> e2e`、`16 passed (3.1s)`、`check 通過`；`cargo xtask bench --check` exit 0。截圖：`.scratch/m1/evidence/18-touch-keyboard.png`、`18-touch-anr.png`（首選「門口」）、`18-touch-commit.png`（輸入框「門口」）、`18-touch-monkey.png`（亂按後）。
- 2026-11-04 code-review（`873097f` 起）已處理：「不可以」欄的子句改成整句比對（`同上的 I/O` 要完全相同、`被 … 依賴` 要有 `qim-*`、`任何 workspace crate` 後只能接括號說明），補三個舊寫法會漏的變異；`Page::hit` 直接回位置，綁定不再反查；`Action::name()` 給 Gboard 的 id 與綁定共用；網頁 fetch 去重、佈局網址搬到 `main.ts`；`xtask web` 改名 `serve_data_outputs`；綁定的鍵型別改名 `WebLayoutKey`；`touch.spec.ts` 的載入收進 `beforeEach`。
- 2026-11-04 code-review 沒處理：
  - `style.css` 的 `padding-bottom` 照畫布比例 390 × 216 寫死，改畫布要跟著改；票 20 做橫向時一起改成由佈局算。
  - 畫布高 216 不同於 plan §2.6 例子的 260：例子只是示意，票 19 定稿時決定是否改例子。
  - `qim-layout` 三個測試檔各自組 JSON 外殼、綁定用 `session.engine.scheme().radicals` 取字根：小，等票 19 加欄位時再看要不要抽。
  - 錯誤訊息與 log 用中文：跟著 repo 既有慣例。
  - 超出票寫的部分保留：宿主預設動作（沒有它觸控鍵盤的刪除、空白在沒組字時沒反應）、候選單列與鍵盤固定底部（亂按找到的問題）、`from_json` 擋 `format` 錯與沒有版本或頁面（綁定靠它保證有第一頁）。
- 2026-11-04 合併 main（票 10，`09f6116`）：沒有衝突，只把 progress 裡 union 合併排在前面的本票那列與偏差移到票 10 之後。合併後 `cargo xtask data` 的兩份 sha256 不變，`cargo xtask check` 通過（含 `unit`，e2e `23 passed`），`cargo xtask bench --check` exit 0、沒有翻盤、固定題 11/11。觸控路徑再亂按三組各 400 步（觸控點擊夾實體 Enter、`, . ? ! ;`、方向鍵、Tab、Delete、Home、End、大寫）：沒有 console 錯誤、鍵盤不移動，組字中點觸控 Enter 共 25 次都清空組字；`.scratch/m1/evidence/18-touch-monkey.png` 換成合併後的截圖。
