# libime 的使用者模型：歷史資料如何併入 lattice 分數

檢視日 2026-11-04；fcitx/libime master `53df6a248793f9a00c1769e658630de313f3e419`；授權 LGPL-2.1-or-later（`LICENSES/LGPL-2.1-or-later.txt`，各檔 SPDX 標頭同）。工單寫「GPL 只取做法」，實際是 LGPL-2.1+；不論哪種，我們只取做法與公式，不抄程式碼（見 ADR 0006）。路徑都在 `src/libime/` 下。

## 1. 分數合併

`UserLanguageModel::score`（`core/userlanguagemodel.cpp:122-140`）：

```cpp
score = LanguageModel::score(state, word, out);          // 靜態 LM，log10
userScore = history_.score(prev, &word);                  // 歷史，log10
return std::max(score, sum_log_prob(score + wa_, userScore + wb_));
// wa_ = log10(1 - w), wb_ = log10(w), w = 0.2   (:36-39, constants.h:19)
```

- 全部是 **log10** 機率。`sum_log_prob(a, b)` = `log10(10^a + 10^b)`（`:111-120`），即 `log10((1-w)·P_static + w·P_user)`。
- 外層 `max(score, ...)`：混合只會**抬高**分數，不會比純靜態 LM 低。歷史沒見過的詞，混合值約 `score + log10(0.8)`，被 `max` 吃掉，等於不影響。
- 靜態 LM 本身：`kenlm Score + (idx==unknown ? unknownPenalty : 0)`（`core/languagemodel.cpp:206-219`）。unknown penalty 預設 `1/60000000` 即 log10 ≈ -7.78（`constants.h:13`）。
- `prev` 存在 `State` 尾端（`wordFromState`，`:41-50`），所以歷史 bigram 的前一詞跟著 lattice 路徑走。`useOnlyUnigram_` 時 LM 用 `nullState_` 並關掉歷史 bigram（`:126-128`、`:152-156`）。
- `isUnknown` 也看歷史：LM 未知且歷史也沒有才算未知（`:142-145`）。

## 2. 歷史機率

`HistoryBigram::scoreWithCode`（`core/historybigram.cpp:660-688`）：

```
bigramWeight = 0.8 (useOnlyUnigram 時 0), unigramWeight = 0.2
pr = unigramWeight * unigramFreq(cur) / (unigramSize + poolWeight[0]/2)
   + bigramWeight  * bigramFreq(prev,cur) / (unigramFreq(prev) + poolWeight[0]/2)
pr = min(pr, 1); pr == 0 ? unknown_ : log10(pr)
```

- `prev` 空字串 → `<s>`，`cur` 空字串 → `<unk>`（`:663-668`）。
- `unigramFreq` / `bigramFreq`（`:541-557`）= 各池原始計數 × 該池權重的總和；池內計數是整數（`WeightedTrie`，DATrie 的值，`:82-253`）。
- **分母的設計**：池權重讓 `unigramSize()` = Σ(池容量 × 池權重) 恰為 1.0（見 §4），所以 `unigramFreq/unigramSize` 就是「加權計數占滿載歷史的比例」，不是占實際已學句數的比例。bigram 項則除以前一詞的加權 unigram 計數（條件機率）。兩個分母都加 `poolWeight[0]/2`（約 0.0008，註解 "avoid div 0"）。
- 沒見過的詞：unigram 與 bigram 計數皆 0 → `pr == 0` → 回傳 `unknown_`（log10 ≈ -7.78，與 LM 的 unknown 同值；`:568-569`、`:683-685`）。
- 數量級：一個詞只學過 1 次（在第 0 池）時，`unigramFreq ≈ 0.0016`，unigram 項 ≈ 0.2 × 0.0016 = 3e-4；若它也有 bigram（前詞 count 1 時 bigram 項 ≈ 0.8 × 1），歷史機率可達 ~0.8，這就是「同一個詞接在同一個前詞後」被大幅抬高的來源。
- 有「驗證碼」變體（`WordWithCode`，詞 + 碼，用 `\x02` 分隔，`:41-44`）：key 同時匹配 `word` 與 `word\x02code`（`WeightedTrie::freq`，`:94-128`）。table 引擎用它把學習綁在碼上。

## 3. 何時學、學什麼

- 歷史的單位是**句子**（`vector<WordWithCode>`），`HistoryBigramPool::add`（`:383-419`）對每個詞 `unigram +1`、相鄰詞 `bigram +1`，另加 `<s>`→首詞、末詞→`</s>` 的 bigram 與 `<s>`、`</s>` 的 unigram。含 `\0` 的詞整句丟棄（`:389-394`）。
- **拼音** `PinyinContext::learn()`（`pinyin/pinyincontext.cpp:1089-1119`）：
  - 只在 `selected()`（整句都選定）時學。
  - `learnWord()`（`:328-403`）：只選了一個多字詞（`selected_.size()==1 && selected_[0].size()==1`）不學新詞；多段選字會合成新詞寫進 user 詞庫（`addWord(UserDict, pinyin, ss, cost 0)`，有自訂段為 -1），條件：非純單字時總拼音音節 ≤ 4，且 `containsNonUnigram`（LM 或歷史已有 bigram）為真就不造詞。
  - 歷史寫入：造出新詞 → 把整句當**一個詞**寫進（`newSentence = {sentence(), encodedPinyin}`），自訂詞第一次不寫歷史；否則寫 `selectedWordsWithPinyin()`（每個選定 segment 一個詞 + 拼音碼）。**任一詞沒有拼音（非拼音字）整句不寫**（`:1109-1114`）。
  - `addWithContext(contextWords, newSentence)`（`core/historybigram.cpp:829-836`）：若新句緊接著第 0 池最新那句的結尾（context 相符），改成**續接**最新句（拆掉舊的 `</s>` bigram），否則另開新句。
- **碼表** `TableContext::learn()`（`table/tablecontext.cpp:740-784`）：`tableOptions().learning()` 為 false 整個跳過；每個 `selected_` 項先 `learnWord`（見 §5），再把 **已 commit** 的每個選定 segment 當一個詞寫進歷史，詞 = 該 segment 的字串（多個子項則串接，碼由 `dict_.generate` 造，失敗就中止），碼 = 輸入碼；任一子項未 commit 的 segment 略過。`learnLast()`（`:786-821`）是只學最後一個 segment 的版本。
- 沒有「密碼框 / 不學習模式」的邏輯在 libime 內，那是上層 fcitx5 引擎的責任（本次未讀 fcitx5-chinese-addons）。

## 4. 衰減與池

- 3 個池，容量（句數）`{128, 8192, 65536}`（`:579`）。每池是一個 `recent_` 句子 list（新的在前）+ unigram/bigram 兩棵計數 trie。
- **老化 = 句子被擠出去**：`add` 時若 `recent_.size() >= maxSize_`，從尾端 pop 最舊句子、把它的計數全部 `-1`（`remove`，`:395-399`、`:484-496`），被 pop 的句子 `populateSentence` 轉進下一個池（`:529-539`）。最後一池擠出去就消失。
- 沒有時間衰減，只有「句序」衰減。**每筆計數都是整數 1，衰減全靠池權重**：`p = 1/(1+α)`，`α = 0.25`（`constants.h:16`）→ `p = 0.8`；池 i 的總質量 = `(1-p)·p^i`，最後一池拿剩下的 `p^(n-1)`：0.2 / 0.16 / 0.64。每池權重 = 質量 / 容量（`:578-590`）→ 約 `0.2/128 = 1.56e-3`、`0.16/8192 = 1.95e-5`、`0.64/65536 = 9.8e-6`。註解稱「older selections keep 64% of total history weight」。
- 因此一句剛學的詞每出現一次貢獻 1.56e-3；被擠到第 2 池後貢獻降為 1/80；第 3 池再降一半。
- `forget(word, code)`（`:435-449`、`:750-756`）：把含該詞（且碼符合）的**整句**從所有池移除並扣計數。`clear()` 全清。
- 持久化：zstd 壓縮、版本 4，只存各池的句子序列（舊→新），載入時重放 `add`（`:259-282`、`:728-737`），計數是重建的。

## 5. table 引擎的學習順序與自動造詞

- **user 詞庫的順序 = 單調遞增 index**：`insertOrUpdateTrie`（`table/tablebaseddictionary.cpp:182-191`）每次 `trie.set(entry, index); index += 1`，且註解「即使已存在也要重插，因為要更新 index」。所以 index 越大 = 越近期使用過；不存計數，只存「最後使用序」。
- 候選比較器 `TableCandidateCompare`（`table/tablecontext.cpp:60-121`）：先看 `noSortInputLength`（短碼固定序，且 User 項在短碼時不回傳，`tablebaseddictionary.cpp:1644-1650`）、`sortByCodeLength`；之後：
  - `OrderPolicy::No` / `Fast`：`index` 大的在前（User 的 index 高於一般詞）。
  - `OrderPolicy::Freq`：先比 lattice `score`（含上述 UserLanguageModel 分數），分數相同再比 `index`。（`OrderPolicy` 定義見 `table/tableoptions.h:19-23`。）
  - 去重時（`:140-170`）`Fast`/`Freq` 讓 User 旗標的項勝出，`No` 不讓。
- **user 項寫入**（`TableContext::learnWord`，`:265-289`）：選單一詞 → `insert(code, word, User)`（Auto 先 `removeWord` 再轉 User）；多段 → 把串接詞用規則造碼後 `insert(word, User)`，任一段未 commit 則不造詞。
- **自動造詞**（`learnAutoPhrase`，`:823-870`）：輸入是上屏歷史字串；`learning` 為 false 或 `autoPhraseLength <= 1` 跳過；對歷史字串每個起點取長度 2..`autoPhraseLength` 的尾段，用造詞規則 `generateWithHint` 產碼；已存在為一般詞或 User 就略過；否則 `insert(code, word, Auto)`。
- Auto 項存在獨立的 `AutoPhraseDict`：容量 256 的 LRU（`TABLE_AUTOPHRASE_SIZE`，`table/constants.h:10`），`insert` 放最前並累計 `hit_`（`table/autophrasedict.cpp`）。`insert(..., Auto)`（`tablebaseddictionary.cpp:1265-1276`）：若 `saveAutoPhraseAfter >= 1` 且命中次數 +1 達到門檻，就從 auto 刪除並轉成 `User` 項（進入上面的 index 序列）。

## 6. 對 QIM 的取用

M1 票 13 採用的做法（plan §2.3、§2.4）：以（方案, 輸入碼, 字詞）為鍵，記上屏次數 c、衰減值 d（librime `formula_d`：`d ← 1 + d·e^(−Δtick/200)`）、最後 tick t；個人機率代入 `max(static, ln((1−w)·P_static + w·P_personal))`，w = 0.2。

- **混合 + `max`**：照 libime，歷史只加分不扣分，沒學過的段不受影響。libime 用 log10，QIM 用自然對數；公式裡只有比例，底數不影響結果。
- **個人機率的分母取常數**：`P_personal = d_now / D_max`，`D_max = 1/(1−e^(−1/200)) ≈ 200.5` 是衰減值的上限。對應 libime 除以「滿載歷史的總質量」（§2），不是除以已學的總數，所以剛開始用、只學過幾次時不會把學過的段捧得太高。`P_personal` 天然在 (0, 1]，不必另外截頂或加 ε。
  - 試過分母改成「實際上屏的段數的衰減總和」（較貼近使用者自己的 unigram）：`bench --learning` 的四個公開集合修正次數都比常數分母多（數字在 M1 票 13 Comments），沒採用。
- **鍵含輸入碼**：等於 libime 的 `WordWithCode` 變體（§2），同一個字在不同碼下分開記。libime 的 unigram 0.2 + bigram 0.8 兩項只取前者；個人 bigram 留到 M3（plan §2.3）。
- **衰減**：libime 靠句序與三個池（§4），QIM 用 librime 的連續版，不必保存句子佇列；算分時先把 d 衰減到現在的 tick。
- **學什麼**：同 libime table 引擎，上屏時才學，選定的每一段各寫一筆；另外整串也寫一筆，給排位規則 2 比對整串輸入碼。無痕模式（libime 沒有，在 fcitx5 上層）由 `Options.incognito` 關掉。
- 不取：整句 `recent_` 佇列、zstd 序列化、以整句為單位的 `forget`（QIM 之後的刪除以鍵為單位）、table 引擎只記「最後使用序」的 index（QIM 另記次數，plan §2.4 的規則 2 要用）。
