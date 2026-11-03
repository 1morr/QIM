# 24: M1 收尾

Status: ready-for-agent
Blocked by: 01–23
讀: spec〈驗收〉；`docs/progress.md`〈寫法與封存〉；plan §5.2、§7；`docs/development.md`〈全部檢查〉；`docs/research/blackbox-baseline.md`；票 01–23 的 `## Comments`

## What to build

M1 的驗收（使用者任務書的階段 5）全部重驗並留下證據；黑箱測試表能在網頁做的項目在 QIM 上重跑、和競品並列；M1 封存，M2 標進行中。

- 里程碑 code-review：`/code-review <M1 開工時的 commit>`，Spec 軸讀 spec 與票 01–23，Standards 軸讀 `CLAUDE.md`。小問題修掉；沒修的與各票 Comments 裡沒處理的發現，整理成清單寫在本票 `## Comments`，每條註明留到哪個里程碑或不修的理由。
- 在乾淨 checkout（`git worktree add --detach` 到暫存目錄，照 `docs/development.md` 從零安裝）跑 spec〈驗收〉每一項：Playwright 實打（截圖存 `.scratch/m1/evidence/`）、`bench --check`、延遲、`data --verify`、`check` 全套。
- 黑箱：`docs/research/blackbox-baseline.md` 的 T1–T18 中能在網頁做的項目在 QIM 上重跑，照 `closed-source-imes.md` §4 的三類（缺功能、行為不同但合理、行為更差）記錄、和競品並列。
- 封存（〈寫法與封存〉，同一個 commit）：M1 的 session 紀錄與偏差移到 `docs/archive/M1.md`；里程碑表 M1 連到封存檔、M2 進行中；plan §7 表 M1 完成、M2 進行中，§7.2 指向 spec 與封存檔；本 session 的紀錄標 `M2`，下一步寫「新 session 下 `/to-tickets docs/plan.md 拆 M2 的票`」。

## 驗收

- [ ] 里程碑 code-review 跑完；沒處理的發現清單寫在 `## Comments`，每條有去處或理由。
- [ ] spec〈驗收〉每一項在乾淨 checkout 上成立，輸出與截圖路徑貼在 `## Comments`。
- [ ] 黑箱表的 QIM 欄已補上並分類。
- [ ] `docs/archive/M1.md` 有 M1 全部 session 紀錄與偏差；progress 只留 M2；里程碑表與 plan §7 已更新；同一個 commit。
- [ ] 封存後 `cargo xtask check` 綠燈。

## Comments
