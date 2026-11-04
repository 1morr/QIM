# 使用者詞庫純文字格式：librime userdb 與 libime table

查證日期：2026-11-04。目的：QIM 定稿「使用者詞庫匯出格式」前，先對照兩個成熟產品的純文字做法。只記事實，不下決定。

來源一律是原始碼，唯讀 clone 到 scratchpad，沒有執行。授權：librime 為 BSD，libime 為 LGPL-2.1-or-later（只取做法，不取程式碼）。

| 專案 | repo | 查證用 commit | 備註 |
| --- | --- | --- | --- |
| librime | https://github.com/rime/librime | `e930f57b5cc2eeac91aed3b6c7fe62e96f1fc8bc`（`git describe` = `1.17.0-74-ge930f57b`） | brief 已引用此 commit。查證時 master 為 `9777cb2`；本文引用的七個檔案（`user_db.cc`、`user_db.h`、`user_dict_manager.cc`、`user_dict_manager.h`、`db_utils.cc`、`tsv.cc`、`user_db_recovery_task.cc`）在兩者間 `git diff --stat` 無差異 |
| libime | https://github.com/fcitx/libime | `53df6a248793f9a00c1769e658630de313f3e419`（2026-11-01 "bump libime version"，`CMakeLists.txt:2` 為 `project(libime VERSION 1.1.18)`） | 淺 clone 沒有 tag，未查證此 commit 是否對應 release tag |

行號指上表 commit 的檔案，路徑相對 repo 根目錄。

## 1. librime userdb 文字格式（`*.userdb.txt`）

librime 有兩條文字路徑，格式不同，下面分開記。

- **Backup / Restore / Synchronize**：`<dict>.userdb.txt`，保留 c、d、t，同步用。這是 brief 引用的那條。
- **Export / Import**：`rime_dict_manager --export / --import` 用，只有 `phrase<Tab>code<Tab>count`，見 §1.7。

### 1.1 檔案結構

`TsvWriter::operator()` 依序寫三段（`src/rime/dict/tsv.cc:59-87`）：

1. 檔頭註解一行：`# ` + `file_description`（`tsv.cc:64-66`）。userdb 的 `file_description` 是 `Rime user dictionary`（`src/rime/dict/user_db.cc:98-102`），所以第一行固定是 `# Rime user dictionary`。
2. metadata 行：每筆 `#@` + key + Tab + value（`tsv.cc:68-70`）。key 帶前導 `/`，例如 `#@/db_name<Tab>luna_pinyin`。
3. 資料列：formatter 輸出的欄位以 Tab 連接，一列一筆（`tsv.cc:71-84`）。

metadata 與資料列都按底層 `std::map<string,string>`（`TextDbData`，`src/rime/dict/text_db.h:17`）的字串序輸出，所以 metadata 順序是 key 的字典序，資料列按 `code + Tab + phrase` 字典序。

### 1.2 metadata 行

建立 db 時寫入的 key：

| key | 寫入處 | 內容 |
| --- | --- | --- |
| `/db_name` | `src/rime/dict/db.cc:54-57`（`Db::CreateMetadata`） | 詞典名（不含 `.userdb.txt`）；`UserDbWrapper` 以 `name` 建構，見 `user_db.h:104-106`（`Create(name)` 傳入 `DbFilePath(name, extension())` 與 `name`） |
| `/rime_version` | 同上 `db.cc:57` | 寫出時的 librime 版本字串（`RIME_VERSION`） |
| `/db_type` | `text_db.cc:180-182`（`TextDb::CreateMetadata`） | userdb 為 `userdb`（`user_db.cc:107` 傳入 `"userdb"`） |
| `/user_id` | `user_db.cc:109-112`、`user_db.h:84-86`（`UpdateUserInfo`） | 裝置的 `deployer.user_id` |
| `/tick` | `src/rime/dict/user_dictionary.cc:459-463`（`UpdateTickCount`）、`:467-469`（`Initialize` 寫 `0`） | 全域計數，每次上屏 commits>0 時 +1（`user_dictionary.cc:445-446`） |

解析（`tsv.cc:31-39`）：

- 行首 `#@` 才是 metadata；去掉 `#@` 後以 Tab 切，必須剛好 2 欄，否則記 warning 並略過該行（不中止）。
- 其他 `#` 開頭的行是註解，略過。遇到 `# no comment` 這一行之後，後面的 `#` 行不再當註解（`tsv.cc:40-42`），這是 TSV 層的通用規則，不是 userdb 專屬。
- 行尾空白會被 `trim_right`（`tsv.cc:27`）；空行略過。

哪些必有（`UserDictManager::Restore`，`src/rime/lever/user_dict_manager.cc:73-105`）：

- `#@/db_type` 必須是 `userdb`，否則 `IsUserDb()` 為假、Restore 回傳 false（`user_dict_manager.cc:86-87`；`user_db.cc:147-150`）。
- `#@/db_name` 必須存在且去掉 `.userdb` 後綴後非空，否則 Restore 失敗（`user_dict_manager.cc:88-90`；`user_db.cc:152-162`）。注意 `GetDbName` 在 `MetaFetch` 失敗時 `return name`（空字串），行為等價於失敗。
- `#@/tick`、`#@/user_id`、`#@/rime_version` 非必有：`/tick` 缺少時 `their_tick_` 維持 0（`user_db.cc:187-191, 197-206`）；`/user_id` 缺少時 `GetUserId()` 回 `"unknown"`，只用於 log（`user_db.cc:164-168`；`user_dict_manager.cc:98-100`）；`/rime_version` 匯入時不使用（`GetRimeVersion` 在查證範圍內只定義未被 Restore 呼叫，`user_db.cc:170-174`）。

### 1.3 資料列

- 欄位：`code<Tab>phrase<Tab>value`，三欄 Tab 分隔（`user_db.cc:87-96` formatter）。
- 內部 key 是 `code + Tab + phrase`，`code` 以空白結尾，多音節以空白分隔，故輸入碼每個音節後都帶空白（`user_db.cc:69` 註解 `key ::= code <space> <Tab> phrase`）。
- value 欄是 `UserDbValue::Pack()`：`c=<commits> d=<dee> t=<tick>`，三個 `key=value` 以單一空白連接（`user_db.cc:22-26`）。
- parser（`user_db.cc:71-85`）：欄數 <2 或前兩欄為空則拒絕該列；`code` 若不以空白結尾會自動補一個（註解：fix invalid keys created by a buggy version）；第 3 欄缺少時 value 為空字串（之後解成 c=0, d=0, t=0）。
- value 解析（`user_db.cc:28-56`）：以空白切，每段找 `=`，沒有 `=` 的段略過；只認 `c`（`std::stoi`）、`d`（`std::stod`，上限夾在 10000，溢位 `out_of_range` 設為 0）、`t`（`std::stoul`）；其他 key 靜默忽略；數值解析失敗記 error 並讓 `Unpack` 回傳 false，但呼叫端 `UserDbValue(const string&)` 建構式不檢查回傳值（`user_db.cc:18-20`），已解出的欄位保留。

### 1.4 統計欄位語意與刪除記號

- `c`（commits）：累計上屏次數。`UpdateEntry` 在 commits>0 時 `v.commits += commits`（`user_dictionary.cc:442-447`）。
- `d`（dee）：衰減後的權重分，公式 `formula_d(d, t, da, ta) = d + da * exp((ta - t) / 200)`（`src/rime/algo/dynamics.h:6-8`）。
- `t`（tick）：該筆最後更新時的全域 `/tick`（`user_dictionary.cc:455`）。
- **刪除記號**：`c` 為負數即已刪除；刪除時 `v.commits = min(-1, -v.commits)`（`user_dictionary.cc:451-453`），再次上屏 commits>0 且 `v.commits<0` 時取正「復活」（`user_dictionary.cc:443-444`）。查詢時 `v.commits < 0` 的條目被略過（`user_dictionary.cc:548`）。刪除條目仍留在檔內，以負 c 的形式寫出。

### 1.5 浮點 d 的精度

`Pack()` 用 `std::ostringstream` 直接 `<<` 寫出 `dee`（`user_db.cc:22-26`）。`src/rime/dict/` 下沒有 `setprecision`（`grep -rn "setprecision\|precision" src/rime/dict/` 只命中 `table_db.cc:14` 一行註解），所以是 C++ 串流預設格式，即 6 位有效數字、必要時科學記號。這一點由原始碼推得，沒有實跑 round-trip 驗證（未實測）。

### 1.6 匯入：合併，不是取代

`UserDictManager::Restore`（`user_dict_manager.cc:73-105`，由 `Synchronize` 在 `:197` 呼叫）流程：

1. 把檔案載入暫存 db `.temp`（`:74-84`，`TextDb::Restore` 先 `Clear()` 再讀，`text_db.cc:168-178, 203-216`）。
2. 驗證 `/db_type`、取 `/db_name`（`:86-90`）。
3. 開啟目標 db（`:91-93`），以 `UserDbMerger` 逐筆合併（`:101-103`）。

`UserDbMerger`（`src/rime/dict/user_db.cc:187-244`）：

- 建構：`our_tick_` = 目標 db 的 `/tick`（讀不到或解析失敗為 1，`:176-185`）；`their_tick_ = 0`；`max_tick_ = our_tick_`。
- `MetaPut`：只處理 `/tick`，設 `their_tick_`，`max_tick_ = max(our, their)`；其他 metadata 全部忽略（`:197-206`）。metadata 行都在資料列之前（`tsv.cc:68-73`），所以資料列處理時 `their_tick_` 已就緒。
- `Put`（`:208-228`），對每一筆：
  1. `v` = 對方值；若 `v.tick < their_tick_`，`v.dee = formula_d(0, their_tick_, v.dee, v.tick)`，即把 d 衰減到對方當下的 tick。
  2. `o` = 我方既有值（沒有則全 0）；若 `o.tick < our_tick_`，同樣把 `o.dee` 衰減到 `our_tick_`。
  3. 若 `|o.c| < |v.c|`，`o.c = v.c`（取絕對值較大者，連同符號；所以對方的刪除狀態在 |c| 較大時會覆蓋我方）。
  4. `o.d = max(o.d, v.d)`。
  5. `o.t = max_tick_`。
  6. 寫回，成功則 `merged_entries_` +1。
- `CloseMerge`（解構時呼叫，`:193-195, 230-244`）：只有 `merged_entries_ > 0` 時才寫 `/tick = max_tick_` 與 `/user_id`。

另一個匯入方式 `UserDbImporter` 見 §1.7，它不用 tick。

### 1.7 另一條路：Export / Import（plain table 文字）

- `UserDictManager::Export`（`user_dict_manager.cc:107-129`）用 `TableDb::format.formatter`，`file_description` 為 `Rime user dictionary export`（`:118`）；仍會寫出 `#@` metadata 行（共用 `TsvWriter`）。
- 資料列：`phrase<Tab>code<Tab>commits`，code 去掉尾端空白，commits 以 `std::to_string` 寫出，**c<0 的已刪除條目不輸出**（`src/rime/dict/table_db.cc:40-55`）。不輸出 d 與 t。
- `UserDictManager::Import`（`user_dict_manager.cc:131-152`）用 `TableDb::format.parser`（`table_db.cc:18-38`）：第 3 欄解為 `commits`，並令 `dee = (commits + 1) / 1e8`，tick 為 0；再交給 `UserDbImporter`（`user_db.cc:246-268`）：忽略所有 metadata（`:248-250`）；v.c>0 時 `o.c = max(o.c, v.c)`、`o.d = max(o.d, v.d)`；v.c<0 時 `o.c = min(v.c, -|o.c|)` 標記刪除；寫回時 `o.t` 不變（既有條目保留原 t，新條目為 0）。這條路是合併，不是取代。
- 注意：Import 要求目標 db 已存在且 `IsUserDb()`（`user_dict_manager.cc:132-140`）。

### 1.8 版本號

userdb 文字檔**沒有格式版本號**。檔頭註解只有描述字串（`user_db.cc:101`），metadata 有 `/rime_version`（librime 程式版本，`db.cc:57`），但 Restore/Import 不讀它、不依它分支（`user_dict_manager.cc:73-105`；`user_db.cc` 全檔 `GetRimeVersion` 只有定義）。格式相容靠 parser 的容錯：補空白、忽略未知 `key=value`、略過壞列（`user_db.cc:71-85, 28-56`；`tsv.cc:36-39, 48-52`）。

## 2. libime table 引擎文字格式

檔案：`src/libime/table/tablebaseddictionary.cpp`、`src/libime/table/autophrasedict.cpp`、`src/libime/table/table-text-format.md`（repo 內的格式說明）。以下行號都在 `tablebaseddictionary.cpp`，除非另註。

### 2.1 系統碼表文字格式

結構（`table-text-format.md` 開頭，依序）：config、可選 `[Rule]`、`[Data]`、可選 `[Phrase]`。中英文關鍵字都接受（`strConst`，`tablebaseddictionary.cpp:80-85`，兩套字串：`键码=`/`KeyCode=`、`码长=`/`Length=`、`规避字符=`/`InvalidChar=`、`拼音=`/`Pinyin=`、`拼音长度=`/`PinyinLength=`、`[数据]`/`[Data]`、`[组词规则]`/`[Rule]`、`提示=`/`Prompt=`、`构词=`/`ConstructPhrase=`、`[词组]`/`[Phrase]`）。

`TableBasedDictionary::saveText`（`:764-834`）寫出順序，永遠用英文關鍵字（`strConst[1]`）：

1. `KeyCode=<輸入碼字元集>`（`:766-770`）
2. `Length=<最大碼長>`（`:771`）
3. `InvalidChar=...`（有才寫，`:772-778`）
4. `Pinyin=<前綴字元>`、`Prompt=<前綴字元>`、`ConstructPhrase=<前綴字元>`（各自有設才寫，`:779-790`）
5. `[Rule]` 加每條規則一行（有 rule 才寫，`:792-797`）
6. `[Data]`（`:798`），之後依序是 prompt 條目（`<前綴><code> <值>`，`:800-815`）、construct-phrase 條目（`:816-831`），最後是 `saveTrieToText(phraseTrie_)`（`:833`）。
   - **`[Phrase]` 區塊 saveText 不會寫**：`saveText` 全程沒有輸出 `STR_PHRASE`，phrase 條目已展開成 `[Data]` 的 `code value` 行。這是從 `:764-834` 讀出的，未另行實跑。

`[Data]` 行格式：`<key><空白><value>`，以第一個空白切（`parseDataLine`，`:401-438`）；value 可以用雙引號包起來並以反斜線跳脫（`maybeEscapeValue` 寫、`maybeUnescapeValue` 讀，`:103-118`）。`#` 在 `[Data]` 裡不是註解（`table-text-format.md` §3）。`[Data]` 條目**沒有次數欄**：權重由行的出現順序決定（`saveTrieToText` 依 trie 值，即插入序號，由小到大寫出，`:152-170`；插入時 `index += 1`，`:186-190`）。

### 2.2 使用者詞庫的文字存法

`TableBasedDictionary::saveUser(out, TableFormat::Text)`（`:1027-1051`）依序寫三段，載入 `loadUser(in, TableFormat::Text)`（`:946-998`）以單一狀態機讀回。

| 段落 | 標記行 | 條件 | 每列格式 | 讀寫處 |
| --- | --- | --- | --- | --- |
| user phrase | 無（檔首即是，狀態 `Phrase`） | 永遠 | `code<空格>value`，兩欄；**沒有次數** | 寫 `:1028`（`saveTrieToText(userTrie_)`），讀 `:971-973`（`insertDataLine(line, user=true)`） |
| auto phrase | `[Auto]` | `autoPhraseDict_` 非空才寫（`:1030-1031`） | `code<空格>value<空格>hit`，三欄；**有 hit 次數** | 寫 `:1041-1044`，讀 `:974-987` |
| delete | `[Delete]` | `deletionTrie_` 非空才寫（`:1046-1049`） | `code<空格>value`，兩欄 | 寫 `:1048`，讀 `:988-994` |

細節：

- 標記行是整行比對 `[Auto]` / `[Delete]`（`:87-88` 常數、`:960-968` 比對，先 `trimView`）。一旦遇到標記，狀態切換，不能回到 user phrase 段；段落順序在寫出時固定為 user → `[Auto]` → `[Delete]`，讀入時不強制順序。
- user phrase 的排序：依 `userTrie_` 的值（插入序號）由小到大，也就是越新插入的越後面（`:163-165`）。次數不存，所以 user phrase 沒有 hit 統計，只有先後順序。
- **auto phrase 有存 hit**。`AutoPhrase` 結構含 `entry_` 與 `uint32_t hit_`（`autophrasedict.cpp:27-35`）。文字寫出時的順序是：先 `search("")` 依 entry 字串序收集，再 `std::views::reverse` 逆序寫出（`:1032-1044`），即 entry 字串序由大到小，**不是 MRU 順序**（二進位格式 `AutoPhraseDict::save` 才依 MRU 由舊到新，`autophrasedict.cpp:137-145`）。讀入時以 `std::stoi` 解 hit，呼叫 `autoPhraseDict_.insert(entry, hit)`（`:981-983`）。`AutoPhraseDict::insert` 對已存在的 entry 只會 relocate 到最前，並且只在 `value == 0` 時 hit+1（`autophrasedict.cpp:66-82`）；hit 欄不等於 0 時，重複 entry 的 hit 不會被覆蓋（由原始碼推得，未實測）。auto phrase 總數受 `maxItems_` 限制，超過時從尾端丟棄（`autophrasedict.cpp:79-81`）。
- `[Auto]` 列的驗證：欄數必須剛好 3 且 code 全為輸入碼字元（`isAllInputCode`），否則略過該列（`:975-978`）；value 欄可有雙引號跳脫（`:980`）；hit 解析失敗（`std::stoi` 丟例外）略過該列（`:984-986`）。
- user phrase 讀入時，若 key 以 `Pinyin`、`Prompt`、`ConstructPhrase` 的特殊前綴字元開頭，該列被拒絕（`parseDataLine` 的 `if (user) return {}`，`:427-431`）。
- 讀 user phrase（`insert` 到 user 字典）時會把 `deletionTrie_` 中同 entry 移除（`:265-267`）。刪除某字時，`removeWord`（`:1549-1559`）會同時移除 auto 與 user 條目，並且只在系統字典裡有該條目時才加進 `deletionTrie_`。
- 非法 UTF-8 行整行略過（`:957-959`）。

### 2.3 版本號

- **文字格式沒有版本號**：`saveUser` / `saveText` 的文字分支沒有寫任何版本或 magic（`:764-834, 1027-1051`），`table-text-format.md` 也沒有版本欄。
- **二進位格式**有 magic + version：`tableBinaryFormatMagic = 0x000fcabe`／`tableBinaryFormatVersion = 0x2`、`userTableBinaryFormatMagic = 0x356fcabe`／`userTableBinaryFormatVersion = 0x3`、`extraTableBinaryFormatMagic = 0x6b0fcabe`／`extraTableBinaryFormatVersion = 0x1`（`:58-63`）。user 二進位載入接受 version 1、2（未壓縮）與 3（zstd 壓縮），其他丟 `invalid_argument`（`:932-944`）。這些只約束 binary，不約束文字。

## 對 QIM 的意義

只列兩者做法，不替 QIM 決定。

| 面向 | librime userdb（`*.userdb.txt`，Backup/Restore） | libime table user dict（`saveUser` Text） |
| --- | --- | --- |
| 檔頭與版本 | 第一行註解 `# Rime user dictionary`，隨後 `#@key<Tab>value` 的 metadata（`/db_name`、`/db_type`、`/rime_version`、`/user_id`、`/tick`）。**沒有格式版本號**；`/rime_version` 是程式版本，匯入不讀。 | **沒有檔頭**，第一行就是資料；沒有 metadata。**文字沒有版本號**（binary 才有 magic + version）。 |
| 一列一詞的欄位 | `code<Tab>phrase<Tab>c=.. d=.. t=..`，三欄 Tab；code 是空白分隔音節並以空白結尾，phrase 可含空白（Tab 分隔所以安全）。 | 以空白分隔，不用 Tab：user 與 delete 段 `code value`（兩欄）；`[Auto]` 段 `code value hit`（三欄）；value 含空白、引號、反斜線時以雙引號與反斜線跳脫。 |
| 統計欄位的寫法 | value 欄是 `key=value` 以空白連接（`c`、`d`、`t`）；parser 忽略未知 key，因此可擴欄。`d` 是浮點，寫出為 C++ 串流預設 6 位有效數字。 | 固定欄：只有 `[Auto]` 段的第 3 欄 `hit`（整數）。user phrase 無次數，只靠行序表達新舊。無浮點。 |
| 刪除記號 | 以負的 `c` 表示，條目仍留在檔內（`c = min(-1, -c)`）；再上屏時取正復活。Export 的 plain 文字則直接不輸出已刪除條目。 | 獨立的 `[Delete]` 段，列出被刪的 `code value`；只有「系統字典有該條」時才會進這段。 |
| metadata（tick） | 全域 `/tick` 以 `#@/tick` 存；每列的 `t` 是該筆最後更新時的 tick；`d` 隨 tick 差指數衰減（`exp((ta-t)/200)`）。合併時用兩邊 tick 把 d 衰減對齊，再取 max tick。 | 沒有全域計數、沒有每筆時間戳；順序（行序）與 auto phrase 的 MRU 順序是唯一的時間資訊，文字存 auto 時還會丟失 MRU 順序（改為 entry 字串序逆序）。 |
| 匯入是取代或合併 | 合併。Restore：`c` 取 \|c\| 較大者（含符號）、`d` 取 max（先各自衰減到本端 tick）、`t` 設為 max tick，並寫回 `/tick`。Import（plain 三欄）也是合併：c 取 max、d 取 max、負 c 標記刪除、不動 tick。 | 查證範圍內 `loadUser` 文字分支是**逐列插入**到既有 trie，沒有清空動作（`:946-998` 沒有呼叫 `clear`，與 binary 路徑 `loadUserBinary` 是否取代未查證）；auto 重複 entry 只 relocate，不覆蓋 hit（見 §2.2）。user 插入時會移除同條目的 delete 標記（`:265-267`）。 |

其他觀察：

- librime 的 userdb 檔名即詞典名（`<db_name>.userdb.txt`，`user_dict_manager.cc:69, 44-47`），同步目錄結構是 `sync_dir/<裝置id>/<db_name>.userdb.txt`（`user_dict_manager.cc:189-202`）；匯入時以檔內 `#@/db_name` 決定合併到哪個詞典，不是檔名。
- libime 的 `[Auto]` 與 `[Delete]` 標記讓同一檔案分三個語意段；librime 則是同一表格、以 value 欄內容區分狀態。
- 兩者都對壞列採「略過並繼續」，不整檔失敗（librime `tsv.cc:48-52`；libime `:975-986`）。

## 未查證項目

- libime 該 commit 是否對應某個 release tag（淺 clone 沒有 tag）。
- `d` 實際輸出的位數：由「無 `setprecision`」推得為串流預設（6 位有效數字），沒有實跑。
- libime `loadUser` 文字分支與 binary 路徑相比，是否在載入前清空既有 user 資料（文字分支內沒有清空呼叫；呼叫端是否先 `clear` 未查）。
- fcitx5-chinese-addons / fcitx5-table-extra 等上層是否在文字使用者詞庫外另外加包裝或版本：未查證，本文只涵蓋 libime 函式庫本身。
