# 08: 階梯：同步層的 trigram 與詞級 n-gram

Status: ready-for-agent
Blocked by: 07
讀: spec〈模型階梯實驗〉〈採用門檻〉〈共同規則〉；plan §2.3、§4.3；brief §12.2（librime-octagram）；`docs/research/cyime.md` §3；`.scratch/m1/issues/24-m1-close.md` Comments 的「M3」各條（Common Voice、KN 那條）

## What to build

量同步層換語言模型值不值得。

- 字級 trigram、詞級 n-gram：同一份預算（≤ 8 MB）比較，再看放寬到 M2 上限（峰值增量 ≤ 60 MB）的增益；另量一份不設預算、可選下載的詞級 trigram。
- 同時看 M1 留下的兩件事：KN 在乘權重的次數上算、口語幾乎不平滑（比照標準 KN 重訓比一次）；Common Voice 因「說」「記」同碼 `yu` 多錯的題換模型後有沒有改善。
- 每個模型記大小、載入時間、Mac 上每鍵延遲 p50 / p95，與 spec〈採用門檻〉比。

## 驗收

- [ ] README 的結果表每列一個模型：各集合的三個指標、大小、延遲；標出過不過門檻。
- [ ] 訓練用的語料與授權註明（brief §11）。
- [ ] `cargo xtask check` 綠燈。

## Comments
