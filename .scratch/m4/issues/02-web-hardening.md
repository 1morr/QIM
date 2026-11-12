# 02: 網頁修補：換模型競態、模型載入、IndexedDB

Status: ready-for-agent
Blocked by: 01
讀: plan §2.7、§3.1（停鍵層、資料包、使用者資料）；`.scratch/m3/issues/23-m3-close.md` Comments 的 M4 各條；spec〈共同規則〉

## What to build

先把 M3 收尾留下的網頁問題修掉，再往上加新功能（spec〈細節〉「先重構 `main.ts` 再加新功能」）：

- `main.ts` 換模型 A→B 時，A 晚失敗會清掉 B 的 `loadedModel` 並誤報。模型的載入與核對搬進 `NeuralReranker`，競態在那裡處理。
- `try … finally snapshot.free()` 五處收成一處。
- IndexedDB 打不開時 `start()` reject，網頁打不了字。改成照樣能打字：學習資料只留在記憶體，狀態列說明這次不會存。

排在票 01 後面，因為兩張都改 `main.ts`。

## 驗收

- [ ] 先寫會紅的測試：換模型 A→B、A 晚失敗時 B 照樣載入、不誤報；修後綠。
- [ ] 模型載入與核對在 `NeuralReranker`，`main.ts` 不再管 `loadedModel`；`snapshot.free()` 只剩一處；`main.ts` 行數不比開工時多。
- [ ] Playwright：讓 IndexedDB 打不開（stub `indexedDB.open` 失敗）時照樣打得出字、狀態列有說明；先紅再修。
- [ ] 實際操作亂按一輪；`cargo xtask check` 綠燈。
