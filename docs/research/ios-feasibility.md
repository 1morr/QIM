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

- **全部載入約 96 MB footprint**，是 brief §9 當時暫定的 30 MB 的三倍。加上空擴充約 10 MB 的基線約 106 MB，在 iPhone 18 Pro Max 的 177 MB 上限內；低階機的上限不知道。這是用 Mac 推估的；iPhone 實測少約 27 MB：五份資料包加 `Engine` 後 64.4 MB、六份 79 MB（〈鍵盤擴充接上核心〉、〈記憶體上限〉）。
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

## 鍵盤擴充接上核心

M2 票 08，2026-11-10。實驗程式在 `scripts/experiments/ios-keyboard/Keyboard/CoreProbe.swift`，量法與重量步驟見該目錄 README〈接上核心（票 08）〉；每鍵延遲的迴圈與輸入和 Mac 共用（`scripts/experiments/ffi-swift/` 的 `Latency.swift`、`inputs.sh`）。

裝置：iPhone 18 Pro Max（iPhone19,7），iOS 27.2。Xcode 27.1（Swift 6.4），Rust 1.99.1，uniffi 0.32.2；Swift 是 Debug 組態，Rust 靜態函式庫是 release（`cargo xtask ffi --ios`）。擴充沒有完整取用。資料包由 commit `232d528` 建。在兩個新程序裡各量一次（pid 11903、11913）。

### 結論

- **能打字。** 備忘錄裡按擴充的 `a`、`n`、`r`，候選列首選是「門口」；按空白，備忘錄出現「門口」（兩個程序各一次）。截圖：`.scratch/m2/evidence/08-load-memory.png`（載入各步）、`08-notes-anr.png`、`08-notes-commit.png`、`08-notes-latency.png`（第 1 次），`08-notes-anr-run2.png`、`08-notes-latency-run2.png`（第 2 次）。
- **常駐記憶體：五份資料包加 `Engine` 讓 footprint 多 51.4 MB，載入後總共 64.4 MB。** 兩次分別 +51.49、+51.39 MB。載入前的 footprint 約 13 MB：這個程序 `viewDidAppear` 時 11.7 MB，比票 06 的空擴充（約 10 MB）多，因為擴充連結了核心；加上按「載入」前幾次畫面重建。打字中 63.5–64.1 MB，跑完 12 萬鍵的延遲後 63.8 MB。iPhone 18 Pro Max 給擴充的上限是 177 MB（票 06），載入後用了 36%，還剩約 113 MB（`os_proc_available_memory`）。扣掉票 06 空擴充的 10 MB 基線，核心連同資料包約 54 MB；比 brief §9 暫定的 30 MB 多。
- **詞庫最大：+41.4 MB**，佔核心增量的 80%，是檔案（4.95 MB）的 8.4 倍。bigram +5.3 MB、英文 +2.5 MB、速成 +0.6–1.4 MB、簡繁 +0.7 MB；`Engine` 與 `Session` 幾乎不加（+0.06–0.14 MB、0）。
- **比 Mac 少約 27 MB。** 同五份資料包加 `Engine` 在 Mac 上（票 04 那張表的對應各列）合計 78.0 MB。差在 bigram（Mac +12.3 MB）與 `Engine::new`（Mac +12.8 MB），詞庫只差 5 MB。推論 iOS 的 malloc 把前一步釋放的頁（讀檔的緩衝區、uniffi 把位元組複製進 Rust 的那份）重用給後面的配置，所以 `Engine::new` 的暫存 `HashSet` 沒有另外長出 footprint；載入後 footprint 也從 64.4 降到 63.5 MB，Mac 上釋放的頁不會退出 footprint。兩邊載入順序不同（Mac 在 bigram 之後還載入了字根提示），單步的數字不能直接比，總量的差距才有意義。
- **載入 120–138 ms**（背景執行緒），其中詞庫 70 ms、`Engine` 27–33 ms。
- **每鍵延遲：p50 45 µs、p95 64 µs；選開頭一段 p50 246 µs、p95 0.79 ms。** 比 Mac 經 Swift 綁定的同一批輸入（p95 58 µs、選段 0.76 ms）慢 3.5–10%。Mac 那次沒載入簡繁資料包；它只在簡體模式用到，預設關，打字的路徑不經過它。brief §9 的目標是每鍵 50 ms，最重的選段那一鍵 p95 也只用了 1.6%。
- **手按的鍵：** 核心呼叫 0.15–0.77 ms（載入後第一鍵 `a` 0.77 ms，之後 0.15–0.48 ms），比批次量的 p50 慢幾倍；推論是兩鍵之間閒置 1.5 秒，快取變冷、CPU 降頻。含畫面更新的大概值（從按鍵處理開始到這次畫面的 Core Animation transaction 送出）6.8–11.9 ms，不含觸控送達與螢幕刷新。
- 整個程序只建一個 `Engine` 與一個 `Session`（放在靜態變數）。iphone-use 每次點擊與截圖都讓 view controller 重建（第 1 次的程序重建了 11 次），footprint 沒有翻倍：重建 7 次（含跑一次延遲）共多 3.1 MB，每次約 0.3–0.5 MB，推論來自畫面本身。

### 原始輸出摘要

鍵盤上方顯示的量測（第 2 次，`08-notes-latency-run2.png`）：

```text
viewDidLoad footprint=7.02MB available=170.0MB pid=11913 loads=1
viewDidAppear footprint=11.67MB available=165.3MB
core-before footprint=13.03MB available=164.0MB
quick file=0.43MB +0.62MB → 14.45MB 7ms
quick-lexicon file=4.95MB +41.36MB → 55.81MB 70ms
zh-hant-bigram file=2.87MB +5.28MB → 61.10MB 11ms
opencc-hk2s file=0.06MB +0.73MB → 61.85MB 2ms
scowl-en-us file=0.76MB +2.52MB → 64.36MB 12ms
engine +0.06MB → 64.42MB 33ms
session +0.00MB → 64.42MB 0ms
core-loaded 138ms footprint footprint=64.42MB available=112.6MB
typing footprint=63.49MB available=113.5MB
keys a=0.768/8.6 n=0.363/8.3 r=0.476/6.8 space=0.164/7.9
| iPhone | 全部 | 6722 | 123330 | 45.08 | 63.96 |
| iPhone | 第 20 碼起 | 2587 | 27142 | 55.62 | 70.83 |
| iPhone | 選開頭一段 | 6411 | 6411 | 245.46 | 786.54 |
latency done 9.7s footprint footprint=63.75MB available=113.2MB
```

每份資料包「讀檔 → 解開 → 丟掉檔案位元組」前後的 footprint 差（MB）：

| 步驟 | 檔案 | 第 1 次 | 第 2 次 | Mac（票 04） |
| --- | ---: | ---: | ---: | ---: |
| 載入前 | — | 12.95 | 13.03 | — |
| 速成 `quick` | 0.43 | +1.42 | +0.62 | +1.38 |
| 詞庫 `quick-lexicon` | 4.95 | +41.36 | +41.36 | +46.69 |
| bigram `zh-hant-bigram` | 2.87 | +5.28 | +5.28 | +12.34 |
| 簡繁 `opencc-hk2s` | 0.06 | +0.75 | +0.73 | +0.75 |
| 英文 `scowl-en-us` | 0.76 | +2.53 | +2.52 | +4.09 |
| `Engine` | — | +0.14 | +0.06 | +12.78 |
| `Session` | — | +0.00 | +0.00 | — |
| 核心合計 | 9.07 | +51.49 | +51.39 | +78.03 |
| 載入後 | — | 64.44 | 64.42 | — |

每鍵延遲（µs，量法同〈綁定開銷〉；Mac 那列是同一節的 Swift 經 `qim-ffi`）：

| 環境 | 全部 p50 | 全部 p95 | 第 20 碼起 p50 | 第 20 碼起 p95 | 選開頭一段 p50 | 選開頭一段 p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| iPhone 第 1 次 | 45.62 | 64.00 | 56.12 | 70.96 | 246.38 | 789.75 |
| iPhone 第 2 次 | 45.08 | 63.96 | 55.62 | 70.83 | 245.46 | 786.54 |
| Mac（Apple M6） | 42.25 | 58.08 | 51.62 | 64.38 | 235.08 | 759.71 |

手按的鍵（ms，核心呼叫／含畫面更新）：第 1 次只讀到 `r` 0.450 / 11.86、空白 0.146 / 8.48（畫面一次只顯示最後一鍵，之後改成顯示最近 4 鍵）；第 2 次見上方原始輸出。

擴充的 bundle 12 MB：五份資料包 9.07 MB、延遲的輸入 0.13 MB，其餘是程式（Debug）。

### 限制

- 只有一台旗艦機、一個 iOS 版本；低階機的上限與 CPU 都可能更低。
- 沒載入字根提示與佈局：字根提示在 Mac 上解開後 +16.3 MB（heap 14.8 MB），殼要顯示時才需要；佈局 `qim-ffi` 還沒有介面。票 09 加壓時把它們算進去。
- iPhone 上沒量 heap（malloc 還在用的位元組），所以分不出 footprint 裡多少是還在用的、多少是載入途中的峰值留下的。
- 擴充的 log 這次讀不到完整的核心行：手機一開始走 Wi-Fi，接上 USB 後的 log 又被系統訊息洗掉。數字都以截圖讀回的畫面為準。

### 給之後的票

- 票 09：從載入後的 64 MB 開始加壓，記下終止點；資料包格式的決定看詞庫（核心增量的 80%）。iOS 上釋放的頁會被重用，票 04 建議的「不改格式也能壓低峰值」兩項在 iPhone 上省得比 Mac 少，`Engine::new` 的暫存這裡幾乎沒長出 footprint。
- M3：選開頭一段那一鍵 p95 0.79 ms，乘上神經模型的次數再看預算。

## 小型神經模型

M2 票 11，2026-11-10。量測程式、模型形狀、量法與完整結果：`scripts/experiments/neural-cost/`；iPhone 上的操作：`scripts/experiments/ios-keyboard/README.md`〈小型神經模型（票 11）〉。

裝置同上：iPhone 18 Pro Max（iPhone19,7），iOS 27.2（24B119），擴充 Debug 組態、沒有完整取用；Mac18,5（Apple M6），macOS 27.0.1。candle-core、candle-nn 0.11.1，CPU 後端，2 條執行緒；Rust 1.99.1，靜態庫 release 組態。

模型（隨機權重，只量開銷）：

- Gboard 級：CIFG-LSTM，670 單元、投影 96、字表 1 萬、共用 embedding，1,412,250 參數。
- 青簡級：GPT-2 式字級 decoder，字表 8000、寬 512、6 層、8 頭，23,142,400 參數。青簡只公開約 23M 與上下文 256，其餘是近似。
- 各有 f32（safetensors）與 q8（線性層與 embedding 存 Q8_0，GGUF）兩種。
- 一次重排：上下文 0、50、200 字，後接 6 條各 8 字的候選路徑，上下文只算一次、路徑一批算完。

### 結論

- **candle 能在 iOS 鍵盤擴充裡跑。** 不用改 candle 就編得過 `aarch64-apple-ios`，連進擴充、沒有完整取用也能載入模型推論。不需要試 Core ML 等替代。
- **延遲都在停鍵層用得起的範圍，前提是前綴快取。** 量的寫法是上下文只算一次、狀態給 6 條路徑共用（見下方〈前綴快取〉）。iPhone 上重排一次的 p95：
  - 1.4M：上下文 200 字 f32 8.2 ms、q8 6.4 ms，短的上下文更快。
  - 23M q8：上下文 0 字 9 ms、50 字 19 ms、200 字 55 ms。
  - 23M f32：上下文 200 字 78 ms。
  - 對照：青簡的停鍵防抖是 macOS 80 ms、Windows 30 ms（`docs/research/qingjian.md` §4.2）。
  - 載入 2–36 ms，可以等第一次停鍵才載入。
- **記憶體：1.4M 可忽略；23M 只能用 q8，f32 放不下。** iPhone 上的峰值增量：

  | 模型 | 峰值增量 |
  | --- | ---: |
  | 1.4M f32 | +4.7 MB |
  | 1.4M q8 | +2.4 MB |
  | 23M q8 | +46.9 MB |
  | 23M f32 | +94.9 MB |

  對 177 MB 的上限（票 09 加壓確認，〈記憶體上限〉），擴充載入核心與六份資料包（含字根提示）後 footprint 79 MB（同節），再加模型的峰值增量：
  - 23M f32 合計約 174 MB，離上限約 3 MB，等於放不下。
  - 23M q8 合計約 126 MB，在記憶體警告線（約 142 MB）以下。
  - 1.4M 合計約 81–84 MB。
  - 這是把兩個程序量到的增量相加。模型的峰值是在沒載入核心的擴充裡量的，兩者在同一個程序時 malloc 會互相重用頁，實際合計可能較低。低階機的上限不知道。
  - 本票當時用票 08 的五份資料包（64.4 MB），得 f32 約 159 MB、q8 約 111 MB；改用 Mac 的核心數字（〈Mac 上的資料包記憶體〉約 96 MB，加空擴充 10 MB）則是 f32 約 201 MB、q8 約 153 MB。iPhone 上核心比 Mac 少約 27 MB，原因見〈鍵盤擴充接上核心〉。
- **23M q8 的常駐約是檔案的兩倍。** candle 0.11.1 在 aarch64 第一次做 Q8_0 乘法時另存一份重排過的權重，原本那份也留著，沒有開關：檔案 23.6 MB，iPhone heap +48.3 MB。heap 裡約 22 MB 是重排的副本（Q8_0 那幾個矩陣再一份），其餘約 3 MB 是 gemm、rayon 的暫存；自己寫 int8 矩陣乘法（青簡的做法）可以省掉那份副本。
- **執行緒數要自己限。** gemm 在每條跑過矩陣乘法的執行緒各留一塊 L2 大小的暫存。Mac 上每塊約 10 MB，用每核一條的全域 pool 時共 136 MB；iPhone 上小得多（1.4M f32 的 heap 只比檔案多 1 MB）。本實驗用 2 條的專屬 pool。
- **iOS 會還回釋放的大塊配置，macOS 不會。** iPhone 上推論的暫存跑完就退出 footprint，模型丟掉後 footprint 回到開始時的水準；Mac 上停在峰值。上限看的是峰值，所以 iPhone 的數字以 `ledger_phys_footprint_peak` 為準。
- 對 QIM：
  - 停鍵層在 iOS 可以用 1.4M 級的模型，成本幾乎是零。
  - 23M 級只能用 q8；票 09 據此把 M3 模型的峰值增量上限定為 ≤ 60 MB、資料包格式不改（plan §7.8、§8）。
  - 兩種都還沒有正確率數字，值不值得留給 M3 模型階梯（plan §7.8）。
  - 依 plan §8，放不進才寫回 brief §1.5；23M q8 在這台放得進，所以不改 brief §1.5。
  - M3 實作停鍵層的神經模型時要做前綴快取，否則延遲會是這裡的 3–5 倍（plan §7.8）。

### 前綴快取

一次重排是上下文後接 6 條候選路徑。上面的數字都是「上下文只算一次、狀態（LSTM 的 h、c，decoder 各層的 K、V）複製給每條路徑」的寫法。青簡的筆記沒寫這種共用：64 字上下文配 8 條候選在 M1 Mac（Metal）上要 133 ms（`docs/research/qingjian.md` 引的 `neural-rescoring.md`），比本實驗同量級的數字慢得多。

Mac 上另量了不共用的對照：每條路徑各自從句首連上下文算起，其餘照舊（`PREFIX=unshared scripts/experiments/neural-cost/run.sh`）。p50 / p95（ms），3 個程序的中位數：

| 模型 | 上下文 0 | 上下文 50 | 上下文 200 | 200 字時慢幾倍（p50） |
| --- | ---: | ---: | ---: | ---: |
| 1.4M f32 | 4.4 / 4.9 | 12.7 / 13.0 | 37.8 / 38.1 | 4.4 |
| 1.4M q8 | 1.6 / 1.6 | 5.1 / 5.3 | 15.8 / 15.9 | 4.2 |
| 23M f32 | 32.8 / 35.7 | 96.2 / 102.6 | 298.8 / 306.5 | 4.7 |
| 23M q8 | 7.9 / 9.1 | 41.6 / 46.5 | 158.7 / 172.6 | 4.7 |

上下文 50 字慢約 3.3–3.4 倍（23M），上下文 0 字也慢 1.1–2 倍（共用時 6 條一批算，不共用時逐條算）。iPhone 沒量不共用的版本；照 Mac 的倍數推，23M q8 在上下文 200 字會到約 250 ms。記憶體與共用時相近（23M q8 峰值 +66.6–72.7 MB，對共用的 +73.6 MB）。量的時候機器有別的 session 在跑，23M f32 有一個程序的 p95 偏高（77.5、201.9 ms）。

### 結果

延遲（ms，p50 / p95；iPhone 每個模型一個新程序、30 次；Mac 是 3 個程序的中位數）：

| 模型 | 機器 | 載入 | 第一次推論 | 上下文 0 | 上下文 50 | 上下文 200 |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1.4M f32 | iPhone | 4.5 | 15.7 | 3.2 / 4.9 | 4.4 / 4.5 | 8.1 / 8.2 |
| 1.4M q8 | iPhone | 2.1 | 12.6 | 2.4 / 4.3 | 3.3 / 3.4 | 6.2 / 6.4 |
| 23M f32 | iPhone | 35.7 | 28.6 | 15.8 / 15.9 | 29.9 / 30.1 | 69.4 / 77.8 |
| 23M q8 | iPhone | 17.3 | 34.1 | 8.9 / 9.0 | 18.8 / 19.0 | 53.1 / 54.5 |
| 1.4M f32 | Mac | 0.5 | 3.1 | 2.9 / 3.0 | 4.3 / 4.4 | 8.5 / 8.7 |
| 1.4M q8 | Mac | 0.3 | 1.8 | 1.4 / 1.4 | 2.0 / 2.1 | 3.8 / 3.9 |
| 23M f32 | Mac | 9.0 | 17.6 | 15.7 / 15.9 | 28.9 / 29.6 | 63.3 / 64.2 |
| 23M q8 | Mac | 3.0 | 11.1 | 6.1 / 6.3 | 12.2 / 13.4 | 33.6 / 34.0 |

記憶體（MB；載入與峰值是比開始時多的 footprint，heap 是跑完時 malloc 還在用的增量）：

| 模型 | 檔案 | iPhone 載入 | iPhone 峰值 | iPhone heap | Mac 載入 | Mac 峰值 | Mac heap |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1.4M f32 | 5.39 | 3.92 | 4.67 | 6.48 | 5.48 | 7.05 | 25.42 |
| 1.4M q8 | 1.62 | 0.89 | 2.38 | 2.71 | 2.33 | 4.34 | 2.64 |
| 23M f32 | 88.29 | 87.53 | 94.92 | 89.30 | 88.88 | 116.42 | 108.31 |
| 23M q8 | 23.57 | 18.47 | 46.86 | 48.32 | 29.88 | 73.59 | 67.31 |

iPhone 的原始輸出（23M q8，程序 11873，`viewDidAppear` 10.86 MB；`.scratch/m2/evidence/11-iphone-G8.png`）：

```text
nc-before footprint=12.81MB available=164.2MB pid=11873
gpt-23m q8 params=23.14M file=23.57MB threads=2
load=17.3ms first(ctx=0)=34.1ms
ctx=0 p50=8.9ms p95=9.0ms n=30 fp+20.9MB
ctx=50 p50=18.8ms p95=19.0ms n=30 fp+20.2MB
ctx=200 p50=53.1ms p95=54.5ms n=30 fp+20.6MB
footprint 12.88MB load+18.47MB infer+20.56MB total+39.03MB heap+48.32MB
peak before=13.45MB after=59.74MB peak+46.86MB
nc-after footprint=13.63MB available=163.4MB wall=2.7s
```

23M f32（程序 11872，`11-iphone-G32.png`）：`load+87.53MB infer+-0.09MB`，`peak before=13.38MB after=107.74MB peak+94.92MB`，`nc-after footprint=13.14MB`。另兩張截圖是 `11-iphone-L32.png`、`11-iphone-L8.png`。

### 限制

- 一台旗艦機、一個 iOS 版本，每個模型在 iPhone 只量一個程序；量時手機上有一個子母畫面影片在播，會和推論搶 CPU，延遲可能偏高。
- Mac 的數字會隨負載飄：code-review 時在機器較忙時重跑，23M 的峰值高了 3–10 MB、延遲也較高（`scripts/experiments/neural-cost/README.md`〈上次結果〉）。iPhone 一次樣本的記憶體也可能有同樣幅度的差，合計的數字（例如 23M f32 離上限約 3 MB）要看成大約的數。
- 權重是隨機的、形狀有近似，正確率與真正的青簡、Gboard 模型無關；開銷只看形狀。
- 只試了 candle 的 CPU 後端：沒試 `accelerate`、`metal` feature，也沒試 Core ML。青簡在 M1 Mac 上量過 Metal 比 Accelerate 快一倍、Accelerate 比 gemm 快一倍（青簡 `docs/notes/neural-rescoring.md`）。鍵盤擴充能不能用 Metal 沒查。
- 候選路徑固定 6 條、每條 8 字；8 字是自訂的近似（plan §7.8 沒定路徑長度），真實的路徑長短不一，延遲大致隨字數增加。
- 擴充是 Debug 組態（Swift 部分），數字都在 Rust 的 release 靜態庫裡，Swift 的組態不影響推論；擴充的基線與票 06 同為 Debug。
- 量測時擴充的畫面被 iphone-use 重建過幾次，開始時的 footprint（12.6–12.8 MB）比 `viewDidAppear`（約 10.9 MB）高；表上都是對開始時的增量。

### 官方說法

- 上限與量法見本文〈免費佈建與 App Group〉。`ledger_phys_footprint_peak` 是 `task_vm_info` 在 REV3 加的欄位，`<mach/task_info.h>` 只有宣告、沒有說明；從名字推定是 footprint ledger 的最高點，實測 Mac 上它等於跑完時的 footprint（footprint 不退），iPhone 上比跑完時高，與「最高點」一致。
- candle 的 Q8_0 重排與 gemm 的暫存都是讀原始碼與 `heap` 查到的（`candle-core` 0.11.1 的 `src/quantized/repack.rs`、`gemm-common` 0.19.0 的 `src/gemm.rs`），不是官方文件。

## 記憶體上限

M2 票 09，2026-11-10。實驗程式在 `scripts/experiments/ios-keyboard/Keyboard/StressProbe.swift`，量法與重量步驟見該目錄 README〈記憶體上限（票 09）〉。

裝置：iPhone 18 Pro Max（iPhone19,7），iOS 27.2。Xcode 27.2（27B52），iOS SDK 27.2，Swift 6.4；Rust 1.99.1，uniffi 0.32.2。Swift 是 Debug 組態，Rust 靜態函式庫是 release（`cargo xtask ffi --ios`）。擴充沒有完整取用，沒開 `QIM_NEURAL`。資料包與程式由票 09 的 commit 建（實驗碼、`qim-ffi` 的 `RadicalHintPack` 都在其中，父 commit `a096d79`）。

### 結論

- **上限是 177 MB，系統以 footprint 判定。** 第 5 輪終止時 kernel 寫 `exceeded mem limit: ActiveHard 177 MB (fatal)` 與 `per-process-limit … 181249KB`（= 177.0 MiB，本文的 MB 都是 MiB）。SpringBoard 記 `domain:jetsam(1) code:per-process-limit(7)`。只有這一輪取到 log（第 1–3 輪被洗掉，第 4、6、7 輪沒讀），其餘以擴充自己的紀錄為準。7 輪終止前的最後一筆 footprint 都在 176.00–176.81 MB，每步 1 MB，下一步就越過 177 MB。票 06 的 `footprint + available` = 177.0 MB 與這個數字一致，`os_proc_available_memory()` 可以直接當成剩餘量。
- **載入核心與否，上限都一樣。** 先載入核心與六份資料包（79 MB）再加壓，還能再佔 100 MB，終止點同樣在 176.3–176.7 MB。載入途中釋放的頁沒有讓上限提早到來：從開始值加上之後佔的 99 MB 推算，終點應在 177.4–177.9 MB，實測少約 1.1 MB，是新配置重用了約 1 MB 釋放的頁（不載核心的幾輪沒有這個差）。
- **約 142 MB 會收到記憶體警告。** 7 輪的第一次 `didReceiveMemoryWarning` 都在 141.55–142.58 MB，當時 available 約 35 MB（上限的 80%）。只有一輪收到第二次警告。警告之後若不釋放，再多約 35 MB 就被終止。
- **六份資料包加 `Engine` 載入後 footprint 79 MB**（78.7–79.2 MB，三輪），是上限的 45%，還剩約 98 MB（`os_proc_available_memory` 97.8–98.6 MB）。其中：
  - 程序本身約 13 MB：Debug 的 Swift，加上 iphone-use 重建畫面。
  - 五份資料包加 `Engine` +51.5 MB：與票 08 相同，詞庫佔 41.5 MB。
  - **字根提示實測 +14.3–14.5 MB**：Mac 上是 +16.3 MB（〈Mac 上的資料包記憶體〉）。為了量它，`qim-ffi` 加了只能解開的 `RadicalHintPack`。
  - 佈局沒有實測：`qim-ffi` 還沒有佈局的介面。下面的預算用 Mac 的數字估：不驗證 +0.2 MB，經 Schema 驗證 +3 MB。
- **沒有留下報告檔。** 擴充被終止時，`ReportCrash` 只送一筆分析事件（`sending CA event for QIMKeyboard[385]`）。手機的系統 crash 目錄（`xcrun devicectl device info files --domain-type systemCrashLogs`）沒有新的 `JetsamEvent-*.ips`，也沒有 `QIMKeyboard-*.ips`。能取到的證據只有系統 log 與擴充自己寫的紀錄，取法見 README。同一台手機上較早的 `JetsamEvent` 報告記過 SpringBoard 的 `per-process-limit`，所以報告機制本身是開著的。

### 給 M3 的預算（這台旗艦機）

| 項目 | footprint |
| --- | ---: |
| 上限 | 177 MB |
| 記憶體警告線（實測） | 約 142 MB |
| 核心加六份資料包（實測） | 79 MB |
| 佈局（Mac 估算，不驗證） | +0.2 MB |
| 到警告線還剩 | 約 62 MB |
| 到上限還剩 | 約 98 MB |

小型神經模型的峰值增量（〈小型神經模型〉，iPhone 一次樣本）放進這個預算：

- 1.4M f32 / q8：+4.7 / +2.4 MB，合計約 84 MB。
- 23M q8：+46.9 MB，合計約 126 MB，在警告線下、離上限 51 MB。
- 23M f32：+94.9 MB，合計約 174 MB，離上限只剩約 3 MB，一有波動就被終止。票 11 推算的「剩約 18 MB」沒算字根提示。

模型的峰值是在沒載入核心的程序裡量的，同一個程序時 malloc 可能互相重用頁，實際合計可能稍低，不會更高。

### 決定（寫回 plan）

- **M3 模型上限：iOS 鍵盤擴充裡，模型的 footprint 峰值增量 ≤ 60 MB。** 理由是核心加資料包加模型要在記憶體警告線（約 142 MB）以下，不靠警告後釋放來保命。23M 級只能用 q8（+47 MB），f32 不行。寫進 plan §7.8。
- **資料包格式不改。** 核心加資料包用了上限的 45%，放進 23M q8 後還在警告線下。mmap 友善的格式（詞庫優先，見〈Mac 上的資料包記憶體〉〈建議〉）留到低階機的上限量出來、放不下時再做（plan §4、§8）。票 04 的兩項「不改格式也能壓低峰值」不另開票：iOS 的 malloc 重用釋放的頁，`Engine::new` 的暫存在 iPhone 上只有 +0.1–0.3 MB，讀檔緩衝區也沒有留在 footprint 裡（〈鍵盤擴充接上核心〉）。
- **鍵盤擴充略過 Schema 驗證。** 佈局在主 App 與編輯器存檔時已驗過，擴充只讀。驗證讓執行檔多 3.1 MB、footprint 多約 3 MB（Mac），佔掉警告線前餘裕的 5%，但沒有保護作用。`qim-layout` 的 `jsonschema` 改成 feature，鍵盤擴充用不驗證 Schema、只做型別與引用檢查的讀法。實作留給 M7（iOS 殼要讀佈局時，`qim-ffi` 同時加佈局介面）。寫回 plan §1.3、§1.4、§8。

### 原始輸出摘要

每輪是一個新程序。「連續加壓」每 40 ms 佔 1 MB 並用 `arc4random_buf` 寫滿。每步把 footprint 寫進擴充自己容器裡的 `stress.log` 並 fsync，下次叫出鍵盤時顯示上一輪的開頭、第一次警告與最後一步。

| 輪 | 情況 | pid | 開始 footprint（佔完第一個 1 MB 後） | 第一次警告（已佔） | 最後一筆：已佔 → footprint，available |
| ---: | --- | ---: | ---: | --- | --- |
| 1 | 不載入核心 | 12589 | 13.45 MB | 141.55 MB（129 MB），共 2 次 | 164 MB → 176.72 MB，0.3 MB |
| 2 | 不載入核心 | 12595 | 13.50 MB | 141.86 MB（129 MB） | 163 MB → 176.03 MB，1.0 MB |
| 3 | 不載入核心 | 12601 | 12.94 MB | 141.67 MB（129 MB） | 164 MB → 176.81 MB，0.2 MB |
| 4 | 不載入核心 | 12606 | 13.14 MB | 141.95 MB（129 MB） | 163 MB → 176.00 MB，1.0 MB |
| 5 | 載入核心後 | 12609 | 78.88 MB | 142.58 MB（66 MB） | 100 MB → 176.74 MB，0.3 MB |
| 6 | 載入核心後 | 12612 | 78.44 MB | 142.19 MB（66 MB） | 100 MB → 176.33 MB，0.7 MB |
| 7 | 載入核心後 | 12614 | 78.53 MB | 142.22 MB（66 MB） | 100 MB → 176.35 MB，0.7 MB |

第 1–3 輪加壓時，鍵盤每次重畫都會查一次 Foundation Models 的可用性，系統 log 被洗掉、讀不到終止那幾行。第 4 輪起加壓中不畫那幾行，數字沒變。第 5 輪終止時的系統 log 摘錄在 `.scratch/m2/evidence/09-core-run1-kill-log.txt`：

```text
kernel[0] notice: memorystatus: QIMKeyboard [12609] exceeded mem limit: ActiveHard 177 MB (fatal)
kernel[0] notice: memorystatus: killing_specific_process pid 12609 [QIMKeyboard] (per-process-limit 100 33s rf:- type:app) 181249KB - memorystatus_available_pages: 286430
SpringBoard[12505] (FrontBoard) notice: [xpcservice<com.personal.qim.exp.keyboard>:12609] Process exited: <RBSProcessExitContext| specific, status:<RBSProcessExitStatus| domain:jetsam(1) code:per-process-limit(7)>>.
ReportSystemMemory[12475] (CoreDiagnostics) notice: Process QIMKeyboard [12609] killed by jetsam reason per-process-limit
```

載入核心那三輪的各步 footprint 增量（MB，「讀檔 → 解開 → 丟掉檔案位元組」前後的差，同〈鍵盤擴充接上核心〉）：

| 步驟 | 檔案 | 第 5 輪 | 第 6 輪 | 第 7 輪 |
| --- | ---: | ---: | ---: | ---: |
| 載入前 | — | 13.17 | 12.99 | 12.94 |
| 速成 `quick` | 0.43 | +1.31 | +1.33 | +1.30 |
| 詞庫 `quick-lexicon` | 4.95 | +41.48 | +41.47 | +41.44 |
| bigram `zh-hant-bigram` | 2.87 | +5.28 | +5.28 | +5.28 |
| 簡繁 `opencc-hk2s` | 0.06 | +0.75 | +0.75 | +0.77 |
| 英文 `scowl-en-us` | 0.76 | +2.52 | +2.50 | +2.52 |
| `Engine` | — | +0.19 | +0.11 | +0.28 |
| `Session` | — | +0.00 | +0.00 | +0.00 |
| 字根提示 `cangjie-hint` | 1.24 | +14.50 | +14.30 | +14.30 |
| 載入後 | — | 79.20 | 78.74 | 78.81 |

載入耗時 129–130 ms。截圖：`.scratch/m2/evidence/09-bare-run{1,2,3}.png`、`09-bare-run1-during.png`（加壓中）、`09-core-run{1,2,3}-loaded.png`（載入後，畫面同時顯示上一輪的結果）、`09-core-run3-result.png`。

### 限制

- 一台旗艦機、一個 iOS 版本。Apple 說上限隨機型而異，低階機可能低得多：社群說法 30–77 MB（brief §12.3 原本的條目）多半是舊機型、舊系統，沒有查證。
- 擴充是 Debug 組態，程序本身約 13 MB；Release 的基線沒量，預期較小，只會讓餘裕變大。
- 警告線是觀察到的值，Apple 沒有公布門檻，也可能隨系統的記憶體壓力變動；這幾輪手機上沒有其他吃記憶體的前景 App。
- 佈局的記憶體是 Mac 的數字。
- 每步 1 MB，終止點只精確到 1 MB 內。

### 官方說法（2026-11-10 查）

- jetsam 報告：「Search for the `reason` key to identify the jettisoned process and why the system jettisoned it.」「`per-process-limit`: The process crossed the resident memory limit imposed by the system on all apps. … extensions have a much lower per-process memory limit than foreground apps.」記憶體量是 `rpages × pageSize`。[Identifying high-memory use with jetsam event reports](https://developer.apple.com/documentation/xcode/identifying-high-memory-use-with-jetsam-event-reports)。
- 報告在裝置的「設定 → 隱私權與安全性 → 分析與改進 → 分析資料」，jetsam 的檔名是 `JetsamEvent_<DateTime>`，也可以接上 Mac 從 Xcode 的 Devices and Simulators 取。[Acquiring crash reports and diagnostic logs](https://developer.apple.com/documentation/xcode/acquiring-crash-reports-and-diagnostic-logs)。官方沒寫每次 `per-process-limit` 都會產生報告；實測擴充被終止時沒有產生，以上是實測。
- 上限不公布、隨機型而異、超過就終止：見本文〈免費佈建與 App Group〉〈官方說法〉的 Creating a custom keyboard。
