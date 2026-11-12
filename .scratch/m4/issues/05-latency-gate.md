# 05: 每鍵延遲閘門

Status: done
Blocked by: None (can start immediately)
讀: plan §5.2；`docs/development.md`〈每鍵延遲〉；spec〈每鍵延遲上限〉；`.scratch/m3/issues/12-personal-bigram.md` Comments（學過資料的量法）

## What to build

容錯與萬用鍵會放大 lattice，每鍵延遲要從只報不擋改成會擋：

- `cargo xtask bench --check` 照 spec〈每鍵延遲上限〉擋原生的 p95（全部、第 20 碼起、選開頭一段），WebAssembly 照舊只報。
- 報告另量一組「學過的學習資料」：一個 `UserModel` 依序學完全部公開集合後再計時（M3 票 12 的量法），原生與 WebAssembly 都量，原生照它那一列的上限擋。
- 超過時的處理照 spec（負載低時單獨重跑一次，仍超過才紅），寫進 `docs/development.md`。

## 驗收

- [x] 判斷函式的雙向測試：量到的數字超過任一格會紅，在上限內不紅。
- [x] 報告有空白與學過兩組的 p50 / p95（原生、WebAssembly）。
- [x] plan §5.2 與 `docs/development.md` 改成「原生擋、WebAssembly 只報」，上限連到 spec。
- [x] Comments 附這次量到的兩組數字；`cargo xtask check` 綠燈。

## Comments

- 2026-11-12 做法：`Latency` 分 `blank`、`learned` 兩組 `Runs`（原生、WebAssembly）；`Latency::violations` 只比原生的 p95 和 `qim_bench::latency::p95_limits_ms`（spec〈每鍵延遲上限〉的表），`--check` 時併進違規。學過的 `UserModel`：`score::learned_model` 照修正次數的模擬依序學完各語料集合與「有前文」的題（給前文），共 6,638 + 3,025 = 9,663 句，就是 M3 票 12 的「約 1 萬句」；原生各題共用它的一份複本，WebAssembly 由 `--user-dict <暫存檔>` 給 `latency.ts`、`importUserDict` 後同一個 `Session` 打到底（打字與選開頭一段不上屏、不學）。整個 `cargo xtask bench` 因多學一次約 3 分鐘（Apple M6）。
- 2026-11-12 超過時「負載低時單獨重跑一次」照票寫進 `docs/development.md`〈每鍵延遲〉，由人做、程式不自動重跑：自動重跑會把負載雜訊藏起來，也讓一次 `--check` 時間加倍。
- 2026-11-12 這次量到的數字（µs，p50 / p95；Apple M6，`cargo xtask bench`，第一次跑，負載較低）：

  | 學習資料 | 環境 | 全部 | 第 20 碼起 | 選開頭一段 |
  | --- | --- | ---: | ---: | ---: |
  | 空白 | 原生 | 47.9 / 70.2 | 52.9 / 74.8 | 372 / 1254 |
  | 空白 | WebAssembly | 61.8 / 99.3 | 68.8 / 107.7 | 480 / 1667 |
  | 學過 | 原生 | 101.1 / 190.9 | 107.4 / 282.5 | 605 / 2223 |
  | 學過 | WebAssembly | 127.4 / 261.8 | 136.3 / 534.3 | 759 / 2921 |

  空白的接近 M3 收尾（73 µs / 1.29 ms）；學過的接近 M3 票 12（219 / 441 / 1583 µs）。之後兩次 `bench --check` 是在機器 load average 約 20（別的 worktree 在建資料與跑模擬器）時跑的，原生 p95 空白 130 / 140 / 2399、學過 341 / 1066 / 4501，以及 112 / 114 / 1794、397 / 1049 / 4262，都在上限內、`bench 通過`。負載高時學過的「第 20 碼起」和「選開頭一段」漲到約 4 倍，離上限（3 / 10 ms）約剩 2–3 倍的餘裕；容錯進核心後若常在高負載下紅，照 development.md 重跑。
- 2026-11-12 驗證：`cargo xtask check` 通過（e2e 222 passed），`cargo test --workspace` 976 passed，`cargo xtask bench --check` 通過（三次）。
- 2026-11-12 code-review（`1cf2f1c` 起）已處理：手寫的暫存檔改用已有的 `tempfile`（從 dev-dependency 移到 dependencies）；超過上限的判斷收成 `Summary::over`，報告與閘門共用；`measure_runs` 改收 `Option<&UserModel>`，不再同時傳模型與匯出文字；去掉同名遮蔽；`native_timing_with`、`p95_limits_ms` 不再從 crate 根匯出；development.md 的孤立清單項併進「載入」那條；測試註解不再抄上限數字。
- 2026-11-12 code-review 沒處理：
  - `--check` 擋原生延遲的那行接線（`violations.extend(latency.violations())`）沒有整合測試：要讓原生超過上限得有慢的引擎，小資料包做不出來；判斷函式本身有雙向測試。
  - `run.rs` 的 `Mode::Check` 測試現在也受真實的原生計時擋（debug，小資料包 p95 約 40 µs，上限 1 ms，約 25 倍餘裕），極高負載下理論上會偶發紅。
  - Node 那條路（`node_wasm_timer` 寫暫存檔、`latency.ts` 解析 `--user-dict`）沒有自動測試，同以前的 Node 路徑；以實跑 `cargo xtask bench` 驗證（學過那組 WebAssembly 量選開頭一段的輸入數 4,809 與原生相同）。
  - `learned_model` 和 `score::learning` 的學習迴圈相似，但 `learning` 每題要交錯打學習關與學習開，不硬抽。
  - 發現與本票無關的既有競態（留給 M4 票 02 的網頁加固或收尾票）：`web/e2e/key-table.spec.ts` 與 `keys.spec.ts` 的「翻頁」測試在打完 `ar` 立刻擷取候選列當 `first`，機器高負載時停鍵層的重排晚到、改了候選順序，之後的 `toHaveText(first)` 逾時；高負載下 `cargo xtask check` 三次裡兩次因它紅，單獨跑都過。
