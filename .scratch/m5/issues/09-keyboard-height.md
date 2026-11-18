# 09: 調整鍵盤高度

Status: ready-for-agent
Blocked by: 08
讀: brief §4.4；`ARCHITECTURE.md`〈佈局格式〉；`docs/design/06-keyboard-settings.html`；`web/src/touch.ts`

## What to build

設定頁加「鍵盤高度」，段數照盤點清單與定稿。照 Gboard 只改高度、寬度維持全寬（使用者 2026-11-18 同意；盤點時確認 Gboard 的實際行為，不同就停下問）：

- 現在高度上限寫死在 CSS（`#keyboard` 的 `45vh`），手機直向時寬度是 100%、高度由寬度除以畫布比例得出。改成高度由設定的段決定，寬度不變，按鍵在縱向拉伸或壓扁。
- 點擊判定：`touch.ts` 的 `mmPerUnit` 現在只用寬度算；只改高度後要拆成 x、y 兩個比例，點擊位置換算到畫布座標、觸控按鍵機率的毫米距離都照兩個比例算。`qim-layout` 收的機率輸入要不要改，先看程式再定。
- 直向、橫向各自記，存檔走票 08 的設定存檔。

## 驗收

- [ ] Playwright 換段後讀回鍵盤高度變了、寬度不變，點擊仍打中該打的鍵（含按鍵機率讀回，縱向點偏時機率照新高度算）。
- [ ] 重新整理後高度還在；直向、橫向各自記。
- [ ] 動到網頁的部分照 spec〈共同規則〉亂按一輪，找到的問題先寫成會紅的 Playwright 測試再修。
- [ ] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments
