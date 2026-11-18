# 進度

工作單位是票（`.scratch/<里程碑>/issues/NN-slug.md`），狀態與做票的 session 紀錄都在票上（`python3 scripts/ticket_notes.py <里程碑的目錄名>` 印出來）。這裡只記進行中里程碑裡不是做票的 session（訪談、規劃、開工、驗收、流程調整）；里程碑的順序、狀態與閱讀指引在 `docs/roadmap.md`，決定在 `docs/decisions.md`。

## 寫法與封存

- 一列不超過 200 字元；細節在票、spec 或 brief，這裡只放指向它的一句話。
- 「票」欄標里程碑（`M0 開工`），封存時靠它分。
- 里程碑驗收時，它的列搬到 `docs/archive/<里程碑>.md`，步驟在 `docs/agents/milestone.md`〈驗收票〉。本檔只留進行中的里程碑。
- 「下一步」只在沒有票可做的階段（訪談、寫架構、拆票、回顧）決定下一步；有票可做時 `python3 scripts/next_step.py` 從票的狀態算，這一欄只有以 `等使用者：` 開頭時才照樣列出。
- 〈Session 紀錄〉裡到 2026-11-17 gates 01 為止的做票列是舊做法留下的：那時合併的人照票抄一列進來。
- `python3 scripts/check_docs.py`（`cargo xtask check` 的 `docs` 步驟會跑它）強制的規則列在腳本開頭；本檔的是：兩個 `##` 段的順序、session 紀錄是一整張表、每列的長度、最後一列的下一步。

## Session 紀錄

| 日期 | 票 | 結果 | 下一步 |
| --- | --- | --- | --- |
| 2026-11-17 | M5 封存 M4 | 票 31：里程碑 code-review、乾淨 checkout 重驗 spec〈驗收〉、v3 定稿、使用者試用第二期通過；M4 封存到 `docs/archive/M4.md`；見票 31 | 新 session 下 `/1morr:project-init`，再 `/1morr:checkpoint M4` |
| 2026-11-17 | M5 project-init 同步 | 開工加「對程式碼」一步、〈UI〉搬到 `docs/agents/ui.md`、加〈視覺化〉、換新版 `next_step.py`、pre-commit 清 `GIT_*`；新檢查點名進 flow-sync 票 01 | `/1morr:checkpoint M4` |
| 2026-11-17 | M5 checkpoint M4 | 回顧 M4：驗收逐條有證據、6 個機制對程式碼一致；使用者照建議定 7 題（先跑 gates、flow-sync、arch-diet 再拆 M5，M5 分兩期） | `/implement .scratch/gates/issues/01-missing-gates.md` |
| 2026-11-17 | M5 gates 01 | 補三道閘門：gitleaks 密鑰掃描（pre-commit 與 CI 全歷史）、osv-scanner 依賴漏洞、Dependabot；掃出 2 條已不維護公告不修；見票 01 | `/implement .scratch/flow-sync/issues/01-ticket-notes.md` |
| 2026-11-18 | M5 flow-sync 驗收 | 協調者跑完票 01、02 與修復票 02b（e2e 轉發收尾）、02c（picomatch 公告忽略到 2027-02-18）；spec〈驗收〉逐條過，flow-sync 已驗收；見各票 | `/implement .scratch/arch-diet/issues/01-skeleton.md` |
| 2026-11-18 | M5 arch-diet 驗收 | 協調者跑完 6 張票（02–05 平行）；plan 刪除、`ARCHITECTURE.md` 10,792 字、ADR 0009–0019、識別符表進 `GLOSSARY.md`；使用者回覆「都可以，按你建議」；見票 06 | `/to-tickets docs/roadmap.md 拆 M5 的票` |
| 2026-11-18 | M5 開工 | spec、26 張票（第一期 01–12、第二期 13–25、26 驗收）；計劃頁 4 題與對程式碼 14 條使用者照建議同意；見 spec〈票序〉 | `/implement .scratch/m5/issues/01-stale-comments.md` |
