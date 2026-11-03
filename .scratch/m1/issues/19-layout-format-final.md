# 19: 佈局格式定稿

Status: ready-for-agent
Blocked by: 18
讀: plan §1.4（`serde_json`、`jsonschema`）、§2.6、§5.3；brief §4.6；ADR 0004；`docs/research/open-source-imes.md` §4；brief §12.4（Delta skin）；spec〈共同規則〉

## What to build

佈局格式 `qim-layout/1` 定稿：plan §2.6 的全部欄位都有、有 JSON Schema、驗證分錯誤與警告兩級；矩形、圓角、旋轉與上下層順序的點擊判定正確。做一份兩鍵重疊的測試佈局，在網頁觸控模式點重疊處輸出上層按鍵。這是對外格式（之後改要加版本並提供轉換，ADR 0004），定稿前對照 Delta skin、Trime、FlorisBoard，採用了誰的慣例寫回 plan §2.6。

- 欄位：佈局版本與選法（裝置與方向都符合 → 方向符合 → 第一個版本縮放並警告）、`grid`、`initialPage`、多頁、`rotate`、`shape`（rect 含 `radius`；ellipse、polygon 的欄位留好，實作與點擊判定在 M5，遇到時給警告）、`touchInset`、`label`、`a11yLabel`、`style`（淺色與深色）、`longPress`、`composing`、動作清單（plan §2.6）。
- JSON Schema 放在 repo，`qim-layout` 用 `jsonschema` 驗證。
- 兩級驗證（plan §2.6）：錯誤（檔案壞掉；頁面切不回有字母的頁；iOS 需要切換鍵盤鍵時卻沒有）、警告（重疊、完全被蓋住、超出畫布、缺字母、缺直向或橫向版本）。
- 內建佈局的嚴格檢查（plan §5.3）：票 18 的 Gboard 佈局通過；橫向版本在票 20 才有，嚴格檢查的「有直向與橫向」在本票先寫好測試、標明票 20 補齊。
- 點擊判定的性質測試：上層優先、旋轉後內外判定正確、縮放後結果不變。

## 驗收

- [ ] plan §2.6 寫明定稿的欄位與參考了誰；Schema 進 repo。
- [ ] 驗證的雙向測試：造出每種錯誤會紅（不能存檔），只有警告不紅。
- [ ] 點擊判定的性質測試綠燈（上層、旋轉、縮放）。
- [ ] Rust 測試：兩鍵重疊的測試佈局，點重疊處判定為上層按鍵。
- [ ] Playwright：載入重疊測試佈局，點重疊處讀回輸出的是上層按鍵。

## Comments
