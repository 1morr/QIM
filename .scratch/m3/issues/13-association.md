# 13: 聯想

Status: done
Blocked by: 05, 12
讀: plan §2.1、§2.4；brief §1.3（iOS 速成的聯想）、§1.5、§7；`docs/research/blackbox-baseline.md`（T6、T8）；`GLOSSARY.md`〈聯想〉；spec〈範圍〉〈共同規則〉

## What to build

上屏之後、還沒開始打下一個字時，候選列出現接下來可能要打的詞，選了就上屏、可以一直接下去。

- 先查成熟做法定按鍵行為（iOS / macOS 速成、Microsoft 速成的聯想詞）：數字鍵或點擊選、空白要不要選、Esc 收起、打字母直接開始新的組字；寫進 plan §2.1，註明採用誰的慣例。
- 來源：詞庫裡以剛上屏的字詞結尾開頭的詞、字級 bigram、個人 bigram，用同一把尺排（plan §2.3）；排序規則寫回 plan §2.3 或 §2.4。
- `Options` 加聯想開關（預設開，brief §7）；無痕時不用前文；網頁設定區加開關。

## 驗收

- [x] 單元測試：上屏「門」後沒有組字時有聯想候選（例如「口」）；選了之後接著出下一輪；打字母時聯想消失、照常組字；關掉開關時沒有聯想。
- [x] Playwright：實體鍵盤與觸控各打一次「上屏 → 選聯想 → 再選聯想」讀回輸入框文字；截圖。
- [x] 黑箱 T6、T8 在 QIM 網頁重跑，結果補進 `docs/research/blackbox-baseline.md` 的 QIM 欄。
- [x] `bench --check` 不變差；實際操作亂按一輪；`cargo xtask check` 綠燈。

## Comments

- 2026-11-11 按鍵行為（plan §2.1，查證摘進 brief §12.4）：照 macOS 速成（黑箱 T8）：數字鍵或點選選、沒有預選、空白不選照打空格；Esc 與退格只收起聯想、吃下不交還（librime-predict `920bd41` 的 `predictor.cc`；Gboard 第一下刪除也是），字母直接開始新的組字，其他鍵收起後照常。不學 Microsoft「關聯片語」的 Shift＋數字：實體鍵盤的 Shift＋數字是全形標點。代價：上屏中文後緊接著打數字要先按 Esc 或關掉聯想（同 macOS）。標點、大寫、Enter、英文候選、emoji、`text` 的上屏不出聯想（同 librime-predict 遇到 `punct`、`raw`、`thru` 就停）。
- 2026-11-11 來源與排序（plan §2.3）：補完詞庫的詞、字級 bigram 留下的接續字、以前 9 個接續字開頭的詞、個人 bigram，分數是「接上這個聯想的路徑比不接時多出多少」。第一版補完只把剛上屏的字當單字算（`ln P(詞) − ln P(那段) + β`），真資料上「日 → 本」之後給「本身、本科」而不是「日本人」；改成照剛上屏的分段算差（併段省 γ、單字補成詞多 β），之後「日本」→ 人 第 1。只從分段邊界補，不拆開剛上屏的詞。選聯想記個人 bigram「最後一段 → 聯想」（`UserModel::learn_association`），沒有輸入碼所以不記學習計數。真資料：門 → 檻、口、店、票、診…；日 → 本、常、子、語、本人、本身…；日本人的 → 觀點、時候、話…。
- 2026-11-11 無痕：剛上屏的字就是下一個字的前文，無痕不讀前文（brief §8），所以無痕時不出聯想（`incognito_gives_no_association`、e2e「無痕時上屏後沒有聯想」）。第一版照給、還用個人 bigram 加分，code-review 指出和「無痕時不用前文」不一致後改掉。沒做（不在票內）：emoji 聯想（macOS 有）、空輸入的句首建議（iOS 有）、數字或標點後的量詞聯想（iOS 有），黑箱基線 T8 標成缺功能、沒排進里程碑。
- 2026-11-11 成本：詞庫另依文字排序存一份（`WordsByText`：文字接成一串、u32 位移、f32 機率），約 26 萬詞常駐多約 4 MB；最大常駐記憶體（`/usr/bin/time -l`，載入全部資料包的 release 程式）89–92 MB → 97 MB，第一版每詞一個 `Box<str>` 是 102 MB。載入 78–109 ms → 98–114 ms。上屏時算聯想約 1.5 ms（原生 release）。每鍵延遲不變（bench 只量組字中的鍵）。
- 2026-11-11 `bench --check`：各集合答錯、修正與改之前逐字相同、`bench 通過`（聯想只在組字之外，基準的組字不經過它）；`bench --learning`：前後輸出逐字相同（學習開的修正 yue-wiki 4220、zh-wiki 4368、common-voice-yue 3763、hkcancor 1980，前文四集 2204／2152／590／1396）。延遲（µs，p50/p95）：原生 34.12/56.88 → 32.62/49.12，WebAssembly 39.58/54.00 → 39.50/53.87（雜訊）。
- 2026-11-11 網頁：設定區加「聯想」；交還宿主的鍵（空白、Enter、方向鍵、觸控的空白）之後也重畫候選列，聯想才會跟著收起；失焦時清掉候選列（核心 `clear_context` 收起聯想）。既有 e2e 有 8 條假設上屏後候選列是空的，改成斷言組字清空；`keys.spec.ts` 的「a 1-」改成上屏後先按 Esc 再打數字（數字鍵會選聯想）。新 e2e `association.spec.ts` 八條（實體鍵盤「上屏 → 選聯想 → 再選聯想」、字母照常組字、空白／Esc／退格、開關兩條、失焦；觸控「點空白 → 點聯想 → 再點」、刪除鍵與空白），截圖 `.scratch/m3/evidence/13-desktop-after-*.png`、`13-touch-after-*.png`。核心性質測試「候選打得出列著的輸入碼」「逐鍵等於從頭算」改成只在組字中檢查：聯想沒有輸入碼。
- 2026-11-11 黑箱 T6、T8 在 QIM 網頁重跑，結果寫進 `docs/research/blackbox-baseline.md` 的 QIM 欄與〈給 QIM 的取捨〉（截圖 `13-t6-*.png`、`13-t8-chain.png`）。亂按一輪：實體鍵盤 400 鍵（字母、空白、數字、Esc、退格、標點、Enter、方向鍵、Tab、↓、單按 Shift、點候選，中途切聯想、簡體、無痕）與觸控 150 下，沒有 console 錯誤，沒有組字時按 Esc 後候選列都清空（`13-chaos.png`、`13-chaos-touch.png`）。
- 2026-11-11 code-review（`c623f86` 起，Standards、Spec 兩軸）已處理：無痕照給聯想（上一條）；點輸入框移游標後舊聯想還在、按數字插在新位置 → 殼給的前文不以剛上屏的字結尾時收起（核心測試 `a_context_not_ending_with_the_commit_closes_the_association`、e2e「游標移走時聯想收起」），網頁的失焦與給前文統一由 `mountContext` 的回呼重畫候選列，拿掉 `main.ts` 另掛的 blur；只呼叫 `set_options` 關掉聯想後 `select` 沒收起 → 收起；`typeable` 改名 `continuous_chars`（plan §0 連打字集 → `continuous`）；`fresh` 改名 `score_as_new_segment`；簡轉與去重抽成 `shown_candidates` 兩處共用；`ASSOCIATION_LIMIT` 不再公開（crate 外沒人用）；欄位註解、f32 捨入寫明；補測試：`clear_context`、`text`、修飾鍵、上屏切英文、分兩段選定的組字補完。沒處理（判斷項）：
  - `association.rs` 的 `segment_static` 與 `Engine::word_edge`、`user_word_edge` 公式重複：邊要拆成首字與詞內兩半存（lattice 逐鍵接前一字），聯想要整段的值，合成一個函式要改 `Edge` 的建法；兩邊都寫明照 plan §2.3，公式改時一起改。
  - `Engine::words_starting_with`、`word_log_prob` 只轉給 `WordsByText`：留著讓 `association.rs` 不碰 `Engine` 的欄位。
  - `ASSOCIATION_TAIL` 在 `commit_and_associate` 截段（不讓一直接下去時無限長）與 `associations` 補完時各用一次，用途不同。
  - 「只從分段邊界補、不拆開剛上屏的詞」縮小了票的「以剛上屏的字詞結尾開頭的詞」：拆開要扣 γ 又丟 β，很少贏；記進 progress〈偏差與決定〉。
  - 黑箱基線〈給 QIM 的取捨〉的「打碼時看前文」一列順手改成票 04、05 的現況。
- 2026-11-11 驗證（code-review 修完後）：`cargo xtask check` 通過（e2e 198 passed）、`cargo test --workspace` 854 passed、`cargo xtask bench --check` 通過（數字同上）。
