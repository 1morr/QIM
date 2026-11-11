# 09: 階梯：停鍵層的小型神經模型

Status: done
Blocked by: 07
讀: spec〈模型階梯實驗〉〈採用門檻〉〈共同規則〉；plan §2.7、§4.3、§8（小型神經模型那列）；brief §12.3（candle、記憶體）；票 06 的研究筆記；`scripts/experiments/neural-cost/README.md`

## What to build

訓練兩個尺寸的神經重排模型，量它們把正確答案排進前 2 名的能力與成本。

- 開工先問使用者：口語語料（HKCanCor + Common Voice，約 5.6 萬句）不夠時要不要另找來源（plan §8 的口語語料那列）；答覆前可先用現有語料做。
- Gboard 級（約 1.4M）與青簡級（約 23M）各訓練一個，用 plan §4.3 的 train 切分；推論用 candle（M2 驗過能進鍵盤擴充），做前綴快取。
- 重排票 07 定的前 N 條路徑；量指標、模型大小、Mac 上的推論延遲（上下文 0、50、200 字），23M 只量 q8 能否在 60 MB 內。

## 驗收

- [x] README 有訓練設定（語料、步數、時間、機器）、結果表、與門檻比較。
- [x] 訓練產物不進 git；重現步驟寫在 README。
- [x] `cargo xtask check` 綠燈。

## Comments

- 2026-11-11 開工：`fa07bd3`。開工先問口語語料（plan §8），使用者選「先用現有語料」。`bench/personal` 用軟連結接主 checkout 的個人集；`cargo xtask data` 在建軟連結之前跑，沒改到共用檔；協調者交代本票不 `--update-baseline`，`bench/personal/baseline.json` 沒動。
- 2026-11-11 做法（細節在 `scripts/experiments/neural-rerank/README.md`）：
  - 訓練用 PyTorch 2.15.0（MPS，uv 的 PEP 723 inline 依賴、`train.py.lock` 釘版本），推論照票用 candle 0.11.1（`model-ladder` 的 `src/neural.rs`）。兩個模型的結構、權重名與參數量照 `neural-cost`（1,412,250、23,142,400），M2／票 07 量的開銷直接適用；訓練完寫 `reference.json`，candle 載入時核對 PyTorch 的分數（f32 差 ≤ 0.0001 nat）。
  - 語料是四個語料的 train 切分（子句依原文順序，子句間插 `<sep>` 接回連續文字）；評估句（基準、dev、長前文、個人集，≥ 5 字）剔除後只少 3 句。前文裡的標點收成 `<sep>`、英文數字收成 `<unk>`。
  - 接法 `路徑分 + λ·神經分`，前綴快取，路徑長度不一時補齊遮掉（變異驗證：拿掉遮罩測試會紅）。研究筆記的 `λ·(神經分 − 靜態二元分)` 要拆出 bigram 項，`Session::paths` 沒給，沒做。
- 2026-11-11 調參（只看 dev）：LSTM 口語視窗比例掃自然、0.1、0.3、0.5（口語 dev 53.1 → 58.4%），訓練到 8,000 步沒有幫助；GPT 先試 0.3 在第 1500 步後過擬合口語（第 3000 步後停掉），改 0.15 並每 500 步存檢查點，挑第 4000 步（口語 dev 60.1%）。λ 掃 0.25–inf，兩個都定 1。q8 在 dev 和 f32 幾乎一樣。
- 2026-11-11 test 只跑一次（設定定案後，四個：1.4M、23M 各 f32、q8，λ 1；基線同 commit 重量、數字同票 07）。口語前 2 名：基線 49.05%、1.4M 57.54%（翻好 268／翻壞 44）、23M 58.95%（294／33）；23M 比 1.4M +1.4 點（98／61，p 0.004），沒到門檻的 +2.0。個人集首選 0 題翻盤，前 2 名 85.52 → 85.9–86.0%。
- 2026-11-11 iPhone（使用者同意後用 iphone-use 重連）：實驗鍵盤加 `L8·9`、`G8·9` 兩鍵（9 條 × 18 字）。23M q8 上下文 200 字 p95 84.2、84.3 ms、峰值 +75.4、+74.4 MB（兩個程序），超過 80 ms 與 60 MB；1.4M q8 p95 10.7 ms、+2.0 MB。截圖 `.scratch/m3/evidence/09-iphone-*.png`；brief §12.3、plan §8 的推估改成實量。量完按「結束」、回主畫面、釋放手機。
- 2026-11-11 與門檻：1.4M q8 每項都過；23M q8 正確率沒比 1.4M 多 2.0 點，iPhone 延遲與記憶體也超標。前文 200 字比 50 字 ±0.1 點。採用與否、brief 的寫回由票 11 定。
- 2026-11-11 code-review（`fa07bd3...02acc67`；Standards 讀 `CLAUDE.md`、全域偏好、spec〈共同規則〉、plan §0，Spec 讀本票、spec〈模型階梯實驗〉〈採用門檻〉與 model-ladder README）。Spec 軸從逐題結果重算了 README 的 test、dev 數字與 McNemar，全部對得上。已處理：
  - brief 新加的 iPhone 實量直接連實驗 README：先寫進 `docs/research/ios-feasibility.md`〈小型神經模型〉，brief 改連它（spec〈共同規則〉）。
  - Mac 延遲表少了前文 50 字一欄，且「前文200」實際的前文 p50 只有 37 token：補欄並註明真的 200 字看 iPhone 表。
  - 沒說明口語長前文不接、洩漏防線只做一條的理由：寫進 README〈限制〉。
  - 門檻表 23M 的第一列「過」容易誤讀：改成「對基線」並指向「比 1.4M 再多 2.0 點」那列；+11.1 改 +11.05。
  - `latency_table` 自己重寫 `Group::is_context` 的判斷：改公開直接用；`check_reference` 改用已讀出的 `Format`，不再看副檔名；Swift 探針的三元 tuple 改成具名的 `Model`；`train.py` 關掉 log 檔、拿掉沒用的變數。改完 model-ladder 測試、clippy、fmt 與 Xcode 建置都過。
- 2026-11-11 沒處理的發現：
  - `neural.rs` 約 300 行與 `neural-cost/src/model.rs` 同形：實驗 crate 各自獨立（spec〈共同規則〉），本票的版本從權重讀形狀、加了補齊遮罩；iPhone 量的是 neural-cost 那份，形狀相同、開銷相同。票 11 若採用 1.4M，產品 crate 只留一份（plan §2.7）。
  - `reference.json` 濾掉 `<unk>`，PyTorch 與 candle 的核對沒蓋到 `unk_penalty`（純 Rust 的加法）。
  - `SEP` 與名詞表的「分隔記號」（`SEPARATOR`）名字相近、意思不同：只在實驗碼，不改。
  - `score_one` 遇到空路徑會 panic：只有核對與測試呼叫，`score` 已擋空路徑。
  - `02acc67` 的 commit body 有兩行 74 字元（上限 72）：改寫要動已有的 commit，沒改。
