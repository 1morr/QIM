# 05b: qim-bench 的 check 測試不受機器負載影響

Status: ready-for-agent
Blocked by: None (can start immediately)
讀: 票 05 的 Comments（「`run.rs` 的 `Mode::Check` 測試現在也受真實的原生計時擋」那條）；`docs/development.md`〈每鍵延遲〉；spec〈每鍵延遲上限〉

## What to build

協調者插入的修復票（2026-11-12）。票 05 讓 `cargo xtask bench --check` 擋原生每鍵 p95 後，`crates/qim-bench/tests/run.rs` 裡走 `Mode::Check` 的測試也用真實的原生計時：debug 建置、小資料包平常 p95 約 40–70 µs，但機器負載高時量到 1.45–1.50 ms，超過 1 ms 上限而紅。票 06 的 session 跑 `cargo xtask check` 四次紅了兩次（只紅在這裡，單獨重跑會過）；整批並行跑票時機器常在高負載，會反覆擋住每張票。

- 先分清是產品的問題還是測試的問題：產品的閘門（`cargo xtask bench --check` 對真實資料包）照票 05 的設計保留真實計時與「負載低時單獨重跑」；要修的是單元／整合測試不該依賴真實計時。
- 先寫出會紅的 repro：在高負載下（例如另開數個佔滿 CPU 的程序）跑 `cargo test -p qim-bench --test run`，記下紅的測試名與量到的數字；或用注入的慢計時器證明現在的測試會因延遲違規而紅。
- 修法：`run.rs` 的 check 測試改用假計時器（票 05 已有 `native_timing_with` 一類的注入點），只在專門測延遲閘門的測試裡用固定數字驗「超過會紅、在內不紅」；不靠拉長上限或逾時。說不出原因、只放寬數字的修法不收。

## 驗收

- [ ] repro 與紅的輸出貼在 Comments（測試名、負載、量到的數字）。
- [ ] `crates/qim-bench/tests/` 沒有測試依賴真實的原生計時；高負載下連跑 `cargo test -p qim-bench` 三次都綠，輸出貼在 Comments。
- [ ] 延遲閘門的判斷仍有雙向測試（假計時器：超過任一格紅、在上限內不紅），`cargo xtask bench --check` 對真實資料包照舊擋。
- [ ] `cargo xtask check` 綠燈。
