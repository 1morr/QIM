# 28: 手動分隔鍵

Status: ready-for-agent
Blocked by: 27
讀: brief §4.3、§4.4；plan §2.1（`Key`、`Composition`）、§2.3（lattice）；票 21、27 的 Comments；`docs/research/blackbox-baseline.md`（Gboard）；spec〈共同規則〉

## What to build

使用者 2026-11-05 決定：照 Gboard 速成，連打時可以手動指定「這裡斷開」。Gboard 在組字中把 ⇧ 換成分隔鍵「'詞」；QIM 核心目前沒有分隔功能。

- 核心：組字中按分隔鍵，在目前位置插入一個分隔記號；lattice 的段不能跨過分隔記號，其餘照 §2.3 打分。分隔記號顯示在組字裡（樣式照 Gboard 或 Rime 的 `'`，本票定），退格先刪分隔記號。沒有組字時按分隔鍵照原本的字元輸出。
- 實體鍵盤：分隔鍵用 `'`（Rime 拼音方案的慣例；速成不用這個字元），brief §4.3 的鍵位表補一列。
- 觸控：組字中 ⇧ 的主標籤換成「'詞」並送分隔鍵，沒有組字時照票 27 的 Shift 行為（照 Gboard 實測）。
- 學習與自動造詞照分隔後的分段記（plan §2.4）。
- 同輪改 brief §4.3、§4.4、plan §2.1、§2.3；progress〈偏差與決定〉記一行（使用者決定），session 紀錄票欄寫「M2（補 M1 28）」。

## 驗收

- [ ] 核心測試：打 `mir` 首選「一台」；打 `mi'r` 時分隔後第一段只能是 `mi` 的字詞，首選照分段排；退格先刪分隔記號。
- [ ] Playwright：實體鍵盤打 `mi'r` 讀回組字含分隔記號、上屏結果符合分段；觸控組字中 ⇧ 顯示「'詞」，點了插入分隔。
- [ ] brief、plan 已更新；`cargo xtask check`、`bench --check` 綠燈；/code-review 跑完。

## Comments
