# 21: 組字中修改

Status: done
Blocked by: 10, 18
讀: plan §2.1（組字中修改、`Composition`、`focus_segment`）、§2.3；brief §4.3（← / →）、§4.4（點組字的某一段）；spec〈共同規則〉

## What to build

連打後發現中間某段的字錯了，可以移到那段改選，不必整串重打：桌面用 ← / →，觸控點組字裡的分段。改完的段固定，其他分段用同一把尺重算。

- `Snapshot.composition` 帶各分段（碼、字、是否已選定）與目前分段（plan §2.1）；網頁顯示分段並標出目前那段。
- `Left` / `Right` 在分段間移動、`focus_segment(i)` 直接跳；候選改列該段的字詞（參考 librime caret 與 Mozc 分段轉換）。
- 選定後該段固定，其餘未固定的段重算（plan §2.1）；空白上屏全部。
- 票 10 讓 `Left` / `Right` 先吃下不動作，本票換成真的行為，`handled` 為假時組字是空的仍成立。

## 驗收

- [x] 核心測試：移到中間分段改選後，其他未固定分段的字等於「固定該段後重算」的結果。
- [x] Playwright（實體鍵盤）：打一串連打，← 移到中間分段改選別的字，空白上屏，讀回輸入框符合重算結果。
- [x] Playwright（觸控）：點組字裡的一段，讀回候選改列該段，選字後上屏結果正確。
- [x] 「`handled` 為假時組字是空的」性質測試仍綠。

## Comments

- 前面的分段照顯示的字固定（偏離票的字面「其餘未固定的段重算」）：重算的輸入碼從頭建、不接選定分段的字（和選開頭一段相同），單獨重算前面只會丟掉跨段 bigram 選出、使用者看過的字。所以選定的分段永遠在開頭，`Session` 的「選定 + 剩下的 lattice」結構不變；跨段接前文等 M3 `set_context`。寫進 plan §2.1、progress 偏差。
- `Composition.focus` 預設是分段數（沒有目前分段，候選列剩下的整串），← 從最後一段往前，所以票的「← 移到中間分段」成立；原本「目前分段是第一個沒選定的」的兩個測試改成新語意。打字、退格回到沒有目前分段；有目前分段時空白上屏整個組字。
- 改選第 i 段：選的只蓋住它開頭時，那段剩下的碼和後面的分段（含已選定的）一起重算；整段換掉時後面已選定的不動。選完沒有剩下的輸入碼就上屏（維持「組字中剩下的輸入碼不是空的」）。
- 票外加的：公開 `Session::select(index)`（全部候選的位置，照 librime `select_candidate`）、wasm `select` / `focusSegment` / `pageSize` 與 `Snapshot.segments` / `focus` / `page`，網頁點候選選字：觸控驗收「選字後上屏」要用。`WebSnapshot.selected` / `unselected` 沒人用了，刪掉，延遲腳本改讀 `segments`。
- 網頁的分段顯示：選定的顯示字詞、沒選定的顯示輸入碼（textContent 和原本相同，既有 e2e 不動），目前分段反白。看不到沒選定分段的字，要靠候選判斷哪段錯；票 22 改成字根後再看要不要顯示字詞。
- 性質測試加 `Action`（按鍵、`focus_segment`、`select`）進隨機序列，新增「沒選定的分段等於把剩下的輸入碼從頭打一次」。
- bench 前後對照：改動前 = `bench/baseline.json`（HEAD）；改動後 `cargo xtask bench --check` 通過，各集合答錯、修正次數都和 baseline 相同（yue-wiki 1139 / 4607、zh-wiki 1121 / 4495、common-voice-yue 1213 / 6236、hkcancor 811 / 4518、personal 1106 / 1500），預設路徑行為沒變。
- code-review 未處理：`Session::segments()` 每次呼叫都回溯首選路徑，← / → 一次算 4–5 次；打字的熱路徑每鍵仍只算一次，每鍵延遲 p95 沒變，不快取。改選中間段後上屏會學到前面照顯示固定的分段，和組字中按標點上屏整個組字時一樣，不另處理。
- 前面固定、不重算這個決定已由使用者確認（協調者轉告），plan §2.1 與 progress 偏差已註明。
- merge main（票 13、22）：字根改放在每個分段（`Segment.radicals`），`handle`、`select`、`focus_segment` 回傳前都經 `with_radicals` 補上；`Composition.radicals` 改成 `unselected_radicals()`，不留兩份。網頁沒選定的分段顯示字根，本票 e2e 改讀字根。
- 點候選和票 22 的長按候選衝突（都在 `pointerdown`）：改成放開時才選字、長按到門檻切換提示不選字、按住滑動兩者都不做（Gboard 的做法），brief §4.4 補上；票 22 的兩個 e2e 原本「點一下不叫出提示」，現在點一下會選字上屏，改成先驗選字再重打。
- merge 後重跑：`data --verify` 通過（格式 5），`bench --check` 通過、各集合答錯與修正次數和 baseline 相同，`cargo xtask check` 全套通過（e2e 41 項）。
