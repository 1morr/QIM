# 進度

工作單位是票（`.scratch/<里程碑>/issues/NN-slug.md`），狀態在票的 `Status:` 行。這裡只記各里程碑的閱讀指引、session 紀錄、偏差與決定。

本檔只留目前與上一個里程碑的 session 紀錄和偏差；里程碑驗收後，更早的段落整段移到 `docs/progress-archive/<里程碑>.md`，這裡留一行連結。

## 里程碑與閱讀指引

| 里程碑 | 票目錄 | 拆票前讀 | 備註 |
| --- | --- | --- | --- |
| M0 | `.scratch/m0/issues/` | brief 全文；plan §1、§6、§7.1 | 未拆票。brief 與 plan 定稿後跑 `/to-tickets docs/plan.md` |

## Session 紀錄

| 日期 | 票 | 結果 | 下一步 |
| --- | --- | --- | --- |
| 2026-10-30 | 框架 | 參照 Berth 的流程建立文件骨架：`CLAUDE.md`、brief 與 plan 骨架、本檔、`docs/agents/` | 新對話跑 `/grill-with-docs` 填 brief，再寫 plan |

## 偏差與決定

- 2026-10-30 框架：名詞表用 `GLOSSARY.md`（mattpocock skills 現行慣例），不沿用 Berth 的 `CONTEXT.md`。
- 2026-10-30 框架：技術棧未定，lint、CI、commit-msg hook 等閘門延到 M0 票 01（plan §6）。
