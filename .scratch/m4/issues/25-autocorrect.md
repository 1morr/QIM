# 25: 英文自動修正

Status: done
Blocked by: 24, 18c
讀: brief §4.7、§7；spec〈細節〉〈英文模式〉；票 19、21；spec〈共同規則〉

## What to build

英文模式的自動修正（開關，預設關，brief §7）：

- 修正的時機照票 19 的 Gboard 實測；用 `delete_before` 換掉打錯的詞。
- 對原樣讀得通的要保守；照 AOSP 不自動修的條件（含數字、全大寫、email 字元等）。
- 退格可還原；還原時不學修正字（原樣的那串學不學，由票定）。
- 英文的容錯開關（預設開，brief §7）照票 21 的 ADR。

## 驗收

- [x] 核心測試：AOSP 不修的各條件；退格還原後不學修正字。
- [x] Playwright：預設不修正 `teh`；打開自動修正後英文模式打 `teh` 修正成 `the`，退格還原；截圖。
- [x] `cargo xtask check` 綠燈。

## Comments

- 做到（開工 `9a35379`）：
  - 核心：`Options` 加 `autocorrect`（關）與 `english_corrections`（開）。英文容錯：建議列也列一處拼錯（Damerau：換、對調、漏打、多打一個字母或 `'` `-`）的英文詞典詞與過了出現門檻的英文使用者詞（`english_mode::corrections`，種類 `SpanKind::EnglishCorrection`，面板標「英文修正」）。自動修正：串打完那一下（空白、`,` `;` `:` `!` `?` 這類句讀、觸控送文字的標點、Enter）換掉明確的一處拼錯（`english_mode::autocorrection`），會換成的字排建議列第一；緊接著退格還原（`Session::revert_correction`）。Enter 是 `handled` 為假而帶 `delete_before` 與 `commit`，殼先換詞再交還。
  - 網頁：設定加「英文自動修正」「英文容錯」兩列；`host.ts` 加 `commitBeforeHost`，觸控（`hostDefault`）與實體鍵盤的交還路徑先照做快照帶的刪字與上屏；debug 面板加「自動修正：打完會換成…」一行與「英文修正」種類。`qim-wasm` 加 `setAutocorrect`、`setEnglishCorrections`、`Inspection.autocorrection`；`qim-ffi` 的 `handled` 註明例外。
  - 文件：plan §2.1（`Options`、`Snapshot.handled` 的例外與各平台怎麼做、英文容錯、自動修正、還原、和 Gboard 的差異）；事實 M51（LatinIME 的不修條件、`revertCommit`、whitelist）；實驗 `scripts/experiments/autocorrect-threshold/`（門檻與每鍵延遲）。brief 不必改（§4.7、§7 已寫自動修正、英文容錯與預設值）。
- 測試：核心 `crates/qim-core/tests/english_mode.rs` 加 16 個（預設不修、空白與句讀修、AOSP 不修的各條件、只修明確的、學過的不修、英文容錯關掉不修、修正列在建議列而自動修正排第一、退格還原不學修正字而學原樣、只有緊接著的退格還原、Enter 修、觸控標點修、修飾鍵夾在中間照樣還原、修正排在補完後、漏撇號、學過的串當修正對象、時限過了不還原、Enter 修的不還原），票 24 的三個補完測試改成關掉英文容錯；網頁單元測試 `web/test/host.test.ts` 加 1 個；Playwright `web/e2e/autocorrect.spec.ts` 4 個（手機 viewport，真的資料包）：預設不修 `teh` 而建議列有 the、打開後 `teh` 修成 `the` 並退格還原（面板讀回會修正成 the、種類英文修正）、之後不再修、觸控與實體鍵盤的 Enter 都先換詞再換行。截圖 `.scratch/m4/evidence/25-autocorrect-the.png`、`25-autocorrect-reverted.png`。全部 e2e 307 過（2 個照舊略過）。
- 門檻（`scripts/experiments/autocorrect-threshold/`，AOSP 詞表）：

  | 門檻（倍） | Gboard 22 個修對 | AOSP 47 條修錯 | 隨機錯字修對 | 隨機錯字修錯 |
  | ---: | ---: | ---: | ---: | ---: |
  | 2 | 18 | 6 | 61.2% | 9.1% |
  | 4（採用） | 17 | 4 | 58.9% | 6.7% |
  | 8 | 16 | 1 | 56.5% | 5.4% |

  門檻是在亂按找到 `helo` 被修成 held 之後，看了這張表才從 2 倍改成 4 倍：修錯少 2.4%、修對少 2.3%。日後有空間模型或真實的英文打錯語料時重量。
- 英文模式每鍵延遲（原生 release，空白學習資料，同實驗）：只補完 p95 224 µs、開英文容錯 321 µs、再開自動修正 404 µs，在〈每鍵延遲上限〉1000 µs 內。
- `cargo xtask bench --check`（`0144e75`）：各集合答錯、修正都等於 baseline（yue-wiki 941／3239、zh-wiki 979／3493、common-voice-yue 892／3249、hkcancor 572／2148，打錯碼與觸控集合也同），16 個集合都沒有翻盤，固定題 84/84；英文模式不在基準裡，中文路徑沒動。每鍵 p95（原生，機器有負載）：空白 全部 403 µs、第 20 碼起 347 µs、選開頭一段 4.14 ms；學過 795 µs／616 µs／6.66 ms，都在上限內。
- 亂按一輪（Playwright 手機 viewport，腳本不留）：修完退格三下再打字、修完按 ⇧ 一兩下再退格、修完切回中文再退格、`teh.` 加空白、退格還原後再打空白、`Teh`、Enter 修完再退格、`dont`、`helo wrold`、還原後點建議、關掉英文容錯、無痕中打 `wierd peice`、實體鍵盤在英文模式打字與退格。找到 `helo` 被修成 held（門檻 2 倍時 held 只比 help 高 9 分），先寫成會紅的核心測試再把門檻改 4 倍。console 沒有錯誤。
- code-review（`9a35379` 起，Standards／Spec 兩軸）已處理：超過退格撤銷時限才還原會留下修正字的學習 → 還原改成同一個時限（`expire_undo` 一起丟掉）；修飾鍵順手保住了剛點的建議（票外的行為改動）→ 拿掉；實驗沒留腳本、plan 與註解引用「票 25 Comments」→ 收成 `scripts/experiments/autocorrect-threshold/`；LatinIME 的行為沒有事實編號 → 加 M51；補五個講明的行為的測試（Enter 修的不還原、時限、`hel` 的排序、漏撇號、學過的串當修正對象）；去詞尾 `.` `'` 轉小寫兩處收成 `english_mode::kept_key`；`english_mode::Listing` 和 `lattice::Listing` 撞名改成 `Sources`；`Corrected.typed` 改名 `original`；離開英文模式時清掉 `Session::autocorrection`；(1) 名詞表沒加「英文容錯」「自動修正」「還原」、〈容錯〉那條的「設定裡的『容錯』開關指它」現在設定頁另有「英文容錯」→ 使用者確認草稿後寫進 `GLOSSARY.md`。
- code-review 未處理：(2) `autocorrection`、`corrections` 收裸的 `user_words` bool，沒併進 `Sources`：參數只兩三個，沒改。(3) 自動修正在列建議與打完時各算一次，結果一致，沒改成共用。(4) 門檻的結論只寫在實驗 README 與 plan，沒摘進 brief：是內部參數，brief §4.7 已寫「對原樣讀得通的要保守」。
- 決定：Enter 也觸發自動修正，照票 19 的 Gboard 實測（M47）：`handled` 為假時也可以帶 `delete_before` 與 `commit`，殼先換詞再交還 Enter；改契約只動網頁兩處（觸控與實體鍵盤的交還路徑），各平台的做法寫進 plan（Agent 定）
- 決定：學過的英文使用者詞（一年內打過，不論次數）與這個輸入框還原過的詞不自動修，照 LatinIME 與 Gboard 把使用者歷史當有效詞（M47、M51）；自動修正關著時打過一次的 `recieve` 打開後也不修（Agent 定）
- 決定：英文容錯不看鍵位鄰近，合 ADR 0008 的定義（「建議列也列拼錯的修正」，沒要求空間模型）；觸控點擊的按鍵機率接進修正要讓正在打的串逐字記機率，留給之後（Agent 定）
- 決定：修正是一處 Damerau 編輯的詞典詞與過了出現門檻的英文使用者詞，漏撇號與重複字母不扣分、換對調漏打扣一倍、多打非重複字母扣三倍且不自動修；排在補完之間再扣 64 倍（Agent 定）
- 決定：自動修正要贏過次一個修正 4 倍、至少 3 個字元、大寫字母超過一個不修（LatinIME 的 `isMostlyCaps`，比 ADR 0008 的「不是全大寫」嚴，只是少修、不算推翻）、有漏撇號的修正而最高的不是它時不修、不收 LatinIME 的 whitelist（Agent 定）
- 決定：退格還原的時限同退格撤銷（殼呼叫 `expire_undo` 後不還原），中間只按修飾鍵照樣還原；觸發的空白一起拿掉（Gboard 的子設定），句讀留著（LatinIME）；修正字學到的撤銷，換回的詞照常在打完時學（票上「原樣的那串學不學由票定」：學，同 Gboard 還原後教原字）（Agent 定）
- 決定：Enter 修的不還原：Enter 交還了宿主，核心不知道宿主有沒有換行，刪的字數會錯（Agent 定）
- 決定：自動修正會換成的字排建議列第一（Gboard 中格粗體），debug 面板另列「自動修正：打完會換成…」（Agent 定）
- 決定：名詞表加英文容錯、自動修正、還原，〈容錯〉補另見英文容錯（使用者同意）
Session 紀錄：英文自動修正（預設關）與英文容錯（預設開）：一處拼錯的修正列進建議列，串打完（空白、句讀、Enter）時換掉明確的修正，緊接著退格還原且不學修正字；Enter 改成交還宿主時也能帶上屏；bench 不變
