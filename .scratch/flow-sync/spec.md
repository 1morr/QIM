# flow-sync：做票的紀錄留在票上、加變更紀錄

不是里程碑，規格就是本檔（使用者 2026-11-14 同意開成票）。把做票流程對齊 1morr 的 project-init 現在的範本（plugin 裡 `skills/project-init/`，下稱範本）。不改產品行為；只動 `AGENTS.md`、`docs/`、`scripts/`、`CHANGELOG.md`、`.scratch/flow-sync/`。

## 目標

做票的 session 紀錄、決定與使用者看得到的改動只寫在票上，沒有人再抄進 `docs/progress.md` 與 `docs/decisions.md`；封存與 `CHANGELOG.md` 由驗收票用腳本從票上組出來。平行的票不再互撞，合併的人少一步。

## 範圍

### 做

- 票的 Comments 改成三種行：`Session 紀錄：<YYYY-MM-DD> <做了什麼>`、`決定：<一句>（<決定者>）`、`變更：<分類>｜<功能>｜<一句>`（或 `變更：無`）。progress.md 與 decisions.md 只記不是做票的 session（訪談、規劃、開工、驗收）。
- 換成範本的 `scripts/check_docs.py`、加 `scripts/ticket_notes.py`、`scripts/docs_moved_lines.py` 與它們的測試；`cargo xtask check` 的 `docs` 步驟照舊跑 `scripts/check_docs.py` 與 `scripts/tests`。
- 照範本建 `CHANGELOG.md`（Keep a Changelog）。
- 里程碑的最後一張票從「收尾票 `NN-m<N>-close`」改叫「驗收票 `NN-<里程碑>-acceptance`」；已有的 `-close.md` 不改名（檢查兩種都收）。

### 不做

- M0–M4 已封存的紀錄、票與 `docs/archive/` 不改；不替已驗收的里程碑補 `變更：` 或 `CHANGELOG.md` 條目。
- 不碰 `docs/plan.md` → `ARCHITECTURE.md`（那是 `.scratch/arch-diet/`）。

## 共同規則

1. 開工時 M4 已驗收、封存（票 31 做完），`.scratch/gates/` 也做完（`docs/development.md`〈閘門〉沒有「待定」）；M4 還沒封存就停下來回報，不在第二期做到一半換流程。
2. 指令檔與 `docs/agents/` 的段落以範本為準逐段改，QIM 自己的段落（〈目錄〉〈驗收輸入行為〉、協調者的寫法、`cargo xtask` 的閘門）原樣保留。
3. 和 `.scratch/arch-diet/` 改同一批檔（`AGENTS.md`、`docs/agents/milestone.md`、`issue-tracker.md`、`ticket-template.md`）：後做的那張在對方的改動上接著改，不覆蓋。
4. 一個 session 一張票；每張票收尾跑 `cargo xtask check --fast`。

## 驗收

- `cargo xtask check` 全套綠燈，`scripts/check_docs.py` 與範本相同（QIM 自己加的測試保留）。
- `AGENTS.md`、progress.md、decisions.md、`docs/agents/`、`docs/development.md` 裡沒有「合併的人抄進 progress.md／decisions.md」的寫法。

### 使用者試

- 無：只改流程文件與檢查腳本，沒有使用者看得到的產品改動。下一個里程碑第一張票做完時，使用者跑 `python3 scripts/ticket_notes.py <里程碑的目錄名>` 看得到那張票的紀錄。

## 票序

01 → 02 → 02b → 02c（02b、02c 是協調者插的修復票：e2e 紅燈、依賴漏洞閘門）。
