# 18: 佈局 tracer：觸控點出「門口」

Status: ready-for-agent
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

- [ ] Playwright（行動裝置 viewport，`tap`）：依序點 `a`、`n`、`r` 的位置，讀回首選「門口」，點空白讀回輸入框「門口」；截圖。
- [ ] `qim-layout` 單元測試：上層優先、點在空白處取最近的鍵。
- [ ] `deps` 對 `qim-layout` 綠燈；plan §1.3 表與 `RULES` 的「可以」「不可以」兩欄都有同步測試。
- [ ] 〈M0 留下的〉已處理。

## Comments
