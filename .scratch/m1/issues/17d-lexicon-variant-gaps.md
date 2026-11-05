# 17d: 詞庫字形補漏與建置接線測試

Status: done
Blocked by: 17c
讀: 票 17c 全文與 Comments、票 17b Comments；brief §1.3、§4.1、§12（字形、OpenCC）；plan §4.2、§4.3；`docs/development.md`〈資料建置〉；spec〈共同規則〉

## What to build

M1 封存後補票的審查（2026-11-05，協調者派的獨立審查）找到 17c 的漏洞；使用者 2026-11-05 同意照審查建議修。審查的探測腳本在 `/private/tmp/claude-501/-Users-roxy-orca-projects-QIM/4808186f-7ee6-44a4-9688-748b966c8ba8/scratchpad/review-c/`（`probe.mjs` 查候選、`merge.py` 看 essay 合併、`mut/` 變異副本），可參考，不必照用。

1. **被 `t2hk` 單向換掉、使用者沒選過的不同碼字**（使用者原則：不同碼兩種都收，打哪個碼出哪個）：
   - 鉤、溼、擡、糉、柺：純異體，照 17c 的雙向並存寫法加進去（代表寫法用 `t2hk` 實際換成的那個字；鉤先查清楚被換成什麼——審查看到 `crnd` 首選「釦子」、`cind` 出「鈎子」；有疑義的照 17c 的做法問使用者）。現況：`crnd` 出不了「鉤子」、`ggen` 出不了「臺灣」。
   - 臺、菸：不能雙向（「台州」不可寫「臺州」、「煙霧」不可寫「菸霧」），照「着/著」的單向做法：在換字前記下原字，詞庫只為原文本來寫「臺」「菸」的詞另收原寫法（如「臺灣」「香菸」），不從「台」「煙」生成。使用者同意照協調者建議做；做法本票定，寫進 Comments。
   - 再掃一次 `t2hk` 對 rime-essay 與口語詞表單向換掉、碼不同、詞頻 ≥ 門檻的字，列成表放 Comments；表上新出現的、上面沒列的，照 17c 的規則分類（純異體→雙向；有別義→單向；罕用→不動），拿不準的用 AskUserQuestion 問。
2. **同頻重複相加**：rime-essay 有詞同時列兩種寫法且詞頻相同（14 組，如「巔峯／巔峰」9448、「審覈／審核」8908、「泄露／洩露」3267），轉換後撞在一起相加變兩倍（`lexicon.rs` `build` 的 `Entry::Occupied` 分支）。改成：兩筆詞頻相同時取最大，不同時照舊相加（261 組不同頻的是獨立計數，如「才是／纔是」）。先寫會紅的測試。
3. **單向「着→著」跨義項**：「合着」(1365) 併進「合著」(604，合寫的書)，義項不同。查 essay 裡會被併的「X着」（審查列了 合着、刻着、張着、親着、所着、舊着），義項不同的不併（另收），「所着」「舊着」這類疑似錯字照「顯着」處理或不動，結論寫進 Comments；plan / 票 17c 寫「同一個義項」的句子改正。
4. **NOTICE** 補寫「顯着」併進「顯著」（及本票新增的改動），LGPL 要說明對來源做了什麼。
5. **建置接線沒有測試擋**（審查已證實）：以下三個變異 `cargo test -p qim-databuild` 全綠——bigram 傳空的 `BothForms`、essay 改走 `S2hk`、拿掉 `correct()`（pipeline.rs 約 312、361 行，lexicon.rs 約 60 行）。補測試讓三個變異都變紅（例如 `tests/run.rs` 小語料放「羣／喫／裡／顯着」各一詞斷言結果；`verify::lexicon_forms` 加 bigram 檢查：雙向兩字 `log_prob(None, c)` 與抽樣 bigram 相等）。在 Comments 記錄雙向變異驗證（造違規會紅、改無關的不紅）。
6. **簡體模式**：`nbta` 出「随著」，應是「随着」。先查原因（`hk2s` 對單字「著」的處理、逐段轉換），有會紅的測試才修；修不了記成已知限制。
7. **文件**：plan §4.3「台／臺不換」改成照實際（`t2hk` 已把臺換成台，本票再補單向收原寫法）。progress〈偏差與決定〉記一行（使用者決定：照審查建議補漏），session 紀錄票欄寫「M2（補 M1 17d）」。

## 驗收

- [x] 核心或資料測試：`crnd` 出得了「鉤子」、`ggen` 首選或候選有「臺灣」、打「台州」的碼不出「臺州」；巔峯／巔峰詞頻不翻倍；合着不併進合著。
- [x] 上面三個接線變異各自讓 `cargo test` 變紅（Comments 附輸出摘要）。
- [x] NOTICE、plan §4.3、brief（如有）已更新；`cargo xtask data --verify`、`bench --check`（數字變了 `--update-baseline` 並在 commit body 寫理由）、`cargo xtask check` 綠燈；/code-review 跑完。

## Comments

### 查證：建詞庫實際換成不同碼的字（`GAPS=1` 跑 `scripts/experiments/lexicon-forms/`）

essay 與口語詞表的詞（詞頻 ≥ 100）經建置同一套轉換後逐字比對；同碼的換了也打得出，不列。改動前：

| 原字 → 換成 | 碼 | 詞數 | 詞頻和 | 本票怎麼處理 |
| --- | --- | ---: | ---: | --- |
| 臺→台 | gg→ir | 978 | 643045 | 單向（平臺、臺灣、舞臺…） |
| 菸→煙 | ty→fg | 98 | 53093 | 單向（抽菸、香菸、菸草…） |
| 溼→濕、鉤→鈎、柺→枴、囪→囱 | eg→ef、cr→ci、ds→dh、hk→he | 115、91、10、4 | 59666、38588、5314、1593 | 雙向，代表寫法是 `t2hk` 換成的那個 |
| 喫衆纔祕覈脣癡竈棱鉢皁鍼踊 → 吃眾才秘核唇痴灶稜缽皂針踴 | 不同碼 | 531…2 | — | 不動：票 17c 的表列過，使用者決定照 `t2hk` 換（17c 決定 1） |
| 麪牀啓檯洩 | — | — | — | 17c 已是雙向 |
| 着→著 | tu→ta | 1 | 279 | 「顯着」改回「顯著」（17c） |

- 鉤：`t2hk` 換成「鈎」（審查看到的 `cind` 出「鈎子」就是它），代表寫法「鈎」。
- 擡、糉：essay 沒有含它們的多字詞（只有單字列），`t2hk` 對詞庫沒換掉東西；照票補成雙向（抬擡、粽糉），效果在字頻、bigram 共用與照抄出的「擡頭」「糉子」。
- 囪：表上新出現、票沒列的只有它（17c 和「鈎」一起歸罕用不動）。用 AskUserQuestion 問，使用者選「雙向，代表寫法「囱」」（2026-11-05）。
- 改動後同一張表只剩「—」的 13 字（17c 已決定）與「顯着」。

### 臺、菸的做法（協調者更正後）

票原文寫「詞庫只為原文本來寫的詞另收原寫法」。實作中提醒：語料一樣先經 `t2hk`，「臺」「菸」字頻是 0、沒有 bigram，單打 `gg` 的「臺」會排在後面。協調者確認是寫票漏的，改成照「着／著」：
- 「臺→台」「菸→煙」加進 `ONE_WAY_FORMS`，和「着→著」同一套：「臺灣」另收「台灣」，「台州」不生出「臺州」。
- `Converter::convert` 在 OpenCC 之後把原文的單向寫法放回去（`BothForms::as_written`，逐字對應；字數變了照轉換結果），所以語料與 essay 都照原文寫，兩字的字頻與 bigram 各算各的。簡體原文（中文維基）沒有「臺」，轉出來的照 `s2hk`。
- essay 裡「X臺」和「X台」都有的 8 組（上臺／上台 3614 同頻、下臺 1135／下台 1016…），「X菸」「X煙」都有的 26 組（香菸 1690／香煙 1246…）都是同一個詞，照合併規則（同頻只算一次）。essay 沒有「臺州」「菸霧」這種反過來的寫法。

會紅的測試先寫：`tests/corpora.rs` `one_way_forms_stay_as_the_source_wrote_them`（改前 `臺灣香菸` 轉成 `台灣香煙`）、`tests/run.rs` 斷言「臺」的字頻 > 0。

實測（Node 載入網頁的 WebAssembly 與資料包，候選第幾個；前 = 開工時 `7a23944` 的資料包）：

| 碼 | 前 | 後 |
| --- | --- | --- |
| `gg` | 埋 堆 圭…，「臺」第 15 | 埋 臺 堆…，「臺」第 2 |
| `ggen` | 埋測，沒有「臺灣」 | 臺灣（首選） |
| `iren`、`iril` | 台灣、台州 | 台灣、台州 |
| `ggil` | 沒有「臺州」 | 沒有「臺州」 |
| `ty` | 「菸」第 8 | 「菸」第 3 |
| `haty`、`hafg` | 籍菲、香煙 | 香菸、香煙 |
| `crnd` | 釦子 鍋子…，沒有「鉤子」 | 釦子 鉤子（第 2） |
| `eggd`、`qgmc`、`dsdk`、`fghk`、`fend` | 塗地、扭頭、朽木大、堂奧、爆除 | 溼地、擡頭、柺杖、煙囪、糉子 |
| `ucuj` | 巔峰（9448 + 9448 = 18896） | 巔峰（9448） |
| `orta` | 合著（604 + 合着 1365） | 含著 佔著 合著（604，第 3） |

`ggen` `iren` `iril` `haty` `eggd` `qgmc` `dsdk` `fghk` `fend` 加進 `bench/fixed.tsv`（80/80，合併 main 的票 28b 後 84/84；`iren`、`iril` 改前就對，只當防退步）。Playwright `web/e2e/common-forms.spec.ts` 加 `crnd` 第一頁有「鉤子」、`gg` 第一頁有「臺」、`ggil` 不出「臺州」、`ggen`／`iren`／`haty` 首選（13/13；換回改前的資料包時 `crnd`、`gg`、`ggen`、`haty` 4 項紅）。

### 同頻重複（`lexicon.rs` 的 `combined`）

撞成同一個詞的幾筆：詞頻相同的只算一次，不同的相加（取不重複的詞頻相加，和順序無關）。同一條規則也用在單向生出的寫法撞上 essay 原有的詞（上臺／上台、刻着／刻著）。先寫 `tests/lexicon.rs` `two_spellings_with_the_same_frequency_are_one_count_listed_twice_and_not_summed`（改前 600 ≠ 300）。詞庫報告：換成常見寫法 8412 → 6993 詞（臺、菸不再換）、和別的詞合併 290 → 252。

### 單向跨義項（`ONE_WAY_DISTINCT`）

essay 裡「X着」與「X著」都有的 7 組：

| 詞 | 詞頻 | 另一種 | 結論 |
| --- | ---: | --- | --- |
| 合着 | 1365 | 合著 604（合寫的書） | 義項不同，不生出、不併 |
| 所着 | 100 | 所著 750（寫的） | 「所着」是穿的（所着之衣），不併 |
| 舊着 | 100 | 舊著 354（舊作） | 義項不明、不像同一個詞，不併 |
| 刻着、張着、親着 | 756、740、309 | 刻著 412、張著 422、親著 100 | 「著」是台灣寫法的 zhe，同一個詞，照併 |
| 顯着 | 279 | 顯著 4417 | 錯字，17c 已併進「顯著」 |

不併的三個列成 `ONE_WAY_DISTINCT`：`BothForms::spellings` 只展開雙向，`--verify` 的單向檢查跳過它們。票 17c 與 plan §4.2「同一個義項」的句子已改。測試：`tests/common_forms.rs` `a_one_way_word_whose_other_spelling_is_another_word_keeps_one_spelling`（先紅）。

### 簡體模式「随著」（已知限制，使用者決定）

原因：簡繁轉換照 OpenCC `hk2s`（`TSPhrases`、`TSCharacters`、`HKVariants` 都沒有「著」「着」的列），「隨著」逐段轉成「随著」。OpenCC 的 `tw2s` 才有 `TWVariantsRev`「著→着」加 `TWVariantsRevPhrases` 17 個保護詞（著名、顯著、名著…），但它漏了「土著」「原著」「專著」等，照搬會把它們轉成「土着」。要修得讓簡繁資料包帶一份 essay 的詞表（約 1300 詞「X著→X着」），簡繁資料包就多 LGPL、依賴詞庫。用 AskUserQuestion 問，使用者選「記成已知限制」（2026-11-05）；寫進 plan §4.3。`nbtu` 打「隨着」出「随着」照常。

### 建置接線測試與變異驗證

新測試：`tests/run.rs` `lexicon_and_language_model_are_built_in_the_forms_of_the_corpus`（夾具加羣組、喫飯、這裡、顯着／顯著、晒日、臺日、台口與含「裡」「臺」的語料句；斷言詞與詞頻、「臺」字頻 > 0、「裏／裡」字頻相同、unigram 與前後字 bigram 相同）；`verify::lexicon_forms` 加 bigram 檢查（雙向兩字的 unigram、詞庫裡含它的相鄰字每種寫法的 bigram 相同），`tests/verify.rs` `two_way_forms_with_different_bigrams_fail`。

變異（各自改一處跑 `cargo test -p qim-databuild`，跑完還原）：

| 變異 | 結果 |
| --- | --- |
| A：`bigram::train` 傳 `BothForms::default()`（pipeline.rs） | 紅：`lexicon_and_language_model_…`（run.rs:693，裡的 unigram 與裏不同）、`builds_pack_with_license_and_notice_and_verifies_it` 等 5 個走 `--verify` 的測試（報告的 bigram 檢查失敗） |
| B：essay 改走 `Conversion::S2hk` | 紅：`lexicon_and_language_model_…`（run.rs:673，晒日變曬日、不在字集）、`builds_lexicon_pack_…`（詞數） |
| C：`Forms::common` 拿掉 `correct()`（lexicon.rs） | 紅：`lexicon_and_language_model_…`（run.rs:676，顯着還在）等 6 個 |
| D（無關）：`essay_converter` 改名 | 綠，0 個失敗 |

### NOTICE

詞庫資料包 NOTICE 補寫：OpenCC t2hk 之後「着、臺、菸」照原文；合併時同頻只算一次；「著」寫成「着」的錯字（這版 essay 只有「顯着」）改回併進去；單向的例外 合着、所着、舊着。`tests/run.rs` 斷言這三句。語料的 `ATTRIBUTION.md` 轉換說明加「着, 臺 and 菸 are kept as the source wrote them」。

### 驗證

`cargo xtask data --verify` 通過，建兩次 sha256 相同（速成 `ae3bc207` 不變、詞庫 `08bc8adb`、bigram `9e393a0a`）：
- `詞庫：… 換成常見寫法 6993 詞（其中 252 詞和別的詞合併）… 收 251808 詞，並存寫法另收 5660 種寫法`
- `bigram：… 留下次數 ≥ 2 的 1356104 對（D = 0.569），並存寫法另抄 10121 對`
- `[通過] 字形用常見寫法：對照表 34 組（…）、並存寫法 17 組（…），換掉的寫法在 train 都沒有`
- `[通過] 詞庫用常見寫法：對照表 34 組換掉的寫法與反向的錯字在詞庫都沒有；雙向並存寫法 17 組：含它的詞 6520 個，每種寫法都收、詞頻相同，字頻與 bigram 相同；單向 3 組：含它的詞 2405 個都另收另一種寫法`

### bench 前後對照（前 = 開工時 `7a23944` 的資料包）

| 集合 | 題數 前 → 後 | 答錯 前 → 後 | 修正 前 → 後 |
| --- | --- | --- | --- |
| fixed | 71 → 80 | 全過 | — |
| yue-wiki | 2000 | 1131 → 1131 | 4466 → 4467 |
| zh-wiki | 2000 | 1127 → 1126 | 4541 → 4505（−0.8%） |
| common-voice-yue | 1647 → 1649 | 1030 → 1032 | 4036 → 4043（+0.2%） |
| hkcancor | 989 | 661 → 663 | 2582 → 2588（+0.2%） |
| personal | 4517 | 1081 → 1081 | 1433 → 1433 |

題目跟著前處理變了（原文的「臺」「菸」不再換成「台」「煙」）：yue-wiki、zh-wiki 各 10 句、common-voice-yue 多 2 句。共同句（yue-wiki 1990、zh-wiki 1990、common-voice-yue 1647、hkcancor 989）新舊資料包各打一次：

| 集合 | 答錯 前 → 後 | 修正 前 → 後 | 翻盤（錯→對／對→錯） |
| --- | --- | --- | --- |
| yue-wiki | 1125 → 1125 | 4452 → 4451 | 0／0 |
| zh-wiki | 1121 → 1120 | 4488 → 4483 | 2／1 |
| common-voice-yue | 1030 → 1030 | 4036 → 4033 | 2／2 |
| hkcancor | 661 → 663 | 2582 → 2588 | 0／2 |

由對變錯的 5 題：
- 「但你就鬧埋我」→「鬧臺的」：「臺」有了自己的字頻，和「埋」同碼 `gg`。
- 「跟住阿王菲呢」→「阿雅菸民」：essay 的「菸民」（605）照原文保留，和「菲呢」同碼 `tyrp`；改前它被換成「煙民」，在別的碼上。
- 「所以我就揀咗另外一啲嘅經文」→「姐放」、「你真係北京嘅」→「係真係」、「並要求北京政府保護其祠廟」：同碼接近平手。
前兩題是「臺」「菸」照原文保留的代價（協調者要的做法），之後靠 bigram 與學習分。

`--update-baseline`：答錯 zh-wiki −1、hkcancor +2、common-voice-yue +2（題數 +2，共同句持平），修正 zh-wiki −0.8%、hkcancor +0.2%、common-voice-yue +0.2%、yue-wiki +1，個人集不變；題目照原文寫「臺」「菸」，變差的題原因如上。理由寫在 commit body。

### 個人集

只報數字：答錯 1081 → 1081、修正 1433 → 1433，沒有翻盤。

### code-review（`7a23944` 起）

已處理：
- 新寫的「純異體」「非異體」（名詞表〈並存寫法〉的 _Avoid_ 列了異體字）：改成「兩種寫法同義」（Standards）。
- `BothForms::as_written` 字數不同的分支沒有測試：補 `tests/common_forms.rs`（Standards）。
- `two_way_bigrams` 名字看不出含 unigram、回的是不同處：改名 `two_way_probability_mismatches`，變數改成 `prev`／`next`，閉包不再叫 `forms`（Standards）。
- e2e 的 `firstPage` 與 `expectTop` 開頭重複：抽出 `typeCode`；NOTICE 的 `kept` 自己串清單：改短（Standards）。
- `verify::lexicon_forms` 另判 `ONE_WAY_DISTINCT`：加註原因（照設計不生出另一種寫法，不是重算建置）；`ONE_WAY_DISTINCT` 補進名詞表〈並存寫法〉（Standards）。
- 簡體「随著」的已知限制只在 plan：brief §4.1 補一句（Spec）。
- 驗收「`crnd` 出得了鉤子」「台州的碼不出臺州」：真資料包上的檢查是 Playwright `common-forms.spec.ts`（`crnd`、`ggil`）與 `bench/fixed.tsv`（`ggen`、`iril`）；Rust 測試驗的是規則（Spec）。

未處理：
- 實驗腳本 `gaps` 自己重寫收詞條件（兩字以上、門檻、同詞取最大）、用字串標來源：實驗腳本照 17c 的寫法，只讀快取回答一次性的問題，改建置規則時 README 寫明要重跑，不抽共用。
- `tests/run.rs` 的 `kept`／`ending` 兩段迭代相似、`Converter::convert` 的 `self.forms.both().as_written`：判斷題，不改。
- 17c 決定 6 的「合併計數只在同一個義項之間」：配上 `ONE_WAY_DISTINCT` 仍成立，不改。

### merge main（票 28b）與 code-review（`main...HEAD`）

`bench/fixed.tsv` 衝突：兩邊都在檔尾加題，取 main 的 `an'r` 說明與 28b 的 4 題，再接本票的 9 題（84 題）。合併後 `cargo xtask data --verify` 通過、資料包 sha256 與合併前相同；`cargo xtask bench --check` 通過，各集合與 baseline 相同、固定題 84/84；`cargo xtask check` 通過（e2e 138 項）。

已處理：plan §4.3 用到的 OpenCC「著／着」行為補進 brief〈查證過的事實〉附來源；`forms_with_one_way` 的註解和程式碼不符；progress 的偏差排在 28b 之後、session 紀錄改用代表寫法的寫法。
未處理：最後一列的下一步照協調者並行派票的寫法（26b、27b 由協調者派），不改成 `/implement`；`combined` 以同頻認定重複是票定的規則（plan §4.2）。

