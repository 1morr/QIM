# 11e: 網頁：停鍵重排接上神經模型

Status: wontfix
Blocked by: 11c
讀: plan §2.7、§3.1、§4.6；brief §7（停鍵重排的開關）；`docs/research/model-ladder.md`；票 11c、11d 的 Comments；spec〈共同規則〉

## What to build

網頁照 ADR 0005 的停鍵層跑 1.4M：停鍵後在背景重排，結果最高到第 2 位。

- 在 Web Worker 裡跑 `qim-neural`（編成 WebAssembly；和 `qim-wasm` 同一份或另一份由本票定，plan §1.3 已允許 `qim-wasm` 依賴它），載入模型資料包。停鍵後把 `rerank_request` 交給 Worker，回來的名次經 `apply_rerank` 套用，過期的丟掉；打字不等它。
- 「重排」選單加「神經模型」並設成預設（brief §7：停鍵重排預設開）；假模式留給測試用。設定區的資料包清單列出模型資料包；沒有模型時不重排，打字與候選照常。
- 記下 WebAssembly 增加的大小與瀏覽器裡重排一次的 p50 / p95。
- 寫回 plan §3.1。

## 驗收

- [ ] Playwright 實打：打一句神經模型會把候選列沒有的路徑排到第 2 位的輸入碼，等停鍵後讀回第 2 位；關掉開關不再變動；重排回來前繼續打字，舊結果不套用；截圖存 `.scratch/m3/evidence/`。
- [ ] 沒有模型資料包時打字與候選照常（Playwright）。
- [ ] WebAssembly 大小與瀏覽器重排延遲記在 Comments。
- [ ] 實際操作網頁亂按一輪；`cargo xtask check` 綠燈。

## Comments

- 2026-11-11（票 11d）：wontfix。1.4M 在詞級 trigram 的新基線上口語前 2 名 +5.5 點，但個人集前 2 名 −0.53 點，超過門檻的 0.5 點；使用者選照門檻停下（`docs/research/model-ladder.md`〈停鍵層在新基線重量〉）。停鍵層與雲端都不做，本票的改動沒人用。
