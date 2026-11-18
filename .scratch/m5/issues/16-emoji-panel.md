# 16: emoji 面板

Status: ready-for-agent
Blocked by: 13
讀: brief §4.4、§7；`docs/research/emoji-candidates.md`；`docs/design/13-emoji-panel.html`；spec〈共同規則〉

## What to build

照票 13 的定稿：從工具列開 emoji 面板，分類、最近用過、搜尋；點了上屏 emoji。現有的 `cldr-emoji-yue-hant` 資料包只有「字詞 → emoji」，沒有分類、膚色與全集，不夠（開工對程式碼補正，使用者 2026-11-18 同意）：要新的分類資料，候選是 Unicode `emoji-test.txt` 的 group 欄（同一個來源，已釘 18.0）。做成新的資料包種類（對外介面）或網頁自帶的檔，先看 Gboard、iOS 的做法，在票上先問使用者再做；新來源照 `data/sources.toml` 釘版本與授權。

## 驗收

- [ ] Playwright 開面板、換分類、點一個 emoji 上屏、最近用過出現它、搜尋找得到。
- [ ] 動到網頁的部分照 spec〈共同規則〉亂按一輪，找到的問題先寫成會紅的 Playwright 測試再修。
- [ ] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments
