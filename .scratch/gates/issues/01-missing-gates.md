# 01: 補密鑰掃描、依賴漏洞、依賴更新三道閘門

Status: done
Blocked by: `.scratch/m4/issues/31-m4-close.md`
讀: `.scratch/gates/spec.md`；`docs/development.md`〈閘門〉〈全部檢查〉；`xtask/src/check.rs`；`.github/workflows/ci.yml`；`.githooks/pre-commit`

## What to build

- 密鑰掃描接進 pre-commit（經 `cargo xtask check --fast`）與 CI。
- 依賴漏洞掃描涵蓋 `Cargo.lock` 與 `web/package-lock.json`，進完整的 `cargo xtask check` 與 CI，不進 `--fast`。
- 依賴更新的設定檔涵蓋 cargo、npm（`web/`）、GitHub Actions。
- 〈閘門〉三列換成指令或設定檔；〈環境需求〉補新工具與版本；〈全部檢查〉的表與 `xtask/src/check.rs` 加新步驟。

## 驗收

- [x] 〈閘門〉沒有「待定」，每一列有行內程式碼。
- [x] 造假 token 檔 commit 被 pre-commit 擋下、拿掉後放行，輸出貼在 Comments。
- [x] 漏洞掃描跑過一次，掃出來的各記一條到 Comments（不修）。
- [x] `cargo xtask check` 綠燈；`cargo test -p xtask` 比對〈全部檢查〉與程式碼的步驟一致。

## Comments

### 假 token 檔被 pre-commit 擋下（`git add fake-token.txt && git commit`，內容是隨機產生的 `ghp_` 開頭字串）

```
==> secrets
Finding:     github_token = "REDACTED
Secret:      REDACTED
RuleID:      github-pat
Entropy:     4.903056
File:        fake-token.txt
Line:        1
Fingerprint: fake-token.txt:github-pat:1

10:41PM INF 0 commits scanned.
10:41PM INF scanned ~58 bytes (58 bytes) in 24.5ms
10:41PM WRN leaks found: 1
check 失敗：secrets
```

拿掉後放行（`git rm --cached fake-token.txt` 刪檔，再 commit 本票的改動 `dbaeeb0`）：

```
==> secrets
10:48PM INF 0 commits scanned.
10:48PM INF scanned ~10933 bytes (10.93 KB) in 27.5ms
10:48PM INF no leaks found
==> fmt
==> clippy
==> deps
==> docs
==> wasm
==> tsc
==> biome
==> unit
check 通過
```

### 掃出來的（不修）

- 漏洞：`RUSTSEC-2024-0436`，`paste` 1.0.15 已不維護、沒有修正版；經 `candle-core` 的 `gemm` 進來，是 proc-macro，不進產出的程式。寫進 `osv-scanner.toml` 忽略並附理由。
- 漏洞：`RUSTSEC-2026-0374`，`basic-toml` 0.1.10 已不維護、沒有修正版；經 `uniffi_bindgen` 的 `askama` 巨集進來，只在 `qim-ffi` 開 `bindgen` feature 時編譯。同上忽略。
- `web/package-lock.json`：0 筆（`npm audit` 也是 0）。
- 密鑰（全部歷史 `gitleaks git`）：1 筆誤報，`docs/research/open-source-imes.md` 引用的 Android 鍵盤 XML（`android:keyLabel=` 被 `generic-api-key` 當成 key）；`.gitleaks.toml` 只對這條規則、含 `android:<屬性>="` 的行放行，另驗過一般的 `api_key = "…"` 照擋。

### code-review 未處理的發現

- osv-scanner 的整串參數寫在 `check.rs`、`ci.yml`、〈閘門〉三處，gitleaks 版本寫在 `ci.yml` 與〈環境需求〉兩處：沿用 CI 直接寫 `cargo test --workspace` 的做法，沒加「只跑單一步驟」的 xtask 參數。
- 「找不到工具時印安裝指引」只寫 stderr、沒有測試：最外層觀察得到的只有步驟失敗。
- `exec` 裡 `Action::Tool` 與 `Action::Cargo` 兩個分支形狀相同，只差程式名，沒合併。
- 改了 `docs/plan.md`〈開發環境與閘門〉兩句（`check` 的步驟列表、CI 跑什麼），只同步事實、不改決定，合併時請留意。

Session 紀錄：2026-11-17 補上 gitleaks 密鑰掃描（`secrets`，進 pre-commit 與 CI 全歷史）、osv-scanner 依賴漏洞掃描（`audit`，完整 check 與 CI）、Dependabot 設定；事實 M54–M56。
決定：密鑰掃描用 gitleaks 8.30.1，不用後繼 betterleaks（v2 還在 rc）或會連網驗證的 trufflehog；betterleaks v2 穩定後再評估換（Agent 定）
決定：依賴漏洞用 osv-scanner 一個工具掃 Cargo.lock 與 web/package-lock.json，不用 cargo-audit 加 npm audit：npm audit 沒有忽略單一公告的機制（Agent 定）
決定：依賴更新用 Dependabot 不用 Renovate：GitHub 內建、不必裝 app；每週一次，minor 與 patch 併成一個 PR，wasm-bindgen 一家另開（Agent 定）
決定：掃出來沒有修正版的已不維護公告寫進 `osv-scanner.toml` 忽略並附理由，不設到期日，讓 `check` 維持綠燈（Agent 定）
決定：`secrets` 步驟只看暫存區（pre-commit 用），CI 另跑一次全部歷史，擋用 `--no-verify` 跳過 hook 的 commit（Agent 定）
