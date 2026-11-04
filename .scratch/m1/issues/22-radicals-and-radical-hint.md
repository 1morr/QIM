# 22: 字根顯示與字根提示

Status: done
Blocked by: 07
讀: plan §2.5（`[radicals]`、`radical_hint`）、§4.1 步驟 5；brief §4.2（預設五代、可改三代）、§4.3（組字中顯示字根、快捷鍵查字根提示）；ADR 0003；spec〈共同規則〉

## What to build

組字中顯示已打的字根（打 `anr` 顯示「日弓口」）；用快捷鍵查目前候選的倉頡碼字根提示，預設五代、可改三代。

- 組字的字根來自方案檔 `[radicals]`，經 `Snapshot.composition` 給殼。
- 字根提示資料包（plan §4.1 步驟 5）：字 → 倉頡三代 / 五代碼，只給提示用，不進 lattice（ADR 0003）；`LICENSE` / `NOTICE` 同 libcangjie。
- 查詢介面放核心或資料包讀取層（守 plan §1.3），殼只顯示。
- 快捷鍵與觸控上的叫出方式在本票定，參考黑箱基線與 macOS 速成（已做完的話），寫回 brief §4.3。

## 驗收

- [x] Playwright：打 `anr`，讀回組字顯示「日弓口」。
- [x] Playwright：按快捷鍵，讀回首選的字根提示是五代碼；切三代後讀回三代碼。
- [x] `--verify` 抽查幾個字的三代 / 五代提示；建兩次 sha256 相同。
- [x] brief §4.3 寫明快捷鍵。

## Comments

- 2026-11-04 叫出方式（brief §4.3、§4.4、§12.4）：組字中按 Tab 在目前頁每個候選旁逐字列出「字根(碼)」，再按一次收起，組字結束（上屏、Esc、刪光）也收起；觸控長按候選（500 ms）同 Tab。參考 fcitx5 table：組字列顯示字根（Prompt）、候選旁以「~ 」接提示，但它是設定開關、沒有快捷鍵；黑箱基線沒測競品的字根提示，macOS 速成的殼在 M6 才做。選 Tab：組字中它本來就吃下不動作，不必加修飾鍵，也不佔分段用的 ← / →；沒有組字時交還宿主（焦點照常移走）。五代 / 三代先用網頁上的選單切換，設定的保存跟著平台殼的設定做。
- 2026-11-04 組字的字根：`Composition.radicals` 是還沒選定的輸入碼的字根，選定的分段由殼顯示字詞（「門口大月人」），plan §2.1 的註解跟著改。由 `Session::handle` 用 `Scheme::radicals_of` 補上，方案沒有字根的鍵顯示原字母。票 21 加 `select`、`focus_segment` 等公開方法時要一樣補上 radicals（或把它移進 `snapshot`），現在只有 `handle` 是公開的。既有 e2e 對組字的期待由輸入碼改成字根。
- 2026-11-04 資料包：`cangjie-hint`（魔數 `QIMH`），每個漢字合併 libcangjie 各列的三代、五代碼，各代依列序去重，1.3 MB；`LICENSE`、`NOTICE` 只有 libcangjie。核心的 `RadicalHints` 不放進 `Engine`，倉頡碼結構上進不了 lattice（ADR 0003）；殼用 `RadicalHints::for_scheme` 取出方案指名的那份，`hint(方案, 文字, 版本)` 逐字回碼與字根。方案的 `Packs` 多了欄位，`FORMAT_VERSION` 4 → 5。`--verify` 加兩項：碼表每個字（含擴展區）都有倉頡碼（74,663 字）、抽查「丟」三代 `hgi` / 五代 `mgi,xxmgi` 與「問」兩代 `anr`；四份資料包建兩次 sha256 相同（字根提示 `9e755488f431`）。
- 2026-11-04 e2e 用「俾」（`oj` 首選，五代 `ohwj`、三代 `ohhj`）驗版本切換。整個候選都沒有倉頡碼（之後的英文候選）時不顯示提示；詞裡個別字沒有碼時那個字顯示「—」。截圖：`.scratch/m1/evidence/22-composition-anr.png`、`22-hint-cangjie5.png`、`22-hint-cangjie3.png`。
- 2026-11-04 亂按：組字中夾 Tab、翻頁、Backspace、數字選字、← →、`z`、Enter、標點、Esc，切三代後打 `hidpovrn`：翻頁與刪碼後提示照開、上屏或 Esc 後收起，沒有組字時 Tab 讓焦點離開；沒找到新問題。
- 2026-11-04 bench 前後對照（前：baseline 與票 11 的記錄；後：本票，資料包 quick `0fd82b8adae0`、quick-lexicon `f17de0b58ac6`、zh-hant-bigram `baf9cc30a7b6`）：排序沒動，各集合答錯與修正次數都等於 baseline（yue-wiki 1139 / 4607、zh-wiki 1121 / 4495、common-voice-yue 1213 / 6236、hkcancor 811 / 4518、personal 1106 / 1500），沒有翻盤、固定題 11/11，`--check` exit 0。每鍵 p95（µs，前 → 後）：原生 34.12 → 34.96、WebAssembly 42.46 → 45.71（第一次量到 61.50，重跑回到 45.71，當成雜訊）；組字多算一次字根，量級不變。
- 2026-11-04 code-review（`25834f0` 起）已處理：brief §12.4 插入 fcitx5 那條時把上一條的出處切到新條目，修回；碼換字根與版本字串的解析由綁定移進核心（`RadicalHints::hint` 收方案、`CangjieVersion: FromStr`），綁定記住建立時的方案，網頁不必每次傳 `Session`；`Hint` 改名 `RadicalHint`（名詞表的「字根提示」）；pipeline 的字根提示 id 只取一次、存在 `QuickBuild`；英文這類沒有碼的候選不顯示一串「—」；補行動裝置 viewport 的觸控長按 e2e（CDP 觸控事件），含「滑動不叫出」（變異驗證：拿掉 `pointercancel` 的取消會紅）。
- 2026-11-04 code-review 沒處理：
  - `PackError` 三個同形的 `*Mismatch`：沿用既有的寫法，等再多一種資料包時一起收成一個帶種類的變體。
  - `verify::run` 的 sha256 用位置 tuple（`.0`–`.3`）：只在這個函式裡用，四份還看得懂，再加資料包時改成具名結構。
  - `RadicalHints::iter` 只有建置端與測試用：和 `CodeTable::iter`、`Lexicon::iter` 同樣是給建置與驗證讀資料包的，保留。
  - 組字中與否、組字結束收起提示、Tab 不轉給核心、提示的排版在網頁的 `main.ts`：這些是殼的按鍵對照與顯示（plan §1.3 只禁止殼重寫排序、查碼、點擊判定），各平台殼照 brief §4.3、§4.4 做；平台殼開工時若重複太多再考慮放進 `Snapshot`。
- 2026-11-04 合併 main（票 13，`0736054`）：`session.rs`（`set_options` 取 main，`handle` 補字根取本票，`handle_key` 改用 `user`）、`verify.rs` 與 `qim-wasm` 的 `use`、`web/src/main.ts`（無痕開關與字根提示兩段都留）有衝突；票 13 新的 `tests/learning.rs` 補 `..Packs::default()`。合併後 `cargo xtask data --verify` 通過，四份資料包 sha256 不變（quick `0fd82b8adae0`、字根提示 `9e755488f431`）；`cargo xtask bench --check` exit 0，各集合答錯、修正與 baseline 相同、沒有翻盤、固定題 11/11，每鍵 p95 原生 48.25 µs、WebAssembly 60.38 µs（含票 13 的 `personal` 加分）；`cargo xtask check` 通過（e2e `37 passed`）。
