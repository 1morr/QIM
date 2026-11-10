# 資料包載入後的常駐記憶體（Mac）

回答的問題（M2 票 04）：每份資料包 postcard 反序列化後在記憶體裡多大、`jsonschema` 讓執行檔與記憶體多多少。結果給票 08 決定鍵盤擴充放哪些資料包、給票 09 決定要不要改 mmap 友善格式；結論與建議在 `docs/research/ios-feasibility.md`〈Mac 上的資料包記憶體〉。

## 跑法

先在 repo 根目錄跑過 `cargo xtask data`（要 `build/packs/` 與 `build/layouts/gboard-quick.json`），再：

```sh
scripts/experiments/pack-memory/run.sh            # 有無 Schema 驗證各跑 3 次，最後另開程序跑 PROBE=hashset
RUNS=1 scripts/experiments/pack-memory/run.sh     # 各 1 次
scripts/experiments/pack-memory/run.sh <build 目錄> # 不用 repo 的 build/（相對路徑照目前目錄）
```

這是獨立的 crate（`Cargo.toml` 有空的 `[workspace]`），不在 repo 的 workspace 裡，`cargo xtask check` 不編它；改程式後自己跑 `cargo clippy --release [--features schema] -- -D warnings`。`run.sh` 編兩個版本：預設版只用 `serde_json` 把佈局讀進型別；`--features schema` 版走 `qim_layout::validate`（含 JSON Schema 驗證）。release 組態開 `lto`、`codegen-units = 1`、`strip`，接近上架的組態。

換 Rust 版本、資料包格式、資料包內容（詞庫門檻、bigram 剪枝）或 macOS 主版本時重量。

## 量法

`src/main.rs` 依序做下面幾步，每步前後各讀一次三個指標，表上是差：

1. 速成、詞庫、bigram、字根提示、簡繁、英文六份資料包：`fs::read` 讀進 `Vec<u8>`、`*Pack::from_bytes` 反序列化、丟掉檔案位元組。量的是這一整段，檔案緩衝區釋放後沒退掉的部分也算在內。
2. `Engine::new(速成, 詞庫, bigram).with_simplifier(簡繁).with_english(英文)`：五份資料包被移進 `Engine`，這步量的是它另外建的（依字頻重排的碼表、建構時的暫存）。字根提示不進 `Engine`（ADR 0003），照殼的用法另外留著。
3. 讀預設佈局 `build/layouts/gboard-quick.json`，有無 Schema 驗證看版本。
4. 量完後（暫存不混進上面的增量）：呼叫 `malloc_zone_pressure_relief(NULL, 0)` 看能還多少；另讀一次詞庫數碼與詞條，並照 `Engine::new` 算詞頻總和的做法對全部詞建一個 `HashSet<&str>` 再丟掉；配置 13 MB、寫滿、丟掉，看釋放的大塊配置會不會從 footprint 退掉，再呼叫一次 `malloc_zone_pressure_relief`；`anr` 的候選要有「門口」，確認資料包真的載入了。
5. `run.sh` 最後用 `PROBE=hashset` 另開一個程序，只載入詞庫、建同一個 `HashSet` 再丟掉：前面的程序裡有大量釋放過的頁可以重用，單獨量才看得出它要多少。

1 MB = 1,048,576 位元組，與 `scripts/experiments/ios-keyboard/` 的讀數同單位。

## 指標

都由 `proc_pid_rusage(getpid(), RUSAGE_INFO_V4)` 與 `malloc_zone_statistics(NULL)` 讀（2026-11-10 查）：

- **RSS**（`ri_resident_size`，同 `mach_task_basic_info.resident_size`）：XNU 在 `task_info(MACH_TASK_BASIC_INFO)` 回傳 `phys_mem` ledger（[`osfmk/kern/task.c`](https://github.com/apple-oss-distributions/xnu/blob/main/osfmk/kern/task.c)），pmap 每建一個有效的頁映射就加一頁（[`osfmk/arm/pmap/pmap.c`](https://github.com/apple-oss-distributions/xnu/blob/main/osfmk/arm/pmap/pmap.c)）。推論：含共享 dylib 與檔案映射的頁、不分乾淨或髒；被壓縮器收走的頁不算。Apple 沒有逐字的完整定義。
- **footprint**（`ri_phys_footprint`，同 `task_vm_info.phys_footprint`）：XNU 的定義（`osfmk/kern/task.c` 的 Task ledgers 註解）是「(internal − alternate_accounting) + (internal_compressed − alternate_accounting_compressed) + iokit_mapped + purgeable_nonvolatile + purgeable_nonvolatile_compressed + page_table」，internal 是「The task's anonymous memory」，internal_compressed 是被壓縮器收走的匿名頁。和 RSS 差在：footprint 含壓縮的匿名頁、不含乾淨的檔案映射頁（推論：公式沒有 external 項）。`man footprint`（macOS 27.0.1）：「the 'MEM' column in top(1), the 'Memory' column in Activity Monitor.app, and the Memory Debug Gauge in Xcode.app all report values from the process footprint ledger.」iOS 鍵盤擴充的上限看的推定也是它（`docs/research/ios-feasibility.md`〈免費佈建與 App Group〉：`footprint + os_proc_available_memory()` 恆定）。
- **heap**（`malloc_statistics_t.size_in_use`，所有 zone 的合計）：[`malloc/malloc.h`](https://github.com/apple-oss-distributions/libmalloc/blob/main/include/malloc/malloc.h) 對這欄沒有註解，從欄位名推定是已配置、還沒 free 的位元組；用來分出「還在用的」與「釋放了但沒還給系統的」。
- **footprint 最高**（`ri_lifetime_max_phys_footprint`）：到讀數時為止的最高點。
- 倍數 = footprint 增加 ÷ 檔案大小。
- 前一步釋放的頁會被後一步重用（見〈觀察〉），所以每步的 footprint 增量與步驟順序有關；合計與 heap 不受影響。

## 上次結果（2026-11-10）

環境：Mac18,5（Apple M6，32 GB），macOS 27.0.1（26A434），Rust 1.99.1（58cf8f9c3 2026-10-13），commit `5186e6e` 建的資料包。`run.sh` 跑了兩輪，每個版本共 6 次；表是預設版其中一次的完整輸出，各列 6 次的差都在 0.5 MB 內。

| 步驟 | 檔案 | RSS 增加 | footprint 增加 | heap 增加 | footprint ÷ 檔案 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 速成 `quick` | 0.43 MB | 1.39 MB | 1.38 MB | 0.74 MB | 3.2× |
| 詞庫 `quick-lexicon` | 4.95 MB | 46.67 MB | 46.69 MB | 40.73 MB | 9.4× |
| bigram `zh-hant-bigram` | 2.87 MB | 12.34 MB | 12.34 MB | 7.45 MB | 4.3× |
| 字根提示 `cangjie-hint` | 1.24 MB | 16.33 MB | 16.33 MB | 14.78 MB | 13.1× |
| 簡繁 `opencc-hk2s` | 0.06 MB | 0.77 MB | 0.75 MB | 0.86 MB | 11.7× |
| 英文 `scowl-en-us` | 0.76 MB | 4.11 MB | 4.09 MB | 3.31 MB | 5.4× |
| `Engine`（拿走上面五份） | — | 12.80 MB | 12.78 MB | -0.03 MB | — |
| 佈局 `gboard-quick.json`（不驗證） | 0.23 MB | 0.27 MB | 0.19 MB | 0.40 MB | 0.8× |
| 合計 | 10.54 MB | 94.73 MB | 94.58 MB | 68.25 MB | 9.0× |

```text
開始時 RSS 1.77 MB、footprint 1.19 MB、heap 0.01 MB
結束時 RSS 96.50 MB、footprint 95.77 MB、heap 68.26 MB；footprint 最高 95.77 MB
malloc_zone_pressure_relief 還給系統 0.00 MB；之後 RSS 96.50 MB、footprint 95.77 MB
對照：對 257468 個詞建 HashSet<&str> 再丟掉後 footprint 增加 0.22 MB、heap 增加 0.00 MB
對照：配置 13 MB 寫滿再丟掉後 footprint 增加 13.02 MB、heap 增加 0.00 MB；再呼叫 malloc_zone_pressure_relief 還給系統 0.00 MB，footprint 變化 0.00 MB
詞庫 234036 個碼、283387 個詞條（不重複的詞 257468 個）
anr 前三個候選：門口、日名、日阿
== PROBE=hashset
PROBE=hashset：新程序裡對 257468 個詞建 HashSet<&str> 再丟掉後 footprint 增加 17.27 MB、heap 增加 0.00 MB
```

有無 Schema 驗證（6 次的範圍；資料包各列兩版相同）。「不驗證」只用 `serde_json` 讀進型別；「Schema 驗證」是整條 `qim_layout::validate`：先建 `serde_json::Value`、跑 JSON Schema、讀進型別、再查引用與每頁有沒有回字母頁的鍵。所以差距是整條 `validate` 的成本，不只是 `jsonschema`；「只留型別與引用檢查」的讀法沒量，`qim-layout` 沒有公開那條路。

| 版本 | 執行檔（strip 後） | 開始時 RSS | 開始時 footprint | 佈局那步 RSS 增加 | 佈局那步 footprint 增加 | 佈局那步 heap 增加 | 結束時 footprint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 不驗證 | 0.67 MB | 1.70–1.77 MB | 1.19–1.20 MB | 0.20–0.28 MB | 0.12–0.20 MB | 0.40 MB | 95.6–95.8 MB |
| Schema 驗證 | 3.75 MB | 1.88–1.92 MB | 1.33–1.34 MB | 4.66–4.86 MB | 2.92–3.14 MB | 0.84 MB | 98.7–98.9 MB |

## 觀察

- **釋放的頁不退出 footprint，但會被之後的配置重用。** 13 MB 的對照配置丟掉後 heap 歸零、footprint 照樣 +13 MB，前後兩次 `malloc_zone_pressure_relief` 都還回 0。同一個程序裡再建那個 `HashSet` 只 +0.2 MB：它重用了前面釋放的頁。所以 footprint 跟著 heap 用量的高水位走，不是一路累加。
  - 詞庫、字根提示、英文的 footprint 比 heap 多的部分約等於檔案大小：檔案緩衝區丟掉了、頁還算在 footprint 裡（詞庫 46.69 − 40.73 = 6.0 MB，檔案 4.95 MB）。bigram 多 4.9 MB、比檔案的 2.87 MB 大，推論是反序列化時的暫存（差分還原成 `u32` 陣列）。
  - `Engine` 那步 heap 不變、footprint +12.8 MB。`Engine::new` 為了算詞頻總和，對 25.7 萬個不重複的詞建暫存的 `HashSet<&str>`，單獨量是 +17.3 MB：擴容時每一代較小的表都放不進下一代，加起來約為最終表的兩倍。實跑時它部分重用前面釋放的檔案緩衝區，所以整步只 +12.8 MB；`by_frequency` 整份複製碼表再丟掉舊的，影響多少沒有分開量。
  - 系統記憶體吃緊時這些頁會不會被回收、iOS 的 jetsam 前會不會先回收，沒有查到依據，票 09 加壓時看。
- **還在用的 68 MB 裡，詞庫佔 41 MB。** 詞庫 4.95 MB 的檔案有 23.4 萬個碼、28.3 萬個詞條，每個碼與每個詞都是一個獨立配置的 `String`，加上 `BTreeMap` 節點與 `Vec<Word>`；字根提示 1.24 MB → 14.8 MB 同樣是每個字兩個 `Vec<String>`。bigram 已經是攤平的陣列，heap 只是檔案的 2.6 倍。
- **整條 `validate` 讓執行檔多 3.1 MB**（0.67 → 3.75 MB），驗證那一下 footprint 多約 3 MB（驗證器與 regex 的暫存，照上一條留在 footprint），驗完還在用的只多 0.44 MB。程式還沒做事時 RSS 多約 0.15 MB、footprint 多約 0.14 MB：程式碼頁是檔案映射的乾淨頁、推論不算在 footprint，多出的 footprint 推論是載入時要改寫的資料段。
