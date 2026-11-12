# 使用者詞的學習、遺忘、上限與跨裝置合併：成熟做法

查證日期 2026-11-12。範圍：新詞要出現幾次才算數、最短長度、舊習慣怎麼衰減、條目上限與淘汰、刪除、多台裝置的快照怎麼合併與傳播刪除。給 M4（記住的 email 與英文學到的字串併成一種使用者詞、學習次數只算最近一段、條目上限、學習資料匯出 v3 加刪除墓碑）用。

只採原始碼（釘到 commit）與官方文件。查不到的寫「未查到」。標「由原始碼推得」的是讀程式碼得到的結論，沒有實跑。已在 repo 裡查過的不重抄，直接連結：[`libime-user-model.md`](libime-user-model.md)（libime 的歷史模型）、[`userdict-text-formats.md`](userdict-text-formats.md) §1（userdb 文字格式與 `UserDbMerger` 流程）、[`remember-strings.md`](remember-strings.md)（email 記憶）、brief §12.2。

快照：

| 專案 | commit |
| --- | --- |
| librime | `e930f57b5cc2eeac91aed3b6c7fe62e96f1fc8bc` |
| AOSP LatinIME | `127336e9f29d69607eab55982324b210279ae8c5`（連結形式 `https://android.googlesource.com/platform/packages/inputmethods/LatinIME/+/<commit>/<路徑>`） |
| HeliBoard | `3e91b4f1bf711a8bd3eb2d049c8b285fcc9cfa10` |
| AnySoftKeyboard | `1ee43e7f037b905e1c762e3beceac1c0178797b6` |
| FlorisBoard | `bac346e8a5e7ee7104359cebdbce7d4105d3f0fa` |
| libime | `53df6a248793f9a00c1769e658630de313f3e419` |
| fcitx5-chinese-addons | `e8d930f402e359cfd76a101ee3e46815d1331d83` |

## 1. librime（Rime）

連結：<https://github.com/rime/librime/tree/e930f57b5cc2eeac91aed3b6c7fe62e96f1fc8bc>。

### 1.1 衰減與 tick

時鐘不是牆鐘，是**上屏次數**。`UpdateEntry` 只在 `commits > 0` 時 `UpdateTickCount(1)`（`src/rime/dict/user_dictionary.cc:446`）；刪除（`commits < 0`）與純查詢更新（`commits == 0`）不推進 tick。衰減公式（`src/rime/algo/dynamics.h:6-8`）：

```cpp
inline double formula_d(double d, double t, double da, double ta) {
  return d + da * exp((ta - t) / 200);
}
```

上屏更新（`user_dictionary.cc:447`）：`v.dee = formula_d(commits, tick_, v.dee, v.tick)`，即 `d ← commits + d·e^(−(tick_−t)/200)`。時間常數 200 次上屏，半衰期 200·ln2 ≈ 139 次。`c`（`v.commits`）是**累計**上屏次數，不衰減；只有 `d` 衰減。

兩層「忘記」：

- 排序時 `d` 先衰減到現在的 tick，再進 `formula_p` 換成權重（`user_dictionary.cc:550-562`）。
- 讀取時丟掉太舊的（`user_dictionary.cc:532-555`）：

```cpp
// Corresponds to log2(1/T)*200*ln(2) ticks.
constexpr double kDiscardThreshold = 1e-200;
...
  if (v.commits < 0)  // deleted entry
    return e;
  if (v.tick < present_tick)
    v.dee = algo::formula_d(0, (double)present_tick, v.dee, (double)v.tick);
  // tick==0: table/stabledb phrases (custom_phrase.txt etc.) never entered
  // userdb decay. Discard only aged user-history entries.
  if (v.tick != 0 && v.dee <= kDiscardThreshold)
    return e;
```

`d ≤ 1e-200` 約等於 200·ln(1e200) ≈ 92,100 次上屏沒再用。這是**查詢時略過**，不是刪檔案：該列仍留在 userdb。

### 1.2 條目上限與清理

未查到。`src/rime/dict/user_dictionary.cc`、`user_db.cc`、`user_dict_manager.cc`、`deployment_tasks.cc` 裡沒有條目數上限、LRU、或實際刪除舊條目的程式（`grep` 上限／清理類字眼只命中查詢的 `limit`／`depth_limit`）。`CleanupTrash`（`deployment_tasks.cc:627`）清的是部署的 trash 目錄，不是使用者詞。所以 Rime 的使用者詞只增不減，靠 §1.1 的讀取時略過讓舊詞不再出現。

### 1.3 `c` 負值是刪除記號

詳見 `userdict-text-formats.md` §1.4。補充 `UpdateEntry` 的原文（`user_dictionary.cc:442-457`）：

```cpp
  if (commits > 0) {
    if (v.commits < 0)
      v.commits = -v.commits;  // revive a deleted item
    v.commits += commits;
    UpdateTickCount(1);
    v.dee = algo::formula_d(commits, (double)tick_, v.dee, (double)v.tick);
  } else if (commits == 0) {
    const double k = 0.1;
    v.dee = algo::formula_d(k, (double)tick_, v.dee, (double)v.tick);
  } else if (commits < 0) {  // mark as deleted
    v.commits = (std::min)(-1, -v.commits);
    v.dee = algo::formula_d(0.0, (double)tick_, v.dee, (double)v.tick);
  }
  v.tick = tick_;
```

刪除把 `c` 改成 `−|c|`（至少 −1），**保留絕對值**；`d` 只衰減不歸零；再次上屏把負號翻回正再加。

### 1.4 同步資料夾與合併

流程見 `userdict-text-formats.md` §1.6。`Synchronize`（`src/rime/lever/user_dict_manager.cc:178-204`）：走訪 `sync_dir` 下**每個子目錄**（含自己的 `<installation_id>/`）的 `<dict>.userdb.txt`，逐一 `Restore`（合併），最後 `Backup` 把合併後的整本寫回自己的子目錄。沒有「誰比較新」的全域判斷，每個快照都是獨立合併。

`UserDbMerger::Put` 原文（`src/rime/dict/user_db.cc:208-228`）：

```cpp
  UserDbValue v(value);
  if (v.tick < their_tick_) {
    v.dee = algo::formula_d(0, (double)their_tick_, v.dee, (double)v.tick);
  }
  UserDbValue o;
  string our_value;
  if (db_->Fetch(key, &our_value)) {
    o.Unpack(our_value);
  }
  if (o.tick < our_tick_) {
    o.dee = algo::formula_d(0, (double)our_tick_, o.dee, (double)o.tick);
  }
  if (std::abs(o.commits) < std::abs(v.commits))
    o.commits = v.commits;
  o.dee = (std::max)(o.dee, v.dee);
  o.tick = max_tick_;
```

同一鍵兩邊都有時：`c` 取絕對值大的（連符號）、`d` 先各自衰減到自己 db 的 tick 再取大的、`t` 設成 `max(兩邊 /tick)`。兩台裝置的 tick 是各自的計數，沒有對齊，合併後 `d` 是「各自衰減到各自現在」的最大值，不是同一個時間點上的比較。

**刪除怎麼傳播（由原始碼推得，未實跑）**：

1. 刪除保留 `|c|`（§1.3），合併的比較是嚴格小於（`<`）。A 刪除後 `c=−5`，B 仍是 `c=+5`：B 合併 A 的快照時 `|5| < |−5|` 為假，B 保持 `+5`；A 合併 B 的快照同樣保持 `−5`。**兩邊永遠不收斂，刪除傳不過去**，且合併結果取決於「誰是本機」。
2. 只有一方的 `|c|` 較大時才贏：B 在 A 刪除前後又上屏過（`c=6`），A 合併 B 後變成 `+6`，等於刪除被復活；反過來 A 刪除時 B 的 `|c|` 較小，B 合併後變 `−|c_A|`（刪除傳過去）。所以「刪除」的勝負由累計次數決定，不是由先後。
3. 墓碑（負 `c` 的列）沒有任何回收機制（§1.2），永遠留在檔案裡。
4. 匯入的另一條路 `UserDbImporter`（`user_db.cc:252-268`）不同：`v.commits < 0` 時 `o.commits = min(v.commits, −|o.commits|)`，刪除**一定**蓋過本機的正值；但它只用在 `Import`（純文字 `phrase\tcode\tcommits`，匯出時不輸出 `c<0` 的列，`table_db.cc:40-55`），不在 `Synchronize` 的路徑上。

結論：Rime 的同步合併是「`c` 取大、`d` 取大」的 max-merge，對只增的計數是正確的，對刪除有缺口。`userdict-text-formats.md` §1.6 已寫了合併規則，上面 1 是它沒寫的後果。

## 2. AOSP LatinIME

連結基底 `https://android.googlesource.com/platform/packages/inputmethods/LatinIME/+/127336e9f29d69607eab55982324b210279ae8c5/`。Gboard 的開源前身，Gboard 現況不一定相同。路徑省略 `native/jni/src/dictionary/` 前綴時以 `…/` 標示。

### 2.1 新詞要出現幾次

沒有「第 N 次才算」的設定，但字典內外有差別。`DictionaryFacilitatorImpl.addWordToUserHistory`（`java/src/com/android/inputmethod/latin/DictionaryFacilitatorImpl.java`）：

```java
// We demote unrecognized words (frequency < 0, below) by specifying them as "invalid".
// We don't add words with 0-frequency (assuming they would be profanity etc.).
final boolean isValid = maxFreq > 0;
UserHistoryDictionary.addToDictionary(userHistoryDictionary, ngramContext, secondWord,
        isValid, timeStampInSeconds);
```

native 端（`…/structure/v4/ver4_patricia_trie_policy.cpp` 的 `updateEntriesForWordWithNgramContext`）：詞不在使用者歷史時先建 unigram，`HistoricalInfo(timestamp, 0 /* level */, 0 /* count */)`；`isValidWord` 為假就**直接 return**，不加次數。查詢時（`…/structure/v4/content/language_model_dict_content.cpp`）：

```cpp
if (mHasHistoricalInfo && unigramProbabilityEntry.getHistoricalInfo()->getCount() == 0) {
    // The word should be treated as a invalid word.
    return WordAttributes();
}
```

所以**主字典沒有的詞（未知字串，如 email）第一次上屏只建空殼、`count=0` 不可見；第二次 `count=1` 才可見**；主字典有的詞第一次就 `count=1`。這是由原始碼推得。HeliBoard 作者的實測註解與之相符，見 §4。

`UserHistoryDictionary.addToDictionary` 在 `word.length() > BinaryDictionary.DICTIONARY_MAX_WORD_LENGTH`（48）時直接不記；沒有最短長度。

### 2.2 遺忘：時間與計數

本快照使用者歷史的機率是**計數**算的，不是 forgetting-curve 的等級表（和 `remember-strings.md` §1 把 `forgetting_curve_utils.cpp` 的等級、15 天當作現行衰減不同，那段要更正）。依據：

- `LanguageModelDictContent::createUpdatedEntryFrom` 寫入時 level 固定為 0，`count = 原 count + 本次 count`。
- `getWordAttributes` 用 `computeRawProbabilityFromCounts(count, contextCount, ngramType)`；`getDecayedProbability` 的註解與實作都是不隨時間改（`dynamic_language_model_probability_utils.h`）：`// We don't modify probability depending on the elapsed time.`
- 我讀的 `ver4_patricia_trie_policy.cpp`、`language_model_dict_content.cpp`、`ver4_patricia_trie_writing_helper.cpp` 裡，`ForgettingCurveUtils` 只出現在 `needsToDecay`（觸發 GC）與條目上限（`getEntryCountHardLimit`）。沒有逐檔掃描整個 native 目錄，所以其他路徑是否用到等級表，未查到。

GC 時的真實遺忘（`language_model_dict_content.cpp` 的 `updateAllProbabilityEntriesForGCInner`、`dynamic_language_model_probability_utils.{h,cpp}`）：

```cpp
// dynamic_language_model_probability_utils.cpp
// This value is used to remove too old entries from the dictionary.
const int DynamicLanguageModelProbabilityUtils::DURATION_TO_DISCARD_ENTRY_IN_SECONDS =
        300 * 24 * 60 * 60; // 300 days
```

- 最後使用時間超過 **300 天**的條目在 GC 時移除。
- 全域計數器接近上限（`COUNTER_VALUE_NEAR_LIMIT_THRESHOLD`、`TOTAL_COUNT_VALUE_NEAR_LIMIT_THRESHOLD = 1<<30`，`language_model_dict_content_global_counters.cpp`）時，所有計數**減半**，減半後為 0 的條目移除。
- 機率的分母有下限 `ASSUMED_MIN_COUNTS = {8192, 2, 2, 1}`（unigram 的 `contextCount` 至少當 8192），註解「Used to provide stable probabilities even if the user's input count is small」，所以剛開始用、次數少時不會被抬太高。

另一支仍在的 `forgetting_curve_utils.cpp` 常數（`…/utils/forgetting_curve_utils.cpp:28-40`）：`DECAY_INTERVAL_SECONDS = 2 * 60 * 60`、`MAX_LEVEL = 15`、`DURATION_TO_LOWER_THE_LEVEL_IN_SECONDS = 15 * 24 * 60 * 60`、`ENTRY_COUNT_HARD_LIMIT_WEIGHT = 1.2`、`DISCARD_LEVEL_ZERO_ENTRY_TIME_STEP_COUNT_THRESHOLD = 30`。其中只有 2 小時的 GC 檢查間隔與 1.2 的硬上限係數在上面確認的路徑上。

### 2.3 上限與淘汰

- 預設 `DEFAULT_MAX_NGRAM_COUNTS[] = {10000, 30000, 30000, 30000}`（`…/header/header_policy.cpp:38`；unigram 1 萬、bigram 3 萬）。Header 的 `MAX_UNIGRAM_ENTRY_COUNT` 可覆寫，Java 層沒找到覆寫（`remember-strings.md` 寫「Java 層沒找到預設值」，預設值在 native，這裡補上）。
- **硬上限 = 上限 × 1.2**（`getEntryCountHardLimit`：1.2 萬／3.6 萬）。條目數達硬上限就觸發 GC（`needsToDecay`）；GC 後再 `truncateEntries` 截到 `maxEntryCounts`。
- 淘汰順序（`EntryInfoToTurncate::Comparator`）：先比 `priority`（= 最後使用的 timestamp，越舊越先被刪），再比 `count`（小的先刪），再比 key。程式註解：`// More recently input entries get higher priority.`
- 每一層（unigram、bigram…）各自截斷。

### 2.4 刪除、隱私

- 單詞刪除：`removeUnigramEntryDynamically(word)`，直接從 trie 移除，無墓碑（單機詞典沒有同步）。長按建議拖垃圾桶走這條。
- 修正時 `unlearnFromUserHistory`：事件不是 `EVENT_BACKSPACE` 才移除該詞（`DictionaryFacilitatorImpl.java`，原碼有 `TODO: Decide whether or not to remove the word on EVENT_BACKSPACE`）。
- 密碼欄、自動修正關閉、慢的 InputConnection 不學：見 `remember-strings.md` §1。

## 3. libime／fcitx5

模型細節見 `libime-user-model.md` §2–§5。這裡只補與本題有關的：

- **拼音歷史上限**：3 個池，容量 `{128, 8192, 65536}` 句；池滿時最舊的句子轉進下一池，最後一池滿了就消失。**淘汰以句序為準，沒有時間**（`libime-user-model.md` §4）。
- **新詞門檻**：拼音沒有門檻，上屏即寫歷史。table 引擎的自動造詞有：`TableOptions::saveAutoPhraseAfter`（預設 `-1` 即不升格，`src/libime/table/tableoptions.cpp:31`），設為 N≥1 時，auto 項（獨立的 256 筆 LRU `AutoPhraseDict`）命中到第 N 次才升格成 User 項。原文（`src/libime/table/tablebaseddictionary.cpp:1265-1276`）：

```cpp
case PhraseFlag::Auto: {
    const auto entry = generateTableEntry(key, value);
    auto hit = d->autoPhraseDict_.exactSearch(entry);
    if (tableOptions().saveAutoPhraseAfter() >= 1 &&
        static_cast<uint32_t>(tableOptions().saveAutoPhraseAfter()) <= hit + 1) {
        d->autoPhraseDict_.erase(entry);
        insert(key, value, PhraseFlag::User, false);
    } else {
        d->autoPhraseDict_.insert(entry);
    }
} break;
```

  這是兩層結構：候選區（容量 256，LRU，命中次數）→ 達門檻才進正式使用者詞。各碼表的預設值在 fcitx5-chinese-addons 的 `im/table/*.conf.in.in`，未逐一查。
- **table 的 User 項沒有計數也沒有上限**：只記單調遞增的最後使用序（`libime-user-model.md` §5）；條目數上限未查到。
- **刪除**：`forget(word, code)` 把含該詞的整句從所有池移除；`clear()` 全清；table 有 `removeWord`。fcitx5 拼音引擎有「從歷史刪詞」的選字模式與「清除用戶詞典」動作（`remember-strings.md` §7）。沒有墓碑（單機）。
- **同步**：fcitx5 的使用者詞典是各自的檔案，本次未查到內建的跨裝置合併。

## 4. HeliBoard（LatinIME 的 fork）

`app/src/main/java/helium314/keyboard/latin/DictionaryFacilitatorImpl.kt`，native 目錄同樣有 `forgetting_curve_utils.cpp`（`DURATION_TO_LOWER_THE_LEVEL_IN_SECONDS = 15 * 24 * 60 * 60`）與 `dynamic_language_model_probability_utils.cpp`（300 天），和 AOSP 同值。學習邏輯基本沿用，但多了兩處值得借的做法：

1. **延遲寫入以便更正**（約 L81、L411）：`private val unlearnDuration = 25.seconds`；`addWordToUserHistory` 在 `delay(unlearnDuration)` 之後才呼叫 `UserHistoryDictionary.addToDictionary`，期間使用者改字會取消（`wordsWaitingForHistory`）。註解：`Adding words to history is delayed by unlearnDuration.`。
2. **歷史詞升格成個人詞典的門檻**（約 L428-448），含作者實測的原文：

```kotlin
// User history always reports words as invalid, so we check the frequency instead.
// Testing shows that after 2 times adding, the frequency is 111, and then rises slowly with usage (values vary slightly).
// 120 is after 3 uses of the word, so we simply require more than that. todo: Could be made configurable.
if (userHistoryDict.getFrequency(word) > 120) { ... UserDictionary.Words.addWord(userDict.mContext, word, 250, null, dictionaryGroup.locale) }
```

  條件另有 `word.length <= 1` 不處理、字典已有的詞不處理。這同時旁證 §2.1：未知詞第 2 次才有頻率（「after 2 times adding, the frequency is 111」）。

刪除：`removeUnigramEntryDynamically`（歷史、使用者、通訊錄、應用程式詞典各一行，約 L715-729）；`clear()` 清歷史（約 L605）。

## 5. AnySoftKeyboard

`ime/app/src/main/java/com/anysoftkeyboard/dictionaries/sqlite/AutoDictionary.java`（L60-83）、`dictionaries/SuggestImpl.java`、`dictionaries/Suggest.java`。

兩層：`AutoDictionary`（候選區，SQLite `auto_dict_2.db`，`isValidWord` 恆 false）累積分數，達門檻才升到使用者詞典。

```java
// Don't add words if they're too long, too short, or user has decided not to learn words:
if (length < 2 || length > MAX_WORD_LENGTH || mLearnWordThreshold == -1) return false;
int freq = getWordFrequency(word);
freq = freq < 0 ? frequencyDelta : freq + frequencyDelta;
if (freq >= mLearnWordThreshold) {
  Logger.i(TAG, "Promoting the word '%s' to the user dictionary. It earned it.", word);
  // no need for this word in this dictionary any longer
  deleteWord(word);
  return true;
} else {
  super.addWord(word, freq);
  return false;
}
```

- 加分：`AdditionType.Picked(3)`（從建議選的）、`Typed(2)`（自己打的）。
- 門檻 `settings_key_auto_dictionary_threshold`，值 `9`（預設，「After some usage」）、`1`（「Immediately after I type it」）、`-1`（「Do not learn new words」）；`res/values/settings_keys_dont_translate.xml`、`settings_defaults_dont_translate.xml:32`。預設 9 分 = 打 5 次，或選 3 次，或混合（如打 3 次 + 選 1 次）。
- 最短長度 2；`mIncognitoMode` 時不學（`SuggestionsProvider.tryToLearnNewWord`）。
- 累計分數不隨時間衰減；候選區的上限與過期：未查到。

## 6. FlorisBoard

`app/src/main/kotlin/dev/patrickgold/florisboard/ime/dictionary/UserDictionary.kt`：使用者詞典是 Room 資料表，欄位含 `freq`（`FREQUENCY_MIN = 1`、`FREQUENCY_MAX = 255`、`FREQUENCY_DEFAULT = 128`），來源是使用者手動新增與匯入／匯出。我讀的 `UserDictionary.kt`、`NlpProviders.kt` 沒有自動學習、門檻、衰減或上限的邏輯；專案內搜尋 `learn` 只命中 NLP 介面註解（「should only provide suggestions based on already learned data」）。其他檔案是否有自動學習：未查到。可當作「沒有自動學習，只有手動詞典與頻率欄」的對照。

## 7. SwiftKey、Gboard 官方說明

- **SwiftKey**（<https://support.microsoft.com/en-us/swiftkey-keyboard/microsoft-swiftkey-keyboard-sharing-your-typing-data-faq>）：只說「The service stores the fragments in a format that can be synchronised with any other devices where you use SwiftKey」，以及可在設定中要求刪除資料。**合併規則、刪除是否傳到其他裝置、過期、上限：未查到**。
- **Gboard**（<https://support.google.com/gboard/answer/12373137?hl=en>）：「Delete learned words & data」「Clears all on-device data that Gboard has saved」。**跨裝置同步學到的詞、同步刪除、上限、過期：頁面沒有說明，未查到**。

## 8. 一般的同步合併做法（只取與使用者詞庫有關的）

來源：Wikipedia 的 CRDT 條目 <https://en.wikipedia.org/wiki/Conflict-free_replicated_data_type>（定義的轉述，原論文 Shapiro et al. 2011 的 PDF 本次被擋，未取得原文）；Shapiro 的 *Replicated Data Types*（Encyclopedia of Database Systems 2016 作者版）<https://hal.science/hal-01578910v1/file/replicated-data-types-Encyclopedia-DB-systems-2016-authorversion.pdf>；Cassandra 文件 <https://cassandra.apache.org/doc/latest/cassandra/managing/operating/compaction/overview.html>。

| 做法 | 規則 | 對使用者詞的意義 |
| --- | --- | --- |
| G-Counter | 每台裝置一格，只增自己的格；合併「取每格的 max」，值為各格總和 | 次數不會重複計算、重複合併安全（idempotent）；代價是要存每台裝置一格 |
| PN-Counter | 兩個 G-Counter（加、減），合併各自取 max | 能表示「減」，但不是「刪除」；衰減（乘係數）不是單調的，不能直接用這兩種表示 |
| 單一整數取 max（Rime 的 `c`） | 兩邊取大 | 合併安全、冪等；兩台各自在同一基準上 +1 時只算一次（少計）。使用者詞的次數本來就近似，可接受 |
| 2P-Set | 加入集合＋墓碑集合，remove-wins，刪除後不能再加 | 刪了就不能重新學，不適合使用者詞 |
| LWW-Element-Set | 每個元素帶加入時間、刪除時間，較晚者勝；時間相等時由「bias」（偏加入或偏刪除）決定 | 簡單；要靠牆鐘，裝置時鐘誤差會讓刪除或復活判錯 |
| OR-Set | 加入時帶唯一 tag，刪除只刪「已觀察到的 tag」，並行的新加入存活（add-wins） | 精確表達「刪掉我看到的那些，別台後來新學的不受影響」；tag 或墓碑會累積，需要回收 |
| 墓碑回收 | Cassandra：`gc_grace_seconds` 預設 864000 秒（10 天），「the purpose of the grace period is to give unresponsive nodes time to recover and process tombstones normally」，超過期限才在 compaction 時移除；沒收到墓碑的節點若在期限後才回來，會把刪除的資料「resurrected as a zombie」 | 墓碑不能立刻丟；回收時間要長於「最久沒同步的裝置回來」的時間 |

## 9. 比較表

| 產品 | 新詞門檻 | 最短／最長 | 遺忘 | 條目上限與淘汰 | 刪除 | 跨裝置合併 |
| --- | --- | --- | --- | --- | --- | --- |
| librime | 無（每次上屏即記） | 無 | 計數 `d` 以上屏次數衰減，時間常數 200；`d ≤ 1e-200`（約 9.2 萬次）查詢時略過 | 未查到（只增不刪檔） | `c` 取負（保留 `|c|`），可被再次上屏復活；墓碑不回收 | 快照逐筆合併：`c` 取 `|c|` 大者、`d` 取大、`t` 取大；刪除與正值同 `|c|` 時不收斂（由原始碼推得） |
| AOSP LatinIME | 字典外的詞第 2 次才可見（由原始碼推得），字典內的詞第 1 次 | 最長 48 | 計數，不隨時間改機率；GC 時 300 天未用移除；計數近上限時全部減半 | unigram 1 萬、bigram 3 萬；硬上限 ×1.2 觸發 GC；淘汰最舊 timestamp，同則較小 count | 直接移除，無墓碑 | 無（單機） |
| HeliBoard | 沿用 AOSP；升格到個人詞典需歷史頻率 >120（約第 3 次） | 長度 >1 才升格 | 同 AOSP | 同 AOSP | 同 AOSP | 無 |
| AnySoftKeyboard | 累計分數：打 +2、選 +3，預設達 9 升格；可選 1 或關閉 | 最短 2、有最長 | 無衰減 | 未查到 | 升格後從候選區刪 | 無 |
| libime 拼音 | 無 | 無 | 句序（三池 128／8192／65536 句），無時間 | 池滿後最舊句子轉下池，最後一池滿則消失 | `forget` 整句移除、`clear` | 未查到 |
| libime table | 自動造詞可設 N 次升格（預設關），候選區 256 筆 LRU | 自動造詞長度 `autoPhraseLength` | 只記最後使用序 | 未查到 | `removeWord` | 未查到 |
| FlorisBoard | 無自動學習（手動詞典） | — | — | — | 手動 | 未查到 |
| SwiftKey、Gboard | 未查到 | 未查到 | 未查到 | 未查到 | 有「清除全部」 | 未查到 |

## 10. 對 QIM 的建議

M4 要定的四件事：出現門檻、只算最近一段、條目上限、v3 匯出的刪除墓碑。

### 10.1 有來源支持

1. **用上屏次數當時鐘，與 librime 同。** tick 只在「學習性的上屏」時 +1，刪除與查詢不推進（§1.1）。「次數只算最近一段」可直接沿用同一個機制：窗口以 tick 計（brief §4.5 已訂初值 1 萬次）。librime 的 `d` 以 e^(−Δtick/200) 衰減、約 9.2 萬次後視為失效（§1.1）；QIM 的窗口是更窄的「兩次要在 1 萬次內」，不衝突，是另一個量。
2. **新詞門檻有三種成熟形式，可混用**：(a) 字典外的詞要第 2 次才可見（LatinIME，§2.1）；(b) 累計分數達門檻，且「選」比「打」權重高（AnySoftKeyboard 3 vs 2、門檻 9，§5）；(c) 兩層：候選區（容量小、LRU）→ 達次數才進正式使用者詞（AnySoftKeyboard、libime table 的 `saveAutoPhraseAfter`，§3、§5）。「在一段時間內打過幾次」是 (a)(c) 的時間版，沒有成熟產品用時間窗口，見 10.2。
3. **最短長度**：AnySoftKeyboard 要 ≥2，LatinIME 沒有最短、有最長 48（§2.1）。HeliBoard 升格也要 >1（§4）。
4. **更正要有緩衝**：HeliBoard 延遲 25 秒才寫歷史，讓改字取消（§4）；LatinIME 在修正時移除（§2.4）。對應 `remember-strings.md` §9 記的「退格撤銷沒做」的缺口。
5. **上限用兩段式**：到「硬上限（1.2×）」才觸發清理，清到「上限」；淘汰按最後使用時間，同則按次數（LatinIME，§2.3）。這避免每次寫入都做淘汰。
6. **上限的常見量級**：LatinIME 預設 unigram 1 萬（§2.3）；libime 拼音歷史最多約 7.4 萬句（§3）；Baidu 自動清最不常用 10% 的自造詞（`remember-strings.md` §5）。
7. **刪除沿用「負值＋再次出現復活」的模型**（librime，§1.3），單機成熟產品都有單條刪除（§2.4、§3）。
8. **墓碑不可立刻丟，回收期要長於最久未同步的裝置回來的時間**（Cassandra 的 10 天 `gc_grace_seconds`，§8）；沒有墓碑的並行刪除會讓資料復活（同頁）。
9. **librime 的同步合併對刪除不收斂**（§1.4 的 1，由原始碼推得）。v3 加墓碑正是補這個缺口；`UserDbImporter` 的做法（刪除一定蓋過）是 librime 自己的另一條路，可旁證「刪除蓋過」是被接受的語義。

### 10.2 我的推論（無來源，需要實測或決定）

1. **次數窗口的表示**。沒有產品公開做過「最近 N 次上屏內出現 ≥2 次」。最省的表示是每筆只存最近兩次上屏的 tick（`t1 < t2`），條件 `t2 − t1 ≤ 窗口` 時才算過門檻，再打一次時 `(t1,t2) ← (t2,tick)`；不必存歷史清單。兩台裝置的 tick 是各自的計數（Rime 也是），不能直接比較，合併時這兩個值怎麼處理需要決定：最簡單是合併後視為「已通過門檻」就保留旗標（通過後不再需要 tick），沒通過的各自保留。
2. **墓碑用「刪除時看到的 `c`」而不是時間**（類似 OR-Set 的 observed-remove）。每筆存 `c`（合併取 max）與 `del`（刪除時的 `c`，合併取 max）；存活條件 `c > del`。刪除後再上屏 `c` 變 `del+1`，即復活；合併時另一台沒看到刪除、`c` 仍是 `del`，所以刪除傳得過去，另一台在刪除後才新增的次數會復活它。比 librime 的 `|c|` 比較強的地方：同值時刪除勝（librime 是同值時不收斂）。缺點同 Rime：兩台並行各 +1 只算一次（max-merge 少計），但對近似次數可接受。
3. **不使用牆鐘（LWW）**。裝置時鐘誤差會讓刪除與復活判錯；`c`／`del` 的做法不依賴時鐘。
4. **墓碑回收**：墓碑只有在「任何一台裝置上對應的條目都已因窗口過期而不再起作用」之後丟掉才不會造成有害的復活；實務上取一個與窗口同量級、且大於最久未同步時間的期限（類比 Cassandra 的 10 天，但 QIM 的時間單位是上屏次數，換算需要決定）。tick 是裝置本地的，回收期限若用 tick 算，只能在本機判斷，不保證其他裝置已看到墓碑；若要保證，需要在匯出裡記牆鐘時間做粗略期限（只用於回收，不用於決勝負）。
5. **匯出 v3 要保持「匯入是合併」**。librime 的 Restore／Import 都是合併不是取代（`userdict-text-formats.md` §1.6–§1.7），v3 沿用，並建議匯入自己的舊快照時冪等（`max`、`min`、`>` 這類運算天然冪等，但 `d` 的衰減要先對齊 tick，見 §1.4 的 `Put` 原文）。
6. **上限數字**：沒有產品公開理由。一個起點是 LatinIME 的 1 萬條（unigram）、硬上限 1.2 倍；QIM 的使用者詞含 email 與英文字串，筆數遠少於一般詞典，數字需要用本機資料量測再定。淘汰順序可直接用「最後使用的 tick 最舊，同則 `c` 小」，墓碑不計入上限但要另設回收。
7. **記住的 email 與英文字串併成一種使用者詞後，門檻要分流**。email 通常只打幾次，要求第 2 次才可見會損失價值（`remember-strings.md` §8 建議 1 次就記）；LatinIME 對字典外的詞要第 2 次才可見，兩者衝突。M4 要明確選一邊，並把「字典外」定義成 QIM 的哪一類（英文詞表未收的字串？）。這是設計決定，來源只能提供兩邊的做法。

### 10.3 未查到，M4 不要寫成「業界標準」

- 時間窗口形式的出現門檻（沒有產品公開）。
- Rime 以外的輸入法的跨裝置合併規則：SwiftKey、Gboard 只有官方「可同步」或「可清除」的描述，沒有合併與刪除傳播的說明。
- librime、libime table 的條目數上限。
- 各 table 方案 `saveAutoPhraseAfter` 的實際預設值。
