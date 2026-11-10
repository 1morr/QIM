# 20: 資料包 manifest 與設定頁的資料包管理

Status: ready-for-agent
Blocked by: 01, 02
讀: plan §1.2、§4；brief §5.1、§5.3、§10；spec〈範圍〉〈共同規則〉

## What to build

每份資料包帶一份 manifest；網頁設定頁看得到裝了哪些資料包，可切換、刪除，也可從檔案匯入一份。

- manifest：id、種類、版本、最低核心版本、sha256、大小、授權（spec〈範圍〉）。這是對外格式，先看成熟產品（Rime 的 plum / `rime-install`、Gboard 的語言包、Mozc 的字典）定欄位與檔名，註明採用誰的慣例，寫回 plan §4；`qim-databuild` 產生，`--verify` 檢查。
- 核心載入時檢查最低核心版本與 sha256，不合時拒絕並給明確錯誤。
- 網頁設定頁：列出資料包（名稱、版本、大小、授權）、切換（例如換一份詞庫）、刪除、從檔案匯入；網頁存在瀏覽器裡（儲存方式在票內定）。

## 驗收

- [ ] 每份資料包有 manifest；改壞 sha256 或要求更新的核心版本時載入被拒（測試）。
- [ ] Playwright：從檔案匯入一份資料包、切換、刪除，每步讀回候選有變；截圖。
- [ ] plan §4 寫明 manifest 格式；`cargo xtask check` 綠燈。

## Comments
