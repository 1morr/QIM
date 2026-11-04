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
- 進度（2026-11-05，額度用完中斷）：只做完查證，核心、網頁、文件都還沒改。
- 查證：librime `74bd5dc4`（2026-09-15，`feat(ascii_composer): commit_raw_input`）改寫 `inline_ascii`；改寫前的最後發佈版是 1.17.0（`33e78140`），`ascii_composer.cc` 從 1.16.0 到 `74bd5dc4^` 沒變。舊版 `ProcessKeyEvent`：`ascii_mode` 且組字中時 0x20–0x7f（含空白）一律 `PushInput`，其餘鍵交給後面的 processor；組字清空時 `OnContextUpdate` 關 `ascii_mode`。
- 實測（鼠鬚管內建 librime 1.16.0、quick5，`scripts/experiments/rime-inline-ascii/run.sh`）：`anr`、左 Shift、`hello`、空白、`world`、Enter → 上屏 `anrhello world`、回中文。空白可連打兩個；數字、標點、`-` `=`、大寫都接上；退格逐字刪，刪光回中文；Esc 清空回中文；右 Shift（`commit_text`）上屏英文回中文；再按左 Shift 一律把目前整串當中文輸入碼重新組字（`anrx` → 中文「問的」組字、`anrx y` 的空白當分隔），不看打過字或退格過。已選定的分段留著（同 QIM 票 25 的偏離）。← / Home 移游標、Delete 刪游標後的字；Tab、PageDown、↑ ↓ 無作用。
- 與票 25 不同、要改的：空白接進英文不結束；再按留碼的 Shift 一律回中文組字（英文全是方案字母時用它重建 lattice；含數字、空白、大寫、標點、`z` 時 QIM 的中文組字裝不下，改上屏英文回中文，brief 寫成偏離）；`InlineAscii.edited` 可刪。維持 QIM 的：方向鍵等吃下不動作（QIM 沒有輸入碼內的游標）、選定分段遇退格 / Esc 照票 25（舊版會把選定分段拆回英文、Esc 只清最後一段，不學）。
- 要動的地方：`crates/qim-core/src/session.rs` 的 `toggle_ascii`、`handle_inline`（`Key::Space` 改成接 `' '`）；`crates/qim-core/tests/session.rs` 加驗收那條與再按 Shift 的測試；`web/e2e/key-table.spec.ts:183` 那條改成空白接進英文、加驗收的 Playwright；`web/src/main.ts:629` 的回報判斷確認仍對；brief §4.3 臨時英文那條、§12.2 加舊版一條（附 `74bd5dc4`、1.17.0）；plan §2.1 若有提空白也改。
