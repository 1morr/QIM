# 01: debug 面板

Status: done
Blocked by: None (can start immediately)
讀: plan §2.1（`Session`、`Snapshot`、`rerank_request`）、§2.3、§2.7、§3.1；spec〈範圍〉的第一期前置、〈共同規則〉

## What to build

網頁加一個可開關、預設關的 debug 面板，打字時看得到核心為什麼這樣排（使用者 2026-11-12 要求；之後各票的測試都用它讀回原因）：

- 殼給核心的前文，與核心是否「不知道前文」；無痕時註明不讀。
- 輸入碼的切段；首選的路徑分數與學習加分。
- 每個候選的來源：詞庫、使用者詞、個人 bigram、英文、記住的字串（票 14 換成英文使用者詞時面板跟著改）、emoji、重排插入。
- 停鍵重排的要求（前文、各路徑）與結果（各路徑分數、選了哪條、毫秒、套用或作廢）。

資料由只給網頁用的查詢給（例如 `Session::inspect()`，名字由票定），不進 `Snapshot`；`qim-ffi` 不用加。重排的模型分數與毫秒在網頁那邊記。面板關著時不呼叫這個查詢，每鍵不多花時間。

## 驗收

- [x] Playwright：預設關；打開後讀得到前文的三種狀態（有前文、不知道前文、無痕不讀）、切段、首選的路徑分數與學習加分。
- [x] Playwright：候選來源每一種都至少讀回一例。
- [x] Playwright：停鍵後讀得到重排的要求與結果；用 `?rerank-delay` 讀到作廢。
- [x] `Snapshot` 沒加欄位；面板關著時不呼叫查詢（測試守）。
- [x] plan §3.1 加一段說明面板；實際操作亂按一輪、截圖存 `.scratch/m4/evidence/`；`cargo xtask check` 綠燈。

## Comments

- 2026-11-12 介面：核心 `Session::inspect(&Engine, &UserModel) -> Inspection`（`crates/qim-core/src/session/inspect.rs`），不進 `Snapshot`、`qim-ffi` 沒加；`qim-wasm` 的 `Session.inspect()` 攤平成字串種類（綁定只轉換列舉、不解讀）。給：前文三態 `ContextState`、組字開頭接什麼 `PrecedingChar`（句首、某字、unigram）與前一個字詞、切段、每個候選的 `CandidateKind`、各段 `SegmentSource`（碼表、詞庫、使用者詞）與學習計數、個人 bigram 旗標、路徑分與學習加分、排位規則編號、重排放上來的路徑序號（插入或移上）。重排的模型總分改由 `qim_neural::Reranker::rank_scored` 一併給（`rank` 照舊只給名次），網頁 Worker 回傳 `totals`。
- 2026-11-12 動到核心的地方：`Engine::edge_score` 拆成 `edge_score_parts`（沒學習時的分數、`personal`），lattice 的 `Edge` 多一個 `source`；`CandidateSpan::pinned_after_top` 換成 `rule: Option<u8>`（`raise` 看 `rule.is_some()`，第 0 位不在 `[1..]` 裡，和原本 `p > 0` 等價）。`Engine::inspect_path` 照 `Lattice::push` 逐段重算（前一字、詞級 trigram 的前兩段、個人 bigram）；同一段同文字有幾條邊時取分數最高的，和 lattice 一樣（同前綴節點的同文字延伸只留最高分），測試 `a_candidate_scores_like_its_lattice_path…` 比對路徑分與 `Session::paths` 相同。
- 2026-11-12 票外但為了「為什麼這樣排」加的：排位規則編號、組字開頭接什麼與前一個字詞、聯想的種類、重排「移上」（候選列原本就有、從後面移上來）。
- 2026-11-12 測試：`crates/qim-core/tests/inspect.rs` 11 個（前文三態、來源、使用者詞、路徑分 = lattice 路徑分、學習加分與個人 bigram、開頭接什麼、排位規則、重排插入、聯想、切段、`Snapshot` 不帶 debug 欄位）。`Snapshot` 用不帶 `..` 的解構守：變異加一個欄位（lib 照編得過）時這個測試編譯不過（E0027）。`qim-neural` 加 `rank_scored` 的測試；`web/test/` 加來源標記與 `onRequest` 的單元測試；`web/e2e/debug-panel.spec.ts` 7 個，連跑三次都綠。「關著時不問」的守法：變異拿掉 `draw()` 的 `open` 判斷，e2e 紅（`data-inspect-count` 變成非 0）；它守的是面板自己，網頁只有 `debug-panel.ts` 呼叫 `inspect()`。
- 2026-11-12 實際亂按（桌面 1280 寬、手機 412 寬觸控鍵盤；組字中夾 ←、退格、數字選字、單按 Shift 英文直通打 email、分隔記號、標點、Esc、合成 keydown 快速連按、切簡體、無痕、重排「最後一條提前」）：找到一個問題，候選多的碼（`hk` 三百多個）把面板撐到上萬 px，手機上鍵盤與設定被推遠；先寫會紅的 e2e（高 8411 > 450）再修成面板最高半個畫面、在裡面捲。截圖：`.scratch/m4/evidence/01-debug-panel-composing.png`、`01-debug-panel-incognito-simplified-rerank.png`、`01-debug-panel-touch.png`。console 只有兩個本來就有的 404（favicon、沒起本機 release 伺服器時的 `pack-updates/index.json`）。
- 2026-11-12 觀察（不改）：組字從開無痕之前就開始時，面板顯示「無痕：不讀前文」但開頭接「句首」：lattice 是開無痕前建的，`set_options` 只丟前文、不重建已在打的組字。是核心現有行為（brief §8 的「無痕不讀前文」在下一次組字才完全生效），要改另開票。
- 2026-11-12 `bench --check` 前後對照（前 = 開工的 `1cf2f1c`，`git archive` 到 scratchpad 另編；後 = 本票；同一份資料包，sha256 前 12 碼都相同；個人集用主 checkout 的軟連結）：延遲與載入以外的整份報告逐行相同，兩次都 `bench 通過`、固定題 84/84、每個集合「沒有翻盤」。答錯／修正（前 = 後）：yue-wiki 941／3239、zh-wiki 979／3493、common-voice-yue 892／3249、hkcancor 572／2148、yue-wiki·前文 468／1738、zh-wiki·前文 477／1729、common-voice-yue·前文 187／594、hkcancor·前文 392／1512、personal 1083／1463。`--learning` 前後也逐行相同（學習開修正：yue-wiki 3216、zh-wiki 3578、common-voice-yue 3108、hkcancor 1719、personal 1390）。停鍵層的輸入（`rerank_request`）沒動，不跑 `model-ladder`。
- 2026-11-12 每鍵延遲前後對照：在高負載下交替量（協調者指示：別的 session 的 Android 模擬器與並行的票讓負載降不下來，12 核的 load average 6–33）。A = 開工的 `1cf2f1c`、B = 本票，照 A B A B A B 交替，空白與學過各三輪，每次記下開跑時的 1／5／15 分鐘 load average。空白 = `cargo xtask bench` 的延遲表（原生 release、WebAssembly Node v26.12.0）；學過 = 一次性的範例（沒進 repo，量法：同一份 `UserModel` 依序打 `latency_inputs` 的每題，每 3 題選第 2 個候選、其餘按空白直到上屏，之後每題用它的複本逐鍵量 `Session::handle`，原生；票 05 會做成正式量法）。第 3 輪第一次跑時和沒停掉的背景迴圈重疊，那幾筆丟掉、單獨重跑。p50 / p95，µs：

  | 輪 | 組 | load | 原生全部 | 原生第 20 碼起 | 原生選開頭一段 | Wasm 全部 | Wasm 選開頭一段 |
  | --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
  | 1 | 空白 A | 13.2 / 18.3 / 18.4 | 51.6 / 94.0 | 55.9 / 96.3 | 423 / 1479 | 68.6 / 124.1 | 554 / 2092 |
  | 1 | 空白 B | 14.0 / 17.6 / 18.1 | 62.8 / 215.3 | 72.9 / 250.3 | 556 / 3406 | 66.2 / 119.0 | 529 / 1973 |
  | 2 | 空白 A | 6.6 / 16.4 / 18.1 | 59.0 / 96.2 | 66.4 / 100.8 | 464 / 1651 | 72.1 / 123.9 | 585 / 2163 |
  | 2 | 空白 B | 7.2 / 14.5 / 17.3 | 52.8 / 93.8 | 58.9 / 101.8 | 418 / 1562 | 63.9 / 107.3 | 505 / 1756 |
  | 3 | 空白 A | 6.7 / 13.2 / 16.7 | 55.3 / 95.0 | 60.1 / 98.0 | 445 / 1555 | 94.8 / 190.7 | 775 / 4271 |
  | 3 | 空白 B | 21.6 / 14.8 / 16.9 | 53.6 / 87.6 | 58.8 / 91.4 | 421 / 1483 | 66.6 / 109.7 | 523 / 1865 |
  | 1 | 學過 A | 17.6 / 18.5 / 18.4 | 85.8 / 226.8 | 90.0 / 1064.4 | 648 / 2244 | — | — |
  | 1 | 學過 B | 32.6 / 24.2 / 20.7 | 75.8 / 188.4 | 79.7 / 903.5 | 566 / 1961 | — | — |
  | 2 | 學過 A | 6.9 / 12.8 / 16.4 | 92.3 / 465.1 | 95.4 / 1200.5 | 779 / 3807 | — | — |
  | 2 | 學過 B | 14.2 / 15.6 / 16.9 | 111.4 / 582.0 | 115.6 / 1576.5 | 956 / 5205 | — | — |
  | 3 | 學過 A | 11.0 / 13.5 / 16.2 | 106.0 / 383.0 | 110.4 / 1379.0 | 840 / 3516 | — | — |
  | 3 | 學過 B | 23.2 / 18.3 / 17.6 | 74.0 / 185.0 | 77.7 / 886.8 | 552 / 1912 | — | — |

  判斷：前後差在雜訊內、沒有一致的方向。學過的原生全部 p95，B 比 A：0.83×、1.25×、0.48×（兩輪快、一輪慢）；空白原生全部 p95：2.29×、0.98×、0.92×（第 1 輪 B 的 215 µs 是單次離群，後兩輪沒再出現）；Wasm 全部 p95 三輪 B 都不比 A 慢。同一輪內 A、B 差到兩倍的都有，量級來自負載，不是本票。機器閒時重量一次，留給票 05 的延遲閘門（它量空白與學過兩組、會擋）。
- 2026-11-12 code-review（`1cf2f1c` 起，Standards 與 Spec 兩軸）。已處理：綁定裡解讀 `CLAUSE_START` 的邏輯搬回核心（`PrecedingChar`）；名詞表補「debug 面板」；TS 的種類改字面值聯集、表頭與 `free()` 抽 helper；「選了哪條」改在結果回來當下記下（之後翻頁、打字不消失，e2e 守），`unchanged` 註明意思、名次欄改名「模型名次」；「關著時不問」的測試名稱與註解照實寫守的範圍。沒處理（判斷）：`rule` 用 1–5 的數字不另立列舉（plan §2.4 本來就以編號稱規則，呼叫處有註解）；`inspect_path` 與 `Lattice::push` 的逐段前後文平行一份（抽共用要動每鍵路徑，現由路徑分相同的測試守）；`ScoredSegment` 與 `InspectedSegment` 欄位重疊（前者是 `pub(crate)` 的計算結果，後者是對外的形狀）。
- 2026-11-12 留給之後的票（不在本票範圍）：`web/e2e/learning.spec.ts:153` 每次跑 e2e 都改寫已完成里程碑的 `.scratch/m3/evidence/23-personal-bigram.png`，工作樹因此出現不屬於本票的改動；本票沒有 commit 它。
- 2026-11-12 驗證：`cargo xtask check` 全綠（e2e 229 passed、2 skipped、`check 通過`）；`npm test` 51 passed。新 worktree 先 `cargo xtask data`、`npm ci --prefix web`。

Session 紀錄：debug 面板：核心 `Session::inspect`（不進 Snapshot）、網頁面板與重排模型總分；bench、--learning 前後逐行相同，延遲高負載下交替量在雜訊內
