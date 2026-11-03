# 08: 補齊 M0 按鍵與翻頁

Status: done
Blocked by: 07
讀: plan §2.1、§5.1；brief §4.3；spec〈範圍〉〈共同規則〉

## What to build

使用者在網頁打 `ar` 能翻頁找到「問」並用數字鍵上屏；打錯時用 Backspace 刪一碼、Esc 清掉整串。M0 的單字查詢在網頁上可以實際使用。

- `Session` 加 `Backspace`、`Escape`、`Digit`、`PageUp`、`PageDown`：
  - Backspace 刪最後一碼；Escape 清除輸入碼。
  - 數字 1–9 上屏目前頁對應的候選；沒有該位置時不動作。
  - 翻頁依 `has_more` 與 `page`，到頭或到尾時不動作。
  - 沒有組字時，這些鍵都不攔截，交給宿主（Backspace 刪輸入框的字、數字照打）。
- 輸入碼超過方案的 `max_code_length`（速成是 2）時候選為空、空白不上屏；連打是 M1。
- 網頁：`-` / `=` 與 PageUp / PageDown 轉成翻頁（spec〈共同規則〉）。
- 性質測試（`proptest`）：任何 `Snapshot` 的每個候選，碼表裡都有一個碼等於目前輸入碼。

## 驗收

- [x] Playwright：打 `ar`，讀回候選含「問」（必要時翻頁），用數字鍵選它，讀回輸入框為「問」。
- [x] Playwright：打 `a` 再按 `2`，讀回輸入框為「日」（`a` 的首選是「曰」，見票 07 Comments）。
- [x] Playwright：Backspace 刪一碼後候選更新；Esc 清空組字；沒有組字時 Backspace 刪輸入框的字。
- [x] 截圖存證：`.scratch/m0/evidence/08-page-1.png`、`08-page-2.png`、`08-ar-found.png`、`08-ar-committed.png`、`08-backspace.png`。
- [x] `qim-core` 單元測試涵蓋上述每個鍵，含沒有組字時不攔截、翻頁到頭與到尾、超過 2 碼。
- [x] 性質測試綠燈。

## Comments

- 2026-11-03 `ar` 第一頁是「謈誾閜晑昫闆闊閭暱」，「問」在第二頁第 5 位（Big5 序，沒有字頻）：e2e 按 `=` 翻到含「問」的頁再按它的數字鍵，不寫死位置，M1 加字頻後不必改測試。
- 2026-11-03 翻頁後空白上屏目前頁的第一個，不是全部候選的第一個：使用者翻到這頁就是在看這頁。brief §4.3 與 `GLOSSARY.md`「候選頁」同步改（code-review Spec 軸發現）。
- 2026-11-03 翻頁讓 `generation` 加一、到頭到尾不動作時不加：plan §2.1 寫「每次候選變動 +1」，`Snapshot.candidates` 只有目前頁。M1 做重排時再看翻頁中的重排結果要不要丟。
- 2026-11-03 組字中「不動作」的鍵（沒有該位置的數字、`0`、到頭到尾的翻頁）照樣吃下：交給宿主會把數字或 `-` 打進輸入框，組字卻還在。組字中方案字母表外的字母（`z`）與 Enter、←、→ 仍交還宿主，等 M1 定。
- 2026-11-03 `xtask/src/deps.rs` 的變異測試原本在 `qim-core/Cargo.toml` 檔尾追加依賴，假設最後一段是 `[dependencies]`；`qim-core` 加了 `[dev-dependencies]`（`proptest`）後失準。改寫成 `[dependencies.<名字>]` 子表，dev 依賴的綠燈案例改用 `serde_json`。
- 2026-11-03 性質測試在 `crates/qim-core/tests/properties.rs`：字 → 碼的對照直接由生成的碼表建、不經核心；在「Backspace 不重查候選」的變異上會紅。驗不到 `max_code_length`（查了超長碼也符合性質），那條由單元測試管。
- 2026-11-03 驗收輸出：`cargo xtask check` 各步驟通過，`qim-core` 的 `tests/session.rs` 17 passed、`tests/properties.rs` 1 passed，e2e `5 passed`，最後一行 `check 通過`。
- 2026-11-03 code-review 已處理：空白上屏目前頁第一個、翻頁加 `generation`、補第一頁截圖；`handle` 改成沒有組字時提早交還宿主，去掉五個分支重複的條件；`look_up` 改名 `refresh_candidates`；性質測試的 `code_table()` 改名 `random_table()`；名詞表加「候選頁」，避開佈局的「頁面」。
- 2026-11-03 code-review 沒處理的發現：
  - 測試資料包的 `Scheme` 建構在 `tests/pack.rs`、`tests/session.rs`、`tests/properties.rs` 各一份：三份的碼表各有用途，抽成 `tests/common` 省不了多少。`keys.spec.ts` 的 `ui()` 與 `type-a.spec.ts` 的定位器重複，同理。
  - `Key::Digit(u8)` 收得到 0–255：plan §2.1 的既有介面，核心用 `checked_sub` 與 `get` 擋掉，不另立型別。
