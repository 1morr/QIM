# 25: 單按 Shift 照 Rime：左 Shift 留碼接英文

Status: done
Blocked by: None (can start immediately)
讀: brief §4.3（Shift 那列與下方說明）、§7；plan §2.1（`ToggleAscii`、`Options`）、§3.1；`docs/research/rime.md`；票 23 的 Comments；spec〈共同規則〉

## What to build

使用者 2026-11-05 試用後決定：組字中單按 Shift 的預設改成 Rime（小狼毫）預設，並可在設定改。推翻票 23 時「左右都先上屏中文」的決定。

- 預設：左 Shift 照 librime `inline_ascii`：打到一半的碼留著，直接當英文接下去（切到英文狀態，已打的碼原樣成為英文的一部分）；右 Shift 照 `commit_text`：先上屏組字（同按空白）再切英文。沒有組字時兩邊都只切換。行為細節（再按 Shift 回中文時那段英文怎麼處理、Enter / 空白在 inline 狀態的行為）先查 librime `ascii_composer` 與 `default.yaml` 的 `switch_key`（context7 或原始碼，GPL/BSD 只取做法），照 Rime 做，記進 brief §12。
- 設定：`Options` 加一個欄位（名稱照名詞表或本票定），可選「左右照 Rime」「左右都先上屏中文」「左右都留碼接英文」，預設第一種；網頁加對應的選單。殼負責分辨左右 Shift（網頁用 `KeyboardEvent.code`）。觸控鍵盤的 Shift 鍵不在本票（票 27）。
- 同輪改 brief §4.3、§7 的開關清單、plan §2.1；progress〈偏差與決定〉記一行（使用者決定，推翻票 23）。

## 驗收

- [x] 核心測試：組字 `anr` 後左 Shift，組字保留為英文、再打 `x` 接在後面；右 Shift 先上屏「門口」再切英文；三種設定各有一條。
- [x] Playwright：預設下打 `anr` 按左 Shift 再打 `x`、按 Enter，輸入框讀回 `anrx`；按右 Shift 讀回「門口」接英文。切換設定後行為改變。
- [x] brief §4.3、§7、plan §2.1 已更新；`cargo xtask check`、`bench --check` 綠燈。

## Comments

- 照 librime `9777cb2` 的 `ascii_composer`（brief §12.2）：臨時英文中空白上屏英文加一個空白、Enter 上屏英文、退格刪光或 Esc 放棄，都回中文；再按留碼的 Shift，沒動過就還原中文組字，打過字或退格過就上屏（librime 退格就丟掉進來前的輸入）；按 `commit_text` 那邊上屏英文。左右一起按照 librime：從第一個按下計時，先放開的那邊算。
- 偏離 Rime：已選定的分段在臨時英文中留著中文（librime 把整串換回按鍵），和 Enter 一致；退格刪光英文時最後選定的分段放回來重選，同中文的退格。brief §4.3 寫明。
- 命名：`Options.shift_switch: ShiftSwitch { Rime, CommitText, InlineAscii }`（Rime 的 `switch_key`）、`Key::ToggleAscii(ShiftKey)`、`Segment.raw`（Rime 的 `raw` 標記：殼顯示英文、不顯示字根）。`Snapshot.ascii` 在臨時英文中也為真；核心的 `ascii` 欄位只記英文直通。名詞表加「臨時英文」，plan §0 對照表加一列。
- 網頁：`LoneShift` 讀 `KeyboardEvent.code`；臨時英文中 `-` / `=` 是字元、不翻頁，Tab、↑ ↓ 不開字根提示與面板，空白與數字不算選字（回報不收，e2e 有變異驗證）。設定選單同其他開關，不存 `localStorage`。
- 觸控鍵盤的中英鍵先當右 Shift（M1 票 23 的行為），選「左右都留碼」時也會留碼；預設佈局沒有中英鍵。觸控的 Shift 留給票 27。
- bench 前後：改動前 = `bench/baseline.json`（HEAD）；改動後 `cargo xtask bench --check` 通過，沒有翻盤、固定題 25/25。只改 Shift 的處理，排序沒動。`cargo xtask data --verify` 通過。
- 亂按：桌面兩輪各 1,500 步（夾左右 Shift、三種設定、簡繁、Tab、↑ ↓、`-` `=`、數字、組合鍵、點面板），進臨時英文 71、185 步，沒有頁面錯誤與原生對話框，也沒有「沒組字或英文卻有候選、面板」。
- code-review 已處理：退格後與左右一起按改照 librime（原本打了又刪回原樣會還原、左右一起按不算）、`shiftSwitch` 補單元測試、網頁型別 `ShiftSide` 改名 `ShiftKey`、核心的守衛收成 `composing_chinese`、網頁收成 `composingChinese`。
- code-review 未處理：`toKey(event, composing, inlineAscii)` 兩個布林參數（Data Clumps），網頁 prototype 先不包；多一種 `ShiftSwitch` 要改核心、wasm、`keys.ts`、選單四處（跨 wasm 邊界）；`keys.ts` 自己寫 `ShiftSwitch` / `ShiftKey` 字串型別、不 import wasm 的：`npm test` 不必先建 wasm（`docs/development.md`），同 `rerank.ts` 的做法。
- 協調者要 session 紀錄的票欄寫「M1 25」，但 `cargo xtask check` 的 `docs` 步驟要求標進行中的里程碑（M2），所以寫「M2（補 M1 25）」。
