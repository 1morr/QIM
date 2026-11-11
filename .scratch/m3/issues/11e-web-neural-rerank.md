# 11e: 網頁：停鍵重排接上神經模型

Status: done
Blocked by: 11c
讀: plan §2.7、§3.1、§4.6；brief §7（停鍵重排的開關）；`docs/research/model-ladder.md`；票 11c、11d 的 Comments；spec〈共同規則〉

## What to build

網頁照 ADR 0005 的停鍵層跑 1.4M：停鍵後在背景重排，結果最高到第 2 位。

- 在 Web Worker 裡跑 `qim-neural`（編成 WebAssembly；和 `qim-wasm` 同一份或另一份由本票定，plan §1.3 已允許 `qim-wasm` 依賴它），載入模型資料包。停鍵後把 `rerank_request` 交給 Worker，回來的名次經 `apply_rerank` 套用，過期的丟掉；打字不等它。
- 「重排」選單加「神經模型」並設成預設（brief §7：停鍵重排預設開）；假模式留給測試用。設定區的資料包清單列出模型資料包；沒有模型時不重排，打字與候選照常。
- 記下 WebAssembly 增加的大小與瀏覽器裡重排一次的 p50 / p95。
- 寫回 plan §3.1。

## 驗收

- [x] Playwright 實打：打一句神經模型會把候選列沒有的路徑排到第 2 位的輸入碼，等停鍵後讀回第 2 位；關掉開關不再變動；重排回來前繼續打字，舊結果不套用；截圖存 `.scratch/m3/evidence/`。
- [x] 沒有模型資料包時打字與候選照常（Playwright）。
- [x] WebAssembly 大小與瀏覽器重排延遲記在 Comments。
- [x] 實際操作網頁亂按一輪；`cargo xtask check` 綠燈。

## Comments

- 2026-11-11 曾隨票 11d 的重量結果 wontfix（`30a043c`）；使用者把模型階梯的「個人集不退步」改成只報不擋後 1.4M 採用，票 25 revert 那個 commit，本票恢復。
- 2026-11-12 實作（開工 `3851966`）：
  - WebAssembly 用同一份 `qim-wasm`（plan §1.3 已允許依賴 `qim-neural`）：加 `Reranker`（由模型資料包建，`rank(前文, 路徑, 路徑分)` 回序號），Web Worker（`web/src/rerank-worker.ts`）和主執行緒各自一個實例。不另建 crate：省一份建置與 `RULES`，代價是主執行緒也下載 candle 的碼（見下面的大小）。11d 留的「`Reranker::new` 四個參數要不要包型別」：綁定裡只有 `WebReranker::new` 從 `ModelPack` 的欄位呼叫一次，不包。
  - 排程（`web/src/rerank.ts` 的 `rerankScheduler`）：停鍵 80 ms（青簡在 macOS 的值，brief §12.4）後取 `rerankRequest`；同時只有一個在算，算的時候又停鍵了就記著、回來後再取最新的；過期的由核心丟掉。假實作（原樣、最後一條路徑提前）改走同一條路、留給測試。
  - 模型資料包（11d 的發現）：`sessionPacks()` 排除種類 `model`，`modelPack()` 另給 Worker；換資料包時主執行緒先建一次 `Reranker` 核對，不收就照 plan §4.6 退回。內建清單加 `zh-hant-neural-rerank`，列為可缺：沒有時不重排、打字照常。
  - 簡體模式的前文（11c 的發現，使用者選「核心逐字轉回繁體」）：`Session::rerank_request(&Engine)` 在簡體模式把前文逐字轉回繁體，表由 `Simplifier::traditional_chars` 反查簡繁資料包的轉換表，一對多取 `LanguageModel::unigram`（KN continuation）最高的，同分取繁體寫法。只看單字：「头发」→「頭發」、「干了」→「幹了」。同步層 bigram 的前文沒改（plan §2.7 註明）。建表在 `with_simplifier`，原生約 1.2 ms。
  - SIMD128：`.cargo/config.toml` 給 `wasm32-unknown-unknown` 開 `+simd128`（candle 的 q8 內積有 simd128 實作）。合成量法 p50 89 → 27 ms。票只要求記下延遲，這是自己加的：瀏覽器下限因此是 Chrome 91、Firefox 89、Safari 16.4（brief §12.5 附來源）。
- WebAssembly 大小（`qim_wasm_bg.wasm`，wasm-bindgen 後，兩邊都開 SIMD128）：`3851966` 5,328,491 B（gzip 1,411,804、brotli 895,448）→ 7,257,991 B（gzip 1,824,748、brotli 1,109,505），多 1,929,500 B（1.84 MB；gzip 多 0.40 MB）。沒開 SIMD 時是 5,381,697 → 7,183,797。
- 瀏覽器重排延遲（Chrome，Playwright 的 Desktop Chrome，Apple M6，load average 約 3–4；`QIM_RERANK_LATENCY=1 npm run --prefix web e2e -- e2e/rerank-latency.spec.ts`）：
  - 合成（照 `qim-neural` 的 `latency` example：9 條 × 18 字、前文 50 字，熱身 20 次後 500 次，主執行緒）：p50 27.1 ms、p95 27.4 ms、最大 27.7 ms，載入 3.9 ms。沒開 SIMD：p50 88.8、p95 89.5 ms。原生（2 條執行緒）p95 約 6 ms。
  - 實打（口語集 hkcancor 前 200 題照順序打，讀 Worker 回報的毫秒數）：p50 25.4 ms、p95 44.7 ms、最大 60.5 ms（輸入碼長度 p50 12、p95 29）。沒開 SIMD：p50 58.9、p95 97.3 ms。
- Playwright（`web/e2e/neural-rerank.spec.ts`）：`fukjctd` 首選「光大額葉」不動，第 2 位插入全部候選裡沒有的「光大實業」（口語集的正解），按 2 照它上屏；關掉不再變動；`?rerank-delay=1500` 下送出要求後退格再打回同一碼，舊結果過期（`data-rerank-stale`）、第 2 位不變，新的要求照常套用；擋掉內建模型時打字與候選照常；從檔案匯入模型資料包後照它重排。變異：殼改用最新的 `generation` 套用，「舊結果不套用」那條紅。截圖 `.scratch/m3/evidence/11e-neural.png`、`11e-stale.png`、`11e-no-model.png`。單元測試（`web/test/rerank.test.ts`）用假核心驗停鍵計時、同時只算一個、關掉與沒有模型；拿掉「算的時候又停鍵」的處理各有一條紅。
- 亂按：固定種子 500 步與 900 步兩輪（字母、空白、退格、Esc、Enter、方向鍵、翻頁、Shift、Tab、數字鍵、標點，夾雜切重排四種模式、簡體、停鍵 200–600 ms），沒有頁面錯誤與 console 錯誤，78 次停鍵前後首選都沒變；截圖 `11e-monkey.png`，腳本沒進 repo。
- `bench --check`（開工 `3851966`；本票改了 `qim-core` 的 `rerank_request` 簽名與簡繁的反查表，沒改打分，bench 不載簡繁資料包）：通過，12 個集合答錯／修正都等於 baseline（yue-wiki 941／3239、zh-wiki 979／3493、common-voice-yue 892／3249、hkcancor 572／2148，有前文給 468／477／187／392），沒有翻盤，固定題 84/84；本機沒有個人集。開工的 commit 在另一個 worktree 重跑 `bench --check` 對照：各集合與依長度分組的表逐行相同。WebAssembly 每鍵延遲（全部）p50／p95 58.1／79.8 µs（沒開 SIMD）→ 57.4／78.6 µs，載入 185.4 → 178.5 ms（只報不擋）。
- code-review（`3851966` 起，Standards 與 Spec 兩軸）。已修：
  - Worker 起不來（綁定載入失敗）或出錯時沒有回覆，`rerankScheduler` 會一直等、之後連假實作也不再重排：Worker 端把 `await ready` 包進 `try` 回 `failed`，主執行緒接 `onerror`／`onmessageerror` 讓等著的都失敗、之後的要求直接失敗；`failed` 統一在 `send` 轉成丟錯。單元測試補「模型丟錯記成沒有結果、之後照常要求」。
  - 換資料包時原本先把新模型交給 Worker 再 `setPacks`：其他資料包被核心拒收時 Worker 已換了；改成先核對模型、核心收下後才交給 Worker，Worker 載入失敗時清掉記錄，下次再試。
  - `reorderer` 改名 `rerankScheduler`（名詞表的「重排」在程式碼叫 rerank）；假實作的模式收成 `FakeMode`、`rerankMode` 查一張表；Worker 收的要求不帶用不到的 `generation`；`LanguageModel::unigram` 收成 `pub(crate)`；補 `traditional_chars` 的單元測試（簡體字本身比各寫法常見時不轉，如「后」；同分取碼位小的）。
  - e2e 補「套用後再關、送出要求後才關」（變異：拿掉等待中關掉的判斷會紅）；bench 補依長度分組的前後對照（上一條）。
- 沒處理：
  - `console.error("重排失敗", …)` 與 `JsError` 的訊息是中文：repo 一直用中文的 log 與錯誤訊息（專案慣例優先）。
  - `Session::rerank_request(&Engine)` 只為轉前文多收 `Engine`：和 `apply_rerank` 一致。
  - 簡體字本身比繁體寫法常見時不轉（「后」「干」可能留著）：使用者選的是「一對多取 unigram 最高的」，這裡把簡體字本身也算進比較，是延伸。
  - 「啟動時模型還沒載完，那次停鍵算用掉」：Worker 照收到的順序處理，載入的訊息在任何重排之前送出，重排一定等到載入完；不會發生。
