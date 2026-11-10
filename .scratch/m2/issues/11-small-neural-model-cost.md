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
- 給票 09（合併票 08 時改）：神經模型探針改成可選。`ios-keyboard/project.yml` 只在 `QIM_NEURAL=YES` 時經 xcodegen 的 `include` 併進 `neural.yml`（`Neural/` 的原始碼、模型檔、靜態庫），預設的擴充不連結 candle、不帶模型檔，Debug dylib 3.0 MB。開著建的是 12.9 MB，量基線時兩種別混用。合併後開著建、裝上，備忘錄打 `anr` 仍上屏「門口」（`.scratch/m2/evidence/11-merged-anr.png`）。
- 合計改用 iPhone 的核心數字（協調者要求）：票 08 量到擴充載入核心後 64.4 MB，加模型峰值後 23M q8 約 111 MB、f32 約 159 MB，都在 177 MB 內；先前用 Mac 的 96 MB 算出「f32 放不進」，已改掉。
- 前綴快取（協調者轉 Claude Code 的提醒）：iPhone 的延遲是上下文只算一次、6 條路徑共用的寫法。Mac 補量不共用的對照：上下文 200 字慢 4.2–4.7 倍、50 字慢約 3.4 倍。前提寫進 README、ios-feasibility〈前綴快取〉、plan §7.8 與 brief §12.3。
- 另一個 session（票 08）用同一個 Bundle ID 裝過它的版本，蓋掉本票的擴充；量之前看鍵盤上有沒有 L32 那一列（README 有寫）。
- code-review 沒處理的發現（都是判斷題）：
  - GPT 的權重名在 `Kind::shapes` 與 `Block::load` 各寫一次。
  - 四個模型檔名在 `model::file_name`、`run.sh`、`NeuralProbe.models`、`build-ios.sh` 各寫一次。
  - 預設的 30 次、2 條執行緒在 `main.rs`、`NeuralProbe`、`run.sh` 各寫一次。

  實驗程式規模小，不再抽。`pack-memory` 的 panic 與 echo 也是中文，沒有跟著改。
