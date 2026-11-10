# 11: 小型神經模型在鍵盤擴充裡的開銷（candle）

Status: done
Blocked by: 06
讀: plan §2.7、§7.8、§8（小型神經模型那列）；ADR 0005；brief §9；spec〈共同規則〉；`docs/research/qingjian.md`；`docs/research/ranking-techniques.md`

## What to build

用隨機權重的兩個模型量鍵盤擴充跑神經模型的成本：Gboard 級約 1.4M 參數、青簡級約 23M 參數。只量開銷，不訓練、不量正確率（M3）。推論引擎先試青簡用的 candle；candle 在 iOS 能不能跑本身就是要回答的問題。

- 模型形狀照兩個參照的公開描述（研究文件有的照寫，沒有的取合理近似並註明）；輸入長度取 M3 的用法（重排前 6 條路徑、上下文 0 / 50 / 200 字）。
- `scripts/experiments/neural-cost/`：不加進 workspace 的 Rust crate，先在 Mac 量，再交叉編譯到 `aarch64-apple-ios`，接進票 06 的擴充量。
- 指標：模型檔大小、載入時間、單次推論延遲 p50 / p95、擴充常駐記憶體增量；Mac 與 iPhone 各一份。
- candle 在 iOS 編不過或跑不動時，記下原因並試一個替代（查證後選，例如 Core ML 轉換），或停下問使用者。
- 結果寫進 `docs/research/ios-feasibility.md` 的〈小型神經模型〉一節；放不進票 09 的上限時，照 plan §8 寫回 brief §1.5 的候選做法。

## 驗收

- [x] 兩個模型在 Mac 與 iPhone 上各有四項指標，附機型、iOS 版本、candle 版本；或明確的「不能」與原因。
- [x] README 能照著重量。
- [x] `cargo xtask check` 綠燈。

## Comments

- 2026-11-10 結果：candle 0.11.1 不用改就能在 iPhone 的鍵盤擴充裡跑，沒試替代。iPhone 上 1.4M 兩種格式 p95 ≤ 8.2 ms、峰值 ≤ +4.7 MB；23M q8 上下文 200 字 p95 54.5 ms、峰值 +46.9 MB；23M f32 77.8 ms、+94.9 MB。對 177 MB（票 06）加核心約 106 MB，只有 23M f32 放不進；23M q8 放得進，所以沒改 brief §1.5，只在 plan §8 那列記結果。細節在 `docs/research/ios-feasibility.md`〈小型神經模型〉。
- 票內的取捨：
  - 每種模型另量 q8：Gboard 的 1.4 MB 本來就是 8-bit，青簡也是 int8 線性層，只量 f32 會高估。
  - 推論固定 2 條執行緒的專屬 rayon pool：用全域 pool 時 gemm 每條執行緒各留一塊 L2 大小的暫存（Mac 共 136 MB）。
  - 記憶體以 `ledger_phys_footprint_peak` 為準：iPhone 會還回釋放的大塊配置，跑完時的 footprint 量不到峰值。
  - 候選路徑每條 8 字是自訂的近似（plan §7.8 沒定）。
- 給票 08、09：`ios-keyboard` 的 `project.yml` 現在一定連結 `../neural-cost` 的靜態庫、bundle 裡放 118 MB 的模型檔，建置前要先跑 `scripts/experiments/neural-cost/build-ios.sh`，模擬器也建不起來（只編了實機）。擴充的 Debug dylib 因此變成 12.9 MB；`viewDidLoad` 的 footprint 約 7.2 MB、`viewDidAppear` 約 10.9 MB，與票 06 的基線相近。票 09 量終止點時要是嫌它干擾，可以把 `project.yml` 裡票 11 的三行設定與 `models` 資料夾拿掉。
- 另一個 session（票 08）用同一個 Bundle ID 裝過它的版本，蓋掉本票的擴充；量之前看鍵盤上有沒有 L32 那一列（README 有寫）。
- code-review 沒處理的發現（都是判斷題）：
  - GPT 的權重名在 `Kind::shapes` 與 `Block::load` 各寫一次。
  - 四個模型檔名在 `model::file_name`、`run.sh`、`NeuralProbe.models`、`build-ios.sh` 各寫一次。
  - 預設的 30 次、2 條執行緒在 `main.rs`、`NeuralProbe`、`run.sh` 各寫一次。

  實驗程式規模小，不再抽。`pack-memory` 的 panic 與 echo 也是中文，沒有跟著改。
