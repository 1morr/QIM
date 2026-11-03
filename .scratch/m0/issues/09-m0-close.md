# 09: M0 收尾

Status: ready-for-agent
Blocked by: 08b
讀: spec〈驗收〉；`docs/progress.md`〈寫法與封存〉；plan §6、§7；`docs/development.md`〈全部檢查〉；票 01–08b 的 `## Comments`

## What to build

M0 的驗收全部重驗一次並留下證據；M0 的 session 紀錄與偏差封存到 `docs/archive/M0.md`，M1 標成進行中，下一個 session 可以直接 `/to-tickets docs/plan.md 拆 M1 的票`。

- **里程碑 code-review**：`/code-review a2a4d5b`（M0 開工時的 commit），Spec 軸讀 spec，Standards 軸讀 `CLAUDE.md`。小問題修掉；不修的與票 01–08b Comments 裡沒處理的發現，整理成一份清單寫在本票 `## Comments`，每條註明留到哪個里程碑或不修的理由。
- **重驗 spec〈驗收〉**，在乾淨的 checkout（`git worktree add --detach` 到暫存目錄，照 `docs/development.md` 從零安裝）跑：
  - `cargo xtask check` 全套（含 `e2e`）。
  - 暫存 repo 啟用 hook，違規的 commit message 被擋、合規的通過（不在本 repo 留測試 commit）。
  - `deps`、`docs` 閘門的雙向變異測試在 `cargo test` 裡（點名測試名稱即可）。
  - `cargo xtask data --verify`：報告的連打字集字數、擴展區字數、Unihan 差異數；建兩次 sha256 相同。
  - Playwright：照 spec〈驗收〉（08b 改過後是 `a` 空白上屏「日」、`ar` 翻頁找到「問」）；截圖路徑。
- **封存**（〈寫法與封存〉，同一個 commit）：
  - M0 的 session 紀錄與偏差整段移到 `docs/archive/M0.md`。
  - 里程碑表 M0 的備註換成 `docs/archive/M0.md` 的連結；M1 的備註改成進行中、尚未拆票。
  - plan §7 表 M0 狀態改完成、M1 改進行中；§7.1 改成指向 spec 與封存檔。
  - progress 這個 session 的紀錄標 `M1`（例如 `M1 封存 M0`），「下一步」寫「新 session 下 `/to-tickets docs/plan.md 拆 M1 的票`」。
  - spec 與票留在原處不搬。

## 驗收

- [ ] 票 08b 已 done；spec〈驗收〉的 Playwright 項照 08b 改過的版本驗。
- [ ] 里程碑 code-review 跑完；沒處理的發現清單（含票 01–08b 留下的）寫在 `## Comments`，每條有去處或理由。
- [ ] 乾淨 checkout 上 `cargo xtask check` 全套綠燈（貼輸出最後幾行）。
- [ ] 暫存 repo 的 hook 實測：違規 message 被擋、合規通過（貼輸出）。
- [ ] `cargo xtask data --verify` 綠燈，貼出三個字數與兩次建置的 sha256。
- [ ] spec〈驗收〉的 Playwright 項讀回成立，截圖路徑寫在 `## Comments`。
- [ ] `docs/archive/M0.md` 有 M0 全部的 session 紀錄與偏差；progress 只留 M1；里程碑表與 plan §7 已更新；同一個 commit。
- [ ] 封存後 `cargo xtask check` 的 `docs` 步驟綠燈。

## Comments

- 2026-11-03 使用者決定（開票後回覆）：`a` 首選要「日」、空白一律上屏第一頁首選，兩項由 08b 做；暫時不用 `/wayfinder`，文件閘門不放寬（票 04 Comments 那條結案）。
