# 17: 文字編輯面板

Status: ready-for-agent
Blocked by: 14
讀: brief §4.4；`ARCHITECTURE.md`〈平台殼〉；`docs/design/14-edit-panel.html`；spec〈共同規則〉

## What to build

照票 14 的定稿：從工具列開文字編輯面板，方向鍵移游標、選取、全選、複製、貼上。組字中按方向鍵的行為先定（照 Gboard 實測）；網頁的輸入框是唯讀的 `textarea`，面板用 `setSelectionRange` 移游標與選取（唯讀不擋它）；移游標後殼照〈平台殼〉重給前文。

## 驗收

- [ ] Playwright 用面板移游標、選取、全選、複製再貼上，讀回輸入框內容與游標位置。
- [ ] 動到網頁的部分照 spec〈共同規則〉亂按一輪，找到的問題先寫成會紅的 Playwright 測試再修。
- [ ] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments
