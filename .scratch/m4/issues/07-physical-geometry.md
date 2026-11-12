# 07: 實體鍵盤的鄰鍵幾何

Status: done
Blocked by: 06
讀: plan §2.1、§2.6；spec〈細節〉〈空間模型〉；brief §4.8；`docs/research/ranking-techniques.md` §5.2；spec〈共同規則〉

## What to build

實體鍵盤也產生同一種 `Weighted`，讓容錯對兩種輸入用同一套機制：

- 網頁送 `KeyboardEvent.code`；`qim-layout` 依實體鍵盤的幾何（各排錯位，含上下排與斜角；libime 只取同排左右）轉成 `Weighted`：按下的鍵最高、鄰鍵較低。這是先驗，不是點擊位置；分布的參數由票定，留給容錯的票調。
- 以實體鍵位為準，不受配列影響（換 Dvorak 等配列時照 `code`）。
- 組合鍵、非字母鍵照舊，不送 `Weighted`；核心照樣只用機率最高的字母，行為不變。

## 驗收

- [x] `qim-layout` 測試：`KeyF` 的鄰鍵含 D、G、R、T、C、V；`key` 值不同但 `code` 相同時結果相同。
- [x] 現有實體鍵盤的 e2e 照綠；Playwright 按一個字母，debug 面板讀到它與鄰鍵的機率。
- [x] plan §2.1 寫明實體鍵盤的 `Weighted`；`cargo xtask check` 綠燈。

## Comments

2026-11-12 實作（`9bed74c` 起）：

- 介面：`qim_layout::PhysicalModel::letter_weights(code)`（`crates/qim-layout/src/physical.rs`）只收 `KeyboardEvent.code`，不是 `KeyA`–`KeyZ` 時回 `None`；綁定加 `Session.pressCode(code)`，網頁 `physicalCode(event, ascii)`（`web/src/keys.ts`）決定哪些鍵送它：中文模式、沒帶 Shift、Ctrl、Meta、Alt、AltGr、CapsLock、別的輸入法沒在組字、`code` 是字母鍵。其餘照 `toKey`，英文直通與臨時英文照配列打 `key`（`shown.ascii` 每次重畫取 `Snapshot.ascii`）。`code` 的寫法照 W3C UI Events KeyboardEvent code Values，沒有另訂格式。
- 幾何：字母區三排，每排第一個鍵左緣在 1.5、1.75、2.25 鍵距（ANSI、ISO 相同），只列字母鍵；按下的鍵固定 0.9，剩下的依鍵中心距離的 Gaussian（σ 0.5 鍵距）分給 1.5 鍵距內的鄰鍵，參數都沒有來源，容錯的票調。F 的結果：f 0.9、d／g 0.023、r 0.020、c／v 0.014、t 0.007。不拿觸控的 `Page` 算（plan §2.6 寫了理由）。spec〈共同規則〉「鄰鍵從 `qim-layout` 的按鍵幾何算、不寫死 QWERTY」：實體鍵盤的幾何就是 `code` 的位置，`ROWS` 是那份幾何本身、在 `qim-layout`，不是佈局的鍵；觸控與自訂佈局照舊從 `Page` 算。
- 驗收「`key` 不同但 `code` 相同時結果相同」：`qim-layout` 的介面只收 `code`，`key` 進不來，所以 Rust 端沒有能紅的測試；能紅的那一半在 `web/test/keys.test.ts`（Dvorak 的實體 F 鍵 `key` 是 u）與 e2e `physical-weighted.spec.ts` 的合成事件（改成照 `key` 會打出「山」而紅）。
- 名詞：「觸控機率」現在也含實體鍵盤，名詞表改成**按鍵機率**、新增**鄰鍵先驗**；`Inspection::touch_weights` → `key_weights`、綁定的 `touchLetters`／`touchProbabilities` → `weightLetters`／`weightProbabilities`、面板 `debug-touch` → `debug-weights`、文字「按鍵機率：」。
- 對非美式配列的影響：先照票上「速成碼照 `code`」只換字母，Dvorak、AZERTY 在中文模式因此打不出部分標點（Dvorak 的 `,` `.` 在實體 W、E 鍵上）、一個字母兩個鍵。協調者轉達使用者決定（2026-11-12）：中文模式整個當美式鍵盤，字母與標點都照實體鍵位，英文直通照配列。實作：`web/src/keys.ts` 的 `usCharacter` 照 `code` 與 Shift 給美式字元（CapsLock 只管字母；美式沒有的鍵、具名的鍵、數字鍵台、組合鍵、AltGr 照 `key`），`usKeyEvent` 在中文模式換掉給 `toKey` 的 `key`；核心沒吃下的換過字元的鍵（例如沒組字時 AZERTY 的實體 1 鍵 `&`），網頁攔下預設動作、自己插入美式字元，宿主的字與核心的前文一致。臨時英文也照配列：它和英文直通一樣是英文（`Snapshot.ascii`），使用者沒提，照「英文照配列」歸同一類。`key` 已是美式鍵盤上那個字元時不換。第一版照 Shift 一律換，`composition-keys`、`digit-separators` 三條既有 e2e 紅了：Playwright 的 `type("@")` 送 `key` @、`code` Digit2 卻沒帶 `shiftKey`（真的美式鍵盤會帶），是測試不寫實，改成 `press("Shift+Digit2")` 這類；中間一版在產品碼放過「沒帶 Shift 而 `key` 是上排字元也不換」的例外，code-review 指出是測試工具的限制滲進產品碼，拿掉。字母的大小寫照 `key` 本身（code-review：macOS 的 Shift 加 CapsLock 是大寫、Windows 是小寫，照系統給的），`key` 不是字母時（Dvorak 的實體 Q 鍵 `'`）照 Shift 與 CapsLock；dead key（`Dead`，AZERTY 的實體 [ 鍵）不換、交給系統組字，換了而攔下時系統的 dead key 狀態不一定跟著取消，Playwright 的合成事件驗不到，平台殼（M7）實機再看。測試：`web/test/keys.test.ts` 七條（Dvorak、AZERTY、Shift 與 CapsLock、不換的鍵、已是美式的不換、字母大小寫照 `key`、dead key）；e2e `physical-weighted.spec.ts` 三條用合成 keydown（Dvorak 實體 W、Q、E、Z 與逗號、句號、分號鍵；AZERTY 實體 Q、W、M、逗號、1、Shift+1；英文直通不攔下），另一條臨時英文照配列（Dvorak 的實體 W 鍵接上逗號）。變異驗證：拿掉換字時 Dvorak、AZERTY 兩條紅，英文直通也換時英文那條紅。亂按：一次性腳本混著美式按鍵與 Dvorak、AZERTY 的合成 keydown（含 Shift、單按 Shift 切中英、ISO 的 `IntlBackslash`）各 500 鍵 × 2 個種子，中文模式的字母鍵讀回面板 74 次，第一名都是實體鍵位的字母，沒有頁面錯誤。
- `bench --check`：各集合答錯、修正數與 baseline 相同、沒有翻盤，固定題 84/84；每鍵延遲原生空白 p95 74.8 µs／第 20 碼起 81.5 µs／選開頭一段 1.32 ms，學過 186.8 µs／393.6 µs／2.05 ms，都在上限內。核心只改名、lattice 沒動，不必跑 `model-ladder`。
- 驗證：`cargo xtask check` 綠（e2e 241 passed、2 skipped；改成整個美式鍵盤後 245 passed、2 skipped）。截圖 `.scratch/m4/evidence/07-physical-weights.png`（面板「按鍵機率：f 90.0% d 2.3% g 2.3% r 2.0% c 1.4% v 1.4% t 0.7%」）。亂按一輪：一次性 Playwright 腳本按 600 鍵 × 3 個種子（字母、Shift 加字母、單按 Shift、數字、標點、方向鍵、Tab、Ctrl+A、CapsLock、Dvorak 的合成事件），173 次讀回面板，第一名都是按下的字母、和為 1，沒有頁面錯誤。第一輪紅過一次是腳本的問題：沒有組字時 Tab 照瀏覽器預設把焦點移走，之後的鍵沒進輸入框；改成每鍵前放回焦點。
- code-review 處理了：CapsLock 改用 `getModifierState`（Dvorak 上 `key` 看不出大小寫）、排除 AltGr、`physicalLetter` 改名 `physicalCode`（回的是 `code`）、拿掉和半徑重複的 `limit`、`ROWS` 改存字母不再從字串位元組拆、plan 裡沒查證的「macOS 輸入法把字母鍵固定成美式鍵盤」刪掉。沒處理：兩個 e2e 讀面板機率的 helper 重複（`touch-weighted` 與 `physical-weighted` 各 10 行，等第三個用到再抽）；`total > 0` 不成立的分支照現在的幾何到不了，留著防除以 0。
- 美式鍵盤這輪的 code-review（`d7d7af2` 起）處理了：上面的大小寫、dead key、拿掉遷就 Playwright 的例外、`main.ts` 多餘的條件、兩個函式重複的修飾鍵判斷收成 `plain`、`us`／`want` 這類太短的名字、臨時英文補 e2e 並在 brief、plan、progress 標明是推論。沒處理：Chrome 在 macOS 的 ISO 鍵盤上 `Backquote` 與 `IntlBackslash` 對調（審查自己說信心低，沒有實機可驗，平台殼照系統的實體鍵位）；名詞表收「美式鍵盤」「配列」（一般用語，不是本專案的名詞）；`[string, string]` 的下標取值（兩處，改成具名欄位沒有多得什麼）；e2e 舊的 Dvorak F 合成事件沒改用新的 `dispatch` helper（同檔，留著不影響）。
- 2026-11-12 合併 main（票 11 萬用鍵、14 英文使用者詞、02 網頁修補）：`main.ts` 的送鍵改用票 02 的 `withSnapshot`、`pressCode` 留在裡面；plan §3.1 的 debug 面板取 main 的寫法、換回按鍵機率；票 11 的 `wildcard.spec.ts` 讀 `debug-weights`。萬用鍵 `z` 照實體鍵位：實體 Z 鍵送 `code` KeyZ，`PhysicalModel` 最高的是 z，核心照萬用鍵處理；補 e2e（Dvorak 實體 Z 鍵 `;`、AZERTY 實體 Z 鍵 `w`，打 hzo 都是「竹＊人」、首選「的人」）。合併後 `cargo xtask data` 重建成資料包格式 v8。
