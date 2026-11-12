# 21: 英文模式 ADR 與 `delete_before`

Status: ready-for-agent
Blocked by: 19, 18b
讀: brief §4.7、§12.3；plan §1.3、§2.1（`Snapshot`、英文直通、臨時英文）、§4.4；spec〈細節〉〈英文模式〉；票 19 的結論；spec〈共同規則〉

## What to build

核心的英文直通把鍵交還宿主；英文模式要核心自己組字、出建議，是新的組字狀態。寫一份 ADR，一起定：

- 英文模式的組字狀態，和中文組字、英文直通、臨時英文的關係；桌面的英文仍是直通（spec〈細節〉）。
- `Snapshot` 加「先刪游標前 N 字再上屏」（例如 `delete_before`），給自動修正還原、連按兩下空白、成對符號用；殼照做，不自己判斷（plan §1.3）。先查證 iOS `deleteBackward()`、Android `deleteSurroundingText`、macOS IMK `replacementRange` 的行為，寫進 brief §12.3。
- 英文的切詞：`@`、`.` 這類字元算不算詞內（email 要整串是一個詞，LatinIME 以 `.` 分隔）。
- 「預測」包不包括下一個詞。
- 自動修正、句首大寫、英文容錯在 `Options` 的欄位與預設（brief §7）。

ADR 之外，`Snapshot` 加上這個欄位、網頁照做（核心先只在測試裡送），之後的票直接用。

## 驗收

- [ ] ADR 寫進 `docs/adr/`，各項決定附成熟產品的做法（LatinIME、Gboard 實測、librime）。
- [ ] brief §12.3 加三個平台刪除游標前文字的查證與來源。
- [ ] `Snapshot` 有 `delete_before`，網頁照做（測試掛鉤或單元測試驗）；plan §2.1 更新；`cargo xtask check` 綠燈。
