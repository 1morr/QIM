# 23: 展開面板、實體鍵位、重排假實作

Status: done
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

- [x] 核心測試：過期 `generation` 的 `apply_rerank` 回 `None`；要求把第 5 個提到最前時，結果只到第 2 位，首選不變。
- [x] Playwright：觸控與桌面都能打開展開面板，讀回含第一頁以外的候選並可選字上屏。
- [x] Playwright：單按 Shift 後打字直接輸出英文，再按一次回中文；brief §4.3 表上每個鍵都有一項讀回。
- [x] Playwright：簡體模式下點「常用」頁的「多謝」，輸入框讀回「多谢」；繁體模式讀回「多謝」。
- [x] plan §2.1 寫明翻頁時 `generation` 與重排的規則；〈M0 留下的〉已處理。

## Comments

- 票 10 留下：組字中 Tab、Delete、↑ ↓、Home、End 經 `Key::Other` 只吃下不動作，要不要給真的行為（Rime：↑ ↓ 選候選、Home / End 移 caret）在本票定；帶 Ctrl / Cmd 的鍵組字中仍交給瀏覽器（Cmd+V 會貼進輸入框而組字還在）。細節見票 10 Comments。
- 決定（寫進 plan §2.1）：翻頁不是候選變動，`generation` 不加；翻頁中收到的重排結果照樣套用。重排要求最多帶第一頁（`RERANK_LIMIT` = 9），所以只動第一頁，看得到的那頁不變。翻頁加 `generation` 的話結果作廢，殼重要求後仍在翻頁中套用，沒有保護到什麼。〈M0 留下的〉票 09「M1 票 4」就此結案。
- `apply_rerank` 多一個 `engine` 參數（回快照要組字的簡繁與字根）。名次真的變了才加 `generation`，同一份要求的另一層結果因此作廢；原樣回傳不作廢。殼只在按鍵等事件之後要求，套用結果後不再要求，免得自己重排自己。
- `RerankRequest.context` 是候選前面的分段的繁體原文，網頁送給模型的候選也用原文（review 發現原本簡體模式下候選是簡體、上下文是繁體）。
- 單按 Shift：組字中先上屏整個組字再切，照 Rime `ascii_composer` 的 `commit_text`；不學 `inline_ascii`（留著組字接英文），要原碼有 Enter。判定在殼（`LoneShift`，500 ms，同 librime），按著 Ctrl / Cmd / Alt 時不算（review 發現 Ctrl+Shift 會誤切）。英文狀態下核心每個鍵都不吃。brief §12.2 補了 librime 的事實。
- 桌面叫出面板用 ↓ / ↑：Microsoft 速成是 ↓ 進候選窗、Tab 或 → 展開成表格，Tab 與 → 這裡已給字根提示與分段（brief §12.4）。面板是殼的 UI，↓ / ↑ 由殼攔下，plan §2.1 寫明。選了字就收起。
- 票 10 留下的 Tab、Delete、↑ ↓、Home、End：↑ ↓ 改開關面板；Delete、Home、End 照舊吃下不動作。我們的「caret」是目前分段，Home / End 對應移到第一段 / 整串，等有人要再加。
- 組字中 Ctrl / Cmd / Alt 組合鍵吃下（`toKey` 多一個 `composing` 參數），e2e 實打 Cmd+V 與 Cmd+A，兩個都做了變異驗證：沒修時讀回「貼」與「門」。代價：組字中 Cmd+C、Cmd+Z、Option＋字母都沒作用，brief §4.3 寫明；Windows 的 AltGr（瀏覽器回報成 Ctrl+Alt）組字中打不出字元，網頁 prototype 先不處理，平台殼開工時看平台的鍵碼再定。
- 協調者轉入的簡體文字鍵：`Session::text` 的文字照選項轉簡體、原文留繁體，票 20 的核心測試改成新行為。
- 網頁的「重排」選單是重排的開關（brief §7 的雲端候選，預設開）：關、原樣（預設）、第 5 個提前（測上限用）。等待中切到「關」，結果就不套用。
- 版面：候選列改成 flex 後，`#candidates` 與 `.candidate-bar` 要 `min-width: 0`，不然手機上候選撐寬版面視窗（412 → 643 px），整頁縮小、觸控 e2e 點不到鍵。
- 亂按：桌面 1,500 步（夾 Shift、↑ ↓、組合鍵、翻頁、切簡繁與重排、點面板）、觸控 600 步，沒有頁面錯誤，組字為空時沒有候選、面板與展開鍵。
- bench 前後對照：改動前 = `bench/baseline.json`（HEAD）；改動後 `cargo xtask bench --check` 通過，沒有翻盤、固定題 15/15，預設路徑的排序沒動。
- code-review 未處理：「原樣回傳」的 e2e 只看得出順序沒變，分不出結果是否送達（原樣本來就看不出），送達與丟棄由核心測試驗。`main.ts` 的 `core` 與 `start()` 的 `session` 是同一個物件兩個名字：面板與重排在 `start()` 外要用它，網頁 prototype 先不重構。
