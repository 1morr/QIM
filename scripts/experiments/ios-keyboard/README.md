# iOS 鍵盤擴充實驗專案

回答的問題：

- M2 票 06：沒有付費帳號時，這台 Mac 能不能用免費佈建做出鍵盤擴充、裝到 iPhone、在真實宿主程式打字；App Group 能不能用；空擴充的常駐記憶體是多少。結論在 `docs/research/ios-feasibility.md`〈免費佈建與 App Group〉。
- M2 票 10：鍵盤擴充裡能不能用 Foundation Models、延遲與記憶體；`documentContextBeforeInput` 實際讀得到多少字。結論在同一份文件的〈Foundation Models〉〈上下文長度〉。
- M2 票 08：鍵盤擴充接上核心（`qim-ffi`），在備忘錄打 `anr` 讀回「門口」；資料包的常駐記憶體與每鍵延遲。結論在同一份文件的〈鍵盤擴充接上核心〉。
- M2 票 11：candle 能不能在鍵盤擴充裡跑 1.4M、23M 參數的模型，載入、推論延遲與記憶體。量測程式在 `../neural-cost/`，結論在同一份文件的〈小型神經模型〉。

之後的 iOS 實驗（票 09）也長在這個專案上。

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
| `Keyboard/CoreProbe.swift` | 票 08：載入資料包、建一個 `Engine` 與 `Session`、逐步量 footprint、打字、量每鍵延遲 |
| `core.sh` | 票 08：`cargo xtask ffi --ios`、寫出延遲的輸入、檢查資料包，再 `xcodegen generate` |
| `neural.yml`、`Neural/` | 票 11：神經模型探針（`NeuralProbe.swift` 在背景執行緒呼叫 `../neural-cost` 的 Rust 靜態庫）；預設不併進專案，見下方〈小型神經模型（票 11）〉 |

產生的 `.xcodeproj`、`Info.plist`、`.entitlements` 與 `build/` 都由 `project.yml` 產生，不進 git。

## 建置與安裝

需要 Xcode（已登入 Apple ID；免費帳號在 Xcode 顯示為 Personal Team）、`brew install xcodegen`，以及接上核心要的 Rust（`rustup target add aarch64-apple-ios`）與資料包（repo 根目錄先跑過 `cargo xtask data`）。擴充的 target 引用 repo 根目錄 `build/` 下的綁定、xcframework 與資料包，沒有它們 Xcode 建不起來，所以每次都先跑 `core.sh`。在本目錄：

```sh
cp Local.xcconfig.example Local.xcconfig   # 填 Team ID 與自己的 Bundle ID 前綴
./core.sh                                  # 備好核心要的檔，最後跑 xcodegen generate；要票 11 的神經模型探針時見〈小型神經模型（票 11）〉
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

## 接上核心（票 08）

擴充的 bundle 放五份資料包：速成 `quick`、詞庫 `quick-lexicon`、bigram `zh-hant-bigram`、簡繁 `opencc-hk2s`、英文 `scowl-en-us`，與網頁打字時載入的相同（票 04 建議的組合）。不放的：字根提示 `cangjie-hint`（只給殼顯示，`Engine` 不用；Mac 上解開後 16 MB）與佈局（`qim-ffi` 還沒有佈局的介面，實驗的鍵也不讀佈局）。放 bundle 不放 App Group：擴充本來就讀得到自己的 bundle，不必先開主 App 複製；免費佈建的 App Group 擴充只能讀（票 06），兩處都行。

### 鍵

| 鍵 | 做什麼 |
| --- | --- |
| 載入 | 在背景執行緒依序「讀檔 → 解開 → 丟掉檔案位元組」五份資料包，再建 `Engine` 與 `Session`，每步記 footprint 的增量與耗時；整個程序只建一次 |
| a、n、r、空白 | 按核心的鍵；候選列顯示輸入碼與前 6 個候選，`commit` 上屏，沒有組字時空白交還宿主（插入空白）；每鍵記核心呼叫與含畫面更新的時間、之後的 footprint |
| 延遲 | 用同一個 `Session` 跑 ffi-swift 那批輸入（`Latency.swift` 的量法、`inputs.sh` 寫出的 `latency-inputs.txt`），印出與 Mac 同欄的表；跑的時候其他核心鍵不動作 |

按過「載入」後，鍵盤上方只顯示核心的行（票 06 的記憶體兩行與 `taps=` 照舊在最上面），其他探測的行照樣寫 log。

### 量法

1. 照〈建置與安裝〉建好、裝上；新程序第一次出現時 `viewDidAppear` 那行就是這個程序的基線（約 10 MB，見〈常駐記憶體〉）。
2. 在備忘錄新增一則備忘錄、切到「QIM 實驗鍵盤」，按「載入」，等 `core-loaded` 出現。讀 `core-before` 到 `session` 各行的增量（`+X MB`）與 `core-loaded` 的總 footprint。
3. 按 `a`、`n`、`r`，候選列首選是「門口」；按「空白」，備忘錄裡出現「門口」。截圖。每鍵的 `key … core=… screen=…` 與 `typing footprint=…` 寫在 log。
4. 按「延遲」，等表出現（iPhone 18 Pro Max 約 10 秒）。表的欄位同 `scripts/experiments/ffi-swift/README.md`。
5. 系統 log：Console.app 選手機、篩 `qim.exp`，核心的行以 `core ` 開頭。

量的是 Debug 組態的 Swift 加 release 的 Rust 靜態函式庫（`cargo xtask ffi --ios`）；核心的計算都在 Rust 那邊，Swift 的組態只影響鍵盤畫面。

`core.sh` 重新產生 `.xcodeproj`；`build/ffi/swift/qim.swift` 與 xcframework 跟著 `qim-ffi` 的介面變，介面改了要重跑 `core.sh` 再建置。


### 上次結果（2026-11-10）

環境：iPhone 18 Pro Max（iPhone19,7），iOS 27.2；macOS 27.0.1；Xcode 27.1（27A9275），iOS SDK 27.1，Swift 6.4；Rust 1.99.1，uniffi 0.32.2；xcodegen 2.46.0；Debug 組態（Rust 是 release）；沒有完整取用；量測時 repo 在 `232d528` 加上票 08 的改動（資料包由 `232d528` 建）。兩個新程序各量一次。

| 項目 | 第 1 次 | 第 2 次 |
| --- | ---: | ---: |
| 載入前 footprint | 12.95 MB | 13.03 MB |
| 五份資料包加 `Engine`、`Session` | +51.49 MB | +51.39 MB |
| 其中詞庫 | +41.36 MB | +41.36 MB |
| 載入後 footprint | 64.44 MB | 64.42 MB |
| 載入耗時 | 120 ms | 138 ms |
| 每鍵 p50 / p95 | 45.62 / 64.00 µs | 45.08 / 63.96 µs |
| 選開頭一段 p50 / p95 | 246.38 / 789.75 µs | 245.46 / 786.54 µs |

備忘錄打 `anr` 首選「門口」、空白上屏「門口」，兩次都是。各步的數字、與 Mac 的對照與截圖清單見 `docs/research/ios-feasibility.md`〈鍵盤擴充接上核心〉。

用 iphone-use 操作時，擴充的鍵照樣要點座標：`tap_locator` 回報成功，`taps=` 卻沒加（同〈注意事項〉的地球鍵）。手機只走 Wi-Fi 時 `phone_logs` 與 `phone_crashes` 用不了（`not attached to usbmuxd`），接上 USB 才行。

## 小型神經模型（票 11）

### 要建進專案時

神經模型探針要 candle 的靜態庫與 118 MB 的模型檔，預設不併進專案：`project.yml` 用 xcodegen 的 `include` 在環境變數 `QIM_NEURAL` 為 `YES` 時才併進 `neural.yml`（加 `Neural/` 的原始碼、`../neural-cost/models` 資料夾、bridging header、連結 `-lneural_cost`、編譯條件 `QIM_NEURAL`）。沒開時鍵盤沒有 L32 那一列，也不需要 candle；票 09 這類只量核心的實驗照〈建置與安裝〉就好。要量時：

```sh
../neural-cost/build-ios.sh       # 靜態庫（只編了實機的 aarch64-apple-ios）與 ../neural-cost/models/
QIM_NEURAL=YES ./core.sh          # core.sh 最後的 xcodegen generate 會讀這個環境變數
xcodebuild ...                    # 同〈建置與安裝〉
```

開著建時模擬器建不起來。開與不開的擴充大小不同（Debug 的 `QIMKeyboard.debug.dylib` 約 12.9 MB 對 3.0 MB），量基線時兩種別混用。

### 鍵

| 鍵 | 做什麼 |
| --- | --- |
| L32、L8 | 量 1.4M 參數的 CIFG-LSTM，f32 與 q8 |
| G32、G8 | 量 23M 參數的字級 decoder，f32 與 q8 |
| 結束 | `exit(0)`：擴充自己結束程序，下次叫出鍵盤就是新程序 |

每個模型鍵在背景執行緒呼叫 `nc_run(<bundle 裡的 models/檔名>, 30, 2)`，跑完把結果顯示在鍵盤上方（取代上下文與 Foundation Models 那幾行）並寫 log。前後各加一行 Swift 這邊讀的 footprint（`nc-before`、`nc-after`，後者是模型丟掉之後）與整次的耗時 `wall=`。每行的意思見 `../neural-cost/README.md`〈指標〉。

### 量法

1. 開著神經模型建好、裝上（見上方〈要建進專案時〉）。
2. 備忘錄新增一則備忘錄，切到「QIM 實驗鍵盤」，確認第一行的 `pid=` 是新的、`loads=1`。
3. 按一個模型鍵，等結果出現（23M f32 約 4 秒），截圖。
4. 按「結束」，點一下備忘錄叫出鍵盤；系統會換成別的鍵盤，長按地球鍵切回「QIM 實驗鍵盤」，`pid=` 換了才量下一個。每個模型都要在新程序量（理由見 `../neural-cost/README.md`〈量法〉）。

用 iphone-use 操作時，這一列的鍵和票 06 的地球鍵一樣，`tap_label` 會點穿到宿主（`taps=` 沒加），要點座標。地球鍵的長按點座標也行。

### 上次結果（2026-11-10）

環境同上：iPhone 18 Pro Max（iPhone19,7），iOS 27.2（24B119）；Xcode 27.1，Debug 組態；candle 0.11.1，2 條執行緒；沒有完整取用。量測時 repo 在 `78ae0c2`，實驗碼即票 11 的 commit。

candle 能在擴充裡跑。iPhone 上 23M q8 重排一次（6 條路徑各 8 字）上下文 200 字 p95 54.5 ms、峰值 +46.9 MB；23M f32 峰值 +94.9 MB；1.4M 兩種都在 9 ms、5 MB 內。完整表見 `../neural-cost/README.md`〈上次結果〉，截圖 `.scratch/m2/evidence/11-iphone-{L32,L8,G32,G8}.png`。

合併票 08 之後（神經模型探針改成可選），開著 `QIM_NEURAL=YES` 建的擴充在備忘錄按「載入」、`a`、`n`、`r`、「空白」，仍上屏「門口」，同一個鍵盤上也有 L32 那一列（`.scratch/m2/evidence/11-merged-anr.png`）；神經模型的數字沒重量。

另一個 session 用同一個 Bundle ID 裝它的版本時會蓋掉這個擴充（票 08 與本票同時用過）；量之前看鍵盤上有沒有 L32 那一列。
