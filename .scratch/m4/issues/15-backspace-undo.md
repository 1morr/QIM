# 15: 上屏後退格撤銷學習

Status: done
Blocked by: 13, 14
讀: brief §4.5、§12.2（librime）；plan §2.4；spec〈細節〉（退格撤銷）；`docs/research/learning-forgetting-sync.md`；spec〈共同規則〉

## What to build

選錯上屏後馬上退格，這次的學習不該留下（照 librime `memory.cc` 的 `DiscardSession`，中英共用）：

- 中文：撤銷這次上屏的整串上屏次數、衰減值、自動造詞、個人 bigram。
- 英文：撤銷剛打完的那個英文使用者詞。英文直通時「打完」的界線（空白、標點）與退格的語意由票定並測。
- 「緊接著」照 librime：上屏後的第一個鍵是退格；中間夾了其他鍵就不撤銷。退格照常交給宿主刪字。

## 驗收

- [x] 核心測試：上屏後緊接退格，學習資料回到上屏前（匯出比對）；中間夾別的鍵就不撤銷；英文的情況照票定的語意測。
- [x] Playwright：同一串碼選第 2 個候選兩次，第二次上屏後緊接退格，再打同一串碼首選不變。
- [x] plan §2.4 寫明；`cargo xtask check` 綠燈。

## Comments

- 設計（plan §2.4〈退格撤銷〉）：撤銷紀錄 `Undo` 在 `UserModel`（`crates/qim-core/src/user/undo.rs`），記每個被改到的條目第一次改之前的值與各 tick，上限淘汰刪掉的也記；`Session` 以「動作」為單位收起與撤銷（`begin_action` / `end_action`）。沒用整份複本：學過的資料約 2 萬列，每次上屏複製一份會吃掉「選開頭一段」的延遲預算。
- 票定的語意：
  - 「緊接著」：上一個動作之後第一個交還宿主的鍵是退格。修飾鍵、只收起聯想的退格不算夾在中間（librime-predict 吃下的退格不到 `OnUnhandledKey`）。所以聯想開著時，第一下退格只收起聯想，第二下刪字並撤銷。Esc、單按 Shift、字母都算夾在中間。
  - 英文直通：「打完」的界線照票 14（空白、標點、Enter，或接著上屏的中文），緊接著的退格撤銷那個英文使用者詞，正在打的串回到那串的結尾，改字後再打完學的是改過的。方向鍵、Tab 這類不送字的鍵讓英文串打完時不能撤銷（code-review Spec 的發現，已修並補測試）。
  - 範圍外但必要：撤銷紀錄記著是哪個 `Session` 寫的（一開始沒做，性質測試 `one_scheme_neither_writes_nor_reads_the_other_schemes_learning` 抓到：別的 Session 的退格撤銷了這裡的上屏）；無痕、學習關掉時不撤銷（性質測試 `incognito_leaves_the_user_model_unchanged` 抓到）；手動加詞、常用語、刪英文使用者詞、匯入丟掉紀錄。兩個 proptest 種子留在 `crates/qim-core/tests/properties.proptest-regressions`。
- 3 秒時限（協調者 2026-11-12 決定）：一開始因為核心不讀時間就拿掉了 librime 的 3 秒，協調者改成照 librime 保留、由殼計時。核心加 `Session::expire_undo`（wasm `expireUndo`），網頁在 `saveLearning` 從頭計 3 秒（`web/src/undo-window.ts`，單元測試用 `mock.timers`），點輸入框（`pointerdown`）也讓紀錄失效（code-review Spec 的發現：滑鼠移游標後的退格不該撤銷）。Playwright 用 `page.clock.runFor`：2.9 秒撤銷、3 秒不撤銷。
- librime 的行為現查（`bb56876`），補進 brief §12.2。
- `bench --check`（改動後）：12 個集合的答錯、修正都等於 baseline（yue-wiki 941／3239、zh-wiki 979／3493、common-voice-yue 892／3249、hkcancor 572／2148，前文各集合同 baseline），沒有翻盤，固定題 84/84。基準不按退格，所以預期不變。每鍵 p95（原生）：空白 126 µs／138 µs／2.20 ms，學過 196 µs／286 µs／2.36 ms，都在上限內。比票 14 記的高，但量的時候 load average 約 25（別的 worktree 在建置），學習完全沒經過的「空白・全部」也高了約 44%，判斷是負載雜訊；沒超過上限，所以沒有在低負載時重跑。
- `bench --learning`：學習關／開的修正 yue-wiki 3239／3216、zh-wiki 3493／3578、common-voice-yue 3249／3108、hkcancor 2148／1719、yue-wiki·前文 1738／1715、zh-wiki·前文 1729／1764、common-voice-yue·前文 594／550、hkcancor·前文 1512／1179，和票 14 記的逐項相同。
- 網頁亂按一輪（600 個鍵，組字中夾退格、Esc、數字、Enter、Shift、方向鍵、標點，中途切換聯想開關）沒有頁面錯誤：`.scratch/m4/evidence/15-backspace-undo-random.png`；撤銷後首選不變的截圖：`15-backspace-undo-top.png`。
- code-review（從 `c39b5b4` 起）：
  - Spec：方向鍵、Tab 讓英文串打完後的退格會撤銷，以及滑鼠移游標後仍會撤銷，都已修並補測試；plan 的「沒有 3 秒」已照協調者的決定改寫；單按 Shift 算夾在中間，已在 plan 註明是刻意從嚴。
  - Standards：plan §0 對照表補了退格撤銷 / 動作；brief 的「撤銷學習」改成「退格撤銷」（名詞表的 Avoid）；名詞表加「動作」；動作的開頭與結尾收成 `begin_action` / `end_action`、`learning_scheme`、`discard_undo`，`save_row` / `save_bigram` / `save_english` 不再把同一個物件傳兩次，裸 tuple 換成 `LastSent`、`Row`；`UserModel` 的 `PartialEq` 改手寫、跳過撤銷紀錄（不再讓 `Undo` 永遠相等）；`keep_undo` 不會發生的分支改成 `debug_assert!`。
  - 沒改：`UndoOwner` 的 `Default` 每次拿新號碼。審查建議改成 `UndoOwner::next()`，但 `Session` 用 `#[derive(Default)]`，欄位型別要有 `Default`；改成手寫 `Session::default` 要列出 20 多個欄位，代價比收穫大，所以在型別的文件註明了這個行為。
- 驗證：`cargo xtask check` 通過（`cargo test --workspace` 1084 passed；web 單元 56 passed；e2e 251 passed、2 skipped，含 `backspace-undo.spec.ts` 7 個）。
