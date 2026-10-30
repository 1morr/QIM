# 五平台輸入法框架：能做與不能做的事

查證日：2026-10-30。範圍：iOS 鍵盤擴充、Android InputMethodService、macOS InputMethodKit（IMK）、Windows TSF、Linux Fcitx5 / IBus（含 Wayland）。

標記規則：

- **官方**：Apple、Android Developers、Google、Microsoft Learn、Fcitx5 / IBus / Wayland / GNOME 的官方文件或原始碼。
- **社群實測**：第三方專案原始碼、README、論壇、部落格。
- **未查證**：沒找到第一手來源。不要當成事實用。

本檔只列事實與取捨，不決定平台順序、核心語言或隱私設計。

---

## 1. 摘要

1. **iOS 是限制最多的平台**。鍵盤跑在獨立擴充程序，只能經 `textDocumentProxy` 看到游標前後的文字；密碼欄與電話欄（phone pad）由系統強制換回系統鍵盤；App 可以整個拒絕第三方鍵盤；有記憶體上限但 Apple 不公布數字；網路、剪貼簿、共享容器寫入都要使用者開 Full Access。App Review 4.4.1 要求「沒有 Full Access 也必須能用」。
2. **Android 限制最少的行動平台**。IME 是一般 App 的 service，是預設 IME 時讀剪貼簿是官方例外，可申請 `INTERNET`，沒查到官方記憶體硬上限。密碼欄仍交給當前 IME，只要求 IME 自己不顯示、不儲存。Gemini Nano（ML Kit GenAI）官方只允許「最上層前景 App」推論，IME 程序算不算未查證，是風險點。
3. **macOS IMK 沒有「游標前 N 字」API**，只能用 `selectedRange` 加 `attributedSubstring`，而且協定標頭明說很多 App 會回 `nil` 或 `NSNotFound`。沒有 email / URL 欄位類型提示。密碼欄只能靠 Secure Event Input 判斷。
4. **Windows TSF 的 DLL 會被載入每個宿主程序**。官方要求密碼欄應直接停用 text services，不要靠 `IS_PASSWORD`。app container 宿主內網路不保證可用。成熟專案（Weasel、PIME、青簡）都把引擎放獨立 server 程序，DLL 只做薄層。
5. **Linux 欄位類型資訊最豐富**（Wayland `content_purpose` / `content_hint`、IBus `InputPurpose` / `InputHints`、Fcitx5 `CapabilityFlag`），沒有硬記憶體上限，也沒有沙箱。代價是碎片化：GNOME 走 IBus 加 text-input-v3，不實作 input-method-v2；Fcitx5 addon 只有 C++ API。
6. **共用核心已有先例**。Rime 系列（Squirrel、Weasel、Trime、Hamster、fcitx5-android）是 C++ 核心；青簡（Qingjian）是 Rust 核心加各平台 Rust 殼（macOS 用 `objc2-input-method-kit`、Windows 用 `windows` crate 寫 TSF DLL 加獨立 server、Android 用 `jni`），沒有 iOS 殼。
7. **仍未查證的關鍵項目**需要實機 prototype：iOS 擴充的確切記憶體上限、Foundation Models / Core ML 在鍵盤擴充內是否可用、Android IME 內 Gemini Nano 是否被 `BACKGROUND_USE_BLOCKED`、macOS 15.4 Pasteboard privacy 對 IMK 輸入法是否生效、Mac App Store 能否上架 IMK 輸入法。

---

## 2. 對照表

| 平台 | 上下文 | 密碼欄 | 剪貼簿 | 網路 | 記憶體 | 殼語言 |
|---|---|---|---|---|---|---|
| iOS | `textDocumentProxy.documentContextBeforeInput` / `AfterInput`；上限與何時為 nil 官方未寫（未查證） | 系統強制換成系統鍵盤；phone pad 同 | 需 Full Access 才能用 `UIPasteboard`；iOS 14+ 程式化讀取會有系統通知 / iOS 16 提示 | 預設無網路；需 `RequestsOpenAccess` 加使用者開 Full Access | 有上限，官方不公布數字，隨機型變；社群實測 48 MB、77 MB（版本不明） | Swift / ObjC；核心經 C ABI 或 framework 嵌入 |
| Android | `InputConnection.getTextBeforeCursor` / `getTextAfterCursor` / `getSurroundingText`（API 31）；宿主可少回，IPC，可能 null | `TYPE_TEXT_VARIATION_PASSWORD` 等旗標；仍由當前 IME 處理，IME 須自己不顯示不儲存 | 預設 IME 是官方例外，可讀 | 一般 App 模型，可申請 `INTERNET` | 未查到官方硬上限 | Kotlin / Java；NDK + JNI 接 C / C++ / Rust |
| macOS | `IMKTextInput`：`selectedRange` + `attributedSubstring`；可能 `nil` / `NSNotFound` | 無專用 API；用 Secure Event Input 判斷 | 15.4+ Pasteboard privacy；對 IMK 是否適用未查證 | 非沙箱即可；Squirrel 有 `network.client` | 未查到官方上限 | Swift / ObjC；C ABI 嵌 C++ / Rust 核心 |
| Windows | `ITfContext` + `ITfRange`，經 edit session | 官方：密碼欄應停用 text services；`IS_PASSWORD` 不支援 | 未查證 | 桌面無限制；app container 不保證 | 無硬上限說法；DLL 在每個宿主內 | C++ COM（官方範例）；Rust `windows` crate 有實例；C# 未查證 |
| Linux | Fcitx5 `surroundingText()`；IBus `ibus_engine_get_surrounding_text`；Wayland text-input-v3 上限 4000 bytes | Wayland `content_purpose=password` + `hidden_text`；Fcitx5 `Password` / `Sensitive` | 輸入法與一般 client 一樣；Wayland 靠 data-control 協定，GNOME 沒有 | 無沙箱；Flatpak 另有規則 | 無限制（未查證到聲明） | Fcitx5 addon 為 C++；IBus engine 為任意 GObject Introspection 語言 |

---

## 3. iOS 鍵盤擴充

### 3.1 註冊、安裝、程序模型

- **官方**：在 Xcode 加 Custom Keyboard Extension target，擴充隨 App 的 bundle 發行。使用者要到 設定 > 一般 > 鍵盤 > 鍵盤 > 新增鍵盤 手動加入。擴充跑在**自己的程序**，與宿主 App 隔離。
  <https://developer.apple.com/documentation/uikit/creating-a-custom-keyboard>
- **官方**：`Info.plist` 的 `NSExtension` > `NSExtensionAttributes` 有 `IsASCIICapable`、`PrefersRightToLeft`、`PrimaryLanguage`、`RequestsOpenAccess`。`NSExtensionPointIdentifier` 為 `com.apple.keyboard-service`。多語言鍵盤要各自一個 target。
  <https://developer.apple.com/documentation/uikit/creating-a-custom-keyboard> ；<https://developer.apple.com/library/archive/documentation/General/Conceptual/ExtensibilityPG/CustomKeyboard.html>
- **官方**：包含鍵盤的 App 不能只是空殼。「If your containing app's sole purpose is to deliver your custom keyboard, there must be some utility provided even if it's purely informational.」
  <https://developer.apple.com/documentation/uikit/configuring-a-custom-keyboard-interface>
- **官方**：擴充程序在鍵盤收起後**不一定**終止，不要假設記憶體會被系統釋放。
  <https://developer.apple.com/documentation/uikit/creating-a-custom-keyboard>

### 3.2 讀輸入框上下文

- **官方**：鍵盤沒有直接存取文字輸入框，只有 `UITextDocumentProxy`。透過它可插入、刪除、移動插入點、判斷是否有文字（`hasText`），以及讀「insertion point 周圍的 textual context」（`documentContextBeforeInput` / `documentContextAfterInput`）。另有 `selectedText`、`documentIdentifier`、`documentInputMode`、`setMarkedText`（多段輸入，如中日文）。
  <https://developer.apple.com/documentation/uikit/handling-text-interactions-in-custom-keyboards> ；<https://developer.apple.com/documentation/uikit/uitextdocumentproxy>
- **官方**：文件稱 context 是用來做 autocomplete / autocapitalization 的，並沒有寫能讀多少字。
- **未查證**：`documentContextBeforeInput` 的實際長度上限（常見說法是只回當前段落附近）、何時為 `nil`。需要實機測。
- **官方**：文字或選取在鍵盤作用期間被使用者改變時，`UIInputViewController`（遵守 `UITextInputDelegate`）會收到 `textWillChange` / `textDidChange` / `selectionWillChange` / `selectionDidChange`。
  <https://developer.apple.com/documentation/uikit/handling-text-interactions-in-custom-keyboards>

### 3.3 欄位類型

- **官方**：用 `UITextInputTraits`（透過 `textDocumentProxy`）判斷 `keyboardType`（email、URL、number 等），以及 autocapitalization、autocorrection、return key 等。
  <https://developer.apple.com/documentation/uikit/configuring-a-custom-keyboard-interface>
- **官方**：「Secure text field entries always show the system keyboard when the user begins entering text in a secure text field, temporarily removing your custom keyboard if it's active. The system shows your keyboard again when the user begins entering text into a non-secure text field.」
  <https://developer.apple.com/documentation/uikit/configuring-a-custom-keyboard-interface>
- **官方**：phone pad 欄位（`UIKeyboardTypePhonePad`、`UIKeyboardTypeNamePhonePad`，如聯絡人電話欄）也強制換成系統鍵盤（App Extension Programming Guide，2017 版歸檔文件；現行頁面也列有「某些 keyboard type 顯示系統鍵盤」但型別名未在擷取文字中出現）。
  <https://developer.apple.com/library/archive/documentation/General/Conceptual/ExtensibilityPG/CustomKeyboard.html>
- **官方**：App 可以實作 `application(_:shouldAllowExtensionPointIdentifier:)` 對 `.keyboard` 回傳 `false`，**整個拒絕第三方鍵盤**（歸檔文件舉例：銀行 App、HIPAA 合規 App）。
  <https://developer.apple.com/documentation/uikit/uiapplicationdelegate/application(_:shouldallowextensionpointidentifier:)>

### 3.4 剪貼簿

- **官方**：`RequestsOpenAccess=true` 並獲使用者允許後，鍵盤可使用 `UIPasteboard`（2017 歸檔文件列出此能力）。
  <https://developer.apple.com/library/archive/documentation/General/Conceptual/ExtensibilityPG/CustomKeyboard.html>
- **官方**：iOS 14 起，App 在沒有「user intent」（點系統控制項、Command-V）下取得跨 App 的 general pasteboard 內容，系統會通知使用者。可用 `detectPatterns` 檢查內容型別而不觸發通知。
  <https://developer.apple.com/documentation/uikit/uipasteboard>
- **未查證**：iOS 16.x 的貼上提示（alert）是否也套用到 Full Access 鍵盤擴充。社群報導指一般 App 在 iOS 16.0 每次提示、16.1 起可在設定改成允許（**社群實測**，針對 App 而非鍵盤）：<https://sarunw.com/posts/uipasteboard-privacy-change-ios16>。鍵盤擴充是否同樣處理，Apple 文件沒寫。
- 沒有 Full Access 時，歸檔文件未把剪貼簿列入能力，現行文件只列「不能做的事」包含檔案系統與麥克風。把「無 Full Access 即不碰剪貼簿」當保守假設。

### 3.5 網路與 Full Access

**沒有 Full Access（`RequestsOpenAccess=false`，或使用者未允許）**：

- **官方**：能做一般鍵盤的所有事；可用 `UILexicon`（通訊錄姓名、文字替換、常用字）；可存取文字替換清單；**不能**存取檔案系統（除自己的 sandbox container），對 containing app 的共享容器**唯讀**；沒有麥克風與喇叭；不能參與 iCloud、Game Center、In-App Purchase；**沒有網路**。
  <https://developer.apple.com/documentation/uikit/configuring-open-access-for-a-custom-keyboard> ；<https://developer.apple.com/library/archive/documentation/General/Conceptual/ExtensibilityPG/CustomKeyboard.html>

**有 Full Access**：

- **官方**：以上全部，加上：定位服務與通訊錄（需使用者授權）、與 containing app 共享**可寫**容器（App Group）、傳送按鍵與輸入事件到伺服器、用 iCloud 同步設定與詞庫、經 containing app 用 Game Center 與 In-App Purchase、支援 MDM 管理的 App。使用者開啟時知道「their keystrokes are available to the keyboard developer」。
  <https://developer.apple.com/documentation/uikit/configuring-open-access-for-a-custom-keyboard>
- **官方**：Apple 的建議是「Use a non-networked keyboard if your goal is to provide normal keyboard functionality」，因為受限 sandbox 讓你更容易符合隱私規範並取得使用者信任。若傳送按鍵到伺服器：「don't store received keystroke or voice data beyond the time needed to provide text back to the user or to provide features that you explain to the user.」
  <https://developer.apple.com/documentation/uikit/configuring-open-access-for-a-custom-keyboard>
- **觸覺回饋**：本研究**未查證** Full Access 對 haptic（`UIFeedbackGenerator`）的要求。官方歸檔文件把「播放音訊，包含 `playInputClick`」列入 Full Access 能力。
- **社群實測**（原始碼）：Hamster（iOS Rime 鍵盤）的 `Info.plist` 設 `RequestsOpenAccess=true`，並使用 App Group `group.dev.fuxiao.app.Hamster`，用來和主 App 共享輸入方案與詞庫。<https://github.com/imfuxiao/Hamster>（本地 clone commit `6569370`）

**App Review Guidelines 原文**（4.4 與 4.4.1）：

> 4.4 Extensions: Apps hosting or containing extensions must comply with the App Extension Programming Guide, the Safari app extensions documentation, or the Safari web extensions documentation and should include some functionality, such as help screens and settings interfaces where possible. You should clearly and accurately disclose what extensions are made available in the app's marketing text, and the extensions may not include marketing, advertising, or in-app purchases.
>
> 4.4.1 Keyboard extensions have some additional rules.
> They must:
> - Provide keyboard input functionality (such as typed characters);
> - Follow Sticker guidelines if the keyboard includes images or emoji;
> - Provide a method for progressing to the next keyboard;
> - Remain functional without full network access and without requiring full access;
> - Collect user activity only to enhance the functionality of the user's keyboard extension on the iOS device.
>
> They must not:
> - Launch other apps besides Settings; or
> - Repurpose keyboard buttons for other behaviors (such as holding down the "return" key to launch the camera).

來源：<https://developer.apple.com/app-store/review/guidelines/>（本研究擷取時 4.4 / 4.4.1 全文；5.1.x 的擷取範圍內**未見**鍵盤專條，未讀完整頁尾段，標未查證）。

含意：核心功能不能依賴網路或 Full Access；「按鍵資料只能用來增強鍵盤本身在裝置上的功能」。

### 3.6 記憶體與效能

- **官方**：擴充程序有記憶體上限，超過會被系統終止；「The memory limits vary from model to model」；處理 low memory notification；關閉鍵盤不代表程序終止。**沒有公布數字**。
  <https://developer.apple.com/documentation/uikit/creating-a-custom-keyboard>
- **社群實測**（數字都不是 Apple 官方，版本不明，請當量級參考）：
  - 48 MB：React Native GitHub issue 作者稱 iOS 只允許鍵盤 48 MB（RN 0.63.2，約 2021 年，iOS 版本未載明）。<https://github.com/facebook/react-native/issues/31910>
  - 約 77 MB：Medium 文章稱鍵盤到 77 MB 總記憶體會當掉，並在測試鍵盤配置到 70% 時觸發 memory warning（作者以 EmojiTest 實測，裝置與 iOS 版本未載明）。<https://medium.com/@mohasalah/from-2014-to-now-fixing-coretext-emoji-glyphs-memory-retention-on-ios-1c6a227d592d>
  - 30 MB：2014 年部落格稱「up to 30MB, but it's not exactly mentioned in documentation」。<https://medium.com/@inFullMobile/limitations-of-custom-ios-keyboards-3be88dfb694>
  - 這些數字互不一致，也不保證適用於 iOS 26。**結論：要在目標最低機型上實測**。`os_proc_available_memory()` 可在執行期取得剩餘額度（官方說明為 advisory，且「if the calling process isn't an app」回傳 0，對擴充是否有效未查證）。<https://developer.apple.com/documentation/os/os_proc_available_memory>
- **Apple Foundation Models framework**：iOS 26.0+、需支援 Apple Intelligence 的裝置與地區，on-device 模型（`SystemLanguageModel`）。<https://developer.apple.com/documentation/foundationmodels/systemlanguagemodel> 。**鍵盤擴充是否可用：未查證**（官方頁面沒有關於 app extension 的說明）。
- **Core ML 在鍵盤擴充內**：未查證。一般 app extension 可用 Core ML，但受上述記憶體上限；需實測模型載入後的 footprint。
- 鍵盤擴充的 jetsam 優先順序屬「UI Support」（**社群實測**，jailbreak 工具 jetsamctl 文件）：<https://github.com/conradev/jetsamctl>。

### 3.7 UI 自由度

- **官方**：寬度由系統決定（通常等於螢幕寬）；高度可用 Auto Layout 調整 `UIInputViewController.view` 的 height constraint。必須支援 compact / regular 寬度與橫豎向；iPadOS 還要支援浮動視窗。
  <https://developer.apple.com/documentation/uikit/configuring-a-custom-keyboard-interface>
- **官方**（歸檔）：鍵盤只能畫在自己的 primary view 內，所以**不能選取文字**（選取由宿主控制）、沒有宿主的編輯選單存取、**不能在插入點附近做 inline 自動修正 UI**、**不能顯示超出鍵盤上緣的按鍵圖示**（系統鍵盤在 iPhone 長按首排鍵會超出，第三方不行）。
  <https://developer.apple.com/library/archive/documentation/General/Conceptual/ExtensibilityPG/CustomKeyboard.html>
- **官方**（歸檔，2017）：「Custom keyboards, like all app extensions in iOS 8.0, have no access to the device microphone, so dictation input is not possible.」現行文件改為：鍵盤若自帶聽寫能力，可把 `UIInputViewController` 的 `hasDictationKey` 設為 `true`，避免系統再顯示聽寫按鈕。
  <https://developer.apple.com/documentation/uikit/configuring-a-custom-keyboard-interface>
- **官方**：必須提供切換鍵盤的按鈕（地球鍵）：`needsInputModeSwitchKey` 為 `true` 時要自己顯示；Face ID iPhone 系統會在鍵盤下方自動顯示地球鍵，此屬性為 `false`。按鈕動作接到 `handleInputModeList(from:with:)` 讓長按能出系統鍵盤清單。
  <https://developer.apple.com/documentation/uikit/creating-a-custom-keyboard> ；<https://developer.apple.com/documentation/uikit/uiinputviewcontroller/needsinputmodeswitchkey>
- **社群實測**：鍵盤背景不能透明（2014 部落格）。<https://medium.com/@inFullMobile/limitations-of-custom-ios-keyboards-3be88dfb694>
- 長按 popup、滑動手勢、候選列：都在自己的 view 內自繪，沒有系統 API；popup 受上述「不能超出鍵盤上緣」限制，常見做法是頂部留一列候選列作緩衝（2014 部落格提到的解法，**社群實測**）。

### 3.8 語言與 SDK

- **官方**：擴充用 Xcode 的 Custom Keyboard Extension 模板建立，主要語言 Swift 或 Objective-C。
- **社群實測**：Hamster 以 `librime`（C++）編成 framework 嵌進 App 與鍵盤擴充，`LibrimeKit` 專案負責用 GitHub Action 建置並發行 framework。<https://github.com/imfuxiao/Hamster> 。
- 嵌入 Rust / C++ 核心：標準做法是編成 static library / xcframework，透過 C ABI（或 uniffi 產生的 Swift binding）呼叫。本研究未逐項查證 uniffi 的 iOS 擴充相容性，列為待驗證。

### 3.9 同步與背景

- **官方**：Full Access 才能用 iCloud 同步設定與詞庫、才能寫 App Group 共享容器。沒有 Full Access 時共享容器對鍵盤唯讀，也就是 containing app 可以寫、鍵盤可以讀（單向）。
  <https://developer.apple.com/documentation/uikit/configuring-open-access-for-a-custom-keyboard>
- **未查證**：鍵盤擴充的背景執行能力；一般 app extension 沒有長時間背景執行，這點未找到鍵盤專屬的官方說明。

---

## 4. Android InputMethodService

### 4.1 註冊、安裝、程序模型

- **官方**：IME 是含 IME service 的一般 App。Manifest 宣告 service，`android:permission="android.permission.BIND_INPUT_METHOD"`，intent-filter action `android.view.InputMethod`，`<meta-data android:name="android.view.im" android:resource="@xml/method"/>`。可選設定 Activity 與 `<subtype>`。
  <https://developer.android.com/develop/ui/views/touch-and-input/creating-input-method>
- **官方**：使用者要在 `Settings.ACTION_INPUT_METHOD_SETTINGS` 頁啟用、再選為當前輸入法。
  <https://developer.android.com/reference/android/provider/Settings>
- **官方**：IME 與宿主不同程序，經 `InputConnection`（IPC）溝通；文件寫明 `getTextBeforeCursor` 會觸發 IPC round-trip，連線可能因對方程序 crash 失效。
  <https://developer.android.com/reference/android/view/inputmethod/InputConnection>
- **官方**：隱藏輸入視窗後應立即釋放大塊記憶體。
  <https://developer.android.com/develop/ui/views/touch-and-input/creating-input-method>

### 4.2 讀輸入框上下文

- **官方**：`getTextBeforeCursor(n, flags)`、`getTextAfterCursor`、`getSelectedText`、`getSurroundingText(before, after, flags)`（API 31）。回傳 `null` 的情況：連線失效、編輯器太久沒回應（「given a couple seconds to return」）、`getSurroundingText` 另包含協定不支援。編輯器可以為了效能少回字元，但文件要求不應限制在當前行。每次呼叫是 IPC，建議用 `EditorInfo.getInitialTextBeforeCursor` 等取得初始上下文。
  <https://developer.android.com/reference/android/view/inputmethod/InputConnection>
- **官方**：`onUpdateSelection(...)` 在宿主回報選取變動時呼叫。`InputType.TYPE_NULL` 表示目標連線不是 rich，不能取得目前文字，IME 只能送 key events。
  <https://developer.android.com/reference/android/inputmethodservice/InputMethodService> ；<https://developer.android.com/reference/android/text/InputType>
- **未查證**：WebView、遊戲等具體何時讀不到的官方清單。

### 4.3 欄位類型

- **官方**：`EditorInfo.inputType` 的 `TYPE_TEXT_VARIATION_PASSWORD`、`VISIBLE_PASSWORD`、`WEB_PASSWORD`、`TYPE_NUMBER_VARIATION_PASSWORD`、`TYPE_TEXT_VARIATION_URI`、`EMAIL_ADDRESS` 等。
  <https://developer.android.com/reference/android/text/InputType>
- **官方**：IME 指南對密碼欄的要求：在輸入視圖與候選視圖都要隱藏密碼，且「Don't store passwords on a device」。**文件沒有說系統會強制換成系統 IME**，密碼欄仍交給當前 IME（強制切換「不存在」只是文件未載明，嚴格說是未查證）。
  <https://developer.android.com/develop/ui/views/touch-and-input/creating-input-method>
- **官方**：`IME_FLAG_NO_PERSONALIZED_LEARNING`：宿主要求 IME 不更新個人化資料（無痕模式等），是請求而非強制。`IME_FLAG_FORCE_ASCII`、`IME_FLAG_NO_FULLSCREEN`、`IME_FLAG_NO_EXTRACT_UI` 同為宿主對 IME 的請求。
  <https://developer.android.com/reference/android/view/inputmethod/EditorInfo>

### 4.4 剪貼簿

- **官方**：Android 10 起「Unless your app is the default input method editor (IME) or is the app that currently has focus, your app cannot access clipboard data on Android 10 or higher.」`ClipboardManager.getPrimaryClip()` 對非預設 IME 且無焦點者回 `null`。
  <https://developer.android.com/about/versions/10/privacy/changes> ；<https://developer.android.com/reference/android/content/ClipboardManager>
- **官方**：Android 12+ 讀取他人 App 的剪貼簿，系統通常顯示 toast「APP pasted from your clipboard」。預設 IME 讀取是否也顯示：未查證。
  <https://developer.android.com/develop/ui/views/touch-and-input/copy-paste>
- **官方**：Android 13+ 可在 `ClipDescription` 加 `EXTRA_IS_SENSITIVE` 以免敏感內容出現在系統複製預覽（只是渲染提示，不影響存取）。
  <https://developer.android.com/reference/android/content/ClipDescription>

### 4.5 網路、Play 政策

- IME 是一般 App，可宣告 `INTERNET`。官方沒有專條說明，由一般 App 模型推得（未查證專條）。
- **未查證**：Google Play 政策中針對 Input Method / 鍵盤的專條（User Data policy 與 Permissions policy 全文搜尋 keyboard / input method 都沒有命中）。一般義務（隱私政策、prominent disclosure）仍適用。
  <https://support.google.com/googleplay/android-developer/answer/10144311>

### 4.6 記憶體與效能、AI

- **未查證**：沒有查到類似 iOS 的官方硬性記憶體上限。官方只有「釋放大塊記憶體」與一般 low memory killer 說明。
  <https://developer.android.com/develop/ui/views/touch-and-input/creating-input-method>
- **官方**：ML Kit GenAI APIs 建於 AICore，用 Gemini Nano 做 on-device 推論。原文：「GenAI API inference is permitted only when the app is the top foreground application. Using the API when the app is not in the foreground, including using a foreground service, will result in an ErrorCode.BACKGROUND_USE_BLOCKED response.」另有 per-app 配額（`BUSY`、`PER_APP_BATTERY_USE_QUOTA_EXCEEDED`）。
  <https://developers.google.com/ml-kit/genai>
- **未查證**：IME 程序算不算「top foreground application」。官方頁面沒提 IME，需實機 prototype（在 IME service 內呼叫 GenAI API 看是否回 `BACKGROUND_USE_BLOCKED`）。支援機型清單未整理。

### 4.7 UI 自由度

- **官方**：`onCreateInputView()` 建輸入區、`onCreateCandidatesView()` 建候選視圖、`setCandidatesViewShown`、`onCreateExtractTextView()`（fullscreen 模式）、`onEvaluateFullscreenMode()`（預設只在橫向全螢幕）、`onComputeInsets(Insets)` 控制可見與可觸控區（影響鍵盤高度與宿主佈局）。
  <https://developer.android.com/reference/android/inputmethodservice/InputMethodService>
- **未查證**：IME 視窗內 `PopupWindow` 的限制、手勢細節（未逐項查官方）。這些是自繪 view，一般認為自由度高，但需求清單要自己驗證。

### 4.8 語言與 SDK

- Kotlin / Java；NDK 為官方支援的 native 路徑。
- **社群實測**：Trime 為「Java/Kotlin with JNI」使用 librime（<https://github.com/osfans/trime>）；fcitx5-android 把 C++ fcitx5 核心經 JNI 放進 App，app 側為 Kotlin（<https://github.com/fcitx5-android/fcitx5-android>）；青簡的 Android 核心 crate 為 `cdylib` + `jni = "0.21"`（<https://github.com/qingjian-team/qingjian>，commit `3d5fd2d`）。

### 4.9 背景與同步

- **官方**：Android 12+ 限制背景啟動前景服務，豁免清單包含「Your app is the device's current input method」。
  <https://developer.android.com/develop/background-work/services/fgs/restrictions-bg-start>
- 其他同步限制：沒有 IME 專屬限制的官方條文（一般 App 規則適用，WorkManager 等）。

---

## 5. macOS InputMethodKit（IMK）

### 5.1 註冊、安裝、程序模型

- **官方**：輸入法是獨立 `.app`，在 `main` 建立 `IMKServer`，由它為每個 client 的 input session 建一個 `IMKInputController`；controller 只收到單一 session 的通訊。
  <https://developer.apple.com/documentation/inputmethodkit/imkserver> ；<https://developer.apple.com/documentation/inputmethodkit/imkinputcontroller>
- **社群實測**：Squirrel 的 `Info.plist` 用到 `InputMethodConnectionName`、`InputMethodServerControllerClass`、`ComponentInputModeDict`（含 `TISInputSourceID`、`tsInputModeScriptKey` 等）、`LSUIElement=true`。Apple 線上文件沒有完整 key 清單（未查證）。
  <https://github.com/rime/squirrel/blob/46962d2efae5be70389c778d0a6c52f669072827/resources/Info.plist>
- **社群實測**：安裝位置 `/Library/Input Methods`，Squirrel 的 pkg 要求登出再登入；fcitx5-macos 也要登出登入後到 系統設定 > 鍵盤 > 輸入來源 新增。`~/Library/Input Methods` 是否被系統接受：未查證（青簡 README 稱開發安裝用此路徑，**社群實測**）。
  <https://github.com/rime/squirrel/blob/46962d2efae5be70389c778d0a6c52f669072827/INSTALL.md> ；<https://github.com/fcitx-contrib/fcitx5-macos>
- 程序模型：輸入法與宿主跨程序（`IMKServer` 以 `NSConnection` 讓宿主連線，標頭用詞）。Rime / Fcitx5 的做法是「引擎與 `IMKServer` 在同一程序」。IPC 細節 Apple 文件未寫（未查證）。

### 5.2 讀輸入框上下文

- **官方**（SDK 標頭 `IMKInputSession.h`）：`selectedRange`、`attributedSubstring(from:)`、`length` 都依賴宿主支援 TSMDocumentAccess。標頭原文：「Many applications do not support these so input methods should be prepared for these methods to return nil or NSNotFound.」不支援時 `selectedRange` 與 `length` 回 `NSNotFound`。
  標頭路徑：`MacOSX.sdk/System/Library/Frameworks/Carbon.framework/Versions/A/Frameworks/HIToolbox.framework/Versions/A/Headers/IMKInputSession.h`。Apple 線上文件沒有 `IMKTextInput` 獨立頁。
- **官方**：`firstRect(forCharacterRange:actualRange:)` 回 global 座標的第一個矩形；`attributes(forCharacterIndex:lineHeightRectangle:)` 供候選窗定位；`bundleIdentifier` 回傳宿主 bundle id；`windowLevel` 供自繪候選窗用。
- **社群實測**：fcitx5-macos 原始碼註解指出 VSCode terminal、Terminal、iTerm、JetBrains、Chrome 網址列等需要 dummy preedit，否則按鍵處理異常；SwiftUI TextField 的 `length` 與 `selectedRange` 可靠。
  <https://github.com/fcitx-contrib/fcitx5-macos/blob/3466612499c149d5002b1aab37b92bb257da1a75/macosfrontend/macosfrontend.swift>
- 結論：沒有「游標前 N 字」單一 API，每一步都可能是 `NSNotFound`；核心必須允許「無上下文」。

### 5.3 欄位類型

- **官方**：Secure Event Input：`EnableSecureEventInput` / `IsSecureEventInputEnabled`；`NSSecureTextField` 等會自動保護。TN2150 正文沒有提到輸入法。
  <https://developer.apple.com/library/archive/technotes/tn2150/_index.html>
- **未查證**：輸入法在密碼欄是否被系統強制換成 ASCII 輸入源的官方原文（TN2150、`NSSecureTextField`、`NSTextInputContext` 頁都沒有）。
- **社群實測**：fcitx5-macos 用 `IsSecureEventInputEnabled()` 判斷，但有 App 啟用後不關閉（Apple Discussions 回報），所以再用 IORegistry 的 `kCGSSessionSecureInputPID` 比對持有者是否為目前宿主，並硬編碼幾個「只有密碼欄卻不呼叫 `EnableSecureEventInput`」的系統 App（loginwindow 等）。註解稱 PID 資訊「not guaranteed accurate」。
  <https://github.com/fcitx-contrib/fcitx5-macos/blob/3466612499c149d5002b1aab37b92bb257da1a75/src/secure.swift>
- email / URL / number 欄位類型提示：`IMKTextInput` 與 `IMKInputController` 標頭都沒有（未查到其他 API）。

### 5.4 剪貼簿

- **官方**：macOS 15.4+ 的 `NSPasteboard.AccessBehavior`：general pasteboard 預設在「程式化存取」時詢問，使用者的 paste 動作一律允許；觸發後 App 列入 系統設定 > 隱私與安全性 > Paste from Other Apps，可設 ask / alwaysAllow / alwaysDeny。`detectPatterns` / `detectValues` / `detectMetadata` 可不通知使用者地檢查型別。
  <https://developer.apple.com/documentation/appkit/nspasteboard/accessbehavior-swift.enum>
- **未查證**：此機制是否對 IMK 輸入法程序（背景 App）套用、有無豁免。Apple 文件以「app」為單位，沒提 IMK。需實機驗證。
- **社群實測**：fcitx5-macos 的剪貼簿 addon 輪詢 `changeCount`，用 `org.nspasteboard.ConcealedType` 判斷是否為密碼。
  <https://github.com/fcitx-contrib/fcitx5-macos/blob/3466612499c149d5002b1aab37b92bb257da1a75/macosfrontend/pasteboard.mm>

### 5.5 網路與沙箱

- **社群實測**：Squirrel entitlements：`com.apple.security.app-sandbox=false`、`com.apple.security.network.client=true`、`com.apple.security.cs.disable-library-validation=true`（載入 librime plugin 用）；fcitx5-macos 的 entitlements 只有 `get-task-allow`（無 sandbox）。
  <https://github.com/rime/squirrel/blob/46962d2efae5be70389c778d0a6c52f669072827/resources/Squirrel.entitlements>
- **未查證**：Mac App Store 能否上架 IMK 輸入法；沙箱下註冊 `IMKServer` 連線名所需的 entitlement。

### 5.6 記憶體、AI

- **未查證**：IMK 輸入法程序的記憶體硬上限（Apple 文件未提）。
- **官方**：Foundation Models framework 在 macOS 26.0+，需 Apple Intelligence 裝置。<https://developer.apple.com/documentation/foundationmodels> 。**在 IMK 輸入法程序內呼叫的限制：未查證**。

### 5.7 UI 自由度

- **官方**：`IMKCandidates` 提供系統候選窗（單欄捲動、多欄格狀、單列步進三種 panel）；也可自繪（以 `windowLevel` 加一定位在宿主視窗上）。事件接收有三種模式：key binding、text data only、`handleEvent`（直接給 `NSEvent`）；`recognizedEvents(_:)` 回傳要接收的事件 mask。
  <https://developer.apple.com/documentation/inputmethodkit/imkcandidates> ；<https://developer.apple.com/documentation/inputmethodkit/imkserverinput> ；<https://developer.apple.com/documentation/inputmethodkit/imkstatesetting>
- **社群實測**：Squirrel 自繪 `NSPanel` 候選窗，`recognizedEvents` 回 `.keyDown | .flagsChanged`。
- 系統保留快捷鍵（Ctrl+Space 等）的官方說明：未查證。

### 5.8 語言與 SDK

- Objective-C API，Swift 可直接用。
- **社群實測**：Squirrel 為 Swift 加 librime（C++，經 C API）同程序；fcitx5-macos 為 C++20 + Swift 6，fcitx5 核心以 addon 形式在同程序，Swift 與 C++ 以 module map 互通。
- **社群實測**：青簡的 macOS 殼完全用 Rust，經 `objc2` / `objc2-input-method-kit` / `objc2-app-kit` crate 直接呼叫 IMK，並附 `bundle.sh` 做打包與 pkg。<https://github.com/qingjian-team/qingjian/tree/main/apps/macos>（commit `3d5fd2d`）。可行性有例可循，但屬單一專案經驗。

### 5.9 簽名、公證、分發

- **官方**：以 Developer ID 發行的軟體需公證（`notarytool`）；經 Mac App Store 發行不需另外公證。
  <https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution>
- **社群實測**：Squirrel 發行流程為 `codesign --options runtime --timestamp`、`productsign`、`notarytool submit`，pkg 裝 `/Library/Input Methods`。
- **未查證**：macOS 15 以後對輸入法的額外安裝要求。

---

## 6. Windows TSF

### 6.1 註冊、安裝、程序模型

- **官方**：除標準 COM in-proc server 登錄外，text service 要向 TSF 註冊：`ITfInputProcessorProfiles::Register`、`AddLanguageProfile`，再用 `ITfCategoryMgr::RegisterCategory` 註冊類別（如 `GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER`）。Vista+ 可用 `ITfInputProcessorProfileMgr::RegisterProfile`。建議二進位數位簽章。
  <https://learn.microsoft.com/en-us/windows/win32/tsf/text-service-registration> ；<https://learn.microsoft.com/en-us/windows/win32/api/msctf/nn-msctf-itfinputprocessorprofilemgr>
- **官方**：Windows 8+ 的第三方 IME 須：數位簽章、TSF-aware 並設對旗標；舊式第三方 IME 能在桌面 App 跑，但在 Windows Store apps 被阻擋。要與 immersive 環境相容需註冊 `GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT`；UILess 支援需 `GUID_TFCAT_TIPCAP_UIELEMENTENABLED`。
  <https://learn.microsoft.com/en-us/windows/win32/w8cookbook/third-party-input-method-editors> ；<https://learn.microsoft.com/en-us/windows/apps/develop/input/input-method-editor-requirements>
- **官方**：IME 在各宿主程序內執行（「runs in the context of each application's process」），所以不應宣告自己的 DPI awareness。IME 必須能在 app container 內運作：辭典檔放 Program Files 或 Windows 目錄（app container 預設可讀），放別處須自行改 ACL；app container 內網路不保證，要連網更新辭典應另跑桌面程序。
  <https://learn.microsoft.com/en-us/windows/apps/develop/input/input-method-editor-requirements>
- **社群實測**：DLL 被載入到每個前景 TSF 程序；32 位元宿主需另編 32 位元版並用 `SysWOW64\regsvr32.exe` 註冊（khiin-rs README）；PIME 安裝程式註冊 x86 與 x64 / ARM64 兩套。
  <https://github.com/OMAMA-Taioan/khiin-rs/blob/master/windows/README.md> ；<https://github.com/EasyIME/PIME>
- **社群實測**：純 MSIX 部署在 Microsoft Q&A 有人遇到 `RegisterProfile` 失敗（0x80004005），建議用簽章的 MSI / EXE 安裝程式。
  <https://learn.microsoft.com/en-us/answers/questions/5995922/native-tsf-ime-pure-msix-deployment>

### 6.2 讀輸入框上下文

- **官方**：讀寫文字須經 edit session（`ITfContext::RequestEditSession`），用 `ITfRange`。`ITfContext::GetStatus` 的 `TS_STATUS` 有 `TS_SD_READONLY`、`TS_SD_LOADING`、`TS_SS_TRANSITORY`（短暫使用週期的 context）、`TS_SS_NOHIDDENTEXT` 等；回 `TF_E_DISCONNECTED` 表示 context 不在 document stack。
  <https://learn.microsoft.com/en-us/windows/win32/api/textstor/ns-textstor-ts_status> ；<https://learn.microsoft.com/en-us/windows/win32/api/msctf/nf-msctf-itfcontext-getstatus>
- **官方**：`ITfContextView::GetTextExt` 回範圍的螢幕座標；文字不可見或視窗最小化時回 `S_OK` 且矩形為全 0；`TS_E_NOLAYOUT` 表示尚未排版。
  <https://learn.microsoft.com/en-us/windows/win32/api/msctf/nf-msctf-itfcontextview-gettextext>
- **未查證**：`ITfRange::GetText` 的長度上限、console / 遊戲何時讀不到的官方說明。官方只有：不想讓 TIP 畫 UI 的 App 可用 UILess 模式。

### 6.3 欄位類型、密碼欄

- **官方**（`IS_PASSWORD` 說明原文）：「IS_PASSWORD is not supported and may be altered or unavailable in the future… IS_PASSWORD only indicates the password; it doesn't provide any security around the password. All passwords fields should have text services disabled to maintain password secrecy, and therefore it is not valid to have a password field with an IS_PASSWORD input scope.」也就是官方做法是密碼欄直接停用 text services，不要靠 `IS_PASSWORD` 偵測。
  <https://learn.microsoft.com/en-us/windows/win32/api/inputscope/ne-inputscope-inputscope>
- 其他 `InputScope` 值（email、URL 等）存在於 `InputScope` 列舉；本研究未逐項整理 TSF 讀取 `InputScope` 的流程（`ITfInputScope`）。
- **未查證**：`GUID_COMPARTMENT_KEYBOARD_INPUTMODE_*`、`TSATTRID_Text_ReadOnly` 偵測密碼欄的官方文件；Windows 密碼欄是否一律強制切英文。

### 6.4 剪貼簿

- **未查證**：Windows 對 TSF 輸入法讀寫剪貼簿的專屬限制。剪貼簿是一般桌面 API；app container 內的存取限制需另查。

### 6.5 網路

- **官方**：app container 內網路「isn't always permitted」；若該容器有 Internet 存取，對 IME 可連的 endpoint 沒有限制，可用雲端伺服器做 on-the-fly learning。桌面宿主對 TSF 網路無特別限制（沒有官方禁止，未查證專條）。
  <https://learn.microsoft.com/en-us/windows/apps/develop/input/input-method-editor-requirements>

### 6.6 記憶體、效能、崩潰

- **官方**：IME 在每個宿主程序內執行。官方對 DLL 在宿主內造成記憶體 / 執行緒 / 崩潰影響沒有專章（未查證），也沒有硬記憶體上限說法。
- **社群實測**：PIME、Weasel、青簡都把引擎放獨立 server 程序，TSF DLL 只做按鍵轉發與寫入文件。青簡 README 說明原因：DLL 若含引擎，會被複製進幾十個程序、狀態無法共享、崩潰會連累宿主。
  <https://github.com/qingjian-team/qingjian/blob/main/apps/windows/README.md> ；<https://github.com/EasyIME/PIME>
- 重新編譯 DLL 前須關閉所有已載入它的程序（khiin-rs README，**社群實測**）。

### 6.7 UI 自由度

- **官方**：UILess 模式：App 以 `ITfThreadMgrEx::ActivateEx(ITF_AE_UIELEMENTENABLEDONLY)` 建 UI-less 執行緒並實作 `ITfUIElementSink`；TIP 要實作 `ITfTextInputProcessorEx`，顯示 UI 前呼叫 `ITfUIElementMgr::BeginUIElement`；系統預定義 `ITfCandidateListUIElement`。適用於遊戲與全螢幕 App。
  <https://learn.microsoft.com/en-us/windows/win32/tsf/uiless-mode-overview>
- **官方**：Windows 8 immersive 環境：候選窗必須是 owned window（owner 由 `ITfContextView::GetWnd` 取得，失敗時用 `GetFocus()`）、用 `NotifyWinEvent` 發 `EVENT_OBJECT_IME_SHOW` / `HIDE` / `CHANGE`、不得全螢幕、不得互相重疊；觸控鍵盤在更高 z-order band，要用 InputPane API 避開。
  <https://learn.microsoft.com/en-us/windows/apps/develop/input/input-method-editor-requirements>
- 自繪候選窗位置：`ITfContextView::GetTextExt`（見 6.2）。

### 6.8 語言與 SDK

- **官方**：TSF 是 COM 介面（`msctf.h`），官方範例為 C++（`microsoft/Windows-classic-samples` 的 `Samples/IME/cpp/SampleIME`）。
  <https://github.com/microsoft/Windows-classic-samples/tree/master/Samples/IME/cpp>
- **社群實測**：Rust + `windows` crate 寫 TSF 的專案有 khiin-rs（README 稱 Microsoft 文件在許多地方不精確或過時，他們參考了至少六個開源 TSF IME）、windows-chewing-tsf、青簡的 `qingjian-windows-tsf`（`cdylib`，依賴官方 `windows` crate）。Weasel 為 C++。
  <https://github.com/OMAMA-Taioan/khiin-rs> ；<https://github.com/chewing/windows-chewing-tsf> ；<https://github.com/rime/weasel>
- **未查證**：C# / .NET 寫 in-proc TSF DLL 的可行性（需 CLR 載入每個宿主，未查官方說法）。

### 6.9 背景與同步

- **未查證**：無 TSF 專屬背景 / 同步限制的官方條文。官方建議網路更新交給獨立桌面程序（見 6.1）。

---

## 7. Linux：Fcitx5 與 IBus

### 7.1 Fcitx5 架構與註冊

- **官方**（原始碼）：前端（frontend）addon 有 `dbusfrontend`、`fcitx4frontend`、`ibusfrontend`、`waylandim`、`xim`、`virtualinputcontext`。GTK / Qt im module 是獨立 repo（fcitx5-gtk、fcitx5-qt），經 DBus 連 daemon。Wayland 前端支援 `zwp_input_method_v1` 與 `v2`。
  <https://github.com/fcitx/fcitx5/tree/145e981d6377550ffb39ee8de61452910622652c/src/frontend>
- **官方**：addon 是 C++ shared library：引擎繼承 `fcitx::InputMethodEngineV2`，用 `FCITX_ADDON_FACTORY` 導出；唯一提到的其他語言是 Lua（API 有限）。註冊檔為 `share/fcitx5/addon/<addon>.conf` 與 `share/fcitx5/inputmethod/<im>.conf`。
  <https://fcitx-im.org/wiki/Develop_an_simple_input_method>
- **未查證**：官方 C ABI 或 Rust 綁定。實務做法是 C++ 薄層 addon 包 C ABI 核心（fcitx5-rime、ibus-rime 把 librime 包成各自的 addon / engine，未逐項讀碼，標未查證）。

### 7.2 Surrounding text

- **官方**（Fcitx5 原始碼）：`CapabilityFlag::SurroundingText`；`InputContext::surroundingText()` 的 text、cursor、anchor 單位為字元；`deleteSurroundingText(offset, size)`。
  <https://github.com/fcitx/fcitx5/blob/145e981d6377550ffb39ee8de61452910622652c/src/lib/fcitx/inputcontext.h#L167-L195>
- **官方**（fcitx5-qt 原始碼）：環境變數 `FCITX_QT_ENABLE_SURROUNDING_TEXT`（預設開）；字串長度 `< 4096` 才送；非法 UTF-8 不送；`Password` 或 `Sensitive` 時不送。
  <https://github.com/fcitx/fcitx5-qt/blob/0285a5d18367d8f3af80dab7e4819d5555982340/qt5/platforminputcontext/qfcitxplatforminputcontext.cpp#L440-L505>
- **官方**（fcitx5-gtk 原始碼）：GTK 端 `retrieve-surrounding` / `delete-surrounding` 訊號，原始碼註解稱 `_request_surrounding_text may trigger freeze in Libreoffice`，故延遲請求。
  <https://github.com/fcitx/fcitx5-gtk/blob/0445d4f88b08e7058ec26770a65db4bf0b06988e/gtk3/fcitximcontext.cpp#L866-L886>
- **官方**（IBus 原始碼）：`IBUS_CAP_SURROUNDING_TEXT`；`ibus_engine_get_surrounding_text()` 取 engine 快取的 text/cursor/anchor 並對 client 發 `RequireSurroundingText`；client 端 `ibus_input_context_set_surrounding_text()` 只在 `needs_surrounding_text` 為真時送。
  <https://github.com/ibus/ibus/blob/d464316d9b9613005c5573a010fec7f9b40b93b6/src/ibusengine.c#L2184-L2216>
- **官方**（Wayland text-input-v3）：`set_surrounding_text(text, cursor, anchor)` 是 client 到 compositor 的 request，UTF-8，「text can not be longer than 4000 bytes」，不含 preedit，client 不知道周圍文字時「should not issue this request」，初始狀態為空代表不支援；雙緩衝，`commit` 才生效。
  <https://gitlab.freedesktop.org/wayland/wayland-protocols/-/raw/main/unstable/text-input/text-input-unstable-v3.xml>
- GNOME 以 IBus 後端：gnome-shell 的 `inputMethod.js` 把 Mutter 收到的 text-input-v3 surrounding 轉給 IBus input context。<https://gitlab.gnome.org/GNOME/gnome-shell/-/raw/main/js/misc/inputMethod.js>
- **未查證**：哪些客戶端沒有 surrounding text 的完整清單。X11 XIM 前端沒有 surrounding 路徑（原始碼 grep 推論，非官方聲明）。

### 7.3 欄位類型

- **官方**（Fcitx5 `CapabilityFlag`）：`Password`、`Email`、`Digit`、`Url`、`Number`、`Dialable`、`Name`、`Terminal`、`Sensitive`、`Multiline`、`NoSpellCheck`、`Disable` 等；全域設定 `AllowInputMethodForPassword`（預設 false）。
  <https://github.com/fcitx/fcitx5/blob/145e981d6377550ffb39ee8de61452910622652c/src/lib/fcitx-utils/capabilityflags.h#L19-L88>
- **官方**（Wayland text-input-v3）：`content_hint` 有 `hidden_text`（0x40，字元應隱藏）、`sensitive_data`（0x80，輸入內容不應被儲存）、`latin`、`multiline`、`completion`、`spellcheck` 等；`content_purpose` 有 `alpha`、`digits`、`number`、`phone`、`url`、`email`、`name`、`password`、`pin`、`date`、`time`、`datetime`、`terminal`。`password` 與 `pin` 文件註明應搭配 `sensitive_data` hint。
  <https://gitlab.freedesktop.org/wayland/wayland-protocols/-/raw/main/unstable/text-input/text-input-unstable-v3.xml>
- **官方**（IBus 原始碼）：`IBusInputPurpose`（含 `PASSWORD`、`PIN`、`EMAIL`、`URL`、`TERMINAL`）、`IBusInputHints`（含 `PRIVATE`、`HIDDEN_TEXT`、`SPELLCHECK`、`WORD_COMPLETION`）；client 以 `ibus_input_context_set_content_type` 送出。
  <https://github.com/ibus/ibus/blob/d464316d9b9613005c5573a010fec7f9b40b93b6/src/ibustypes.h#L258-L400>
- 結論：Linux 欄位類型提示比 macOS 豐富。X11 XIM 路徑無（推論）。

### 7.4 剪貼簿

- **官方**：Wayland 核心 `wl_data_device.selection`：data_offer 在 client 失去鍵盤焦點時失效，一般 client 只有鍵盤焦點時才能讀剪貼簿。
  <https://gitlab.freedesktop.org/wayland/wayland/-/raw/main/protocol/wayland.xml>
- **官方**：`ext-data-control-v1`（staging，testing phase）允許 privileged client 管理選取、當 clipboard manager。實作狀況（依 wayland.app 表的推讀，欄位意義頁面未標示）：Sway、Hyprland、KWin、COSMIC、niri 等有；GNOME Mutter、Weston 沒有。
  <https://wayland.app/protocols/ext-data-control-v1>
- Fcitx5 的 clipboard addon 有 `xcbclipboard.cpp` 與 `waylandclipboard.cpp`，後者引用 `ext_data_control_manager_v1` 與 `zwlr_data_control_manager_v1`。因此在 GNOME Wayland 沒有 data-control 協定時，Wayland 原生剪貼簿路徑不可用（推論；Fcitx 文件未直接寫，GNOME 實際行為未查證）。
  <https://github.com/fcitx/fcitx5/blob/145e981d6377550ffb39ee8de61452910622652c/src/modules/clipboard/waylandclipboard.cpp#L28-L30>
- X11 無焦點限制：未查證第一手來源。

### 7.5 網路、沙箱

- 原生桌面程序無沙箱限制（無獨立官方宣告，未查證）。
- **官方**（Flatpak）：`--socket=inherit-wayland-socket`：「Input method applications may need this. It's a very sensitive permission as it prevents Wayland client state from being sandboxed」。Fcitx5 的 ibus 前端提供 `org.freedesktop.portal.IBus` DBus 名，供沙箱 client 連入。
  <https://docs.flatpak.org/en/latest/sandbox-permissions.html> ；<https://github.com/fcitx/fcitx5/blob/145e981d6377550ffb39ee8de61452910622652c/src/frontend/ibusfrontend/ibusfrontend.cpp#L74>
- 把輸入法本身打包成 Flatpak / Snap 的限制：未查證。

### 7.6 記憶體

- 未查到官方上限聲明。

### 7.7 UI 與 Wayland 差異

- **官方**（Fcitx5 Wayland wiki，無日期，頁面自述 Chromium 資訊可能過時）：
  - text-input-v1：Chromium 原生 Wayland；text-input-v2：僅 KWin；text-input-v3：GTK 3/4 與多數 compositor（Weston 除外）。
  - Fcitx 自己的 input-method frontend：`zwp_input_method_v1`（KDE、Weston）、v2（Sway，需 1.10+）。
  - GNOME 走 IBus DBus 協定，需 ibus frontend；fcitx 啟動時會取代執行中的 ibus-daemon。
  - Wayland 沒有一般 client 的全域座標系，候選窗位置需靠 `zwp_input_popup_surface_v2` 或 im module；GNOME 需 Kimpanel 擴充才能把 popup 顯示在 gnome-shell UI 上方。
  - 在 zwp_input_method 下 fcitx 只看見單一 input context，啟用狀態是全域的。
  <https://fcitx-im.org/wiki/Using_Fcitx_5_on_Wayland>
- **官方**：`input-method-unstable-v2`（wlroots 維護，頁面警告實驗性、不保證向後相容）：每個 seat 最多一個輸入法物件；`grab_keyboard` 取得硬體鍵盤事件；`input_popup_surface` 僅在輸入法 active 時可見。IBus 自帶 Wayland client（`ibuswaylandim.c`），使用 `zwp_input_method_v1/v2`、`zwp_input_popup_surface_v2` 與 `zwp_virtual_keyboard_v1`。
  <https://wayland.app/protocols/input-method-unstable-v2> ；<https://github.com/ibus/ibus/tree/d464316d9b9613005c5573a010fec7f9b40b93b6/client/wayland>
- **官方**：GNOME 不實作 input-method-v2；Mutter 實作 text-input-v3（`meta-wayland-text-input.c`）並把事件導給 IBus。
  <https://gitlab.gnome.org/GNOME/mutter/-/raw/main/src/wayland/meta-wayland-text-input.c>
- **更正一般印象**：依 wayland.app 的協定支援表（欄位意義為推讀），**KWin 與 Mutter 都沒有實作 input-method-v2**；Fcitx 文件寫 KDE 用 `zwp_input_method_v1`。Sway、Hyprland、COSMIC、Labwc、niri、river 等 wlroots 系有 v2。
  <https://wayland.app/protocols/input-method-unstable-v2>
- **社群實測**：KDE 虛擬鍵盤設定與 fcitx 競爭同一個 input method 槽，plasma-keyboard 與 fcitx5 無法並用。<https://discuss.kde.org/t/plasma-keyboard-and-fcitx5-possible-to-use-them-together/40949>
- X11：Fcitx5 classicui 自繪 X11 視窗；XWayland 下輸入法支援「as good as normal X11」（Fcitx wiki）。

### 7.8 語言與 SDK

- Fcitx5：C++（見 7.1）。IBus：核心 C + GObject，engine 是獨立程序（經 DBus 與 daemon 通訊），可用任何有 GObject Introspection 的語言；repo 的 `bindings/` 有 `pygobject` 與 `vala`。Rust 官方綁定：未查證。
  <https://github.com/ibus/ibus/tree/d464316d9b9613005c5573a010fec7f9b40b93b6/bindings>
- **社群實測**：青簡的 Linux 殼有 `fcitx4`、`fcitx5`、`ibus` 目錄與一個 `server`，另有 `surrounding.h`；README 稱 Linux 需手動啟動背景服務。<https://github.com/qingjian-team/qingjian/tree/main/apps/linux>

### 7.9 同步

- 無平台限制（輸入法為獨立程序），雲端同步需自行實作（未查證官方說明）。
- IBus 在 Wayland 下有 `IBUS_CAP_SYNC_PROCESS_KEY` capability，影響引擎能否非同步處理按鍵。
  <https://github.com/ibus/ibus/blob/d464316d9b9613005c5573a010fec7f9b40b93b6/src/ibustypes.h#L112-L114>

---

## 8. 對 QIM 的影響（只列取捨，不拍板）

### 8.1 平台順序

| 考量 | 偏向先做 | 原因（事實依據） |
|---|---|---|
| 驗證核心與速成引擎的成本 | Android 或 macOS / Windows 桌面 | Android 無已查證的硬記憶體上限、可用 JNI；桌面無沙箱與記憶體限制。iOS 要先過記憶體上限與 Full Access 規則 |
| 市場與審核風險 | iOS 放後 | App Review 4.4.1 要求「無 Full Access 仍須可用」；但核心功能（離線碼表查詢）本來就不需要網路，可以符合 |
| 上下文依賴功能（聯想、語言模型） | 桌面 Linux / Windows 較有把握 | Android 上下文可讀但宿主可少回；macOS 可能 `NSNotFound`；iOS 長度未查證。核心需允許「無上下文」 |
| 平台碎片 | Linux 放後 | GNOME / KDE / wlroots 協定不同；Fcitx5 與 IBus 要各做一個薄層 |
| 已有成熟 Rime 例子可學 | 全部皆有 | Squirrel、Weasel、Trime、Hamster、fcitx5-android；青簡為 Rust 全平台例（無 iOS） |

### 8.2 核心語言

- 每個平台都能以 C ABI 嵌入 C / C++ / Rust 核心：
  - iOS：static library / xcframework（Hamster 以 librime framework 為例）。
  - Android：NDK + JNI（Trime、fcitx5-android、青簡）。
  - macOS：同程序鏈結（Squirrel、fcitx5-macos）；Rust 殼有青簡例。
  - Windows：TSF DLL 薄層 + 獨立 server 程序（Weasel、PIME、青簡）。核心不應放進 DLL。
  - Linux：Fcitx5 需 C++ 薄層 addon；IBus engine 是獨立程序，語言較自由。
- 取捨：
  - C / C++：與 Rime 生態最近，Fcitx5 addon 零摩擦；記憶體安全與跨平台建置成本較高。
  - Rust：青簡證明各桌面殼與 Android 可全 Rust；Fcitx5 仍要 C++ 薄層；iOS 擴充的 Rust 經驗在本研究中**未查證**（uniffi / C ABI 的實作需 prototype）。
  - 授權：青簡為 GPL-3.0-or-later；librime 為 BSD；若要直接參考原始碼，授權相容性需另評估（本研究未做授權查證）。
- iOS 擴充的記憶體上限決定「核心常駐記憶體」預算，碼表載入策略（mmap、分段載入、精簡索引）要在最低機型實測。

### 8.3 隱私

- iOS：不開 Full Access 的鍵盤，系統保證按鍵只送到宿主；這是最強、最可對外聲明的隱私姿態。但這同時鎖死網路、共享容器可寫與剪貼簿。App Group 在沒有 Full Access 時鍵盤只能讀，所以詞庫 / 設定要由 containing app 寫、鍵盤讀。
- 若 QIM 主打「完全離線、無網路」，iOS 的 `RequestsOpenAccess=false` 剛好吻合 4.4.1 的精神，也最簡單過審。需要同步或雲端功能時，才在 containing app 做，而不是在鍵盤擴充做。
- Android：可請求 `IME_FLAG_NO_PERSONALIZED_LEARNING` 時要遵守；密碼欄應自行不學習、不顯示候選、不儲存。沒有系統強制換 IME，責任在輸入法。
- macOS / Windows / Linux：密碼欄判斷方式各自不同且不可靠（macOS 靠 Secure Event Input 加啟發式、Windows 官方做法是停用 text services、Linux 靠 content hint / capability）。統一策略建議由核心提供「sensitive 模式」開關，各殼盡力判斷、判斷不到時預設不學習不記錄（取捨：誤判成非敏感會多記一筆）。
- 剪貼簿：Android 預設 IME 才可讀；iOS 需 Full Access；macOS 15.4+ 有權限提示。若「速成」功能不需要讀剪貼簿，可整個不碰，省去各平台權限與提示。
- 本地 AI（聯想、重排）：iOS 擴充內的 Foundation Models / Core ML 可用性未查證；Android Gemini Nano 在 IME 內可用性未查證。若計畫依賴，先做 prototype，不要先寫進需求。

### 8.4 其他需要實機驗證（prototype 清單）

1. iOS：目標最低機型 / iOS 版本的鍵盤擴充記憶體上限；`documentContextBeforeInput` 的實際長度與 nil 情境；Foundation Models / Core ML 在擴充內是否可呼叫。
2. Android：IME 內呼叫 ML Kit GenAI 是否被 `BACKGROUND_USE_BLOCKED`。
3. macOS：Pasteboard privacy 對 IMK 輸入法是否生效；`~/Library/Input Methods` 是否被接受；Mac App Store 能否上架。
4. Windows：`GetStatus` 在 console / 遊戲的行為；app container 宿主下 DLL 載入與資料檔 ACL。
5. Linux：GNOME Wayland 下 Fcitx5 clipboard 實際行為。

---

## 9. 來源

### Apple（官方）

- Creating a custom keyboard — <https://developer.apple.com/documentation/uikit/creating-a-custom-keyboard>
- Configuring a custom keyboard interface — <https://developer.apple.com/documentation/uikit/configuring-a-custom-keyboard-interface>
- Configuring open access for a custom keyboard — <https://developer.apple.com/documentation/uikit/configuring-open-access-for-a-custom-keyboard>
- Handling text interactions in custom keyboards — <https://developer.apple.com/documentation/uikit/handling-text-interactions-in-custom-keyboards>
- UITextDocumentProxy — <https://developer.apple.com/documentation/uikit/uitextdocumentproxy>
- UIInputViewController.needsInputModeSwitchKey — <https://developer.apple.com/documentation/uikit/uiinputviewcontroller/needsinputmodeswitchkey>
- UIApplicationDelegate application(_:shouldAllowExtensionPointIdentifier:) — <https://developer.apple.com/documentation/uikit/uiapplicationdelegate/application(_:shouldallowextensionpointidentifier:)>
- UIPasteboard — <https://developer.apple.com/documentation/uikit/uipasteboard>
- os_proc_available_memory — <https://developer.apple.com/documentation/os/os_proc_available_memory>
- App Extension Programming Guide: Custom Keyboard（歸檔，2017）— <https://developer.apple.com/library/archive/documentation/General/Conceptual/ExtensibilityPG/CustomKeyboard.html>
- App Review Guidelines — <https://developer.apple.com/app-store/review/guidelines/>
- Foundation Models / SystemLanguageModel — <https://developer.apple.com/documentation/foundationmodels> ；<https://developer.apple.com/documentation/foundationmodels/systemlanguagemodel>
- InputMethodKit：IMKServer、IMKInputController、IMKCandidates、IMKServerInput、IMKStateSetting — <https://developer.apple.com/documentation/inputmethodkit>
- NSPasteboard.AccessBehavior — <https://developer.apple.com/documentation/appkit/nspasteboard/accessbehavior-swift.enum>
- TN2150 Secure Event Input — <https://developer.apple.com/library/archive/technotes/tn2150/_index.html>
- Notarizing macOS software — <https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution>

### Android / Google（官方）

- Create an input method — <https://developer.android.com/develop/ui/views/touch-and-input/creating-input-method>
- InputConnection — <https://developer.android.com/reference/android/view/inputmethod/InputConnection>
- InputMethodService — <https://developer.android.com/reference/android/inputmethodservice/InputMethodService>
- InputType — <https://developer.android.com/reference/android/text/InputType>
- EditorInfo — <https://developer.android.com/reference/android/view/inputmethod/EditorInfo>
- Android 10 privacy changes（剪貼簿）— <https://developer.android.com/about/versions/10/privacy/changes>
- ClipboardManager — <https://developer.android.com/reference/android/content/ClipboardManager>
- Copy and paste — <https://developer.android.com/develop/ui/views/touch-and-input/copy-paste>
- ClipDescription — <https://developer.android.com/reference/android/content/ClipDescription>
- Background FGS start restrictions — <https://developer.android.com/develop/background-work/services/fgs/restrictions-bg-start>
- Settings — <https://developer.android.com/reference/android/provider/Settings>
- ML Kit GenAI APIs — <https://developers.google.com/ml-kit/genai>
- Google Play User Data policy — <https://support.google.com/googleplay/android-developer/answer/10144311>

### Microsoft（官方）

- Text service registration — <https://learn.microsoft.com/en-us/windows/win32/tsf/text-service-registration>
- ITfInputProcessorProfileMgr — <https://learn.microsoft.com/en-us/windows/win32/api/msctf/nn-msctf-itfinputprocessorprofilemgr>
- Third-party IMEs（Windows 8）— <https://learn.microsoft.com/en-us/windows/win32/w8cookbook/third-party-input-method-editors>
- Input Method Editor requirements — <https://learn.microsoft.com/en-us/windows/apps/develop/input/input-method-editor-requirements>
- UILess mode overview — <https://learn.microsoft.com/en-us/windows/win32/tsf/uiless-mode-overview>
- ITfContext::GetStatus — <https://learn.microsoft.com/en-us/windows/win32/api/msctf/nf-msctf-itfcontext-getstatus>
- TS_STATUS — <https://learn.microsoft.com/en-us/windows/win32/api/textstor/ns-textstor-ts_status>
- ITfContextView::GetTextExt — <https://learn.microsoft.com/en-us/windows/win32/api/msctf/nf-msctf-itfcontextview-gettextext>
- InputScope — <https://learn.microsoft.com/en-us/windows/win32/api/inputscope/ne-inputscope-inputscope>
- SampleIME — <https://github.com/microsoft/Windows-classic-samples/tree/master/Samples/IME/cpp>

### Linux（官方文件與原始碼）

- Fcitx5 on Wayland — <https://fcitx-im.org/wiki/Using_Fcitx_5_on_Wayland>
- Develop a simple input method（Fcitx5）— <https://fcitx-im.org/wiki/Develop_an_simple_input_method>
- fcitx5 原始碼（commit `145e981`）— <https://github.com/fcitx/fcitx5/tree/145e981d6377550ffb39ee8de61452910622652c>
- fcitx5-qt / fcitx5-gtk — <https://github.com/fcitx/fcitx5-qt> ；<https://github.com/fcitx/fcitx5-gtk>
- IBus 原始碼（commit `d464316`）— <https://github.com/ibus/ibus/tree/d464316d9b9613005c5573a010fec7f9b40b93b6>
- Wayland text-input-unstable-v3 — <https://gitlab.freedesktop.org/wayland/wayland-protocols/-/raw/main/unstable/text-input/text-input-unstable-v3.xml>
- Wayland core protocol — <https://gitlab.freedesktop.org/wayland/wayland/-/raw/main/protocol/wayland.xml>
- input-method-unstable-v2、ext-data-control-v1（wayland.app 依 XML 產生的瀏覽頁）— <https://wayland.app/protocols/input-method-unstable-v2> ；<https://wayland.app/protocols/ext-data-control-v1>
- GNOME Mutter text-input — <https://gitlab.gnome.org/GNOME/mutter/-/raw/main/src/wayland/meta-wayland-text-input.c>
- gnome-shell inputMethod.js — <https://gitlab.gnome.org/GNOME/gnome-shell/-/raw/main/js/misc/inputMethod.js>
- Flatpak sandbox permissions — <https://docs.flatpak.org/en/latest/sandbox-permissions.html>

### 社群實測 / 第三方專案

- Hamster（iOS Rime）— <https://github.com/imfuxiao/Hamster>（commit `6569370`）
- Squirrel（macOS Rime）— <https://github.com/rime/squirrel>（commit `46962d2`）
- Weasel（Windows Rime）— <https://github.com/rime/weasel>
- Trime（Android Rime）— <https://github.com/osfans/trime>
- fcitx5-android — <https://github.com/fcitx5-android/fcitx5-android>
- fcitx5-macos — <https://github.com/fcitx-contrib/fcitx5-macos>（commit `3466612`）
- PIME — <https://github.com/EasyIME/PIME>
- khiin-rs — <https://github.com/OMAMA-Taioan/khiin-rs>
- windows-chewing-tsf — <https://github.com/chewing/windows-chewing-tsf>
- 青簡 Qingjian（Rust 全平台）— <https://github.com/qingjian-team/qingjian>（commit `3d5fd2d`）
- React Native issue 31910（48 MB）— <https://github.com/facebook/react-native/issues/31910>
- Medium：CoreText emoji 記憶體（77 MB）— <https://medium.com/@mohasalah/from-2014-to-now-fixing-coretext-emoji-glyphs-memory-retention-on-ios-1c6a227d592d>
- Medium：Limitations of custom iOS keyboards（30 MB，2014）— <https://medium.com/@inFullMobile/limitations-of-custom-ios-keyboards-3be88dfb694>
- Sarunw：iOS 16 UIPasteboard privacy — <https://sarunw.com/posts/uipasteboard-privacy-change-ios16>
- jetsamctl — <https://github.com/conradev/jetsamctl>
- Microsoft Q&A：MSIX TSF IME — <https://learn.microsoft.com/en-us/answers/questions/5995922/native-tsf-ime-pure-msix-deployment>
- KDE discuss：plasma-keyboard 與 fcitx5 — <https://discuss.kde.org/t/plasma-keyboard-and-fcitx5-possible-to-use-them-together/40949>
