# 02: 補欄位輸入框與排序原因的文件

Status: ready-for-agent
Blocked by: None
讀: `web/src/host.ts`、`web/src/field.ts`；`docs/agents/ticket-template.md`

## What to build

兩條做票時才會踩到的知識還沒寫下（arch-diet 票 06 第七題，使用者 2026-11-18 同意）：

- 欄位類型的輸入框為什麼用 `textarea`（`type=email`、`number` 的 `input` 沒有 `selectionStart`、`setRangeText`）：寫在 `web/src/host.ts` 的 `Editable` 型別旁；`field.ts` 不碰輸入框，它的檔頭加一句指過去（開工對程式碼補正，使用者 2026-11-18 同意）。
- `docs/agents/ticket-template.md` 加一句：新的排序原因要在 `Inspection`（`crates/qim-core/src/session/inspect.rs`）與 debug 面板加一欄、測試讀回。

## 驗收

- [ ] `host.ts` 的 `Editable` 旁寫了原因，`field.ts` 指過去。
- [ ] `ticket-template.md` 有那一句。
- [ ] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments
