# 25b: 留碼接英文時，空白接進英文（舊版 Rime 行為）

Status: done
Blocked by: None (can start immediately)
讀: brief §4.3（Shift 那列與下方說明）、§12.2；plan §2.1；票 25 的 Comments；spec〈共同規則〉

## What to build

使用者 2026-11-05 確認：自己用慣的小狼毫是舊版 librime 的行為：左 Shift 留碼接英文後，按空白會把空格接進英文、可以一路打出「hello world」，新版「英文加一個空白上屏、回中文」用不慣。票 25 照的是 librime 新版原始碼，本票改成舊版。

- 先查清舊版 librime（`ascii_composer` / `inline_ascii` 那段改寫前的版本，用 git 歷史找出改寫的 commit 與版本號）在這個狀態下各鍵的行為：空白、Enter、退格、Esc、再按 Shift、數字與標點。照舊版做，記進 brief §12.2（附 commit / 版本）。
- 至少：空白把空格接進英文、狀態不結束；Enter 上屏整段英文（含空格）並回中文。其他鍵照舊版；和票 25 不同的地方逐條寫進 brief §4.3。
- 不加新開關（使用者要的就是這個行為）；票 25 的三種 Shift 設定照舊。
- progress〈偏差與決定〉記一行（使用者決定，推翻票 25 的空白行為）；session 紀錄的票欄寫「M2（補 M1 25b）」。

## 驗收

- [x] 核心測試：打 `anr`、左 Shift、打 `hello`、空白、打 `world`、Enter，上屏 `anrhello world`（或照舊版查到的實際行為，在 Comments 說明）。
- [x] Playwright：同上讀回輸入框。
- [x] brief §4.3、§12.2 已更新；`cargo xtask check`、`bench --check` 綠燈。

## Comments

- 查證：librime `74bd5dc4`（2026-09-15，`feat(ascii_composer): commit_raw_input`）改寫 `inline_ascii`；改寫前的最後發佈版是 1.17.0（`33e78140`），`ascii_composer.cc` 從 1.16.0 到 `74bd5dc4^` 沒變。用鼠鬚管內建 librime 1.16.0 逐鍵實測（`scripts/experiments/rime-inline-ascii/`），結果記在 brief §12.2「舊版 ascii_composer」。驗收那條的實際行為與票上寫的相同：上屏 `anrhello world`。
- 照舊版改的兩處（brief §4.3）：空白接進英文、不結束；再按留碼的 Shift 一律把英文當輸入碼回中文組字（`Lattice::from_input` 重建，選定的分段留著），不再看打過字或退格過，`InlineAscii.edited` 與整個結構刪掉，`Session.inline` 只存英文（plan §0 對照表跟著改）。
- 偏離舊版：英文含方案字母以外的字元（空白、數字、大寫、標點、`z`）時再按留碼的 Shift 上屏英文：QIM 的中文組字只裝方案字母，舊版 librime 把它們切成原樣的分段、空白當分隔。← / Home / Delete 照票 25 吃下不動作（舊版移游標、刪游標後的字，QIM 沒有輸入碼內的游標）。選定的分段遇退格、Esc 照票 25（舊版第一下退格把選定的分段拆回英文、Esc 只清最後一段）。Shift＋空白在舊版會上屏，QIM 的 `Key::Space` 不分 Shift，照空白接進英文。
- 網頁：臨時英文中空白、數字不再上屏，`main.ts` 原本排除它們算選字的判斷成了死碼，拿掉；`report.spec.ts` 那條改成打了空白與數字再按 Enter 上屏、仍不能回報。`key-table.spec.ts` 把「空白上屏英文加空白」那段換成兩條：驗收那條（`hello world`），和打了字再按左 Shift 回中文組字、含空白時上屏英文。
- bench 前後：改動前 = `bench/baseline.json`（HEAD）；改動後 `cargo xtask bench --check` 通過，四個語料集合都沒有翻盤、固定題 25/25。只改臨時英文的處理，排序沒動。`cargo xtask data --verify` 通過。
- 亂按：桌面兩輪各 1,500 步（字母為主，夾左右 Shift、空白、Enter、退格、Esc、數字、標點、大寫、方向鍵、Tab、翻頁、`z`、Ctrl+A，每 300 步換一次 Shift 設定），進臨時英文 186、161 步，沒有頁面錯誤與原生對話框，也沒有「沒組字卻有候選」「臨時英文卻有候選」。
- code-review 已處理：brief 那條改成只寫現況（原本用「原本…現在」寫票 25 的舊行為，歷史留在 progress）；「整串都是方案字母」抽成 `Scheme::spells`，觸發詞的檢查也改用它；`toggle_ascii` 的區域變數 `inline` 與欄位同名，改成 `keeps_code`（`ShiftSwitch::inline` 跟著改名），用 `take_if` 拿掉 `expect`；補「左右都留碼」設定下按右 Shift 回中文組字的測試；brief 補觸控 `text` 在臨時英文中先上屏英文；§12.2 的鍵碼範圍照原始碼寫。
- code-review 未處理：偏離舊版的四處（含方案字母以外的字元時再按 Shift 上屏、Shift＋空白接進英文、選定分段遇退格 / Esc 照票 25、← / Home / Delete 不動作）沒有另外問使用者，在總結裡請使用者確認；`Session.inline` 用 `Option<String>` 表示臨時英文（Primitive Obsession）：原本的結構只剩一個欄位，取最簡；`probe.c`、`run.sh` 的註解是英文，同旁邊 `rime-cases/` 的慣例。
