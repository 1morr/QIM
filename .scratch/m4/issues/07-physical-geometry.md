# 07: 實體鍵盤的鄰鍵幾何

Status: ready-for-agent
Blocked by: 06
讀: plan §2.1、§2.6；spec〈細節〉〈空間模型〉；brief §4.8；`docs/research/ranking-techniques.md` §5.2；spec〈共同規則〉

## What to build

實體鍵盤也產生同一種 `Weighted`，讓容錯對兩種輸入用同一套機制：

- 網頁送 `KeyboardEvent.code`；`qim-layout` 依實體鍵盤的幾何（各排錯位，含上下排與斜角；libime 只取同排左右）轉成 `Weighted`：按下的鍵最高、鄰鍵較低。這是先驗，不是點擊位置；分布的參數由票定，留給容錯的票調。
- 以實體鍵位為準，不受配列影響（換 Dvorak 等配列時照 `code`）。
- 組合鍵、非字母鍵照舊，不送 `Weighted`；核心照樣只用機率最高的字母，行為不變。

## 驗收

- [ ] `qim-layout` 測試：`KeyF` 的鄰鍵含 D、G、R、T、C、V；`key` 值不同但 `code` 相同時結果相同。
- [ ] 現有實體鍵盤的 e2e 照綠；Playwright 按一個字母，debug 面板讀到它與鄰鍵的機率。
- [ ] plan §2.1 寫明實體鍵盤的 `Weighted`；`cargo xtask check` 綠燈。
