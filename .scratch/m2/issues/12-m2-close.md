# 12: M2 收尾

Status: ready-for-agent
Blocked by: 01, 02, 03, 04, 05, 06, 07, 08, 09, 10, 11
讀: spec〈驗收〉；`docs/progress.md`〈寫法與封存〉；plan §7、§8；`docs/development.md`〈全部檢查〉；`docs/research/ios-feasibility.md`；票 01–11 的 `## Comments`

## What to build

M2 的驗收全部重驗並留下證據，四個問題的結論寫回 brief 與 plan，然後封存 M2、M3 標進行中。

- 里程碑 code-review：`/code-review <M2 開工時的 commit>`，Spec 軸讀 spec 與票 01–11，Standards 軸讀 `CLAUDE.md`。小問題修掉；沒修的與各票 Comments 沒處理的發現，整理成清單寫在本票 `## Comments`，每條註明留到哪個里程碑或不修的理由。
- 結論寫回：brief §9（記憶體目標從暫定改成實測值，註明只量了 iPhone 18 Pro Max）、§11（免費佈建那條結案）、§12.3；plan §7.8 的前置與模型上限、§8 各相關風險列的狀態。
- 在乾淨 checkout（`git worktree add --detach` 到暫存目錄，照 `docs/development.md` 從零安裝）跑 spec〈驗收〉能在 Mac 上重跑的每一項；iPhone 上的項目用乾淨 checkout 建出的擴充至少重打一次「門口」。
- 封存（〈寫法與封存〉，同一個 commit）：M2 的 session 紀錄與偏差移到 `docs/archive/M2.md`；里程碑表 M2 連到封存檔、M3 進行中；plan 開頭狀態行與 §7 表更新，§7.3 指向 spec、封存檔與本票；本 session 的紀錄標 `M3`，下一步寫「新 session 下 `/to-tickets docs/plan.md 拆 M3 的票`」。

## 驗收

- [ ] 里程碑 code-review 跑完；沒處理的發現清單寫在 `## Comments`，每條有去處或理由。
- [ ] spec〈驗收〉每一項成立，輸出與截圖路徑貼在 `## Comments`。
- [ ] brief §9、§11、§12.3 與 plan §7.8、§8 已寫回。
- [ ] `docs/archive/M2.md` 有 M2 全部 session 紀錄與偏差；progress 只留 M3；里程碑表與 plan §7 已更新；同一個 commit。
- [ ] 封存後 `cargo xtask check` 綠燈。
