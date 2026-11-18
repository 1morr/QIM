# 10: 觸控殘項：地球鍵、選單、刪除鍵退格撤銷、z 鍵標示

Status: ready-for-agent
Blocked by: 03, 04d
讀: brief §4.4；`ARCHITECTURE.md`〈佈局格式〉〈核心對殼〉；`.scratch/m4/issues/31-m4-close.md` 的 Comments

## What to build

M4 留下的觸控鍵盤問題（M4 票 18c、23、11）：

- 觸控鍵盤靠標籤 `🌐` 認要畫成圖示的鍵（`ICON_LABELS`），改看按鍵的動作（`keyboardMenu`）或 id。`touch.ts` 的 `ascii` 變數是核心狀態推下來的副本、只給選單打勾：刪掉它，打勾由 `main.ts` 傳（wasm `Layout` 換頁用的那份不算；開工對程式碼補正，使用者 2026-11-18 同意）。
- 地球鍵選單的「點外面收起」只在鍵盤範圍內：點頁面其他地方也收起。
- 觸控刪除鍵的退格撤銷學習只在核心驗過：補 Playwright（觸控上屏後緊接著點刪除鍵，再打同一串碼首選不變）。
- `z` 鍵標示照盤點清單的選擇（維持「重」或改「＊」）；要改時改 `qim-databuild` 的 `gboard.rs` 並重產佈局（`web/public/layouts/` 與 `build/layouts/` 兩份）。

## 驗收

- [ ] 換一個標籤不是 `🌐` 的地球鍵也畫成圖示（測試讀回）；`touch.ts` 沒有 `ascii` 變數，選單打勾照舊。
- [ ] 點鍵盤外面收起地球鍵選單的測試先紅後綠。
- [ ] 觸控刪除鍵退格撤銷有 Playwright。
- [ ] `z` 鍵標示照使用者勾的結果。
- [ ] 動到網頁的部分照 spec〈共同規則〉亂按一輪，找到的問題先寫成會紅的 Playwright 測試再修。
- [ ] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments
