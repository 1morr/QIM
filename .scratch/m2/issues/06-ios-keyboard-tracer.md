# 06: 空的鍵盤擴充裝上 iPhone（iOS tracer）

Status: done
Blocked by: None (can start immediately)
讀: brief §10、§11、§12.3；plan §3.2、§3.3、§7.3、§8（免費佈建那列）；`docs/research/platform-frameworks.md` §3；spec〈共同規則〉

## What to build

先不接核心，證明這台 Mac 能做出鍵盤擴充、用免費佈建裝到使用者的 iPhone 18 Pro Max、在真實宿主程式裡打字。之後的 iOS 實驗（票 08、10、11）都長在這個專案上。

- 開工先查證（Apple 官方文件，查到的補進 brief §12.3 附來源）：免費佈建（Personal Team）能否在實機跑鍵盤擴充、描述檔多久過期、App Group 能否用。
- `scripts/experiments/ios-keyboard/`：一個主 App 加一個鍵盤擴充的 Xcode 專案，擴充畫一個鍵，按下送出固定字「日」；擴充每次啟動時把常駐記憶體寫到 log（量法查證後寫進 README，給票 08 沿用）。
- 裝到 iPhone 18 Pro Max，在設定加入鍵盤，用 iphone-use 在備忘錄切到這個鍵盤、按鍵、讀回畫面。
- App Group：主 App 寫一個檔，擴充讀出來（沒有完整取用權限時共享容器唯讀，brief §12.3），讀到的內容顯示或寫 log。
- 要使用者動手的（Xcode 登入 Apple ID、iPhone 上信任開發者、加入鍵盤）到了就停下問。免費佈建不能跑鍵盤擴充或不能用 App Group 時，停下請使用者決定：提前申請付費帳號，或改在模擬器做並註明限制（plan §8）。
- 結果寫進 `docs/research/ios-feasibility.md` 的〈免費佈建與 App Group〉一節。

## 驗收

- [x] 備忘錄裡讀回擴充送出的「日」；截圖 `.scratch/m2/evidence/06-*.png`。
- [x] App Group 的讀寫結果有實測記錄（能、不能、或只在某條件下能）。
- [x] 空擴充的常駐記憶體有一個數字（之後扣掉這個基線），附機型與 iOS 版本。
- [x] README 有日期、Xcode 版本、iOS 版本、步驟；brief §12.3 補了查證的事實與來源。
- [x] `cargo xtask check` 綠燈。

## Comments

- 2026-11-10 實作：Xcode 已登入免費帳號（Team 先前已信任），不用使用者登入或信任開發者。加入鍵盤與第一次切到 QIM 實驗鍵盤由使用者手動做；之後按鍵與讀回由 iphone-use 完成。iphone-use 判定地球鍵被遮住，點座標才有反應，長按座標打到格式選單。
- 2026-11-10 實作：`os_proc_available_memory()` 在擴充裡有值，順手記下 footprint + available = 177.0 MB；這是依 Apple 定義推得的上限，實際終止點仍由票 09 加壓確認。
- 2026-11-10 實作：每個擴充各佔一個免費帳號的 App ID（本實驗 2 個），用到 bundle 前綴 `com.personal.qim`；他人重量時換自己的前綴（`Local.xcconfig.example`）。
- code-review 未處理（實驗碼，留給票 08 視需要處理）：主 App 與擴充各寫一份「組 lines 再逐行寫 log」（Duplicated Code）；`Probe` 同時管記憶體與 App Group（Divergent Change）；讀寫結果用字串前綴表示（Primitive Obsession），要程式判斷成敗時改 enum；量測沒記 PID，分不出冷啟動與重新載入，票 08 量基線時建議加上；主 App 讀 `from-keyboard.txt` 與重寫按鈕是票外的小附加。
