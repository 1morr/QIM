# Gboard 英文鍵盤黑箱實測

實測日期：2026-11-15，M4 票 `.scratch/m4/issues/19-gboard-english-blackbox.md`。回答第二期英文模式、自動修正、欄位類型、智慧標點要照的行為：自動修正何時修、退格怎麼還原、連按兩下空白、句首大寫、成對符號、建議列、欄位類型。照 ADR 0006 只打字、讀回畫面與欄位內容，不開產品的檔案、不逆向。

截圖在 `.scratch/m4/evidence/`，檔名以 `19-` 開頭，下文寫成 `ev/<檔名>`。測試頁與點鍵腳本在 `scripts/experiments/gboard-english-blackbox/`（用法見它的 README）。

## 環境

| 項目 | 內容 |
| --- | --- |
| 裝置 | 本票新建的 AVD `qim_t19_gboard`（`medium_phone`，1080×2400、420 dpi，映像 `system-images;android-37.0;google_apis_playstore;arm64-v8a`），跑在 Mac18,5 |
| 系統 | Android 17（`CE2A.260420.051.A2`） |
| 輸入法 | Gboard `18.5.10.993608717-release-arm64-v8a`。映像內建的是 17.2.2 preload；18.5.10 的四個 APK（base 與三個 split）從另一台已由 Play 商店更新的模擬器拉出，用 `adb install-multiple` 裝上。裝完沒開過 Gboard，所以沒有學習資料 |
| 語言 | English (US) QWERTY（預設，唯一）；量中文成對符號時另加「粵語」的粵拼、速成兩種佈局 |
| 帳號 | 裝置上沒有任何 Google 帳號；Chrome 選「Stay signed out」 |
| 宿主 | Chrome 145.0.7632.218 開本機測試頁：多行欄、`type=text`、`email`、`url`、`number`、`tel`、`inputmode=numeric`、`search`、`password`、`autocorrect="off"` 的一般欄 |
| 怎麼打 | `adb shell input tap` 點軟鍵盤各鍵的中心（`kb.py`），鍵與鍵間隔 0.25 秒 |
| 怎麼讀 | 測試頁把每個 `input` 事件的 `inputType`、`data`、欄位值、游標位置 POST 回本機記檔（`server.py`）；建議列與鍵盤用截圖讀；設定頁與 `inputType` 用 `uiautomator dump`、`dumpsys input_method` 讀 |

### 設定（全部是預設值，逐項讀回）

截圖 ev/19-settings.png。

- Corrections & suggestions：Auto-correction 開、Auto-capitalization 開、Spell check 開、Grammar check 開、Don't suggest offensive words 開、Suggestion strip 開、Word suggestions 開、Next-word suggestions 開、Suggest contacts 關。
- Preferences：Number row 關、Number row for passwords 開、Emoji key 開、Comma key 開、Period key 開、Suggestion strip 開、One-handed keyboard 關、Double-space period 開、Auto-space after punctuation 關、Auto-switch after apostrophes 開、Touch & hold keys for symbols 關、Flick keys to enter symbols 關、Sound 關、Keyboard vibration 開。
- Privacy：Share usage statistics 開、Personalize for you 開、Improve for everyone 開。

### 為什麼新建 AVD、做過的清除

先用的是已開著的共用模擬器 `Medium_Phone_API_37.0`：它的 Gboard 2026-11-14 才由 Play 商店更新到 18.5.10，前一天票 09b 用過，裝置登入了 Google 帳號，鍵盤開著數字列（預設是關），所以 Gboard 可能學過詞、設定被改過。照協調者指示先 `adb shell pm clear com.google.android.inputmethod.latin`、重新設為預設輸入法，讀回設定都回到預設（只是沒有 Grammar check 這一項），個人字典是空的。量到一半，另一個 session 把自己的 App 切到那台模擬器的前景，就改到本票自己的 AVD 從頭重量。下文的結果全部來自新 AVD；舊機上看到、新機重現不了的列在〈沒解開的〉。

### 量法的限制

- Chrome 收到的 `inputType` 對應到 Gboard 呼叫了哪個 `InputConnection` 方法是推測：`insertText` 是上屏文字、`insertCompositionText` 是組字區、`deleteContentBackward` 是刪游標前的字（`deleteSurroundingText` 也報成這個）、`insertLineBreak` 是 Enter。
- Gboard 會學打過的字（見〈學習〉），所以每項用沒打過的錯字；前一項學到的會影響後一項時另寫明。
- 時間窗的數字含 `adb` 點擊的誤差，約 ±0.1 秒。

## 1. 自動修正

### 1.1 什麼時候修

打字中途不改：Chrome 裡看不到組字底線，每個字母都是一個 `insertText` 直接上屏，欄位裡一直是原樣。建議列中格是粗體時，打下面的鍵就把整個詞換成它（ev/19-autocorrect-strip.png：打 `my freind`，建議列〔freind｜**friend**｜friends〕）。

| 觸發 | 打的鍵 | 結果 | 事件 |
| --- | --- | --- | --- |
| 空白 | `my freind␣` | `My friend␣` | `deleteContentBackward`（一次刪掉整個詞）＋`insertText "friend "`（ev/19-autocorrect-space.png） |
| 逗號 | `the goverment,` | `The government,` | 刪詞＋`insertText "government,"` |
| 句點 | `a wierd.` | `A weird.` | 刪詞＋`insertText "weird."` |
| Enter | `its neccessary⏎` | `It's necessary⏎` | 先換詞再 `insertLineBreak`（ev/19-autocorrect-enter.png） |
| 點建議 | `i like hel` 點中格 | `I like hello␣` | 換掉整個詞並自動補一個空白 |

- 換詞是一步完成：刪掉游標前整個詞再上屏修正字（含觸發的空白或標點）；在句首時改用組字區蓋掉原詞（`insertCompositionText "Because "`），結果一樣。
- `teh␣` 修成 `the␣`，`i␣` 修成 `I␣`，`its␣` 修成 `It's␣`，`mr.` 修成 `Mr.`。
- 修不修看字：`begining`、`embarass`、`occured` 中格不粗體、照原樣；`calender` 是真字不修。

### 1.2 不修的條件

每條先打一次同一個錯字的小寫原形確認會修（對照），再打變體。

| 條件 | 例 | 對照 | 結果 |
| --- | --- | --- | --- |
| 含數字 | `the definately2␣` | `definately` → definitely | 不修 |
| 全大寫（雙按 Shift 鎖大寫） | `the INDEPENDANT␣` | `independant` → independent | 不修 |
| 句中首字大寫 | `the Sucess␣` | `sucess` → success | **照修**成 `Success`，保留首字大寫 |
| 詞裡有 `@` | 一般欄打 `mail jon@gmial.com␣` | — | 打到 `@` 建議列就換成工具列，整串不修（ev/19-field-strips.png 第二列） |
| 網址 | 一般欄打 `visit gogle.com␣` | `gogle` → Google | 打 `.` 時就把 `gogle` 修成 `Google.`，結果 `Google.com`：`.` 是詞界 |
| 欄位沒有 AUTO_CORRECT 旗標 | `type=search`、`autocorrect="off"` 打 `i recieve␣`、`i acheive␣` | — | 照出建議，中格不粗體，不修（ev/19-field-strips.png 第三列） |
| email、網址欄 | `becuase␣`、`wierd␣` | — | 不修、沒有建議（〈欄位類型〉） |

### 1.3 退格與還原

- 修完緊接著退格：只刪掉修正後的那個空白，字不會變回去；建議列換成〔原字｜修正字｜次選〕（`the adress␣` → 退格 → `The address`，建議列〔adress｜address｜addresses〕，ev/19-undo-backspace.png）。再退格就是刪修正字的最後一個字母（`The addres`）。
- 點左格的原字才是還原：換回原字並自動補空白（`a suprise␣` → 退格 → 點 `suprise` → `A suprise␣`，ev/19-undo-revert.png）。事件是刪掉兩字不同的尾巴再上屏（刪 `rprise` 後上屏 `prise `）。
- 還原之後緊接的第一下空白沒有任何事件（被吃掉），第二下才多一個空白。
- 還原過的原字之後在任何欄位都不再修（`suprise␣` 再打兩次都照原樣）：還原等於教 Gboard 這個字。
- 退格後不點原字、直接再打空白：留著修正字（`The library␣`），之後同一個錯字照修。

## 2. 學習：使用者打過的字何時出現

- 怪字 `qimzorb␣` 上屏一次（打的當下中格不粗體，沒被修成 Windsor），再打 `qi` 兩個字母，建議列中格就是 `Qimzorb`；換到另一個欄位、`am force-stop` 重啟 Gboard 後照樣出現（ev/19-learned-word.png，依序是打 `q`、`qi`、`qim`、別的欄位打 `qimz`、重啟後打 `qimz`）。沒有次數門檻。
- 原樣上屏的錯字也學：在 email 欄打 `becuase` 上屏後，一般欄的 `the becuase␣` 就不修了。
- 學到的字會變成自動修正的目標：原樣上屏過 `seperate2`（含數字）、`UNTILL`（全大寫）之後，打小寫的 `the seperate␣` 被「修」成 `seperate2`、`the untill␣` 被修成 `UNTILL`。
- 測試頁直接清空欄位時留在裡面的半截字（`Qimz`）也被學到，建議列多出 `Qimz`；這是本測試的副作用，真實宿主少見。
- 一般欄打過一次 `jon@gmial.com`，再打 `jo` 建議列沒有它（M1 實測 17.2.2 時打 3 次才有時學會，事實 M5）。

## 3. 連按兩下空白

- `hello␣␣` → `Hello.␣`：刪掉第一個空白，上屏「. 」；下一個字自動大寫（`hello␣␣world` → `Hello. World`，ev/19-double-space.png）。
- 兩下的間隔：0.65、0.81、0.96 秒都會變句點；1.06、1.07、1.15、1.16、1.27 秒都不會。時間窗約 1 秒。
- 逗號後（`hello,␣␣`）、句點後（`hello.␣␣`）是兩個空白，不加句點；數字後（`123␣␣`）照樣變 `123.␣`。
- 三下是 `Hello.␣␣`（句點後再一個空白）。
- 變成句點後退格不還原：`Hello.␣` → 退格 `Hello.` → 再退格 `Hello`。
- 點建議自動補的空白之後，第一下空白被吃掉（同〈退格與還原〉）。

## 4. 句首大寫

- 空的欄位、`.` `?` 後接空白、換行之後，Shift 亮起，下一個字母大寫：`Ok. Then`、`Really? Yes`、`Ok⏎Then`。
- 縮寫：`use e.g. this` 不大寫；`i met mr. smith` 的 `mr.` 先被修成 `Mr.`，之後的 `smith` 大寫成 `Smith`（Gboard 把它當句尾）。
- email、網址、密碼欄 Shift 不亮，打出小寫（ev/19-field-keyboards.png）。
- 雙按 Shift 鎖大寫；測試頁清空欄位不會解除。

## 5. 成對符號

- 英文的 ?123 頁：`(` 只上屏 `(`；`"` 是直引號，第二次按還是直引號：打 `"hi"` 得到 `"hi"`，不補右半邊、不左右輪流、不換成彎引號（ev/19-pairs-en.png）。建議列也不提示右括號。
- 粵語（粵拼與速成共用同一個符號頁）：`(` 上屏全形 `（`；`“` `”` 是兩顆鍵，`「` `」` 在第二個符號頁也是兩顆鍵；按左半邊只上屏左半邊（ev/19-pairs-zh.png）。
- 結論：Gboard 英文與粵語鍵盤都沒有成對符號的自動配對，左右半邊是分開的鍵。

## 6. 建議列

- 三格：左格固定是已打的原樣（是真字也一樣，打 `the` 左格是 the），中格是首選（會自動修正時是粗體），右格是次選；有 emoji 時右格換成兩個 emoji，共四格（ev/19-strip-order.png：`the`〔the｜them｜these〕、`hel`〔hel｜hello｜👋｜🤗〕）。
- 打完空白後出下一個詞的預測（`i like␣`〔it｜the｜😍｜💗〕）；剛聚焦的空欄是工具列。
- 點任一格換掉整個詞並補一個空白；接著打標點時先刪掉補的空白再上屏標點（點 `hello` 後打 `,` 得 `hello,`）。
- email、網址、密碼欄與詞裡打了 `@` 之後，建議列換成工具列，沒有字詞建議。
- 使用者打過的字：見〈學習〉。

## 7. 欄位類型

Chrome 145 把 HTML 欄位轉成下表的 `EditorInfo.inputType`（`dumpsys input_method` 讀回）；常數依 [Android `InputType`](https://developer.android.com/reference/android/text/InputType)（2026-11-15 查）：TEXT 1、NUMBER 2、PHONE 3，URI 0x10、WEB_EDIT_TEXT 0xa0、WEB_EMAIL_ADDRESS 0xd0、WEB_PASSWORD 0xe0，CAP_SENTENCES 0x4000、AUTO_CORRECT 0x8000、MULTI_LINE 0x20000、ENABLE_TEXT_SUGGESTION_SELECTED 0x200000（API 37），NUMBER_FLAG_DECIMAL 0x2000。數字與電話類別帶的 0x4000 在官方文件沒有定義。

| HTML | `inputType` | 鍵盤（ev/19-field-keyboards.png） | 建議 | 自動修正 | 句首大寫 |
| --- | --- | --- | --- | --- | --- |
| `textarea` | `0x22c0a1`（TEXT、WEB_EDIT_TEXT、CAP_SENTENCES、AUTO_CORRECT、MULTI_LINE、0x200000） | QWERTY，Enter 換行 | 有 | 有 | 有 |
| `type=text` | `0x20c0a1`（同上，沒有 MULTI_LINE） | QWERTY，Enter 是「下一欄」 | 有 | 有 | 有 |
| `type=search`、`autocorrect="off"` | `0x2040a1`（沒有 AUTO_CORRECT） | QWERTY | 有 | 無 | 有 |
| `type=email` | `0xd1`（TEXT、WEB_EMAIL_ADDRESS） | QWERTY，逗號鍵換成 `@` | 無 | 無 | 無 |
| `type=url` | `0x11`（TEXT、URI） | QWERTY，逗號鍵換成 `/` | 無 | 無 | 無 |
| `type=password` | `0xe1`（TEXT、WEB_PASSWORD） | QWERTY 加數字列，沒有 emoji 鍵，麥克風停用（ev/19-field-password.png） | 無 | 無 | 無 |
| `type=number` | `0x6002`（NUMBER、DECIMAL） | 數字鍵盤：1–9、0、`,` `.` `-`、空白 | — | — | — |
| `inputmode=numeric` | `0x4002`（NUMBER） | 同 `type=number` 的數字鍵盤 | — | — | — |
| `type=tel` | `0x4003`（PHONE） | 電話鍵盤：數字鍵標 ABC…WXYZ，`* #`、`0 +` | — | — | — |

## 給後續票的結論

- 英文模式 ADR 與 `delete_before`（票 21）：
  - Gboard 在 Chrome 裡不留組字區，字母逐個直接上屏；修正、兩下空白、選建議後的標點、還原，都是「刪掉游標前 N 字、再上屏」。`delete_before` 加上屏一串就重現得了本票量到的每種改寫。
  - 切詞：`.` 是詞界（`gogle.com` 打 `.` 就修 `gogle`）；詞裡出現 `@` 就整串不給建議、不修；撇號在詞內（`its` → `It's`）。
  - 預測含下一個詞：空白後建議列出下一個詞（Next-word suggestions 預設開）。
- 自動修正（票 25）：
  - 觸發是空白、逗號、句點、Enter 與點建議；打字中途不改。
  - 退格不會一鍵還原：第一下只刪空白、把原字放到建議列左格，點了才還原，還原的同時學會原字、之後不再修。brief 寫的「退格可還原」若要一鍵還原，是比 Gboard 多做一步，要在票裡定採誰的做法。
  - 不修：含數字、全大寫、詞裡有 `@`、欄位沒有 AUTO_CORRECT；句中首字大寫照修、保留大寫。
  - 原樣上屏一次就學、學到的字還會當修正目標（`seperate` → `seperate2`）。QIM 的英文使用者詞有 2 次的出現門檻，打錯一次不會這樣，這點比 Gboard 保守。
- 欄位類型（票 26）：email、網址欄是 QWERTY 加 `@` 或 `/` 鍵，沒有建議、不修、不大寫；數字欄與 `inputmode=numeric` 開數字鍵盤，`tel` 開電話鍵盤；`search` 有建議但不修；密碼欄加數字列、沒有建議。殼可以照上表從 `inputType` 的類別、變體與 AUTO_CORRECT 旗標判斷。
- 智慧標點（票 27）：
  - Gboard 英文與粵語鍵盤都沒有成對符號自動配對，左右半邊各一顆鍵，英文引號是直引號。spec 寫的「插入一對還是左右輪流，照 Gboard 實測定」照不了 Gboard，要另找成熟產品當依據，或請使用者定。
  - 兩下空白：約 1 秒內、前一字不是標點時，刪一個空白、上屏「. 」，下一字自動大寫；數字後照做；退格不還原。

## 沒解開的

- 舊機（清資料後）頭幾分鐘：`teh`、`Teh` 中格不粗體、空白不修；`Freind`、`Tommorow` 打 Enter 不修，之後這兩個字也不再修。新 AVD 上 `teh` 中格粗體、會修，Enter 試了 `neccessary`、`existance`、`posession`、`adress`、`suprise`、`peice` 都會修（`begining` 中格不粗體、不修）。推測是清資料後語言模型還沒載好，或舊機的學習狀態，沒有再追。
- Gboard 學到 `Qimz` 是測試頁直接改欄位值造成的；Gboard 怎麼看待宿主改值，沒有在真實 App 裡驗證。
- 只測了 Chrome；原生 `EditText` 裡 Gboard 是否留組字區沒有測（共用模擬器的原生 App 要登入帳號才能用，新 AVD 沒有試）。
