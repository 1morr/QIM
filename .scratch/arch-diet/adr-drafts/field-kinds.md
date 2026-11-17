# 欄位類型由殼聚焦時給；email、網址、密碼欄預設英文直通

> 草稿（arch-diet 票 02 從 plan〈核心〉〈介面〉〈欄位類型〉搬來），使用者確認後由票 06 編號寫進 `docs/adr/`。行為由 `crates/qim-core/tests/field_kind.rs`、`crates/qim-layout/tests/number_page.rs`、`web/test/field.test.ts` 說明。

殼每次聚焦輸入框時呼叫 `Session::set_field(FieldKind)`。它是每個欄位的資訊、不是使用者的設定，所以不放在 `Options`：殼改設定時讀出 `options()` 改一項，放進去就分不出「換了欄位」。分法照 Android `EditorInfo.inputType` 的類別與變體（事實 P24）：`Text`、`Email`、`Url`、`Number`、`Password`；iOS 對應 `keyboardType`（`.emailAddress`、`.URL`、數字類）與 `isSecureTextEntry`；網頁照 Chrome 轉給 Android 的規則從 `input` 的 `type` 與 `inputmode` 讀。

- **email、網址、密碼欄切到英文直通**，離開到一般欄、數字欄時還原進去之前的中英；在那種欄位裡切過的不留到下一個欄位。依據：Trime 在 email 與密碼欄設 `ascii_mode`、離開時還原（`KeyboardWindow.kt` `onStartInput`，commit `2e2b4d5` L340–393）；青簡 Android 對信箱、網址、密碼欄直通（`docs/research/qingjian.md`）；AOSP LatinIME 在這三種欄位不給建議（`InputAttributes` 的 `mShouldShowSuggestions`）。不用英文模式：桌面的英文是直通，核心分不出殼是觸控還是桌面（見下面〈Considered Options〉）。
- 使用者在這三種欄位切到英文模式時，也不自動修正、不句首自動大寫（LatinIME `InputAttributes`；Gboard 在 email 與網址欄不修不大寫，事實 M48）；email、網址欄照出建議。
- **密碼欄**照無痕不學、不讀前文，英文模式也不出建議（LatinIME、Gboard 的密碼欄沒有建議列）；使用者的 `Options.incognito` 不動，速成的輸入碼與候選不會顯示在畫面上。
- **數字欄**核心的中英不動，殼開佈局的 `numberPage`（照 Trime 換 `number` 鍵盤、fcitx5-android 換數字鍵盤）。

## Considered Options

- 照 Gboard：Gboard 在粵語速成下不切英文（事實 M48），不照它。
- 在這三種欄位用英文模式：桌面的英文是直通，核心分不出殼是觸控還是桌面。

## Consequences

- 觸控在 email 欄打開頭不提示學過的 email（點地球鍵兩下進英文模式才有）；直通打的 email 照學、照樣出在中文組字。
