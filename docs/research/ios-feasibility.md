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

## Mac 上的資料包記憶體

M2 票 04，2026-11-10。實驗程式、量法、指標的定義與完整輸出：`scripts/experiments/pack-memory/`。

環境：Mac18,5（Apple M6，32 GB），macOS 27.0.1（26A434），Rust 1.99.1，release 組態開 `lto`、`strip`，commit `5186e6e` 建的資料包。指標用 `phys_footprint`（同上一節的 footprint；XNU 定義為匿名頁加壓縮的匿名頁等，不含乾淨的檔案映射頁），另記 RSS 與 malloc 還在用的位元組（heap）。各跑 3 次，差在 0.5 MB 內。這是 Mac 的數字，iOS 的 malloc 與頁大小可能不同，票 08 在 iPhone 上重量。

### 結果

每步是「讀檔 → 反序列化 → 丟掉檔案位元組」前後的差：

| 資料包 | 檔案大小 | 載入後增加的 footprint | 倍數 | 其中還在用的（heap） |
| --- | ---: | ---: | ---: | ---: |
| 速成 `quick` | 0.43 MB | 1.36 MB | 3.2× | 0.74 MB |
| 詞庫 `quick-lexicon` | 4.95 MB | 46.67 MB | 9.4× | 40.73 MB |
| bigram `zh-hant-bigram` | 2.87 MB | 12.34 MB | 4.3× | 7.45 MB |
| 字根提示 `cangjie-hint` | 1.24 MB | 16.38 MB | 13.2× | 14.78 MB |
| 簡繁 `opencc-hk2s` | 0.06 MB | 0.69 MB | 10.8× | 0.86 MB |
| 英文 `scowl-en-us` | 0.76 MB | 4.09 MB | 5.4× | 3.31 MB |
| `Engine`（拿走速成、詞庫、bigram、簡繁、英文） | — | 12.88 MB | — | −0.03 MB |
| 合計（含不驗證的佈局） | 10.54 MB | 94.56 MB | 9.0× | 68.25 MB |

有無 `jsonschema`（佈局 `gboard-quick.json` 0.23 MB）：

| 佈局讀法 | 執行檔（strip 後） | 佈局那步的 footprint | 其中還在用的 | 全部載入後的 footprint |
| --- | ---: | ---: | ---: | ---: |
| 不驗證（`serde_json` 讀進型別） | 0.67 MB | 0.12–0.16 MB | 0.40 MB | 95.6–95.8 MB |
| Schema 驗證（`qim_layout::validate`） | 3.75 MB | 2.98–3.14 MB | 0.84 MB | 98.7–98.9 MB |

### 結論

- **全部載入約 96 MB footprint**，是 brief §9 暫定的 30 MB 的三倍。加上空擴充約 10 MB 的基線約 106 MB，在 iPhone 18 Pro Max 的 177 MB 上限內；低階機的上限不知道。
- **釋放的記憶體留在 footprint 裡。** 配置 13 MB 再丟掉，footprint 照樣 +13 MB，`malloc_zone_pressure_relief` 也還不回來。所以不驗證那版的 96 MB 裡只有 68 MB 還在用，約 1 MB 是程式本身，其餘約 26 MB 大多是用完丟掉的：
  - 讀檔的緩衝區，共 10.5 MB。
  - `Engine::new` 的暫存，12.9 MB。為了算詞頻總和，它對 25.7 萬個詞建了一個 `HashSet`（同樣大小的 `HashSet` 單獨量過是 8.5 MB），又把碼表整份複製一次。
  - Schema 驗證那版另多約 2–3 MB 驗證器與 regex 的暫存。
- 吃緊時系統會不會回收這些頁、jetsam 終止前會不會先回收，沒查到依據；票 09 加壓時確認。
- **詞庫最大**：佔還在用的 68 MB 裡的 41 MB，heap 是檔案的 8.2 倍。23.4 萬個碼與 28.3 萬個詞條，每個都是獨立配置的 `String`，外加 `BTreeMap` 節點。
- **字根提示倍數最高**：13.2 倍，每個字兩個 `Vec<String>`。
- **bigram 已經是攤平的陣列**，heap 只是檔案的 2.6 倍。速成、簡繁、英文合計不到 5 MB 的 heap。

### 建議

- **鍵盤擴充略過 Schema 驗證。** 它讓執行檔多 3.1 MB；驗證一次，footprint 多約 3 MB 而且不退。佈局在主 App 與編輯器存檔時已經驗過，擴充只讀。
  - 要做的事：`qim-layout` 加一個 feature，關掉 `jsonschema`，只留型別與引用檢查（plan §8 那列）。
  - 現在的 `Layout::from_json` 一定會走 `validate`，所以擴充需要一條不經 Schema 的讀法。
  - 依 spec，決定在票 09 寫回 plan §1.3、§1.4。
- **最可能要改格式的是詞庫，其次是字根提示。**
  - 詞庫：改成 mmap 友善的格式（例如 `fst` 存碼、詞放一整塊字串、用位移索引），常駐的量會接近檔案大小（約 5 MB），而且是乾淨的檔案映射頁，推論不算 footprint。
  - 字根提示：只給殼顯示用，可以同樣改格式，或在擴充裡等使用者打開字根提示才載入。
  - bigram 可以先不動。速成、簡繁、英文都小，不必改。
- **不改格式也能先省約 23 MB**，因為丟掉的記憶體不退：
  - 讀資料包改用 mmap 或串流解析，省掉一份整檔的緩衝區（10.5 MB）。
  - `Engine::new` 不在載入時建全詞的 `HashSet`，詞頻總和改由 `qim-databuild` 算好存進資料包；碼表就地排序，不複製（12.9 MB）。
  - 這兩項是改格式之外的獨立工作，票 09 決定要不要另開票。

### 給之後的票

- 票 08：擴充先照現在的格式載入 `Engine`（速成、詞庫、bigram）加英文、簡繁與佈局。在 iPhone 上量的 footprint 跟這裡比，看 iOS 的 malloc 是否同樣不退釋放的頁。字根提示可以先不放。
- 票 09：加壓時記下終止點的 footprint 與上面的組成；決定格式、Schema 驗證與兩項「不改格式的省法」。
