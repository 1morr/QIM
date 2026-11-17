# 01: 做票的紀錄只留在票上

Status: done
Blocked by: 無（原本等 gates 的票把 `docs/development.md`〈閘門〉的「待定」換掉，範本的檢查才過；2026-11-17 已做完）
讀: `.scratch/flow-sync/spec.md`；範本的 `templates/instructions.md`〈文件〉〈使用者確認〉〈每個 session 的流程〉、`templates/progress.md`、`templates/decisions.md`、`templates/milestone.md`、`templates/issue-tracker.md`

## What to build

- `AGENTS.md`〈文件〉〈使用者確認〉〈每個 session 的流程〉換成範本的寫法：`Session 紀錄：<日期> …` 與 `決定：` 留在票上、沒人抄；第 1 步加跑 `python3 scripts/ticket_notes.py <里程碑的目錄名>`；驗收票自己改 progress.md 與 decisions.md。〈Skills〉的「收尾票 `NN-m<N>-close`」改成「驗收票 `NN-<里程碑>-acceptance`」。
- progress.md〈寫法與封存〉、decisions.md〈寫法〉、`docs/development.md`〈全部檢查〉那句、`docs/agents/issue-tracker.md`〈Conventions〉、`docs/agents/milestone.md`〈驗收票〉照範本改；封存步驟用 `ticket_notes.py --archive` 與 `docs_moved_lines.py`。
- 加 `scripts/ticket_notes.py`、`scripts/docs_moved_lines.py`、`scripts/tests/test_ticket_notes.py`、`scripts/tests/test_docs_moved_lines.py`；`scripts/check_docs.py` 與 `scripts/tests/test_check_docs.py` 換成範本的版本，QIM 自己的 `HookEnvironmentTest` 加回去。這張票先不建 `CHANGELOG.md`（票 02），所以 `變更：` 的檢查還不生效。
- `docs/agents/issue-tracker.md`〈Conventions〉加範本的「等使用者：」那一條；`AGENTS.md` 的 session 結尾照範本改成：做票的 session 要等使用者時寫在票的 Comments、票不改 `done`，答案記成下一行 `決定：`。
- 範本 2026-11-17 版的 `check_docs.py` 比開票時多幾條，換上去時一起過：`決定：` 行不超過 160 字元、四種紀錄行（`Session 紀錄：`、`決定：`、`變更：`、`等使用者：`）寫成清單或縮排也認、done 的票不留沒回答的「等使用者：」、roadmap 有票沒做完的列寫「排在 … 之後」、`CHANGELOG.md` 的格式（票 02 建檔後生效）、`docs/development.md`〈閘門〉八列。`AGENTS.md` 的 session 結尾第 3 步照範本改成「照 `next_step.py` 的結果告訴使用者」（2026-11-17 已換上新版 `next_step.py`）。`docs/agents/ui.md`、`docs/agents/visual.md` 的「收尾票」也改成「驗收票」。
- 進行中的工作（例如 `.scratch/arch-diet/`）已經 done 的票，`Session 紀錄：` 沒有日期的補上：取 progress.md 那一列的日期，沒有就取票檔最後一次 commit 的。

## 驗收

- [x] `python3 scripts/check_docs.py` 與 `python3 -m unittest discover -s scripts/tests` 綠燈；`diff` 範本的 `check_docs.py` 沒有差異。
- [x] `grep -rn "抄進\|抄成" AGENTS.md docs/progress.md docs/decisions.md docs/agents docs/development.md` 沒有結果。
- [x] `python3 scripts/ticket_notes.py m4` 印得出 M4 的票（舊票沒日期時取 commit 日期）。
- [x] `cargo xtask check --fast` 綠燈。

## Comments

範本以 1morr-skills 的 project-init（commit 8ba45e6，2026-11-17）為準；`check_docs.py`、`ticket_notes.py`、`docs_moved_lines.py` 與三個測試逐字照抄，`test_check_docs.py` 只多加回 QIM 的 `HookEnvironmentTest`。範本裡 `變更：`、`CHANGELOG.md`、`--changelog` 的句子留給票 02。

code-review 未處理的發現：
- 驗收票同時是「做票」（check_docs 要它 done 時留帶日期的 `Session 紀錄：`）又自己改 progress.md、把 `等使用者：試用` 寫進 progress.md（milestone.md〈驗收票〉第 4 步）。範本就是這樣寫，照範本留著；要改先改範本。
- 「紀錄只在票上」這條在 AGENTS.md、progress.md、decisions.md、issue-tracker.md 各寫一次，流程再改要動多處；各處都是範本原文，照範本留著。
- commit type 用 `docs:`（新增的是文件檢查腳本），不改。

Session 紀錄：2026-11-17 做票的紀錄改成只留在票上：換範本新版 `check_docs.py`、加 `ticket_notes.py` 與 `docs_moved_lines.py`，AGENTS.md、progress、decisions、`docs/agents/` 照範本改，收尾票改名驗收票
決定：progress.md、decisions.md 裡舊做法抄進來的做票列與決定留著，〈寫法〉各加一句註明到 gates 01 為止是舊做法（Agent 定）
決定：gates 已驗收、不算進行中，仍替它 done 的票 01 補 `Session 紀錄：` 日期 2026-11-17，新檢查與 `ticket_notes.py` 才讀得到日期（Agent 定）
決定：AGENTS.md〈目錄〉的 `scripts/` 列補上 `next_step.py`、`ticket_notes.py`、`docs_moved_lines.py`，雖是 QIM 自己的段落，不補就和 repo 不符（Agent 定）
決定：plan〈文件的閘門〉那句改成要 `-acceptance.md` 驗收票（`-close.md` 也算），只同步 `check_docs.py` 的行為，改名本身是 spec 已定的（Agent 定）
決定：issue-tracker.md〈Conventions〉的 `讀:` 例子與 milestone.md 寫架構的步驟仍寫 plan，留給 arch-diet 改（Agent 定）
