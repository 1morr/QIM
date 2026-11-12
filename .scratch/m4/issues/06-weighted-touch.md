# 06: `Key::Weighted` 與觸控空間模型

Status: ready-for-agent
Blocked by: 01
讀: plan §2.1（按鍵的入口、觸控機率）、§2.6；spec〈細節〉〈空間模型〉；brief §4.8；`docs/research/spatial-correction-swipe.md`（空間模型的段落）；spec〈共同規則〉

## What to build

觸控的每次點擊變成各字母的機率，供之後的容錯用：

- `Key` 加 `Weighted`（一組「字母, 機率」），`Key::parse` 收得下；綁定照「種類 + 值」的入口傳。
- `qim-layout` 依點擊位置與按鍵幾何算機率：點擊到每個鍵形狀的距離（旋轉、多邊形、橢圓只是距離函式不同），套 2D Gaussian，σ 跟該鍵的大小成比例並有手指精度的下限（VelociTap、FFitts），單位用實體尺寸；重疊的鍵各自算，最上層加成。AOSP 的 `ProximityInfo` 假設矩形與共用尺度，不搬。參數的初值附來源，之後由容錯的票在 dev 調。
- 網頁觸控的字母鍵送 `Weighted`。容錯進核心之前，核心只用機率最高的字母，打字行為不變；debug 面板顯示前幾名的機率。
- 核心不知道佈局（plan §1.3）。

## 驗收

- [ ] `qim-layout` 性質測試：點在鍵中心時該鍵機率最高；離得越遠機率越低；旋轉、縮放後結果一致；重疊時上層加成；機率和為 1，只回前 N 個（N 由票定）。
- [ ] `Key::parse` 收得下 `Weighted`，壞格式回錯誤。
- [ ] 現有觸控 e2e 照綠；Playwright 點兩鍵中間，debug 面板讀到兩個字母的機率；截圖。
- [ ] plan §2.1「觸控機率」改成實際的介面；`cargo xtask check` 綠燈。
