# 03: 網頁前置整理：e2e 共用 helper、free() 寫法、undoWindow 改名

Status: ready-for-agent
Blocked by: None
讀: `ARCHITECTURE.md`〈測試策略〉；`docs/development.md`〈網頁〉；`.scratch/m4/issues/31-m4-close.md` 的 Comments

## What to build

M4 留下的網頁整理（M4 票 31），先做好，M5 後面改網頁與寫 e2e 的票才容易（開工對程式碼補正，使用者 2026-11-18 同意）：

- e2e helper：`ready` 20 檔、`ui` 17 檔、`shown` 15 檔、`place` 3 檔各自定義，讀面板機率（`data-probability`）在 `physical-weighted`、`touch-weighted` 兩檔。各檔的回傳值與定位器不完全一樣（例如有的 `ui` 多 `panel`、`expand`，有的 `ready` 多收 `errors`）：共用模組只收共通的部分，各 spec 特有的欄位留在 spec、用組合擴充。行為不變。
- 釋放 wasm 物件的寫法現在有三種：`withSnapshot`（`main.ts`）、手寫 try/finally（`settings.ts`、`touch.ts`、`field.ts`）、直接 `.free()`（約 20 處，含 `bench/latency.ts`）。收成一種，範圍含 Snapshot 以外用完就釋放的 wasm 物件（`Pack`、`Segment` 這類）；選哪一種寫在 Comments（Agent 定）。
- `undoWindow` 也當兩下空白的計時器：函式改成通用的名字，檔名 `undo-window.ts`、測試 `test/undo-window.test.ts` 跟著改；時限由呼叫端給（退格撤銷的 3000 ms 與兩下空白的 `DOUBLE_SPACE_WINDOW_MS` 各自傳），`UNDO_WINDOW_MS` 留在退格撤銷的呼叫端。

純重構：使用者看得到的行為不變，e2e 的測試數不減。

## 驗收

- [ ] e2e 共通的 helper 從共用模組來，各 spec 只留自己特有的部分。
- [ ] 網頁原始碼只剩一種釋放 wasm 物件的寫法。
- [ ] 計時器改了名，檔名、測試、呼叫端跟著改，時限由呼叫端給。
- [ ] e2e 測試數不少於開工時。
- [ ] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments
