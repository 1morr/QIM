# 05: 設計稿：候選列與工具列

Status: ready-for-agent
Blocked by: 04d
讀: brief §4.4；`docs/agents/ui.md`；盤點清單（票 04d 彙整的 `docs/research/touch-inventory.md`）；`docs/design/23-touch-english-keyboard.html`（現有鍵盤的設計稿）

## What to build

用 `impeccable:impeccable` 做候選列與工具列的設計稿，放 `docs/design/05-keyboard-top.html`：

- 候選列接在觸控鍵盤頂端：組字中的候選列（單列、可左右滑、右側展開鍵、展開面板）；沒有組字時同一格換成工具列，入口照盤點清單選的；第一期還沒做的面板入口怎麼處理（隱藏或停用）。
- 規格照 brief §4.4：直向 390 寬與橫向 844 寬、淺色與深色、照 Gboard 實測的現有鍵盤樣式。
- 有爭議的地方給 A／B 兩案。

## 驗收

- [ ] 設計稿在 `docs/design/`，涵蓋 390／844 寬、淺深色、組字中與沒組字兩種狀態。
- [ ] 使用者確認過，回覆記成 `決定：…（使用者定或同意）`。
- [ ] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments
