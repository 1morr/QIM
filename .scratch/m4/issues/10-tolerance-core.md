# 10: 容錯進核心

Status: ready-for-agent
Blocked by: 09
讀: brief §4.8、§7；plan §2.1（`Composition`、分段、Enter）、§2.2、§2.4、§2.7；spec〈細節〉〈容錯〉；票 09 的結論；spec〈容錯採用門檻〉〈共同規則〉

## What to build

照票 09 採用的修正與懲罰，讓打錯一碼的詞與句照樣讀得出來，原樣讀得通的照原樣排：

- 容錯邊帶懲罰進 lattice；`Options` 加容錯開關，預設開（brief §7），網頁設定表加一列。
- 票要定、寫回 plan：§2.1「各分段的碼接起來就是 `input`」的不變式怎麼改寫、分段顯示哪個字根、學習記修正後的碼、Enter 原樣上屏哪一串。
- 停鍵重排只看沒修正的路徑，或驗收加「沒打錯的集合重排後前 2 名不降」：兩種都量，比較後定，寫進 plan §2.7。
- debug 面板標出哪一段是修正、修正的種類。

## 驗收

- [ ] `bench --check`：打錯集合達〈容錯採用門檻〉、沒打錯集合不退步、改錯比例在上限內；`--update-baseline` 寫理由；每鍵延遲（空白與學過）在上限內。
- [ ] 「原樣正確卻被改錯」的比例接上容錯開關（`crates/qim-bench/src/score.rs` 的 `literal_options` 關掉容錯），bench 報告量到實際數字，並照〈容錯採用門檻〉的上限擋（`--check`）。
- [ ] 每鍵延遲照殼送的按鍵機率量：`crates/qim-bench/src/latency.rs` 的原生計時與 WebAssembly 計時腳本（`inputChar`）現在送單一字母，容錯進核心後要改送實體鍵的按鍵機率（同正確率那邊的 `qim_bench::physical_key`、網頁的 `pressCode`），否則閘門量的是比較便宜的路徑（票 08 的 spec 審查）。
- [ ] `model-ladder` 的 `neural --split test` 與 `compare` 結果附在 Comments。
- [ ] Playwright：打錯碼的詞讀到修正讀法（面板標修正）；關掉容錯後打錯碼不出修正讀法；原樣讀得通的照原樣排首選。
- [ ] plan §2.1、§2.4、§2.7 與 brief §4.8 依決定改寫；名詞表加詞；`cargo xtask check` 綠燈。
