# 25b: 留碼接英文時，空白接進英文（舊版 Rime 行為）

Status: ready-for-agent
Blocked by: None (can start immediately)
讀: brief §4.3（Shift 那列與下方說明）、§12.2；plan §2.1；票 25 的 Comments；spec〈共同規則〉

## What to build

使用者 2026-11-05 確認：自己用慣的小狼毫是舊版 librime 的行為：左 Shift 留碼接英文後，按空白會把空格接進英文、可以一路打出「hello world」，新版「英文加一個空白上屏、回中文」用不慣。票 25 照的是 librime 新版原始碼，本票改成舊版。

- 先查清舊版 librime（`ascii_composer` / `inline_ascii` 那段改寫前的版本，用 git 歷史找出改寫的 commit 與版本號）在這個狀態下各鍵的行為：空白、Enter、退格、Esc、再按 Shift、數字與標點。照舊版做，記進 brief §12.2（附 commit / 版本）。
- 至少：空白把空格接進英文、狀態不結束；Enter 上屏整段英文（含空格）並回中文。其他鍵照舊版；和票 25 不同的地方逐條寫進 brief §4.3。
- 不加新開關（使用者要的就是這個行為）；票 25 的三種 Shift 設定照舊。
- progress〈偏差與決定〉記一行（使用者決定，推翻票 25 的空白行為）；session 紀錄的票欄寫「M2（補 M1 25b）」。

## 驗收

- [ ] 核心測試：打 `anr`、左 Shift、打 `hello`、空白、打 `world`、Enter，上屏 `anrhello world`（或照舊版查到的實際行為，在 Comments 說明）。
- [ ] Playwright：同上讀回輸入框。
- [ ] brief §4.3、§12.2 已更新；`cargo xtask check`、`bench --check` 綠燈。

## Comments
