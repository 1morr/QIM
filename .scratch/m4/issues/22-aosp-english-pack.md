# 22: 英文資料包換成 AOSP 英文詞表

Status: done
Blocked by: 18, 18b
讀: brief §4.3、§4.7、§5、§11；plan §2.4（規則 4）、§4.4；`docs/research/spatial-correction-swipe.md` §6；spec〈共同規則〉（資料來源）

## What to build

英文資料包改用 AOSP LatinIME 的 `en_US_wordlist.combined.gz`（Apache-2.0，逐詞頻率與 bigram、含 `z` 開頭的詞），取代 `scowl-en-us`；原定的 aosp-dictionaries experimental 詞典把這份詞表整個併進去、另加授權沒寫版本的 Leipzig 詞頻（事實 T39），協調者 2026-11-15 定改用 LatinIME 原表；同一份給英文模式與中文狀態的規則 4：

- 來源釘進 `data/sources.toml`，資料包的 `LICENSE`、`NOTICE` 跟著換。
- `z` 只在中文組字的比對時濾掉（plan §4.4）。
- 用 `bench` 確認中文狀態的第 2 位不會被多搶：規則 4 觸發的題數前後並列。

## 驗收

- [x] `cargo xtask data --verify` 決定性；`LICENSE`、`NOTICE` 只寫用到的來源。
- [x] `bench --check` 前後對照與規則 4 觸發題數附在 Comments，變多的說明原因。
- [x] Playwright：中文狀態打 `receive` 第 2 位是 receive，打 `recieve` 不出英文候選。
- [x] plan §4.4 與 brief 相關段落更新；`cargo xtask check` 綠燈。

## Comments

- 來源查證（協調者要求，commit 資料包前）：aosp-dictionaries 的 experimental 詞典有三層來源：Leipzig 語料的詞頻（條款「下載的語料 CC BY」沒寫版本、沒記用哪份）、hunspell en_US 的過濾、AOSP LatinIME 舊詞表整份併入（66,178 詞頻率照抄）。寫成事實 T39（`594aa9b`），用 AskUserQuestion 問，協調者代答，選直接用 LatinIME `127336e` 的 `dictionaries/en_US_wordlist.combined.gz`（Apache-2.0，repo 的 NOTICE）。票名、票面、spec〈範圍〉〈細節〉與票序表、plan §4.4、brief §5.2 同一個 commit 改。
- 做到：`data/sources.toml` 換成 `[source.latinime]`，googlesource 只給 base64，來源檔加 `encoding = "base64"`（下載後解開、sha256 釘解開後的）；`qim-databuild` 收 `f ≥ 1`、沒標 `possibly_offensive`／`flags=offensive`／`not_a_word` 的詞（含 `z`、撇號、連字號），bigram 與 shortcut 不收（ADR 0008）；資料包 `aosp-en-us` 帶逐詞頻率給票 24，`EnglishWords::frequency`，資料包格式 9；`--verify` 加抽查「英文資料包有 zero 與頻率」；web fixture 重產；實驗腳本裡的舊資料包路徑跟著改。
- `cargo xtask data --verify`：通過，決定性十份 sha256 相同（英文 `763ad70f3c56…`）。英文：詞表 160,715 詞，可能冒犯的 1,301、只當 shortcut 的錯拼 47、`f=0` 的 72 不收，依小寫歸併後收 156,636 詞（舊的 85,959 詞）。`aosp-en-us` 的 LICENSE 只有 LatinIME 的 NOTICE（Apache-2.0），NOTICE 寫釘的 commit、檔的 sha256 與取用、改動方式；`tests/run.rs` 擋沒有 OpenCC 的 Apache 全文。
- `cargo xtask bench --check`（前：`594aa9b` 的資料包；後：本票）：各集合答錯數、修正次數、依長度分組都相同，翻盤清單全部「沒有翻盤」，固定題 84/84；唯一差異是個人集修正 1,463 → 1,462。每鍵延遲後全在上限內（空白全部 p95 227 µs、選開頭一段 4,671 µs；含萬用鍵選開頭一段 4,791 µs）。前的那次在含萬用鍵、選開頭一段 p95 5,331 µs 超過 5 ms，量的期間我在同台機器編譯，後在不干擾下重跑全綠。
- 規則 4 觸發題數（`scripts/experiments/english-rule4/`，每題打完整題看第一頁有沒有英文候選）：全部 54 → 87 題（在第 2 位 53 → 85），個人集 47 → 77 題（4,517 題的 1.0% → 1.7%），公開集合只多 1 題（common-voice-yue 第 1108 行的 `METV`，在打錯、觸控集合也出現）。變多的原因：LatinIME 詞表大（156,636 對 85,959 詞），多出來的是 ESDB size 60 不收的冷僻詞與縮寫，AOSP 頻率都在 18–89（`the` 222、`receive` 137），速成 3–4 個字母的碼撞到的機會多。英文候選不搶首選，首選與修正次數不變；被擠的是原本第 2 位的中文候選往後一格。
- 待使用者：要不要讓中文狀態只出頻率 ≥ 90 的英文詞（87 → 27 題），要的話另開票。
- Playwright：`e2e/english.spec.ts` 新增「打 receive 第 2 位是 receive、來源是英文；打 recieve 沒有英文候選」，debug 面板讀回來源「英文、排位規則 4」；截圖 `.scratch/m4/evidence/22-english-receive.png`。亂按一輪：`zero`（中文狀態 `z` 是萬用鍵、沒有英文候選）、`englishh` 退格後選 2 上屏 English、Shift 切英文直通打 ` zoom `、切回中文連打 `receivegoat`（不是完整單字、沒有英文候選），讀回正確。
- 全部檢查：`cargo xtask check` 通過（含 `cargo test --workspace`，e2e 287 passed）。
- code-review（`594aa9b` 起，Standards／Spec 兩軸）：已處理：四份 web fixture 還是格式 8（`cargo test` 會紅，重產）；`in_english_dictionary` 把含 `z`、撇號的詞也算詞典有，學過的 `doesn't` 這類英文使用者詞因此不給前綴補完（改成只算整個詞都是方案字母的，補會紅的測試 `dictionary_words_the_scheme_cannot_type_still_get_completion`，plan §2.4 規則 5 補一句）；實驗腳本的舊資料包路徑；development.md「下載不整份讀進記憶體」補 base64 的例外；spec〈細節〉與 `data-sources.md` 的舊說法。這個修正只影響學過的英文使用者詞的補完，`bench` 與 `--learning` 都不學英文串，所以沒重跑 bench。未處理：頻率先進資料包、目前只有 verify 與實驗在讀（審查標 Speculative Generality），保留給票 24，見下面的決定；兩個測試檔各有一段 gzip 樣板（不同測試檔，不抽）。

Session 紀錄：查出 aosp-dictionaries experimental 詞典來源混雜（T39），協調者改用 AOSP LatinIME 的 en_US 詞表建 `aosp-en-us`（帶頻率、收 z 詞），規則 4 觸發 54 → 87 題、首選與修正次數不變。
決定：英文資料包來源改用 AOSP LatinIME `127336e` 的 `en_US_wordlist.combined.gz`，不用 aosp-dictionaries 的 experimental 詞典（Agent 定）
決定：LatinIME 不建議的詞（可能冒犯、`not_a_word` 的錯拼、`f=0`）不收，撇號、連字號、`z` 的詞照收給英文模式（Agent 定）
決定：資料包本票就帶逐詞頻率（格式 9），票 24 的建議列排序不必再升一次格式（Agent 定）
決定：googlesource 只給 base64，`sources.toml` 加 `encoding = "base64"`、sha256 釘解開後的，不改用第三方鏡像（Agent 定）
決定：中文組字的「英文詞典有」只算整個詞都是方案字母的詞，含 `z`、撇號的詞典詞照樣進前綴補完（Agent 定）
決定：規則 4 不加頻率門檻，觸發變多只說明原因、交給使用者決定，協調者定（Agent 定）

