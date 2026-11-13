# 04: 簡體模式的前文轉回繁體

Status: done
Blocked by: None (can start immediately)
讀: plan §2.3、§2.7（簡體模式那條）、§4.4；spec〈共同規則〉

## What to build

簡體模式下，殼給的前文是簡體字，但同步層的統計（字級 bigram、詞級 trigram）都是繁體：簡體字沒有統計，前文等於沒用，退回 unigram。停鍵層已經把前文逐字轉回繁體（`Simplifier::traditional_chars`，M3 票 11e），同步層照做。個人 bigram 的前文要不要也轉，由票定（它記的是上屏的原文還是轉換後的字，先查清楚）。

## 驗收

- [x] 先寫會紅的核心測試：簡體模式的前文接一串碼，首選和繁體模式下對應繁體前文的首選相同；修後綠。
- [x] 個人 bigram 的前文照決定處理，有測試。
- [x] Playwright：簡體模式下前文改變候選；截圖。
- [x] `cargo xtask bench --check` 前後對照（基準是繁體，預期不變）；plan §2.7 刪掉「同步層 bigram 的前文還沒轉」；`cargo xtask check` 綠燈。

## Comments

### 決定：個人 bigram 的前文也轉

個人 bigram 記的是原文（`LastWord.original`；`learn` 記選定分段的 `text`，簡體模式下也是繁體）。所以前文不是剛上屏那段時，退回的「前文最後一字」在簡體模式也轉回繁體，讀與寫用同一個字：簡體模式在「门」後學的，繁體模式在「門」後用得到，反過來也是（`personal_bigram.rs` 的 `simplified_context_reads_and_writes_the_traditional_previous_word`）。

字級 bigram 與詞級 trigram 的前一字同一規則（`Session::context_last_char`）：前文以剛上屏的那一段（原文或顯示的字）結尾時取那段原文的最後一字，「幹」顯示「干」仍接「幹」；否則簡體模式逐字轉回（`Engine::traditional_char`，同停鍵層）。停鍵層的 `rerank_request` 照舊只逐字轉，沒改（plan §2.7 已註明差別）。

### 測試

- 核心：`context.rs` 三條（簡體前文、繁體模式不轉、剛上屏那段取原文；含一串碼 `ab`），`word_trigram.rs` 一條（詞級 bigram 接轉回的前一段），`personal_bigram.rs` 一條。拿掉轉換時三個檔各紅一條，修後綠。
- Playwright：`simplified.spec.ts`「簡體模式下前文「点」讓 nq 的首選變成「解」」：前文以英文結尾（沒有統計）是「建」，「点」與「點」都是「解」。拿掉轉換重編時讀到「1建」、紅；截圖 `.scratch/m4/evidence/04-simplified-context.png`。
- 沒有另外亂按網頁：這張票只改核心，網頁程式碼沒動。

### `cargo xtask bench --check` 前後對照

前是 baseline（`157f3a7`），後是這張票。各集合（含「有前文」的給與不給、個人集）答錯數與修正次數都和 baseline 相同，依長度分組不變，13 個集合都沒有翻盤，固定題 84/84。基準是繁體，簡體模式的分支走不到，符合預期。

| 集合 | 答錯（前 / 後） | 修正（前 / 後） |
| --- | --- | --- |
| yue-wiki | 941 / 941 | 3239 / 3239 |
| zh-wiki | 979 / 979 | 3493 / 3493 |
| common-voice-yue | 892 / 892 | 3249 / 3249 |
| hkcancor | 572 / 572 | 2148 / 2148 |
| 四個「·前文」合計 | 1524 / 1524 | 5573 / 5573 |
| 四個「·前文·不給」合計 | 1568 / 1568 | 5651 / 5651 |

每鍵延遲（原生 p95）：空白 全部 91 µs、第 20 碼起 95 µs、選開頭一段 1.52 ms；學過 208 µs、307 µs、2.59 ms，都在上限內。

### `cargo xtask bench --learning` 前後對照

動到個人 bigram 的讀寫，前（把 `engine.rs`、`session.rs` 換回 `157f3a7`）後各跑一次，表格逐列相同（學習開的修正：yue-wiki 3216、zh-wiki 3578、common-voice-yue 3108、hkcancor 1719、個人集 1390）。

### code-review 未處理的發現

- 測試的 `simplifier()` 這種 fixture 現在有好幾份（emoji、simplified、personal_bigram、context 各一份，word_trigram 就地組）。搬進 `tests/common` 會讓 include 它但用不到的測試檔出現 dead_code 警告，沒搬。
- e2e 的 `place`（把文字放進輸入框）有三份：`context.spec.ts`、`learning.spec.ts` 的 `placeBefore`、這張票的 `simplified.spec.ts`。`web/e2e/` 還沒有共用 helper 檔，留給之後整理 e2e 的票。
- `LastWord.shown` 是那一段單獨轉換的結果。整次上屏用 TSPhrases 跨段轉、結果和單段轉不同時，`last_word_ending` 對不上，會退回逐字轉，一對多的字取 unigram 最高的。這是原本就有的行為，沒改。
- 詞級 trigram 在「前文以剛上屏的多字詞結尾」（上屏「頭髮」顯示「头发」）沒有直接的核心測試；`Preceding::segment` 沒改，`character` 與 `word` 都取那段原文，所以前一段是整個詞。

Session 紀錄：簡體模式的前文轉回繁體再接字級 bigram、詞級 trigram 與個人 bigram（剛上屏那段取原文）；bench 與 --learning 前後相同
