# 26: 欄位類型

Status: ready-for-agent
Blocked by: 25
讀: brief §4.7、§8（密碼欄）；plan §2.1（`Options`）、§3.1；票 19；spec〈共同規則〉

## What to build

殼告訴核心輸入框的類型（介面由票定，照 Android `EditorInfo.inputType`、iOS `keyboardType` / `UITextContentType` 的慣例，採用誰的寫在 Comments）：

- email、網址：預設英文模式並關自動修正。
- 數字欄：開數字頁。
- 網頁從 `input` 的 `type` / `inputmode` 讀；密碼欄現有的無痕照舊。

## 驗收

- [ ] Playwright：email 欄位預設英文且不修正；網址欄同；數字欄打開是數字頁；一般欄位照舊。
- [ ] 介面寫進 plan §2.1；`cargo xtask check` 綠燈。
