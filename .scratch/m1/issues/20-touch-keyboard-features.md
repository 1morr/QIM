# 20: 觸控鍵盤功能

Status: ready-for-agent
Blocked by: 19
讀: plan §2.6、§3.1、§5.3；brief §4.4、§4.6；spec〈共同規則〉

## What to build

觸控模式像一個能用的手機鍵盤：長按字母彈出英文字母、第一排長按另含數字；有符號頁（打完不跳回，按「ABC」回字母頁）；有一個自訂頁面示範多頁；按鍵放大預覽；深色模式；轉橫向用橫向的佈局版本。

- 預設 Gboard 佈局由 `qim-databuild` 算出直向與橫向兩個版本，淺色與深色都有顏色；通過票 19 的嚴格檢查（含「有直向與橫向」）。
- 字母鍵主標籤是字根、角標是字母（brief §4.4）；長按清單照佈局的 `longPress`。
- 符號頁：打完留在符號頁（brief §4.4）；自訂頁面示範 `switchPage` 與 `text` 動作。
- 按鍵放大預覽；深色跟隨 `prefers-color-scheme`，網頁另有手動切換。
- 長按、預覽、切頁的狀態在網頁，判定仍經 `qim-layout`。

## 票 19 留下的

- `crates/qim-databuild/tests/gboard.rs` 的 `has_portrait_and_landscape` 標了 `#[ignore]`：加橫向版本時拿掉 ignore。
- 綁定（`crates/qim-wasm/src/lib.rs` 的 `Session.tap`）把 `text`、`switchPage`、`nextKeyboard`、`toggleAscii` 都當 `Key::Other`：組字中被吃掉、沒組字時網頁照 `tap` 處理（`text` 插入文字，其餘不動作）。接上符號頁時要決定組字中按文字鍵的行為（例如先上屏組字再輸入，照票 10 的標點）並寫 Playwright。
- 綁定只送淺色的 `fill`、`labelColor`，`WebLayout` 只在載入時選一次佈局版本；深色與轉向時重選版本在本票。鍵盤底色不在佈局格式裡（plan §2.6），深色時由網頁的 CSS 換。
- `web/src/style.css` 的 `padding-bottom` 照畫布比例 390 × 216 寫死（票 18 Comments），做橫向時改成由佈局算。

## 驗收

- [ ] Playwright（行動裝置 viewport）：點按鍵打出「門口」；長按字母鍵讀回彈出的英文字母，選了上屏該字母；長按第一排讀回含數字。
- [ ] Playwright：切到符號頁打兩個符號，讀回仍在符號頁；按「ABC」回字母頁；切到自訂頁面並輸入。
- [ ] Playwright：轉成橫向 viewport 後仍打得出「門口」；切深色讀回按鍵顏色是深色版本。
- [ ] 內建佈局通過嚴格檢查（含直向與橫向）。

## Comments
