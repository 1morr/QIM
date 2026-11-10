# 10: 鍵盤擴充裡的 Foundation Models 與上下文長度

Status: done
Blocked by: 06
讀: brief §8、§12.3；plan §2.7、§7.8；ADR 0005；spec〈共同規則〉；`docs/research/platform-frameworks.md` §3.2

## What to build

在票 06 的擴充裡回答兩件事：Foundation Models 能不能在鍵盤擴充內用；`textDocumentProxy.documentContextBeforeInput` 實際讀得到多少字。

- Foundation Models：先查官方文件的可用條件（裝置、Apple Intelligence 設定、擴充內是否允許）並補進 brief。擴充裡加一個鍵，送一段固定提示（例如給 10 個候選要它回序號，plan §7.8 的格式），記下能否使用、`availability` 回什麼、首次載入與單次回應的延遲、擴充的常駐記憶體變化。手機沒開 Apple Intelligence 時停下請使用者打開。
- 上下文長度：在備忘錄與另一個宿主（例如 Safari 的文字欄位）貼入長文、游標放在不同位置，擴充讀 `documentContextBeforeInput` 的長度並寫 log 或顯示；看它在換行、段落處是否截斷。
- 結果寫進 `docs/research/ios-feasibility.md` 的〈Foundation Models〉〈上下文長度〉兩節，結論摘進 brief §12.3。

## 驗收

- [x] Foundation Models：明確的能或不能；能的話有延遲與記憶體數字，附機型、iOS 版本與 Apple Intelligence 狀態。
- [x] 上下文：至少兩個宿主、多個游標位置的實測字數；截圖 `.scratch/m2/evidence/10-*.png`。
- [x] brief §12.3 更新並附來源。
- [x] `cargo xtask check` 綠燈。

## Comments

- 2026-11-10 實作：Foundation Models 能用（`available`、AFM 3 Core Advanced、context 8192），不需完整取用；連續請求約 190 ms，停頓後第一個請求 12 次裡 4 次要 6–15 s，`prewarm()` 擋不住，原因沒查出來（iphone-use 讓鍵盤畫面一直重建，可能干擾）。數字見 ios-feasibility〈Foundation Models〉。
- 2026-11-10 實作：Apple Intelligence 沒有要使用者打開：iOS 27 起預設開啟、不能關，`availability` 回 `available`。
- 2026-11-10 實作：上下文大致按句子切、換行不截斷；備忘錄最多 300 字（句中給所在句 + 前一整句），Safari `<textarea>` 最多 100 字（句中只有所在句，段首從上一句中間切 35 字）。剛 `insertText` 後讀到 499 字是暫態。空一行、`<input>`、`contenteditable` 沒測。
- 2026-11-10 實作：Safari 宿主用本機網頁；手機預設瀏覽器是 Edge，開網址時跳出 Edge 的本地網絡權限，使用者決定按「不允許」，改用 Safari 開。測試留下的備忘錄（長文）與 Safari 分頁沒刪，使用者自行處理。
- 2026-11-10 實作：iphone-use 每次點擊或截圖都讓鍵盤 view controller 重建，結果改存在 `ModelProbe` 的靜態狀態；地球鍵單按只在最近兩個鍵盤切換，要長按選 QIM。都寫進實驗 README。
- 2026-11-10 實作：commit 前做了小整理（`run` 改用共用的 `timedRequest`、code-review 後改顯示行與命名）；使用者解鎖手機後在備忘錄重按 FM 確認：889 ms，之後 186–190 ms，都回「3」，沒併進 ios-feasibility 的表。
- code-review（2026-11-10）：數字與措辭（0.29–0.87 s、中位數 189 ms、footprint 區間、Safari 句首與段首的例外、推論標註、缺的來源）與 README 寫錯藏哪幾行都已修；未處理：`ModelProbe.run` 與 `runGaps` 仍有相似的前後量測流程、每次 `textDidChange` 都重算可用性行（實驗碼，不影響數字）。
- 留給 M3（plan §7.8）：brief §8 的「游標前 200 字」在 iOS 可能常拿不到（測試文字只讀到一兩句，真實文字未測）；要不要自己記住這次編輯上屏過的字補上下文，M3 決定。
