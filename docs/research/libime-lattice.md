# libime 的 lattice 建構與逐鍵增量更新

檢視日 2026-11-04；fcitx/libime master `53df6a248793f9a00c1769e658630de313f3e419`；授權 LGPL-2.1-or-later，依 ADR 0006 只取做法與公式，不抄程式碼。路徑都在 `src/libime/` 下。

## 摘要

- libime 的 lattice 是「每個 segment graph 節點（輸入位置）一串候選節點」，Viterbi 前向 + A* 回溯取 n-best。
- **增量更新只有拼音引擎在用**（`SegmentGraph::merge` + `Lattice::discardNode` + `decode` 的 `ignore` 集合）。**碼表引擎每鍵整個重算**，沒有增量。
- 碼表引擎的 graph 基本上只有一條 `0 → 全碼長度` 的邊，沒有「碼表 + 詞庫混合分段」的格子；所以它對 QIM 的參考價值在 lattice 資料結構與前置選字，不在 segmentation。

## 1. Lattice 資料結構

- `LatticeNode`（`core/lattice.h`）：繼承 `WordNode`（`word_`、`idx_` 詞表索引）；欄位 `path_`（`SegmentGraphPath`，至少 2 個 graph 節點，`from()` = front、`to()` = back）、`cost_`（詞本身的調整分，字典 callback 傳入）、`score_`（累積分）、`prev_`（Viterbi 回指指標）、`state_`（語言模型 State）。
- 儲存（`core/lattice_p.h`）：`LatticeMap = unordered_map<const SegmentGraphNode*, ptr_vector<LatticeNode>>`，key 是**結束位置的 graph 節點**；一個特殊 key `nullptr` 放 EOS 節點，`graph.start()` 底下放唯一的 BOS 根節點（`Decoder::buildLattice`）。
- 句子結果：`SentenceResult` = `vector<const LatticeNode*>` + `score`；`LatticeNode::toSentenceResult` 沿 `prev_` 走回去（`core/lattice.h`）。
- 輸入位置：`SegmentGraph` 每個 byte 位置一個節點（`graph_[idx]`），邊 = 一個 segment（拼音音節或整段碼）；一個位置可有多個節點（拼音歧義切法）（`core/segmentgraph.h`）。

## 2. Viterbi、beam、frame、n-best

- 常數：`Decoder::beamSizeDefault = 20`、`frameSizeDefault = 40`（`core/decoder.h`）。碼表引擎自己寫死 `beamSize = 20`、`frameSize = 10`，`nbest` 預設 1，碼長等於最大碼長且設了 autoRuleSet 時為 5（`table/tablecontext.cpp` `TableContext::update`）。
- `decode` 三步（`core/decoder.cpp`）：`buildLattice` → `forwardSearch` → `backwardSearch`。
- **frame size** 是建 lattice 時「同一對 (from, to) 的候選詞最多留幾個」：`buildLattice` 的 `dictMatchCallback` 以 `(path.front(), path.back())` 分組，超過 `frameSize` 就用 heap 淘汰 `singleWordScore(word) + cost` 最低者；from 是 BOS 時不套用（`path.front() != &graph.start()`）。
- **beam size** 不是「每位置只存 N 個」，而是前向搜尋時**只拿前 N 個前驅當候選**：`forwardSearch` 對每個節點取 `lattice[from]` 的前 `min(beamSize, size)` 個，算 `parent.score + model.score(parent.state, node, newState)` 取最大，再 `node.score = max + node.cost`，記 `prev`/`state`。
- 每個 graph 節點底下的 lattice 節點在算完後依 `score` 由高到低排序（`needSort`，預設 true）；這個順序是 beam 取「前 N」的前提。碼表的 `TableDecoder::needSort` 在 `graph.start().nextSize() == 1`（只有整段一條邊）時回 false（`table/tabledecoder.cpp`）。
- unknown 詞（`isNodeUnknown`）以 `from` 為 key 快取最佳前驅，省重複評分（`forwardSearch` 的 `unknownIdCache`）。
- n-best：`backwardSearch` 先取 EOS 的 `prev` 鏈當第 1 名；`nbest > 1` 時做 A*：從 EOS 往回擴，`gn` = 已走的後綴實際分，`fn = gn + from.score`（前綴最佳分作 heuristic），用 priority queue 依 `fn` 取出；到 BOS 就得一個句子，字串重複的丟掉；回溯時前驅只看前 `nbest * 2` 個（`decode` 的 `beamSize` 只給前向用，回溯呼叫寫死 `nbest * 2`）；另有 `max`（與最佳分的容許差距）、`min`（單步最低分）剪枝與 `MAX_BACKWARD_SEARCH_SIZE = 10000` 上限。
- 分數 = 語言模型分（`UserLanguageModel::score`，混合基礎模型與使用者歷史）+ 節點 `cost`；沒有 per-segment 的 β/γ 項。（`core/userlanguagemodel.cpp`）

## 3. 逐鍵增量更新（拼音引擎）

流程在 `pinyin/pinyincontext.cpp` `PinyinContext::update`：

1. 每鍵都對「未選定的剩餘輸入」**整串重新** `parseUserPinyin` 得到 `newGraph`。
2. `d->segs_.merge(newGraph, discardCallback)`（`core/segmentgraph.cpp`）：
   - `check()` 從 start 同步走新舊兩張圖，逐節點比出邊數與每條邊的 segment 文字，回傳**第一個不一致的節點 index** `since`（完全一致回傳 `end().index() + 1`）。
   - index < `since` 的舊節點保留，其指向 >= `since` 的邊改指新圖的節點；index >= `since` 的舊節點整批換成新圖的，並收進 `nodeToDiscard`（含超出新長度而被 `resize` 砍掉的）。
3. discard callback 呼叫 `Lattice::discardNode`：以 `nodeToDiscard` 為 key 的 lattice 項整個刪除，其他項裡 `from()` 落在被丟節點的 `LatticeNode` 也刪除（`core/lattice.cpp`）。同時丟掉 `matchState_`（拼音 trie 的比對狀態）。
4. `Decoder::decode`：先清 `nbests_`、刪 `nullptr`（EOS）key，把 **lattice 裡已存在的所有 graph 節點收進 `ignore`**；`buildLattice` 傳給 `dict->matchPrefix(graph, callback, ignore, helper)`，字典對 `ignore` 內的結束節點**不再產生詞**（`pinyin/pinyindictionary.cpp` 的 `ignore_.contains(&currentNode)`；碼表版 `TableBasedDictionary::matchPrefixImpl` 同理）；`forwardSearch` 遇到 `ignore` 內的節點直接略過，保留原 `score`/`prev`。EOS 每次重建。
5. 結果：新增一個字母時，只有「從第一個邊集合改變的位置起」的節點重算，前面不動。未被丟的節點之所以安全，是因為它們的所有前驅都在 `since` 之前、沒變（推論，從 `check` 的語意推出）。
6. 候選：n-best 之外，`update` 再掃每個結束位置的 lattice 節點，`from == bos` 的當作整詞候選（可選到某位置為止），其餘用分數窗（`min`、`maxDistance`）過濾（`PinyinContext::update`）。
7. **碼表引擎不做這件事**：`TableContext::update` 先 `d->lattice_.clear()` 再 decode，graph 每鍵由 `graphForCode(lastSeg, dict)` 全新建立，沒有 `merge`（`table/tablecontext.cpp`）。原因（推論）：graph 只有一條整段邊，沒有可重用的前綴。

## 4. 前綴選字與其餘繼續解碼

- 拼音：`PinyinContext::select(idx)` → `selectHelper`：把候選句的每個詞包成 `SelectedPinyin{offset = selectedLength + p->to()->index(), word, type}` 推進 `selected_`（`vector<vector<...>>`，一次選字一個群組），然後 `update()`。`update()` 的 `start = selected_.back().back().offset_`，新 graph 只解析 `userInput().substr(start)`，`state()` 用 `contextWords_` + 所有已選詞逐個 `model->score` 推出新的開頭 State，傳給 `decode` 當 `beginState`（`pinyin/pinyincontext.cpp`）。
- 候選覆蓋範圍小於游標時：`updateCandidatesToCursor` 把多詞句子從尾端 `pop_back` 到不超過游標，截成部分句子，即「選前半段、後半再解」（`candidatesToCursor`、`selectCandidatesToCursor`）。
- 取消：`cancel()` 彈出最後一組 `selected_`，並 `lattice_.clear()`、`matchState_.clear()`、`segs_ = SegmentGraph()`；註解說明前綴相同但開頭 State 不同，沒有重用意義。`cancelTill(pos)` 重複 `cancel` 直到 `selectedLength() <= pos`。
- 碼表：`TableContext::select(idx)` 一樣推進 `selected_`（`offset + p->to()->index()`），但候選句子一定覆蓋整個 graph（只有 `0→全長` 邊，外加 autoRuleSet 的切分邊），所以選字後 `selected()`（最後 offset == 輸入長度）為真，`update()` 走 `resetMatchingState()`，**碼表沒有「只選前綴、剩下繼續解」**。
- 碼表的「前綴固定」靠 `typeOneChar` 的自動上屏：在輸入新字元前，若「碼長達 `maxLength`」、「最後一字是 end key」或「`lastSeg + chr` 沒有任何匹配詞（noMatchAutoSelect）」，先 `autoSelect()` 把目前候選第一名（`autoSelectIndex_`）選掉，再用新字元重建 graph。`canDoAutoSelect` 要求首選不是 Auto 詞；沒候選時用原碼當 unknown 詞固定（`autoSelect`）。選定者的 State 由 `currentState()` 累積，之後的解碼從它開始。
- 選定的詞進 `learn()` 學習；多個 segment 可合併成新詞（`learnWord`）。

## 5. 對 QIM 的做法

QIM 的設計：位置 = 輸入碼的字母邊界；邊來自碼表（1–2 碼 → 字）與詞庫（碼串 → 多字詞）；分數 = unigram log prob + β·詞庫段數 − γ·段數；beam 20；每鍵只算新增位置；留 top-k。

**採用**
- lattice 以「結束位置」為索引、每位置一串節點（`word`、`from`、`score`、`prev`），EOS 單獨處理且每鍵重建。QIM 的位置是固定字母邊界，直接用 `Vec<Vec<Node>>` 即可，不需要 libime 的 graph 節點指標。
- 逐鍵只算新結束位置：新增一鍵時，QIM 的邊集合只在尾端新增（以字母邊界為位置，沒有拼音歧義切法），等價於 libime 的 `since` = 舊 EOS 位置；已算好的位置不動。這比 libime 的 `check()` 比對更簡單（推論）。
- 每位置節點按分數排序，前向只看前 `beamSize` 個前驅，與 beam 20 吻合；frame size 的概念（同一 (from,to) 最多保留 N 個詞）可對應「同碼多詞只留前 N」。
- n-best 用 A*：前向算出的 `score` 當 `fn` 的 heuristic，從 EOS 回溯，字串去重，有上限；QIM 的 top-k 可照此做。
- 前綴固定：把「已選字」當成 BOS 狀態的一部分，新 graph 只解析剩餘輸入；取消時整個重建（cancel 不嘗試重用）。

**避開或需要自己處理**
- libime 的分數是 n-gram LM + `cost`，沒有 β/γ 的段數項；QIM 的段數懲罰要放進邊分數（推論：每條邊加 −γ，詞庫邊再加 β），同一路徑上與 unigram 可加，不破壞 Viterbi 最優子結構。
- 碼表引擎沒有「碼表 + 詞庫」混合分段的先例，只有拼音引擎的多切法 graph；不能指望碼表端的程式碼回答分段問題。
- beam 只限制前驅數，不限制每位置的儲存量（儲存量靠 frame size）；QIM 若要限制記憶體需自己加。
- 部分前綴選字只有拼音引擎支援；QIM 若要「選前半詞、後半重解」要參考 `candidatesToCursor` 的截斷做法。
- `merge` 後根節點的開頭 State 是否會因 `select` 而過期，沒有查證（`selectHelper` 沒有清 lattice）；QIM 的前綴固定若改變 BOS 狀態，一律清 lattice 重建最安全。

## 來源

commit `53df6a248793f9a00c1769e658630de313f3e419`（master，2026-11-04 讀取）：
- https://github.com/fcitx/libime/blob/53df6a248793f9a00c1769e658630de313f3e419/src/libime/core/lattice.h
- .../core/lattice.cpp、.../core/lattice_p.h
- .../core/decoder.h、.../core/decoder.cpp
- .../core/segmentgraph.h、.../core/segmentgraph.cpp
- .../core/userlanguagemodel.cpp
- .../table/tablecontext.cpp、.../table/tabledecoder.cpp、.../table/tablebaseddictionary.cpp
- .../pinyin/pinyincontext.cpp、.../pinyin/pinyindictionary.cpp
（以上 `...` 皆同上述前綴 `https://github.com/fcitx/libime/blob/53df6a248793f9a00c1769e658630de313f3e419/src/libime/`）
