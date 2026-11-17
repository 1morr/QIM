# 進度

工作單位是票（`.scratch/<里程碑>/issues/NN-slug.md`），狀態在票的 `Status:` 行。這裡只記進行中里程碑的 session 紀錄；里程碑的順序、狀態與閱讀指引在 `docs/roadmap.md`，決定在 `docs/decisions.md`。

## 寫法與封存

- 做票的分支不改本檔：票的 Comments 留 `Session 紀錄：` 行，合併的人在主分支抄成這裡的一列（`AGENTS.md`〈每個 session 的流程〉）。
- 一列不超過 200 字元；細節在票的 `## Comments`，這裡只放指向它的一句話。
- 「票」欄標里程碑（`M0 01`），封存時靠它分。
- 里程碑驗收時（步驟在 `docs/agents/milestone.md`〈收尾票〉），把它的 session 紀錄移到 `docs/archive/<里程碑>.md` 的 `## Session 紀錄`、`docs/decisions.md` 裡它的決定移到同一個檔的 `## 決定`，逐列逐條比對沒有遺失，`docs/roadmap.md` 里程碑表的狀態換成那個檔的連結。同一個 commit 把下一個里程碑標成進行中，「下一步」寫 `/1morr:checkpoint <封存的里程碑>`。本檔只留進行中的里程碑。
- `python3 scripts/check_docs.py`（`cargo xtask check` 的 `docs` 步驟會跑它）強制的規則列在腳本開頭；本檔的是：兩個 `##` 段的順序、session 紀錄是一整張表、每列的長度、最後一列的下一步。

## Session 紀錄

| 日期 | 票 | 結果 | 下一步 |
| --- | --- | --- | --- |
| 2026-11-17 | M5 封存 M4 | 票 31：里程碑 code-review、乾淨 checkout 重驗 spec〈驗收〉、v3 定稿、使用者試用第二期通過（31b 修區網 http）；M4 封存到 `docs/archive/M4.md`；見票 31 | 新 session 下 `/1morr:checkpoint M4` |
