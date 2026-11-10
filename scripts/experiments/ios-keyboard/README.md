# iOS 鍵盤擴充實驗專案

回答的問題：

- M2 票 06：沒有付費帳號時，這台 Mac 能不能用免費佈建做出鍵盤擴充、裝到 iPhone、在真實宿主程式打字；App Group 能不能用；空擴充的常駐記憶體是多少。結論在 `docs/research/ios-feasibility.md`〈免費佈建與 App Group〉。
- M2 票 10：鍵盤擴充裡能不能用 Foundation Models、延遲與記憶體；`documentContextBeforeInput` 實際讀得到多少字。結論在同一份文件的〈Foundation Models〉〈上下文長度〉。

之後的 iOS 實驗（票 08、11）也長在這個專案上。

## 內容

| 路徑 | 內容 |
| --- | --- |
| `project.yml` | xcodegen 的專案描述：主 App `QIMKeyboardExp` 內嵌鍵盤擴充 `QIMKeyboard`，兩邊都有 App Group 權限 |
| `Local.xcconfig.example` | Team ID、Bundle ID 前綴、App Group 名；複製成 `Local.xcconfig`（不進 git） |
| `Shared/Probe.swift` | 兩個 target 共用：常駐記憶體、App Group 讀寫 |
| `App/` | 主 App：啟動時把時間戳寫進 App Group 的 `from-app.txt`，畫面列出讀寫結果 |
| `Keyboard/KeyboardViewController.swift` | 鍵盤擴充：「日」鍵送出「日」；每次載入量記憶體、讀 `from-app.txt`、試寫 `from-keyboard.txt`；票 10 的鍵（見下方〈Foundation Models 與上下文長度〉）；結果顯示在鍵盤上並寫 log |
| `Keyboard/ContextProbe.swift` | 票 10：測試長文與 `documentContextBeforeInput` / `AfterInput` 的摘要 |
| `Keyboard/ModelProbe.swift` | 票 10：Foundation Models 的可用性與每次請求的延遲 |

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

## Foundation Models 與上下文長度（票 10）

### 鍵

| 鍵 | 做什麼 |
| --- | --- |
| 長文 | 插入 9 段的測試長文：每段 40 個 10 字單元 `[段.序]天地玄` 加「，」（每第 5 個是「。」），第 4、5 段之間空一行 |
| 長段 | 插入 3000 字、沒有標點與換行的一段：300 個 `[序號]天地玄黃` |
| 讀 | 重讀游標前後文（`viewDidAppear`、`textDidChange`、`selectionDidChange` 也會讀） |
| FM | 依序送 6 個請求，每個新開 `LanguageModelSession`；提示照 plan §7.8（上下文 + `anr` + 10 個候選，只回序號，正解 3） |
| 間隔 | 同一個程序裡隔 0、2、5、10、20、40 秒各送 1 個請求，再隔 10、20、40 秒各送 1 個先 `prewarm()`、等 1 秒才送的請求；整輪約 2.5 分鐘 |
| ←1000 … →100 | `adjustTextPosition(byCharacterOffset:)` 移游標，0.3 秒後重讀 |
| ⇤ | 往前移 `before` 的字數，連按就一句一句往前走 |

上下文那行的格式：`before chars=<Character 數> utf16=<UTF-16 長度> nl=<換行數> head='<前 12 字>' tail='<後 12 字>'`，換行顯示成 `⏎`。看 `head` 的單元編號就知道從哪起讀。螢幕放不下的行（`full-access`、`group-read`、`group-write`、`fm-languages`）只寫 log。`taps=` 是這個程序裡各鍵被按的次數、`loads=` 是 view controller 載入次數，用來發現漏掉的點擊與畫面重建。

### 量法

1. 照〈建置與安裝〉裝好。Foundation Models 要 Apple Intelligence；iOS 27 起預設開啟、不能關，舊版要到「設定 → Apple Intelligence 與 Siri」打開。鍵盤上的 `fm-availability` 不是 `available` 時先處理它。
2. 備忘錄：新增一則備忘錄，切到「QIM 實驗鍵盤」，按 FM（連按三次、每次隔半分鐘左右）與「間隔」，讀 `fm` 那幾行。
3. 同一則備忘錄按「長文」，用 ←、→、⇤ 移到文末、句中、句首、段首各讀一次；再按「長段」，在長段中間與文末各讀一次。剛插入完讀到的是暫時值（見結果），先移開再移回。
4. Safari：在本機起一個有 `<textarea>` 的網頁，用 Safari（不是預設瀏覽器，免得問本地網絡權限）開 `http://<Mac 的區網 IP>:8765/t.html`，點進 textarea 重做第 3 步。

```sh
mkdir -p /tmp/qim-ctx && cat > /tmp/qim-ctx/t.html <<'HTML'
<!doctype html><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<textarea style="width:95%;height:40vh;font-size:16px"></textarea>
<input type="text" style="width:95%;font-size:16px">
HTML
python3 -m http.server 8765 --bind "$(ipconfig getifaddr en0)" --directory /tmp/qim-ctx
```

### 用 iphone-use 操作時

- 每次點擊或截圖，鍵盤的 view controller 都會重建（同一個程序，`loads=` 加一）；量測結果因此存在 `ModelProbe` 的靜態狀態，新畫面出現時讀回。
- 剛重建時的點擊偶爾漏掉（`taps=` 沒加），或點穿到宿主讓游標跳走；連點之間留 1.5 秒以上，並以讀數裡的 `head` / `tail` 判斷游標實際位置。
- 單按地球鍵只在最近兩個鍵盤之間切換；要切到 QIM 實驗鍵盤，長按地球鍵叫出清單再點。
- 重新安裝後系統會換回別的鍵盤，要再切一次。

### 上次結果（2026-11-10）

環境同票 06 的〈上次結果〉：iPhone 18 Pro Max（iPhone19,7），iOS 27.2（24B119）；Xcode 27.1（27A9275），iOS SDK 27.1；Debug 組態；沒有完整取用。量測時 repo 在 `fc133ca`，實驗碼即票 10 的 commit（commit 前做了兩次小整理：`ModelProbe.run` 改用共用的 `timedRequest`、顯示行改由 `ModelProbe.displayLines` 給；整理後在備忘錄重按一次 FM：889 ms，之後 186–190 ms，6 次都回「3」，與下表一致，沒併進下表）。

Foundation Models：`available`、AFM 3 Core Advanced、`contextSize` 8192；連續請求 185–194 ms；停頓後第一個請求 286 ms–15.5 s（12 次裡 4 次超過 6 s）；27 次都回「3」；擴充 footprint 第一輪 +3.8 MB（含畫面重建）。

上下文：備忘錄句中給所在句加前一整句、最多 300 字；Safari `<textarea>` 句中只給所在句、最多 100 字（段首例外）；換行不截斷。

各位置的數字與截圖清單見 `docs/research/ios-feasibility.md`〈Foundation Models〉〈上下文長度〉。
