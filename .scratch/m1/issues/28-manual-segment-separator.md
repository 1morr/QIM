# 28: 手動分隔鍵

Status: done
Blocked by: 27
讀: brief §4.3、§4.4；plan §2.1（`Key`、`Composition`）、§2.3（lattice）；票 21、27 的 Comments；`docs/research/blackbox-baseline.md`（Gboard）；spec〈共同規則〉

## What to build

使用者 2026-11-05 決定：照 Gboard 速成，連打時可以手動指定「這裡斷開」。Gboard 在組字中把 ⇧ 換成分隔鍵「'詞」；QIM 核心目前沒有分隔功能。

- 核心：組字中按分隔鍵，在目前位置插入一個分隔記號；lattice 的段不能跨過分隔記號，其餘照 §2.3 打分。分隔記號顯示在組字裡（樣式照 Gboard 或 Rime 的 `'`，本票定），退格先刪分隔記號。沒有組字時按分隔鍵照原本的字元輸出。
- 實體鍵盤：分隔鍵用 `'`（Rime 拼音方案的慣例；速成不用這個字元），brief §4.3 的鍵位表補一列。
- 觸控：組字中 ⇧ 的主標籤換成「'詞」並送分隔鍵，沒有組字時照票 27 的 Shift 行為（照 Gboard 實測）。
- 學習與自動造詞照分隔後的分段記（plan §2.4）。
- 同輪改 brief §4.3、§4.4、plan §2.1、§2.3；progress〈偏差與決定〉記一行（使用者決定），session 紀錄票欄寫「M2（補 M1 28）」。

## 驗收

- [x] 核心測試：打 `mir` 首選「一台」；打 `mi'r` 時分隔後第一段只能是 `mi` 的字詞，首選照分段排；退格先刪分隔記號。
- [x] Playwright：實體鍵盤打 `mi'r` 讀回組字含分隔記號、上屏結果符合分段；觸控組字中 ⇧ 顯示「'詞」，點了插入分隔。
- [x] brief、plan 已更新；`cargo xtask check`、`bench --check` 綠燈；/code-review 跑完。

## Comments

- 2026-11-05 設計：分隔記號（`qim_core::SEPARATOR`）留在輸入碼裡、算在前一段的碼尾（`mi'r` 的分段是 `mi'`、`r`），所以「各分段的碼接起來是輸入碼」、選定與重算的寫法都不用改。lattice 遇到它照抄前一欄；碼表與詞庫的碼不含 `'`，所以沒有邊跨過它。結尾的分隔記號在候選列查碼時去掉（`mi'` 的候選同 `mi`），只蓋住開頭一段的字詞後面緊接分隔記號時連它一起蓋住。組字顯示照 Gboard 實測的「日弓' 口」（`.scratch/m1/evidence/09-android-t18.png`），字根照原字元顯示 `'`。
- 2026-11-05 不另加 `Key`：實體鍵盤的 `'` 本來就是 `Key::Punct('\'')`，核心在組字中把它當分隔記號；觸控組字中的「'詞」用佈局既有的 `composing` 覆寫成 `{ "punct": "'" }`，殼不必知道哪個字元是分隔記號。各方案共用一個、不放進方案檔（Rime 是方案的 `speller/delimiter`），要換時再加。
- 2026-11-05 查證（子代理讀 librime `9777cb2` 原始碼，補進 brief §12.2）：Rime 的分隔符只擋音節、詞可以跨（`xi'an` 仍出「西安」）；組字顯示含分隔符；使用者詞典的碼不含它。本票照票面「段不能跨過分隔記號」，比 Rime 嚴，brief §4.3 寫明；`an'r` 由「門」「口」兩段拼出，首選仍是門口，同 Gboard。
- 2026-11-05 學習的碼去掉分隔記號（實作決定，記在 progress 偏差）：`mi'r` 選兩次「歹口」記成 `mir` → 歹口，所以不分隔打 `mir` 也成首選，同 Rime 使用者詞典。票面只寫「照分隔後的分段記」，每段的碼照分段、整串的碼不含分隔記號；使用者要分開記再改。自動造詞照舊要分兩次以上選定。
- 2026-11-05 票外補的：
  - 臨時英文裡 `'` 和其他字元一樣接在英文後；按回留碼的 Shift 時，不連著兩個的分隔記號回中文組字（brief 承諾「沒動過就是原樣還原」，`mi'r` 進出臨時英文要回到原樣），連著兩個時上屏英文。英文由組字的輸入碼起頭、只從尾端增刪，不會以分隔記號開頭。
  - 佈局格式加 `composing.a11yLabel`（只加不改、不升版本）：組字中讀屏念「分隔」，不念鍵本身的「大寫」。`Key::accessibility_label` 改收 `KeyState`。
  - 基準：`fixed.tsv` 的輸入碼可含 `'`（「回報」照打的記），`qim-bench` 與 WebAssembly 計時腳本經 `Key::for_input`（wasm `inputChar`）送標點鍵；加固定題 `an'r 門口`。
  - 刪掉票 27 的 Playwright「組字中點 Shift 再點字母：先上屏組字再接大寫字母」：行為被本票推翻，新行為在 `web/e2e/separator.spec.ts`。
- 2026-11-05 沒照 Gboard 的：「'詞」沿用 ⇧ 的圖示字級，比 Gboard 的文字標籤大：格式的 `composing` 沒有樣式欄位，要分開字級得加格式欄位，影響小，不做。組字中按住「'詞」有放大預覽（動作是 `punct`），Gboard 的功能鍵沒有，同樣不做。
- 2026-11-05 已知、不改：有目前分段時按 `'` 和打字母一樣接在輸入碼最後（QIM 沒有輸入碼內的游標），有測試。`x` 本身沒有字（速成的「難」鍵），`x'w` 沒有候選，同單打 `x`，退格可改。
- 2026-11-05 bench 前後對照（改動前 = 開工時 `bench/baseline.json`）：`bench --check` exit 0，四個公開集合與個人集都沒有翻盤、答錯與修正次數同 baseline，固定題 26/26；沒有分隔記號的輸入走的路徑不變，是預期的。p95 原生 41.46 µs、WebAssembly 54.88 µs。
- 2026-11-05 亂按（Playwright 暫存腳本，跑完刪掉）：實體鍵盤 5 個種子 × 150 步（字母、`'` 加權、退格、Esc、空白、Enter、數字、← →、Tab、↓、標點、大寫、左 Shift），觸控 Pixel 7 5 個種子 × 150 步（字母、⇧、退格、空白、Enter、切頁、點候選）：沒有 page error 或 console 錯誤；組字裡的分隔記號不在開頭、不連續；組字中字母頁的 ⇧ 都是「'詞」。第一次觸控跑卡住是腳本在字母頁等「返回」鍵，修腳本後重跑。
- 2026-11-05 code-review（`fe9610a` 起，兩軸）已處理：progress 偏差三條、plan §0 對照表加「分隔記號 `SEPARATOR`」；「輸入碼字元 → 鍵」收成 `Key::for_input`（基準與測試共用）；`lattice.rs` 兩種碼改名 `lookup_code` / `user_code`；臨時英文連打兩個 `'` 回中文時上屏英文（先寫會紅的測試）；有目前分段時按 `'` 的語意寫進 brief 並補測試；觸控 Shift 開著時用實體鍵盤開始組字，⇧ 原本仍畫 ⬆、讀屏 `aria-pressed` 為真，改成 `isShift` 跟著 `tap_action(state)`、殼清掉按下狀態（先寫會紅的 Playwright）。
- 2026-11-05 code-review 沒處理：臨時英文中觸控 ⇧ 也顯示「'詞」、送 `'` 接在英文後，同實體鍵盤按 `'`，不另測。
- 2026-11-05 合併 main（票 17c，`561c92d`）：只有 `bench/fixed.tsv` 衝突，兩邊都留（17c 的題接本票的 `an'r`，共 71 題）。合併後依序跑：`cargo xtask data --verify` 通過；`bench --check` exit 0，四個公開集合沒有翻盤、數字同 main 的 baseline，固定題 71/71，p95 原生 40.33 µs、WebAssembly 51.96 µs；個人集比本機舊 baseline 少錯 9 題（17c 帶來的），`--update-baseline` 後 `bench/baseline.json` 內容不變，只更新本機的個人集 baseline；`cargo xtask check` 通過（e2e `130 passed`）。
- 2026-11-05 第二輪 code-review（`9839ad1..528927f` 的修正與 merge 的解衝突，兩軸）已處理：`Scheme::composes` 的「不在開頭」走不到，刪掉並在註解寫明；WebAssembly 計時腳本原本自己判斷分隔記號（重寫一次 `Key::for_input`），改成 wasm `inputChar` 直接用它，拿掉剛加的 `separator()`；「在目前位置插入」實作成接在最後，progress 偏差補一行。
- 2026-11-05 第二輪 code-review 沒處理：
  - 觸控 Shift 開一次時用實體鍵盤開始組字：⇧ 是「'詞」、不畫按下，但 Shift 仍開著，字母鍵顯示大寫、點了先上屏組字再接大寫（同實體鍵盤 Shift＋字母）。要同時用實體與觸控鍵盤才碰得到，看得到大寫標籤，不另清狀態。
  - 組字中整頁重畫（切頁再回來）時新的 ⇧ 沒有 `aria-pressed`，組字結束的重畫會補回，不留錯的狀態。
  - 「名詞表沒有分隔記號」是誤報：`GLOSSARY.md` 已有條目。commit `fix(core)` 也改了 wasm 與網頁：已 commit，不改寫歷史。
- 2026-11-05 第二輪修正後重跑（依序，bench 與 e2e 不同時）：`bench --check` exit 0，五個集合都沒有翻盤，固定題 71/71，p95 原生 40.38 µs、WebAssembly 51.83 µs；`cargo xtask check` 通過（e2e `130 passed`）。
