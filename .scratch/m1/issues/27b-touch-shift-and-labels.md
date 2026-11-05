# 27b: 觸控標點照鍵上的字、Shift 狀態與讀屏標籤細節

Status: done
Blocked by: 26b
讀: 票 27、28 的 Comments；brief 觸控鍵盤章節、§4.4；plan §2.6（佈局格式）；`.scratch/m1/evidence/27-*.png`；spec〈共同規則〉

## What to build

M1 封存後補票的審查（2026-11-05，獨立審查，實際用觸控操作過）找到以下小問題，使用者 2026-11-05 同意照建議修。審查的探測與截圖在 `/private/tmp/claude-501/-Users-roxy-orca-projects-QIM/4808186f-7ee6-44a4-9688-748b966c8ba8/scratchpad/review-b/`。排在 26b 之後是因為都改 `web/src/main.ts`。

0. **觸控標點照鍵上的字打**（使用者 2026-11-05 決定，推翻票 27「觸控 ASCII 標點送 `punct`、中文狀態由核心轉全形」）：觸控鍵每個符號都標得清楚，不必像實體鍵盤猜半形全形。現況：字母頁「。」送 `punct '.'`，數字後打出半形「.」；符號頁「.」在中文狀態打出全形「。」（Gboard 打半形）。改成：
   - 觸控的標點鍵一律輸出鍵上的字（字母頁「，」「。」是全形、符號頁的 ASCII 標點是半形），同 Gboard；「數字後自動半形」（票 26）只留給實體鍵盤。
   - 組字中點標點仍先上屏首選再接標點、臨時英文中照舊接在英文後（票 27 已有的行為與測試保留；接上去的是鍵上的字）。做法本票定（例如核心加一種「照原字輸出」的標點鍵，或 `text` 走同一條先上屏的路），寫進 Comments 與 plan §2.1、§2.6。
   - 先寫會紅的 Playwright：數字列「3」→ 字母頁「。」讀回「3。」；符號頁「日」…「.」在中文狀態讀回半形「.」；組字中點「。」先上屏首選再接「。」。改掉與新行為衝突的舊測試（票 26、27 的「3.5」那幾條改成符號頁的「.」；票 26b 的「符號頁「3」→「返回」→ 字母頁句號讀回「3.」」改成讀回「3。」，`Key::Modifier`（切頁、⇧ 不結束數字）本身保留）。
   - 同輪改 brief §4.4、§4.7（觸控那句）與 plan；progress〈偏差與決定〉記一行（使用者決定）。
1. Shift 開著時字母鍵讀屏念「Q，q」：`crates/qim-layout/src/model.rs` 約 329、347 行 `accessibility_label` 用 `self.hint`，不是 `display_hint(state)`，畫面上卻沒有角標。改用 `display_hint(state)`，補測試。
2. 開一次的 Shift 打了逗號（punct）或切頁後仍開著，只有打字母才解除（`model.rs` 約 244 行 `after_key` 只看 `Letter`）。先在 Android 模擬器的 Gboard 實測（`orca emulator-android`；若沒有 AVD 照 27 的做法建一個，做完刪），照 Gboard 改；實測不了就記在 Comments 並維持現狀。
3. 切數字列開關會重建 `Layout`，鎖定的大寫與目前頁被清掉（`web/src/main.ts` 約 572 行）：把 Shift 與頁的狀態搬過去，補 Playwright。
4. 數字列開時鍵變矮，字母角標貼著字根（「中l」「心p」，見 `27-number-row.png`）：角標位置照鍵高等比例調整。
5. Shift 時大寫字母比 Gboard 小很多（`27-letters.png`）：照 Gboard 截圖加大字級。
6. 逗號鍵讀屏念「，，長按切到常用」：改寫它的 `a11yLabel`。
7. 佈局驗證：`composing` 只寫 `a11yLabel`、沒寫 `label` 時被默默忽略，`validate.rs` 加一條警告（M5 開放自訂佈局前補上），補測試。
8. progress session 紀錄票欄寫「M2（補 M1 27b）」。

改 UI 的項目都要實跑 Playwright 截圖對照 Gboard 截圖。

## 驗收

- [x] 測試：觸控標點照鍵上的字（數字後字母頁「。」仍是「。」、符號頁「.」中文狀態是「.」、組字中先上屏首選）；Shift 開著時字母鍵讀屏標籤不含角標；切數字列後鎖定大寫還在；驗證器對只有 `a11yLabel` 的 `composing` 發警告。
- [x] 第 2 項有 Gboard 實測結論（改了或記了為何不改）；4、5 有前後截圖放 `.scratch/m1/evidence/27b-*.png`。
- [x] `cargo xtask check` 綠燈；/code-review 跑完。

## Comments

- 第 0 項做法：觸控標點改成佈局的 `text` 動作（經 `Session::text`），不另加核心的鍵。`text` 原本就是「沒組字上屏、組字中先上屏首選再接」，只差臨時英文：原本先上屏英文再接文字，改成和實體鍵盤的可見字元一樣接在英文後（Enter 才上屏）。觸控數字、表情、常用語在臨時英文中也跟著接上去，和實體鍵盤打數字一致；推翻票 25b 寫進 brief §4.3 的「`text` 先上屏英文」（實作決定，非使用者決定），progress 偏差記了。`punct` 動作留著：組字中的分隔鍵與自訂佈局要「同實體鍵盤」時用。
- 第 0 項與票面的出入：票寫「符號頁的 ASCII 標點是半形」，但票 27 實測 Gboard 符號頁的 `：；！？（）` 打全形。為了同時滿足「照鍵上的字」與「同 Gboard」，這六個鍵的標籤改成全形字、打全形；其餘 ASCII 標點（含底排 `, .`、`@ # $ & - + /`、數字鍵盤的 `+ - * / % = , .`）半形。中文、英文直通都照鍵上的字（英文直通時字母頁「。」也是「。」）。`databuild` 加測試：每頁每個 `text` 鍵的標籤等於它打的字、沒有 `punct` 的 `tap`。
- 第 0 項 Playwright 先紅後綠：數字列「3」→「。」讀回 `3。`；符號頁「.」在中文狀態讀回半形；臨時英文接 `anr。.`；英文直通「。，.？（」照鍵上的字；組字中「。」先上屏首選（這條本來就綠，留著當回歸）。票 26b 的觸控情境改成「符號頁 3 → 返回／⇧ → 實體鍵盤 `.`」讀回 `3.`，`Key::Modifier` 仍由觸控切頁與 ⇧ 驗到，另加字母頁「。」讀回 `3.4.5。`。
- 第 1 項：`accessibility_label` 改用 `display_hint(state)`；`tests/shift.rs` 先紅（念「A，a」）後綠。
- 第 2 項 Gboard 實測（2026-11-05）：自建 AVD `qim_t27b_gboard`（Pixel 7、android-37 Play 映像，Android 17、Gboard 17.2.2.895242737），Gboard 加「粵語 → 速成」，輸入框用系統設定的搜尋框。開一次的 Shift 打了逗號、句號、空白、刪除、切到 ?123 再返回，都回到字根；鎖定時打逗號、切頁仍鎖定。照 Gboard 改：`Shift::after_key(key)` 換成 `after_other_key()`，任何 Shift 以外的鍵都解除開一次。截圖 `.scratch/m1/evidence/27b-gboard-shift-once.png`。AVD 已刪；使用者的 `Medium_Phone_API_37.0` 開著，沒動。
- 第 3 項：wasm 加 `Layout.keepStateOf(old)`，換數字列佈局時接手 Shift 與目前頁（新佈局有同 id 的頁就留著，同轉向）；Playwright 先紅後綠。
- 第 4 項：等比例的 `top` 實測只移 1 px（間距 7 → 1 px，仍貼著），因為字根置中、角標貼頂，鍵變矮時兩者靠近的量是鍵高差的一半。改成角標上緣跟著鍵中心（`touch.ts` 用畫布座標算，`max(1.5, h/2 − 17.8)`），字級與右距也改用畫布座標。數字列開時間距 7 → 3 px（Gboard 量到 12 → 8，同比例縮小），沒數字列時位置不變。順手修好橫向：原本角標用 cqw（依鍵盤寬），橫向畫布 844 寬時角標比字根還大，這是票 27 起就有的。截圖 `27b-number-row-hints.png`（Gboard／改前／改後）、`27b-landscape-hints.png`。
- 第 5 項：Gboard 截圖量到字根同大時大寫 A 字高 24 px、QIM 13 px，所以大寫字母的字級是 `labelSize` 的 1.85 倍。放在 `qim-layout` 的 `Key::display_label_size(state)`（和 `display_label`、`display_hint` 同處，各平台殼共用），網頁在 `paint` 時跟著 Shift 更新字級。單元測試與 Playwright（字級大於 1.8 倍、解除後回原字級）。截圖 `27b-shift-capitals.png`（Gboard／改前／改後）。
- 第 6 項：逗號鍵讀屏念「逗號，長按切到常用」，`databuild` 測試。
- 第 7 項：`Warning::ComposingA11yLabelWithoutLabel`，只寫 `a11yLabel` 的 `composing` 有警告、有 `label` 或只寫 `tap` 的沒有；plan §2.6 的警告清單補上。
- 2026-11-05 使用者決定（code-review 後問的）：符號頁 `：；！？（）` 維持標全形、打全形（同 Gboard 的輸出），接受英文直通時觸控打不出半形的 `? ! : ; ( )`；其他選項是照票字面標半形打半形，或依中英狀態換標籤（要加格式欄位）。progress 偏差記了。
- 2026-11-05 協調者說明上面那個選擇的理由：使用者的原則是「觸控按什麼鍵就出什麼」，並要對齊 Gboard；票上「符號頁 ASCII 標點是半形，同 Gboard」是協調者寫票時的錯誤假設，以 Gboard 實測為準。所以 `：；！？（）［］` 標全形、打全形（`［］` 在 =\< 頁，票 27 起就是全形），其餘照 Gboard 半形。英文直通時觸控打不出半形 `? ! : ; ( )` 是已知限制：預設觸控佈局沒有中英鍵，只有接實體鍵盤時才進得了英文直通，那時用實體鍵盤打。brief §4.4、plan §2.6 照這個寫。
- 驗證（依序跑，bench 與 e2e 不同時）：`cargo xtask data --verify` 通過；`cargo xtask bench --check` 通過，五個集合都沒有翻盤、答錯與修正同 baseline，固定題 84/84，p95 原生 41.17 µs、WebAssembly 52.54 µs（臨時英文中 `text` 的改動不經基準的路徑，數字不變是預期的）；`cargo xtask check` 通過（e2e 147 條）。
- 亂按（一次性 Playwright 腳本，沒進 repo）：Pixel 7 viewport 5 個種子 × 150 步，在目前頁的鍵裡亂點（含切頁、⇧ 單點與快速兩下、標點、數字），夾雜實體鍵（左右 Shift、`.`、`,`、`3`、字母、Enter、Esc、退格）與數字列開關；每步檢查沒組字時觸控標點打出的字等於鍵上的字、開一次的 Shift 點了別的鍵後解除。沒有頁面錯誤或 console 錯誤，檢查都過。
- code-review（`da901b1` 起，兩軸）已處理：progress 偏差拆成四條（觸控標點與符號頁全形是使用者決定，臨時英文 `text` 與 Shift 解除各一條）；`display_hint` 改用 `shows_capital`，和字級同一個判斷；wasm 的換頁收成 `show_page_or_initial`（轉向與數字列共用）；符號頁全形問了使用者（見上）。
- code-review 沒處理：
  - GLOSSARY〈觸控 Shift〉在 `d7d2f89` 才改，晚了改行為的 `436269d` 一個 commit；已 commit，不改寫歷史。
  - 臨時英文中接進去的 `text` 由退格一次刪一個碼位：「❤️」這類兩個碼位的表情要退兩次，常用語逐字刪（要照字素刪得加 `unicode-segmentation`，和實體鍵盤打的英文不同的只有觸控文字，影響小）。簡體模式下接進去的是轉好的簡體，臨時英文上屏的原文也是簡體；原文只給「回報」用，而回報只算空白選字，不受影響。
  - 橫向角標一起改成畫布座標：和第 4 項同一個原因（cqw 只看鍵盤寬），票外但改動小，記在上面。
  - `symbol_pages_match_gboard_quick…` 逐鍵檢查 `Text(id)` 和新的「標籤等於打的字」測試部分重複：留著，它釘住 Gboard 實測的全形與半形各是哪些鍵。
  - `ComposingA11yLabelWithoutLabel { variant, page, key }` 沿用既有警告的三欄寫法。
  - Gboard 開 Shift 時第一排角標是 1–0，QIM 不顯示：票 27 已記理由（長按不出數字）。
