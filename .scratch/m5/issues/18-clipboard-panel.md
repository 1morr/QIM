# 18: 剪貼簿面板

Status: ready-for-agent
Blocked by: 15
讀: brief §4.4、§8；`docs/design/15-clipboard-panel.html`；票 04 的剪貼簿限制

## What to build

照票 15 的定稿做剪貼簿面板：瀏覽器做得到的範圍（例如 QIM 自己複製過的文字、讀得到系統剪貼簿時的最新一筆）。剪貼簿內容只存本機，無痕時不記（brief §8）：無痕開關在核心，網頁自己存的剪貼簿歷史不會自動遵守，由殼讀無痕狀態自己判斷。

## 驗收

- [ ] Playwright 讀回面板列出剛複製的文字、點了上屏；無痕時不記。
- [ ] 動到網頁的部分照 spec〈共同規則〉亂按一輪，找到的問題先寫成會紅的 Playwright 測試再修。
- [ ] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments
