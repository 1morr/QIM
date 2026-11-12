# 29: 譯詞資料包

Status: ready-for-agent
Blocked by: 20, 18b
讀: brief §1.5、§5；plan §2.1、§4.6（manifest）；票 20 的結論；spec〈共同規則〉

## What to build

打中文時看得到並能上屏英文譯詞：

- 照票 20 的來源建可選資料包，走 M3 的 manifest，加一種 `PackKind`；以繁體原文為鍵，簡體模式查原文。
- 候選旁顯示譯詞（網頁候選列）。
- 修飾鍵加數字上屏譯詞（青簡：macOS ⌥、Windows Ctrl）；觸控怎麼上屏跟 M5 一起定。
- 網頁資料包頁可裝、可刪。

## 驗收

- [ ] `cargo xtask data --verify` 決定性；新 `PackKind` 有測試；`LICENSE`、`NOTICE` 只寫用到的來源。
- [ ] Playwright：候選旁讀到譯詞；修飾鍵加數字上屏譯詞；簡體模式照樣有；刪掉資料包後不顯示；截圖。
- [ ] plan §2.1、§4.6 更新；`cargo xtask check` 綠燈。
