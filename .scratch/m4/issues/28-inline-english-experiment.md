# 28: 句中英文混輸實驗

Status: ready-for-agent
Blocked by: 22
讀: brief §4.7、§12.1；plan §2.3、§5.2；spec〈細節〉（句中英文混輸）；票 22；spec〈共同規則〉

## What to build

在 `scripts/experiments/inline-english/` 量「今日有meeting」這類一串輸入碼夾英文詞值不值得做：

- 英文詞帶懲罰進 lattice，只收拼對的常見詞、不容錯，並設最短長度。
- 量純中文集（`bench` 各集合）的首選與修正次數，與含英文詞的句子（從語料取）的收益；懲罰只在 dev 調。
- 採用門檻先寫在 Comments 再跑：純中文集的首選與修正次數不退步才採用（brief §4.7）。

結論寫回 brief §4.7、§1.5；採用的話補一張實作票進 spec〈票序〉、擋收尾票。

## 驗收

- [ ] 門檻先寫在 Comments；README（日期、機器、commit、量法、結果表）。
- [ ] 結論寫回 brief；採用的話實作票已補。
