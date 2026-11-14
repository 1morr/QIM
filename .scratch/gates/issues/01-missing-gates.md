# 01: 補密鑰掃描、依賴漏洞、依賴更新三道閘門

Status: ready-for-agent
Blocked by: `.scratch/m4/issues/31-m4-close.md`
讀: `.scratch/gates/spec.md`；`docs/development.md`〈閘門〉〈全部檢查〉；`xtask/src/check.rs`；`.github/workflows/ci.yml`；`.githooks/pre-commit`

## What to build

- 密鑰掃描接進 pre-commit（經 `cargo xtask check --fast`）與 CI。
- 依賴漏洞掃描涵蓋 `Cargo.lock` 與 `web/package-lock.json`，進完整的 `cargo xtask check` 與 CI，不進 `--fast`。
- 依賴更新的設定檔涵蓋 cargo、npm（`web/`）、GitHub Actions。
- 〈閘門〉三列換成指令或設定檔；〈環境需求〉補新工具與版本；〈全部檢查〉的表與 `xtask/src/check.rs` 加新步驟。

## 驗收

- [ ] 〈閘門〉沒有「待定」，每一列有行內程式碼。
- [ ] 造假 token 檔 commit 被 pre-commit 擋下、拿掉後放行，輸出貼在 Comments。
- [ ] 漏洞掃描跑過一次，掃出來的各記一條到 Comments（不修）。
- [ ] `cargo xtask check` 綠燈；`cargo test -p xtask` 比對〈全部檢查〉與程式碼的步驟一致。

## Comments
