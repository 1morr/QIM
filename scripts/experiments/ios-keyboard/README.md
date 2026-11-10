# iOS 鍵盤擴充實驗專案

回答的問題（M2 票 06）：沒有付費帳號時，這台 Mac 能不能用免費佈建做出鍵盤擴充、裝到 iPhone、在真實宿主程式打字；App Group 能不能用；空擴充的常駐記憶體是多少。結果與結論在 `docs/research/ios-feasibility.md`〈免費佈建與 App Group〉。之後的 iOS 實驗（票 08、10、11）都長在這個專案上。

## 內容

| 路徑 | 內容 |
| --- | --- |
| `project.yml` | xcodegen 的專案描述：主 App `QIMKeyboardExp` 內嵌鍵盤擴充 `QIMKeyboard`，兩邊都有 App Group 權限 |
| `Local.xcconfig.example` | Team ID、Bundle ID 前綴、App Group 名；複製成 `Local.xcconfig`（不進 git） |
| `Shared/Probe.swift` | 兩個 target 共用：常駐記憶體、App Group 讀寫 |
| `App/` | 主 App：啟動時把時間戳寫進 App Group 的 `from-app.txt`，畫面列出讀寫結果 |
| `Keyboard/` | 鍵盤擴充：一個鍵送出「日」；每次載入量記憶體、讀 `from-app.txt`、試寫 `from-keyboard.txt`，結果顯示在鍵盤上並寫 log |

產生的 `.xcodeproj`、`Info.plist`、`.entitlements` 與 `build/` 都由 `project.yml` 產生，不進 git。

## 建置與安裝

需要 Xcode（已登入 Apple ID；免費帳號在 Xcode 顯示為 Personal Team）與 `brew install xcodegen`。在本目錄：

```sh
cp Local.xcconfig.example Local.xcconfig   # 填 Team ID 與自己的 Bundle ID 前綴
xcodegen generate
xcodebuild -project QIMKeyboardExp.xcodeproj -scheme QIMKeyboardExp -configuration Debug \
  -destination 'platform=iOS,id=<UDID>' -derivedDataPath build -allowProvisioningUpdates build
xcrun devicectl device install app --device <UDID> build/Build/Products/Debug-iphoneos/QIMKeyboardExp.app
xcrun devicectl device process launch --device <UDID> <前綴>.exp   # 主 App 寫 App Group 檔
```

UDID 用 `xcrun devicectl list devices` 查。`-allowProvisioningUpdates` 讓 xcodebuild 自動向 Apple 註冊 App ID、App Group 與描述檔。免費佈建的描述檔 7 天後過期，過期後重跑 `xcodebuild` 與 `install`。

第一次在手機上：

1. 這個 Apple ID 的開發者還沒被信任時，到「設定 → 一般 → VPN 與裝置管理」信任它（本機的 Team 先前已信任，票 06 沒遇到）。
2. 「設定 → 一般 → 鍵盤 → 鍵盤 → 新增鍵盤」加入「QIM 實驗鍵盤」。不要開「允許完整取用」（`RequestsOpenAccess` 是 `NO`，本來也沒有那個開關）。
3. 在備忘錄新增一則備忘錄，按地球鍵切到「QIM 實驗鍵盤」，按「日」。

## 注意事項

- **iphone-use 的 runner 也是免費佈建，同樣 7 天過期。** 驗收靠它操作手機，runner 過期時沒辦法在備忘錄打字、截圖。查目前的到期時間（runner 是 `…iphone-use.runner….xctrunner` 那份）：

  ```sh
  for f in ~/Library/Developer/Xcode/UserData/Provisioning\ Profiles/*.mobileprovision; do
    security cms -D -i "$f" | plutil -extract Name raw -o - - | tr '\n' ' '
    security cms -D -i "$f" | plutil -extract ExpirationDate raw -o - -
  done
  ```

  手機上實際裝的是 `~/.iphone-use/runner-build/Build/Products/Debug-iphoneos/iPhoneUse-Runner.app/embedded.mobileprovision` 那份，同樣用 `security cms -D -i <檔> | plutil -extract ExpirationDate raw -o - -` 查。
- iphone-use 0.27.7 重新連線（`phone_reconnect`）時，描述檔 48 小時內到期就自動續期並重裝 runner（`~/.iphone-use/runner-agent.log` 會出現 `Renewing the runner provisioning profile`）。2026-11-10 13:36 就這樣續過，到期日從 11/11 15:18 延到 11/17 13:36（CST）。
- 已經過期、或自動續期沒成功時，手機解鎖後在 Mac 上跑 `iphone-use setup`（另一個 session 握著手機時會拒絕；`--force` 會搶走別人的 runner，由使用者決定）。要強制重建加 `IPU_RUNNER_REBUILD=1`。iOS 若跳出要密碼才能允許 UI 自動化的提示，只能使用者在手機上輸入。之後 `iphone-use status` 回 0（ready）就能操作；卡住時先跑 `iphone-use doctor`。
- 鍵盤擴充自己也是 7 天過期，照〈建置與安裝〉重跑 `xcodebuild` 與 `install`。兩者到期日不同，跨天的票（08、10、11）開工前兩個都先查。

## 常駐記憶體

量法：擴充在 `viewDidLoad` 與 `viewDidAppear` 各量一次，兩個數字：

- `footprint`：`task_info(TASK_VM_INFO)` 的 `phys_footprint`。
- `available`：`os_proc_available_memory()`。Apple 文件說它等於「目前的記憶體上限減去 App 的 footprint」，所以 `footprint + available` 就是系統給這個程序的上限。

Apple 沒有明文說 `phys_footprint` 就是上限所看的 footprint（官方給的指標是 Xcode 的 memory gauge 與 jetsam 報告的 `rpages × pageSize`），但三次實測 `footprint + available` 都是 177.0 MB，與這個解讀一致。官方說法與連結見 `docs/research/ios-feasibility.md`。

結果顯示在鍵盤上方的文字（截圖可讀），同時以 `Logger(subsystem: "qim.exp", category: "probe")` 寫進系統 log，可用 Console.app 選手機、篩 `qim.exp` 看。

量新的程式時：同樣在 `viewDidAppear` 讀 `footprint`，扣掉下面的基線；組態要一樣（下面是 Debug）。

## 上次結果（2026-11-10）

環境：iPhone 18 Pro Max（iPhone19,7），iOS 27.2（24B119）；macOS 27.0.1；Xcode 27.1（27A9275），iOS SDK 27.1，Swift 6.4；xcodegen 2.46.0；不涉及 Rust（票 08 接上核心後補 Rust 版本）；Debug 組態；量測時 repo 在 `17727d9`，實驗程式碼即本目錄首次 commit 的版本。Team 為免費 Personal Team。

| 次 | `viewDidLoad` footprint | `viewDidAppear` footprint | `viewDidAppear` available | footprint + available |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 9.86 MB | 9.99 MB | 167.0 MB | 177.0 MB |
| 2 | 10.36 MB | 10.53 MB | 166.5 MB | 177.0 MB |
| 3 | 10.14 MB | 10.24 MB | 166.8 MB | 177.0 MB |

第 1 次是切到鍵盤時；第 2、3 次是按「日」後與切換鍵盤一輪後的重新載入。沒有查證每次 `viewDidLoad` 是不是新的程序。

App Group（沒有完整取用）：

| 誰 | 動作 | 結果 |
| --- | --- | --- |
| 主 App | 寫 `from-app.txt` | 成功（擴充讀到內容） |
| 擴充 | 讀 `from-app.txt` | 成功：`ok:app 2026-11-10T05:35:08Z` |
| 擴充 | 寫 `from-keyboard.txt` | 失敗：`NSCocoaErrorDomain#513`（`NSFileWriteNoPermissionError`）／`NSPOSIXErrorDomain#1`（`EPERM`） |

截圖：`.scratch/m2/evidence/06-keyboard-probe.png`（鍵盤上的量測）、`.scratch/m2/evidence/06-notes-ri.png`（備忘錄讀回「日」）。
