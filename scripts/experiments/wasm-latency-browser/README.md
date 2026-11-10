# WebAssembly 每鍵延遲：Node 對 Chromium

回答的問題：`cargo xtask bench` 的 WebAssembly 每鍵延遲要用 Node 跑，還是用 Playwright 在瀏覽器跑（M1 票 05）。結論：用 Node，理由見下面〈結論〉與票 05 的 `## Comments`。

## 跑法

先跑過 `cargo xtask data` 與 `cargo xtask web`（綁定與資料包），在 repo 根目錄：

```sh
node scripts/experiments/wasm-latency-browser/compare.mjs
```

`compare.mjs` 用 `web/` 裝的 `@playwright/test` 開無頭 Chromium，以 `page.route` 在 `http://localhost:5199/` 供應 `web/src/wasm/` 的綁定與基準載入的四份資料包（速成、詞庫、bigram、英文；不起伺服器），回應帶 COOP / COEP 讓頁面 `crossOriginIsolated`；照 `qim-bench` 的順序讀固定題與各語料，跑和 `web/bench/latency.ts` 相同的迴圈（含「選開頭一段」那一鍵，見 `docs/development.md`〈每鍵延遲〉），印三組的 p50 / p95。接著在 Node 跑同一個迴圈，兩邊各印整份工作量的總時間 ÷ 鍵數。

2026-11-10（M2 票 07）起：迴圈改用目前的綁定 API（`inputChar`、`press("escape")`）、載入四份資料包、加選段那一鍵。下面〈上次結果〉的 2026-11-04 表是 M0 核心只載速成資料包時量的。

核心變重（連打、bigram）或換 Node、Chromium 主版本時重量。

## 上次結果（2026-11-10，M2 票 07）

環境：Apple M6、macOS 27.0.1，Rust 1.99.1，Node v26.12.0、HeadlessChrome 156.0.8078.4（Playwright 1.64.1），commit `2b49606` 加上票 07 的改動，綁定 release，M1 結束後的核心（詞庫、連打、bigram）加四份資料包，6,722 個輸入、123,330 鍵；跑兩次，列第一次（第二次的差在一個刻度內）。

| 環境 | 輸入 | 輸入數 | 鍵數 | p50 µs | p95 µs |
| --- | --- | ---: | ---: | ---: | ---: |
| Chromium，`crossOriginIsolated` | 全部 | 6722 | 123330 | 40.00 | 55.00 |
| Chromium | 第 20 碼起 | 2587 | 27142 | 45.00 | 55.00 |
| Chromium | 選開頭一段 | 6411 | 6411 | 275.00 | 890.00 |
| Node v26.12.0 | 全部 | 6722 | 123330 | 39.42 | 52.67 |
| Node | 第 20 碼起 | 2587 | 27142 | 44.54 | 55.83 |
| Node | 選開頭一段 | 6411 | 6411 | 276.38 | 887.04 |

平均每鍵（含選段、Esc 與計時）：Chromium 69.61 µs、Node 69.08 µs。核心變重後每鍵約 40 µs，Chromium 的 5 µs 刻度已看得出分佈，與 Node 差不到一個刻度：票 05 擔心的「核心變重後兩者差距變大」沒有發生，基準照舊用 Node。最重的選段那一鍵 p95 0.9 ms，仍遠低於 50 ms。

## 2026-11-04 的結果（M1 票 05）

環境：Apple M6、macOS，Node v26.12.0、HeadlessChrome 156.0.8078.4（Playwright 1.64.1），綁定 release，M0 核心（只查碼表），6,631 個輸入、122,410 鍵；各跑三次。

| | p50 | p95 | 平均每鍵（含 Esc 與計時） |
| --- | ---: | ---: | ---: |
| Node（`cargo xtask bench`） | 0.21 µs | 1.75–1.83 µs | 0.45–0.47 µs |
| Chromium，`crossOriginIsolated` | 0.00 µs | 5.00 µs | 0.63–0.64 µs |
| Chromium，沒隔離（`http://bench.local`，非安全來源） | 0.00 µs | 0.00 µs | — |

## 結論（2026-11-04，選 Node 的理由；2026-11-10 的重量見上）

- Chromium 的 `performance.now()` 粗化：沒隔離時 100 µs，隔離後 5 µs。現在的核心每鍵不到 1 µs，瀏覽器裡的 p50 / p95 只剩 0 或一個刻度，前後對照看不出變化；Node 的解析度約 0.04 µs。
- 平均值 Chromium 約慢 1.4 倍，同是 V8，差在瀏覽器的額外開銷；兩者都比 50 ms 的預算小四個數量級以上。
- Node 已是開發需求，不用起瀏覽器與伺服器，整個 `cargo xtask bench` 約 4 秒。
- 風險：核心變重後兩者差距可能變大。票 12 驗延遲預算時重跑本實驗，Chromium 若明顯比 Node 慢，再把瀏覽器的數字加進報告。M2 票 07 重量：沒有變大。
