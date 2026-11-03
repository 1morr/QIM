# 05: 基準：每鍵延遲

Status: ready-for-agent
Blocked by: 02
讀: plan §2.3（beam、增量）、§5.2（指標）；brief §9；spec〈共同規則〉〈驗收〉（延遲預算）

## What to build

`cargo xtask bench` 量每次按鍵到 `Snapshot` 回來的時間，原生與 WebAssembly 各量一次，報 p50 / p95。票 12 的驗收（原生 ≤ 10 ms、WebAssembly ≤ 50 ms）與之後每張動核心的票都靠它看有沒有變慢。

- 原生：`qim-bench` 逐鍵計時，排除載入資料包。
- WebAssembly：用 `qim-wasm` 的同一份綁定量；用 Node 跑還是用 Playwright 在瀏覽器跑，在本票比較後定，理由寫進 `## Comments`。
- 題目：基準集合的句子之外，加一組 20 碼以上的長輸入（票 12 驗收要）。
- 延遲只報不擋（機器負載會飄）；`--check` 不比延遲，baseline 也不記。報告註明機器與 build profile。

## 驗收

- [ ] `cargo xtask bench` 印出原生與 WebAssembly 的每鍵 p50 / p95，含 20 碼以上那組（貼輸出）。
- [ ] 量測方式與機器寫在 `## Comments`；`docs/development.md` 寫怎麼跑。
- [ ] `--check` 不因延遲數字而紅（測試）。

## Comments
