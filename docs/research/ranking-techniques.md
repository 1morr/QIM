# 輸入法候選排序：技術、資料需求與成本

階段 1 研究。目標讀者：沒做過輸入法的人。查證日期 2026-10-30。

標記說明：
- 「[S#]」對應文末〈來源〉。有 [S#] 的數字都是來源原文給的。
- 「估計」是我用來源的數字或公式自己算的，會寫怎麼算。
- 「推論」是邏輯推演，不是來源的結論。
- 「未查證」是沒找到第一手來源，不要當事實用。

---

## 0. 摘要

1. 速成連打的核心問題是「一串不分隔的按鍵有很多種切法，每種切法的每一段又有很多候選字」。標準做法是把所有切法畫成一張 lattice（word graph），再用 Viterbi（動態規劃）找分數最高的路徑。libime、librime、Mozc、Gboard 都是這個結構 [S6][S7][S8][S1]。
2. 排序只是「給 lattice 上每條邊和每個相鄰邊對一個分數」。分數的來源由簡到繁是：unigram 詞頻 → bigram/trigram → 使用者歷史 → 神經 LM → LLM。每往上一層，改善的是「同音/同碼時選對的機率」，代價是資料、大小、延遲都上升。
3. 端上的 n-gram 是成熟、便宜的：Gboard 英文靜態 LM 是 5-gram、125 萬條 n-gram [S2]，文獻說鍵盤 LM 不應超過 5–10 MB [S1]；Rime 的繁體字級 bigram 模型壓縮版約 10 MB、詞級約 41 MB [S12]。
4. 按鍵回饋要在約 20 ms 內 [S1][S2]；Nielsen 引 Miller 1968 / Card 1991，約 0.1 秒是「感覺即時」的上限 [S14]。沒找到專為 IME「按鍵到候選」延遲寫的論文（未查證），所以 50–100 ms 是由這兩個數字推出來的工程預算，不是文獻結論。
5. 端上神經 LM 有先例：Gboard 的 CIFG-LSTM 約 140 萬參數、量化後 1.4 MB、詞表 1 萬 [S3]；MobileBERT 在 Pixel 4 上 62 ms [S15]。但 Gboard 的 CIFG 是「下一詞預測」用，不是 lattice rescoring；以神經 LM 重排 lattice 在鍵盤上的公開延遲數字未查證。
6. LLM 在端上可行但不適合「每鍵都跑」：iPhone 15 Pro 跑 TinyLlama 1.1B（Q4_0）prompt 處理 646 token/s、生成 57 token/s；Phi-2 2.7B 級（3B 欄）分別是 157 與 25 token/s [S16]。Apple 自家 3B 模型每個 prompt token 約 0.6 ms、生成 30 token/s [S10]。適合「候選已出、使用者停頓後」的非同步重排。
7. 容錯有兩套成熟做法：libime 用「相鄰鍵（只限同一排左右鍵）」展開 lattice 邊 [S6]；Gboard 用以按鍵中心為均值的 Gaussian 空間模型，並用 FST 處理替換、漏打、多打 [S1]。
8. 使用者學習：librime 用「指數衰減計數 + 提交次數」公式 [S7]；libime 用三個大小遞增的 LRU 池（128 / 8192 / 65536 句）把歷史 bigram 以固定權重混進靜態 LM [S6]；Mozc 的使用者歷史主要靠 LRU（last_access_time）加 bigram 加成，並有 Revert 機制 [S8]。
9. 給 QIM 的第一層建議看「三個案例要在哪一層才選對」：見第 10 節。結論先說：案例 `anr`（門口 vs 問）需要詞級條目與詞頻，單字 unigram 不夠（推論，第 1 節）；案例 `mir` 通常靠字級 unigram 或 bigram 即可，但要用實測資料確認。

---

## 1. 背景：連打為什麼是 lattice 問題

### 1.1 一個具體例子

速成每字打首尾兩碼；只有一個碼的字是單鍵。使用者打 `mir` 三個鍵，不打空格分隔。引擎不知道這三個鍵是幾個字：

```
位置:   0     1     2     3
        |--m--|--i--|--r--|        每個字 1 碼 (m|i|r)
        |-----mi----|              歹 ... (mi 一字)
              |-----ir----|        台 ... (ir 一字)
```

可能的切分：

| 切法 | 意思 |
|---|---|
| m｜ir | 一｜台 → 「一台」 |
| mi｜r | 歹｜口 → 「歹口」 |
| m｜i｜r | 三個單鍵字 |
| mir（若有字的倉頡全碼是 mir） | 一個字 |

`anr` 同理：`an｜r` 可為「門口」，`anr` 整段可為一個字的倉頡全碼（使用者例子是「問」）。注意這代表 lattice 的邊長度不限於 1–2 碼，只要產品允許速成與全碼混打，邊長可到 5（倉頡全碼最長 5 碼；這是倉頡的通則，未在本次查證）。

### 1.2 路徑數會爆炸，所以用動態規劃

n 個鍵、每段長度 1–2：路徑數是費氏數列（n=3 → 3 條，n=10 → 89 條，n=20 → 10,946 條；我自己算的，不是來源給的）。每段長度可到 5：最多 2^(n-1) 條（n=10 → 512，n=20 → 524,288）。每段還有多個候選字，窮舉不可行。

Viterbi 的想法：不要窮舉整句，而是在每個位置只保留「到這裡為止最好的前幾條部分路徑」，往右一格擴展。這也是 beam search（保留前 B 條，其他丟掉）。libime 預設 `beamSize = 20`、`frameSize = 40`（`src/libime/core/decoder.h` 第 33–34 行）；librime 的 poet 用 `beam_width = max_sentences * 3`（`src/rime/gear/poet.cc` 的 `MakeSentences`）。

### 1.3 一個關於「長度偏誤」的重要細節（推論）

如果一條路徑的分數是各段機率相乘（取 log 就是相加），段數越多乘的小於 1 的數越多，分數越低。所以：

- 路徑 A = 單字「問」一段。
- 路徑 B = 「門」「口」兩段。

只用「字」的 unigram 時，B 天生吃虧；要讓 B 在該贏時贏，需要詞級條目「門口」帶自己的詞頻，或有 bigram P(口|門) 補回來。這是 `anr` 案例要詞級資料的原因。librime 的做法是詞條自帶 weight，再加 grammar 分數（見第 3 節）[S7]。

---

## 2. 字頻 / 詞頻 unigram 排序

### 原理
每個候選有一個權重 w = log P(word)，由語料詞頻除以總詞數取對數。同一個碼下，權重高的排前面。連打時，路徑分數 = 各段權重相加。librime 的詞典條目就是「文字、碼、權重」，`Poet` 把它們加起來 [S7]。

### 資料
- 一份詞頻表（字 + 詞）。來源與授權是另一個題目，不在本文範圍（brief〈碼表與資料〉）。
- Rime 的 octagram 語言模型資料本身就是「從開放網路語料自動精煉」，附免責聲明 [S12]。

### 大小、延遲、記憶體
- 大小：就是碼表本身，不額外增加。數量級取決於詞庫，本次未查證任何速成碼表的實際條目數。
- 延遲：一次 hash/trie 查詢，可忽略。
- 記憶體：與碼表同量級。

### 端上 / 雲端
完全端上。

### 它解決與解決不了的
- 解決：單字同碼時，常用字排前面。
- 解決不了：句子層級的選擇（`一台` vs `歹口` 靠的是「一」與「台」都比「歹」「口」更常見還是靠相鄰關係，要看資料；第 10 節建議用實測回答）。
- 解決不了：上下文。「在」跟「再」永遠同一個順序。

---

## 3. n-gram 語言模型、平滑、壓縮與查詢

### 3.1 原理
bigram 模型估計 P(w_i | w_{i-1})：前一個詞是什麼，決定下一個詞的機率。trigram 看前兩個。階數越高越準，但條目數成長極快，大部分組合在語料裡根本沒出現過，所以需要「平滑」：

- **Backoff**（Katz）：高階查不到就退到低階，並乘上一個 backoff 權重。Gboard 的英文靜態 LM 就是 Katz 平滑、Bayesian 內插的 5-gram [S2]。
- **Kneser-Ney**：退到低階時，不用「這個詞出現幾次」，而用「這個詞接在多少種不同前文後面」。細節見 SLP3 第 3 章 [S18]（本次沒有逐字查證公式，只引用該章作為標準教材）。
- 本專案的 Rime 做法：不做平滑，而是用「有查到 → 加成；沒查到 → 固定懲罰」。`Grammar::Evaluate` 沒有 grammar 時固定扣 `-13.8155`（= ln 1e-6）（`src/rime/gear/grammar.h`）；octagram 預設 `non_collocation_penalty = -12`、`collocation_penalty = -12`、`weak_collocation_penalty = -24`、`rear_penalty = -18`（`librime-octagram/src/octagram.cc`）。這是「便宜但粗糙」的選項。

### 3.2 資料需求
- 一份大型斷詞後語料。Rime 的 octagram 資料是現成的，有字級與詞級、繁簡版本。
- 實際檔案大小（`lotem/rime-octagram-data` release `20260712` 的 asset，來自 GitHub API）[S12]：

| 檔案 | 位元組 | 約 MB |
|---|---|---|
| zh-hant-t-essay-bgc-compact.gram | 10,318,892 | 10.3 |
| zh-hant-t-essay-bgc.gram | 30,165,036 | 30.2 |
| zh-hant-t-essay-bgw-compact.gram | 41,415,724 | 41.4 |
| zh-hant-t-essay-bgw.gram | 208,672,812 | 208.7 |

「bgc / bgw」在 README 與 `grammar.yaml` 裡的設定：`hant_char` 用 bgc 並設 `collocation_max_length: 2`，`hant` 用 bgw。我推測 bgc = bigram over characters、bgw = bigram over words（推論，來源沒有直接寫出名詞定義）。每個檔案的條目數本次未查證，所以無法算每條多少 byte。

### 3.3 壓縮與查詢

**KenLM**（Heafield 2011）[S4]：
- 兩種資料結構：PROBING（hash table，快、吃記憶體）與 TRIE（排序陣列 + 位元壓縮，小）。
- 論文 Table 1（單執行緒，perplexity 任務；測試用的模型大小我沒有抓到，未查證）：

| 實作 | 查詢速度（queries/ms） | RAM（GB） |
|---|---|---|
| KenLM PROBING | 1818 | 5.28 |
| KenLM TRIE | 1139 | 2.72 |
| KenLM TRIE，8-bit 量化 | 1127 | 1.59 |
| SRILM 預設 | 750 | 9.19 |
| IRSTLM | 368–426 | 2.91 |

  換算：同一模型上，TRIE 8-bit 量化的 RAM 是 SRILM 預設的 17%（1.59 / 9.19），速度是 1.5 倍（1127 / 750）。
- TRIE 的總位元數公式（論文原式）：unigram 每條 (32+32+64+64) bit；中間階每條 `⌈log2 c1⌉ + q + r + ⌈log2 c_{n+1}⌉` bit；最高階每條 `⌈log2 c1⌉ + q` bit。c_n 是第 n 階條目數，q、r 是機率與 backoff 的量化位元數；「詞表過濾能有效縮小模型」因為 `⌈log2 c1⌉` 在每條裡都出現。
- 估計（用上面公式）：詞表 5 萬詞（16 bit）、1 百萬條 bigram、q=8：unigram 5 萬 × 24 byte ≈ 1.2 MB；bigram 1 百萬 × 3 byte = 3 MB；合計約 4.2 MB（加量化表）。這是「純 bigram、最高階無 backoff」的下限估算；真實資料條目數未知。
- libime 直接用 KenLM：`QuantArrayTrieModel`（`src/libime/core/languagemodel.cpp`）。

**Pauls & Klein 2011** [S5]：「compressed implicit trie」把 Google n-gram corpus 的 40 億條 n-gram 連計數存成每條 23 bit；對照 SRILM 最緊湊實作每條 33 byte。這是離線大模型的壓縮上限，QIM 規模遠小於此。

**Bloom filter LM**（Talbot & Osborne 2007）[S13]：用 Bloom filter 存 n-gram，空間低於無損下限，代價是有可量化的偽陽性機率；論文圖表的基準是 SRILM Witten-Bell 5-gram 924 MB。適合「模型大到放不下」的雲端/SMT 場景。對手機輸入法，詞庫規模下用 trie 就夠，Bloom filter 的偽陽性會讓偶爾出現錯誤的高分，我認為不值得（推論）。

**OpenGrm NGram**：把 n-gram 模型編碼成 weighted FST，由 OpenFst 操作，提供計數、平滑、剪枝、評估 perplexity [S11]。Gboard 的 FST 解碼器就是這個路線（第 4 節）。本次沒查證它各平滑法的細節。

### 3.4 大小數據彙整
- Gboard 英文：5-gram 共 125 萬條，其中 16.4 萬 unigram [S2]。文獻說鍵盤 LM 不應超過 5–10 MB，大約能表示「幾十萬詞」[S1]；注意這是上限，不是 Gboard LM 實際檔案大小。
- Rime octagram：見 3.2 表。

### 3.5 延遲與記憶體（估計）
KenLM TRIE 在 2011 年 server 級 CPU 上 1139 queries/ms [S4]。假設 beam 20、每位置 2 種長度、每段 50 個候選：n=10 時整句重算約 10×2×50×20 = 20,000 次查詢，約 18 ms（20,000 / 1139）；增量只算新位置約 2,000 次，約 2 ms。手機 CPU 比較慢，常數可能差數倍，**這個估算沒有實測**，要在目標硬體上量。

### 3.6 端上 / 雲端
端上。

---

## 4. 解碼：lattice、Viterbi、beam、k-best

### 4.1 結構
- **節點**：按鍵位置 0..n。
- **邊**：從位置 i 到 j，代表「第 i 到 j 個鍵構成一個字/詞」，每條邊有一組候選字（同碼字）。libime 稱為 `SegmentGraph` + `Lattice`；librime 稱為 `WordGraph`。
- **路徑分數**：Σ（邊的詞頻權重 + 語言模型轉移分數）。
- **Viterbi**：每個節點保留前 B 條最佳部分路徑，往右擴展；到終點後回溯得到最佳句，k-best 就是終點節點的前 k 條。

### 4.2 成熟實作
- **libime**（fcitx5 的核心）`src/libime/core/decoder.cpp`：每個 frame 用 heap 維護 `frameSize` 個最佳節點；`beamSizeDefault = 20`、`frameSizeDefault = 40`；`backwardSearch` 產生 nbest。
- **librime poet**（`src/rime/gear/poet.cc`）：`MakeSentences` 的註解寫「beam search and dp on word graph」；每個終點保留 `beam_width = max_sentences * 3` 條；以文字 hash 去重；結果還有 `cutoff_threshold`，分數落差超過比例就不再回傳。每個詞的分數 = `entry->weight + Grammar::Query(...)`（grammar 即 octagram）。注意它是對整張 graph 一次跑完，看程式碼沒有跨按鍵重用上次的狀態（未查證是否有上層快取）。
- **Mozc**（`src/converter/immutable_converter.cc`）：用 `Connector` 取得左右詞類 id 間的 transition cost，加上詞本身的 `wcost`；`NBestGenerator` 產生多候選。用的是「詞類 id 轉移矩陣」，不是詞對詞 n-gram。我只讀到呼叫點與註解，矩陣大小未查證。
- **Google FST 解碼**（Ouyang et al. 2017）[S1]：把「按鍵空間模型 ∘ lexicon ∘ LM」都做成 FST 再合成。針對鍵盤的錯誤：替換由空間模型給每個按鍵一個機率；漏打允許 lexicon FST 每個 frame 跳過一條非 epsilon 邊並帶權重；多打（insertion）把該次 tap 當額外輸入而不前進狀態。「literal word」子圖用來保證使用者實際打的字串一定是候選。
- **OpenGrm** [S11]：產生可放進這類 FST 的 n-gram 模型。

### 4.3 形碼連打怎麼表示成 lattice
把「拼音音節邊」換成「碼表邊」即可：
1. 對每個起點 i 和長度 L ∈ {1..Lmax}，查碼表：key[i..i+L) 對應哪些字/詞？有就加一條邊，邊上掛候選與詞頻。
2. 容錯邊（第 5 節）：對同一位置用「相鄰鍵替換」查，額外加邊，邊帶懲罰分。
3. 詞條邊：若碼表或詞庫有「多字詞對應的連續碼」，額外加一條跨多字的邊（`anr` 的「門口」若以詞條形式存在，會是 0→3 的一條邊；若沒有詞條，就靠兩條單字邊 + bigram）。

libime 的拼音就是這樣建的：`PinyinEncoder::parseUserPinyin` 產生 `SegmentGraph`。

### 4.4 每按一鍵增量更新
libime `PinyinContext::update()`（`src/libime/pinyin/pinyincontext.cpp` 約 672–704 行）：
1. 只對尚未選定的尾段重新 parse，得到 `newGraph`。
2. `segs_.merge(newGraph, ...)`：合併進舊圖；被改動的節點通過 callback 呼叫 `lattice_.discardNode(nodes)` 和 `matchState_.discardNode(nodes)` 丟掉舊結果。
3. `decoder()->decode(lattice_, segs_, ...)`：只對新增的節點補算。

也就是：舊位置的 Viterbi 狀態保留，只算新字母帶來的新節點。這比整句重算省很多。若是刪除鍵（backspace），只要丟掉被刪位置之後的節點即可（推論；libime 實際處理在其他分支，未逐行確認）。

### 4.5 端上 / 雲端
端上。

---

## 5. 容錯

### 5.1 兩種錯誤來源
1. **實體鍵盤**：按到鄰近鍵、漏打、多打、換位。
2. **觸控鍵盤**：手指位置沒有精確落在鍵中心。

### 5.2 鍵盤鄰近鍵錯誤模型（離散）
libime 的 `PinyinCorrectionProfile`（`src/libime/pinyin/pinyincorrectionprofile.cpp`）：`mappingFromRows` 對 QWERTY 的三排 (`qwertyuiop`, `asdfghjkl`, `zxcvbnm`)，每個鍵只能被糾正成**同一排左右各一個鍵**。註解寫「the key can only be corrected to the key adjacent to it」。這代表：
- 只會處理水平方向的鄰近錯誤，上下排不處理。
- 每鍵最多 2 個替代，lattice 增加的邊數有限且可控。

### 5.3 觸控的空間模型（連續）
Ouyang 2017 [S1]：tap 的空間模型「typically a Gaussian distribution centered on each key center」，對每個 tap 給所有按鍵一個機率分布，再與 LM 一起做搜尋；另外用 bi-key（上一鍵作 left context）的 context dependency，類比語音辨識的 triphone。Goodman et al. 2002 是更早的先驅（Microsoft Research，"Language modeling for soft keyboards"）[S17]；本次只確認該論文存在及主題，數字未查證。

### 5.4 編輯距離
Ouyang 2017 的替換/漏打/多打三種運算就是編輯距離的三種基本操作，只是權重由模型學出來，不是固定 1 [S1]。Mozc 對 kana 輸入有 `KeyCorrector`，被糾正的節點加 `GetCorrectedCostPenalty`（`immutable_converter.cc` 第 113 行附近）；細節未查證。

### 5.5 在形碼（速成）上怎麼套（推論）
- 速成一個字只有 2 碼，資訊量很低：一個替換錯誤就換到另一整組候選。形碼沒有拼音那種「音節邊界冗餘」，所以相同的容錯寬度造成的誤判比拼音更多。
- 估計 lattice 增量：每個位置若容許相鄰鍵替換（同排左右各 1，每鍵最多 2 個替代），一個 2 碼的邊會變成最多 1 + 2 + 2 + 4 = 9 種碼組合（原碼、換第一碼、換第二碼、兩碼都換）；漏打再加 1–2 種。這是我算的組合數，不含是否有字對應。
- 取捨選項：
  - A. 只在「無候選」時才啟動容錯（精確匹配優先，避免汙染正常輸入）。
  - B. 永遠啟動，但容錯邊帶固定懲罰（libime / Gboard 做法）。
  - C. 只對觸控鍵盤用 Gaussian，實體鍵盤用離散相鄰表。
- 「打錯一兩碼首選仍對」要靠：懲罰夠小讓正確字能勝出，又不能小到讓正確輸入被錯字蓋過。這個閾值只能用實測資料調，文獻沒有速成的數字（未查證）。

### 5.6 延遲
Gaussian 計算是每個 tap × 鍵數（約 26–30 個鍵）次浮點運算，可忽略；主要成本是 lattice 邊數增加導致 LM 查詢數成倍增加（見 3.5 的估算，乘上邊的倍數）。

### 5.7 端上 / 雲端
端上。

---

## 6. 使用者學習

### 6.1 目標
1. 常打的詞排前面（頻率加成）。
2. 很久沒用的詞逐漸退回去（時間衰減）。
3. 學到「前一個詞是 X 時我常接 Y」（使用者 bigram）。
4. 一次誤選不會永久汙染排序。

### 6.2 librime user_dict（`src/rime/algo/dynamics.h`、`src/rime/dict/user_dictionary.cc`）
每個條目存 `commits`（提交次數）、`dee`（指數衰減後的累計值）、`tick`（上次時間）。tick 是全域計數，每次提交加 1，不是牆上時間。

- 更新衰減值：`formula_d(d, t, da, ta) = d + da * exp((ta - t) / 200)`。舊值 `da` 隨 tick 差指數衰減（時間常數 200 tick），再加上這次的貢獻 `d`（提交一次 d=1；`commits == 0` 時 d=0.1；刪除時 d=0）。
- 轉權重：`formula_p(s, u, t, d)`，在 `s = 0` 時 `m = u * (1 - exp(-t/10000))^10`，`u = commits / present_tick`；`d < 20` 時 `p = m + (0.5 - m) * (d / kM)`，其中 `kM = 1/(1 - exp(-0.005)) ≈ 200.5`；`d ≥ 20` 時另一支式子。最後 `weight = ln(p) + credibility`。
- 丟棄：`dee ≤ 1e-200` 的條目被捨棄（註解寫對應 `log2(1/T)*200*ln(2)` tick）。
- 估計（用上面公式）：只提交一次的條目，`d = 1` 時 `d/kM ≈ 0.005`，`p ≈ m + 0.5×0.005 ≈ m + 0.0025`，非常小，加上每 139 個 tick（200×ln2）減半。意思是「單次誤選」帶來的加成既小又很快消失，這是它能避免汙染的原因。要「記住」需要多次提交。

### 6.3 libime（`src/libime/core/historybigram.cpp`、`userlanguagemodel.cpp`、`constants.h`）
- 歷史存成句子，分三個池：大小 128、8192、65536 句；越新的句子在越小的池子，越舊被擠到大池；權重隨池子遞減（`p = 1/(1+alpha)`，`alpha = 1.0`，每池權重是 `(1-p) * p^(i-1)` 的比例再除池大小）。
- 使用者分數：`pr = 0.2 * unigramFreq(cur)/unigramSize + 0.8 * bigramFreq(prev,cur)/unigramFreq(prev)`（沒有 bigram 時 bigram 權重為 0）；零機率則回傳固定的 unknown penalty `log10(1/60000000)`。
- 與靜態 LM 混合：`score = max(staticScore, logsumexp(staticScore + log10(1-w), userScore + log10(w)))`，`w` 的預設常數是 `DEFAULT_USER_LANGUAGE_MODEL_USER_WEIGHT = 0.2`。`max(...)` 的意思是：使用者歷史只會讓分數變高，不會把靜態 LM 本來就高的詞壓低。
- 汙染控制：新句先進最小池（128 句），自然被後來的擠走；`forget(word, code)` 可刪。沒有看到「單次誤選」的特殊處理，靠池子結構稀釋（推論）。

### 6.4 Mozc `UserHistoryPredictor`（`src/prediction/user_history_predictor.{h,cc}`）
- `GetScore` 註解：分數基本上由 `last_access_time` 決定（LRU 策略），再依字串長度、`bigram_boost` 旗標微調。
- Bigram：每個條目最多記 `kMaxNextEntriesSize = 6` 個「下一個條目」。
- Revert：`Revert(revert_id)` 可撤銷上一次 Finish；`kRevertCacheSize = 16`。使用者在轉換後立刻改選別的候選，之前的學習會被回滾，這是對「誤選」最直接的處理。

### 6.5 可選策略（不拍板）
| 策略 | 做法 | 優點 | 風險 |
|---|---|---|---|
| 計數 + 指數衰減（librime） | 每次提交 d += 1，隨 tick 衰減 | 單次誤選影響小；公式簡單 | 要調參數；tick 不是真實時間 |
| 池子 LRU（libime） | 分級池，新的小池、舊的大池 | 沒有時間參數；自然遺忘 | 固定容量，高頻舊詞要靠大池 |
| 純 LRU + Revert（Mozc） | 最近用過的優先 | 直覺、可預測 | 誤選會立刻排第一，所以需要 Revert |
| 門檻 | 提交 ≥ N 次才生效 | 擋住誤選 | 新詞學得慢 |

### 6.6 資料 / 大小 / 延遲 / 端上
- 資料：使用者自己的提交紀錄，不需外部資料。
- 大小：libime 池子最多 65,536 句 + 8,192 + 128 句；實際位元組未查證。
- 延遲：一次 trie 查詢，可忽略。
- 端上：必須端上（隱私）。跨裝置同步是另一個題目。

---

## 7. 神經網路語言模型

### 7.1 Gboard CIFG-LSTM [S3]
- 結構：單層 CIFG（輸入閘與遺忘閘耦合，每個 cell 參數少 25%），670 個單元，embedding 與輸出投影維度 96，輸入輸出共用 embedding。
- 規模：140 萬參數，其中超過三分之二在 embedding 矩陣；詞表 1 萬；權重量化後「shipped to Gboard devices is 1.4 megabytes」。
- 比較（下一詞預測 top-1 recall，在使用者端資料上）：n-gram 12.5 ± 0.2、伺服器訓練 CIFG 15.0 ± 0.5、聯邦學習 CIFG 15.8 ± 0.3 [S3]。
- 約束：延遲要低、鍵盤回饋 20 ms 內 [S3]；語言模型通常限制在「數十 MB、詞表數十萬詞」[S3]。
- 限制：這篇的神經模型做的是「下一詞預測」，不是 lattice rescoring，也不是漢字轉換。

### 7.2 Apple iOS 17 [S9]
Apple 新聞稿只說：「Autocorrect receives a comprehensive update with a transformer language model」，描述為「a state-of-the-art on-device machine learning language model for word prediction」，並說句子層級的修正能修更多文法錯誤。模型大小、延遲、架構細節：**未查證**（沒有找到官方技術文件）。

### 7.3 量化與推論引擎延遲量級
- 量化：Gboard 把 140 萬參數壓到 1.4 MB（約 1 byte/參數，推論為 8-bit）[S3]。
- Transformer 參考點：MobileBERT 比 BERT-base 小 4.3 倍、快 5.5 倍，Pixel 4 上 62 ms [S15]；這是整句分類/問答用的編碼器，序列長度設定未抓取，不能直接當作 LM 的 per-key 延遲。
- ONNX Runtime / Core ML / TFLite 對「幾 MB 的小 LM」的實測延遲：**未查證**。要做就必須在目標裝置上量，不能引用別人的數字。

### 7.4 用神經 LM 做 lattice rescoring
做法：先用 n-gram + Viterbi 產生 k-best 或剪枝後的 lattice，再用神經 LM 給每條候選句重新打分、與 n-gram 分數線性組合後重排。理由：神經 LM 單次推論貴，所以只對少量候選（幾十條）跑，而不是在搜尋過程中每個節點都呼叫。

Google 的 FST 解碼器用的是同一個思路的一部分：動態模型（使用者歷史、聯絡人）不烘進主 FST，而是用「on-the-fly lattice rescoring」套在解碼結果上 [S1]。學術界的 lattice rescoring 實驗（如 Interspeech 2021 的 LT-LM）顯示，單次對整個 lattice 評分比逐條 n-best 或剪枝 RNNLM lattice rescoring 快兩個數量級以上，但那是語音辨識，模型 54M–155M 參數，不是輸入法量級 [S19]。

### 7.5 中文輸入法的學術證據
PinyinGPT（arXiv:2203.00249）[S20]：把 GPT 用於拼音輸入法，縮寫拼音（只打聲母，跟速成同樣是資訊量低的輸入）下 P@1 約 22–34%，加入拼音條件後的 PinyinGPT-Concat 比純 GPT 的 P@1 高約 0.2–1.9 個百分點、P@5 高約 4–6.4 個百分點（取自擷取到的 9 個領域列，未涵蓋全部領域）。這說明：資訊量低的輸入，top-1 準確率天花板就不高，top-5/10 的候選清單設計更重要。

### 7.6 適用性
- 適合端上：是，模型小（MB 級）；
- 對速成的價值：能考慮較長上下文，對 `anr` 這類詞級歧義有幫助；但需要中文語料訓練與驗證，QIM 目前沒有這些（brief 尚未決定語料來源）。

---

## 8. LLM

### 8.1 做法
把輸入框已有的文字當 prompt，把 lattice 的 k-best 候選（或連打的幾個切法）作為選項，讓 LLM 算每個選項的條件機率（或讓它生成），用來重排或補全。兩種接法：
1. **打分**：對每個候選句算 log P(候選 | 上文)。每個候選要跑一次前向（可共用 KV cache 的上文部分）。
2. **生成**：讓模型直接輸出候選，再用 lattice 約束（只允許符合碼的字）。PinyinGPT 的做法是限制只輸出同拼音的字 [S20]。

### 8.2 端上實測數字
| 項目 | 數字 | 來源 |
|---|---|---|
| Apple 端上基礎模型 | 約 30 億參數；平均 3.7 bits-per-weight（可到 3.5）；iPhone 15 Pro 每個 prompt token 約 0.6 ms；生成 30 token/s（未用 speculative decoding）；詞表 49K | [S10] |
| Apple 2025 技術報告 | 約 3B 參數端上模型，KV-cache sharing 與 2-bit quantization-aware training；報告頁沒有延遲數字 | [S10b] |
| llama.cpp，iPhone 15 Pro（A17 Pro） | TinyLlama 1.1B Q4_0：prompt 646 token/s、生成 57 token/s；Phi-2（約 3B 級欄位）Q4_0：157 / 25；Mistral 7B Q4_0：81 / 9 | [S16] |
| llama.cpp，iPhone 14 Pro（A16） | TinyLlama 1.1B Q4_0：506 / 54；Phi-2 Q4_0：122 / 23 | [S16] |
| llama.cpp，iPhone 12（A14） | TinyLlama 1.1B Q4_0：227 / 38 | [S16] |
| llama.cpp，M4 Mac mini | llama 3B Q4_K_M：prompt 530 token/s、生成 46 token/s | [S16b] |
| Gemini Nano | 透過 Android AICore 系統服務提供，所有 app 共用同一個模型；官方文件把它定位為「低成本、隱私」的場景 | [S21] |

Gemini Nano 的每 token 延遲與記憶體：**未查證**（只找到二手部落格的數字，不採用）。0.5B 級模型（如 Qwen2.5-0.5B）在手機上的數字：**未查證**。

### 8.3 估計（我用上表算的，未實測）
- 設上下文 30 token + 候選 10 個各 4 token，共 70 個 token 要處理一次前向：
  - TinyLlama 1.1B，iPhone 15 Pro：70 / 646 ≈ 108 ms；
  - Phi-2 級 3B，iPhone 15 Pro：70 / 157 ≈ 446 ms；
  - Phi-2 級 3B，iPhone 12 級（A14，51 token/s 的 mini 數字）：70 / 51 ≈ 1.4 s（A14 的 Phi-2 prompt 速度取自 12 mini 51.39 token/s）。
  這些都超過「每鍵 50–100 ms」，也沒算載入時間與耗電。
- 記憶體（估計）：模型 Q4_0 約 4.5 bit/參數。1.1B ≈ 0.6 GB；3B ≈ 1.7 GB；0.5B ≈ 0.28 GB。公式：參數數 × 4.5 / 8。
- iOS 鍵盤擴充（extension）的記憶體上限：**未查證**，這是會直接卡死 LLM 方案的限制，下一步必須查（見〈未查證清單〉）。

### 8.4 雲端
- 延遲：一次網路往返加推論；有部落格寫 150–300 ms 往返，但是二手來源，不採用。雲端 LLM 一定超過每鍵預算，只能用於「使用者停頓後」的非同步升級或「補全」。
- 隱私：每次要把輸入框內容送出；這與輸入法常見的「不上傳打字內容」期待衝突（brief 尚未決定隱私立場）。
- 費用：Gemini 等 API 是按 token 計費（價格頁已確認存在，未逐項引用數字）。

### 8.5 每鍵延遲預算的依據
| 依據 | 內容 |
|---|---|
| Gboard | 「a key press is expected to produce visible feedback within about 20 msec」[S1]；同樣 20 ms 的說法在 [S3]。這是**按鍵回饋**，不是候選更新。 |
| Nielsen | 「0.1 second is about the limit for having the user feel that the system is reacting instantaneously」，引自 Miller (1968) 與 Card et al. (1991) [S14] |
| Deber et al. CHI 2015 | 前人研究指出最小可感知延遲介於 2 ms 到 100 ms，差異源自直接/間接觸控與敲擊/拖曳；少掉一個 60 Hz frame（16.7 ms）在許多情況下就能被感知 [S22] |

結論：沒有找到專為輸入法「按鍵到候選更新」寫的延遲研究（**未查證**）。工程上合理的拆法是：按鍵回饋 ≤ 20 ms（Gboard 值）；候選列更新 ≤ 100 ms 為上限（Nielsen 值），目標 ≤ 50 ms 留餘裕。這是推論，用來當設計預算，不是文獻結論。

---

## 9. 英文候選混排

在中文（速成）狀態下，判斷要不要給英文候選：

### 9.1 查到的現況
- Google Input Tools 的作法：按 Enter 直接上屏原始字母，即「使用者明確要求」[S23]。
- Rime 社群有以 librime-lua 實作的「中文模式下輸入英文單詞」方案，例如 shewer/rime-english，所有功能由 lua 插件實現 [S24]；本次沒有讀它的程式碼，不清楚判斷規則。
- Gboard 的多語言輸入與雙語建議：只找到使用者討論串，沒有第一手技術文件（**未查證**）。

### 9.2 可選做法（我歸納，不是查到的某產品的規格）
| 做法 | 判斷依據 | 優點 | 風險 |
|---|---|---|---|
| 明確切換 / 快捷鍵 | 使用者指令 | 零誤判 | 多一個動作 |
| 碼表無結果才給英文 | 目前緩衝區在碼表（含容錯）查不到任何字 | 實作簡單，不干擾正常輸入 | 速成的 2 碼空間幾乎被用滿，很少「查不到」，這條件很少觸發（推論） |
| 英文詞典匹配 | 緩衝區是英文詞（前綴）且長度 ≥ 3 | 打 `hello` 能直接出 | 速成碼 `hel` 之類也可能是合法碼組合，會互相搶位置 |
| 以 LM 比較 | 比較中文路徑分數與英文詞分數 | 有原則 | 兩個機率尺度要校準；需要英文 unigram 與混合語料 |
| 上下文 | 前面剛輸入英文 → 提高英文權重 | 符合連續書寫習慣 | 需要讀取輸入框上下文，各平台權限不同（見 brief 平台章節） |

英文候選的數量上限與位置（固定第幾位）也是需要決定的產品問題。

---

## 10. 總結表

| 技術 | 改善什麼 | 資料 | 大小 | 延遲（每鍵） | 端上 / 雲端 |
|---|---|---|---|---|---|
| 字/詞 unigram | 同碼字的基礎排序 | 詞頻表 | 與碼表同量級 | 可忽略（hash 查詢） | 端上 |
| bigram（KenLM trie 或 Rime octagram） | 句子層級選擇、詞級歧義 | 斷詞語料 | 繁體 octagram：字級壓縮版 10.3 MB、詞級壓縮版 41.4 MB [S12]；自建估計見 3.3 | 估計 ≤ 數 ms（3.5 的估算，未實測） | 端上 |
| trigram / 5-gram + Katz / KN | 更長上下文 | 更大語料 | Gboard 英文 5-gram：125 萬條 [S2]；文獻限制 5–10 MB [S1] | 同上 | 端上 |
| lattice + Viterbi + beam | 讓連打切分可計算 | 無額外資料 | 記憶體 O(beam × 位置) | 增量更新；libime beam=20 [S6] | 端上 |
| 鄰近鍵 / Gaussian 容錯 | 打錯一兩碼仍可選對 | 鍵盤佈局；觸控還要座標 | 可忽略 | lattice 邊數增加，成倍增加 LM 查詢 | 端上 |
| 使用者學習 | 個人用詞排前面 | 使用者提交紀錄 | libime 池最多 65,536+8,192+128 句 [S6] | 可忽略 | 端上（隱私） |
| 小型神經 LM（CIFG-LSTM 級） | 更好的上下文判斷 | 中文訓練語料 | Gboard：140 萬參數、1.4 MB [S3] | 未查證 | 端上 |
| 神經 LM 重排 k-best | 修正 n-gram 的錯選 | 同上 | MB 級到數十 MB | 未查證；MobileBERT 62 ms 為參考 [S15] | 端上 |
| 端上 LLM（0.5–3B） | 長上下文、補全 | 預訓練模型 | 估計 0.3–1.7 GB（Q4） | 1.1B：約 100 ms 級；3B：400 ms 以上（8.3 估計） | 端上，但只適合非同步 |
| 雲端 LLM | 最強語意 | 預訓練模型 | 無本地成本 | 超過每鍵預算 | 雲端；隱私成本 |
| 英文候選混排 | 中英混打 | 英文詞典與詞頻 | 小 | 可忽略 | 端上 |

---

## 11. 給 QIM 的分層選項（不拍板）

### 11.1 三個案例各自需要哪一層（推論，須用實測驗證）

| 案例 | 候選路徑 | 只有 unigram 夠不夠 | 還需要什麼 |
|---|---|---|---|
| `mir` | 一｜台 vs 歹｜口 | 字級 unigram 通常有機會（「一」遠比「歹」常見），但「台」與「口」要比的是同碼候選的頻率，取決於資料 | 若不夠：字級 bigram |
| `anr` | 門｜口 vs 問 | 不夠。單字「問」一段 vs 兩段相乘，兩段天生吃虧（1.3 節） | 詞級條目「門口」帶詞頻，或 bigram P(口｜門) |
| 同碼字（無上下文） | 同碼候選 | 夠 | — |

### 11.2 分層

| 層 | 內容 | 解決 | 成本 | 取捨 |
|---|---|---|---|---|
| L0 | 碼表 + 字頻 unigram + lattice/Viterbi 切分 | 切分可計算；`mir` 有機會 | 最低；libime/librime 都是這個骨架 | `anr` 這類詞級歧義靠詞條 |
| L1 | 詞級條目與詞頻（`門口` 當詞） | `anr` | 需要詞表與詞頻來源（授權待查） | 詞表品質決定上限 |
| L2 | 靜態 bigram（octagram 風格固定懲罰 或 KenLM 平滑） | 句子層級；`mir` 的穩定性 | 檔案 10–41 MB 級 [S12]；或自建 | octagram 做法粗（固定懲罰）；KenLM 精準但要自己訓練語料 |
| L3 | 使用者學習（衰減計數 + 使用者 bigram） | 個人化 | 小；要設計防汙染 | 參數要調；跨裝置同步另算 |
| L4 | 容錯（相鄰鍵、觸控 Gaussian） | 打錯 | lattice 變大 | 速成 2 碼資訊少，誤判風險高於拼音 |
| L5 | 神經 LM 重排 k-best | 上下文 | 要語料與訓練；延遲未知 | 先量 L0–L3 的剩餘錯誤再決定 |
| L6 | 端上 LLM 非同步重排 / 補全 | 長上下文 | 記憶體與電力大 | iOS 擴充記憶體上限未查證，可能直接排除 |
| L7 | 雲端 LLM | 最強 | 隱私與網路 | 與離線承諾衝突 |

### 11.3 建議先驗證的事（對應「讓三個案例首選正確」）
1. 建立測試集：以 `mir`、`anr` 為代表，收集真實速成連打句子與標準答案，量「首選正確率」（brief 第 7 節也要求怎麼量）。
2. 先做 L0+L1，量錯誤剩多少；再單獨加 L2，量增量。每一層都用同一個測試集比，才知道值不值。
3. 效能預算設為：每鍵候選更新 ≤ 50 ms 目標、≤ 100 ms 上限（推論值，見 8.5）；在最低階目標裝置上實測。

---

## 12. 未查證清單（下一步要補才能進 brief）

1. 速成碼表的條目數、每個二碼下的平均候選數（決定 lattice 邊上的候選數 K，3.5 的估算用了假設值 50）。
2. 繁體語料與詞頻的授權（brief〈碼表與資料〉題目）。
3. iOS 鍵盤擴充的記憶體上限、Android 輸入法服務的記憶體限制（決定 L5/L6 能不能做）。
4. Apple iOS 17 鍵盤 transformer 的公開細節（WWDC 2023 內容沒有查）。
5. ONNX Runtime / Core ML / TFLite 對 MB 級小 LM 的實測延遲。
6. Gemini Nano 與 0.5B 模型的手機端 token 延遲與記憶體。
7. IME 按鍵到候選延遲的專門文獻。
8. Mozc 連接矩陣大小、librime poet 是否有跨按鍵快取。
9. KenLM 論文 Table 1 所用測試模型的條目數。
10. Rime octagram 檔案的條目數與 bgc / bgw 名詞定義。
11. 中英混排的成熟產品規則（Sogou、Gboard、微信輸入法等）。

---

## 來源

### 論文
- [S1] Ouyang, Rybach, Beaufays, Riley. *Mobile Keyboard Input Decoding with Finite-State Transducers*. arXiv:1704.03987. https://arxiv.org/abs/1704.03987 （20 ms 回饋、LM 5–10 MB、Gaussian 空間模型、bi-key、替換/漏打/多打、on-the-fly lattice rescoring）
- [S2][S3] Hard et al. *Federated Learning for Mobile Keyboard Prediction*. arXiv:1811.03604. https://arxiv.org/abs/1811.03604 （[S2] 與 [S3] 皆指此文：Katz 5-gram 125 萬條 / 16.4 萬 unigram；CIFG 670 單元、140 萬參數、詞表 1 萬、1.4 MB；top-1 recall 12.5 / 15.0 / 15.8）
- [S4] Heafield. *KenLM: Faster and Smaller Language Model Queries*. WMT 2011. https://kheafield.com/papers/avenue/kenlm.pdf
- [S5] Pauls, Klein. *Faster and Smaller N-Gram Language Models*. ACL 2011. https://aclanthology.org/P11-1027.pdf
- [S13] Talbot, Osborne. *Smoothed Bloom Filter Language Models: Tera-Scale LMs on the Cheap*. EMNLP-CoNLL 2007. https://aclanthology.org/D07-1049.pdf ；*Randomised Language Modelling for SMT*. ACL 2007. https://aclanthology.org/P07-1065.pdf
- [S15] Sun et al. *MobileBERT*. arXiv:2004.02984. https://arxiv.org/abs/2004.02984
- [S17] Goodman, Venolia, Steury, Parker. *Language Modeling for Soft Keyboards*. 2002. https://www.microsoft.com/en-us/research/publication/language-modeling-for-soft-keyboards-3 （只確認存在與主題）
- [S19] Mitrofanov et al. *LT-LM*. Interspeech 2021. https://www.isca-archive.org/interspeech_2021/mitrofanov21_interspeech.pdf
- [S20] Tan et al. *Exploring and Adapting Chinese GPT to Pinyin Input Method*. arXiv:2203.00249. https://arxiv.org/abs/2203.00249
- [S22] Deber, Jota, Forlines, Wigdor. *How Much Faster is Fast Enough?* CHI 2015. https://www.tactuallabs.com/papers/howMuchFasterIsFastEnoughCHI15.pdf
- [S18] Jurafsky, Martin. *Speech and Language Processing* 第 3 章 N-gram Language Models. https://web.stanford.edu/~jurafsky/slp3/3.pdf

### 原始碼（檢視日 2026-10-30，各 repo master/main）
- [S6] fcitx/libime：`src/libime/core/decoder.h`、`decoder.cpp`、`userlanguagemodel.cpp`、`historybigram.cpp`、`constants.h`、`languagemodel.cpp`；`src/libime/pinyin/pinyincontext.cpp`、`pinyincorrectionprofile.cpp`、`pinyinime.cpp`。https://github.com/fcitx/libime
- [S7] rime/librime：`src/rime/gear/poet.cc`、`grammar.h`、`src/rime/algo/dynamics.h`、`src/rime/dict/user_dictionary.cc`；lotem/librime-octagram：`src/octagram.cc`。https://github.com/rime/librime ；https://github.com/lotem/librime-octagram
- [S8] google/mozc：`src/converter/immutable_converter.cc`、`src/prediction/user_history_predictor.{h,cc}`。https://github.com/google/mozc
- [S12] lotem/rime-octagram-data：README、`grammar.yaml`、release `20260712` 的 asset 大小（GitHub API）。https://github.com/lotem/rime-octagram-data

### 官方文件與其他
- [S9] Apple Newsroom, *iOS 17 makes iPhone more personal and intuitive*, 2023-06-05. https://www.apple.com/newsroom/2023/06/ios-17-makes-iphone-more-personal-and-intuitive/
- [S10] Apple ML Research, *Introducing Apple's On-Device and Server Foundation Models*（2024）. https://machinelearning.apple.com/research/introducing-apple-foundation-models
- [S10b] Apple ML Research, *Apple Intelligence Foundation Language Models Tech Report 2025*. https://machinelearning.apple.com/research/apple-foundation-models-tech-report-2025
- [S11] OpenGrm NGram Library. https://www.opengrm.org/twiki/bin/view/GRM/NGramLibrary
- [S14] Nielsen Norman Group, *Response Times: The 3 Important Limits*. https://www.nngroup.com/articles/response-times-3-important-limits/
- [S16] llama.cpp discussion #4508（A 系列 / iPhone 效能）. https://github.com/ggml-org/llama.cpp/discussions/4508
- [S16b] llama.cpp discussion #4167（M 系列效能）. https://github.com/ggml-org/llama.cpp/discussions/4167
- [S21] Android Developers, *Gemini Nano*. https://developer.android.com/ai/gemini-nano
- [S23] Google Input Tools, *Input method (IME)*. https://www.google.com/inputtools/services/features/input-method.html
- [S24] shewer/rime-english. https://github.com/shewer/rime-english
