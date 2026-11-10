# 09: 加壓找鍵盤擴充的記憶體上限，決定資料包格式

Status: ready-for-agent
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

- [ ] 上限有數字與每次的實測紀錄，附機型與 iOS 版本。
- [ ] 資料包格式、Schema 驗證兩個決定寫回 plan；M3 模型上限寫進 plan §7.8。
- [ ] brief §12.3 更新（社群說法改成實測，附連結）。
- [ ] `cargo xtask check` 綠燈。

## Comments

- 2026-11-10（票 11 留言）：`ios-keyboard` 預設不建票 11 的神經模型探針，照 README〈建置與安裝〉建就好，不需要 candle。開 `QIM_NEURAL=YES` 的擴充多約 10 MB 程式碼（Debug dylib 12.9 MB 對 3.0 MB），量終止點的基線時別用它。小型神經模型的峰值增量（iPhone）：1.4M 約 +2–5 MB、23M q8 約 +47 MB、f32 約 +95 MB，給 plan §7.8 的模型上限參考（`docs/research/ios-feasibility.md`〈小型神經模型〉）。
