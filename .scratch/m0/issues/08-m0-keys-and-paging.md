# 08: 補齊 M0 按鍵與翻頁

Status: ready-for-agent
Blocked by: 07
讀: plan §2.1、§5.1；brief §4.3；spec〈範圍〉〈共同規則〉

## What to build

使用者在網頁打 `ar` 能翻頁找到「問」並用數字鍵上屏；打錯時用 Backspace 刪一碼、Esc 清掉整串。M0 的單字查詢在網頁上可以實際使用。

- `Session` 加 `Backspace`、`Escape`、`Digit`、`PageUp`、`PageDown`：
  - Backspace 刪最後一碼；Escape 清除輸入碼。
  - 數字 1–9 上屏目前頁對應的候選；沒有該位置時不動作。
  - 翻頁依 `has_more` 與 `page`，到頭或到尾時不動作。
  - 沒有組字時，這些鍵都不攔截，交給宿主（Backspace 刪輸入框的字、數字照打）。
- 輸入碼超過方案的 `max_code_length`（速成是 2）時候選為空、空白不上屏；連打是 M1。
- 網頁：`-` / `=` 與 PageUp / PageDown 轉成翻頁（spec〈共同規則〉）。
- 性質測試（`proptest`）：任何 `Snapshot` 的每個候選，碼表裡都有一個碼等於目前輸入碼。

## 驗收

- [ ] Playwright：打 `ar`，讀回候選含「問」（必要時翻頁），用數字鍵選它，讀回輸入框為「問」。
- [ ] Playwright：Backspace 刪一碼後候選更新；Esc 清空組字；沒有組字時 Backspace 刪輸入框的字。
- [ ] 截圖存證（貼路徑）。
- [ ] `qim-core` 單元測試涵蓋上述每個鍵，含沒有組字時不攔截、翻頁到頭與到尾、超過 2 碼。
- [ ] 性質測試綠燈。
