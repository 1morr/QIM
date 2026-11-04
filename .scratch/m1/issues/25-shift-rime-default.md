# 25: 單按 Shift 照 Rime：左 Shift 留碼接英文

Status: ready-for-agent
Blocked by: None (can start immediately)
讀: brief §4.3（Shift 那列與下方說明）、§7；plan §2.1（`ToggleAscii`、`Options`）、§3.1；`docs/research/rime.md`；票 23 的 Comments；spec〈共同規則〉

## What to build

使用者 2026-11-05 試用後決定：組字中單按 Shift 的預設改成 Rime（小狼毫）預設，並可在設定改。推翻票 23 時「左右都先上屏中文」的決定。

- 預設：左 Shift 照 librime `inline_ascii`：打到一半的碼留著，直接當英文接下去（切到英文狀態，已打的碼原樣成為英文的一部分）；右 Shift 照 `commit_text`：先上屏組字（同按空白）再切英文。沒有組字時兩邊都只切換。行為細節（再按 Shift 回中文時那段英文怎麼處理、Enter / 空白在 inline 狀態的行為）先查 librime `ascii_composer` 與 `default.yaml` 的 `switch_key`（context7 或原始碼，GPL/BSD 只取做法），照 Rime 做，記進 brief §12。
- 設定：`Options` 加一個欄位（名稱照名詞表或本票定），可選「左右照 Rime」「左右都先上屏中文」「左右都留碼接英文」，預設第一種；網頁加對應的選單。殼負責分辨左右 Shift（網頁用 `KeyboardEvent.code`）。觸控鍵盤的 Shift 鍵不在本票（票 27）。
- 同輪改 brief §4.3、§7 的開關清單、plan §2.1；progress〈偏差與決定〉記一行（使用者決定，推翻票 23）。

## 驗收

- [ ] 核心測試：組字 `anr` 後左 Shift，組字保留為英文、再打 `x` 接在後面；右 Shift 先上屏「門口」再切英文；三種設定各有一條。
- [ ] Playwright：預設下打 `anr` 按左 Shift 再打 `x`、按 Enter，輸入框讀回 `anrx`；按右 Shift 讀回「門口」接英文。切換設定後行為改變。
- [ ] brief §4.3、§7、plan §2.1 已更新；`cargo xtask check`、`bench --check` 綠燈。

## Comments
