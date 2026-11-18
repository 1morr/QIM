# 04: 盤點 Gboard 與 iOS 鍵盤的觸控功能

Status: ready-for-agent
Blocked by: None
讀: brief §4.4、§7；`docs/research/gboard-english-2026-11-15.md`（上次 Gboard 實測的做法）；`docs/research/facts-products.md`；spec〈範圍〉

## What to build

M5 要把網頁觸控鍵盤做到平台樣板，先知道成熟產品有什麼：

- 用 `orca emulator-android` 在 Gboard 實測觸控鍵盤的全部功能：工具列的入口、各面板（emoji、剪貼簿、文字編輯、常用語、其他）、長按、手勢、設定頁的分類與每個項目、鍵盤高度分幾段。
- 用 `orca emulator` 在 iOS 模擬器對照內建鍵盤，只記 Gboard 沒有的（使用者 2026-11-18 同意）。
- Gboard 速成的 `z` 鍵：鍵上標什麼、點了在速成做什麼（M4 票 11），給出要不要把 QIM 的「重」改標「＊」的建議。
- 瀏覽器讀寫剪貼簿的限制（Clipboard API 的權限、iOS Safari 與 Android Chrome 的差異），查證寫成事實。
- 寫到 `docs/research/touch-inventory-<日期>.md`，截圖存 `.scratch/m5/evidence/`；外部事實補進對應的 `facts-*.md`。

最後列成一張清單（每項：Gboard 怎麼做、iOS 有沒有、QIM 現在有沒有、建議第一期／第二期／不做），請使用者逐項勾。勾完寫回 spec：第一期工具列的入口、設定頁的分類、鍵盤高度的段數、期界；第二期要加或刪的票交給期中票（票 12）。

## 驗收

- [ ] 研究檔寫完，Gboard 每個觀察有截圖或讀回的畫面文字。
- [ ] 清單每一項都有使用者的勾或叉，記成 `決定：…（使用者定）`。
- [ ] spec〈範圍〉照勾的結果更新（工具列入口、設定分類、高度段數、期界）。
- [ ] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments
