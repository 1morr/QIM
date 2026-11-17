# 02c: 依賴漏洞閘門擋下 picomatch

Status: ready-for-agent
Blocked by: 02b
讀: `osv-scanner.toml`；`docs/development.md`〈閘門〉（依賴漏洞那一列）；`.scratch/gates/issues/01-missing-gates.md` 的 Comments（前兩條不修的先例）

協調者插的修復票：票 02b 合併後 push（`89403b6`），CI 的依賴漏洞閘門紅了。紅的原因不是 02b 的改動：osv 新列出一則 npm 公告，上一次 CI（`d566db4`）同一份 `web/package-lock.json` 還是綠的。flow-sync spec〈範圍〉只列文件與腳本，這張票為了讓 CI 回到綠燈，例外動 `osv-scanner.toml` 或 `web/package*.json`；不改產品行為。

## What to build

CI 與本機的 `osv-scanner scan source --config=osv-scanner.toml --lockfile=Cargo.lock --lockfile=web/package-lock.json` 回到綠燈。

CI 擋下的那一條：GHSA-fc5q-39xc-vjpf，picomatch `<= 4.0.7` 的 ReDoS（重複的 extglob），High，公告沒有修正版；npm 上最新就是 4.0.7。picomatch 是 dev 依賴，經 `vite@8.3.5` 與它底下的 `tinyglobby` → `fdir` 進來。

- 先查有沒有能修的路：vite、tinyglobby 有沒有不依賴 picomatch 4.0.7 的新版，`overrides` 換得掉嗎。有就修。
- 修不了就照 `osv-scanner.toml` 的慣例忽略這一條，`reason` 寫為什麼現在不修、誰在用它（只在開發伺服器與建置時跑、比對的 glob 是我們自己寫的、不進產出的網頁），並寫明等什麼才回來修。

## 驗收

- [ ] 本機跑上面那條 `osv-scanner` 指令 exit 0，貼結尾輸出。
- [ ] Comments 寫明修了還是忽略、查過哪些路。
- [ ] `cargo xtask check --fast` 綠燈。

## Comments
