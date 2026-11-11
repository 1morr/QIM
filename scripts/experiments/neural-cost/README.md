# 小型神經模型的開銷（Mac 與 iPhone 鍵盤擴充）

回答的問題（M2 票 11）：停鍵層（plan §2.7）要是用小型神經模型重排候選，Gboard 級（約 1.4M 參數）與青簡級（約 23M 參數）的模型在 iOS 鍵盤擴充裡要多少記憶體、載入多久、重排一次多久；推論引擎先試青簡用的 candle，它能不能在 iOS 跑本身也是問題。只量開銷，權重是隨機的，不訓練、不量正確率（M3，plan §7.8）。結論在 `docs/research/ios-feasibility.md`〈小型神經模型〉。

## 內容

| 路徑 | 內容 |
| --- | --- |
| `src/model.rs` | 兩個模型、權重檔的寫出與讀入、重排打分；單元測試對拍批次與前綴快取的分數 |
| `src/bench.rs` | 一個模型檔的四項指標 |
| `src/footprint.rs` | `phys_footprint`、它的峰值、malloc 還在用的量 |
| `src/lib.rs` | 給 iOS 的 C 介面 `nc_run`、`nc_free`（`include/neural_cost.h`） |
| `src/main.rs` | Mac 的命令列：`gen <目錄>`、`run <模型檔> [次數] [執行緒] [unshared]` |
| `run.sh` | Mac：寫出模型檔，四個模型各開 3 個新程序量；`PREFIX=unshared` 量不共用上下文的對照 |
| `build-ios.sh` | iOS：寫出模型檔，編 `aarch64-apple-ios` 的靜態庫給 `../ios-keyboard` 連結 |

這是獨立的 crate（`Cargo.toml` 有空的 `[workspace]`），不在 repo 的 workspace 裡，`cargo xtask check` 不編它。改程式後自己跑 `cargo test --release`、`cargo clippy --release --all-targets -- -D warnings`、`cargo fmt --check`。模型檔（`models/`，共 118 MB）與 `target/` 不進 git，`gen` 隨時重寫。

## 模型

兩個都是字級語言模型，給一段上下文和 6 條候選路徑，回每條路徑各字 log 機率的和。

| | Gboard 級 `lstm-1.4m` | 青簡級 `gpt-23m` |
| --- | --- | --- |
| 參照 | Gboard 的 CIFG-LSTM（Hard et al. 2018，`docs/research/ranking-techniques.md` §7.1） | 青簡含章 small 的字級 decoder（`docs/research/open-source-imes.md` §2.2；青簡 `docs/notes/neural-rescoring.md`） |
| 結構 | 單層 CIFG（輸入閘與遺忘閘共用），670 個單元，cell 輸出投影回 96 維，輸入輸出共用 embedding | GPT-2 式 decoder，pre-LayerNorm，學到的位置 embedding，輸入輸出共用 embedding |
| 照寫的 | 670 單元、embedding 與投影 96、詞表 1 萬、共用 embedding | 約 23M 參數、上下文 256 |
| 近似的 | 論文是詞級，這裡當字級用、字表同取 1 萬；沒加 peephole | 字表 8000、寬 512、6 層、8 頭、前饋 2048：青簡只公開參數量與上下文，這組是湊出約 23M 的近似 |
| 參數 | 1,412,250 | 23,142,400 |

每個模型兩種格式：

- `f32`：全部 f32，safetensors，用 mmap 讀（不先整檔讀進緩衝區，票 04 量過整檔緩衝區釋放後會留在 footprint）。
- `q8`：內層維度是 32 倍數的矩陣（含共用的 embedding）存 Q8_0，其餘 f32；GGUF。同青簡的做法（線性層 int8），連 embedding 也量化則同 Gboard（140 萬參數 1.4 MB）。LSTM 的投影矩陣內層是 670，不是 32 的倍數，留 f32。

權重是 N(0, 0.02) 的隨機值（LayerNorm 權重 1、偏差 0）；值不影響開銷。

## 量法

`bench::run` 對一個模型檔做：

1. 讀 footprint、峰值、heap。
2. 載入：開檔、讀權重、建好模型（不含第一次推論）。記時間與 footprint 增量。
3. 第一次推論：上下文 0 字，記時間（含 candle 第一次用到的初始化，Q8_0 的重排見〈觀察〉）。
4. 上下文 0、50、200 字（plan §7.8）各先跑 3 次不計，再跑 30 次，記 p50、p95（最近秩法）與跑完時的 footprint 增量。每次重排：上下文前加句首記號跑一次，狀態（LSTM 的 h、c；decoder 的各層 K、V）複製給 6 條路徑，每條 8 個字，一批算完。6 條與上下文長度照 plan §7.8；每條 8 字是自訂的近似（plan 沒定路徑長度，取一段短句）。上下文與候選是固定種子的假字 id。
5. 讀 footprint、峰值、heap，算增量。模型在函式結束時丟掉。

推論在 2 條執行緒的專屬 rayon pool 裡跑，並把 `RAYON_NUM_THREADS` 設成 2（candle 的矩陣乘法照它切份）。2 條是 candle 在 Apple 晶片的預設（效能核數），也是 iPhone 18 Pro Max 的效能核數；青簡限 4 條。理由見〈觀察〉的 gemm 暫存。

每個模型要在新程序量：釋放的頁會被下一個模型重用，混在同一個程序裡增量就不準。Mac 由 `run.sh` 每次開新程序；iPhone 每量完一個按「結束」讓擴充自己 `exit(0)`，收起再叫出鍵盤就是新程序（看 `pid=`）。

## 前綴快取

**這裡的延遲以前綴快取為前提：M3 實作要做到同樣的共用，否則延遲會是幾倍。** 〈量法〉第 4 步的寫法是上下文只跑一次，狀態複製給 6 條路徑（`Scorer::score`）。青簡的筆記沒寫這種共用：64 字上下文配 8 條候選，在 M1 Mac 上就要 133 ms（`docs/research/qingjian.md` 引的 `neural-rescoring.md`）。

對照用的 `Scorer::score_unshared` 讓每條路徑各自從句首連上下文算起，其餘照舊（只在預測候選字的位置算輸出層）；單元測試確認兩種寫法分數相同。只在 Mac 量（`PREFIX=unshared ./run.sh`，2026-11-10，環境同〈上次結果〉），p50 / p95（ms），3 個程序的中位數：

| 模型 | 上下文 0 | 上下文 50 | 上下文 200 | 200 字時對共用慢幾倍（p50） |
| --- | ---: | ---: | ---: | ---: |
| 1.4M f32 | 4.4 / 4.9 | 12.7 / 13.0 | 37.8 / 38.1 | 4.4 |
| 1.4M q8 | 1.6 / 1.6 | 5.1 / 5.3 | 15.8 / 15.9 | 4.2 |
| 23M f32 | 32.8 / 35.7 | 96.2 / 102.6 | 298.8 / 306.5 | 4.7 |
| 23M q8 | 7.9 / 9.1 | 41.6 / 46.5 | 158.7 / 172.6 | 4.7 |

- 上下文越長差越多：上下文 50 字慢 3.3–3.4 倍（23M），0 字也慢 1.1–2 倍（共用時 6 條一批算）。
- 照這個倍數推，iPhone 上 23M q8 在上下文 200 字會到約 250 ms；iPhone 沒量。
- 記憶體與共用時相近：23M q8 峰值 +66.6–72.7 MB，共用時 +73.6 MB。
- 量的時候機器有別的 session 在跑：23M f32 有一個程序的 p95 偏高（77.5、201.9 ms），表上取中位數。

## 指標

- **footprint**：`task_info(TASK_VM_INFO)` 的 `phys_footprint`，同 `../ios-keyboard` 的 `Shared/Probe.swift` 與 `../pack-memory`；iOS 鍵盤擴充的上限看的推定是它（`docs/research/ios-feasibility.md`〈免費佈建與 App Group〉）。
- **峰值**：同一個呼叫的 `ledger_phys_footprint_peak`（`task_vm_info` REV3 起的欄位，`<mach/task_info.h>`），程序到讀數時為止 footprint 的最高點。輸出的 `peak+` 是「跑完時的峰值 − 開始時的 footprint」。開始前的峰值（`peak before`）比開始時的 footprint 高時，這個增量最多多算那段差（iPhone 上是 0.6–0.8 MB）。
- **heap**：`malloc_zone_statistics(NULL)` 的 `size_in_use`，跑完時（模型還沒丟）malloc 還在用的位元組，含權重、candle 的快取與 gemm 的暫存。
- 1 MB = 1,048,576 位元組。

輸出（Mac 印在終端機，iPhone 顯示在鍵盤上方並寫進 `qim.exp` 的系統 log）：

```text
gpt-23m q8 params=23.14M file=23.57MB threads=2
load=3.0ms first(ctx=0)=11.1ms
ctx=0 p50=6.1ms p95=6.2ms n=30 fp+23.6MB        # fp+ 是這個上下文長度跑完時比載入完多的 footprint
ctx=50 p50=12.2ms p95=12.3ms n=30 fp+25.3MB
ctx=200 p50=33.6ms p95=33.9ms n=30 fp+43.7MB
footprint 1.36MB load+29.81MB infer+43.66MB total+73.47MB heap+67.31MB
peak before=1.36MB after=74.83MB peak+73.47MB
```

## 跑法

Mac（約 1 分鐘）：

```sh
scripts/experiments/neural-cost/run.sh                 # 四個模型各 3 個新程序
RUNS=1 ITER=50 THREADS=4 scripts/experiments/neural-cost/run.sh
scripts/experiments/neural-cost/target/release/neural-cost run scripts/experiments/neural-cost/models/gpt-23m-q8.gguf 30 1
PREFIX=unshared scripts/experiments/neural-cost/run.sh  # 不共用上下文的對照（〈前綴快取〉）
```

iPhone：`../ios-keyboard` 預設不建神經模型探針。照 `../ios-keyboard/README.md`〈小型神經模型（票 11）〉：先跑 `build-ios.sh`，再用 `QIM_NEURAL=YES` 產生專案、建置、安裝，同一節有在鍵盤上的操作步驟。

換 candle、Rust、Xcode 或 iOS 主版本，或改模型形狀、執行緒數時重量。

## 上次結果（2026-11-10）

路徑數與長度可用環境變數 `NC_PATHS`、`NC_PATH_LEN` 蓋掉（預設 6、8；iOS 擴充的 `L8·9`、`G8·9` 兩個鍵在呼叫前設成 9、18，其餘照預設）。M3 票 07 用 9 條 × 7 字、9 條 × 18 字重量了 Mac，結果與照倍數推估的 iPhone 數字在 `../model-ladder/README.md`〈N 與路徑長度〉；M3 票 09 在 iPhone 實量 9 × 18（23M q8 上下文 200 字 p95 84 ms、峰值 +75 MB），在 `../neural-rerank/README.md`〈成本〉；下面是 M2 票 11 的 6 × 8。

環境：

- Mac：Mac18,5（Apple M6，32 GB），macOS 27.0.1（26A434）。
- iPhone：iPhone 18 Pro Max（iPhone19,7），iOS 27.2（24B119）；擴充 Debug 組態（Swift），Rust 靜態庫一律 release（`lto`、`codegen-units = 1`）；沒有完整取用。Xcode 27.1（27A9275），iOS SDK 27.1。
- Rust 1.99.1（58cf8f9c3 2026-10-13）；candle-core、candle-nn 0.11.1，CPU 後端（預設 feature，矩陣乘法用 gemm 0.19.0，沒開 `accelerate`、`metal`）；2 條執行緒，每個上下文長度 30 次。
- repo 在 `78ae0c2`，實驗碼即本票的 commit。

Mac 每個模型 3 個程序，表上是 3 次的中位數；這 3 次間 p50 差在 0.2 ms 內、記憶體差在 0.5 MB 內。code-review 時在機器較忙（load average 約 5、別的 session 同時在編譯）時重跑一次：23M f32 峰值 119.6–126.9 MB、23M q8 75.0–80.4 MB、1.4M f32 上下文 200 字 p50 10.4 ms，比表上高。Mac 的記憶體會隨負載飄幾 MB，延遲也是，重量時讓機器空著。iPhone 每個模型 1 個程序（截圖 `.scratch/m2/evidence/11-iphone-{L32,L8,G32,G8}.png`）；量時手機上有一個嗶哩嗶哩的子母畫面影片在播，會和推論搶 CPU，延遲可能偏高或較不穩。

延遲（ms，p50 / p95）：

| 模型 | 機器 | 載入 | 第一次推論 | 上下文 0 | 上下文 50 | 上下文 200 |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1.4M f32 | Mac | 0.5 | 3.1 | 2.9 / 3.0 | 4.3 / 4.4 | 8.5 / 8.7 |
| 1.4M f32 | iPhone | 4.5 | 15.7 | 3.2 / 4.9 | 4.4 / 4.5 | 8.1 / 8.2 |
| 1.4M q8 | Mac | 0.3 | 1.8 | 1.4 / 1.4 | 2.0 / 2.1 | 3.8 / 3.9 |
| 1.4M q8 | iPhone | 2.1 | 12.6 | 2.4 / 4.3 | 3.3 / 3.4 | 6.2 / 6.4 |
| 23M f32 | Mac | 9.0 | 17.6 | 15.7 / 15.9 | 28.9 / 29.6 | 63.3 / 64.2 |
| 23M f32 | iPhone | 35.7 | 28.6 | 15.8 / 15.9 | 29.9 / 30.1 | 69.4 / 77.8 |
| 23M q8 | Mac | 3.0 | 11.1 | 6.1 / 6.3 | 12.2 / 13.4 | 33.6 / 34.0 |
| 23M q8 | iPhone | 17.3 | 34.1 | 8.9 / 9.0 | 18.8 / 19.0 | 53.1 / 54.5 |

Mac 的載入是檔案已在快取時；每個模型第一個程序（剛 `gen` 完、檔案冷）是 1.4M f32 6.0、q8 4.5、23M f32 69.3、q8 7.5 ms。iPhone 每個模型只量一次，裝完第一次讀，推論是冷的。

記憶體（MB；載入、峰值是比開始時多的 footprint，heap 是跑完時 malloc 還在用的增量）：

| 模型 | 檔案 | Mac 載入 | Mac 峰值 | Mac heap | iPhone 載入 | iPhone 峰值 | iPhone heap |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1.4M f32 | 5.39 | 5.48 | 7.05 | 25.42 | 3.92 | 4.67 | 6.48 |
| 1.4M q8 | 1.62 | 2.33 | 4.34 | 2.64 | 0.89 | 2.38 | 2.71 |
| 23M f32 | 88.29 | 88.88 | 116.42 | 108.31 | 87.53 | 94.92 | 89.30 |
| 23M q8 | 23.57 | 29.88 | 73.59 | 67.31 | 18.47 | 46.86 | 48.32 |

iPhone 量時擴充開始的 footprint 是 12.6–12.8 MB（`viewDidAppear` 10.8–10.9 MB，之後畫面又重建一兩次）；模型丟掉後回到 13.1–13.6 MB。原始輸出摘要在 `docs/research/ios-feasibility.md`〈小型神經模型〉。

## 觀察

- **candle 不用改就能編到 `aarch64-apple-ios`，在鍵盤擴充裡跑**（沒有完整取用、CPU）。靜態庫 77 MB（未 strip）；連進擴充後整個 Debug 的 `QIMKeyboard.debug.dylib` 是 12.9 MB。
- **gemm 在每條跑過矩陣乘法的執行緒各配一塊 L2 大小的暫存，用完不還。** Mac（`hw.perflevel0.l2cachesize` 20 MB）每塊約 10 MB；用 rayon 的全域 pool（每個邏輯核一條，這台 12 條）時 13 塊共 136 MB 掛在 heap，碰過的部分算進 footprint。改用 2 條的專屬 pool 後只剩 2 塊（1.4M f32 的 Mac heap 25 MB 裡約 20 MB 是它）。iPhone 上這塊小得多（1.4M f32 的 heap 只比檔案多 1 MB）。用 `heap <pid>` 查到的配置點是 `gemm_common::gemm::L2_SLAB`。
- **candle 0.11.1 在 aarch64 第一次做 Q8_0 乘法時，把權重另外重排一份 `Q8_0x4` 存起來，原本那份也留著**（`quantized/repack.rs` 的 `PackedCache`），沒有開關能關。所以 q8 的常駐約是檔案的兩倍：23M q8 的 heap 是 48–67 MB，上下文 0 跑完 footprint 就比載入完多 21–24 MB。青簡為了別的原因（x86 的 SIMD 選擇）自己寫了 int8 矩陣乘法（`docs/research/qingjian.md` 引的 `crate-notes.md`），自己寫的話這份可以省掉：23M q8 的 heap +48.3 MB 約是權重 23.6 MB、重排的副本約 22 MB（Q8_0 的那幾個矩陣再一份）與 gemm、rayon 的暫存約 3 MB。
- **iOS 會把釋放的大塊配置還給系統，macOS 不會。** iPhone 上 23M f32 跑完、模型還在時，footprint 只比載入完多 0.1 MB 以內，峰值卻多 7 MB；模型丟掉後 footprint 回到開始時的水準。Mac 上 footprint 停在峰值（同票 04 的觀察）。所以 iPhone 要看峰值，Mac 看跑完時的 footprint 也近似峰值。
- 23M 的延遲大致跟 token 數成正比（上下文 200 是 0 的 4–6 倍）；1.4M 的 LSTM 每個字一個小矩陣乘法，主要是逐字的固定開銷。
- iPhone 上 q8 載入那步的 footprint 增量比檔案小（23M q8 18.5 MB 對 23.6 MB，1.4M q8 0.9 對 1.6 MB），heap 卻不小；原因沒查，推測是重用了開始前畫面建立時釋放的頁。看峰值不受影響。
- iPhone 第一次推論比之後慢（12–34 ms 對 2–9 ms），含 rayon pool 與 Q8_0 重排的初始化。
