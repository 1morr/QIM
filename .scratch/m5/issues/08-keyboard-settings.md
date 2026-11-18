# 08: 鍵盤內設定頁取代頁面下方的勾選框

Status: ready-for-agent
Blocked by: 06, 07
讀: brief §7；`ARCHITECTURE.md`〈平台殼〉；`docs/development.md`〈網頁〉；`docs/design/06-keyboard-settings.html`；spec〈共同規則〉

## What to build

照票 06 的定稿做鍵盤內設定頁（開工對程式碼補正，使用者 2026-11-18 同意）：

- 從工具列打開，分類照定稿。搬進去的是設定表的 17 列（`settings.ts` 的 `ROWS`，15 個勾選框、2 個下拉選單）加工具列的數字列、深色、字根提示；頁面上的這些控制項拿掉。觸控鍵盤、debug、欄位選單是測試頁的工具，留在頁面工具列；桌面模式的設定鈕也放在頁面工具列，打開同一個設定頁。資料管理（常用語、英文使用者詞、詞庫匯入、資料包、學習資料匯入匯出）留在頁面。
- **設定存檔（新功能）**：現在這些設定一重新整理就回預設，只有學習資料、資料包選擇、線上更新開關存在 `localStorage`。開關、數字列、深色、字根提示、鍵盤高度（票 09）都存進 `localStorage`，存不了時照樣能改、只在記憶體。鍵名與格式照現有的 `qim.*` 慣例，讀到不認得的值用預設。同步改 `docs/development.md`〈網頁〉的存放清單，`main.ts` 那句「學習資料與設定這次不會存」改成和實際一致。
- e2e：約 24 個 spec 直接點設定的勾選框或下拉選單（association、backspace-undo、context、debug-panel、digit-separators、edit-segments、emoji、english-user-words、english、field-types、forget、keys、key-table、learning、neighbor-readings、neural-rerank、rerank-cache、rerank、separator、settings、simplified、simplified-characters、sync、touch-features），設定頁平常收起後都會壞。在票 03 的共用模組加「開設定頁再改一項」的 helper，這些檔改用它；用到工具列數字列、深色、字根提示的（desktop-mode、radical-hint、physical-weighted、touch-features、touch）一起改。
- M4 留下的 bug（M4 票 16）：debug 面板開著、打過字後，第一次點「無痕」勾選框沒反應。原因還沒查到：先照這個步驟寫成會紅的 Playwright 測試，查清是重畫還是焦點問題再修，不假設搬位置就會好。

## 驗收

- [ ] Playwright 從工具列開設定頁，每個開關改了會生效（至少：關聯想後不出聯想、開數字列後多一排數字、深色切換）。
- [ ] 重新整理後設定還在；`localStorage` 打不開時照樣能改（e2e 讀回）；`docs/development.md` 的存放清單與 `main.ts` 的提示一致。
- [ ] 頁面上不再有搬走的控制項；觸控鍵盤、debug、欄位選單與資料管理各段照舊可用。
- [ ] 桌面模式的設定鈕打開同一個設定頁（e2e 讀回）。
- [ ] 24 個 spec 改用共用 helper，e2e 測試數不少於開工時。
- [ ] 「無痕」第一次點就生效的測試先紅後綠，原因寫在 Comments。
- [ ] 動到網頁的部分照 spec〈共同規則〉亂按一輪，找到的問題先寫成會紅的 Playwright 測試再修。
- [ ] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments
