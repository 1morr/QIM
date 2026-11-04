# WebAssembly 每鍵延遲：Node 對 Chromium

回答的問題：`cargo xtask bench` 的 WebAssembly 每鍵延遲要用 Node 跑，還是用 Playwright 在瀏覽器跑（M1 票 05）。結論：用 Node，理由見下面〈結論〉與票 05 的 `## Comments`。

## 跑法

先跑過 `cargo xtask data` 與 `cargo xtask web`（綁定與資料包），在 repo 根目錄：

```sh
node scripts/experiments/wasm-latency-browser/compare.mjs
```

`compare.mjs` 用 `web/` 裝的 `@playwright/test` 開無頭 Chromium，以 `page.route` 在 `http://localhost:5199/` 供應 `web/src/wasm/` 的綁定與 `build/packs/quick/quick.pack`（不起伺服器），回應帶 COOP / COEP 讓頁面 `crossOriginIsolated`；照 `qim-bench` 的順序讀固定題與各語料，跑和 `web/bench/latency.ts` 相同的迴圈，印 p50 / p95。接著在 Node 跑同一個迴圈，兩邊各印整份工作量的總時間 ÷ 鍵數。

核心變重（連打、bigram）或換 Node、Chromium 主版本時重量。

## 上次結果（2026-11-04）

環境：Apple M6、macOS，Node v26.12.0、HeadlessChrome 156.0.8078.4（Playwright 1.64.1），綁定 release，M0 核心（只查碼表），6,631 個輸入、122,410 鍵；各跑三次。

| | p50 | p95 | 平均每鍵（含 Esc 與計時） |
| --- | ---: | ---: | ---: |
| Node（`cargo xtask bench`） | 0.21 µs | 1.75–1.83 µs | 0.45–0.47 µs |
| Chromium，`crossOriginIsolated` | 0.00 µs | 5.00 µs | 0.63–0.64 µs |
| Chromium，沒隔離（`http://bench.local`，非安全來源） | 0.00 µs | 0.00 µs | — |

## 結論

- Chromium 的 `performance.now()` 粗化：沒隔離時 100 µs，隔離後 5 µs。現在的核心每鍵不到 1 µs，瀏覽器裡的 p50 / p95 只剩 0 或一個刻度，前後對照看不出變化；Node 的解析度約 0.04 µs。
- 平均值 Chromium 約慢 1.4 倍，同是 V8，差在瀏覽器的額外開銷；兩者都比 50 ms 的預算小四個數量級以上。
- Node 已是開發需求，不用起瀏覽器與伺服器，整個 `cargo xtask bench` 約 4 秒。
- 風險：核心變重後兩者差距可能變大。票 12 驗延遲預算時重跑本實驗，Chromium 若明顯比 Node 慢，再把瀏覽器的數字加進報告。
