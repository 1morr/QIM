# 01: 做票的紀錄只留在票上

Status: ready-for-agent
Blocked by: `.scratch/m4/issues/31-m4-close.md`
讀: `.scratch/flow-sync/spec.md`；範本的 `templates/instructions.md`〈文件〉〈使用者確認〉〈每個 session 的流程〉、`templates/progress.md`、`templates/decisions.md`、`templates/milestone.md`、`templates/issue-tracker.md`

## What to build

- `AGENTS.md`〈文件〉〈使用者確認〉〈每個 session 的流程〉換成範本的寫法：`Session 紀錄：<日期> …` 與 `決定：` 留在票上、沒人抄；第 1 步加跑 `python3 scripts/ticket_notes.py <里程碑的目錄名>`；驗收票自己改 progress.md 與 decisions.md。〈Skills〉的「收尾票 `NN-m<N>-close`」改成「驗收票 `NN-<里程碑>-acceptance`」。
- progress.md〈寫法與封存〉、decisions.md〈寫法〉、`docs/development.md`〈全部檢查〉那句、`docs/agents/issue-tracker.md`〈Conventions〉、`docs/agents/milestone.md`〈驗收票〉照範本改；封存步驟用 `ticket_notes.py --archive` 與 `docs_moved_lines.py`。
- 加 `scripts/ticket_notes.py`、`scripts/docs_moved_lines.py`、`scripts/tests/test_ticket_notes.py`、`scripts/tests/test_docs_moved_lines.py`；`scripts/check_docs.py` 與 `scripts/tests/test_check_docs.py` 換成範本的版本，QIM 自己的 `HookEnvironmentTest` 加回去。這張票先不建 `CHANGELOG.md`（票 02），所以 `變更：` 的檢查還不生效。
- 進行中的工作（例如 `.scratch/arch-diet/`）已經 done 的票，`Session 紀錄：` 沒有日期的補上：取 progress.md 那一列的日期，沒有就取票檔最後一次 commit 的。

## 驗收

- [ ] `python3 scripts/check_docs.py` 與 `python3 -m unittest discover -s scripts/tests` 綠燈；`diff` 範本的 `check_docs.py` 沒有差異。
- [ ] `grep -rn "抄進\|抄成" AGENTS.md docs/progress.md docs/decisions.md docs/agents docs/development.md` 沒有結果。
- [ ] `python3 scripts/ticket_notes.py m4` 印得出 M4 的票（舊票沒日期時取 commit 日期）。
- [ ] `cargo xtask check --fast` 綠燈。

## Comments
