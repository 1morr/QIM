# 19: 空白鍵滑動移游標

Status: ready-for-agent
Blocked by: 12, 20
讀: 票 20 的 ADR；brief §4.4；`docs/research/spatial-correction-swipe.md`；`docs/research/facts-products.md`；盤點清單（票 04d 彙整的 `docs/research/touch-inventory.md`）

## What to build

按住空白鍵左右滑動移游標（Gboard 做法，靈敏度照盤點）；組字中滑動的行為照 Gboard 實測（例如移組字內的游標或不動）。滑動結束不觸發空白上屏。移游標後重給前文。空白鍵的滑動和長按、flick 共用 `touch.ts` 的同一段手勢判斷，照票 20 的 ADR 分。開關進設定頁（Gboard 有這個開關時）。

## 驗收

- [ ] Playwright 用觸控拖曳空白鍵，讀回游標移了幾格、沒有多打空白。
- [ ] 動到網頁的部分照 spec〈共同規則〉亂按一輪，找到的問題先寫成會紅的 Playwright 測試再修。
- [ ] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments
