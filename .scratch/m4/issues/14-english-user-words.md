# 14: 英文使用者詞

Status: done
Blocked by: 12
讀: brief §1.5、§4.5、§4.7、§7、§8；plan §2.1（記住的字串）、§2.4（規則 4、5）；spec〈細節〉〈英文使用者詞〉；`docs/research/remember-strings.md`；`scripts/experiments/remembered-prefix/`；`GLOSSARY.md`；spec〈共同規則〉

## What to build

英文使用者詞取代「記住的字串」（使用者 2026-11-12 決定，照 LatinIME）：

- 核心看得到送出去的英文串都學（第一期是英文直通；英文模式在票 24 接上），不另認 email；數字、電話、卡號照學。不學：無痕、密碼欄、學習關（brief §8、§4.5）。至少 4 個字元才學。
- 出現門檻：英文詞典沒有的串一年內打過 2 次才出；中文組字的前綴補完要 3 次，只給最常用的 20 條；詞典有的詞不受門檻，學習只讓它排前。沒到門檻的照樣匯出、同步，條目到上限時最先刪它們（票 13）。
- 用自己的 tick，不推進個人 bigram 的 tick。v3 加它的列種類；v2 匯入的記住的字串轉成已過門檻。
- 出在哪裡（第一期）：中文組字打完整串時，規則 4 的英文詞表加上學到的（速成打得出的字母才算），排第 2；中文組字的前綴補完取代現在的規則 5，email 不再特別。英文建議列在票 24。
- 網頁設定區「記住的 email」一段改成英文使用者詞的清單（可逐條刪）；對應的開關改名。

## 驗收

- [x] 核心測試：4 字元起才學；兩種門檻與 20 條上限；詞典有的詞不受門檻；無痕、學習關不學；數字串照學。
- [x] v2 的記住的字串匯入後是已過門檻的英文使用者詞；沒到門檻的照樣匯出。
- [x] 上限淘汰先刪沒到出現門檻的英文使用者詞，有測試（票 13 的 `Eviction` 留了「沒到門檻」的接點，中文條目都算已過門檻；票 13 的 Comments）。
- [x] Playwright（實體鍵盤英文直通）：打 `rrrrrr` 一次後，中文組字打 `rrrrrr` 沒有它，打第二次後第 2 位有；v2 匯入的 email 在中文組字打開頭 3 碼出現；debug 面板標來源「英文使用者詞」。
- [x] `bench --check` 不變；brief §1.5、§4.7、§7 與 plan §2.1、§2.4 規則 4、5 改寫，名詞表換掉「記住的字串」；`cargo xtask check` 綠燈。

## Comments

- 2026-11-12 實作（開工 `7d76fc7`）：
  - 模組：`remembered.rs` 改名 `english_user.rs`，放正在打的一串（`TypedRun`，邊界規則不變）、要學的串（`to_learn`）與學到的詞（`EnglishUserWords`，鍵是小寫、留最後打的寫法，記次數、衰減值、自己的 tick）。`UserModel` 只轉呼叫並帶入今天。
  - 學什麼：串字元多收 `'`（`don't`），頭尾的 `.` `'` 去掉（句點、引號），4 字元起、有英文字母或數字才學；網址在 `/` `:` 斷開各段分開學。英文模式的切詞由票 21 的 ADR 定，可能改串字元。
  - 規則 4：詞典有這個詞時照詞典（寫法、種類「英文」、`english_candidates` 管）；沒有時看英文使用者詞：全是方案字母、一年內打過 2 次，debug 面板種類「英文使用者詞」。詞典有的詞不由學習冒出來（關掉英文候選時也不出，Spec 審查的發現，已修並補測試）。
  - 規則 5（協調者 2026-11-12 決定）：前綴補完只從英文詞典沒有、打過 3 次的詞裡取衰減值最大的 20 條，每列最多 3 個；補了核心測試（常打 that/have/with 與 20 個詞典詞後 email 仍在）與 Playwright（混打 that have with there about）。
  - 次數的一年時限照中文的整串上屏：最後一次距今超過 365 天時當作 0、下一次從 1 重算。
  - 上限：英文使用者詞和（方案, 輸入碼, 詞）的列算同一個 `entries` 上限（LatinIME 的 unigram 不分來源），一年內打不到 2 次的是 `Threshold::Below`、最先淘汰。
  - v3：種類 `english` 6 欄（寫法、次數、衰減值、最後 tick、最後使用的日期）、`#@/english_tick`；票 12 草稿的 `string` 不再收（v3 未發佈），當不認得的種類略過並警告：開發中存在瀏覽器的 v3 `string` 列重新整理後會不見，v2 檔照收。v2 的記住的字串匯入成次數 3（規則 4、5 都過），不再要求是 email，英文使用者詞的 tick 接 `#@/tick`。
  - 網頁：設定區「英文使用者詞」清單（`web/src/english-user-words.ts`，沒到門檻的也列、可刪）、開關「英文使用者詞」（設定 id 改名，舊的「記住 email」開關值不沿用、回到預設開）；debug 面板種類 `englishUserWord`。
  - 驗證：`cargo test --workspace`、clippy、`cargo xtask check`（含 Playwright 全套）綠燈；實際操作網頁亂按一輪（英文直通夾標點、退格、Tab、組字中切開關、臨時英文 Enter 後接中文、debug 面板），沒有找到問題，截圖 `.scratch/m4/evidence/14-english-user-words-debug.png`。
  - `bench --check`（改動後；基準不學英文使用者詞，預期不變）：各集合的答錯、修正都等於 baseline（yue-wiki 941／3239、zh-wiki 979／3493、common-voice-yue 892／3249、hkcancor 572／2148，前文各集合同 baseline），12 個集合都沒有翻盤，固定題 84/84。每鍵 p95（原生）：空白 全部 87.5 µs、第 20 碼起 92.5 µs、選開頭一段 1.47 ms；學過 184 µs／252 µs／2.23 ms，都在〈每鍵延遲上限〉內。
  - `bench --learning`：學習關／開的修正 yue-wiki 3239／3216、zh-wiki 3493／3578、common-voice-yue 3249／3108、hkcancor 2148／1719、yue-wiki·前文 1738／1715、zh-wiki·前文 1729／1764、common-voice-yue·前文 594／550、hkcancor·前文 1512／1179，和票 13 記的改動前逐項相同。
- code-review（`7d76fc7` 起）未處理的發現：
  - 上限淘汰把打過一次的詞典詞也當沒到門檻（`UserModel` 看不到英文詞典）：和「詞典有的詞不受門檻」字面不一致，第二期英文建議列靠詞典詞的學習排前時會先被淘汰。plan §2.4〈條目上限〉已註明；票 24 接英文建議列時決定要不要學習時記下「是不是詞典詞」或淘汰時傳入判斷。
  - v3 草稿 `string` 列不轉換（見上）：判斷題，v3 未發佈，不改。
  - Standards 審查的 Feature Envy（`enforce_limits` 直接讀英文使用者詞的欄位）：抽了 `passed`、`insert`，淘汰的排序鍵仍在 `user.rs`（`Eviction` 是它的型別），等票 17 改合併時再看要不要整個搬進 `english_user.rs`。
- 2026-11-12 合併 main（`e57b0cd`，票 06、11、05b、05c、06b）：
  - 票 11 的萬用鍵在新名詞下照樣成立：前綴補完比對時略過 `z`（`lattice.rs` 的 `completions`）；含 `z` 的輸入不出整串的英文使用者詞（`typed_as` 要整串是方案字母）。票 11 加在舊 `remembered_strings.rs` 的測試搬到 `english_user_words.rs`，另補規則 4 一條；拿掉略過 `z` 的過濾時前一條會紅（試過）。plan §2.5 萬用鍵段改用新名詞。
  - 資料包格式升到 8（票 11），跑 e2e、bench 前要先 `cargo xtask data`。
  - 驗證：`cargo test --workspace` 1067 passed；`cargo xtask check` 通過（e2e 244 passed、2 skipped）。`bench --check` 通過，12 個集合的答錯、修正都等於 baseline，沒有翻盤，固定題 84/84。原生 p95：空白 74.1 µs／80.0 µs／1.31 ms，學過 150 µs／228 µs／1.89 ms。`--learning` 和合併前逐項相同。
