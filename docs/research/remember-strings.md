# 自動記住 email、網址：成熟做法

> 2026-11-12 更正：§1 把 `forgetting_curve_utils.cpp` 的等級表當成 LatinIME 現行的衰減，實際路徑是計數加 GC 時移除 300 天沒用的條目，見 `learning-forgetting-sync.md` §2.2。

查證日期 2026-11-11。範圍：打過一次的 email 或網址，之後打開頭幾個字母就出現在候選裡，選了整串上屏。涵蓋 AOSP LatinIME（Gboard 的開源前身）、Gboard、iOS／macOS 內建鍵盤、Microsoft SwiftKey、百度／搜狗／微信輸入法、Rime（librime）、fcitx5。只採官方說明頁、開發者文件、原始碼；找不到官方說明的寫「沒找到官方說明」，社群回報另標。

先講結論：**沒有任何一家公開說明「email／URL 要打幾次、打幾個字母才出現」**。能查到的硬規則只有 LatinIME 的原始碼，其他產品多半只有一句功能描述。

## 1. AOSP LatinIME（原始碼）

快照：`android.googlesource.com/platform/packages/inputmethods/LatinIME` 的 `127336e9f29d69607eab55982324b210279ae8c5`（2025-02-26，main）。Gboard 是閉源，LatinIME 是它的開源前身，Gboard 現況不一定相同。下面路徑都相對於該 commit，連結形式為 `https://android.googlesource.com/platform/packages/inputmethods/LatinIME/+/127336e9f29d69607eab55982324b210279ae8c5/<路徑>`。

- **從哪裡記**：使用者打出並上屏的字。`InputLogic.commitChosenWord` 在每次上屏後呼叫 `performAdditionToUserHistoryDictionary`（`java/src/com/android/inputmethod/latin/inputlogic/InputLogic.java` L1424–1443、L2191），再進 `DictionaryFacilitatorImpl.addToUserHistory`（`.../latin/DictionaryFacilitatorImpl.java` L499）。沒有「要打幾次」的門檻：`UserHistoryDictionary.addToDictionary` 每次以 `count = 1` 寫入（`.../latin/personalization/UserHistoryDictionary.java`）。
- **通訊錄**：另有 `ContactsBinaryDictionary`，載入時先把裝置帳號的 email 位址當單字加入（`loadDeviceAccountsEmailAddressesLocked`，`.../latin/ContactsBinaryDictionary.java` L83–101），再載通訊錄姓名。
- **沒有 email／URL 專用的學習規則**：整個 `java/src` 與 `common/src` 搜尋 `looksLikeURL` 只有 `StringUtils.lastPartLooksLikeURL` 一個定義，兩處使用，都不是學習路徑：
  - `RichInputConnection.textBeforeCursorLooksLikeURL`（`.../latin/RichInputConnection.java` L899）；
  - `InputLogic.insertAutomaticSpaceIfOptionsAndTextAllow`（`InputLogic.java` L2020–2025）：游標前文字像 URL 時**不自動補空格**。
- **怎麼判斷像 URL**：`common/src/com/android/inputmethod/latin/common/StringUtils.java` L439 起的 `lastPartLooksLikeURL`，是從游標往回掃的啟發式，不是正規式：字元碼在 `.`（0x2E）到 `z` 之間算 URL 字元（含小寫字母、`_`、`@`、`?`、`=`、`/`、`.`），碰到範圍外字元就停；符合其一即為 URL：連續 `//`；以 `www` 開頭且含 `.`；以單一 `/` 開頭且前一字元是空白；同時含 `.` 與 `/`。原始碼自己註明 `abc./def` 這類也會誤判。**不判斷 email**，沒有任何 `EMAIL_ADDRESS` 的正規式。
- **含 `@`、`.` 的字串怎麼切**：`res/values/donottranslate-config-spacing-and-punctuations.xml` L33、L35：`symbols_word_separators` 含 `. , ; : ! ? / _` 與括號、空白，`symbols_word_connectors` 只有 `'` 和 `-`。`@` 兩邊都不在，`isWordCodePoint('@')` 為 false（`SpacingAndPunctuations.java` L95–105）。也就是 `.` 一定切字，`user@gmail.com` 不會被當成一整個詞記下來。`@` 實際如何切開沒有實測，只能說原始碼裡沒有把它當詞內字元。
- **未知字串怎麼處理**：`addWordToUserHistory`（`DictionaryFacilitatorImpl.java`）對主詞典查不到的字，用 `isValid = maxFreq > 0` 為 false 寫入，註解寫明「We demote unrecognized words」；`UserHistoryDictionary.isValidWord` 恆回傳 false。所以學到的未知字串權重較低，不會搶在正常字詞前面。
- **什麼會觸發**：全域規則，沒有「幾個字母」的設定。email 欄位（`TYPE_TEXT_VARIATION_EMAIL_ADDRESS`、web email）與 URI 欄位在 `InputAttributes.java` L104–109 被列入 `shouldSuppressSuggestions`，**候選條整個不出現**。
- **候選排在哪**：沒有「第幾格」的規則；使用者歷史詞典的詞和主詞典合併後依機率排序，沒找到針對 email 的位置設定。
- **隱私**：
  - 密碼欄（`InputAttributes.mIsPasswordField`）不顯示候選條（`LatinIME.java` L1603）。
  - 自動修正關閉就不學：`performAdditionToUserHistoryDictionary` 開頭 `if (!settingsValues.mAutoCorrectionEnabledPerUserSettings) return;`，註解寫「avoid unintended additions in some sensitive fields, or fields that expect to receive non-words」；欄位帶 `NO_AUTO_CORRECT` 旗標也會讓它為 false（`SettingsValues.java` L185）。
  - 慢的 InputConnection 不學（避免學進之後被退格刪掉的錯字）。
  - 設定頁 `pref_key_use_personalized_dicts` 可整個關閉個人化詞典（`res/xml/prefs_screen_correction.xml` L57）。
- **上限與過期**：詞典 header 啟用 forgetting curve（`UserHistoryDictionary.getHeaderAttributeMap`）。`native/jni/src/dictionary/utils/forgetting_curve_utils.cpp` L30–41：衰減間隔 2 小時、每級降低時間 15 天、`MAX_LEVEL = 15`、硬上限係數 1.2、單字長度上限 48（`BinaryDictionary.DICTIONARY_MAX_WORD_LENGTH`，超過就不記）。條數上限由 header 的 `MAX_UNIGRAM_ENTRY_COUNT` 決定，Java 層沒找到預設值。

## 2. Gboard（Android 與 iOS）

- **官方有說的**：
  - 「Learn how Gboard gets better」：個人化選項為「Personalize for you」，「Audio recordings and transcripts of what you say and type will be saved on this device」，刪除路徑是 Gboard 設定 › Privacy › **Delete learned words & data**，「Clears all on-device data that Gboard has saved」。來源 <https://support.google.com/gboard/answer/12373137?hl=en>。
  - 「Get word suggestions & fix mistakes」：不想要的建議可「touch and hold it, and then drag the word to Trash」；錯字可「Add to dictionary」。來源 <https://support.google.com/gboard/answer/7068415?hl=en&co=GENIE.Platform%3DAndroid>。
  - Android 11 起 IME 可在建議列顯示 Autofill 的 inline 建議，且「the suggestions are hidden from the IME until the user selects one」，因為可能含密碼或信用卡。來源 <https://developer.android.com/identity/autofill/ime-autofill>。這是系統層的 email 帳號建議路徑，和鍵盤自己學的詞是兩回事。
- **沒找到官方說明**：email 或 URL 要打幾次才記、幾個字母觸發、排第幾格、無痕模式是否停止學習、有幾條上限、是否過期。
- 社群回報（非官方）：Gboard 設定 › Text correction 有「Suggest contacts」，開啟後通訊錄裡的 email 較容易出現在建議列（Pixel 社群串 <https://support.google.com/pixelphone/thread/377891271/predictive-text-quit-working-standard-gboard?hl=en>；MakeUseOf <https://www.makeuseof.com/how-to-turn-on-predictive-text-android>）。K-9 Mail 的 issue 也佐證 Gboard 會建議通訊錄 email（<https://github.com/thunderbird/thunderbird-android/issues/3578>）。這些只能當線索，不是規格。
- 刪單條：長按建議拖到垃圾桶（官方），或 Dictionary › Personal Dictionary 逐條刪（How-To Geek 轉述，非官方）。

## 3. iOS／macOS 內建鍵盤（QuickType）

- **官方有說的**：
  - 預測文字「based on your past conversations, writing style, and even websites you visit in Safari」。來源 <https://support.apple.com/en-us/HT207525>。
  - 預測列會出現資訊類建議，例如輸入「My number is」後出現你的電話號碼、「I'm at」後出現目前位置。來源 <https://support.apple.com/en-al/guide/iphone/iphd4ea90231/ios>。
  - 被拒絕的建議：「If you reject the same suggestion a few times, iPhone stops suggesting it」（同頁）。
  - 全部重置：設定 › 一般 › 傳輸或重置 iPhone › 重置 › **重置鍵盤字典**（Apple 使用指南 <https://support.apple.com/en-bh/guide/iphone/iph6d01d862/ios> 的片段）。
  - `UITextContentType.emailAddress` 的官方說明只有「defines the content in a text input area as an email address」，沒有說鍵盤據此做什麼。來源 Apple 開發者文件 JSON <https://developer.apple.com/tutorials/data/documentation/uikit/uitextcontenttype/emailaddress.json>（2026-11-11 擷取）。
- **沒找到官方說明**：email 是否從通訊錄「我的名片」帶入、使用者打過的 email 要打幾次才記、安全輸入欄（`isSecureTextEntry`）是否停止學習（該頁只講禁止複製與遮罩）、上限、過期。
- 社群回報（非官方）：使用者在 Apple StackExchange 指出，欄位出現的 email 建議來自「我的名片」裡標為 email 的欄位（<https://apple.stackexchange.com/questions/391444/ios-keyboard-suggests-an-old-email-address> 的留言，作者 Todd）。macOS 的行為沒有找到獨立說明。
- 中文（速成、倉頡、拼音）候選列是否學 email／URL：沒找到官方說明。本 repo 的黑箱基線只量了詞，沒量 email（見 `blackbox-baseline.md`）。

## 4. Microsoft SwiftKey

- **從哪裡記**：「learn from your personal language data as you type」，「It doesn't matter what you're typing; be it a Tweet, a Facebook status, SMS message, or email」。來源 <https://support.microsoft.com/en-us/swiftkey-keyboard/how-do-i-personalize-my-typing-with-microsoft-swiftkey-keyboard>。
- **email 作為學習的證據**：官方排錯頁把「begin typing an email address (or other such phrase unique to you)」當作檢查預測是否還在的測試，表示 email 位址確實會被學進預測。來源 <https://support.microsoft.com/en-us/swiftkey-keyboard/what-do-i-do-if-i-ve-lost-my-learned-language-data>（2026-11-11 擷取）。
- **儲存**：「By default, nothing Microsoft SwiftKey learns about your typing leaves your device」；登入帳號並開 Backup & Sync 後，會把詞的片段送到 Microsoft 雲端同步（<https://support.microsoft.com/en-us/swiftkey-keyboard/microsoft-swiftkey-keyboard-sharing-your-typing-data-faq>）。
- **隱私**：
  - 密碼欄「deliberately does not offer predictions」（排錯頁）。
  - Incognito：手動從工具列開，會停止學習；偵測到瀏覽器私密瀏覽或欄位標為 private／sensitive 時**自動進入**；iOS 版不支援。來源 <https://support.microsoft.com/en-us/swiftkey-keyboard/how-does-incognito-mode-work-on-your-microsoft-swiftkey-keyboard>。
  - 清除：設定內的「Clear typing data」，清空所有學到的詞與片語，不可復原；或到 <https://data.swiftkey.com/> 要求刪除。
- **沒找到官方說明**：觸發字母數、候選位置、要打幾次、上限、是否過期、單條刪除（清單頁面沒查到）。

## 5. 中文輸入法

- **百度輸入法**（資料最完整）：
  - 手機版說明頁：分詞鍵（1 鍵）可代替空格、`@`、`-` 等符號，「用於組成帶符號短語和詞組，如：I'm, come on, baidu@baidu.com 等」；拇指鍵盤可用「添加詞」把新英文單字加入用戶詞庫。來源 <https://srf.baidu.com?c=index&e=help>。
  - 通訊錄詞庫：可把名片夾聯絡人導入為用戶詞，「輸入人名直接聯想出手機號、郵箱地址等信息」（同頁）。隱私政策 §1.7：僅在本機處理，不上傳；Android 13.3.6.52 起已移除此功能。來源 <https://srf.baidu.com/privacy/index.html>。
  - 用戶詞整理：可查看、刪除本地中文與英文用戶詞；「用戶詞智能清理」自動清掉 10% 最不常用的自造詞（同說明頁與版本日誌 <https://srf.baidu.com/input/update.html>）。
  - Mac 版 2013-04 更新日誌列有「網址郵箱記憶，輸入更方便」，v1.x 另有「智能網址、郵箱輸入」（<https://srf.baidu.com/input/mac_update.html>）。細節沒有說明。
  - 手機版 2013 年版本日誌：「在地址框郵箱框中輸入『.』提供更多符號選擇」。
- **搜狗輸入法**：手機版（iOS）官方頁面寫「網址郵箱：拼音 26 鍵盤，符號上劃，輸入網址郵箱幫你記憶幫你聯想」（<https://shouji.sogou.com/cydia/index.ipad.html>，年代較舊）。說明頁只列了用戶詞管理：長按刪除自造詞、屏蔽記憶聯想詞（<https://shouji.sogou.com/wap/feedback/faqlist>）。`@` 後的郵箱後綴聯想、要打幾次才記：沒找到官方說明。
- **微信輸入法**：隱私政策只有「隱私保護模式」下輸入內容不上傳、僅本機儲存，以及剪貼簿、個人詞庫同步的開關（<https://z.weixin.qq.com/web/privacy>）。email 記憶與 `@` 聯想：沒找到官方說明。

## 6. Rime（librime）

- **沒有內建的 email 記憶**。`Memory::OnCommit`（`src/rime/gear/memory.cc`，librime `c95c0d2`）只會把「`Language::intelligible` 為真」的候選寫進用戶詞典，也就是已經存在於詞典、能還原成編碼的詞；任意 ASCII 字串不會被學。`table_translator` 的 `enable_encoder`／`encode_commit_history` 也只針對 `table`、`user_table`、`sentence`、`uniquified` 型態的上屏歷史造詞（`src/rime/gear/table_translator.cc` L327–345）。
- **使用者手動記**：`custom_phrase.txt`，每行「詞條 Tab 編碼 Tab 權重」，例如 `xxx@gmail.com	gmail	1`（少數派 <https://sspai.com/post/84373>，社群文章）。
- **英文方案的作者建議關掉用戶詞典**：雾凇拼音作者指出英文方案不需要造詞，開了用戶詞典反而造成重碼（<https://dvel.me/posts/make-rime-en-better>）。
- **librime-lua** 能寫 `lua_processor`／`lua_translator` 自行偵測並記錄，少數派文章提到可依 `commit_history` 判斷上下文（<https://sspai.com/prime/story/automating-rime-with-librime-lua>）。沒找到現成的「自動記 email」腳本。
- **刪除**：選字游標移到該詞，Shift+Delete（macOS 用 Fn+Shift+Delete）；`Memory::OnDeleteEntry` 把 entry 標為已刪除。退格可撤銷剛上屏的學習：`OnUnhandledKey` 遇到 BackSpace 會 `DiscardSession`（`memory.cc`）。

## 7. fcitx5

- **沒找到** email 或 URL 的自動記憶，也沒找到官方說明。fcitx5-chinese-addons（`e8d930f`）的拼音引擎只學上屏的拼音句子，進 `libime` 的 `HistoryBigram`。
- 隱私：`PinyinEngine::shouldLearn` 為 `learning` 設定為真且**欄位不是 Password 或 Sensitive**（`im/pinyin/pinyin.cpp` L858–861）；碼表引擎同樣檢查（`im/table/state.cpp` L1264）。
- 框架層有 `CapabilityFlag::Email`、`Url`、`Password`、`Sensitive`（`fcitx5` `src/lib/fcitx-utils/capabilityflags.h`），Wayland 與 IBus 前端會把欄位用途對應過來（`src/frontend/waylandim/waylandimserverv2.cpp` L389），但拼音引擎沒有拿 `Email`、`Url` 旗標做任何事。
- 刪除：選字時可進「選要從歷史刪除的詞」模式（`pinyin.cpp` L1668–1730），也有「清除用戶詞典」「清除全部詞典與歷史」的設定動作（L2405–2413）。
- `quickphrase` 模組是使用者手動維護的快捷短語，不會自動學。

## 8. 對 QIM 的建議

### 各產品慣例比較

| 產品 | 從哪裡記 | 觸發 | 候選位置 | 隱私與刪除 |
| --- | --- | --- | --- | --- |
| LatinIME（Gboard 前身，原始碼） | 上屏的字，每次 count=1；另載入裝置帳號 email 與通訊錄 | 一般詞前綴，沒有 email 專用規則；email、URI 欄位不出候選條 | 沒有固定格；未知字串以 `isValid=false` 降權 | 密碼欄不出候選條、自動修正關閉就不學；forgetting curve（每級 15 天、最高 15 級）；可關個人化詞典 |
| Gboard | 沒找到官方說明；社群稱可選「Suggest contacts」 | 沒找到官方說明 | 沒找到官方說明 | 單條：長按拖垃圾桶；全部：Delete learned words & data |
| iOS／macOS | 官方只說過去對話、寫作習慣、Safari；「我的名片」是社群說法 | 資訊類建議靠句子前綴（My number is） | 位置無官方說明 | 重置鍵盤字典；連續拒絕同一建議會停止；secure 欄位的學習行為沒找到官方說明 |
| SwiftKey | 上屏的字，email 在內；可選雲端同步 | 沒找到官方說明 | 沒找到官方說明 | 密碼欄不預測；Incognito（偵測私密瀏覽、sensitive 欄自動進入）；Clear typing data |
| 百度 | 通訊錄詞庫（本機）、英文造詞、分詞鍵可含 `@` | 聯想輸入；通訊錄詞庫在輸入人名時聯想出郵箱 | 沒找到官方說明 | 可看、刪本地英文用戶詞；自動清 10% 最不常用的自造詞；通訊錄僅本機 |
| 搜狗 | 官方只說「網址郵箱幫你記憶幫你聯想」 | 沒找到官方說明 | 沒找到官方說明 | 長按刪除自造詞、屏蔽記憶聯想詞 |
| 微信 | 沒找到官方說明 | 沒找到官方說明 | 沒找到官方說明 | 隱私保護模式下僅本機；個人詞庫同步可關 |
| Rime | 不會自動記任意字串；手動 `custom_phrase.txt` | 編碼前綴（自訂編碼） | 依權重，與其他候選同排 | Shift+Delete 單條刪；退格撤銷剛學的 |
| fcitx5 | 只學拼音句子，不學 email／URL | — | — | Password／Sensitive 欄位不學；候選中刪單條；可清除全部 |

### 建議採用的組合

1. **從哪裡記：只記使用者自己在 QIM 英文（直通）模式打出並上屏的字串**，不讀通訊錄、剪貼簿、欄位內容。理由：這是 LatinIME 與 SwiftKey 的共同基礎（上屏即學）；通訊錄、剪貼簿在 macOS、Windows、Linux 三平台都要額外授權，且 Baidu 已因此移除通訊錄功能（Android 13.3.6.52）。欄位內已有文字的讀取沒有任何產品當作來源。
2. **什麼算一個要記的字串：用自己的嚴格規則，不照搬 LatinIME。** LatinIME 的 `lastPartLooksLikeURL` 是給「不要自動補空格」用的寬鬆啟發式，會誤判，不適合當學習條件。建議：以空白、換行分隔的一整段 ASCII 連續字元，符合簡化 email（`local@domain.tld`）或 URL（`scheme://`、`www.`、或含 `.` 的網域加路徑）才記，長度上限沿用 LatinIME 的 48 字元量級，需要時再放寬。切字不要沿用 LatinIME 把 `.`、`/` 當分隔的做法，否則整串學不起來。
3. **要打幾次：1 次就記**（LatinIME 與 Rime 都是 1 次），但上屏後立刻退格要撤銷（Rime 的 `DiscardSession` 做法），避免錯字污染。不設「第 2 次才出現」的門檻，因為 email 通常只打幾次，等到第 2 次已經失去價值。
4. **觸發：前綴 ≥ 3 個字元才出現。** 沒有任何產品公開數字，這是 QIM 自己的決定，需要實測。理由是中文模式下前綴太短會和速成碼的候選打架（速成碼很短，這點要對照 brief 確認），所以建議只在英文直通模式，或輸入已含 `@`、`.`、`www` 時觸發。不單靠 `@` 觸發（Baidu 與 Sogou 的 `@` 後綴聯想是另一種功能，記的是網域後綴而不是整串）。
5. **位置：不搶首選、不自動上屏，放第 2 格以後。** LatinIME 對未知字串降權、SwiftKey 與 Gboard 都不把 email 放首選。建議固定在第一頁較後的位置，並且只能手動選。這個格位同樣沒有任何產品公開數字，屬於 QIM 的決定。
6. **隱私：本機儲存，不上傳；密碼與敏感欄位（各平台的 secure／password 旗標）不記、不顯示；有「忘記這一條」與「清除全部」；有上限與過期。** 前三項是 LatinIME、fcitx5、SwiftKey 的共同做法。刪單條沿用候選上的刪除鍵（Rime 的 Shift+Delete、Gboard 的拖垃圾桶）。上限與過期建議採 forgetting curve 的概念，數字（條數上限、衰減期）沒有公開標準，先取保守值並在設定頁揭露。email 欄位本身（LatinIME 不出候選條）建議反過來：email 欄位正是最有價值的場景，QIM 應該出候選。

### 理由小結

- **LatinIME** 最透明，可以直接借的是：未知字串降權、密碼欄不學、forgetting curve、單字長度上限。不借的是它的切字規則與「email 欄位不出候選」。
- **SwiftKey** 的 Incognito 自動偵測是隱私面最成熟的慣例，QIM 應跟進偵測欄位用途旗標。
- **Rime** 的可預期性最好（不自動學、手動可刪），但完全不自動記，正是這個功能要補的缺口；它的撤銷學習與單條刪除值得抄。
- **Apple、Gboard、搜狗、微信**沒有公開足夠細節，不能當設計依據，只能當作確認「使用者預期有這功能」的旁證。
- 沒查到的項目（觸發字母數、候選格位、條數上限）全部標為 QIM 自己的決定，之後用 `docs/development.md` 的實測流程驗證，不要引用成「業界標準」。

## 9. QIM 採用的做法（M3 票 15）

照上面的建議，兩處不同：

- 觸發在中文組字，不是英文直通：QIM 的英文直通沒有組字（鍵交還宿主），沒有候選列可以放；英文模式的預測在 M4。中文組字的輸入碼長過一個字的最長碼（速成 3 個字母）才給，和英文候選同一個門檻；誤觸率用首選字基準的題目量過（`scripts/experiments/remembered-prefix/`）：常見 email 開頭的前 3 個字母約每 1 萬次組字撞一次。比的是 email 裡方案打得出的字母（組字裡打不出 `.`、`@`、數字與速成沒有的 `z`），所以 `bc12345@example.edu` 打 `bce` 起就有、`zz98765@example.org` 打 `exa` 起就有。
- 只記 email、不記網址：網址很少整串重打，LatinIME 也不學網址。上屏後立刻退格撤銷沒做：交還宿主的退格刪掉正在打的那串的最後一個字元，打完的才記。

細節寫在 plan §2.1〈英文使用者詞〉與 §2.4 規則 4、5（M4 票 14 起記住的字串併入英文使用者詞）。
