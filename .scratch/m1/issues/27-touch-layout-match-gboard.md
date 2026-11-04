# 27: 觸控佈局對齊 Gboard 速成

Status: in-progress
Blocked by: None (can start immediately)
讀: brief §4.4、§1.3（Gboard 那列）；plan §2.6、§3.1、§5.3；`docs/research/blackbox-baseline.md`（Gboard 截圖 `.scratch/m1/evidence/09-*.png`）；票 18、19、20 的 Comments；spec〈共同規則〉

## What to build

使用者 2026-11-05 試用後指出：預設觸控佈局看起來不像 Gboard。對照 `.scratch/m1/evidence/09-android-t13.png`（Gboard 速成）與 `20-long-press-a.png`（QIM），至少要改下列各項；其餘差異由實際比對找出，一併改或在 Comments 說明不改的理由。

- 第三排左邊加 Shift 鍵（Gboard 有 ⇧，QIM 是空位）；它在觸控上的行為照 Gboard 速成實測（大寫、切英文或其他），寫進 brief §4.4。
- `z`、`x` 鍵標籤照 Gboard（「重」「難」等），其他鍵的主標籤、角標字母與位置逐一對照。
- 配色、圓角、功能鍵（⇧、⌫、?123、↵、句號鍵）底色照 Gboard 的淺色與深色。
- 底排：空白鍵顯示「速成」；逗號鍵與句號鍵的長按、Gboard 逗號鍵上的表情入口（QIM 沒有表情頁就說明怎麼處理）。
- 符號頁（?123 與更多符號頁）的鍵位與內容照 Gboard。
- 長按不再出數字：長按字母鍵只出英文字母（推翻 brief §4.4「第一排另含數字」，使用者決定）。改成設定裡的「數字列」開關，照 Gboard 的 Number row，打開時字母頁上方多一排數字；預設關（與 Gboard 預設相同）。
- 比對方式：用 `orca emulator-android` 開一個本票自己建的 AVD（映像內建 Gboard、不登入帳號，照票 09 的做法），截 Gboard 速成的字母頁、符號頁、數字列開與關、深色，和 QIM 網頁同尺寸截圖並排存 `.scratch/m1/evidence/27-*.png`。不要動使用者的 `Medium_Phone_API_37.0`；做完刪掉自己建的 AVD。
- 同輪改 brief §4.4、§7（數字列開關）、plan §2.6；progress〈偏差與決定〉記一行（使用者決定）。

## 驗收

- [ ] 並排截圖存在 `.scratch/m1/evidence/27-*.png`：字母頁、符號頁、數字列開與關、深色各一組。
- [ ] 上面列的每一項已改，或在 Comments 寫明不改的理由；實際比對另外找到的差異也列出。
- [ ] Playwright：長按 `q` 只有字母、沒有數字；打開數字列後點「1」讀回「1」；點 Shift 鍵的行為讀回符合 brief §4.4。
- [ ] 自己建的 AVD 已刪除；`cargo xtask check` 綠燈。

## Comments

- 2026-11-05 進度（session 因額度中斷）：已做並 commit——長按只出字母、數字列開關（另一份佈局檔）、`z` 標「重」、Shift（格式加 `"shift"`，一次性大寫、快速兩下鎖定，規則在 `qim-layout`）、符號頁 ?123 / =\< / 12/34 照 Gboard（全半形照實測碼位）、配色與膠囊鍵、空白鍵「速成」、逗號長按進常用頁（代替表情入口）、句號長按 16 個、圓形預覽。並排截圖 `.scratch/m1/evidence/27-*.png`；自建 AVD 已刪。亂按 4 種子 × 150 步無錯誤；e2e 全過（104）。
- 刻意不同（Comments 待整理）：組字中 ⇧ 不變「'詞」（QIM 沒有分隔）；Shift 時不顯示數字角標；逗號的 ☺ 角標在右上不在正上；數字鍵盤左欄拆成四鍵；?123 不記上次子頁；空白長按不換鍵盤；數字列只有 1 有分數、其他只有上標；橫向沒對 Gboard 實測。
- 還沒做：`/code-review ab27a1c`、`cargo xtask check` 全套與 `bench --check`（沒動核心，預期不變）、progress〈偏差與決定〉一行、勾驗收、Status 改 done。
