# 23: M3 收尾

Status: ready-for-agent
Blocked by: 01, 02, 03, 04, 05, 06, 07, 08, 09, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22（及票 11 補的實作票）
讀: spec〈驗收〉；`docs/progress.md`〈寫法與封存〉；plan §2.4、§7、§8；`docs/development.md`〈全部檢查〉；票 01–22 的 `## Comments`

## What to build

M3 的驗收全部重驗並留下證據，然後封存 M3、M4 標進行中。

- 里程碑 code-review：`/code-review <M3 開工時的 commit>`，Spec 軸讀 spec 與各票，Standards 軸讀 `CLAUDE.md`。小問題修掉；沒修的與各票 Comments 沒處理的發現，整理成清單寫在本票 `## Comments`，每條註明留到哪個里程碑或不修的理由。
- 學習資料匯出 v2 定稿，寫進 plan §2.4（spec〈共同規則〉）。
- 問使用者有沒有在網頁按「回報」留下的案例；有的話照 plan §5.2 加進 `fixed.tsv`（M1 留下）。
- 在乾淨 checkout（`git worktree add --detach` 到暫存目錄，照 `docs/development.md` 從零安裝）跑 spec〈驗收〉的每一項，輸出與截圖路徑貼在 Comments。
- 封存（〈寫法與封存〉，同一個 commit）：M3 的 session 紀錄與偏差移到 `docs/archive/M3.md`；里程碑表 M3 連到封存檔、M4 進行中；plan 開頭狀態行與 §7 表更新，§7.4 指向 spec、封存檔與本票；本 session 的紀錄標 `M4`，下一步寫「新 session 下 `/to-tickets docs/plan.md 拆 M4 的票`」。

## 驗收

- [ ] 里程碑 code-review 跑完；沒處理的發現清單寫在 `## Comments`，每條有去處或理由。
- [ ] spec〈驗收〉每一項在乾淨 checkout 成立，證據貼在 `## Comments`。
- [ ] plan §2.4 寫明 v2 定稿；回報案例已處理或註明沒有。
- [ ] `docs/archive/M3.md` 有 M3 全部 session 紀錄與偏差；progress 只留 M4；里程碑表與 plan §7 已更新；同一個 commit。
- [ ] 封存後 `cargo xtask check` 綠燈。

## Comments
