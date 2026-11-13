# 16: 撤銷學習

Status: done
Blocked by: 12
讀: brief §4.5；plan §2.1、§2.4、§3.1；spec〈細節〉（撤銷學習）；`docs/research/cyime.md` §4；spec〈共同規則〉

## What to build

學錯的候選能清掉：

- 核心 API 清掉一個候選的學習紀錄：詞庫原有的詞回到原位，自動造詞不再出，再選可以重新學。不做永久屏蔽（CyIME 的永久屏蔽被使用者評為偏離預期）。手動加詞與常用語的處理（建議只清計數、不刪詞）由票定。
- 網頁桌面用 Shift+Delete（Rime 慣例）；作用在哪個候選（首選或高亮的）由票定。觸控的手勢跟 M5 重做候選列一起定。
- 刪除在同步時怎麼傳到別台，由票 17 處理。

## 驗收

- [x] 核心測試：學成首選的詞清掉後回到不學習時的位置；自動造詞清掉後不再出；再選兩次又成首選。
- [x] Playwright：Shift+Delete 後讀回候選回到原位，debug 面板的學習加分歸零。
- [x] plan §2.1、§2.4 寫明；`cargo xtask check` 綠燈。

## Comments

- 做了什麼：核心 `Session::forget(index)`（任一位置，觸控 M5 用得上）與 `UserModel::forget`；網頁組字中 Shift+Delete 或 Ctrl+Delete 清目前頁的第一個、存檔；debug 面板每個候選列加 `data-learning`。測試 `crates/qim-core/tests/forget.rs`（9 條）、`english_user_words.rs` 一條、`web/e2e/forget.spec.ts`（4 條）。範圍寫在 plan §2.4〈撤銷學習〉。
- 票內的取捨：
  - 手動加詞與常用語只清計數、詞留著（照票的建議）；英文候選、英文使用者詞與前綴補完清那個英文使用者詞；emoji 不動。
  - 個人 bigram 只清「現在前文 → 它」一組（協調者決定，刪學習資料從嚴）：一開始實作成清所有指向它的組，審查標成超出範圍，清「的」會抹掉它在所有前文後面的學習。debug 面板的學習加分只在現在前文下歸零；`forgetting_a_common_character_keeps_its_bigrams_after_other_words` 守它。
  - 自動造詞刪掉時一起清詞內前後相接的 bigram（「乙 → 丁」）：Spec 審查指出自動造詞選到首選再清，會換成句子候選回到前面。字頻高時重現（「乙丁」回到第 2 位），清掉後回到不學習時的候選列。
  - 句子候選的各段不清（同 librime 只刪那個詞條）。以句子路徑上屏過幾次的詞，清掉後各字學到的計數還在，仍可能排在後面幾位（`the_focused_segment_is_kept_only_while_its_code_stays_the_same` 的第 3 位）。
  - 無痕、學習關掉時也清得掉（刪除不是學習）；`forget` 也是一個動作，撤銷紀錄跟著丟掉。
  - 沒有開關（spec 說改變候選位置的新行為要有開關）：這是使用者明確按鍵才發生的刪除，不是自動改排序，所以不加。
- 桌面按鍵（協調者要求查證）：沒有跨產品的「刪第 N 個候選」官方慣例。fcitx5 的 Ctrl+7 加數字是孤例，而且 Ctrl+7 在瀏覽器是切分頁。librime 預設綁 Ctrl+Delete，Shift+Delete 是 Shift 當 Ctrl 的退路，作用在高亮的候選，所以兩個都收；QIM 桌面沒有高亮，照 librime 的預設高亮（首選，翻頁後是新頁第一個）。出處寫進 brief §12.2、§12.4。
  - 限制：自動造詞釘在第 2 位（規則 3），造錯的在它成首選前桌面清不到。已寫進 plan §2.4 與 §7.9（M5 觸控的撤銷學習手勢要能指定任一個候選）。
- code-review（從 `6ec152a`）已處理：說明裡「Rime 慣例」改成 librime 實際的綁定；`forgetsTop` 改名 `isForgetKey` 並收進 Delete 的判斷；目前分段在重切後換成別段（先寫成會紅的測試再修：碼還在才留著，否則回到整串）；翻頁時清到看不見的第一頁首選（改成清目前頁第一個，e2e 先紅後綠）；bigram 範圍（見上）。
- code-review 未處理：
  - 簡體模式兩個原文轉成同一個簡體字時，去重只留第一個，只清得到它，另一個學過的原文可能以同樣的字頂上來。邊角情況，留給 M5 重做候選列時一起看。
  - `tests/forget.rs` 的 `Data`、`engine()`、`Ime` 和 `tests/learning.rs` 幾乎一樣，e2e 的 `ready`、`shown`、`choose` 和其他 spec 重複：沿用各測試檔自帶小工具的既有寫法，沒有抽共用。
  - `Session::forget` 的 `begin_action`/`end_action` 會讓自己的撤銷紀錄失效，`UserModel::forget` 又丟一次（含別的 Session 的）：前者讓 `forget` 跟其他動作一樣，後者是「使用者明確的操作丟掉紀錄」的共同規則，留著。
- 亂按一輪（`.scratch/m4/evidence/16-monkey.png`）：組字中移到某段、翻頁、分段選字後、臨時英文中、快速連按 Shift+Delete／Ctrl+Delete／Meta+Delete／Alt+Shift+Delete、無痕，頁面沒有錯誤。發現一個跟這張票無關的既有問題：debug 面板開著又打過字之後，第一次點「無痕」勾選框沒反應、第二次才勾上（不按 Shift+Delete 也一樣，debug 關著或沒打字時正常）；推測是失焦時面板重畫、版面移動。沒有修，交給收尾票或 debug 面板的下一張票。
- 截圖：`.scratch/m4/evidence/16-forget-before.png`、`16-forget-after.png`。
- `bench --check`（中途的版本，bigram 改從嚴與目前分段修正之前；基準不呼叫 `forget`，兩項改動都只在 `forget` 裡）：12 個集合答錯、修正都等於 baseline（yue-wiki 941／3239、zh-wiki 979／3493、common-voice-yue 892／3249、hkcancor 572／2148），沒有翻盤，固定題 84/84。每鍵 p95（原生）：空白 71.2 µs／76.3 µs／1.28 ms，學過 162 µs／226 µs／1.99 ms，都在上限內。
- `bench --learning`：學習關／開 yue-wiki 3239／3216、zh-wiki 3493／3578、common-voice-yue 3249／3108、hkcancor 2148／1719、yue-wiki·前文 1738／1715、zh-wiki·前文 1729／1764、common-voice-yue·前文 594／550、hkcancor·前文 1512／1179，和票 15 記的逐項相同。

Session 紀錄：撤銷學習：核心 `Session::forget`、網頁組字中 Shift+Delete／Ctrl+Delete 清目前頁第一個；個人 bigram 只清現在前文那組；第 2 位的自動造詞桌面清不到，留給 M5
