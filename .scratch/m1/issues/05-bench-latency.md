# 05: 基準：每鍵延遲

Status: done
Blocked by: 02
讀: plan §2.3（beam、增量）、§5.2（指標）；brief §9；spec〈共同規則〉〈驗收〉（延遲預算）

## What to build

`cargo xtask bench` 量每次按鍵到 `Snapshot` 回來的時間，原生與 WebAssembly 各量一次，報 p50 / p95。票 12 的驗收（原生 ≤ 10 ms、WebAssembly ≤ 50 ms）與之後每張動核心的票都靠它看有沒有變慢。

- 原生：`qim-bench` 逐鍵計時，排除載入資料包。
- WebAssembly：用 `qim-wasm` 的同一份綁定量；用 Node 跑還是用 Playwright 在瀏覽器跑，在本票比較後定，理由寫進 `## Comments`。
- 題目：基準集合的句子之外，加一組 20 碼以上的長輸入（票 12 驗收要）。
- 延遲只報不擋（機器負載會飄）；`--check` 不比延遲，baseline 也不記。報告註明機器與 build profile。

## 驗收

- [x] `cargo xtask bench` 印出原生與 WebAssembly 的每鍵 p50 / p95，含 20 碼以上那組（貼輸出）。
- [x] 量測方式與機器寫在 `## Comments`；`docs/development.md` 寫怎麼跑。
- [x] `--check` 不因延遲數字而紅（測試）。

## Comments

- 2026-11-04 實作。`cargo xtask bench --check` 的延遲段（上面的正確率表與票 04 相同，各集合沒有翻盤，固定題 1/1，bench 通過）：

  ```text
  每鍵延遲（µs；只報不擋：`--check` 不比、baseline 不記）

  機器 Apple M6（macos aarch64）；固定題與各語料集合的每一題從空的組字逐鍵打、不選字，不含載入資料包。WebAssembly 用 web/src/wasm/qim_wasm.js（`cargo xtask bench` 以 release 編）。

  | 環境 | 輸入 | 輸入數 | 鍵數 | p50 | p95 |
  | --- | --- | ---: | ---: | ---: | ---: |
  | 原生（release） | 全部 | 6631 | 122410 | 0.04 | 1.13 |
  | 原生（release） | 第 20 碼起 | 2555 | 26851 | 0.04 | 0.04 |
  | WebAssembly（Node v26.12.0） | 全部 | 6631 | 122410 | 0.21 | 1.75 |
  | WebAssembly（Node v26.12.0） | 第 20 碼起 | 2555 | 26851 | 0.21 | 0.29 |
  bench 通過
  ```

  - 量法：固定題與四個語料集合的每一題（個人集不量，各機器才比得起來），從空的組字逐鍵打輸入碼、不選字，不含載入資料包。原生在 `qim-bench` 行程內每題用新的 `Session`，只計 `Session::handle`；WebAssembly 由 `qim-bench` 用 Node 跑 `web/bench/latency.ts`，載入網頁同一份綁定，每鍵計 `letter()` 加讀出輸入碼與候選，題與題之間按 Esc（不計時）。百分位數取 nearest-rank。機器與 build profile 由報告自動印出。跑法寫在 `docs/development.md`〈首選字基準〉的〈每鍵延遲〉。
  - 機器：Apple M6、macOS，Rust release，Node v26.12.0。M0 核心只查碼表，p50 已在計時解析度（約 0.04 µs）附近，離預算（原生 10 ms、WebAssembly 50 ms）很遠。
  - Node 還是 Playwright：選 Node。同一份綁定與工作量在無頭 Chromium 重跑（`scripts/experiments/wasm-latency-browser/`）：Chromium 的 `performance.now()` 沒隔離時粗化到 100 µs，加 COOP / COEP 隔離後也只有 5 µs，現在每鍵不到 1 µs，p50 / p95 只剩 0 或一個刻度；整份工作量平均每鍵 Chromium 0.63–0.69 µs、Node 0.45–0.48 µs，同是 V8，差約 1.4 倍，都遠低於預算。Node 已是開發需求，不必起瀏覽器與伺服器。票 12 驗預算時重跑那個實驗，Chromium 若明顯比 Node 慢再把瀏覽器的數字加進報告。
  - 長輸入（偏差）：沒有另外造題，取基準句子本身：輸入碼 ≥ 20 鍵的有 2,555 句，最長 86 鍵。而且這組只算每個輸入的第 20 鍵起（組字已有 19 碼再按的鍵），連打後段的延遲不會被前 19 鍵稀釋；code-review 時由「整句的每一鍵」改成這個口徑。
  - 單位用 µs：現在每鍵不到 1 µs，ms 印四位小數還是 0；票 12 對預算時 10 ms = 10,000 µs、50 ms = 50,000 µs。
  - 量不了延遲（沒有 Node、綁定編不起來、計時筆數和輸入對不上）時整個 bench 失敗，`--check` 也是：只報不擋指的是數字，量測壞了要看得到。`cargo xtask bench` 會先以 release 重編綁定，所以多了 `wasm-bindgen-cli` 與 Node 的需求（`check` 本來就要）。
  - 加了 `@types/node` 26.6.6（plan §1.4）給 `web/bench/tsconfig.json`，`npm run typecheck` 多檢查 `web/bench/`，腳本對綁定的型別由 `tsc` 把關。
  - 測試：`tests/run.rs` 全部改經一個每鍵回報 100 ms × 第幾鍵的假 WebAssembly 計時器，既有的 `--check`、`--update-baseline` 測試因此都在極大延遲下照樣通過；另有 `latency_reports_native_and_wasm_percentiles_with_a_long_input_group`、`check_and_update_baseline_ignore_latency`、`a_wasm_timer_that_misses_keys_is_an_error`，與 `tests/latency.rs` 的 nearest-rank 數值測試。
  - code-review 未處理：Node 端（`web/bench/latency.ts` 與 `qim-bench` 解析它輸出的 `parse_node_output`）沒有單元測試，依開工時和使用者說好的測試介面靠實跑 `cargo xtask bench` 驗；報告裡「WebAssembly 以 release 編」是固定文字，直接 `cargo run -p qim-bench` 用到舊綁定時不保證；`compare.mjs` 的計時迴圈與 `latency.ts` 重複，實驗腳本可接受。
