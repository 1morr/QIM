# 02: 補欄位輸入框與排序原因的文件

Status: done
Blocked by: None
讀: `web/src/host.ts`、`web/src/field.ts`；`docs/agents/ticket-template.md`

## What to build

兩條做票時才會踩到的知識還沒寫下（arch-diet 票 06 第七題，使用者 2026-11-18 同意）：

- 欄位類型的輸入框為什麼用 `textarea`（`type=email`、`number` 的 `input` 沒有 `selectionStart`、`setRangeText`）：寫在 `web/src/host.ts` 的 `Editable` 型別旁；`field.ts` 不碰輸入框，它的檔頭加一句指過去（開工對程式碼補正，使用者 2026-11-18 同意）。
- `docs/agents/ticket-template.md` 加一句：新的排序原因要在 `Inspection`（`crates/qim-core/src/session/inspect.rs`）與 debug 面板加一欄、測試讀回。

## 驗收

- [x] `host.ts` 的 `Editable` 旁寫了原因，`field.ts` 指過去。
- [x] `ticket-template.md` 有那一句。
- [x] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments

Session 紀錄：2026-11-20 `host.ts` 的 `Editable` 寫明輸入框用 `textarea` 的原因（HTML 規範：email、number 的 `input` 不支援選取 API），`field.ts` 檔頭指過去；票範本的驗收段加排序原因要進 `Inspection` 與 debug 面板的提醒
決定：範本那句寫成驗收段開頭的 HTML 註解，不寫成勾選框；每張票都會複製範本，勾選框忘了刪會留下勾不掉的驗收條件（Agent 定）
決定：原因寫 HTML 規範的行為，不另開事實編號；〈事實與查證〉管的是輸入法框架、碼表與外部引擎，不含 DOM（Agent 定）
code-review 未處理：範本寫死了 `inspect.rs` 的路徑，檔案搬家時不會被檢查擋下；票本身指定了這個路徑，所以保留
變更：無
