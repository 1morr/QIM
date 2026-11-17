# 24: 英文模式與英文建議列

Status: done
Blocked by: 14, 21, 22, 23
讀: brief §4.7；plan §2.1；spec〈細節〉〈英文模式〉〈英文建議列〉；票 21 的 ADR；票 14；spec〈共同規則〉

## What to build

觸控英文頁照票 21 的 ADR 進英文模式：

- 字母進英文組字，建議列出前綴補完：英文詞典加英文使用者詞，照使用頻率排；沒到門檻的英文使用者詞不出（票 14）。
- 空白、標點上屏；英文模式上屏的串學成英文使用者詞。
- Shift 與句首自動大寫（開關，預設關，brief §7）。
- 桌面的英文仍是直通、沒有建議列（spec〈細節〉）。
- 票 14 留下：上限淘汰把打過一次的詞典詞也當沒到門檻（`UserModel` 看不到英文詞典），和「詞典有的詞不受門檻」不一致；接英文建議列時決定要不要讓淘汰知道詞典（或記下）。

## 驗收

- [x] 核心測試：前綴補完的排序、門檻、句首大寫開關。
- [x] Playwright 手機 viewport：按中英鍵切到英文鍵盤打 `rrrrrr` 一次後，打 `rrr` 建議列沒有它，打第二次後有；v2 匯入的 email 打開頭字母，英文建議列就有它；截圖。
- [x] 實體鍵盤的英文直通行為不變；`bench --check` 不變；plan §2.1 更新；`cargo xtask check` 綠燈。

## Comments

- 做到（開工 `a940da8`）：
  - 核心：地球鍵另開 `Key::ToggleEnglish`（`Key::parse("toggleEnglish")`），`Session` 多一個 `english`；英文模式的字母、數字、標點與空白吃下、逐字上屏，退格、Enter、方向鍵交還宿主。建議列在 `english_mode.rs`（`suggestions`）：正在打的串的前綴補完，學過的詞加分後和詞典頻率合比（見下面 2026-11-17 那條），最多兩頁，照打的大小寫改寫。點建議用 `delete_before` 換掉打的詞、補空白；`TypedRun` 分開追蹤與學（`private`、`partial`），無痕與學習關掉照樣有建議。`Options::auto_capitalize`（預設關）與 `Snapshot.auto_shift`、`Session::auto_shift`；`qim-ffi` 跟著加欄位。
  - 佈局：`Keyboard::set_auto_shift`（由假變真開一次 Shift、由真變假關掉，使用者點掉的不開回來）。
  - 網頁：地球鍵與切換鍵盤選單送 `toggleEnglish`；設定加「句首自動大寫」；實體鍵盤的 `-` `=` 在任何英文狀態都是字元（`toKey` 的第三個參數改成 `ascii`），英文模式打得出 `-`；`render` 與給前文之後把 `autoShift` 交給觸控鍵盤。
  - plan §2.1（`Key`、`Options`、`Snapshot`、`ToggleAscii`、英文模式整段、英文使用者詞的兩處）與 §2.4〈條目上限〉改寫；brief 不必改（§4.7、§7 已寫）。
- 測試：核心 `crates/qim-core/tests/english_mode.rs` 21 個（切換、逐字上屏、排序、門檻、大小寫、v2 匯入的 email、點建議與空白、退格撤銷、無痕、句首大寫、游標移走）；佈局 `crates/qim-layout/tests/keyboard.rs` 加 1 個；Playwright `web/e2e/english-mode.spec.ts` 6 個（手機 viewport），面板讀回來源。截圖 `.scratch/m4/evidence/24-english-mode-{once,twice,email,auto-shift}.png`。
- `cargo xtask bench --check`（在 `d843f06` 跑；之後的修正只動英文模式的路徑）：各集合答錯、修正都等於 baseline（yue-wiki 941／3239、zh-wiki 979／3493、common-voice-yue 892／3249、hkcancor 572／2148），12 個集合都沒有翻盤，固定題 84/84。每鍵 p95（原生）：空白 全部 183 µs、第 20 碼起 196 µs、選開頭一段 3.96 ms；學過 311 µs／422 µs／5.44 ms，都在〈每鍵延遲上限〉內。
- 亂按一輪（spec〈共同規則〉）：組字中點地球鍵、英文模式快打與退格、點建議後打句號、英文模式用實體鍵盤打 `wor-ld`、退格、方向鍵、單按 Shift 回中文、無痕中打字、句首自動大寫在 `. ` 與退格之間切換、連點地球鍵五下。找到的問題是 code-review 找到的同一批（游標移走後建議與補的空白沒作廢），先寫成會紅的核心測試再修。console 只有原本就有的 favicon 與線上更新索引 404。
- code-review（`a940da8` 起，Standards／Spec 兩軸）已處理：游標移走（前文不以串結尾）時建議收起、點不到舊建議；點建議補的空白在游標移走、前文作廢、換欄位時丟掉；句讀換掉補的空白時只拿掉前文、不把串標成開頭不明（之前 `hello.` 之後的 `com` 沒有建議）；「刪 N 字再上屏」三處收成 `commit_after_deleting`；`ascii || english || inline` 收成 `in_english`；四元組改成 `UsedWord`；`ENGLISH_SUGGESTION_LIMIT` 沒有 crate 外的呼叫者，不匯出。名詞表照草稿補上英文模式、正在打的串、英文建議列、句首自動大寫，改地球鍵、英文使用者詞、出現門檻（使用者同意）。
- code-review 未處理：(1) `ascii` 與 `english` 兩個互斥的 bool 可改成一個 enum：判斷題，動到的地方多，沒改。(2) 佈局的 `Output::ToggleAscii` 現在送 `ToggleEnglish`，名字不符：佈局動作叫 `toggleAscii` 是票 23 使用者同意的格式，沒改。(3) `end_action` 分開收 `commit` 與 `delete_before`：照舊。(4) 點建議補空白、句讀換掉空白、大小寫跟打的、`!` 也算句尾、英文直通的 `-` `=` 當字元，票上沒寫，是定「點建議做什麼」與英文模式打得出 `-` 時的選擇，記在下面的決定。(5) 無痕時殼不給前文：剛聚焦時不知道游標前已有的字（建議只看聚焦後打的）、換行之後不自動大寫，plan 已寫明。
- 決定：名詞表加英文模式、正在打的串、英文建議列、句首自動大寫，改地球鍵、英文使用者詞、出現門檻（使用者同意）
- 決定：地球鍵另開一種鍵 `Key::ToggleEnglish`，殼不另開欄位分辨英文直通與英文模式，都看 `Snapshot.ascii`（Agent 定）
- 決定：組字中按地球鍵同單按右 Shift、照「組字中單按 Shift」的設定，先上屏時進英文模式，沿用票 23 選單與地球鍵的決定（Agent 定）
- 決定：建議從第一個字母起、最多兩頁；和打的一字不差的不列（Agent 定）
- 決定：建議照打的大小寫改寫：開頭大寫就開頭大寫，兩個字母以上全大寫就整個大寫（Agent 定）
- 決定：點建議換掉打的詞並補一個空白，緊接著的空白吃掉、句讀與右括號先刪掉那個空白，照 Gboard（M47、M48；實測只有逗號，擴到 `. , ; : ! ? ) ] }`）（Agent 定）
- 決定：句首自動大寫由殼讓觸控 Shift 亮起（`Snapshot.auto_shift`），核心不改字母，句首也打得出小寫；`!` 同 `?`，`e.g.` 這類縮寫後照樣大寫（Agent 定）
- 決定：票 14 留下的淘汰問題記下不改：淘汰打過一次的詞典詞只少了學習的排前，要改得讓公開的 `UserModel::merge` 收英文詞典或改格式；寫在 plan §2.4〈條目上限〉（Agent 定）
- 決定：網頁實體鍵盤的 `-` `=` 在英文直通、英文模式也當字元，不轉翻頁（Agent 定）

- 2026-11-17 排序改版（協調者轉使用者的決定）：原本做成兩層（學過的一律排前、再來詞典），打過一次的冷門詞會壓過 the、hello。改成學過的加分後和詞典頻率合比。
  - 查證 LatinIME（事實 M50）：它不加分，使用者歷史的次數換成和主詞典同一刻度的機率（1 次 111、20 次 148），同一個詞取兩邊較大的。照它的話，AOSP 詞表的 helmet（`f=111`）打一次名次不動，要打約 20 次才進 `hel` 的前 3，和「打過一次就在前 3 格」對不上。
  - 所以照 LatinIME 的刻度加分：分數 = 詞典 `f`（詞典沒有的用 LatinIME 的歷史機率 `223 + 8.589 × log2(次數 ÷ 8192)`）+ 學過的 `8.589 × log2(64 × 次數)`（1 次 +51.5、5 次 +71.5、20 次 +88.7）。64 倍落在 AOSP 詞表算出的區間中間：`hel` 第 3 名比 helmet 高 31，`t` 第 3 名比 tumors 高 97。
  - 測試：拿掉固定兩層順序的 `learned_dictionary_words_come_before_the_rest_and_the_most_used_first`，換成三個用 AOSP 詞表真實 `f` 的核心測試（打過一次的 helmet 進 `hel` 前 3、打過一次的 tumors 排在 the to this that they 後面、helium 打得越多越前到第一），加一個用真的資料包的 Playwright（helmet 進前 3、tumors 不進 `t` 前 3）。
- 決定：英文建議列學過的詞起始加分 64 倍（打 1 次 +51.5），LatinIME 沒有這一項（使用者同意）
- 決定：英文建議列學過的詞加分後和詞典頻率合比，不分兩層，照 LatinIME（使用者定）
- 決定：LatinIME 沒有加分，照它的刻度另加 `8.589 × log2(64 × 次數)`，讓打過一次的詞典詞進前 3、又不壓過常用詞；詞典沒有的詞底分用 LatinIME 換算的歷史機率（Agent 定）
Session 紀錄：英文模式接上：地球鍵送 `ToggleEnglish`、逐字上屏、英文建議列（詞典加英文使用者詞的前綴補完，學過的照 LatinIME 刻度加分後和詞典頻率合比，點了用 `delete_before` 換詞）、句首自動大寫（`auto_shift`）；bench 不變
