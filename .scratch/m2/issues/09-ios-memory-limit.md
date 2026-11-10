# 09: 加壓找鍵盤擴充的記憶體上限，決定資料包格式

Status: done
Blocked by: 08
讀: brief §9、§12.3；plan §1.4、§4、§7.8、§8（mmap、`jsonschema` 兩列）；spec〈共同規則〉；`docs/research/ios-feasibility.md`

## What to build

在票 08 的擴充裡逐步加壓，找出 iPhone 18 Pro Max 上系統終止鍵盤擴充的記憶體上限，然後決定資料包格式要不要改、M3 的模型能用多大。

- 加壓：擴充裡加一個測試鍵，每按一次多佔一塊確實寫過的記憶體，記下每步的常駐記憶體，直到被終止；終止的證據從裝置 log 或 crash report 取（取法查證後寫進 README）。重複幾次看是否穩定。
- 上限只在這台量：寫的時候註明旗艦機的數字可能比低階機高（M1 review 決定）。
- 決定（有疑問就停下問使用者）：
  - 核心加資料包占上限多少；放不放得下、留給 M3 模型多少空間。M3 模型大小的上限寫進 plan §7.8。
  - 資料包要不要改 mmap 友善格式：放得下就記「不改」與餘裕；放不下或餘裕太小就寫回 plan §1.4、§4、§8，實作另開票或留給 M3。
  - 鍵盤擴充要不要略過 Schema 驗證（依票 04）：寫回 plan §1.3、§1.4、§8。
- 結果寫進 `docs/research/ios-feasibility.md` 的〈記憶體上限〉一節，結論摘進 brief §12.3。

## 驗收

- [x] 上限有數字與每次的實測紀錄，附機型與 iOS 版本。
- [x] 資料包格式、Schema 驗證兩個決定寫回 plan；M3 模型上限寫進 plan §7.8。
- [x] brief §12.3 更新（社群說法改成實測，附連結）。
- [x] `cargo xtask check` 綠燈。

## Comments

- 2026-11-10（票 11 留言）：`ios-keyboard` 預設不建票 11 的神經模型探針，照 README〈建置與安裝〉建就好，不需要 candle。開 `QIM_NEURAL=YES` 的擴充多約 10 MB 程式碼（Debug dylib 12.9 MB 對 3.0 MB），量終止點的基線時別用它。小型神經模型的峰值增量（iPhone）：1.4M 約 +2–5 MB、23M q8 約 +47 MB、f32 約 +95 MB，給 plan §7.8 的模型上限參考（`docs/research/ios-feasibility.md`〈小型神經模型〉）。
- 2026-11-10（票 09）：上限 177 MB（kernel `ActiveHard 177 MB`，`per-process-limit`），7 輪終止前最後一筆 176.0–176.8 MB；載入核心後上限不變；約 142 MB 收到記憶體警告。數字與 log 摘錄見 `docs/research/ios-feasibility.md`〈記憶體上限〉，截圖 `.scratch/m2/evidence/09-*`。
- 字根提示改成實測（協調者轉達的 heads up）：`qim-ffi` 加只能解開的 `RadicalHintPack`（加測試），擴充在 `Session` 之後載入它，iPhone 上 +14.3–14.5 MB（Mac +16.3 MB）。佈局沒有 `qim-ffi` 介面，預算裡標「Mac 估算」（不驗證 +0.2 MB）。
- 決定：M3 模型在 iOS 鍵盤擴充的峰值增量 ≤ 60 MB（留在警告線下，23M 只能 q8）；資料包格式不改；鍵盤擴充略過 Schema 驗證，實作留給 M7。plan §1.3、§1.4、§2.1、§4、§7.8、§8 已改。票 04 的兩項「不改格式也能壓低峰值」不另開票（iOS 上省不到什麼）。
- 終止證據：擴充被終止時不產生 `JetsamEvent` 或 `.ips`（只有 CA 事件），主要證據是系統 log；手機 log 量大，iphone-use 只留約 5000 行，要在按鍵後馬上讀。第 1–3 輪鍵盤每次重畫都查 Foundation Models 可用性、把 log 洗掉，第 4 輪起加壓中不畫那幾行。
- 環境：Mac 的 Xcode 升到 27.2 後首次啟動元件沒裝，`devicectl` 與 iphone-use 卡住，使用者輸入管理員密碼後恢復（README 記了）。
- code-review（`a096d79` 起，Standards 與 Spec 兩軸）：沒有硬性違規、規格都做到。已修：brief 的 kernel log 只說取到的那一輪；〈記憶體上限〉改寫釋放的頁那句（載核心後約 1 MB 被重用）並註明開始 footprint 已含第一塊；plan §1.3 的 Schema 改成「M7 要做」並列進 M7；`docs/development.md` 補 `RadicalHintPack`；README 的 commit 改指票 09 的 commit。沒處理（判斷題）：`KeyboardViewController.render()` 選探針的條件又多一層、各探針重抄 `onUpdate`／`active`／`displayLines`，屬實驗程式，留給 M7 寫產品殼時不沿用；`RadicalHintPack` 只為量測而加，已記偏差。
