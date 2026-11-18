# 07: 候選列接在觸控鍵盤頂端、沒組字時換成工具列

Status: ready-for-agent
Blocked by: 03, 05
讀: brief §4.4；`ARCHITECTURE.md`〈平台殼〉〈核心對殼〉；`docs/design/05-keyboard-top.html`；spec〈共同規則〉

## What to build

照票 05 的定稿：觸控模式下候選列（含中英指示 `#mode`、展開鍵、展開面板、常用語入口）接在觸控鍵盤頂端；沒有組字時同一格換成工具列。工具列入口照定稿，已經有功能的（常用語、設定）接上，第一期還沒做的面板照定稿處理。

現有程式要一起處理的（開工對程式碼補正，使用者 2026-11-18 同意）：

- 觸控鍵盤是 `position: fixed` 的 `#keyboard`；候選列整塊搬進鍵盤頂端的容器。桌面與觸控是同一份 DOM（`display.ts` 的 `setTouch` 只切 `hidden`），桌面模式用 CSS 讓候選列留在原位，不做兩份。
- `display.ts` 的 ResizeObserver 只量 `#keyboard` 來寫頁底留白（`--keyboard-height`）；改成含候選列的高度，否則蓋住輸入框。
- 地球鍵選單（`keyboard-menu.ts`）蓋在候選列之上。
- 組字顯示 `#composition` 留在輸入框旁（模擬宿主的行內組字），不跟著搬。
- e2e 幾乎都用無障礙名稱抓候選列，不會因搬位置壞；直接用 CSS 抓 `#candidates` 的 `rerank-cache.spec.ts` 跟著改。「候選列貼著鍵盤」的斷言要新寫。

## 驗收

- [ ] Playwright 390 寬與 844 寬讀回：候選列下緣貼著鍵盤上緣、地球鍵選單蓋在候選列之上；清空組字後同一格是工具列；截圖淺色、深色。
- [ ] 輸入框最後一行不被候選列或鍵盤蓋住（e2e 讀回）。
- [ ] 原有的候選列 e2e（選字、展開、翻頁、長按字根提示、常用語）在新位置照過。
- [ ] 桌面模式的候選列位置不變（e2e 讀回）。
- [ ] 動到網頁的部分照 spec〈共同規則〉亂按一輪，找到的問題先寫成會紅的 Playwright 測試再修。
- [ ] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments
