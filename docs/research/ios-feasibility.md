# iOS 可行性實驗（M2）

M2 各實驗量到的數字，一個實驗一節（`.scratch/m2/spec.md`〈共同規則〉）。結論摘進 brief §12.3；收尾票再統一寫 brief §9、§11。實機只有使用者的 iPhone 18 Pro Max，旗艦機的記憶體上限可能比低階機高。

## 免費佈建與 App Group

M2 票 06，2026-11-10。實驗專案與重量步驟：`scripts/experiments/ios-keyboard/`。

裝置：iPhone 18 Pro Max（iPhone19,7），iOS 27.2（24B119）。Xcode 27.1（27A9275），iOS SDK 27.1。Apple ID 沒有付費會員，Xcode 的 Team 是 Personal Team。

### 結論

- **免費佈建能在實機跑鍵盤擴充。** `xcodebuild -allowProvisioningUpdates` 自動註冊兩個 App ID（主 App、擴充）與 App Group，簽出兩份 `iOS Team Provisioning Profile`，`devicectl` 裝上手機；在設定加入鍵盤後，備忘錄裡按擴充的鍵讀回「日」（`.scratch/m2/evidence/06-notes-ri.png`）。
- **描述檔 7 天過期。** 兩份描述檔都是 `CreationDate` 2026-11-10 13:34、`ExpirationDate` 2026-11-17 13:34（CST），與 Apple 的說法一致。過期後要重建、重裝。
- **App Group 能用。** 免費帳號的描述檔帶 `com.apple.security.application-groups`；沒有完整取用時，擴充讀得到主 App 寫的檔、寫不進共享容器（`EPERM`），與官方文件一致。開完整取用的情形沒測：擴充的 `RequestsOpenAccess` 是 `NO`，M2 的實驗也不需要寫共享容器。
- **空擴充的常駐記憶體約 10 MB**（`phys_footprint`，Debug 組態，三次 9.99–10.53 MB）。之後的實驗扣掉這個基線。只有第 1 次確定是冷啟動；第 2、3 次沒記 PID，可能是同一程序重新載入畫面。
- **`os_proc_available_memory()` 在鍵盤擴充裡有值**，三次 `footprint + available` 都是 177.0 MB。照 Apple 對這個函式的定義，這就是這台機器、這個 iOS 版本給鍵盤擴充的上限；票 09 加壓時以實際終止點確認。

### 原始輸出摘要

鍵盤上方顯示的量測（同一份也寫進系統 log，subsystem `qim.exp`）：

```text
viewDidLoad footprint=9.86MB available=167.1MB
viewDidAppear footprint=9.99MB available=167.0MB
full-access=false
group-read=ok:app 2026-11-10T05:35:08Z
group-write=write-failed:NSCocoaErrorDomain#513/NSPOSIXErrorDomain#1
```

| 次 | `viewDidLoad` footprint | `viewDidAppear` footprint | `viewDidAppear` available | footprint + available |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 9.86 MB | 9.99 MB | 167.0 MB | 177.0 MB |
| 2 | 10.36 MB | 10.53 MB | 166.5 MB | 177.0 MB |
| 3 | 10.14 MB | 10.24 MB | 166.8 MB | 177.0 MB |

描述檔（`security cms -D` 解出）：兩份的 `Entitlements` 都有 `com.apple.security.application-groups = [group.com.personal.qim]`、`get-task-allow = true`，沒有 `ProvisionsAllDevices`。

### 官方說法（2026-11-10 查）

- 免費帳號：「To install and test your apps on a personal device, you'll need to sign in to your Apple Account in Xcode. If your account is not associated with a developer program membership, Xcode will indicate it's a Personal Team.」限制：「You can register up to 10 App IDs, which expire after 7 days.」「You can register up to 3 devices, which expire after 7 days.」「You can install up to 3 apps per device. Provisioning profiles that enable apps to be installed on a device will expire 7 days from issuance.」[About your developer account](https://developer.apple.com/help/account/basics/about-your-developer-account)。官方沒寫 app extension 能不能用免費佈建，以上是實測。實測每個擴充各佔一個 App ID（本實驗用了 2 個）。
- App Groups 在「Apple Developer」（免費）那欄打勾。[Supported capabilities (iOS)](https://developer.apple.com/help/account/reference/supported-capabilities-ios)。
- 沒有完整取用：「prevents writing to the containing app's shared group containers (reading is permitted)」。[Configuring open access for a custom keyboard](https://developer.apple.com/documentation/uikit/configuring-open-access-for-a-custom-keyboard)。
- `os_proc_available_memory`：「The returned value corresponds to the current memory limit minus the memory footprint of your app at the time of the function call.」「If the calling process isn't an app, or if the process has already exceeded its memory limit, this function returns 0.」「Use the returned value as advisory information only and don't cache it.」iOS 13 起。[os_proc_available_memory](https://developer.apple.com/documentation/os/os_proc_available_memory)。文件沒說擴充算不算「app」；實測鍵盤擴充回傳非 0。
- `phys_footprint` 的文件頁只有宣告、沒有說明；Apple 給的記憶體指標是 Xcode 的 memory gauge 與 jetsam 報告（`rpages × pageSize`），「iOS measures memory use as the number of memory pages in use multiplied by page size」。[Gathering information about memory use](https://developer.apple.com/documentation/xcode/gathering-information-about-memory-use)、[Identifying high-memory use with jetsam event reports](https://developer.apple.com/documentation/xcode/identifying-high-memory-use-with-jetsam-event-reports)、[Reducing your app's memory use](https://developer.apple.com/documentation/xcode/reducing-your-app-s-memory-use)。用 `phys_footprint` 是工程推論，三次 `footprint + available` 恆定支持它。
- 鍵盤擴充的上限沒有公布數字：「that process has a limit on the amount of memory it may use. If your keyboard extension exceeds the memory limit the system terminates it.」「The memory limits vary from model to model.」[Creating a custom keyboard](https://developer.apple.com/documentation/uikit/creating-a-custom-keyboard)。

### 給之後的票

- 票 08、10、11 直接在 `scripts/experiments/ios-keyboard/` 加程式；記憶體在 `viewDidAppear` 讀 `footprint` 扣掉約 10 MB 的基線，組態要一致（這裡是 Debug），換 Release 時重量基線。
- 每 7 天要重建、重裝一次；免費帳號每台裝置最多 3 個 App，每個擴充多佔一個 App ID（10 個、7 天過期）。
- 擴充沒有完整取用時不能寫共享容器，量到的數字帶不回主 App：由操作者讀擴充畫面（iphone-use 截圖）或在 Console.app 篩 `qim.exp` 的系統 log。
- 用 iphone-use 操作時，地球鍵的 accessibility 元素被判定為遮住，點座標才有反應；長按座標會打到別的元件。
