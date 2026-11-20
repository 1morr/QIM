# 03: 網頁前置整理：e2e 共用 helper、free() 寫法、undoWindow 改名

Status: done
Blocked by: None
讀: `ARCHITECTURE.md`〈測試策略〉；`docs/development.md`〈網頁〉；`.scratch/m4/issues/31-m4-close.md` 的 Comments

## What to build

M4 留下的網頁整理（M4 票 31），先做好，M5 後面改網頁與寫 e2e 的票才容易（開工對程式碼補正，使用者 2026-11-18 同意）：

- e2e helper：`ready` 20 檔、`ui` 17 檔、`shown` 15 檔、`place` 3 檔各自定義，讀面板機率（`data-probability`）在 `physical-weighted`、`touch-weighted` 兩檔。各檔的回傳值與定位器不完全一樣（例如有的 `ui` 多 `panel`、`expand`，有的 `ready` 多收 `errors`）：共用模組只收共通的部分，各 spec 特有的欄位留在 spec、用組合擴充。行為不變。
- 釋放 wasm 物件的寫法現在有三種：`withSnapshot`（`main.ts`）、手寫 try/finally（`settings.ts`、`touch.ts`、`field.ts`）、直接 `.free()`（約 20 處，含 `bench/latency.ts`）。收成一種，範圍含 Snapshot 以外用完就釋放的 wasm 物件（`Pack`、`Segment` 這類）；選哪一種寫在 Comments（Agent 定）。
- `undoWindow` 也當兩下空白的計時器：函式改成通用的名字，檔名 `undo-window.ts`、測試 `test/undo-window.test.ts` 跟著改；時限由呼叫端給（退格撤銷的 3000 ms 與兩下空白的 `DOUBLE_SPACE_WINDOW_MS` 各自傳），`UNDO_WINDOW_MS` 留在退格撤銷的呼叫端。

純重構：使用者看得到的行為不變，e2e 的測試數不減。

## 驗收

- [x] e2e 共通的 helper 從共用模組來，各 spec 只留自己特有的部分。
- [x] 網頁原始碼只剩一種釋放 wasm 物件的寫法。
- [x] 計時器改了名，檔名、測試、呼叫端跟著改，時限由呼叫端給。
- [x] e2e 測試數不少於開工時。
- [x] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments

- 2026-11-20 開工時 `npx playwright test --list`：`Total: 328 tests in 55 files`；做完一樣 328。
- 2026-11-20 釋放 wasm 物件只剩 `web/src/free.ts` 的 `freeAfter(item, use)`：把物件（或陣列）交給同步的 `use`，之後釋放、丟錯也釋放；`main.ts` 的 `withSnapshot`、`debug-panel.ts` 的 `using`、手寫 try/finally 與直接 `.free()`（含 `bench/latency.ts`，計時區間不變）都改成它。跨 `await` 的（`pack-update.ts`）先在 `use` 裡讀成一般的值。換掉手上留著的物件（觸控的按鍵、佈局、字根提示、停鍵層的 `Reranker`）先建好新的，`use` 裡只換上它，建新的丟錯時舊的還能用。
- 2026-11-20 e2e 共用模組 `web/e2e/page.ts`：`ui`（輸入框、組字區、候選）、`waitReady`（不換頁）、`ready`（開頁）、`readyCleared`（清 `localStorage` 再載入）、`shown`、`place`、`keyProbabilities`。55 個 spec 都改用它，內聯的 `data-ready` 等待不剩；31 個本地 `ui` / `ready` / `place` 是展開共用的再加自己的（import 時改名 `baseUi` 這類）。`docs/development.md`〈網頁〉補一句。
- 2026-11-20 code-review（`5409891` 起，兩軸各一個子代理）：已修：`UNDO_WINDOW_MS` 的註解改引事實 R20（Standards）；`autocorrect`、`english-mode`、`field-types` 的 `parts` 與 `continuous`、`composition-keys`、`common-forms`、`packs` 內聯的去序號改用共用的 `ui`、`shown`（Spec）；`bench/latency.ts` 遮蔽同名的變數改名。沒處理：`freeAfter` 的型別不擋 async 的 `use`（檔頭註解寫明，現在沒有 async 的呼叫端）；「換掉留著的物件」四處沒另抽 helper（一行的換上，抽了是多一種寫法）；`use` 的參數與外層同名（Biome recommended 不擋，讀的就是同一個物件）。
- 2026-11-20 沒共用、留給之後寫 e2e 的票：19 個 spec 各自收 `pageerror` 的 `errors`、`tapText`（3 檔）、`choose`（5 檔）這類，不在本票列的 helper 裡。
- 2026-11-20 全部檢查第一次跑 e2e 紅 1 個：`emoji.spec.ts`「打『多謝』點 🙏 上屏；簡體模式…」按 2 時上屏了「多诗」（停鍵重排在檢查名次與按鍵之間換了名次）；測試內容沒改，單獨 `--repeat-each=5` 20 次全綠，判為偶發。

Session 紀錄：2026-11-20 e2e 共用 helper 收進 `e2e/page.ts`、釋放 wasm 物件收成 `freeAfter`、`undoWindow` 改名 `restartTimer` 並由呼叫端給時限
決定：釋放 wasm 物件一律用 `freeAfter(item, use)`（`debug-panel.ts` 原有的 `using` 搬出來改名），不用 `using` 宣告：wasm-bindgen 只在瀏覽器有 `Symbol.dispose` 時才掛上它，較舊的手機瀏覽器會丟錯（Agent 定）
決定：換掉手上留著的綁定物件也用 `freeAfter`，先建新的、`use` 裡換上，不留直接 `.free()`（Agent 定）
決定：計時器叫 `restartTimer(fire, ms)`、檔名 `restart-timer.ts`，沒有預設時限；`UNDO_WINDOW_MS` 搬到 `main.ts` 不再 export（Agent 定）
決定：e2e 共用模組叫 `e2e/page.ts`，開頁的 `ready` 與不換頁的 `waitReady` 分開；`slice(1)` 去序號的換成共用 `shown`（序號一定是一位數字，兩者相同）（Agent 定）
變更：無
