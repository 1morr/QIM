# 06: 設計稿：鍵盤內設定頁與鍵盤高度

Status: ready-for-agent
Blocked by: 04d
讀: brief §4.4、§7；`docs/agents/ui.md`；盤點清單（票 04d 彙整的 `docs/research/touch-inventory.md`）；`web/src/settings.ts`（現有的開關）

## What to build

用 `impeccable:impeccable` 做鍵盤內設定頁的設計稿，放 `docs/design/06-keyboard-settings.html`：

- 從工具列打開、畫在鍵盤範圍內的設定頁；分類照盤點清單裡 Gboard 的設定分類（之後平台 App 照同一份），現有的每個開關（brief §7 列的、數字列、深色、字根提示）都有位置。
- 只放開關與外觀；常用語、英文使用者詞、詞庫匯入、資料包、學習資料匯入匯出留在頁面上當主 App（使用者 2026-11-18 同意），設定頁可以放一個連過去的入口。
- 鍵盤高度的調法與段數照盤點。
- 平台才有的（按鍵音、震動、語音、懸浮鍵盤）留位置與說明。
- 桌面模式頁面上的設定鈕打開同一個設定頁（使用者 2026-11-18 同意），畫出它在桌面的樣子。
- 規格照 brief §4.4。

## 驗收

- [ ] 設計稿在 `docs/design/`，涵蓋 390／844 寬、淺深色、每個分類的頁、桌面的設定鈕。
- [ ] 使用者確認過，回覆記成 `決定：…`。
- [ ] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments
