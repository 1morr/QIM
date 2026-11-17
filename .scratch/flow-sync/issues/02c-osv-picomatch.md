# 02c: 依賴漏洞閘門擋下 picomatch

Status: done
Blocked by: 02b
讀: `osv-scanner.toml`；`docs/development.md`〈閘門〉（依賴漏洞那一列）；`.scratch/gates/issues/01-missing-gates.md` 的 Comments（前兩條不修的先例）

協調者插的修復票：票 02b 合併後 push（`89403b6`），CI 的依賴漏洞閘門紅了。紅的原因不是 02b 的改動：osv 新列出一則 npm 公告，上一次 CI（`d566db4`）同一份 `web/package-lock.json` 還是綠的。flow-sync spec〈範圍〉只列文件與腳本，這張票為了讓 CI 回到綠燈，例外動 `osv-scanner.toml` 或 `web/package*.json`；不改產品行為。

## What to build

CI 與本機的 `osv-scanner scan source --config=osv-scanner.toml --lockfile=Cargo.lock --lockfile=web/package-lock.json` 回到綠燈。

CI 擋下的那一條：GHSA-fc5q-39xc-vjpf，picomatch `<= 4.0.7` 的 ReDoS（重複的 extglob），High，公告沒有修正版；npm 上最新就是 4.0.7。picomatch 是 dev 依賴，經 `vite@8.3.5` 與它底下的 `tinyglobby` → `fdir` 進來。

- 先查有沒有能修的路：vite、tinyglobby 有沒有不依賴 picomatch 4.0.7 的新版，`overrides` 換得掉嗎。有就修。
- 修不了就照 `osv-scanner.toml` 的慣例忽略這一條，`reason` 寫為什麼現在不修、誰在用它（只在開發伺服器與建置時跑、比對的 glob 是我們自己寫的、不進產出的網頁），並寫明等什麼才回來修。

## 驗收

- [x] 本機跑上面那條 `osv-scanner` 指令 exit 0，貼結尾輸出。
- [x] Comments 寫明修了還是忽略、查過哪些路。
- [x] `cargo xtask check --fast` 綠燈。

## Comments

### 修了還是忽略：忽略（`osv-scanner.toml` 加 `effectiveUntil = 2027-02-18`）

查過的路，全都修不了：

- 公告（OSV.dev）的範圍是 `introduced 0`、`last_affected 4.0.7`：picomatch 每一版都受影響；npm 上 picomatch 最新就是 4.0.7（2026-08-24），沒有修正版可升級，`overrides` 也沒有不受影響的版本可換。
- vite 最新 8.3.8 與 `vite@beta` 8.4.0-beta.1 都直接依賴 `picomatch ^4.0.7`；tinyglobby 最新 0.2.17 要 `^4.0.4`；fdir 6.5.0 是可選的 peer `^3 || ^4`。升 vite 或 tinyglobby 都甩不掉它。
- lockfile 裡依賴 picomatch 的只有 vite、tinyglobby、fdir，都是 dev。我們的 `src`、`e2e`、`test`、`bench`、`vite.config.ts` 沒有任何 glob；`vite build` 的產出 grep 不到 picomatch。ReDoS 要攻擊者能塞 pattern，這裡 pattern 只有 vite 內建的。
- `effectiveUntil` 有效：把日期改成過去的另一份設定跑同一條指令，exit 1。用舊名 `ignoreUntil` 會印棄用警告，改成 `effectiveUntil`（事實 M55 的欄位名）。

本機 `osv-scanner scan source --config=osv-scanner.toml --lockfile=Cargo.lock --lockfile=web/package-lock.json` 結尾（exit 0）：

```
GHSA-fc5q-39xc-vjpf and 1 alias have been filtered out because: picomatch 全部版本都受影響，沒有修正版，…
Filtered 3 vulnerabilities from output

No issues found
```

### code-review 未處理的發現

- 無。已處理：`ignoreUntil` 換成 `effectiveUntil`；`reason` 不寫死 vite 版本、改說 glob 只有 vite 內建的；到期日的寫法同步到 `docs/development.md`〈閘門〉。

Session 紀錄：2026-11-18 查過 picomatch ReDoS 公告沒有修正版、vite 與 tinyglobby 都甩不掉，照慣例在 `osv-scanner.toml` 忽略並設到期日，依賴漏洞閘門回到綠燈
決定：picomatch 公告忽略並設 `effectiveUntil = 2027-02-18`：套件還有人維護，到期讓閘門再紅一次回來查修正版；已不維護的照舊不設到期日（Agent 定）
決定：`osv-scanner.toml` 開頭與〈閘門〉加一句「還有人維護的套件加 `effectiveUntil`」，把這次的做法寫成慣例（Agent 定）
變更：無
