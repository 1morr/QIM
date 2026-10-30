# QIM 設計綱要（Design Brief）

> 狀態：骨架。由 `/grill-with-docs` 逐節填寫。本檔只寫現況：決定改了就直接改寫，不留「原寫」「取代原」「某日改」；舊版與理由記在 progress〈偏差與決定〉，影響架構的寫 ADR。標【決定】的段落是拍板過的，之後要推翻得在 `docs/progress.md`〈偏差與決定〉記一行。每節底下的「要回答」是訪談的起點，填完就刪掉。

## 0. 一句話

要回答：QIM 是什麼、給誰用、和現有速成輸入法比好在哪。

## 1. 產品定位

### 1.1 本產品負責

### 1.2 本產品不負責

### 1.3 參考的成熟產品與採用的慣例

要回答：參考哪些輸入法（各平台內建速成、Rime 的速成方案、其他第三方），各自採用了哪些慣例、哪些刻意不同。

### 1.4 名稱

## 2. 使用者與情境

要回答：主要使用者（速成熟手、從倉頡轉來、初學者）、主要使用場景、哪些場景不支援。

## 3. 平台範圍與順序

要回答：要支援哪些平台（macOS、Windows、Linux、iOS、Android）、第一個平台是哪個、之後的順序與理由。

## 4. 輸入行為

要回答：取碼規則、選字與翻頁、重碼排序、標點與全形 / 半形、中英切換、繁簡輸出、詞組與聯想、容錯碼 / 萬用字元、字根提示（查碼）。

## 5. 碼表與資料

要回答：碼表來源與授權、字集範圍（Big5、HKSCS、Unicode 擴展區）、字頻來源、使用者自訂詞與學習、碼表的版本與更新。

## 6. 核心引擎

要回答：建在既有引擎（如 librime）上，還是自己寫共用核心；核心用什麼語言；核心與各平台殼的分界。

## 7. 設定與同步

要回答：有哪些設定、放在哪裡、要不要跨裝置同步。

## 8. 隱私與安全

要回答：是否完全離線、是否記錄輸入、iOS「完整取用權限」要不要開、使用者詞庫怎麼存。

## 9. 效能與品質指標

要回答：按鍵到候選出現的延遲上限、記憶體上限、首選字正確率怎麼量。

## 10. 發佈與更新

要回答：各平台怎麼安裝（商店、安裝檔、套件管理器）、簽章與公證、更新方式、授權條款。

## 11. 待決

尚未拍板的問題，每條寫明卡在哪、誰來決定。

## 12. 查證過的事實

外部事實（平台 API 行為、碼表授權、外部引擎能力），每條附來源連結或 `docs/research/` 的檔案。沒有來源的不寫進來。查證日 2026-10-30；細節與未查證清單在各研究檔。

### 12.1 速成與碼表

- 速成碼取倉頡碼首尾兩碼，1–2 碼的倉頡碼不變。[Microsoft Traditional Chinese IME](https://github.com/MicrosoftDocs/globalization/blob/main/globalization/input/traditional-chinese-ime.md)、[Apple 速成說明](https://support.apple.com/en-gb/guide/chinese-input-method/cimsgt11821/mac)、`rime/rime-quick` `quick5.schema.yaml`。
- 速成重碼重：fcitx 速成表全集 674 個碼、單碼最多 480 字，常用集單碼最多 93 字。`docs/research/data-sources.md` §3.2（本機統計）。
- libcangjie `data/table.txt`（77,575 列，倉頡三代與五代、HKSCS 旗標）聲明為 Public Domain。[libcangjie](https://gitlab.freedesktop.org/cangjie/libcangjie) `data/README.table.md`。
- Unihan `kCangjie` 29,189 字，對應倉頡三代，授權 Unicode License v3。[UAX #38](https://www.unicode.org/reports/tr38/)、[Unicode copyright](https://www.unicode.org/copyright.html)。
- rime-cangjie、rime-quick、rime-essay 為 LGPL-3.0；`cangjie5.base.dict.yaml` 檔頭卻寫 GPL，上游《五倉世紀》權利人未查明。各 repo LICENSE；`docs/research/rime.md` §3。
- 繁體詞頻只找到 LGPL 來源：rime-essay（442,688 筆）、libchewing-data（160,462 筆）。教育部國語辭典為 CC BY-ND 3.0 台灣、無頻次。`docs/research/data-sources.md` §5。
- rime-cantonese 為 CC BY 4.0（`maps` 為 ODbL），但詞條參考了非商業授權的粵典及商業輸入法詞庫，授權鏈未查明；粵典 `wordslist`（字 → 讀音）為 Public Domain。[rime-cantonese](https://github.com/rime/rime-cantonese)、[粵典授權](https://words.hk/base/hoifong/)。
- OpenCC（含字典）為 Apache-2.0；純 Rust 實作 ferrous-opencc 亦為 Apache-2.0。[OpenCC](https://github.com/BYVoid/OpenCC)。
- CLDR annotations（yue、zh_Hant、zh_Hant_HK）為 Unicode-3.0。[CLDR](https://github.com/unicode-org/cldr/tree/main/common/annotations)。
- HKSCS 資料集（含倉頡碼、粵拼）商用免費，須署名，有賠償條款。[data.gov.hk 條款](https://data.gov.hk/en/terms-and-conditions)。

### 12.2 Rime 行為（librime `56737cd2`）

- rime-quick 的 quick5 匯入倉頡全碼表 `cangjie5.base`，用 `derive/^([^z])\w+(\w)$/$1$2/` 另加速成碼；`derive` 保留原拼寫且不降權，所以倉頡全碼也是合法輸入、排序時與速成碼不分。`rime-quick` `5dcdb9e`；`src/rime/algo/calculus.h:72-73`。
- script_translator 只要有詞條精確覆蓋整段輸入就不造句。`src/rime/gear/script_translator.cc:502-505`。
- 同長度候選先比是否精確匹配、再比剩餘碼長、最後比詞頻。`src/rime/dict/dictionary.cc:73-84`。
- 不用語言模型造句時每多一個詞扣 log(1e-6)。`src/rime/gear/grammar.h:18-27`。
- 使用者詞熱度隨上屏次數指數衰減，時間常數 200 次。`src/rime/algo/dynamics.h`、`src/rime/dict/user_dictionary.cc:425-457`。
- rime-quick `45d0690`（2026-07-17）拿掉 `min_phrase_weight: 100`，八股文全部約 49.8 萬條多字詞進碼表，含權重 0 的詞。
- 三案例根因：`anr` 的「問」是倉頡全碼、與「門口」同長而比詞頻（50737 > 9435）；`mir` 的「一台」不在詞庫，且精確詞條（䂖、一店、歹口）擋掉造句；只保留速成碼後 `anr` 首選變「門口」。`docs/research/rime.md` §4，復現：`scripts/experiments/rime-cases/`。
- librime-octagram 外掛為 BSD-3-Clause，語法模型資料 rime-octagram-data 為 LGPL-3.0；繁體字級 / 詞級壓縮模型 10.3 MB / 41.4 MB（release 20260712）。[librime-octagram](https://github.com/lotem/librime-octagram)、[rime-octagram-data](https://github.com/lotem/rime-octagram-data)。

### 12.3 平台框架

- iOS：密碼欄與電話欄由系統強制換回系統鍵盤；App 可用 `shouldAllowExtensionPointIdentifier` 拒絕第三方鍵盤。[Configuring a custom keyboard interface](https://developer.apple.com/documentation/uikit/configuring-a-custom-keyboard-interface)、[App Extension Programming Guide](https://developer.apple.com/library/archive/documentation/General/Conceptual/ExtensibilityPG/CustomKeyboard.html)。
- iOS：沒有 Full Access 時無網路、App Group 共享容器唯讀；開了才能網路、寫共享容器、iCloud。[Configuring open access](https://developer.apple.com/documentation/uikit/configuring-open-access-for-a-custom-keyboard)。
- iOS：鍵盤擴充有記憶體上限，Apple 不公布數字、隨機型而異；社群實測 30–77 MB 互相矛盾。[Creating a custom keyboard](https://developer.apple.com/documentation/uikit/creating-a-custom-keyboard)；`docs/research/platform-frameworks.md` §3.6。
- iOS：App Review 4.4.1 要求鍵盤在沒有 Full Access 時仍可用。[App Review Guidelines](https://developer.apple.com/app-store/review/guidelines/)。
- iOS：只能經 `textDocumentProxy` 讀游標前後文，長度上限官方未寫；沒有 marked text。`docs/research/platform-frameworks.md` §3.2。
- Android：10+ 只有預設 IME 或有焦點的 App 能讀剪貼簿。[Android 10 privacy changes](https://developer.android.com/about/versions/10/privacy/changes)。
- Android：`getTextBeforeCursor` 是 IPC，可能回 null 或少於要求的字數。[InputConnection](https://developer.android.com/reference/android/view/inputmethod/InputConnection)。
- Android：密碼欄仍交給當前 IME，IME 須自己不顯示、不儲存。`docs/research/platform-frameworks.md` §4.3。
- Android：ML Kit GenAI（Gemini Nano）只允許 top foreground app 推論，背景回 `BACKGROUND_USE_BLOCKED`；IME 是否算前景未查證。[ML Kit GenAI](https://developers.google.com/ml-kit/genai)。
- Android 11+：IME 可顯示 autofill inline 建議，選取前看不到內容。[IME autofill](https://developer.android.com/identity/autofill/ime-autofill)。
- macOS：IMK 沒有「游標前 N 字」API，`selectedRange` 等可能回 nil 或 `NSNotFound`；沒有 email / URL 欄位提示，密碼欄只能靠 Secure Event Input 判斷。SDK `IMKInputSession.h`；`docs/research/platform-frameworks.md` §5。
- Windows：TSF DLL 載入每個宿主程序；密碼欄應停用 text services，不靠 `IS_PASSWORD`；app container 內網路不保證。[InputScope](https://learn.microsoft.com/en-us/windows/win32/api/inputscope/ne-inputscope-inputscope)、[IME requirements](https://learn.microsoft.com/en-us/windows/apps/develop/input/input-method-editor-requirements)。
- Linux：Wayland text-input-v3 的 surrounding text 上限 4000 bytes，`content_hint` 有 `hidden_text`、`sensitive_data`；GNOME 走 text-input-v3 加 IBus，不實作 input-method-v2。[text-input-v3](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/raw/main/unstable/text-input/text-input-unstable-v3.xml)、[mutter](https://gitlab.gnome.org/GNOME/mutter/-/raw/main/src/wayland/meta-wayland-text-input.c)。
- 共用核心先例：Rime 系列為 C++ 核心；青簡為 Rust 核心加 macOS / Windows / Android 的 Rust 殼，無 iOS。`docs/research/platform-frameworks.md` §1。

### 12.4 成熟產品的做法

- Gboard 解碼：lexicon FST、n-gram LM、空間模型組合後 beam search；按鍵約 20 ms 內回饋，鍵盤 LM 不超過 5–10 MB；個人化用動態模型加 on-the-fly lattice rescoring。[arXiv:1704.03987](https://arxiv.org/abs/1704.03987)。
- Gboard 下一詞預測：CIFG-LSTM 1.4M 參數、量化後 1.4 MB、詞表 1 萬；英文靜態 LM 為 Katz 5-gram 125 萬條。[arXiv:1811.03604](https://arxiv.org/abs/1811.03604)。
- Gboard 的神經 LM 皆以聯邦學習加差分隱私訓練；OOV 學詞取的是全體使用者的高頻詞。[arXiv:2305.18465](https://arxiv.org/abs/2305.18465)、[Google Research blog](https://research.google/blog/advances-in-private-training-for-production-on-device-language-models/)。
- macOS 倉頡 / 速成可輸入英文字後從候選窗選，並有「Adjust word frequency」設定。[Apple 倉頡](https://support.apple.com/guide/chinese-input-method/cangjie-traditional-cimcjt11820/mac)、[Apple 設定](https://support.apple.com/guide/chinese-input-method/change-chinese-and-cantonese-settings-cim21aa5fa50/mac)。
- Apple 端上基礎模型約 3B 參數；iPhone 15 Pro 每個 prompt token 約 0.6 ms、生成 30 token/s。[Apple ML Research](https://machinelearning.apple.com/research/introducing-apple-foundation-models)。
- llama.cpp 在 iPhone 15 Pro 的 Q4_0 實測：TinyLlama 1.1B prompt 646 token/s、生成 57 token/s。[llama.cpp #4508](https://github.com/ggml-org/llama.cpp/discussions/4508)。
- libime：KenLM trie，beam 20，每按一鍵增量更新 lattice；使用者歷史 bigram 以三個 LRU 池（128 / 8192 / 65536 句）固定權重混入。`fcitx/libime` `src/libime/core/decoder.h`、`historybigram.cpp`。
- afm-ime 用 Apple Foundation Models 做異步重排，結果只放第 2 位。`Bemly/afm-ime` `Sources/IMECore/FMReranker.swift`（`a8543a1`）。
- 可重用的滑動輸入實作：FlorisBoard v0.5.2 `StatisticalGlideTypingClassifier.kt`、AnySoftKeyboard `GestureTypingDetector.java`，皆 Apache-2.0；HeliBoard / OpenBoard 的滑動依賴閉源函式庫。`docs/research/open-source-imes.md` §1。
- 開源佈局格式（FlorisBoard、Trime、Hamster、AnySoftKeyboard、HeliBoard）都只支援矩形按鍵。`docs/research/open-source-imes.md` §4。
