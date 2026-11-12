# 19: Gboard 英文鍵盤黑箱實測

Status: ready-for-agent
Blocked by: 18
讀: brief §4.7、§12.4；spec〈細節〉 第二期；`docs/research/blackbox-baseline.md`（量法參考）；skill `orca-emulator-android`

## What to build

在 Android 模擬器上實測 Gboard 的英文鍵盤，第二期的英文模式、自動修正、欄位類型、智慧標點照它定：

- 自動修正的時機（空白、標點、選建議）、退格還原、還原後再打空白會不會又修。
- 連按兩下空白、句首大寫、成對符號（插一對、游標放中間，還是左右輪流）。
- 建議列：幾個、排法、使用者打過的詞何時出現。
- 欄位類型：email、網址、數字欄出什麼鍵盤、開不開自動修正。

寫到 `docs/research/gboard-english.md`（Gboard 版本、步驟、截圖、結論），摘進 brief §12.4。

## 驗收

- [ ] 每項有步驟、Gboard 版本、截圖與結論。
- [ ] 摘進 brief §12.4；列出影響票 21、25、26、27 的決定。
