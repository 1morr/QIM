# 26: 欄位類型

Status: done
Blocked by: 25
讀: brief §4.7、§8（密碼欄）；plan §2.1（`Options`）、§3.1；票 19；spec〈共同規則〉

## What to build

殼告訴核心輸入框的類型（介面由票定，照 Android `EditorInfo.inputType`、iOS `keyboardType` / `UITextContentType` 的慣例，採用誰的寫在 Comments）：

- email、網址：預設英文模式並關自動修正。
- 數字欄：開數字頁。
- 網頁從 `input` 的 `type` / `inputmode` 讀；密碼欄現有的無痕照舊。

## 驗收

- [x] Playwright：email 欄位預設英文且不修正；網址欄同；數字欄打開是數字頁；一般欄位照舊。
- [x] 介面寫進 plan §2.1；`cargo xtask check` 綠燈。

## Comments

- 做到（開工 `0438220`）：
  - 核心：`FieldKind`（`Text`、`Email`、`Url`、`Number`、`Password`，`name()` 與 `FromStr` 共用一份名字）與 `Session::set_field(engine, user, field) -> Snapshot`，殼每次聚焦、給前文之前呼叫；`Session::private()`（無痕或密碼欄）取代各處直接讀 `Options.incognito`；`Inspection::field`。email、網址、密碼欄切英文直通、離開到一般或數字欄還原進去之前的中英；使用者在這三種欄位切到英文模式時也不自動修正、不自動大寫，密碼欄也不出建議；密碼欄不學、不讀前文（使用者的無痕開關不動）；換欄位丟掉上一個欄位的組字（不上屏）、正在打的串打完。
  - 佈局：版本的 `numberPage`（只加不改，Schema、驗證都加）與 `Keyboard::set_number_field`；Gboard 速成兩份佈局指到 `numbers`。
  - 網頁：工具列「欄位」選單模擬宿主的 `input`（選項帶 `data-type`、`data-inputmode`），`web/src/field.ts` 照 Chrome 轉給 Android 的規則讀成欄位類型（`inputmode` 蓋過 `type`，`type=password` 蓋不掉）；輸入框仍是同一個 `textarea`，因為 `type=email`、`type=number` 的 `input` 沒有游標 API；debug 面板加「欄位」一行，前文的「無痕」改寫成「無痕或密碼欄」；`qim-wasm` 加 `setField`、`Layout.setNumberField`、`Inspection.field`，`incognito` getter 改成 `private`。
  - 文件：plan §2.1（`FieldKind`、`set_field`、`private`、〈欄位類型〉一節取代原本的 `Options.field` 草稿、密碼欄的建議、無痕兩處補密碼欄）、§2.4（不寫學習資料的條件）、§2.6（`numberPage`）、§3.1（網頁的「欄位」選單、密碼欄不給前文）；brief §4.7 欄位類型一條。
- 介面採用誰的慣例：分法照 Android `EditorInfo.inputType` 的類別與變體（事實 P24；密碼和 email、網址同是一種變體），對到 iOS 的 `keyboardType` 與 `isSecureTextEntry`；聚焦時給、不放 `Options`，同 Android `onStartInput` 每次帶 `EditorInfo`。預設英文、用直通、離開還原照 Trime（`KeyboardWindow.kt` 的 `onStartInput`，commit `2e2b4d5` L340–393：email、密碼欄設 `ascii_mode`、換英文鍵盤，數字欄換 `number` 鍵盤，離開時還原），網址照青簡 Android 對 URI 欄直通（`docs/research/qingjian.md`）；沒有建議、不修不大寫照 AOSP LatinIME `InputAttributes`（email、URI、密碼欄 `mShouldShowSuggestions = false`）；`numberPage` 照 Trime 的 `number` 鍵盤、fcitx5-android 數字欄換 `NumberKeyboard`。Gboard 在粵語速成下不切英文（M48），不照它。搜狗、百度、微信、微軟、iOS 系統鍵盤查不到一手來源。
- 測試：核心 `crates/qim-core/tests/field_kind.rs` 14 個（三處欄位判斷各拿掉時對應的測試會紅，已還原、沒 commit）；佈局 `crates/qim-layout/tests/number_page.rs` 3 個、`tests/validate.rs` 加 `numberPage` 指錯頁、`crates/qim-databuild/tests/gboard.rs` 加 1 個；網頁單元 `web/test/field.test.ts` 5 個；Playwright `web/e2e/field-types.spec.ts` 7 個（email、網址各一：預設英文直通、英文頁、自動修正開著打 `teh ` 不修、沒有建議、面板讀回欄位、回一般欄是中文；`type=number` 與 `inputmode=numeric` 各一：數字頁、中英不動；密碼欄：英文、沒有建議、面板前文是不讀、無痕開關不動；一般欄照舊；實體鍵盤 email 欄）；`e2e/debug-panel.spec.ts` 的無痕文字照新字改。截圖 `.scratch/m4/evidence/26-field-{email,url,number,password}.png`。
- 亂按一輪（Playwright 手機 viewport，腳本不留，當時是英文模式的版本）：組字中換到 email、email 裡切回中文組字中換到網址、數字欄開數字列再回一般欄、英文模式下進出數字欄、五種欄位快速連換、實體鍵盤組字中換到密碼欄再打字。都照規則（組字丟掉、中英切換與還原、頁跟著換），console 沒有錯誤。改成直通後重跑 e2e 與截圖確認畫面。
- `cargo xtask bench --check`（`c6e8b4f`）：各集合答錯、修正都等於 baseline（yue-wiki 941／3239、zh-wiki 979／3493、common-voice-yue 892／3249、hkcancor 572／2148，前文、打錯碼與觸控集合也同），沒有翻盤，固定題 84/84；中文路徑沒動。每鍵 p95（原生）：空白 全部 181 µs、第 20 碼起 193 µs、選開頭一段 3.89 ms；學過 313 µs／422 µs／5.55 ms；含萬用鍵的也都在上限內。
- `cargo xtask bench --learning`：開工 `0438220`（另開暫時 worktree 量）與改後逐行相同，例如 yue-wiki 學習關／開 3239／3216、hkcancor 2148／1719、hkcancor·前文 1512／1179；`writes_learning` 只多了密碼欄，基準是一般欄。
- code-review（`0438220` 起，Standards／Spec 兩軸）已處理：桌面的英文改成英文模式推翻了「桌面的英文是直通」→ 問使用者，改成一律英文直通（Trime、青簡、LatinIME 都是直通或沒有建議），email 欄點建議補空白、列拼錯修正的問題跟著不在預設路徑；多個行為塞在一個測試裡 → 拆成一個行為一個；「換欄位學到上一個欄位打完的串」沒測 → 補；`FieldKind` 與字串的對照寫兩次 → 核心 `FieldKind::name()` 共用；`field.ts` 寫了沒人讀的 `data-field` → 刪；大小寫不分混在「不認得的值」測試 → 拆開；plan 的無痕兩處沒補密碼欄 → 補；`starts_in_english` 同時管「不修不大寫」→ 改名 `wants_english`；沒附 bench → 補。
- code-review 未處理：`Session::private()` 和 `TypedRun` 的欄位 `private` 同名不同義（`TypedRun` 是內部型別，沒改）；`ContextState::Incognito` 與面板的 `incognito` 沒改名成「不讀」（改名動到網頁與既有 e2e，面板文字已寫「無痕或密碼欄」）；`mode_before_field` 是 `(bool, bool)`（同 `ascii`、`english` 兩個既有欄位，沒改成三態）；使用者在 email、網址欄自己切到英文模式時，點建議仍補空白、建議列仍列拼錯的修正（不在預設路徑，LatinIME 的 `shouldInsertSpacesAutomatically` 沒查證）；`fieldKind` 只在 `inputmode` 是 email、url、numeric、decimal、tel 時蓋過 `type`，`inputmode=text` 蓋不蓋 `type=email` 沒查證 Chrome。
- 決定：email、網址、密碼欄預設英文用英文直通，不用英文模式：桌面的英文是直通、核心分不出殼是觸控還是桌面，照 Trime、青簡、LatinIME（使用者定）
- 決定：密碼欄也預設英文，推翻票面「密碼欄現有的無痕照舊」只做無痕（使用者定）
- 決定：密碼欄不出建議（英文模式也不出），照 LatinIME 與 Gboard（M48），ADR 0008 留給本票定（Agent 定）
- 決定：欄位類型是聚焦時給的 `Session::set_field`，不放 `Options`（plan 原本的草稿），因為它是每個欄位的資訊、殼改設定時讀出 `options()` 改一項就分不出換了欄位；分法照 Android `inputType` 的變體，密碼算一種，核心把密碼欄當無痕、殼不另開 `Options.incognito`（Agent 定）
- 決定：離開要英文的欄位時還原進去之前的中英，在那種欄位裡切過的不留到下一個欄位，照 Trime（Agent 定）
- 決定：email、網址、密碼欄切到英文模式時也不句首自動大寫，票只寫關自動修正，照 LatinIME 與 Gboard（M48）（Agent 定）
- 決定：換欄位丟掉上一個欄位的組字、不上屏（殼要上屏就在失焦時先上屏）（Agent 定）
- 決定：數字欄核心的中英不動，只由殼開佈局新加的 `numberPage`（照 Trime 的 `number` 鍵盤）；電話（`type=tel`、`inputmode=tel`）也算數字欄（Agent 定）
- 決定：網頁用工具列的「欄位」選單模擬宿主的 `input` 屬性，輸入框仍是 `textarea`（`type=email`、`type=number` 的 `input` 沒有游標 API，上屏用不了）（Agent 定）
Session 紀錄：欄位類型：殼聚焦時給 `set_field`，email、網址、密碼欄預設英文直通、離開還原、不修不大寫，密碼欄不出建議並當無痕，數字欄開佈局的 `numberPage`；網頁加「欄位」選單；bench 不變
