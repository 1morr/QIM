# 26: 數字後的句點與逗號用半形

Status: ready-for-agent
Blocked by: None (can start immediately)
讀: brief §4.3（標點預設全形）、§4.7（智慧標點）、§7；plan §2.1（`Options`）、§7.5（M4 的智慧標點）；票 10 的 Comments（全形對照）；spec〈共同規則〉

## What to build

使用者 2026-11-05 試用後決定：把 brief §4.7「智慧標點」裡「數字後的句點與逗號用半形」這一項從 M4 提前到現在做，預設開、可關。像 Rime 一樣，打「3.5」「1,000」時不出「3。5」「1，000」。

- 規則先查 Rime 怎麼做（`punctuator` 的 `digit_separators`、`digit_separator_action`，librime 1.8 起；context7 或原始碼只取做法），照它定：緊接在半形數字後的 `.`、`,`（Rime 還包括 `:` 的話一併評估）輸出半形；之後若不是數字要不要改回全形，照 Rime。結論記進 brief §12。
- 判斷「前一個字元是數字」用核心自己最近上屏的內容即可（還沒有輸入框上下文，`set_context` 在 M3）；寫明這個限制。
- 設定：`Options` 加開關（預設開），網頁加開關；brief §7 的開關清單補上。M4 的其餘智慧標點（成對符號、英文雙空白句點）不在本票，plan §7.5 改成「數字後半形已在 M1 票 26 做」。
- 同輪改 brief §4.7、§7、plan §2.1、§7.5；progress〈偏差與決定〉記一行（使用者決定）。

## 驗收

- [ ] 核心測試：打 `3` `.` `5` 上屏「3.5」；打 `1` `,` `000` 上屏「1,000」；不在數字後的 `.` 仍是「。」；關掉開關後數字後也是「。」。
- [ ] Playwright：網頁輸入框讀回「3.5」「1,000」；關掉開關讀回「3。5」。
- [ ] brief、plan 已更新；`cargo xtask check`、`bench --check` 綠燈。

## Comments
