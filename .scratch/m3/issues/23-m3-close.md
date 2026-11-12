# 23: M3 收尾

Status: done
Blocked by: 01, 02, 03, 04, 05, 06, 07, 08, 09, 10, 11, 11b, 11c, 11d, 11e, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 24, 25（22 依票 11 的結論 wontfix）
讀: spec〈驗收〉；`docs/progress.md`〈寫法與封存〉；plan §2.4、§7、§8；`docs/development.md`〈全部檢查〉；票 01–22 的 `## Comments`

## What to build

M3 的驗收全部重驗並留下證據，然後封存 M3、M4 標進行中。

- 里程碑 code-review：`/code-review <M3 開工時的 commit>`，Spec 軸讀 spec 與各票，Standards 軸讀 `CLAUDE.md`。小問題修掉；沒修的與各票 Comments 沒處理的發現，整理成清單寫在本票 `## Comments`，每條註明留到哪個里程碑或不修的理由。
- 學習資料匯出 v2 定稿，寫進 plan §2.4（spec〈共同規則〉）。
- 問使用者有沒有在網頁按「回報」留下的案例；有的話照 plan §5.2 加進 `fixed.tsv`（M1 留下）。
- 在乾淨 checkout（`git worktree add --detach` 到暫存目錄，照 `docs/development.md` 從零安裝）跑 spec〈驗收〉的每一項，輸出與截圖路徑貼在 Comments。
- 封存（〈寫法與封存〉，同一個 commit）：M3 的 session 紀錄與偏差移到 `docs/archive/M3.md`；里程碑表 M3 連到封存檔、M4 進行中；plan 開頭狀態行與 §7 表更新，§7.4 指向 spec、封存檔與本票；本 session 的紀錄標 `M4`，下一步寫「新 session 下 `/to-tickets docs/plan.md 拆 M4 的票`」。

## 驗收

- [x] 里程碑 code-review 跑完；沒處理的發現清單寫在 `## Comments`，每條有去處或理由。
- [x] spec〈驗收〉每一項在乾淨 checkout 成立，證據貼在 `## Comments`。
- [x] plan §2.4 寫明 v2 定稿；回報案例已處理或註明沒有。
- [x] `docs/archive/M3.md` 有 M3 全部 session 紀錄與偏差；progress 只留 M4；里程碑表與 plan §7 已更新；同一個 commit。
- [x] 封存後 `cargo xtask check` 綠燈。

## Comments

- 2026-11-12 開工：M3 開工的 commit 是 `2dbb7ea`（spec 與票）；本票從 `4bf1345` 起。`bench/personal` 用軟連結接主 checkout 的個人集。使用者確認沒有網頁「回報」留下的案例，`bench/fixed.tsv` 不動。
- 2026-11-12 里程碑 code-review（`2dbb7ea...4bf1345`，121 個 commit、356 檔、+47108／−2200）：Standards 軸分三份（核心與綁定；資料建置、bench、xtask、網頁；文件與實驗），讀 `CLAUDE.md`、plan §0–§2、§3.1、§4–§6、`GLOSSARY.md`、spec〈共同規則〉、`docs/development.md`；Spec 軸分兩份（產品碼；文件與實驗），讀 spec、各票與 progress〈偏差與決定〉。沒有違反依賴規則、個人集沒有漏進 repo、網頁沒有任何雲端請求。已處理：
  - 匯出再匯入不等價：只剩 tick 行、沒有資料列的方案（建過常用語又刪掉，或在空模型上刪常用語）匯入後丟了 tick（Spec）。`UserModel::import` 對每個 tick 行的方案都設 tick；先寫的 `phrases.rs` 兩例 `exporting_then_importing_keeps_a_tick_left_after_removing_the_only_phrase`、`removing_a_missing_phrase_from_an_empty_model_changes_nothing` 修前紅、修後綠；`properties.rs` 的匯出再匯入性質測試加上隨機刪常用語，拿掉修法時紅（最小反例：空模型刪一條）。
  - 個人 bigram 沒有網頁的 Playwright 測試（spec〈驗收〉第 1 條）（Spec）：`learning.spec.ts`「在「我」後面選「你」一次，「我」後面的 of 比在「他」後面選時把「你」排得更前」，開兩個乾淨的瀏覽器狀態各選一次，同碼計數一樣，名次差別只來自 bigram（本 session 的 code-review 後改成這樣，見最後一條）；截圖 `.scratch/m3/evidence/23-personal-bigram.png`。
  - `rerank.ts` 的 `.then(ok, err)` 接不到 `onResult`、`show` 丟的錯，會變成 unhandled rejection（Standards）：改成 `.then().catch()`，只回報一次 `"none"`；`web/test/rerank.test.ts` 加一例，修前紅、修後綠。
  - `engine.rs` 的「`prev` 是 `None` 表示組字開頭」與 `release.rs` 的「七份」過時；`qim-neural` 的 `Vocab`、`SEP`、`UNK`、`Reranker::scores` 在 crate 外沒人用，改 `pub(crate)`，`Vocab::is_empty` 因此成了沒人用的碼、刪掉（Standards）。
  - 文件停在舊結論（兩軸都指出）：`docs/research/model-ladder.md` 的結論表與同步層表還寫放寬版「有條件採用」（票 11b 量到 +71 MB 不改用）；23M「再多 2.0 點」註明是票 25 重訓重量的數字、不採用以 iPhone 峰值為準；p 值寫法對齊 README。plan §2.1 `rerank_request` 的簽章（11e 加了 `engine`）、`CONTEXT_LIMIT` 200 的理由、「補長度由票 22 定」改成不補、刪 `with_segment_model` 的歷史句；§3.1、§4.7、§8 的「雲端經 Vite 轉發」改成重評時的做法；§7 表 M3 的範圍、「之後每項都落在 M3–M8」註明雲端聯想與補全的例外；§4.3 補 MiB。brief §12.3 刪「brief §8 的 200 字」半句。`GLOSSARY.md`〈停鍵層模型〉的「語言模型指字級 bigram」改成同步層的 n-gram。model-ladder README 開頭連到票 09、10 的 README；emoji-keywords README 補 rustc 版本。
- 2026-11-12 沒處理的發現（含各票 Comments 留下的），依去處：
  - M4（網頁 prototype 照用到 M4）：
    - 停鍵層「同一串輸入碼的舊結果留快取」（plan §2.7，M3 之前就寫的）網頁沒做：退格回到同一串碼會再等停鍵、再跑一次模型。plan §2.7 已註明（code-review）。
    - `main.ts` 換模型 A→B 時，A 晚失敗會清掉 B 的 `loadedModel` 並誤報；模型載入與核對可搬進 `NeuralReranker`；`main.ts` 從 237 長到 379 行，`try … finally snapshot.free()` 五處（code-review）。
    - IndexedDB 打不開時 `start()` reject、網頁打不了字；現在只處理單筆壞資料（code-review）。
    - 簡體模式下同步層 bigram 的前文還沒轉回繁體（票 11e，plan §2.7 已寫）。
    - 聯想沒有 emoji 聯想、空輸入的句首建議、數字標點後的量詞聯想（票 13，黑箱 T8 的缺功能）：M4 開工拆票時評估。
  - M6（第一個平台殼）：
    - `qim-ffi` 的 `Engine::new` 用位置參數、直接 `with_word_trigram`，跳過 `Engine::open` 的 `WordTrigramMismatch` 核對，也沒接 emoji 資料包；改走 manifest 的 `open`（票 11b、16，plan §4.6，code-review）。到時一起刪只剩測試在用的 `Engine::from_pack_bytes`（code-review）。
    - `check_word_list` 的試算（複製 `UserModel` 匯入）與 `displacing_shortcuts` 的過濾在 `qim-wasm`：`qim-ffi` 加詞表匯入時搬進核心（`UserModel::check_word_list`），兩個綁定共用（code-review）。
    - 網頁英文直通每個空白、標點都整份存學習資料（票 15）；存檔方式由平台殼定。
    - 粵典詞庫在 App 內建還是另外下載、資料包頁「顯著」連回粵典官網（授權 §1(b)(ii)）（票 24，brief §11、plan §7.7）。
    - 線上更新的發版流程：`pack-release --previous` 要人記得給、manifest 不記詞級 trigram 訓練時用的詞庫 id 與版本（票 11b、20、21）；真的發版前定。
    - 匯入 macOS／iOS 文字替代（plist）成常用語（票 17）：殼要時另開票。
  - M7（iOS）：
    - 學過資料後每鍵 p95 變差（原生 43.5 → 218.6 µs）；低階機有問題時先做「只算不學習首選」或按碼快取（票 12）。
    - 放寬版詞級 trigram 峰值 +71 MB 推論是解開時的暫存，改成就地解開後可以再量（票 11b）。
  - 等使用者決定：
    - 建 GitHub release `packs`（plan §4.7，票 21）。沒建之前網頁的背景檢查每次 404、狀態列報「檢查更新失敗」（Spec 審查的第 6 條；失敗要說是票 21 的設計，有 e2e 守）。
    - 要不要去信粵典確認授權（票 18、24，brief §1.5、§12.1）。
    - 「叙携烟踪迹韵」當異體算簡體是從共用序號推論；「恒」照查證留在簡體表（使用者原本要移出），要改時在 `simplified.rs` 加例外（票 19）。
    - CLDR 關鍵字缺口語叫法（哈哈、開心）、HKCanCor 補的 383 詞混有數字與專名：等使用者回報再補或濾（票 16、18）。
  - 下次重訓停鍵層模型時（plan §2.7 的重訓流程）：λ 掃描擴到網格邊界 2 以上（票 11d）；`neural-import` 核對 `unk_penalty`（`reference.json` 濾掉了 `<unk>`，票 09）。
  - 不修，選定的語義或代價：
    - 無痕時用滑鼠把游標移到中文後面，數字後半形的布林仍是移動前的（開無痕、打 5、點到中文後面打 `.` 得半形）（Spec）：網頁無痕時不給前文，核心也不收，是票 05 照 brief §8「無痕不讀上下文」定的，連最後一字是不是數字也不看（`context.spec.ts`「無痕時不看前文的數字，只看自己送出去的字」守著）。改成讀這一個布林要動 brief §8 的硬限制，不改；plan §2.1 註明這個代價。
    - 放寬版詞級 trigram 列在網頁內建清單（Spec 審查當成範圍外）：票 11b 的 What to build 就寫「兩份都建、都進網頁的內建清單」，60 MB 上限只限 iOS 鍵盤擴充；每次載入多抓 14.56 MB 只在本機。
    - spec〈驗收〉「门排在所有同碼繁體字之後」：擴展區的字照 brief §4.1 排在所有一般字之後，「门」排在 14 個同碼的連打字集繁體字之後（`--verify`），不改。
    - 個人集門檻在看到 1.4M 差 2 題後改成只報不擋（票 25）：使用者決定，spec〈採用門檻〉與 progress 都已寫明。
    - 票 25 的 What to build 還寫「1.4M 未採用」：票是歷史快照，之後的決定在 progress 與 model-ladder。
    - `packs.ts` 的 `origin?` 與 `?? "imported"`：票 21 定的持久化格式不 migration。
    - `vite build` 的靜態頁沒有線上更新：網頁只在開發伺服器跑，平台殼直接讀 release（plan §4.7）。
    - `min_core_version` 寫建它的核心版本，偏保守：只會讓舊核心多拒一些資料包，不會載入不相容的（票 20）。
    - 23M、`deepseek-v4-pro` 沒再量、個人集沒送雲端（票 09、10、25）：成本超標或雲端不做；個人集不送是使用者決定。
    - 判斷題照各票或審查的理由不改：`quantize` 兩份、`parse_bigram`／`parse_string` 同段、wasm 七個 setter 同形、`raise`／`Spanned` 的名字、加一種資料包要改五處（Rust 的 match，可接受）、`imported` 的 Map 名、`PackSource` 的位置、`pipeline.rs`／`verify.rs` 過 1100 行與三份摘要欄位、`packing.rs` 四個 `*_sources`、`rerank-worker.ts` 自己的 `message`（worker 不載入 DOM 模組）、`packs.insert(2, …)`、`qim-bench` 七個 `*_items`（code-review）；`display.ts` 的名字、候選列與面板各存 `chinese`（票 01）；授權晚到才報、統計欄位四處、`built::find` 一種一份、`BuiltPack.name`（票 02）；`NotFinite` 的 `&'static str`（票 03）；句內前文沒獨立量、`CONTEXT_SUFFIXES`、句首一列同形（票 04）；核心與網頁都知道交還鍵作廢前文、上屏的字當前文沒開關（票 05）；研究筆記的重抄（票 06）；實驗程式的重複、`02acc67` 的 commit body 兩行 74 字元、`λ·(神經分−靜態二元分)` 沒做（票 07–10、25）；brief §1.5 的「不做」一級、同一結論寫在多處（票 11）；`Table` 兩份（票 11b）；文件寫死 9 與 50、翻頁多要一次（票 11c）；`neural-import` 與 `Reranker::new` 各查 λ（票 11d）；中文錯誤訊息、`rerank_request(&Engine)`、常見簡體字不轉、SIMD128 的瀏覽器下限（票 11e）；上屏中文後打數字要先 Esc（票 13）；觸發詞等於自己的碼時連計數刪（票 14）；201–254 字元的 email、共用 tick（票 15）；emoji 佔第一頁、`candidates` 兩個 bool（票 16）；連選兩份檔的競態沒測試（票 17）；換算值 300（票 18）；`--verify` 的 `NotSimplified` 不在 `check`（`--verify` 每次改資料管線都跑，plan §4.5）、多段路徑裡的簡體字照分數（票 19）；`sha256_hex` 兩份、`version` 靠人加（票 20）；兩份 `fetchOk` 等（票 21）；粵典詞庫的 `--check` 多錯 2 題、「實係」壓過「真係」、個人集 1 題（可選資料包，票 24）。審查提的 ADR 0005 補一句、名詞表補「模型階梯」「採用門檻」（只在 spec 與實驗用）、brief §8 與 plan §2.7、§8 的數字重抄（spec〈共同規則〉的分工）也不改。
- 2026-11-12 重驗在乾淨 checkout：`git worktree add --detach` 到 session 暫存目錄（`4bf1345`），`build`、`target`、`web/node_modules`、`bench/personal` 都不存在；照 `docs/development.md`〈安裝〉做 `npm ci --prefix web`（26 個套件）、`npx playwright install chromium`、確認 wasm-bindgen 0.2.130、`cargo xtask data`（共用下載快取，exit 0）、`cargo xtask check`；`bench/personal` 用 `ln -s` 接主 checkout 的個人集。另在 `2dbb7ea` 開一個 worktree（個人集複製一份，不動共用的）跑 `cargo xtask data`、`bench`、`bench --learning`，當 M3 開工時的數字。spec〈驗收〉逐項：
  - 網頁功能各有 Playwright 實打讀回：乾淨 checkout 的 `cargo xtask check` e2e `221 passed (52.0s)`、`2 skipped`（`rerank-latency.spec.ts`，設 `QIM_RERANK_LATENCY=1` 才量）。那次跑出的截圖挑 17 張存成 `.scratch/m3/evidence/23-clean-*.png`：前文（`context`）、詞級 trigram、神經重排、重排插到第 2 位（`rerank-insert`）、學習關、匯出再匯入、聯想、常用語、記住的 email、emoji、詞表匯入、`ls` 面板、資料包切換與刪除、線上更新、粵典詞庫、設定區；各票的截圖在同目錄。其中匯出再匯入、詞表匯入兩張原本取的是上屏後的空白畫面（兩張逐像素相同），本 session 的 code-review 後改取匯入前的確認畫面，來自封存後在本 worktree 跑的 e2e（`confirm-import.png`、`word-list-confirm.png`）。個人 bigram 的 e2e 與截圖是本票補的（上面 code-review）。
  - 首選字基準（乾淨 checkout，`bench --check`）：`bench 通過`、固定題 84/84、13 個集合都沒翻盤。「有前文」給前文比不給答錯與修正都少：合計答錯 1524 vs 1568、修正 5573 vs 5651，四批各自也是（yue-wiki 468/475、1738/1750；zh-wiki 477/494、1729/1751；common-voice-yue 187/195、594/610；hkcancor 392/404、1512/1540）。和 M3 開工時比（答錯／修正）：yue-wiki 1131/4467 → 941/3239、zh-wiki 1126/4505 → 979/3493、common-voice-yue 1032/4043 → 892/3249、hkcancor 663/2588 → 572/2148，都變好（詞級 trigram 為主）；個人集 1081/1433 → 1083/1463，答錯 +2、修正 +30（+2.1%），都在各票寫了理由並經使用者同意更新 baseline：票 04 修正 +13、票 16 emoji 佔第一頁 +1.4%、票 18 答錯 +1、票 11b 答錯 +1。
  - 雲端：網頁產品碼的 `fetch` 只有同源的資料包（`main.ts`、`packs.ts`、`pack-update.ts`）與 Vite 轉發資料包更新，`web/src`、`qim-core`、`qim-wasm` 沒有任何 LLM 端點或 API key；票 22 wontfix，重評條件寫在票上。
  - 階梯採用的項目：詞級 trigram 在 `bench --check` 的口語兩集合合計首選 35.7% → 44.5%（+8.8 點，門檻 +2.0）、修正 2.514 → 2.046 次/句（−18.6%，門檻 −3%）；`--verify` 預算版加 bigram 7,842,947 位元組 ≤ 8,000,000；每鍵原生 p95 73 µs、選開頭一段 p95 1.29 ms、載入 151.7 ms（同時有另一個 bench 在跑，仍在 1 ms／5 ms／300 ms 內）。神經重排：`neural-rerank.spec.ts`「預設開著神經模型：停鍵後把候選列沒有的路徑排到第 2 位，首選不動」與 `rerank.spec.ts` 綠燈，截圖 `23-clean-neural-rerank.png`（`hjtt` 第 2 位是插入的「光大實業」）。
  - 階梯結論：brief §1.5（「不做」一級）、§8（前文 50 字）、§11、§12.5 與 `docs/research/model-ladder.md` 一致；本票修了研究筆記與 plan 裡停在舊結論的幾處（上面 code-review）。
  - 個人模型：`bench --learning` 學習開的修正（乾淨 checkout vs M3 開工時）yue-wiki 3216 vs 4406、zh-wiki 3578 vs 4523、common-voice-yue 3108 vs 3983、hkcancor 1719 vs 2186、個人集 1390 vs 1375；公開集合合計 11621 vs 15098。個人集多 15，和學習關的 +30 同因（上一項），票 12 已定驗收看合計（使用者決定）。匯出 v2 再匯入等價由 `properties.rs` 的性質測試與 `user-dict.spec.ts`「匯出、清空、匯入後讀回相同」守，本票修了只剩 tick 的方案那個洞；v1 匯入由 `user_dict.rs` 的 `a_v1_file_imports_the_same_learning_without_bigrams_and_exports_as_v2` 等四例守（`cargo test` 綠燈）。
  - 繁體模式 `ls`：`simplified-characters.spec.ts` 五例綠燈，截圖 `23-clean-ls-panel.png`；`--verify` 真資料「门」在 14 個同碼繁體字與 3 條多段路徑之後（票 19）。
  - 資料包：`cargo xtask data --verify` 的「manifest 核對得過：11 份」；`packs.spec.ts`（匯入、切換、刪除、改過 sha256 被拒）與 `pack-update.spec.ts`（本機靜態伺服器背景更新、sha256 不符拒絕、關掉不連線、斷網照常）綠燈，截圖 `23-clean-packs-*.png`、`23-clean-pack-update.png`。
  - 粵語詞庫：brief §12.1 寫了 rime-cantonese 不用、粵典照非商業條款另建可選詞庫 `quick-lexicon-words-hk`；`--verify`「粵典詞庫是預設詞庫加粵典的詞：詞條 283824 → 308586」，預設詞庫的口語集如上變好。
  - `cargo xtask data --verify`：`verify 通過`，建兩次 sha256 相同（速成 `be151d674422…`、詞庫 `0d760a8d92c0…`、粵典詞庫 `356ffb6b0652…`、語言模型 `03bdd593e5d0…`、詞級 trigram `1c1d5ad0a275…`），語料四份轉碼 0 失敗、test 不洩漏進 train，停鍵層模型抽查把「今日天氣好好」排第一。
  - `cargo xtask check`（乾淨 checkout，`4bf1345`）：`==> fmt` … `==> e2e`、`221 passed`、`check 通過`，exit 0。本票之後的改動（`import` 的 tick、`pub(crate)`、`rerank.ts`、新測試、文件）由封存後在本 worktree 跑的 check 涵蓋（下一條）。
- 2026-11-12 匯出 v2 定稿：plan §2.4 寫明 M3 票 23 定稿，四種列（使用者詞與學習計數、個人 bigram、記住的字串、tick 行）與 v2 的詞欄跳脫就是 v2，之後加列的種類或改欄位升 v3；匯入照收 v1。本票修的 tick 不改格式（匯出本來就寫每個方案的 tick 行，是匯入漏收）。
- 2026-11-12 回報案例：使用者確認沒有網頁「回報」留下的案例，`bench/fixed.tsv` 不動。
- 2026-11-12 封存：M3 的 30 列 session 紀錄與 37 條偏差移到 `docs/archive/M3.md`；里程碑表 M3 連到封存檔、M4 進行中（拆票前讀加上本票 Comments）；plan 開頭的狀態行、§7 表 M3 完成、M4 進行中、M3 的範圍改成實際做的，§7.4 指向 spec、封存檔、model-ladder 與本票，§7 的「留給之後的發現」加上本票。
- 2026-11-12 封存後的檢查（本 worktree，含本票的修正與封存內容，提交前）：`cargo xtask check` `==> fmt` … `==> docs` … `==> e2e`、`222 passed (55.1s)`、`2 skipped`、`check 通過`；`cargo xtask bench --check` `bench 通過`、固定題 84/84、13 個集合都沒翻盤，答錯與修正逐格等於 baseline（本票只改匯入的 tick 與可見性，不動打分）。
- 2026-11-12 本 session 的 code-review（`4bf1345...acf6073`，兩軸）：封存完整（30 列、37 條逐行相同），里程碑表與 plan §7 一致。已處理：`properties.rs` 把加的常用語全刪光，常用語列（含跳脫）不再被匯出再匯入驗到，改成每條隨機刪或留；`phrases.rs`「空模型刪不到常用語」只驗匯出再匯入、沒驗模型不變，加 `assert_eq!(user, UserModel::default())` 先紅（`remove_phrase` 經 `for_scheme_mut` 建出空方案），`remove_phrase` 改成沒學過的方案直接回 `false`；個人 bigram 的 e2e 缺對照：加了「在「他」後面選一次，「我」後面名次不變」的對照後紅（同碼計數就把「我」後面的「你」從第 6 位抬到第 4 位，原測試證明不了 bigram），改成同一個測試開兩個乾淨的瀏覽器狀態，在「我」後面學的比在「他」後面學的排得前；變數 `afterWo` 改 `afterMe`、設前文抽成 `placeBefore`；plan §2.1 `CONTEXT_LIMIT` 那句改成只寫現況；本票「四處」改「五處」；兩張重複的截圖（上一條）。沒處理：`rerank.ts` 的 `.catch` 裡 `onResult` 自己再丟錯仍會變成 unhandled rejection：`onResult` 只是測試與延遲量測的掛勾，丟錯是呼叫端的錯。
- 2026-11-12 上一條修完後的 `cargo xtask check`：`222 passed (53.7s)`、`2 skipped`、`check 通過`。
