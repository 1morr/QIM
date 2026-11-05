# 27: 觸控佈局對齊 Gboard 速成

Status: done
Blocked by: None (can start immediately)
讀: brief §4.4、§1.3（Gboard 那列）；plan §2.6、§3.1、§5.3；`docs/research/blackbox-baseline.md`（Gboard 截圖 `.scratch/m1/evidence/09-*.png`）；票 18、19、20 的 Comments；spec〈共同規則〉

## What to build

使用者 2026-11-05 試用後指出：預設觸控佈局看起來不像 Gboard。對照 `.scratch/m1/evidence/09-android-t13.png`（Gboard 速成）與 `20-long-press-a.png`（QIM），至少要改下列各項；其餘差異由實際比對找出，一併改或在 Comments 說明不改的理由。

- 第三排左邊加 Shift 鍵（Gboard 有 ⇧，QIM 是空位）；它在觸控上的行為照 Gboard 速成實測（大寫、切英文或其他），寫進 brief §4.4。
- `z`、`x` 鍵標籤照 Gboard（「重」「難」等），其他鍵的主標籤、角標字母與位置逐一對照。
- 配色、圓角、功能鍵（⇧、⌫、?123、↵、句號鍵）底色照 Gboard 的淺色與深色。
- 底排：空白鍵顯示「速成」；逗號鍵與句號鍵的長按、Gboard 逗號鍵上的表情入口（QIM 沒有表情頁就說明怎麼處理）。
- 符號頁（?123 與更多符號頁）的鍵位與內容照 Gboard。
- 長按不再出數字：長按字母鍵只出英文字母（推翻 brief §4.4「第一排另含數字」，使用者決定）。改成設定裡的「數字列」開關，照 Gboard 的 Number row，打開時字母頁上方多一排數字；預設關（與 Gboard 預設相同）。
- 比對方式：用 `orca emulator-android` 開一個本票自己建的 AVD（映像內建 Gboard、不登入帳號，照票 09 的做法），截 Gboard 速成的字母頁、符號頁、數字列開與關、深色，和 QIM 網頁同尺寸截圖並排存 `.scratch/m1/evidence/27-*.png`。不要動使用者的 `Medium_Phone_API_37.0`；做完刪掉自己建的 AVD。
- 同輪改 brief §4.4、§7（數字列開關）、plan §2.6；progress〈偏差與決定〉記一行（使用者決定）。

## 驗收

- [x] 並排截圖存在 `.scratch/m1/evidence/27-*.png`：字母頁、符號頁、數字列開與關、深色各一組。
- [x] 上面列的每一項已改，或在 Comments 寫明不改的理由；實際比對另外找到的差異也列出。
- [x] Playwright：長按 `q` 只有字母、沒有數字；打開數字列後點「1」讀回「1」；點 Shift 鍵的行為讀回符合 brief §4.4。
- [x] 自己建的 AVD 已刪除；`cargo xtask check` 綠燈。

## Comments

- 2026-11-05 比對方式：自建 AVD `qim_t27_gboard`（Pixel 7、android-37 Play 映像、不登入），Gboard 設定加「粵語 → 速成」（不用帳號）；輸入框用系統設定的搜尋框，uiautomator 讀回。符號鍵逐一實打讀回碼位，決定全半形。AVD 已刪；使用者的 `Medium_Phone_API_37.0` 原本就開著，沒動。並排截圖：`.scratch/m1/evidence/27-letters.png`（字母頁、點一下 Shift）、`27-symbols.png`（?123、=\<、12/34）、`27-number-row.png`（數字列關、開、開時的 ?123）、`27-dark.png`（深色字母頁、?123、數字列開）。
- 已照 Gboard 改：
  - 第三排左邊加 ⇧（行為見 brief §4.4）。
  - `z` 標「重」（`x` 本來就是方案的「難」），其他字根與角標逐鍵對過，相同。
  - 配色：淺色鍵盤底 `#ededf6`、白鍵 `#ffffff`／字 `#30323a`、角標 `#5d5f68`、功能鍵 `#dce2f9`／字 `#4b5164`；深色 `#17191f`、`#292c34`／`#e4e5f0`、`#a9aab5`、`#353b4d`／`#b8bfd5`。Gboard 是 Material You，顏色跟桌布，這是模擬器預設桌布的值。
  - 功能鍵：⇧ ⌫ ?123 ↵、字母頁的逗號與句號鍵用功能鍵色；?123、返回、Enter 是膠囊形。
  - 尺寸照 411 dp 寬的實測等比例縮成 390；字級照實測。
  - 空白鍵顯示「速成」，讀屏唸「空白」。
  - 句號鍵長按 16 個，打出的字照實測（`；：！？` 全形）。
  - ?123、=\<、12/34 三頁的鍵位與內容照 Gboard；`：；！？（）［］` 照實測打全形，底排逗號句號與其他符號半形。
  - 數字列：另一份佈局檔，數字列比其他排矮，整個鍵盤只高出約 0.57 排，其他頁四排拉高填滿（同 Gboard，符號頁不變）。長按數字出上標，`1` 另有分數。
  - 放大預覽改成圓形氣泡；長按清單選中色 `#4c5e8b`。
- 實際比對另外找到、沒照 Gboard 的（理由）：
  - 組字中 ⇧ 不變成「'詞」分隔鍵：QIM 核心沒有分隔，要做得在連打的 lattice 加強制切段，屬核心功能、超出本票。組字中仍是 Shift（先上屏組字再接大寫，同實體鍵盤）。使用者 2026-11-05 決定做成分隔鍵，協調者另開補票；本票維持現狀。
  - Shift 開一次時 Gboard 第一排角標變 1–0，QIM 不顯示角標：長按不出數字，顯示數字會誤導。
  - 逗號鍵的 ☺ 在右上角：角標位置由殼的 CSS 定，格式沒有逐鍵的角標位置；Gboard 長按逗號是剪貼簿、表情、設定，QIM 沒有表情頁，改成長按進「常用」頁（表情加常用語）。符號頁原本的「常用」鍵拿掉了，因為 Gboard 符號頁沒有那個位置。
  - 句號長按的預設「……」排第一格（Gboard 在第 2 排第 7 格）：格式規定長按清單第一個是預設，清單排序跟著它。
  - 數字鍵盤左欄 Gboard 是一條可捲的鍵，QIM 拆成四個鍵；格式沒有可捲動的鍵。
  - Gboard 按 ?123 會回到上次停留的符號子頁，QIM 一律到 ?123：`switchPage` 指固定的頁，要記憶得加格式欄位，影響小，不做。
  - 長按空白鍵 Gboard 出換鍵盤清單：網頁沒有別的鍵盤；平台殼開工時再定。
  - 數字列只有 `1` 有分數清單（實測過的）；其他數字只放上標，沒實測完整清單。
  - 工具列（表情、GIF、剪貼簿、設定、手寫、麥克風）與 Enter 依欄位換圖示（搜尋框是放大鏡）：網頁沒有這些功能與欄位類型，不做。
  - 橫向沒對 Gboard 實測，只沿用新的頁面與配色。
- 2026-11-05 格式：加動作 `"shift"`（Schema、`Action::Shift`），只加不改、格式還沒發佈，不升 `qim-layout/2`。狀態 `Shift`（關、開一次、鎖定）與 `KeyState` 在 `qim-layout`，`display_label`、`tap_action`、`shows_preview` 改收 `KeyState`（原本收 `composing: bool`），另加 `display_hint`。綁定把狀態記在 `WebLayout`，同目前頁。Schema 拿掉 `shift` 時 `tests/shift.rs` 紅（變異驗證過）。快速點兩下的門檻 300 ms 在殼（Android 的雙擊門檻）。
- 2026-11-05 「常用」頁的 id 由 `phrases` 改成「常用」：長按清單的選項標籤是 `Action::spoken`（「切到 <頁 id>」），格式沒有選項標籤；M5 使用者自訂的頁 id 本來就可能是中文。
- 2026-11-05 Playwright：長按 `q` 只有 `q`；打開數字列點「1」讀回 `1`；Shift 點一下讀回 `A` 並自動解除、快速兩下鎖定讀回 `ASD`、隔 600 ms 的第二下是關掉、組字中 Shift 再點字母讀回 `門口A`、Shift 時長按出小寫並解除。舊測試照新頁面改（「返回」、`？`、長按逗號進常用頁、新顏色）。
- 2026-11-05 亂按：Pixel 7 viewport，4 個種子 × 150 步（點鍵與縫、長按滑動、Shift 單點與快速兩下、兩指交疊、切數字列、深色、轉向、點候選）：沒有 page error 或 console 錯誤，放開後長按清單都收起；常用頁要長按逗號才進得去，亂按沒走到（e2e 有測）。
- 2026-11-05 驗證：`cargo xtask check` 通過（e2e `104 passed`）。第一次跑時同時在跑 `bench`，e2e 有 3 個失敗（含與本票無關的剪貼簿測試），bench 跑完單獨重跑全過；數字列測試順手改成量高度前先等鍵盤出現。`cargo xtask bench --check` exit 0：四個公開集合沒有翻盤，固定題 25/25；沒動核心與資料包，結果同 baseline 是預期的。
- 2026-11-05 code-review（`ab27a1c` 起）已處理：progress〈偏差與決定〉補一行；名詞表加「觸控 Shift」「數字列」（和「英文直通」分開）；`LayoutKey.shift` 改名 `isShift`；`SHIFT_GLYPHS` 註解改正；數字鍵盤不再用字串比對 id 改顏色（加 `Cell::plain`）；兩個符號頁的重複排法收成 `symbol_page`；上面〈沒照 Gboard 的〉逐項補理由。
- 2026-11-05 code-review 沒處理：
  - Shift 狀態放在 `WebLayout`、由 `WebSession::tap` 改：和目前頁一樣，動作在 session 做、狀態在佈局；M2 的 `qim-ffi` 要用時再看要不要收成一個型別。
  - Shift 狀態過綁定變字串（`off`／`once`／`locked`）：wasm-bindgen 的列舉給 JS 是數字，字串在 Playwright 斷言 `data-shift` 時較好讀；網頁只用來選圖示。
  - `letter`、逗號、`to_numbers` 直接改欄位、其他用鏈式方法：只有一兩處，不再加 builder 方法。
  - `49e325c` 的 subject 沒提符號改全形、又混了 progress：已 commit，不改寫歷史，這裡記下。
- 2026-11-05 使用者決定（經協調者）：數字列只對內建佈局有效可以接受，自訂佈局怎麼處理留到 M5 編輯器，已寫進 plan §7.6。
- 2026-11-05 合併 main（票 25，`37b1130`）：三個檔衝突，兩邊都留。
  - `crates/qim-wasm/src/lib.rs`：觸控的中英鍵（`toggleAscii`，預設佈局沒有）照票 25 當右 Shift；觸控 ⇧（`shift`）仍是 `Key::Other`。
  - `docs/design-brief.md` §7：開關清單取 main（多了「組字中單按 Shift」），數字列那條留著。
  - `web/src/main.ts`：Shift 設定與數字列讀佈局都留。
  - progress.md 的 union 合併抄出重複段，手動併回。
  - 觸控 ⇧ 與核心左右 Shift 的關係寫進 brief §4.4、plan §2.1、§2.6：觸控 ⇧ 是大寫鍵，不送 `ToggleAscii`，不切中英，不受 `Options.shift_switch` 影響。
- 2026-11-05 合併後的檢查（依序跑，沒有和 bench 同時跑 e2e）：
  - `cargo xtask data` 通過。sha256：quick `ae3bc207ce7a`、quick-lexicon `20853183b2cc`、zh-hant-bigram `68b18859fd32`、cangjie-hint `9e755488f431`、opencc-hk2s `1c5ab2aa21e6`、scowl-en-us `1afd47942b69`。
  - `cargo xtask bench --check` exit 0：四個公開集合沒有翻盤，固定題 25/25。每鍵 p95：原生 49.00 µs、WebAssembly 78.46 µs，只報不擋。
  - `cargo xtask check` 通過（e2e `108 passed`，修下面的問題之前）。
- 2026-11-05 合併後觸控亂按（5 個種子 × 150 步，另外夾雜實體的左右 Shift、字母、Enter、Esc、退格，並切換「組字中單按 Shift」設定）找到一個問題：頁面捲到「觸控鍵盤」按鈕下緣貼著鍵盤上緣時，點鍵盤上緣的一下，`pointerdown` 的目標變成那個按鈕，鍵盤就被關了。原因是 Chrome 的觸控校正會把點擊吸到附近「可點」的元素，而鍵盤沒有 click 監聽。先寫成會紅的 Playwright 測試（「鍵盤上緣正上方有按鈕時…」），再在鍵盤容器加 click 監聽並吃掉。修好後 5 個種子都沒有錯誤。這是票 20 起就有的問題，橫向加數字列時鍵盤較高才碰到。
- 2026-11-05 協調者要求（合併票 25b 之後）：觸控的標點改送核心的標點鍵。
  - 格式加 `{ "punct": "," }`：只收 ASCII 可見標點，綁定轉成 `Key::Punct`，只加不改。`Snapshot.letter` 改名 `character`，標點也帶，核心沒吃下時網頁照打。
  - 預設佈局：逗號、句號、符號頁的 ASCII 標點送 `punct`，標籤照 Gboard 寫半形，中文狀態由核心轉全形。核心會轉成別的符號的 `_ ^ \ < > [ ]` 照鍵上的字直接輸出：`_` 的標籤打出 `——` 會讓人意外，Gboard 這些也打半形。全形字、非 ASCII 符號、數字、表情、常用語、句號長按清單（明確挑的字形）維持 `text`。
  - 和 Gboard 的差別：符號頁底排的 `, .`，Gboard 打半形，QIM 中文狀態打全形（同實體鍵盤）。數字後的半形等票 26。
  - 先寫的 Playwright：「日」「。」→「日。」、組字中「。」先上屏首選（這兩條在舊的 `text` 路徑就過，留著當回歸），以及英文直通時觸控標點打半形（`.,?(`）、臨時英文中觸控「。」接在英文後（`anr.`）。後兩條先紅再修。`tests/punct.rs` 的 Schema 檢查放寬成任意字串時會紅（變異驗證過）。
- 2026-11-05 合併 main（票 25b，`bab09bd`）：只有 `docs/plan.md` §2.1 的 `ToggleAscii` 那段衝突，取 main 的「照舊版 librime」加本票的觸控 ⇧ 說明。
- 2026-11-05 合併 25b、改標點鍵之後的檢查（依序跑，bench 與 e2e 不同時）：
  - `cargo xtask data` 通過，sha256 同上次合併（quick `ae3bc207ce7a`、quick-lexicon `20853183b2cc`、zh-hant-bigram `68b18859fd32`、cangjie-hint `9e755488f431`、opencc-hk2s `1c5ab2aa21e6`、scowl-en-us `1afd47942b69`）。
  - `bench --check` exit 0：四個公開集合沒有翻盤，固定題 25/25。p95 原生 42.21 µs、WebAssembly 53.83 µs。
  - `cargo xtask check` 通過（e2e `114 passed`）。
  - 觸控亂按 5 個種子 × 150 步：夾雜實體的左右 Shift、字母、`,` `.` `-`、數字、Enter、Esc、退格，並切換 Shift 設定，沒有錯誤。
