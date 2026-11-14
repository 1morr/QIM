# 19: Gboard 英文鍵盤黑箱實測

Status: done
Blocked by: 18
讀: brief §4.7、`docs/research/facts-products.md`；spec〈細節〉 第二期；`docs/research/blackbox-baseline.md`（量法參考）；skill `orca-emulator-android`

## What to build

在 Android 模擬器上實測 Gboard 的英文鍵盤，第二期的英文模式、自動修正、欄位類型、智慧標點照它定：

- 自動修正的時機（空白、標點、選建議）、退格還原、還原後再打空白會不會又修。
- 連按兩下空白、句首大寫、成對符號（插一對、游標放中間，還是左右輪流）。
- 建議列：幾個、排法、使用者打過的詞何時出現。
- 欄位類型：email、網址、數字欄出什麼鍵盤、開不開自動修正。

寫到 `docs/research/gboard-english.md`（Gboard 版本、步驟、截圖、結論），摘進 `docs/research/facts-products.md`。

## 驗收

- [x] 每項有步驟、Gboard 版本、截圖與結論。
- [x] 摘進 `docs/research/facts-products.md`；列出影響票 21、25、26、27 的決定。

## Comments

- 紀錄在 `docs/research/gboard-english-2026-11-15.md`（Gboard 18.5.10、Android 17 模擬器、Chrome 145，預設設定逐項讀回），截圖與事件 `.scratch/m4/evidence/19-*`，測試頁與點鍵腳本 `scripts/experiments/gboard-english-blackbox/`。事實 M47、M48（產品）、P24（`inputType` 常數與 Chrome 各欄位的值），brief〈事實索引〉條數跟著改。
- 給後續票（細節在紀錄的〈給後續票的結論〉）：票 21，Gboard 在 Chrome 不留組字區，所有改寫都是「刪游標前 N 字再上屏」，`.` 是詞界、詞裡有 `@` 不建議不修，空白後預測下一個詞。票 25，觸發是空白、逗號、句點、Enter、點建議；還原有三條路（修完的復原鈕、點建議列左格的原字、預設關的「退格還原」子設定），只有復原鈕不教原字。票 26，email、網址欄不建議不修不大寫，但 Gboard 在粵語速成下**不切英文**，brief 的「預設英文模式」要另找依據。票 27，Gboard 英文與粵語都**沒有成對符號自動配對**，「照 Gboard 實測定」照不了，要另找產品或請使用者定；兩下空白約 1 秒內、刪一個空白上屏「. 」。
- emulator-5554 的 Gboard 資料被清、粵語速成佈局需重加（協調者已轉使用者）。改用自建模擬器後沒再對 5554 送過指令；紀錄裡的每一項都在自建的 `emulator-5560` 上量。
- 自建的 AVD `qim_t19_gboard`（裝了 Gboard 18.5.10，加了粵語粵拼、速成）量完已關機，AVD 留著給換版本時重量；不要了可用 `avdmanager delete avd -n qim_t19_gboard` 刪掉。
- 全部檢查：`cargo xtask check` 通過（含 e2e 286 passed；本 worktree 先 `npm ci`、`cargo xtask data` 建資料包）。
- code-review（`6320fe0` 起，Standards／Spec 兩軸）：已處理：截圖列號對錯（field-strips 改成第 1–5 列各是什麼）；句首大寫與保守條件補截圖與事件紀錄（保守條件用新字重跑一輪）；README 補日期、機器、版本、commit、結果表；給票 25 的門檻比較改成照 spec〈英文使用者詞〉、修正目標留給票定；補測粵語速成下 email、網址欄不切英文；Chrome 的 `inputType` 補成事實 P24；腳本 docstring 改中文；設定截圖補 Preferences 與 Privacy。重跑時另發現漏看的復原鈕與「Undo auto-correct on backspace」子設定，已補進〈退格與還原〉與 M47。未處理：`page/index.html` 與 `p2.html` 的樣式與腳本重複（一次性實驗頁，不抽共用檔）；事件欄位名 `it`、`sel` 沒改（留存的事件紀錄用這兩個名字）；M47、M48 的來源照 M44 寫路徑、不做連結。

Session 紀錄：在自建的 Android 模擬器上黑箱實測 Gboard 18.5.10 英文鍵盤，寫成研究紀錄與事實 M47、M48、P24，列出給票 21、25、26、27 的結論。
決定：改用本票新建的 AVD 並側載 Gboard 18.5.10，共用的 emulator-5554 量到一半被別的 session 佔用（Agent 定）
決定：emulator-5554 的 Gboard 資料用 `pm clear` 清掉後重設預設輸入法，照協調者指示（Agent 定）
決定：研究紀錄照〈文件〉凍結紀錄的檔名寫成 `gboard-english-2026-11-15.md`，不用票上的 `gboard-english.md`（Agent 定）
決定：測試頁與點鍵腳本留在 `scripts/experiments/gboard-english-blackbox/`，換 Gboard 版本時重量（Agent 定）
決定：結論摘成兩條產品事實 M47、M48，Chrome 的 `inputType` 另寫平台事實 P24（Agent 定）
