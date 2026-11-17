# 02: 變更紀錄 CHANGELOG.md

Status: done
Blocked by: 01
讀: `.scratch/flow-sync/spec.md`；範本的 `templates/changelog.md`、`templates/instructions.md`〈文件〉〈每個 session 的流程〉第 5 步與完成標準、`templates/milestone.md`〈驗收票〉第 5 步

## What to build

- 照範本建根目錄的 `CHANGELOG.md`，只有檔頭與空的 `## [Unreleased]`；已驗收的里程碑不補條目。
- `AGENTS.md`〈文件〉加 `CHANGELOG.md` 一行；〈每個 session 的流程〉第 5 步加 `變更：` 行、完成標準加「至少一行 `變更：`」；`docs/agents/milestone.md`〈驗收票〉加 `ticket_notes.py --changelog` 併進 `[Unreleased]`、要不要發版先問使用者。
- 進行中的工作已經 done 的票各補 `變更：` 行（照票的內容寫，沒有使用者看得到的改動寫 `變更：無`）。

## 驗收

- [x] `CHANGELOG.md` 存在，`python3 scripts/check_docs.py` 綠燈（它在有 `CHANGELOG.md` 時才查 `變更：`）。
- [x] `AGENTS.md` 與 `docs/agents/milestone.md` 寫明 `變更：` 行與 `--changelog` 的用法。
- [x] `cargo xtask check --fast` 綠燈。

## Comments

code-review 未處理的發現：
- `變更：` 的六個分類同時寫在 AGENTS.md 第 5 步、`check_docs.py` 與 `ticket_notes.py` 的 `KINDS`、`CHANGELOG.md` 檔頭，改分類要動多處；都是範本原文，照範本留著，要改先改範本。
- AGENTS.md〈文件〉說 `CHANGELOG.md`「依功能記」，`CHANGELOG.md` 檔頭與 milestone.md 是依分類分段、條目以功能名開頭，讀者可能以為要依功能分段；範本措辭，照範本留著。
- 本票驗收只要 `cargo xtask check --fast`（綠燈）。全套 `cargo xtask check` 跑三次都停在 `e2e`：324 passed，另有 1 failed 或「1 error was not a part of any test」，出在 `web/e2e/insecure-context.spec.ts` 的 `beforeEach` route handler（測試結束時 `route.fetch` 還沒回來）；單獨跑該檔 2 passed。本票沒改 `web/`，是既有的 flaky，不在 flow-sync 範圍，沒修。

Session 紀錄：2026-11-17 照範本建 `CHANGELOG.md`，AGENTS.md、`docs/agents/milestone.md`、`issue-tracker.md` 寫明 `變更：` 行與驗收票的 `--changelog`，flow-sync 01 補 `變更：無`
決定：flow-sync 01 補 `變更：無`：它只改流程文件與檢查腳本，使用者看不到產品改動（Agent 定）
決定：`docs/agents/issue-tracker.md`〈Conventions〉的 Comments 那一行票上沒列，照 spec 共同規則「`docs/agents/` 以範本為準逐段改」一併換成範本含 `變更：` 的版本（Agent 定）
變更：無
