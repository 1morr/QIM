# 02: 變更紀錄 CHANGELOG.md

Status: ready-for-agent
Blocked by: 01
讀: `.scratch/flow-sync/spec.md`；範本的 `templates/changelog.md`、`templates/instructions.md`〈文件〉〈每個 session 的流程〉第 5 步與完成標準、`templates/milestone.md`〈驗收票〉第 5 步

## What to build

- 照範本建根目錄的 `CHANGELOG.md`，只有檔頭與空的 `## [Unreleased]`；已驗收的里程碑不補條目。
- `AGENTS.md`〈文件〉加 `CHANGELOG.md` 一行；〈每個 session 的流程〉第 5 步加 `變更：` 行、完成標準加「至少一行 `變更：`」；`docs/agents/milestone.md`〈驗收票〉加 `ticket_notes.py --changelog` 併進 `[Unreleased]`、要不要發版先問使用者。
- 進行中的工作已經 done 的票各補 `變更：` 行（照票的內容寫，沒有使用者看得到的改動寫 `變更：無`）。

## 驗收

- [ ] `CHANGELOG.md` 存在，`python3 scripts/check_docs.py` 綠燈（它在有 `CHANGELOG.md` 時才查 `變更：`）。
- [ ] `AGENTS.md` 與 `docs/agents/milestone.md` 寫明 `變更：` 行與 `--changelog` 的用法。
- [ ] `cargo xtask check --fast` 綠燈。

## Comments
