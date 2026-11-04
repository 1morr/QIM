# 23: 展開面板、實體鍵位、重排假實作

Status: ready-for-agent
Blocked by: 20, 21
讀: plan §2.1（`rerank_request`、`apply_rerank`、`generation`、`ToggleAscii`）、§2.7、§3.1；brief §4.3、§4.4（候選展開面板）；ADR 0005；spec〈共同規則〉；`.scratch/m0/issues/09-m0-close.md` 的 Comments（「M1 票 4」那條）

## What to build

網頁補齊 M1 剩下的互動：候選展開面板一次看全部候選；實體鍵盤模式照 brief §4.3 的鍵位全部可用（含單按 Shift 切中英）；重排介面有假實作，證明背景結果只能動到第 2 位、過期結果丟掉。

- 候選展開面板：觸控候選列右側展開鍵，桌面也能叫出（方式在本票定）；面板裡選字照常。
- 實體鍵位：逐項對照 brief §4.3 的表補齊，`ToggleAscii`（單按 Shift）切英文直通，候選列顯示目前狀態。
- 重排：`Session::rerank_request`（上下文 + 前 k 個候選）與 `apply_rerank(generation, order)`；只接受目前的 `generation`，重排過的候選最高到第 2 位（plan §2.1、ADR 0005）；`Snapshot.generation` 每次候選變動 +1。
- 網頁的假實作：延遲一段時間後原樣回傳（plan §3.1），另有一個測試用的「把第 5 個提到最前」版本驗證上限。
- 翻頁中收到重排結果怎麼處理、`generation` 在翻頁時加不加（〈M0 留下的〉），本票定，寫回 plan §2.1。

## 協調者轉入的

- 簡體模式下觸控文字鍵（「常用」頁的「多謝」「唔該」「冇問題」）也跟著轉簡體，用同一份簡繁轉換（使用者 2026-11-04 決定；票 20 的 Heads up）。繁體原文照票 16 的做法保留給回報與學習。
- 組字中 Cmd+V 等帶修飾鍵的組合仍交給瀏覽器，會貼進輸入框而組字還在（票 10 留下）。

## M0 留下的

票 09 Comments「M1 票 4」：翻頁中的重排結果要不要丟、`generation` 怎麼算。

## 驗收

- [ ] 核心測試：過期 `generation` 的 `apply_rerank` 回 `None`；要求把第 5 個提到最前時，結果只到第 2 位，首選不變。
- [ ] Playwright：觸控與桌面都能打開展開面板，讀回含第一頁以外的候選並可選字上屏。
- [ ] Playwright：單按 Shift 後打字直接輸出英文，再按一次回中文；brief §4.3 表上每個鍵都有一項讀回。
- [ ] Playwright：簡體模式下點「常用」頁的「多謝」，輸入框讀回「多谢」；繁體模式讀回「多謝」。
- [ ] plan §2.1 寫明翻頁時 `generation` 與重排的規則；〈M0 留下的〉已處理。

## Comments

- 票 10 留下：組字中 Tab、Delete、↑ ↓、Home、End 經 `Key::Other` 只吃下不動作，要不要給真的行為（Rime：↑ ↓ 選候選、Home / End 移 caret）在本票定；帶 Ctrl / Cmd 的鍵組字中仍交給瀏覽器（Cmd+V 會貼進輸入框而組字還在）。細節見票 10 Comments。
