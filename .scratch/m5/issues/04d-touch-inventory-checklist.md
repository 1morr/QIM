# 04d: 盤點四：iOS 對照與彙整清單，請使用者選

Status: ready-for-agent
Blocked by: 04, 04b, 04c
讀: 票 04、04b、04c 的研究檔；brief §4.4、§7；spec〈範圍〉；`docs/agents/visual.md`

## What to build

- 用 `orca emulator` 在 iOS 模擬器對照內建鍵盤，只記 Gboard 沒有的（使用者 2026-11-18 同意），寫到 `docs/research/ios-keyboard-<日期>.md`。
- 把 04、04b、04c 與 iOS 的項目彙整成一份清單 `docs/research/touch-inventory.md`（會更新的主題整理），並做成一頁讓使用者逐項作答（使用者 2026-11-18 同意）。每一項的選項：照 Gboard、照 Gboard 再加（寫明加什麼）、不做、平台才做；每項附 Gboard 的截圖或畫面文字、QIM 現況、網頁做不做得到、建議的期別。
- 使用者已定的先填進清單：剪貼簿建議之上要加「只貼連著的數字」「貼全部數字（不連著也收）」「編輯已複製的文字」（使用者 2026-11-18 定）。
- 使用者選完：
  - 寫回 spec〈範圍〉：第一期工具列的入口、設定頁的分類、鍵盤高度的段數、`z` 鍵標示、期界。
  - brief §4.4、§7 的改動先給使用者看草稿，確認後才寫（`AGENTS.md`〈使用者確認〉）。
  - 第二期要加或刪的票寫在 Comments，交給期中票（票 12）開或改。

## 驗收

- [ ] iOS 對照寫完，只含 Gboard 沒有的項目。
- [ ] `docs/research/touch-inventory.md` 每一項都有使用者的選擇，記成 `決定：…（使用者定）`（項目多時一類一行）。
- [ ] spec〈範圍〉照選擇更新；brief 的改動使用者確認過才寫。
- [ ] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments
