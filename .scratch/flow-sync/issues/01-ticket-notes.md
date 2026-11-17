# 01: 做票的紀錄只留在票上

Status: ready-for-agent
Blocked by: `.scratch/gates/issues/01-missing-gates.md`（它把 `docs/development.md`〈閘門〉的「待定」換掉，範本的檢查才過）
讀: `.scratch/flow-sync/spec.md`；範本的 `templates/instructions.md`〈文件〉〈使用者確認〉〈每個 session 的流程〉、`templates/progress.md`、`templates/decisions.md`、`templates/milestone.md`、`templates/issue-tracker.md`

## What to build

- `AGENTS.md`〈文件〉〈使用者確認〉〈每個 session 的流程〉換成範本的寫法：`Session 紀錄：<日期> …` 與 `決定：` 留在票上、沒人抄；第 1 步加跑 `python3 scripts/ticket_notes.py <里程碑的目錄名>`；驗收票自己改 progress.md 與 decisions.md。〈Skills〉的「收尾票 `NN-m<N>-close`」改成「驗收票 `NN-<里程碑>-acceptance`」。
- progress.md〈寫法與封存〉、decisions.md〈寫法〉、`docs/development.md`〈全部檢查〉那句、`docs/agents/issue-tracker.md`〈Conventions〉、`docs/agents/milestone.md`〈驗收票〉照範本改；封存步驟用 `ticket_notes.py --archive` 與 `docs_moved_lines.py`。
- 加 `scripts/ticket_notes.py`、`scripts/docs_moved_lines.py`、`scripts/tests/test_ticket_notes.py`、`scripts/tests/test_docs_moved_lines.py`；`scripts/check_docs.py` 與 `scripts/tests/test_check_docs.py` 換成範本的版本，QIM 自己的 `HookEnvironmentTest` 加回去。這張票先不建 `CHANGELOG.md`（票 02），所以 `變更：` 的檢查還不生效。
- `docs/agents/issue-tracker.md`〈Conventions〉加範本的「等使用者：」那一條；`AGENTS.md` 的 session 結尾照範本改成：做票的 session 要等使用者時寫在票的 Comments、票不改 `done`，答案記成下一行 `決定：`。
- 範本 2026-11-17 版的 `check_docs.py` 比開票時多幾條，換上去時一起過：`決定：` 行不超過 160 字元、四種紀錄行（`Session 紀錄：`、`決定：`、`變更：`、`等使用者：`）寫成清單或縮排也認、done 的票不留沒回答的「等使用者：」、roadmap 有票沒做完的列寫「排在 … 之後」、`CHANGELOG.md` 的格式（票 02 建檔後生效）、`docs/development.md`〈閘門〉八列。`AGENTS.md` 的 session 結尾第 3 步照範本改成「照 `next_step.py` 的結果告訴使用者」（2026-11-17 已換上新版 `next_step.py`）。`docs/agents/ui.md`、`docs/agents/visual.md` 的「收尾票」也改成「驗收票」。
- 進行中的工作（例如 `.scratch/arch-diet/`）已經 done 的票，`Session 紀錄：` 沒有日期的補上：取 progress.md 那一列的日期，沒有就取票檔最後一次 commit 的。

## 驗收

- [ ] `python3 scripts/check_docs.py` 與 `python3 -m unittest discover -s scripts/tests` 綠燈；`diff` 範本的 `check_docs.py` 沒有差異。
- [ ] `grep -rn "抄進\|抄成" AGENTS.md docs/progress.md docs/decisions.md docs/agents docs/development.md` 沒有結果。
- [ ] `python3 scripts/ticket_notes.py m4` 印得出 M4 的票（舊票沒日期時取 commit 日期）。
- [ ] `cargo xtask check --fast` 綠燈。

## Comments
