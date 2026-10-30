# Rime / librime 與速成方案研究（含三個使用者案例的根因）

調查日期：2026-10-30。原始碼版本：librime `56737cd2`（master，2026-10-27；行號以此為準），對照 release `1.16.0`（`a251145d`，鼠鬚管 1.1.2 內建）與 `1.17.0`（`33e78140`，Homebrew 與倉輸入法 2.19.5 起使用）。

## 摘要

1. 三個案例的根因都在 **rime/rime-quick 的資料設計**，與前端（鼠鬚管 / 倉）無關：`quick5.dict.yaml` 匯入的是**倉頡全碼**表（`cangjie5.base`），速成碼靠 `speller/algebra` 的 `derive` **另外加上**首尾碼拼寫，所以全碼也是合法輸入，而且排序時跟速成碼分不開（`derive` 產生的拼寫屬於 `kNormalSpelling`，沒有任何懲罰）。
2. `script_translator` 的規則是：**只要有任何「精確匹配、覆蓋整段輸入」的詞條就不造句**（`script_translator.cc:502-505`）。在 quick5 裡，全碼單字（例如 `anr`→問、`mir`→䂖）和八股文的低頻、甚至權重 0 的詞（`一店` 52、`歹口` 0）都算這種詞條，所以整句連打一律變成「查同長度詞條，再按詞頻排」。
3. rime-quick 在 2026-07-17 的 `45d0690` 拿掉了 `min_phrase_weight: 100`，八股文裡所有詞都進了碼表（約 49.8 萬條，含權重 0 的詞），擋掉造句的情況因此更多。
4. 案例 1（mir）：`一台` 不在詞庫裡，造句又被擋掉，所以不可能排上來。即使改成造句，`台` 的詞頻（3737）也低於 `店`（12541），poet 會造出 `一店`。在真機鼠鬚管上實測首選是 `一店 / 䂖 / 歹口`；要看到「歹口」排第一，條件是用 2024-12-29 以前的八股文（沒有 `一店`），而且 `䂖` 沒顯示出來（推論）。
5. 案例 2（anr）：`問` 的倉頡全碼正好是 `anr`。全碼單字和 `門口`（an+r）落在同一個長度桶裡，比的是詞頻：問 50737 > 門口 9435。**在本機真的鼠鬚管上也能復現**（截圖讀回，首選「問」）。所以「電腦版沒有復現」不是引擎或前端差異造成的，可能是使用者當時的狀態不同，原因未查明。
6. 案例 3 就是案例 2 的一般化：全碼碼表混進了速成碼空間。實驗把 `derive` 改成 `xform`（只保留速成碼）後，`anr` 的首選變成 `門口`。
7. 給 QIM 的啟示：碼表要按「方案的碼」建索引，不要讓另一套編碼滲進來；「單一長詞條 vs. 造句」要用可比較的分數來決定，不能「有精確詞條就跳過造句」；詞庫的低頻垃圾詞要有門檻；使用者分詞選字後要學成新詞（Rime 有做，而且有效）。

## 1. librime 架構

### 1.1 管線

- 按鍵依序交給 `engine/processors`，每個 processor 可以接受、拒絕或放行該鍵。`segmentors` 把輸入碼切成帶 tag 的 segment（`abc`、`punct`、`reverse_lookup`…）；每個 segment 由 tag 相符的 `translators` 產生 `Translation`（候選串流）；`filters` 對合併後的候選逐一處理（例如 `simplifier`、`uniquifier`）。來源：官方 wiki [RimeWithSchemata](https://github.com/rime/home/wiki/RimeWithSchemata)〈理解 Processors / Segmentors / Translators / Filters〉。
- quick5 的實際管線見 `rime-quick/quick5.schema.yaml`（`5dcdb9e`）：translators 只有 `punct_translator`、`script_translator`、`reverse_lookup_translator`；filters 有 `simplifier`、`uniquifier`。`abc_segmentor/extra_tags: [reverse_lookup]` 讓每個 abc segment 也帶上 `reverse_lookup` tag，而 `reverse_lookup_translator` 只有在輸入以 prefix 開頭時才剝掉 prefix（`reverse_lookup_translator.cc:143-158`），所以**沒有打 `` ` `` 也會混入拼音反查結果**（實測 `mir` 出現 `迷人`、`anr` 出現 `安然`，type=`reverse_lookup`）。

### 1.2 schema.yaml / dict.yaml

- `schema.yaml`：`engine`（上述管線）、`speller`（`alphabet`、`delimiter`、`algebra`）、`translator`（`dictionary`、`prism`、`preedit_format`…），用 `__patch` 合併其他 yaml（quick5 會載入 `grammar:/hant?` 和 `quick5.custom:/patch?`）。
- `dict.yaml`：檔頭有 `name`、`version`、`sort: by_weight|original`、`use_preset_vocabulary`（匯入八股文 essay.txt）、`columns`、`import_tables`、`max_phrase_length`、`min_phrase_weight`。資料列以 Tab 分隔「字詞、碼、權重」，權重可以是非負整數或百分比（wiki RimeWithSchemata）。
- 編譯（`entry_collector.cc`）：
  - 有碼的詞條直接建立。權重欄空白、又開了 `use_preset_vocabulary` 時，**用八股文的詞頻當權重**（`entry_collector.cc:169-171`）。
  - 沒碼的詞（包括八股文裡的詞）交給 encoder 編碼（`:134-154`）。對 script 類詞典來說，就是把每個字的碼串起來，例如 `一店` → `m iyr`。
  - 一字多碼時，佔該字總權重不到 5% 的碼不用來構詞（`:236-239`）。
  - `max_phrase_length` / `min_phrase_weight` 在 `preset_vocabulary.cc:102-110` 過濾八股文詞條。
  - 寫入時權重取 `log(w)`，w=0 時改用 `log(DBL_EPSILON)`≈-36（`dict_compiler.cc:257`）；查詢時再減去 `log(1e8)`（`dictionary.cc:163-164`）。

### 1.3 table_translator 與 script_translator

| | `table_translator` | `script_translator` |
|---|---|---|
| 定位 | 碼表（倉頡、五筆），「基於規則的動態碼表」 | 固定音節表（拼音、注音），「以固定音節表為算法基礎」（wiki RimeWithSchemata） |
| 輸入解析 | 整串當成一個碼做前綴查詢（`LazyTableTranslation`，`enable_completion` 預設 true） | 用 prism 把輸入切成音節圖（`BuildSyllableGraph`），在圖上查詞 |
| 造句 | `enable_sentence`（預設 true，`table_translator.h:43`）只在**整串完全查不到任何詞條、連補全都沒有**時才造句（`table_translator.cc:293`）；`sentence_over_completion` 可以讓句子排在補全前面 | 只要「至少兩個音節，而且沒有精確匹配整段的系統詞和用戶詞」就造句（`script_translator.cc:502-505`） |
| 學習 | `Memorize`：逐元素 `UpdateEntry`；`enable_encoder` + `encode_commit_history`（預設 true）會把連續上屏的字依構詞規則編成新詞（`table_translator.cc:311-330`） | 依分段選字串接成新詞（`ConcatenatePhrases` / `ProcessSegmentOnCommit`，`script_translator.cc:264-305`） |

quick5 用的是 **script_translator**：把每個倉頡全碼，以及從全碼 `derive` 出來的首尾二碼，都當成「音節」。

### 1.4 連打（造句）流程

1. `ScriptTranslation::Evaluate`（`script_translator.cc:460-517`）建立音節圖，從位置 0 查所有可達的詞條（`dict->Lookup`）和用戶詞，結果按結束位置分桶。
2. `has_reliable_phrase`：最長桶裡的第一筆是精確匹配（`IsExactMatch`：`matching_code_size == code.size()`，`vocabulary.h:63-65`），而且不是糾錯結果。只要有這種詞條就**不造句**。
3. 造句時，`PrepareForMakingSentence`（`:705-723`）對每個起點收集詞條，每個 (起點, 終點) 最多 `max_homophones` 個，組成 `WordGraph` 交給 `Poet`。
4. `Poet::MakeSentence`（`poet.cc:246-256`）：沒有 grammar 時用 DP，每個終點只留一條最佳路徑；有 grammar 時用 BeamSearch（每個「最後一詞」留一條，最多 7 條）。路徑權重是各詞 `Grammar::Evaluate(context, word, entry.weight, is_rear)` 的總和。沒有 grammar 時，每個詞固定加 `kPenalty = log(1e-6) ≈ -13.8`（`grammar.h:18-27`），所以詞數越多分數越低。整段單一詞條會被排除，不參與造句（`poet.cc:208`）。
5. 1.17.0 起有 `MakeSentences`，可以產生多個句子候選（`max_sentences`，預設 1，`translator_commons.h:179`，commit `9422ca7e`）。
6. 候選順序：有句子時句子在最前（`PrepareCandidate`，`:598-605`），接著由長到短逐桶輸出。同碼長時用戶詞優先（`prefer_user_phrase`），但首位要讓給「系統精確匹配」（`kNumExactMatchOnTop`）。phrase 的 quality = `exp(weight) + initial_quality + quality_len/全長`（`:653-655`）。

### 1.5 桶內排序

`compare_chunk_by_head_element`（`dictionary.cc:73-84`）：先比精確匹配，再比剩餘碼長，最後比 `credibility + weight`。credibility 來自拼寫運算：`abbrev` 和 `fuzz` 各 `log(0.5)`（`calculus.cc:14-15,218,242`）；**`derive` 不改拼寫類型、沒有懲罰**（`calculus.h:72-73` 的 `Derivation` 只是 `Transformation` 的子類）。1.16.0 起另有「全碼長度積分」`quality_len`，只累計 `kNormalSpelling` 的長度（`table.cc:612-621`、`user_dictionary.cc:244-247`，commit `f6756b33`）。在 quick5 裡，速成碼也是 normal spelling，所以這個積分同樣分不出全碼和速成碼。

## 2. 排序與學習

- **系統詞權重**：見 1.2，`entry.weight = log(w) - log(1e8) + credibility`。
- **用戶詞學習**（`user_dictionary.cc:425-457`，`algo/dynamics.h`）：
  - 每次上屏 `commits += 1`，`tick += 1`（tick 是該用戶詞典的全域上屏計數）。
  - `dee = formula_d(commits, now, dee_old, tick_old) = commits + dee_old * exp((tick_old - now)/200)`：時間常數為 200 次上屏的指數衰減。
  - 查詢時 `weight = log(formula_p(0, commits/now, now, dee)) + credibility`（`:561`）。`formula_p` 把「整體頻率」和 `dee`（近期熱度）混合：`dee < 20` 時線性逼近 0.5，否則按 `4^(d/kM)` 往 1 推（kM≈200.5）。
  - commits 為負代表已刪除（`:451-453`）。
- **`enable_user_dict`**：translator 的選項，關掉就不查、不寫用戶詞典；`disable_user_dict_for_patterns` 依正則局部停用（quick5 對 `^z.*`、`^yyy.*` 停用）。
- **`encode_commit_history`**（table_translator，預設 true）：需要 `enable_encoder`，把最近的連續上屏歷史按 `encoder` 規則造新詞（`table_translator.cc:327-`）。script_translator 沒有這個選項，改用分段選字串接：實測在 `mir` 裡先選 `一` 再選 `台` 上屏，就學到用戶詞 `一台`，下次打 `mir` 時排第一（下方 G2）。分兩次各自上屏（先 `m` 上屏、再 `ir` 上屏）則**不會**學成詞（G1）。
- **容錯 / 拼寫運算**：`xform`（改寫，不保留原拼寫）、`derive`（保留原拼寫，另加新拼寫）、`erase`、`xlit`、`fuzz`（派生，標記為模糊，可設成只用於構詞）、`abbrev`（派生，標記為縮略，切音節時另行處理）。來源：wiki [SpellingAlgebra](https://github.com/rime/home/wiki/SpellingAlgebra)。懲罰值見 1.5。另有 `enable_correction`（script_translator 的鍵位糾錯，`script_translator.cc:193,207-211`）。
- **librime-octagram**（`lotem/librime-octagram` `57d18b9`）：
  - 外掛註冊 `grammar` 元件（`grammar_module.cc:12`）。方案設定 `grammar/language` 後，`Poet` 建構時用 `Grammar::Require("grammar")` 取得模型（`poet.cc:73-84`）。
  - 造句改用 BeamSearch，每步的懲罰換成 `Octagram::Query(context, word, is_rear)`：用最多 3 字的上文和詞首查 `.gram` 庫。查不到時給 `non_collocation_penalty -12`，弱搭配給 `-24`，句尾另查 `$`（`octagram.cc`）。
  - `translator/contextual_suggestions: true` 時，`ContextualTranslation` 會依上屏的上文，對同終點、同類型的 phrase 重新按 grammar 權重排序（`contextual_translation.cc`）。
  - 需要的資料：`.gram`（例如 `zh-hant-t-essay-bgw.gram`，詞級；`-bgc` 為字級），以及 `grammar.yaml`（`lotem/rime-octagram-data`，`hant` 片段設定 language、`contextual_suggestions`、`max_homophones: 7`）。`.gram` 由 `tools/build_grammar.cc` 從「n-gram 鍵、分數」對建出，語料處理流程不在該 repo。
  - 鼠鬚管 1.1.2 內建了兩個 `.gram` 和 octagram 外掛，但**沒有 `grammar.yaml`**。所以本機編譯後的 schema 只有 `grammar: 0`（`~/Library/Rime/build/quick5.schema.yaml:6`），等於沒有啟用。

## 3. 速成方案、碼表來源與授權

- 主流方案：**`rime/rime-quick`**（東風破配方 ℞ `quick`，安裝指令 `bash rime-install quick cangjie`），HEAD `5dcdb9e`（2026-07-17）。README 寫明「速成取倉頡碼的首尾二碼」、「支持詞句連書、自動調整詞頻、記憶新詞」。本機 `~/Library/Rime/quick5.*` 與 HEAD 完全相同。
- `quick5.schema.yaml`：script_translator，`speller/algebra: derive/^([^z])\w+(\w)$/$1$2/`（保留全碼，另加首尾二碼），reverse_lookup 用 `luna_pinyin`，`__patch` 載入 `grammar:/hant?`。
- `quick5.dict.yaml`：`sort: by_weight`、`use_preset_vocabulary: true`、`import_tables: [cangjie5.base, quick5.supplement]`，**本身沒有任何詞條**。
- 歷史（`git log`，rime-quick）：
  - 2026-05-08 `9debe7f`：改為依賴 rime-cangjie，匯入 `cangjie5.base`；在那之前碼表是內嵌的，同樣是全碼，例如 `問 anr`、`䂖 mir`。
  - 2026-05-13 `82b95c7`：新增 `quick5.supplement`（6825 條擴展區字）。
  - **2026-07-17 `45d0690`：拿掉 `max_phrase_length: 7`、`min_phrase_weight: 100`**。
  - 2026-07-17 `5dcdb9e`：加上 `grammar:/hant?`。
- 碼表內容：`rime-cangjie/cangjie5.base.dict.yaml`（`fcb3d7d`）共 23935 條單字，只有 `text`、`code` 兩欄、沒有權重，檔頭寫「單字碼表《五倉世紀》來自 www.chinesecj.com／倉頡五代構詞碼碼表由惜緣兄製作／License: GPL」。編譯後的 `quick5.table.bin` 有 528,366 條，其中 497,605 條是多字詞，**全部來自八股文**（`rime_table_decompiler` 實測）。單字權重也來自八股文的詞頻。
- 授權（彼此不一致，要注意）：rime-quick、rime-cangjie、rime-essay、rime-octagram-data 的 `LICENSE` 都是 LGPL-3.0。rime-quick / rime-cangjie 的 `AUTHORS` 寫「Derived from ibus-table by acevery (LGPL) and based on original work by 朱邦復 (Public Domain)」，但 `cangjie5.base.dict.yaml` 檔頭寫「License: GPL」，來源是 chinesecj.com。essay.txt 的 `AUTHORS` 列了新酷音（LGPL）、OpenCC 與 Android Pinyin IME（Apache-2.0）、萌典（CC0）。librime、librime-octagram 為 BSD-3-Clause。

## 4. 三個案例的根因

### 復現方式

- 工具：`/opt/homebrew` 的 librime 1.17.0，加上自寫的 `probe.c`（用 `rime_api.h`：`simulate_key_sequence`，再走 `candidate_list_*` 列出候選；`"碼#i,j"` 表示依序選第 i、j 個候選並上屏）。執行時用 `DYLD_LIBRARY_PATH` 換成**鼠鬚管 app 內的 `librime.1.dylib`（1.16.0）和它的外掛**。另加一個 `lua_filter`，把每個候選的 `type`、`quality`、區間附在 comment 上。
- 資料：複製 `/Library/Input Methods/Squirrel.app/Contents/SharedSupport`（含 essay.txt、`.gram`）和 `~/Library/Rime/*.yaml`（含使用者的 `quick5.userdb`），都放在 scratchpad。沒有修改使用者的 Rime 設定。重新編譯出的 `quick5.table.bin` 與使用者實際部署的檔案 **byte 完全相同**（`cmp`）。
- 真機：用 `orca computer` 在「文字編輯」裡、透過使用者目前的鼠鬚管（Hant）實際按 `a n r` 和 `m i r`，再以螢幕截圖讀回候選窗，最後按 Escape，不上屏。

### 案例 1：`mir`（一戈口）首選「歹口」，想要「一台」

- **現象（實測）**：
  - probe（鼠鬚管 1.16.0 + 使用者 userdb）：`1. 一店 [phrase] 2. 䂖 [phrase] 3. 歹口 [phrase] 4. 迷人 [reverse_lookup] …`，**沒有出現 `一台`**。
  - 真機鼠鬚管截圖：`1. 一店 2. 䂖 3. 歹口 4. 迷人 5. 泌乳`，與 probe 一致。
  - 換成 rime-essay `b271b9e~1`（2024-12-29 以前，沒有 `一店`）：`1. 䂖 2. 歹口`（1.16 與 1.17 結果相同）。
- **根因**：
  1. `一台` 不在八股文裡，碼表沒有這個詞（decompile 查無）。
  2. 長度 3 的桶裡有精確匹配的詞條，所以 `has_reliable_phrase` 為真，**不造句**（`script_translator.cc:502-505`）。這些詞條是：`䂖`（`quick5.supplement` 的全碼 `mir`）、`一店`（`m iyr`，八股文 52）、`歹口`（`mni r`，八股文 0）。後兩者要等 rime-quick `45d0690` 拿掉 `min_phrase_weight: 100` 才會進碼表。
  3. 就算造句，`ir` 對應的 `店`（12541）也比 `台`（3737）高。反事實實驗 envE（恢復 min_phrase_weight、去掉 supplement，讓長度 3 沒有精確詞條）中，候選變成 `1. 一店 [sentence]`。
  4. 「歹口」排第一：在目前的檔案上**沒有復現**。可以推出這個結果的條件是：八股文沒有 `一店`（2024-12-29 以前的版本），而 `䂖`（Ext-A）排在它前面卻沒被看見或被過濾。兩者權重都是 0，同分，順序取決於 chunk 次序。這部分是推論。
- **證據位置**：`rime-quick` `45d0690`（quick5.dict.yaml）；`quick5.supplement.dict.yaml:3592`（`䂖 mir`）；essay.txt（`一店 52`、`歹口 0`，`一台` 不存在）；`script_translator.cc:502-505`；`dictionary.cc:73-84`；`poet.cc:208-224`。
- **可信度**：「一台無法排上來」為**高**（實測加原始碼）。「首選為歹口」的具體成因為**低到中**（無法在目前的檔案上復現）。
- **補充**：使用者在同一個 composition 內分段選 `一`（第 144 個候選）再選 `台`，就學到用戶詞 `一台`，下次 `mir` 首選即為 `一台`（G2：`1. 一台 [user_phrase q=1.0025]`）。

### 案例 2：`anr`（日弓口）首選「問」，想要「門口」

- **現象（實測）**：probe 為 `1. 問 [phrase q=1.0005] 2. 門口 [phrase q=1.0001] 3. 日名 …`；真機鼠鬚管截圖為 `1. 問 2. 門口 3. 日名 4. 安然 5. 黯然`。**桌面版在這台機器上可以復現。**
- **根因**：`問` 的倉頡全碼就是 `anr`（`cangjie5.base.dict.yaml:301`），速成碼應為 `ar`。全碼經由 `derive` 保留為合法拼寫，因此 `問`（單音節 `anr`）和 `門口`（`an`+`r`）都是長度 3 的精確詞條，所以不造句，桶內再按權重排：問 50737 > 門口 9435（essay.txt）。
- **反事實**：envF 把 `derive` 換成 `xform`（只留速成碼）後，`anr` → `1. 門口 2. 日名`。
- **倉 vs. 桌面**：
  - 倉輸入法 2.19.5（2026-06-11）起用 librime 1.17.0（`imfuxiao/HamsterDocument` changelog 第 14 行）；鼠鬚管 1.1.2 用 1.16.0（`~/Library/Rime/installation.yaml`）。兩版之間 script_translator 唯一的相關改動是多句候選（`9422ca7e`），只在造句時生效，而這個案例不會造句。
  - 倉沒有內建 quick5：開源版只內建 prelude、rime-essay、雾凇（`Hamster/InputSchemaBuild.sh`），雲端方案清單也沒有速成（`HamsterDocument/.../input_schema.mdx`），使用者必須自己匯入 zip。
  - 倉 2.1.0 之後閉源（Hamster README），無法核對它打包的 essay 與外掛版本。
  - 結論：用目前的 rime-quick，兩邊都會得到 `問` 首選。使用者覺得「桌面沒復現」，可能是桌面當時的 userdb、方案版本或測試輸入不同（使用者的 userdb 中有 `問下 anr my`，代表曾經在桌面用 `anr` 上屏 `問`）。**未查明**。
- **證據位置**：`quick5.schema.yaml` speller/algebra；`calculus.h:72-73`；`script_translator.cc:502-505`；`dictionary.cc:73-84`；`HamsterDocument/src/content/docs/updates/changelog.mdx:14`。
- **可信度**：「問」排第一的根因為**高**（真機加 probe 加反事實）。「倉與桌面的差異」為**低**（桌面也能復現，差異原因無法驗證）。

### 案例 3：連打碼剛好是某字的倉頡全碼時，那個字排第一

- **現象**：`anr`→問、`mir`→䂖（第 2，因為 `一店` 詞頻較高）。
- **根因**：quick5 的碼表**就是**倉頡全碼表（`import_tables: cangjie5.base, quick5.supplement`），速成碼是用 `derive` 額外加上去的，原本的全碼仍然有效。在 script_translator 裡，三碼全碼是「單音節精確匹配整段輸入」，會擋掉造句，又因為單字詞頻普遍高於二字詞而排在前面。`derive` 不帶懲罰，`quality_len` 也把它當 normal spelling，所以引擎**無從得知**這是「倉頡碼」而不是「速成碼」。
- **證據位置**：同案例 2；`rime-quick` `9debe7f`（改為 import cangjie5.base）。在那之前內嵌的碼表同樣是全碼，所以這是方案一直以來的設計（README：「速成、倉頡詞句連打」）。
- **可信度**：**高**。

### 實驗輸出（節錄）

```
# envB：鼠鬚管 librime 1.16.0 + 使用者 ~/Library/Rime 副本（含 userdb）
== mir  preedit=[一 戈口]
   1. 一店   [phrase q=1.0000 0-3]   2. 䂖 [phrase q=1.0000 0-3]   3. 歹口 [phrase q=1.0000 0-3]
== anr  preedit=[日弓口]
   1. 問   [phrase q=1.0005 0-3]     2. 門口 [phrase q=1.0001 0-3] 3. 日名 [phrase q=1.0000 0-3]
# envF：derive → xform（純速成碼）
== anr  preedit=[日弓 口]  1. 門口 [phrase]  2. 日名 [phrase]
# envE：無長度 3 的精確詞條（反事實）
== mir  1. 一店 [sentence]
# G2：在 mir 裡分段選 一 + 台 後
== mir  1. 一台 [user_phrase q=1.0025]  2. 一店 [phrase]  3. 歹口 [phrase]
```

重跑：`DYLD_LIBRARY_PATH="/Library/Input Methods/Squirrel.app/Contents/Frameworks" <scratchpad>/rime/tool/probe <shared> <user> quick5 10 mir anr`（probe 原始碼在 scratchpad `rime/tool/probe.c`；若要長期保存，需移到 `scripts/experiments/`）。

## 5. 給 QIM 的啟示（觀察與建議，不拍板）

1. **碼表隔離**：速成方案的索引只放速成碼（首尾二碼）。倉頡全碼若要支援，應該是另一個方案，或是明確標記、帶降權的「附加碼」。不要用「保留原拼寫」的派生（Rime 的 `derive`）讓兩套編碼共用同一個排序空間。日後加倉頡、粵拼、注音時，每個方案都要能宣告自己的碼空間。
2. **造句和單一長詞條要放在同一把尺上比較**：Rime 的規則「有精確匹配就不造句」，讓低頻詞（`歹口` 0、`一店` 52）和偶然撞碼的全碼單字壓過合理的句子。QIM 可以讓候選詞條和造句結果都用同一個機率（例如 log P，含每詞懲罰）競爭首位，或至少設一個門檻：精確詞條的分數要高於造句分數若干，才擋掉造句。
3. **詞庫門檻**：匯入通用詞頻表時要有最低詞頻或詞長限制（rime-quick 原本的 `min_phrase_weight: 100` 正是這個作用，拿掉後問題變多）。權重 0 的詞不應該能佔首位。
4. **速成碼高度重碼**：`ir` 對應店、唐、台…。單靠 unigram 詞頻，連打一定常錯；需要上文模型（Rime 的 octagram 是可選外掛，要額外的 `.gram` 和 `grammar.yaml`，而且在「有精確詞條」的路徑上只能影響同終點候選的重新排序）。
5. **分段選字要學成新詞**：Rime 對同一個 composition 內的分段選字會學成用戶詞，下次排第一（實測有效）；分開上屏則不會學。QIM 可以考慮也從連續上屏中學詞，相當於 table_translator 的 `encode_commit_history`。
6. **可測試性**：Rime 的排序橫跨 dictionary、translator、poet、filter 多層，又混入 reverse_lookup 等其他 translator（quick5 不加前綴也會混入拼音反查），很難推測行為。QIM 的核心最好讓「給定碼表、詞頻、用戶詞，輸入某碼 → 候選順序」能用純函式測試，三個案例可以直接寫成回歸測試。
7. **授權**：倉頡碼表的授權敘述互相矛盾（LGPL / GPL / Public Domain，來源 chinesecj.com）。採用前需另外查證（參見 `docs/research/data-sources.md`）。

## 6. 來源

- librime：<https://github.com/rime/librime>，`56737cd2`（master）、`a251145d`（1.16.0）、`33e78140`（1.17.0）。檔案：`src/rime/gear/script_translator.cc`、`table_translator.cc`/`.h`、`poet.cc`、`grammar.h`、`contextual_translation.cc`、`translator_commons.cc`/`.h`、`reverse_lookup_translator.cc`、`abc_segmentor.cc`、`src/rime/dict/dictionary.cc`、`table.cc`、`user_dictionary.cc`、`entry_collector.cc`、`dict_compiler.cc`、`preset_vocabulary.cc`、`vocabulary.h`、`src/rime/algo/calculus.cc`/`.h`、`dynamics.h`。相關 commit：`f6756b33`（全碼長度積分，1.16.0）、`9422ca7e`（多句候選，1.17.0）、`fce47ee4`、`1192d8f1`。
- librime-octagram：<https://github.com/lotem/librime-octagram> `57d18b9`（`src/octagram.cc`、`grammar_module.cc`、`tools/build_grammar.cc`）；Homebrew 版用 `dfcc151`。
- rime-octagram-data：<https://github.com/lotem/rime-octagram-data> `8ceef1b`（`grammar.yaml`、README）。
- rime-quick：<https://github.com/rime/rime-quick> `5dcdb9e`；commit `45d0690`、`5dcdb9e`、`82b95c7`、`9debe7f`、`3fe5911`。
- rime-cangjie：<https://github.com/rime/rime-cangjie> `fcb3d7d`（`cangjie5.base.dict.yaml`、AUTHORS、LICENSE）。
- rime-essay：<https://github.com/rime/rime-essay> `054920d`；`b271b9e`（2024-12-29 加入 `一店`）。
- 官方 wiki：<https://github.com/rime/home/wiki/RimeWithSchemata>、<https://github.com/rime/home/wiki/SpellingAlgebra>。
- Hamster（倉輸入法）：<https://github.com/imfuxiao/Hamster> `6569370`（README、`InputSchemaBuild.sh`、`librimeFramework.sh`、`Packages/RimeKit/Package.swift`）；<https://github.com/imfuxiao/HamsterDocument> `1e70d6d`（`updates/changelog.mdx`、`guides/input_schema.mdx`）；fork <https://github.com/imfuxiao/librime>（master = 1.17.0）。
- Homebrew formula `librime` 1.17.0_2（`brew cat librime`）。
- 本機：鼠鬚管 1.1.2（`/Library/Input Methods/Squirrel.app`，librime 1.16.0，含 lua / octagram / predict 外掛、`zh-hant-t-essay-bg{w,c}.gram`，無 `grammar.yaml`）；`~/Library/Rime`（唯讀參考）。
