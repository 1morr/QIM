# 09: 階梯：停鍵層的小型神經模型

Status: ready-for-agent
Blocked by: 07
讀: spec〈模型階梯實驗〉〈採用門檻〉〈共同規則〉；plan §2.7、§4.3、§8（小型神經模型那列）；brief §12.3（candle、記憶體）；票 06 的研究筆記；`scripts/experiments/neural-cost/README.md`

## What to build

訓練兩個尺寸的神經重排模型，量它們把正確答案排進前 2 名的能力與成本。

- 開工先問使用者：口語語料（HKCanCor + Common Voice，約 5.6 萬句）不夠時要不要另找來源（plan §8 的口語語料那列）；答覆前可先用現有語料做。
- Gboard 級（約 1.4M）與青簡級（約 23M）各訓練一個，用 plan §4.3 的 train 切分；推論用 candle（M2 驗過能進鍵盤擴充），做前綴快取。
- 重排票 07 定的前 N 條路徑；量指標、模型大小、Mac 上的推論延遲（上下文 0、50、200 字），23M 只量 q8 能否在 60 MB 內。

## 驗收

- [ ] README 有訓練設定（語料、步數、時間、機器）、結果表、與門檻比較。
- [ ] 訓練產物不進 git；重現步驟寫在 README。
- [ ] `cargo xtask check` 綠燈。

## Comments
