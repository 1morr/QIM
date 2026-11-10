# 12: 個人 bigram 與學習總開關

Status: done
Blocked by: 03, 04
讀: plan §2.3（`personal`）、§2.4（學習、匯出格式）、§5.2（`--learning`）；brief §4.5、§7；`docs/research/libime-user-model.md`；`docs/research/userdict-text-formats.md`；spec〈範圍〉〈共同規則〉

## What to build

學過的接續習慣影響排序：選過「前一個字詞 → 這一段」，下次同樣的前文下這一段加分。另補上 brief §7 的學習總開關。

- 個人 bigram（plan §2.3）：上屏時記（前一個字詞 → 這一段），只看文字、跨方案共用；前一個字詞來自同一次組字的前一段或 `set_context` 的前文。打分照 libime 的內插做法併進 `personal`，只加不減，參數用 `bench --learning` 調。
- 學習總開關：`Options` 加一項（預設開），關掉時不寫任何學習資料、已學的照用（和無痕不同：無痕連前文也不用）；網頁設定區加開關。
- 匯出格式升 `# qim-userdict v2`（使用者 2026-11-11 決定）：加個人 bigram 的列；`import` 收 v1 與 v2，v1 沒有 bigram。寫回 plan §2.4。

## 驗收

- [x] 單元測試：選過「X → Y」後，前文是 X 時 Y 的排名上升；學習開關關掉時 `UserModel` 不變、已學的仍生效。
- [x] `bench --learning` 學習開的修正次數比改之前少（含「有前文」集合），貼對照；`bench --check` 不變差。
- [x] `export` → `import` 等價（性質測試）；一份 v1 的檔仍匯入得了（測試用固定檔）。
- [x] Playwright：關掉學習後選兩次同一候選，它不成首選；截圖。
- [x] plan §2.3、§2.4 已更新；`cargo xtask check` 綠燈。

## Comments

- 2026-11-11 票 14：`# qim-userdict v2` 的檔頭、v1 匯入、詞欄跳脫（`\\`、`\n`、`\t`）與 `phrase` 來源已在 `qim-core/src/user.rs`；這張票往 v2 加 bigram 的列即可，見 plan §2.4。
- 2026-11-11 做法（plan §2.3、§2.4）：個人 bigram 存在 `UserModel` 層（`Bigrams`，自己的 tick），上屏時記「前一個字詞 → 第一段」、相鄰兩段、兩段以上再記「前一個字詞 → 整串」（給自動造詞）。前一個字詞：lattice 裡是部分路徑的最後一段；組字開頭是最後選定的分段，否則看前文，前文以上一次上屏的最後一段結尾時取那一段（簡體模式的前文也認），否則取前文最後一字。學習用的是組字開頭時的前文（組字中再給的留到下一次，同打分）。前文分不出詞，所以接得上整個詞的只有剛上屏的那段，同 libime 由上層傳上一次上屏的詞。
- 2026-11-11 打分：`P_bigram` = 這組衰減值 ÷ 前一個字詞後面各段衰減值總和（libime `HistoryBigram` 的條件機率）。合成不用 libime 的 `(1−λ)·Pu + λ·Pb`：沒有前一個字詞時學過的段只剩兩成個人分，`learning.rs` 的 `a_learned_word_also_scores_in_longer_continuous_input` 紅。改成 `Pu + (1−Pu)·λ·Pb`，bigram 只往上加。λ 用 dev 切分的學習模式（暫時改 `run_learning` 讀 `dev.tsv` 與 `dev-context.tsv`，已還原）掃，學習開的修正（yue-wiki、zh-wiki、common-voice-yue、hkcancor、四個「前文」，合計）：λ=0 4488/4400/3654/2203/2267/2181/591/1586（21370）；0.05 合計 21320；0.1 21308；0.2 21291；0.4 21297；**0.8 4450/4396/3614/2182/2254/2176/594/1580（21246）**；1 21251。分母加平滑 k（λ=0.8）：k=0.5 21265、1 21266、2 21278、4 21291，不採用。取 0.8（同 libime）。
- 2026-11-11 `bench --learning`（test 切分，學習開的修正，前 → 後）：yue-wiki 4277→4283（+6）、zh-wiki 4400→4372、common-voice-yue 3871→3810、hkcancor 2105→2085、yue-wiki·前文 2230→2235（+5）、zh-wiki·前文 2170→2151、common-voice-yue·前文 612→608、hkcancor·前文 1462→1440、personal 1407→1408（+1）。公開集合合計 21127→20984（−143），「前文」四集合合計 6474→6434。驗收照使用者決定以合計為準（progress〈偏差與決定〉）。學習關的欄前後相同。
- 2026-11-11 `bench --check`：公開集合答錯、修正與 baseline 逐字相同、翻盤清單都是「沒有翻盤」（基準用空白的 `UserModel`，個人 bigram 不起作用）；個人集前後都是答錯 1081、修正 1468，`bench 通過`。session 中途主 checkout 的本機個人集 baseline 被別的 worktree 改過兩次（1081/1468 → 1082/1467 → 1082/1469），本票沒有 `--update-baseline`。每鍵延遲（單獨跑，µs，p50/p95）：原生 34.17/53.17 → 32.46/44.62；WebAssembly 40.75/57.96 → 39.71/54.38，選開頭一段 282/931 → 274/897。沒學過個人 bigram 時 `Lattice::push` 不查（第一版每鍵都建查表，WebAssembly p95 到 65）。
- 2026-11-11 匯出 v2 加個人 bigram（照協調者：沿用票 14 的 v2，只加列）：`#@/tick\t<tick>` 一行（不帶方案）與方案欄空的 5 欄列 `（空）  前一個字詞  這一段  衰減值  最後 tick`，前兩欄照詞欄跳脫；慣例照 librime userdb 的 `#@/` metadata，方案欄空表示各方案共用。v1 有固定檔測試（`user_dict.rs` 的 `V1_FILE`），v1 裡方案欄空的列照舊不收、`#@/tick` 行是註解。
- 2026-11-11 學習總開關 `Options.learning`（預設開）：不寫計數、個人 bigram、自動造詞；手動加詞與常用語照收（使用者確認，同無痕）。性質測試 `learning_off_leaves_the_user_model_unchanged`；網頁設定區加「學習」，Playwright 兩例，截圖 `.scratch/m3/evidence/12-learning-off-top.png`、`12-settings-learning.png`。亂按一輪（400 鍵，組字中切學習、無痕、簡體）沒有 console 錯誤。
- 2026-11-11 測試的取捨：原本的性質測試「兩個方案共用 `UserModel` 互不影響」在個人 bigram 跨方案共用後不再成立，改成 `one_scheme_neither_writes_nor_reads_the_other_schemes_learning`：一個方案的動作不改另一個方案的匯出列，候選也不讀它們（拿掉另一方案的列再匯入，候選相同）。
- 2026-11-11 code-review（`24c53cd` 起）已處理：組字中再給前文時學錯前一個字詞（先紅再修，`composition_word`）；tick 行寫在列之後、比列的最後 tick 小時匯入後相減下溢（個人 bigram 與方案都修，方案沒有 tick 行時也取最後整串上屏 tick）；v1 的 `#@/tick` 行改回註解；無痕不記上一次上屏的段；`LastWord` 具名、`Preceding.character` 改名、測試去重；plan §0 名詞表補兩列。沒處理：無。
- 2026-11-11 驗證：`cargo xtask check` 全綠（e2e 166 passed、`check 通過`），`cargo test --workspace` 767 passed。
- 2026-11-11 協調者轉來的疑慮查證屬實：連打裡選過一次「丁 → 己」（`fg` 整串上屏 0 次），之後前文「丁」單打 `fg`，己成首選。`personal_bigram.rs` 的 `a_bigram_learned_mid_sentence_does_not_make_y_the_top_when_its_code_is_typed_alone` 紅：`left: "己"`、`right: "戊"`。這違反 brief §4.5「句中用過的只在連打裡加分」。先前的測試沒抓到，是因為甲、丙的字頻差 100 倍，bigram 加的分不夠翻盤；換成字頻只差 3 倍的戊、己才紅。修法：規則 2 的「選一次不成首選」放寬到整串上屏 0 次的候選，只限只有一段（就是單打這串碼）、又帶學到的分（學習計數或個人 bigram）的候選。多段的候選是連打，不擋（由 `a_bigram_learned_mid_sentence_still_scores_in_continuous_input` 守）。個人 bigram 跨方案，另一個方案沒有學習計數，所以 bigram 那一支要另判（由 `…_in_another_scheme_…` 守，拿掉這一支就紅）。同一道門也擋住句中學到的學習計數搶單打的首選（plan §2.4 本來就寫了，以前沒擋）。已改 plan §2.4 規則 2。
- 2026-11-11 加門檻後 `--learning`（學習開的修正，加門檻前 → 後）：yue-wiki 4283→4281、zh-wiki 4372→4372、common-voice-yue 3810→3813、hkcancor 2085→2090、yue-wiki·前文 2235→2234、zh-wiki·前文 2151→2152、common-voice-yue·前文 608→608、hkcancor·前文 1440→1447、personal 1408→1404。公開集合合計 20984→20997；和本票開工前（21127）比 −130；個人集和開工前比 1407→1404。
- 2026-11-11 合併 main（票 05、17、18、20）：衝突在 `session.rs`（`Session` 的欄位與 `before_unselected` 的註解，票 05 的前文改由核心接上上屏的字，`context_word` 照用：剛上屏的段就在前文結尾）、`qim-wasm`（`setLearning` 與票 05 的 `setContext` 等並存）、plan §0 名詞表與 §2.1 上下文（取票 05 的版本、補個人 bigram 一句）。重建資料包（票 18 改了詞庫）、`npm ci`（票 20 加了 `idb-keyval`）後 `cargo xtask check` 全綠（e2e 188 passed），`cargo test --workspace` 828 passed。`bench --check` 通過：各集合答錯、修正與 baseline 相同（yue-wiki 1123／4295、zh-wiki 1126／4381、common-voice-yue 1030／3908、hkcancor 650／2478、personal 1082／1469），翻盤清單都是「沒有翻盤」。`--learning` 學習開，main（同一份資料包、暫時 worktree）→ 合併後：yue-wiki 4225→4220、zh-wiki 4396→4368、common-voice-yue 3817→3763、hkcancor 2003→1980、yue-wiki·前文 2198→2204（+6）、zh-wiki·前文 2170→2152、common-voice-yue·前文 594→590、hkcancor·前文 1408→1396、personal 1407→1404。公開集合合計 20811→20673（−138），「前文」四集合合計 6370→6342；照先前的決定以合計為準。
