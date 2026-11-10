# 21: 資料包線上更新

Status: ready-for-agent
Blocked by: 20
讀: plan §4；brief §1.5、§5.3、§8；票 20 的 manifest 格式；spec〈範圍〉〈共同規則〉

## What to build

網頁可以檢查有沒有新版資料包、下載、核對 sha256 後換上；可以關掉。

- 更新來源是 GitHub Releases（`1morr/QIM`，使用者 2026-11-11 決定）：定 release 上放什麼（一份索引列出各資料包的 manifest 與下載網址），寫回 plan §4。
- M3 用本機靜態伺服器扮演 release 跑完整流程；`xtask` 或腳本產生那份索引。真的在 GitHub 建 release 前先問使用者，不同意就只留本機流程並註明。
- 核心打字不依賴網路（brief §8）：更新在背景，失敗或斷網時照用舊的；設定區有開關。

## 驗收

- [ ] Playwright：本機伺服器放一份新版資料包，網頁檢查到、下載、換上，讀回候選變了；sha256 不符時拒絕並保留舊版；關掉開關時不連線（攔請求確認）。
- [ ] 斷網（攔掉請求）時打字照常。
- [ ] plan §4、`docs/development.md` 已更新；`cargo xtask check` 綠燈。

## Comments
