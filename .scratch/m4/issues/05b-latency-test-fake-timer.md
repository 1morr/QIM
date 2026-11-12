# 05b: qim-bench 的 check 測試不受機器負載影響

Status: done
Blocked by: None (can start immediately)
讀: 票 05 的 Comments（「`run.rs` 的 `Mode::Check` 測試現在也受真實的原生計時擋」那條）；`docs/development.md`〈每鍵延遲〉；spec〈每鍵延遲上限〉

## What to build

協調者插入的修復票（2026-11-12）。票 05 讓 `cargo xtask bench --check` 擋原生每鍵 p95 後，`crates/qim-bench/tests/run.rs` 裡走 `Mode::Check` 的測試也用真實的原生計時：debug 建置、小資料包平常 p95 約 40–70 µs，但機器負載高時量到 1.45–1.50 ms，超過 1 ms 上限而紅。票 06 的 session 跑 `cargo xtask check` 四次紅了兩次（只紅在這裡，單獨重跑會過）；整批並行跑票時機器常在高負載，會反覆擋住每張票。

- 先分清是產品的問題還是測試的問題：產品的閘門（`cargo xtask bench --check` 對真實資料包）照票 05 的設計保留真實計時與「負載低時單獨重跑」；要修的是單元／整合測試不該依賴真實計時。
- 先寫出會紅的 repro：在高負載下（例如另開數個佔滿 CPU 的程序）跑 `cargo test -p qim-bench --test run`，記下紅的測試名與量到的數字；或用注入的慢計時器證明現在的測試會因延遲違規而紅。
- 修法：`run.rs` 的 check 測試改用假計時器（票 05 已有 `native_timing_with` 一類的注入點），只在專門測延遲閘門的測試裡用固定數字驗「超過會紅、在內不紅」；不靠拉長上限或逾時。說不出原因、只放寬數字的修法不收。

## 驗收

- [x] repro 與紅的輸出貼在 Comments（測試名、負載、量到的數字）。
- [x] `crates/qim-bench/tests/` 沒有測試依賴真實的原生計時；高負載下連跑 `cargo test -p qim-bench` 三次都綠，輸出貼在 Comments。
- [x] 延遲閘門的判斷仍有雙向測試（假計時器：超過任一格紅、在上限內不紅），`cargo xtask bench --check` 對真實資料包照舊擋。
- [x] `cargo xtask check` 綠燈。

## Comments

- 2026-11-12 repro（修之前，`feb4178`）：另開 48 個佔滿 CPU 的 shell 迴圈（12 核，load average 22.37），`cargo test -p qim-bench --test run` 連跑三次紅兩次，只紅在原生延遲：
  - 第 1 次：`latency_reports_native_and_wasm_percentiles_for_blank_and_learned_user_data` 在 `run.rs:1165` 紅（原生列的 p95 超過上限，報告列尾多了「，超過」）。`59 passed; 1 failed`。
  - 第 3 次：`check_fails_when_a_pack_needs_more_corrections_with_the_same_wrong_count`（`run.rs:387`，多了一條 `每鍵延遲超過上限：原生、學習資料空白、全部 的 p95 10821 µs > 1000 µs`）與 `colloquial_sets_run_after_the_wiki_sets_and_need_a_baseline`（`run.rs:882`，`… 全部 的 p95 2874 µs > 1000 µs`）。`58 passed; 2 failed`。
  - 單獨跑 `latency_reports…` 四次都綠：同一個測試執行檔裡其他測試並行時才會擠到。
- 2026-11-12 修法：`qim_bench::run` 的計時器參數改成 `Timers { native, wasm }`（`NativeTimer` 與 `WasmTimer` 成對）。CLI（`main.rs`）給真實的 `native_timing_with` 與 `node_wasm_timer`；`run.rs` 給 `fast_native`（每鍵 0.05 ms、選開頭一段 0.5 ms），慢的 `fixed_native(2.0, 0.5)` 只在閘門測試用。假計時器的形狀（每鍵一格、哪些輸入量選開頭一段）由 `fake_timing` 照 `lefts_to_first_segment` 算，`slow_wasm` 也改用它。上限沒動、沒加逾時。
- 2026-11-12 新測試：
  - `check_blocks_the_native_latency_groups_over_their_limit`：原生每鍵 2 ms 時 `--check` 只擋空白的「全部」「第 20 碼起」兩格，報告註明「超過」，只印結果時不擋。這補上了票 05 Comments「`--check` 擋原生延遲的那行接線沒有整合測試」那條。變異：拿掉 `lib.rs` 的 `violations.extend(latency.violations())` 會紅。
  - `the_real_native_timer_times_every_key_and_the_same_selects_as_wasm`：code-review 指出換成假計時器後 `native_timing_with` 已經沒有測試，補這條。用真實的原生計時跑 Report，只驗各組的輸入數、鍵數與環境欄（debug／release），不驗時間，所以不受負載影響。量選開頭一段的輸入和 WebAssembly 不同時 `run` 會報錯，所以這條也驗了學過的學習資料下的規則。變異：環境欄改字、原生不量選開頭一段，都會紅。
  - 雙向判斷照舊由 `tests/latency.rs` 的 `mod gate` 負責（假數字：任一格超過就紅，在上限內不紅），沒改。
- 2026-11-12 修之後：同樣開 48 個迴圈，load average 61.59，`cargo test -p qim-bench` 連跑三次都綠（items 10、latency 7、repo_files 2、run 61 passed；補 `the_real_…` 之後 run 是 62）。`grep -rn "native_timing\|Instant" crates/qim-bench/tests/` 沒有結果。
- 2026-11-12 產品閘門照舊：`cargo xtask data` 之後跑 `cargo xtask bench --check`，結果 `bench 通過`，原生列是真實量到的 release 數字（µs，p95：空白 80.96／88.21／1347.75，學過 233.96／671.67／2535.04；load average 約 21–28）。`cargo xtask check` 通過（e2e 237 passed、2 skipped；這個 worktree 先跑了 `npm ci`）。
- 2026-11-12 code-review（`feb4178` 起）已處理：
  - 補真實原生計時的形狀測試（見上）。
  - 閘門測試裡多餘的 closure 改成直接用 `Copy` 的 `Timers`。
  - `native_at` 改名 `fixed_native`。
- 2026-11-12 code-review 沒處理：
  - `native_timing_with` 又從 crate 根匯出，等於推翻票 05 code-review 的決定。原因是 `main.rs` 要把它交給 `Timers`，而 `latency` 是私有模組。審查建議改成 `Timers::real(root)`，但 `node_wasm_timer(root)` 回傳的是借用 `root` 的 closure，`Timers` 存的是 `&dyn`，建構子沒辦法把區域變數的 closure 借出去；要做就得把 `Timers` 改成擁有 `Box<dyn Fn>`，為了收回一個匯出不划算。`NativeTimer` 為了和 `WasmTimer` 成對，一起匯出。
  - `fake_timing` 和 `native_timing_with` 都有「逐鍵打、看分段數」的迴圈。假的那份在測試裡，刻意不重用產品的那份，所以 `the_real_…` 才能拿兩者互相核對。
- 2026-11-12 與本票無關的發現（留給協調者插修復票，或交給票 07 / 收尾票）：`crates/qim-layout/tests/spatial.rs` 的 proptest `on_a_gboard_grid_the_most_likely_letter_is_the_hit_key`（票 06）偶爾抽到失效點，紅在第二個斷言 `w[0].1 > w[1].1`：`(376.8759717002673, 146.41283071299227)` 時 `[('l', 0.5), ('m', 0.5)]`，另一次縮到 `(366.0247247808735, 138.6298953419659)`。抽中後 proptest 會寫 `spatial.proptest-regressions`，之後每次都紅。本 session 的 `cargo xtask check` 五次裡紅一次；regression 檔沒 commit，兩個種子：`cc bfd391913d686817b95a7ad7ef167cf94fd7e9f6f2f2a9ec78f057d2931aef5c`、`cc 36099777f516688467458ef0caa4416f343fb0ee31cd511e6b5ea1db6e572fbd`，放進該檔就能重現。
