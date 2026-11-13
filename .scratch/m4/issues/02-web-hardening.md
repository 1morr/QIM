# 02: 網頁修補：換模型競態、模型載入、IndexedDB

Status: done
Blocked by: 01
讀: plan §2.7、§3.1（停鍵層、資料包、使用者資料）；`.scratch/m3/issues/23-m3-close.md` Comments 的 M4 各條；spec〈共同規則〉

## What to build

先把 M3 收尾留下的網頁問題修掉，再往上加新功能（spec〈細節〉「先重構 `main.ts` 再加新功能」）：

- `main.ts` 換模型 A→B 時，A 晚失敗會清掉 B 的 `loadedModel` 並誤報。模型的載入與核對搬進 `NeuralReranker`，競態在那裡處理。
- `try … finally snapshot.free()` 五處收成一處。
- IndexedDB 打不開時 `start()` reject，網頁打不了字。改成照樣能打字：學習資料只留在記憶體，狀態列說明這次不會存。

排在票 01 後面，因為兩張都改 `main.ts`。

## 驗收

- [x] 先寫會紅的測試：換模型 A→B、A 晚失敗時 B 照樣載入、不誤報；修後綠。
- [x] 模型載入與核對在 `NeuralReranker`，`main.ts` 不再管 `loadedModel`；`snapshot.free()` 只剩一處；`main.ts` 行數不比開工時多。
- [x] Playwright：讓 IndexedDB 打不開（stub `indexedDB.open` 失敗）時照樣打得出字、狀態列有說明；先紅再修。
- [x] （協調者 2026-11-12 加）Playwright：`localStorage` 讀寫丟錯時照樣打得出字、狀態列說明這次不會存；先紅再修。
- [x] 實際操作亂按一輪；`cargo xtask check` 綠燈。

## Comments

- 2026-11-12 票上寫錯：「IndexedDB 打不開時學習資料只留在記憶體」，實際學習資料、設定、每種資料包用哪份都在 `localStorage`，IndexedDB 只放匯入與線上更新下載的資料包。照實際架構修 IndexedDB；協調者把 `localStorage` 打不開也納入本票（上面加了一條驗收）。
- 2026-11-12 IndexedDB 打不開（`createStore` 當下就呼叫 `indexedDB.open`，可能直接丟錯，所以改在 `PackLibrary.open` 裡建）：只用內建的資料包，`PackLibrary.storeError` 記原因、狀態列說明；這次匯入或下載的照樣換上、只留在記憶體（`change` 不寫 IndexedDB），重新整理後回到內建的。plan §4.6 已補。
- 2026-11-12 `localStorage`：新的 `web/src/storage.ts`（`safeStorage`），各模組改經 `storage` 讀寫。第一次出錯（讀 `window.localStorage` 就丟 `SecurityError`，或寫入 `QuotaExceededError`）起，寫入只留在記憶體、讀先看記憶體；任何一個鍵寫不進去（例如回報清單撐爆配額）學習資料也跟著只留記憶體，取捨是不讓一部分存了一部分沒存。plan §3.1、`docs/development.md`〈網頁〉已補。
- 2026-11-12 `NeuralReranker`：Worker 與核對由建構子給（單元測試換成假的，不必先跑 `cargo xtask web`），`load`（開頁時，不在主執行緒核對）與 `prepare`（換資料包時先核對，回傳交給 Worker 的函式）；只有最後一次載入的失敗算數。`checkModelPack` 搬到 `packs.ts`（`neural-rerank.ts` 不在執行期 import 綁定）。
- 2026-11-12 `snapshot.free()`：`main.ts` 的五處收成 `withSnapshot` 一處（驗收句指 M3 票 23 說的 `main.ts` 五處；`settings.ts`、`touch.ts` 各自模組裡的一處不在範圍）。`main.ts` 402 → 392 行。
- 2026-11-12 狀態列：常駐說明（只用內建的資料包、存不了、停鍵層模型載入不了）收成 `main.ts` 的 `notify`，載入完才換掉「載入中…」、之後附加不蓋掉。修前模型在載入中失敗的說明會被載入完清掉、載入後失敗會蓋掉其他說明（code-review 找到）。
- 2026-11-12 先紅：`web/test/neural-rerank.test.ts` 在「原樣搬進 `NeuralReranker`」的版本紅兩例（A 晚失敗誤報、Worker 出錯報兩次），加上載入計數後綠；`web/e2e/storage.spec.ts` 的四條儲存測試在開工 commit `917d2a3` 的程式上全紅（網頁停在載入中，`data-ready` 沒設），改後綠；「兩條說明都留著」在加 `notify` 前紅（只剩模型的說明）、加後綠。另有「IndexedDB 打不開時匯入資料包」一條。
- 2026-11-12 實際亂按（桌面 1280 寬）：組字中夾退格、Esc、單按 Shift、數字選字、翻頁、↑↓、Tab、Enter、快速連按、debug 面板、重排四種模式、換詞庫再換回；IndexedDB 與 `localStorage` 都封鎖時打字、回報、切開關（簡體、無痕等）。沒有 pageerror、沒找到新問題；console 只有 favicon 與線上更新索引的 404（開工前就有）。截圖 `.scratch/m4/evidence/02-monkey.png`、`02-storage-blocked.png`。
- 2026-11-12 沒動核心、排序與資料包，不附 bench 與延遲。
- 2026-11-12 code-review 沒處理的（判斷題）：`checkModelPack` 放 `packs.ts` 而不是和 `NeuralReranker` 一起（留著：`neural-rerank.ts` 要保持不 import 綁定）；`PackLibrary` 的 `store`、`storeError` 可收成一個聯合型別（兩欄位已夠清楚，不改）；IndexedDB 打不開時匯入，`qim.packs.active` 照樣記了一份重新整理後不在的資料包，重新整理退回內建的、不影響打字，不改。

Session 紀錄：換模型競態與模型載入搬進 `NeuralReranker`、`snapshot.free()` 收一處；IndexedDB 或 `localStorage` 打不開照樣能打字（只留記憶體、狀態列說明）
