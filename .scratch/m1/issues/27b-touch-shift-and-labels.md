# 27b: 觸控 Shift 狀態與讀屏標籤細節

Status: ready-for-agent
Blocked by: 26b
讀: 票 27、28 的 Comments；brief 觸控鍵盤章節、§4.4；plan §2.6（佈局格式）；`.scratch/m1/evidence/27-*.png`；spec〈共同規則〉

## What to build

M1 封存後補票的審查（2026-11-05，獨立審查，實際用觸控操作過）找到以下小問題，使用者 2026-11-05 同意照建議修。審查的探測與截圖在 `/private/tmp/claude-501/-Users-roxy-orca-projects-QIM/4808186f-7ee6-44a4-9688-748b966c8ba8/scratchpad/review-b/`。排在 26b 之後是因為都改 `web/src/main.ts`。

1. Shift 開著時字母鍵讀屏念「Q，q」：`crates/qim-layout/src/model.rs` 約 329、347 行 `accessibility_label` 用 `self.hint`，不是 `display_hint(state)`，畫面上卻沒有角標。改用 `display_hint(state)`，補測試。
2. 開一次的 Shift 打了逗號（punct）或切頁後仍開著，只有打字母才解除（`model.rs` 約 244 行 `after_key` 只看 `Letter`）。先在 Android 模擬器的 Gboard 實測（`orca emulator-android`；若沒有 AVD 照 27 的做法建一個，做完刪），照 Gboard 改；實測不了就記在 Comments 並維持現狀。
3. 切數字列開關會重建 `Layout`，鎖定的大寫與目前頁被清掉（`web/src/main.ts` 約 572 行）：把 Shift 與頁的狀態搬過去，補 Playwright。
4. 數字列開時鍵變矮，字母角標貼著字根（「中l」「心p」，見 `27-number-row.png`）：角標位置照鍵高等比例調整。
5. Shift 時大寫字母比 Gboard 小很多（`27-letters.png`）：照 Gboard 截圖加大字級。
6. 逗號鍵讀屏念「，，長按切到常用」：改寫它的 `a11yLabel`。
7. 佈局驗證：`composing` 只寫 `a11yLabel`、沒寫 `label` 時被默默忽略，`validate.rs` 加一條警告（M5 開放自訂佈局前補上），補測試。
8. progress session 紀錄票欄寫「M2（補 M1 27b）」。

改 UI 的項目都要實跑 Playwright 截圖對照 Gboard 截圖。

## 驗收

- [ ] 測試：Shift 開著時字母鍵讀屏標籤不含角標；切數字列後鎖定大寫還在；驗證器對只有 `a11yLabel` 的 `composing` 發警告。
- [ ] 第 2 項有 Gboard 實測結論（改了或記了為何不改）；4、5 有前後截圖放 `.scratch/m1/evidence/27b-*.png`。
- [ ] `cargo xtask check` 綠燈；/code-review 跑完。
