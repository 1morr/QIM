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

環境：Mac18,5（Apple M6，32 GB），macOS 27.0.1（26A434），Rust 1.99.1，release 組態開 `lto`、`strip`，資料包由 commit `5186e6e` 建。

- 指標用 `phys_footprint`，同上一節的 footprint。XNU 的定義是匿名頁加上被壓縮的匿名頁等；公式沒有檔案映射項，推論不含乾淨的檔案映射頁。
- 另記 RSS，以及 malloc 還在用的位元組（heap）。
- 每個版本跑 6 次，差在 0.5 MB 內。
- 這是 Mac 的數字，iOS 的 malloc 與頁大小可能不同，票 08 在 iPhone 上重量。

### 結果

每步是「讀檔 → 反序列化 → 丟掉檔案位元組」前後的差。前一步釋放的頁會被後一步重用，所以單步的 footprint 與順序有關，合計與 heap 不受影響：

| 資料包 | 檔案大小 | 載入後增加的 footprint | 倍數 | 其中還在用的（heap） |
| --- | ---: | ---: | ---: | ---: |
| 速成 `quick` | 0.43 MB | 1.38 MB | 3.2× | 0.74 MB |
| 詞庫 `quick-lexicon` | 4.95 MB | 46.69 MB | 9.4× | 40.73 MB |
| bigram `zh-hant-bigram` | 2.87 MB | 12.34 MB | 4.3× | 7.45 MB |
| 字根提示 `cangjie-hint` | 1.24 MB | 16.33 MB | 13.1× | 14.78 MB |
| 簡繁 `opencc-hk2s` | 0.06 MB | 0.75 MB | 11.7× | 0.86 MB |
| 英文 `scowl-en-us` | 0.76 MB | 4.09 MB | 5.4× | 3.31 MB |
| `Engine`（拿走速成、詞庫、bigram、簡繁、英文） | — | 12.78 MB | — | −0.03 MB |
| 合計（含不驗證的佈局） | 10.54 MB | 94.58 MB | 9.0× | 68.25 MB |

佈局 `gboard-quick.json`（0.23 MB）有無 Schema 驗證：

- 「不驗證」只用 `serde_json` 讀進型別。
- 「Schema 驗證」是整條 `qim_layout::validate`：JSON Schema、讀進型別，再加引用與頁面檢查。所以差距是整條 `validate` 的成本，不只是 `jsonschema`。
- 「只留型別與引用檢查」的讀法沒量，`qim-layout` 沒有公開那條路。

| 佈局讀法 | 執行檔（strip 後） | 佈局那步的 footprint | 其中還在用的 | 全部載入後的 footprint |
| --- | ---: | ---: | ---: | ---: |
| 不驗證 | 0.67 MB | 0.12–0.20 MB | 0.40 MB | 95.6–95.8 MB |
| Schema 驗證 | 3.75 MB | 2.92–3.14 MB | 0.84 MB | 98.7–98.9 MB |

### 結論

- **全部載入約 96 MB footprint**，是 brief §9 暫定的 30 MB 的三倍。加上空擴充約 10 MB 的基線約 106 MB，在 iPhone 18 Pro Max 的 177 MB 上限內；低階機的上限不知道。
- **釋放的頁不退出 footprint，但之後的配置會重用它們。**
  - 配置 13 MB 再丟掉，footprint 照樣 +13 MB；前後兩次 `malloc_zone_pressure_relief` 都還回 0。
  - footprint 因此跟著 heap 用量的高水位走：不驗證那版的 96 MB 裡只有 68 MB 還在用，約 1 MB 是程式本身，其餘約 26 MB 是載入途中的峰值留下的。
  - 峰值主要來自兩處。一是讀檔的緩衝區，共 10.5 MB。二是 `Engine::new` 的暫存：為了算詞頻總和，它對 25.7 萬個詞建一個 `HashSet`，單獨量是 +17.3 MB（擴容時舊表放不進新表）；實跑時它部分重用前面釋放的緩衝區，整步 +12.8 MB。
  - Schema 驗證那版另多約 3 MB，是驗證器與 regex 的暫存。
- 記憶體吃緊時系統會不會回收這些頁、jetsam 終止前會不會先回收，沒查到依據；票 09 加壓時確認。
- **詞庫最大。** 還在用的 68 MB 裡它佔 41 MB，heap 是檔案的 8.2 倍。它有 23.4 萬個碼與 28.3 萬個詞條，每個都是獨立配置的 `String`，外加 `BTreeMap` 節點。
- **字根提示倍數最高**，13.1 倍，因為每個字存兩個 `Vec<String>`。
- **bigram 已經是攤平的陣列**，heap 只是檔案的 2.6 倍。速成、簡繁、英文三份的 heap 合計不到 5 MB。

### 建議

- **鍵盤擴充略過 Schema 驗證。**
  - 理由：整條 `validate` 讓執行檔多 3.1 MB；驗證一次，footprint 多約 3 MB 而且不退。佈局在主 App 與編輯器存檔時已經驗過，擴充只讀。
  - 做法：`qim-layout` 加一個 feature 關掉 `jsonschema`，只留型別與引用檢查（plan §8 那列）。現在的 `Layout::from_json` 一定會走 `validate`，擴充需要一條不經 Schema 的讀法。
  - 依 spec，這個決定由票 09 寫回 plan §1.3、§1.4。
- **最可能要改格式的是詞庫，其次是字根提示。**
  - 詞庫：改成 mmap 友善的格式，例如用 `fst` 存碼、詞放成一整塊字串加位移索引。常駐量會接近檔案大小（約 5 MB），而且是乾淨的檔案映射頁，推論不算 footprint。
  - 字根提示：只給殼顯示用。可以同樣改格式，或在擴充裡等使用者打開字根提示才載入。
  - bigram 可以先不動。速成、簡繁、英文都小，不必改。
- **不改格式也能先壓低峰值**，因為峰值會留在 footprint：
  - 讀資料包改用 mmap 或串流解析，省掉整檔的緩衝區（全部 10.5 MB）。
  - `Engine::new` 不在載入時建全詞的 `HashSet`，詞頻總和改由 `qim-databuild` 算好存進資料包。這一項的單獨峰值是 17.3 MB。
  - 兩項能省多少要改完重量：釋放的頁會互相重用，省下的量不是兩項相加。
  - 這兩項是改格式之外的獨立工作，票 09 決定要不要另開票。

### 給之後的票

- 票 08：擴充先照現在的格式載入 `Engine`（速成、詞庫、bigram）加英文、簡繁與佈局；字根提示可以先不放。在 iPhone 上量 footprint，跟這裡比，看 iOS 的 malloc 是否同樣不退釋放的頁。
- 票 09：加壓時記下終止點的 footprint 與上面的組成；決定格式、Schema 驗證，以及兩項「不改格式也能壓低峰值」的改法。

## Foundation Models

M2 票 10，2026-11-10。實驗程式在 `scripts/experiments/ios-keyboard/Keyboard/ModelProbe.swift`，量法與重量步驟見該目錄 README〈Foundation Models 與上下文長度〉。

裝置與環境同上一節：iPhone 18 Pro Max（iPhone19,7），iOS 27.2（24B119），Xcode 27.1，Debug 組態。擴充沒有完整取用（`RequestsOpenAccess = NO`，`hasFullAccess = false`）。Apple Intelligence 開著：iOS 27 起預設開啟、不能關（見下方官方說法），`availability` 也回 `available`。系統語言地區 `en_HK@rg=twzzzz`。

### 結論

- **能用。** 鍵盤擴充裡 `SystemLanguageModel.default.availability` 回 `available`，`variant` 是「AFM 3 Core Advanced」，`contextSize` 8192；`supportsLocale()`（目前語言）與 `supportsLocale(zh-Hant-HK)` 都是 `true`。沒有完整取用時鍵盤擴充沒有網路（上一節與 brief §12.3），所以推論在裝置上。27 次請求都成功、都回正解「3」。
- **連續請求約 190 ms。** 同一串裡緊接著的請求 15 次都在 185–194 ms（中位數 189 ms）。
- **停頓後的第一個請求不穩：** 12 次裡 8 次在 0.29–0.87 s，4 次要 6–15 s（6.1、9.1、11.0、15.5 s）。慢的那幾次跟閒置長短（2–40 s）沒有明顯關係，送出前 1 秒呼叫 `prewarm()` 也擋不住（40 s 那次 prewarm 後仍 11.0 s）。原因沒查出來，見〈限制〉。
- **擴充自己的記憶體漲得少。** 程序 11622 從 `viewDidAppear` 到按 FM 前 +2.6 MB（查可用性、載入框架，中間 iphone-use 也讓畫面重建過），每輪 6 個請求再 +0.8–1.2 MB，三輪共 10.27 → 15.31 MB；漲幅裡有多少來自畫面重建分不出來。推論模型不在擴充的程序裡跑（官方沒寫）。對 177 MB 的上限（上一節）可以忽略。
- 對 QIM：停鍵層（plan §2.7）在 iOS 可以用 Foundation Models 重排，連續請求的延遲夠用；但停頓後第一個請求可能等十幾秒，重排結果一定要能晚到、晚到時丟掉（只認目前的 `generation`，ADR 0005）。

### 量法

擴充上的「FM」鍵每按一次依序送 6 個請求；「間隔」鍵在同一個程序裡隔 0、2、5、10、20、40 秒各送 1 個，再隔 10、20、40 秒各送 1 個「先 `prewarm()`、等 1 秒才送」的請求。每個請求都新開 `LanguageModelSession`（同 IME 每次停鍵送一個獨立請求），提示照 plan §7.8：上下文、輸入碼 `anr`、前 10 個候選，要求只回序號，正解是 3（門口）；`GenerationOptions(samplingMode: .greedy, maximumResponseTokens: 4)`。延遲是 `respond(to:options:)` 前後的 `ContinuousClock` 差，不含建立 session。記憶體同上一節的 `phys_footprint`。

### 結果

| 輪 | 程序 | 停頓後第一個請求 | 之後的請求 | footprint 前 → 後 |
| --- | --- | ---: | --- | --- |
| FM 第 1 次（新程序） | 11622 | 655 ms | 191、186、185、189、187 ms | 12.89 → 14.08 MB |
| FM 第 2 次（約 30 s 後） | 11622 | 15,460 ms | 194、189、191、185、190 ms | 14.02 → 14.92 MB |
| FM 第 3 次（約 25 s 後） | 11622 | 6,082 ms | 189、186、186、189、186 ms | 14.52 → 15.31 MB |

「間隔」鍵（新程序 11632，每個都是停頓後的單一請求）：

| 停頓 | 0 s | 2 s | 5 s | 10 s | 20 s | 40 s |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 直接送 | 739 ms | 286 ms | 373 ms | 9,052 ms | 869 ms | 802 ms |
| 先 `prewarm()` | | | | 424 ms | 352 ms | 10,961 ms |

整輪 footprint 12.44 → 19.00 MB；這輪 iphone-use 截圖讓鍵盤畫面重建了十幾次（見〈限制〉），漲幅不只來自 Foundation Models。程序 11622 的 `viewDidAppear` 是 10.27 MB，按 FM 時（還沒送請求）12.89 MB。

截圖：`.scratch/m2/evidence/10-fm-notes.png`（第 1 輪）、`10-fm-notes-slow-first.png`（第 2 輪，15.5 s）、`10-fm-gaps.png`（間隔）。

### 限制

- 只有一台旗艦機、一個 iOS 版本；Core Advanced 只在 iPhone 18 Pro 系列與 iPhone Duo，其他支援的機型跑 Core 變體，延遲可能不同。
- 量不到「系統第一次把模型載入記憶體」：程序 11622 之前別的程序已經用過模型，655 ms 只代表這個擴充程序的第一次。
- iphone-use 每次點擊或截圖，鍵盤的 view controller 都會重建（同一個程序，`viewDidLoad` 再跑一次）；「間隔」那輪中途鍵盤還被收起一次（10 s 那格可能受影響）。慢的請求是不是因為擴充在那一刻被系統當成背景，沒辦法從這裡分辨。產品要再量時，改用手指操作並在 Console.app 看 `qim.exp` 的 log。
- 只試了一種提示、一種輸出長度（序號）；正確率留給 M3 模型階梯（plan §7.8）。

### 官方說法（2026-11-10 查）

- 可用條件：「In iOS 27, iPadOS 27, macOS 27, and visionOS 27 or later, Apple Intelligence is on by default and can no longer be turned off.」`appleIntelligenceNotEnabled`「occurs in iOS 26, iPadOS 26, macOS 26, and visionOS 26 or later」。機型表的 iPhone 列：「iPhone 18 Pro models, iPhone Duo」只有 AFM Core Advanced 與 AFM Cloud。「Every device has just one `SystemLanguageModel`, so your app can’t choose which variant or version to use.」[Detecting available Apple foundation models](https://developer.apple.com/documentation/foundationmodels/detecting-available-apple-foundation-models)。
- `UnavailableReason` 三種：`deviceNotEligible`「The device does not support Apple Intelligence.」、`appleIntelligenceNotEnabled`「Apple Intelligence is not enabled on the system.」、`modelNotReady`「The models aren’t available on the user’s device.」「Models are downloaded automatically based on factors like network status, battery level, and system load.」[SystemLanguageModel](https://developer.apple.com/documentation/foundationmodels/systemlanguagemodel)。SDK 27.1 的介面檔也只有這三種。
- 語言：「Apple Intelligence is available with support for these languages: … Chinese (simplified), Chinese (traditional), Japanese, and Korean.」「Device language and Siri language set to the same supported language」。[How to get Apple Intelligence](https://support.apple.com/en-us/121115)。
- `supportsLocale(_:)`：「Use this method over `supportedLanguages` to check whether the given locale qualifies a person for using this model, as this method also takes language fallbacks into consideration.」[SystemLanguageModel](https://developer.apple.com/documentation/foundationmodels/systemlanguagemodel)。
- `prewarm(promptPrefix:)`：「Use only when you have a window of at least 1 second before the call to a respond method」；「Calling this method doesn’t guarantee that the system loads your assets immediately, particularly if your app is running in the background or the system is under load.」[LanguageModelSession](https://developer.apple.com/documentation/foundationmodels/languagemodelsession)。
- 頻率限制：iOS 26 的 [`GenerationError.rateLimited`](https://developer.apple.com/documentation/foundationmodels/languagemodelsession/generationerror/ratelimited(_:))「This error will only happen if your app is running in the background and exceeds the system defined rate limit」；iOS 27 另有 `LanguageModelError.rateLimited`（SDK 27.1 介面檔），沒再限定背景。鍵盤擴充算不算背景官方沒寫。本實驗沒遇到任何錯誤。
- 只有 Private Cloud Compute 要 entitlement `com.apple.developer.private-cloud-compute`；on-device 模型的文件沒提任何 entitlement 或 Info.plist 鍵，本實驗也沒加。[Detecting available Apple foundation models](https://developer.apple.com/documentation/foundationmodels/detecting-available-apple-foundation-models)。
- **官方沒寫 app extension（含鍵盤擴充）能不能用 Foundation Models**，以上「能用」是實測。

## 上下文長度

M2 票 10，2026-11-10。實驗程式在 `scripts/experiments/ios-keyboard/Keyboard/ContextProbe.swift`。裝置同上。

### 結論

- **讀到多少由宿主決定，大致按句子切，不按段落。** 換行本身不截斷：游標在段首時，兩個宿主都會帶出上一段的結尾與換行。句子的界線在「。」之後（測試文字裡「。」後的「[」也被算進前一句，是斷句器的處理，不影響結論）。
- **備忘錄：游標所在句到句首，加上前一整句（放得下才帶），最多 300 字。** 句中 89 字（前一句 50 + 本句 39）；句首或文末只有前一句（49–50 字）；前一句長達 3000 字時只給本句的 30 字；沒有標點的 3000 字長段裡前後各 300 字。`after` 給到本句句尾，同樣最多 300 字。
- **Safari（網頁的 `<textarea>`）：句中只有游標所在句，最多 100 字。** 句中只給本句（39 字），句首或文末給前一句（49–50 字）；段首例外，給 35 字、從上一句中間切；長段裡前後各 100 字。
- **剛 `insertText` 之後讀到的是暫時的值。** 備忘錄文末插入 3000 字後立刻讀是 499 字，游標移開再移回就變回 300 字。量長度時要先讓宿主同步（移一下游標或等 `textDidChange`）。
- 空白文件的 `before`、`after` 與文末的 `after` 都是 `nil`，沒有遇到空字串。全部測試文字都在 BMP，字數與 UTF-16 長度相同。
- 對 QIM：brief §8 的「游標前最多 200 字」在 iOS 可能常拿不到。每句 50 字的測試文字只讀到一兩句，Safari 最多 100 字，備忘錄只有長句才到 300；真實文字的句長不同，這是推論。plan §7.8 量「上下文 200 字」的結果在 iOS 上只對沒標點的長句有意義；iOS 要更多上下文得自己記住這次編輯裡上屏過的字，是否值得留給 M3 決定。

### 量法

擴充上的鍵：「長文」插入 9 段、每段 40 個 10 字單元（`[段.序]天地玄` 加「，」，每第 5 個加「。」）、第 4 與第 5 段之間空一行的長文，開頭就看得出從哪個單元起讀；「長段」插入 3000 字、沒有標點與換行的一段（300 個 `[序號]天地玄黃`）；「←1000」到「→100」用 `adjustTextPosition(byCharacterOffset:)` 移游標，0.3 秒後重讀；「⇤」往前移 `before` 的長度，連按就一句一句走到段首；「讀」重讀。每次 `viewDidAppear`、`textDidChange`、`selectionDidChange` 也重讀。顯示 `documentContextBeforeInput` 與 `AfterInput` 的字數（`Character`）、UTF-16 長度、換行數、頭尾各 12 字。

宿主：備忘錄（新增一則備忘錄）；Safari 開這台 Mac 起的本機網頁，裡面一個 `<textarea>`（`python3 -m http.server`，頁面見 README）。

### 結果

備忘錄：

| 游標位置 | `before` | `before` 從哪起 | `after` |
| --- | ---: | --- | ---: |
| 文末，前一字是「。」 | 49 | 前一句句首（9.36） | `nil` |
| 句中（本句已打 39 字，句首的「[」算前一句） | 89 | 前一句句首（9.31） | 10，到句尾 |
| 句中（另一處） | 89 | 前一句句首（9.21） | 11，到句尾 |
| 句首 | 50 | 前一句句首（9.16） | 50，整句 |
| 段首（換行之後） | 50，含 1 個換行 | 上一段最後一句句首（8.36） | 51，整句 |
| 句中，前一句長 3000 字 | 30 | 本句句首 | 20，到句尾 |
| 3000 字長段中間 | 300 | 往前 300 字（0171） | 300 |
| 長段裡，離句尾 120 字 | 300 | 往前 300 字（0261） | 120，到句尾 |
| 長段在文末 | 300 | 往前 300 字（0271） | `nil` |
| 長段在文末，剛插入完 | 499 | 往前 499 字（0251 的「[」之後） | `nil` |

Safari `<textarea>`：

| 游標位置 | `before` | `before` 從哪起 | `after` |
| --- | ---: | --- | ---: |
| 文末，前一字是「。」 | 49 | 前一句句首（9.36） | `nil` |
| 句中（本句已打 39 字） | 39 | 本句句首（9.36） | 10，到句尾 |
| 句首 | 50 | 前一句句首（9.31） | 49，整句 |
| 段首（換行之後） | 35，含 1 個換行 | 上一句中間（8.37 的「天地玄，」） | 51，整句 |
| 長句裡，離句尾 51 字 | 100 | 往前 100 字（0291） | 51，到句尾 |
| 3000 字長段中間 | 100 | 往前 100 字（0191） | 100 |

截圖：`.scratch/m2/evidence/10-context-notes-*.png`（end、mid、paragraph-start、prev-sentence-too-long、long-paragraph-mid、long-paragraph-end）、`10-context-safari-*.png`（end、mid、paragraph-start、long-paragraph-mid）。

### 限制

- 兩個宿主、一台手機、一個 iOS 版本；其他 App（尤其自己畫文字的，如通訊軟體）可能又不一樣。
- 空一行（連續兩個換行）的位置沒有測到：走到那裡要按太多次，而且單一換行已經不截斷。
- iphone-use 的點擊偶爾漏掉或點穿到宿主，游標會跳到別處；每筆讀數都附頭尾文字，位置以讀數為準。
- 沒測 `<input>` 單行欄位與 `contenteditable`。

### 官方說法

官方只說鍵盤可經 `UITextDocumentProxy` 讀插入點周圍的文字，用途是 autocomplete 與 autocapitalization，沒有寫長度（`docs/research/platform-frameworks.md` §3.2）。[Handling text interactions in custom keyboards](https://developer.apple.com/documentation/uikit/handling-text-interactions-in-custom-keyboards)、[UITextDocumentProxy](https://developer.apple.com/documentation/uikit/uitextdocumentproxy)。以上長度都是實測。

## 綁定開銷

M2 票 07，2026-11-10。`qim-ffi`（uniffi 0.32.2）產生的 Swift 綁定在 Mac 上接核心，與原生、WebAssembly 量同一批輸入的每鍵延遲。重量步驟：Swift 是 `scripts/experiments/ffi-swift/`，Chromium 是 `scripts/experiments/wasm-latency-browser/`，原生與 Node 是 `cargo xtask bench`。

環境：Apple M6、32 GB、macOS 27.0.1，Xcode 27.1（Swift 6.4），Rust 1.99.1，Node v26.12.0，HeadlessChrome 156.0.8078.4；release 編譯。資料包是基準的四份（速成、詞庫、bigram、英文）。

### 結論

- Swift 經 `qim-ffi` 打 `anr` 讀回首選「門口」、空白上屏「門口」：綁定接得上，iOS 鍵盤擴充（票 08）可以直接用。
- 每鍵 p95 最慢的是 Chromium 的選開頭一段，0.9 ms；三種路徑都比每鍵 50 ms 的預算小兩個數量級以上。M1 留下的缺口（瀏覽器沒量、選段那一鍵沒算）補上了，Chromium 仍 ≤ 50 ms。
- Swift 綁定每次呼叫比原生多約 10–20 µs，和組字長短關係不大，像是固定開銷（快照序列化成 uniffi 的 `RustBuffer` 再在 Swift 解開）；WebAssembly 多約 10 µs，選段那一鍵多約 50–150 µs（WebAssembly 本身算得慢，不是呼叫開銷）。
- 選開頭一段（選定後剩下的輸入碼從頭重建 lattice）量的是「← 移到第一段、選它的首選」這條路；在整串的候選裡直接點只蓋住開頭的候選（例如 `anr` 的「門」）同樣從頭重建剩下的碼，沒有另外量。它比一般按鍵慢約 7 倍，p95 約 0.75 ms（原生）；M3 加神經模型時要看它，這一鍵會乘上模型的次數。
- Chromium 與 Node 差不到一個 5 µs 的刻度，基準照舊用 Node（`wasm-latency-browser` 的結論沒變）。

### 原始輸出摘要

6,722 個輸入（固定題、`bench/corpus/`、`bench/colloquial/`），123,330 鍵；長到 20 碼的 2,587 個、第 20 碼起 27,142 鍵；打完有兩段以上、量選開頭一段的 6,411 個。各跑兩次，差在 0.5 µs（Chromium 一個刻度）內，列一次。µs。

| 路徑 | 全部 p50 | 全部 p95 | 第 20 碼起 p50 | 第 20 碼起 p95 | 選開頭一段 p50 | 選開頭一段 p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 原生（`cargo xtask bench`） | 30.71 | 41.75 | 34.75 | 44.25 | 222.25 | 737.12 |
| Swift 經 `qim-ffi` | 42.25 | 58.08 | 51.62 | 64.38 | 235.08 | 759.71 |
| WebAssembly，Node（`cargo xtask bench`） | 39.67 | 53.13 | 44.79 | 56.46 | 277.04 | 889.33 |
| WebAssembly，Chromium（`crossOriginIsolated`，5 µs 刻度） | 40.00 | 55.00 | 45.00 | 55.00 | 275.00 | 890.00 |

Swift 程式（含核心）連結後 163 KB。

### 給之後的票

- 票 08：鍵盤擴充用 `cargo xtask ffi` 的綁定；要加 `aarch64-apple-ios` 的靜態函式庫與 xcframework（`docs/development.md`〈Swift 綁定〉）。在 iPhone 上量同一批輸入的每鍵延遲時，可以照 `scripts/experiments/ffi-swift/main.swift` 的 `latency` 迴圈，和這裡的 Mac 數字比。
- `qim-ffi` 目前只有建立、`press`、`select`；換選項、學習資料存檔、字根提示、佈局由需要的票加。換選項時讓綁定的 `set_options` 同時重列並回快照，殼就碰不到「待生效選項」（票 01 轉入的建議，見票 07 的 Comments）。
