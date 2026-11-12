# 12: 學習資料 v3 格式與日期

Status: ready-for-agent
Blocked by: None (can start immediately)
讀: plan §1.3、§2.4（tick、匯出格式）；spec〈細節〉（學習時限與上限、同步）；brief §4.5、§5.3、§7；`docs/research/learning-forgetting-sync.md`；`docs/research/userdict-text-formats.md`；spec〈共同規則〉的對外格式

## What to build

學習資料的時限、英文使用者詞與同步都要新的欄位與列種類，v2 已定稿，所以先升 v3，把整條路接通：

- 殼交給核心今天的日期（介面由票定，例如 `Session` 或 `UserModel` 的方法）；核心不讀系統時間（plan §1.3）。每個條目記最後使用的日期。
- 匯出升 `# qim-userdict v3`：每列有明確的種類欄（未知種類略過並警告，已知種類照舊嚴格檢查）與最後使用的日期；檔頭記裝置 id，由網頁產生並存起來。
- 匯入照收 v1、v2；沒有日期的列取匯入當天，或其他做法，由票定並寫明理由。匯入暫時照舊取代整份，改成合併在票 17。
- 先對照 librime userdb、LatinIME 的做法（研究筆記已有），採用誰的慣例寫在 Comments。

v3 在 M4 收尾前還沒發佈，之後的票可以往裡加列的種類與欄位，收尾票定稿。

## 驗收

- [ ] 依賴規則守著核心不讀系統時間；測試直接給日期。
- [ ] 匯出 v3 再匯入等價（性質測試）；v1、v2 的檔仍匯入得了；未知種類的列略過並回警告，其他列照收。
- [ ] Playwright：匯出的檔有 v3 檔頭、裝置 id 與日期欄；換一天（測試掛鉤給日期）上屏後日期跟著變。
- [ ] plan §2.4 的匯出格式改寫成 v3 草稿；`cargo xtask check` 綠燈。
