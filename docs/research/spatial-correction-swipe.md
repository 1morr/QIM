# 觸控空間模型、容錯、自動修正與滑動輸入：成熟輸入法怎麼做，對自訂佈局與形碼有什麼影響

查證日期 2026-11-12。為 M4（空間模型與容錯、英文預測與自動修正、萬用鍵）與 M5（英文滑動輸入、空白鍵移游標、評估中文滑動）備料。已在 repo 的內容不重寫：`docs/research/ranking-techniques.md` §5（libime 鄰鍵表、Ouyang 2017 的 Gaussian 與 bi-key）、`docs/research/open-source-imes.md` §1（FlorisBoard v0.5.2 與 ASK 的滑動、HeliBoard 的滑動是閉源函式庫）、`docs/design-brief.md` §4.8、§12.4。本檔只補它們沒有的。

每個斷言的標記：**[原]** 讀過原始碼或論文全文並引用；**[文]** 只讀到摘要或搜尋摘錄；**未查到** 找過沒有。「推論」是我的判斷，不是來源說的，集中在〈對 QIM 的建議〉。

釘住的版本：

| 專案 | 版本 |
| --- | --- |
| HeliBoard（AOSP LatinIME 的活躍分支，含 `ProximityInfo`、auto-correct、PointerTracker） | `3e91b4f1bf711a8bd3eb2d049c8b285fcc9cfa10` |
| AOSP LatinIME（設定檔對照） | `127336e9f29d69607eab55982324b210279ae8c5`（2025-02-26，repo 其他研究也用這個） |
| FlorisBoard | `v0.5.2` = `2e82060251897226c0739b9f52d1d051b02305fb` |
| librime | `bb56876890ba60f446fa2640c244b1794b965eda` |
| libime | `53df6a2` |
| qingjian | `4f5f7d2069e5fcc07dc20636f56aa6397e4fabf2` |
| ESDB（`en-wl/wordlist`） | `1e5b7d3a72f47a71da5d28686c1dd4b397178485`（分支 `v2`） |
| wordfreq | `912caf64b657478d1dff1138efdc078947d54bb1` |
| FrequencyWords | `525f9b560de45753a5ea01069454e72e9aa541c6` |
| aosp-dictionaries（Codeberg） | `ff0cd52be8d9530200fefe8b296c53d4f662091e` |

---

## 1. AOSP LatinIME / HeliBoard 的 `ProximityInfo`

### 1.1 做法 [原]

`ProximityInfo` 把鍵盤幾何轉成兩樣東西交給 native 解碼器：每個格子的鄰鍵清單（離散），以及每個鍵的 sweet spot（連續，預設關閉）。

**鄰鍵（離散）。** 鍵盤切成 `GRID_WIDTH × GRID_HEIGHT` 的格子（HeliBoard `config-common.xml`：32 × 16）。每格存「這格中心離哪些鍵近」。判定：格子中心到按鍵**邊緣**的距離平方小於門檻平方，門檻 = `mostCommonKeyWidth × 1.2`。每格最多存 `MAX_PROXIMITY_CHARS_SIZE = 16` 個字。`ProximityInfo.java:30-35, 239-345`：

```java
// Number of key widths from current touch point to search for nearest keys.
private static final float SEARCH_DISTANCE = 1.2f;
...
final int threshold = (int) (mMostCommonKeyWidth * SEARCH_DISTANCE);
...
if (key.squaredDistanceToEdge(centerX, centerY) < thresholdSquared) {
```

`mostCommonKeyWidth` 是所有鍵寬（含水平間距）的眾數，由 `KeyboardParams.updateHistogram` 統計（`KeyboardParams.java:170-206`）。所有鍵共用這一個值當尺度。

點擊時，第一個字 = 實際點到的鍵，後面接該格的鄰鍵；解碼器把輸入位置的字分三類：`MATCH_CHAR`（點到的）、`PROXIMITY_CHAR`（鄰鍵清單裡的）、`ADDITIONAL_PROXIMITY_CHAR`（寫死的，英文只有母音互換：`a→e i o u` 等，`additional_proximity_chars.cpp`）、其餘 `SUBSTITUTION_CHAR`（`proximity_info_state.cpp:200-252`）。

**代價（tap 路徑）。** `typing_weighting.h:75-100` 的 `getMatchedCost`：

```
cost = DISTANCE_WEIGHT_LENGTH(0.1524) × sweetSpotFactor(normalizedSquaredLength)
     + (若是鄰鍵) PROXIMITY_COST(0.0694) [+ FIRST_PROXIMITY_COST(0.07788)，第一個鄰鍵修正時]
```

`normalizedSquaredLength` = 點到該鍵中心的距離平方 ÷ `mostCommonKeyWidth²`（`proximity_info.cpp:129-137`）。所以 tap 的「機率」其實是連續距離項加離散鄰鍵常數，不是正規化的高斯。其他固定代價（`scoring_params.cpp`）：漏碼 `OMISSION_COST 0.467`、多碼 `INSERTION_COST 0.7248`、對調 `TRANSPOSITION_COST 0.5608`、`SUBSTITUTION_COST 0.3806`；只有 `normalizedSpatialDistance < 0.095`（`NORMALIZED_SPATIAL_DISTANCE_THRESHOLD_FOR_EDIT`）時才允許漏碼／多碼等編輯修正（`typing_traversal.h:42-45`）。

**sweet spot（連續，預設關）。** `touch_position_correction` 是每「排」三個數：x 位移（已廢，`getX` 永遠回 0）、y 位移（比例 × 按鍵高）、半徑（比例 × 按鍵對角線）。`ProximityInfo.java:176-210`：

```java
final float defaultRadius = DEFAULT_TOUCH_POSITION_CORRECTION_RADIUS  // 0.15f
        * (float)Math.hypot(mMostCommonKeyWidth, mMostCommonKeyHeight);
...
sweetSpotCenterYs[infoIndex] += touchPositionCorrection.getY(row) * hitBoxHeight;
sweetSpotRadii[infoIndex] = touchPositionCorrection.getRadius(row) * hitBoxDiagonal;
```

資料（`touch-position-correction.xml`）：預設陣列是空的（註解寫 `correctionX = 0.0f, correctionY = 0.0f, correctionR = DEFAULT_...`），名為 `holo` 的陣列有三排數字，如第一排 `0.0038756 / -0.0005677 / 0.1577026`。`TouchPositionCorrection.load` 在陣列長度為 0 時 `mEnabled = false`。HeliBoard 的 `KeyboardBuilder.kt:55-57` 只在佈局排數不是 4 時才載入 default 陣列，也就是等於不開：

```kotlin
if (keysInRows.size != 4) // that was effectively the default for OpenBoard
    mParams.mTouchPositionCorrection.load(mContext.resources.getStringArray(R.array.touch_position_correction_data_default))
```

啟用時，`TouchPositionCorrectionUtils.getSweetSpotFactor` 是分段線性：標準化平方距離 < 0.25（R1，sweet spot）內因子從 0 升到 0.24，到 1.0（R2）升到 1.20 後封頂。

### 1.2 自訂或非 QWERTY 佈局時的行為 [原]，部分推論

- 鄰鍵與距離都從 `Key` 的位置與大小算，不假設 QWERTY；所以 Dvorak、Colemak、HeliBoard 的自訂 JSON 佈局都照常運作。**事實**：這是「從幾何算」的做法。
- 但有四個隱含假設，換成畫布佈局會破：
  1. **鍵是軸對齊的矩形**：`squaredDistanceToEdge`、`Rect hitBox` 都是矩形（`Key.java:106, 220`）。旋轉、多邊形無處放。
  2. **一個字只對應一個鍵**：native 端 `mLowerCodePointToKeyMap[lowerCode] = i`（`proximity_info.cpp:167`），以小寫 code point 當索引，同字兩個鍵（例如左右各一個 `e`）後者蓋前者。
  3. **全鍵盤共用一個尺度**（眾數鍵寬）：鍵大小差很多的佈局，大鍵與小鍵用同一個正規化距離。
  4. **sweet spot 資料以「排」為單位**（`hitBox.top / mMostCommonKeyHeight` 算排號），沒有排的佈局沒有意義；而且預設就是關的。
- 英文母音互換是寫死在 C++ 裡的（原始碼自己的 TODO：`Stop using hardcoded additional proximity characters`、`Have proximity character informations in each language's binary dictionary`）。

---

## 2. 觸控空間模型的文獻

### 2.1 Goodman et al. 2002（Microsoft Research，IUI 2002）[原，讀全文]

「Language Modeling for Soft Keyboards」是最早的用法：點擊位置的分布（key press model）和語言模型相乘，選最可能的鍵序列。摘要：

> When a soft keyboard user hits a key near the boundary of a key position, the language model and key press model are combined to select the most probable key sequence. This leads to an overall error rate reduction by a factor of 1.67 to 1.87.

key press model 的結論（逐字）：

> We did find three factors that were helpful, which we used to form better models. First, the average position for each key was not the keys center, but was slightly shifted down and towards the center of the keyboard. Second, variances in the x and y position are typically different. Third, some distributions are rotated, rather than being perpendicular to between the x and y coordinates. Based on these observations, we used a bi-variate Gaussian distribution, with the means, variances, and covariance computed [...]

他們也列出**沒幫助**的因素：前一次與這一次位置的相關、pen up 位置、中心偏移的相關、打字速度與變異數的相關。實驗：每種條件打 1000 淨字元，錯誤率比 1.72–1.87（表 1，p = 0.005–0.028），別種算法最低 1.67；速度沒有顯著差別；使用者都偏好有修正的鍵盤。

[論文 PDF](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/p194-goodman.pdf)。

### 2.2 Ouyang et al. 2017（Gboard，arXiv:1704.03987）[原，讀全文]

已在 `ranking-techniques.md` §5.3 摘要，這裡補逐字與未提到的部分：

> The spatial model for tap input is typically a Gaussian distribution centered on each key center. Gesture inputs instead are often modeled with the so-called minimum-jerk model [...]

> Just like acoustic context dependency is encoded in speech with a C transducer, we implemented spatial context dependency in keyboard, and chose to do so with a bi-key model. Accordingly, the arcs of the transducer represent a:b transitions, where a:b means the key b with a as left context (previous key).

> Substitutions: [...] The decoder's spatial model handles substitutions by producing a probability distribution over the set of possible keys for each frame/tap. This substitution model takes into account the proximity between the touch point and the key.
> Deletions: [...] the decoder allows to skip one (non-epsilon) arc per frame in the lexicon FST without any input. The weight for these extra epsilon-transitions should represent the probability of omitting the letter in the given context.
> Insertions: The spatial model also allows taps to be treated as extraneous inputs due to accidental touches or spelling errors [...]

另有：「Text entry on a mobile touch-screen device is a very error-prone process, with per-letter error rates around 8-9%」。

論文**沒有**給 σ、沒有說 σ 如何隨按鍵大小改、沒有 bi-key 的參數怎麼學、也沒有談非 QWERTY 或自訂佈局。這些是**未查到**。

[PDF](https://arxiv.org/pdf/1704.03987)。

### 2.3 σ 與鍵大小的關係

| 來源 | 做法 | 備註 |
| --- | --- | --- |
| Fowler et al. 2015（Google，CHI 2015）[原] | 固定毫米：σx = 1.97 mm、σy = 1.88 mm；模擬用 Nexus 5 的鍵：寬 6.16 mm、高 9.42 mm | σ 約為鍵寬的 0.32、鍵高的 0.20（我算的）。數值取自 Azenkot & Zhai 2012 [1]。產生 per-tap 錯誤率 12.8%、38.4% 的詞至少有一個錯鍵 |
| VelociTap（Vertanen 2015，CHI 2015）[原] | 「The x- and y-variance of a key's Gaussian distribution is computed by multiplying the x- and y-size of each key in pixels by an x- and y-scale factor. We use the same two scale factors for every key.」 | 每鍵 σ 跟自己的大小成比例，全鍵共用兩個比例，在開發集上調。論文寫「variance」，字面是變異數乘以尺寸 |
| RoboType（Komninos 2023）[原] | 「zero as a mean, and a fraction of each key's width and height as the standard deviation」；範例輸出含 QWERTY、FITALY、OPTI-II 三種佈局 | 模擬使用者；在非 QWERTY 佈局上也用同一個比例 |
| FFitts law（Bi, Li, Zhai, CHI 2013）[文] | 觸控落點變異 σ² 拆成兩項：速度精度取捨造成的（隨目標大小與速度變）和手指絕對不精確 σa²（與取捨無關）：`T = a + b·log2( A / √(2πe(σ² − σa²)) + 1 )` | 說明：小鍵上 σ 不會隨鍵大小一路縮到零，有 σa 的底。論文以鍵盤打字實驗驗證 |
| AOSP [原] | 半徑 = 比例 × 該鍵對角線（預設比例 0.15，holo 資料約 0.15–0.158） | 不是 σ 但是同一個想法：尺度跟著鍵 |
| FlorisBoard 滑動 [原] | `LOCATION_STD(0.5109) × min(key.visibleBounds.height, key.visibleBounds.width)`（`StatisticalGlideTypingClassifier.kt:221`） | 形狀與位置兩個高斯，位置的 σ 隨鍵的短邊 |

Fowler 的解碼細節也有用：只考慮離 tap 距離在**剪枝半徑 7.99 mm** 內的鍵（`Algorithm 1`，開發集上調，「fast simulation without degrading model performance」），空間模型與 LM 的權重 γ = 0.000223（開發集調，因為空間模型的絕對值比 LM 大很多）。

### 2.4 針對鍵、姿勢、個人的調整

- Goodman：平均位置要往下、往鍵盤中心偏，不是鍵中心；x、y 變異不同；有旋轉（共變異數）。
- Yin et al. 2013（Hierarchical Spatial Backoff Model，CHI 2013）[原，讀摘要與前文]：把空間模型依「目標鍵位置、手姿勢、個人」分層，資料不夠或姿勢不確定時退回較一般的模型。摘要：「a specific submodel should be applied only if its corresponding input posture can be identified with confidence, and if the submodel has enough training data from the user. We introduce the backoff mechanism to fall back to a simpler model」，原型把「language-model-independent error rate」降 13.2%（姿勢分類 86.4%）。[PDF](https://groups.csail.mit.edu/mug/pubs/Yin2013Making.pdf)。
- Fowler 2015 明說為什麼只用簡單 2D Gaussian：「2D Gaussian is at the core of spatial models used in commercial level keyboards such as the Android Open Source Project (AOSP) keyboard」，更複雜的（Holz、Azenkot & Zhai 提到手指位置、視覺提示、手姿勢造成的偏移）留給之後。[PDF](https://research.google.com/pubs/archive/43272.pdf)。
- Weir et al. 2014（CHI 2014）用 Gaussian Process 做使用者專屬的偏移模型再接 LM 做 autocorrect [文，只讀到他人引述]。Gunawardana et al. 2010 的 anchor area（鍵中心一塊範圍內的觸控固定回該鍵）[文，只讀到 Wobbrock 的引述]。

### 2.5 bi-key context

只有 Ouyang 2017 明說用（見 §2.2），沒有其他來源給參數、收益或訓練方式。**未查到**單獨的收益數字。

---

## 3. 自動修正

### 3.1 LatinIME / HeliBoard 的 auto-correction threshold [原]

**門檻與分數。** AOSP `config-auto-correction-thresholds.xml`（LatinIME `127336e`）：

```
Off: floatMaxValue / Modest: 0.185 / Aggressive: 0.067 / Very Aggressive: floatNegativeInfinity
plausibility_threshold: 0.065   // "slightly less than the aggressive threshold"
```

比較的值是 `normalizedScore`（`autocorrection_threshold_utils.cpp`）：

```
normalizedScore = (score / SUGGEST_INTERFACE_OUTPUT_SCALE) × (1 − editDistance / afterLength)
```

編輯距離用 Damerau-Levenshtein；`editDistance >= afterLength` 或 score ≤ 0 回 0。HeliBoard 把三段檔位改成 0–1 的 confidence：`threshold = 0.5 − 0.5·confidence^0.33`，另有 `scoreLimit = (1.14 − 0.2·(confidence + 0.47)^3.5) × 900000`（`SettingsValues.java:229-234`，原碼註解「just some formulas that give something similar to the old values」）。

**何時自動替換。** `Suggest.kt:183-288` 的 `shouldBeAutoCorrected`：先算 `allowsToBeAutoCorrected`（輸入的字長 > 1 且不在字典，且含 `@` 或 `.` 時建議也要含；或第一建議是 whitelist），再用下面任一條擋掉（節錄原註解，括號內是我的摘述）：

- 設定關閉（"we never auto-correct"）、不在組字（預測狀態）。
- `wordComposer.hasDigits()`：「If the word has digits, we never auto-correct because it's likely the word was type with a lot of care」。
- `isMostlyCaps && !isAllUpperCase`：「we never auto-correct because this is almost certainly intentional」。
- `isResumed`：「We never auto-correct when suggestions are resumed because it would be unexpected」。
- 沒有已初始化的 main dictionary：「no main dict => no auto-correct」。

通過後再比 threshold；若輸入的字本身在字典且分數好，用 ngram context 比「第一建議 + 加成」與「輸入字」的空前文分數，差 20 以上才換。替換實際發生在使用者打下一個分隔符（空白、標點）時（Ouyang 2017：「this candidate will automatically replace whatever was typed after the user enters the next separator」）。

**Ouyang 2017 的保守設計。** 同篇：「If the model is too aggressive, it will start making false autocorrections away from correctly typed words [...] imagine the keyboard autocorrecting "you're far" to "you're fat" since [...] on a QWERTY layout the R is adjacent to the T.」做法：用字元層模型給 literal（實際點的字串）算機率，可和字典內詞比，並且「applies an additional cost for correcting away from an already valid word」。

### 3.2 退格還原（revert）[原]

HeliBoard `InputLogic.java:1308`、`1886-1965`：

```java
if (mLastComposedWord.canRevertCommit() && sv.mBackspaceRevertsAutocorrect) {
    ... revertCommit(inputTransaction); ... restartSuggestionsAfterDelete(inputTransaction);
```

`LastComposedWord` 在每次上屏時存：原打字串 `mTypedWord`、上屏的字 `mCommittedWord`、分隔符、`NgramContext`、原始點擊 `mInputPointers`。`canRevertCommit()` = 還在有效期 `mActive`、上屏非空、**且上屏的字不等於原打字串**（只有真的被改過才可還原）。`revertCommit`：刪掉「上屏字 + 分隔符」，**`unlearnWord(committedWord, ..., UnlearnEvent.REVERT)`**（被還原的修正字不計入學習），改上屏原打字串（空白當分隔符時改成 PHANTOM 空白，下一次按鍵才補），並把被拒絕的字連同原本的建議列放進 `SuggestionSpan`。任何其他編輯、游標移動都會 `mLastComposedWord.deactivate()`。沒有空白的語言（中日）還原後直接回到組字狀態。

### 3.3 「打錯碼的基準集合」怎麼產生（合成錯誤模型）[原]

公開的做法有四種：

1. **Fowler 2015 的 Sketch Jr.**：對目標詞的每個字母，以**鍵中心為平均**從 2D Gaussian 抽一個 tap（σ 見 §2.3）；落在別的鍵內 → 該鍵的字進 literal（打錯）；落在所有鍵外 → 丟掉重抽。這組設定給 12.8% 的 per-tap 錯誤率，38.4% 的詞至少一個錯鍵。解碼取最佳詞，機率高於 autocorrect threshold 才換；0.0 時 false correction > 3.5%（約占所有失敗的一半），門檻 0.6 對整體正確率最佳，選 0.7（false correction < 1%，整體正確率稍降）。評分分「literal 本來就對卻被改錯」（false correction）與其他失敗分開統計。
2. **RoboType（2023）**：同樣用「σ = 鍵寬／高的比例」抽 tap，再加手指速度（隨距離）造成的誤差；佈局可換（QWERTY、FITALY、OPTI-II）。開源 Python。
3. **真實資料**：Vertanen & Kristensson, *A Dataset of Noisy Typing on QWERTY Keyboards*（IUI 2023）[文，只查到列表]。
4. **LLM 合成**（KeySense，arXiv:2602.12432）[文]：依字長設編輯數上限，`E_max(L) = 2 (L ≤ 6), 3 (7 ≤ L ≤ 9), 4 (L ≥ 10)`。與 QIM 的情境不同（十指盲打），列出只為完整。

沒有找到專為「漏碼、多碼、對調」有公開比例的資料；Ouyang 2017 只說解碼器支援三種運算與權重由模型學出，沒給比例。Fowler／RoboType 只產生替換錯誤（空間錯誤），不產生漏碼、多碼、對調。**未查到**公開的漏碼／多碼／對調比例。

---

## 4. 產品對形碼（倉頡、速成、五筆）的容錯或滑動輸入

| 對象 | 找到的 | 來源 |
| --- | --- | --- |
| Rime（librime，含 Trime、Hamster、Squirrel、Weasel） | 有 `Corrector`（`corrector.cc`），**只接在 `script_translator`（拼音類）**，`table_translator`（形碼碼表）與 `table_db.cc` 完全沒有 correction。做法：QWERTY 鄰鍵表寫死在 C++（每鍵只列同排左右；數字行含下排斜角），受限 Damerau 距離，成本：增刪 2、鄰鍵替換 1、其他替換 4、對調 2；對 `Syllabary` 的音節預先建 `SymDeleteCollector`（刪字索引）。fuzzy（`derive` 拼寫代數）也是 script translator 的事 | [原] librime `bb56876` `src/rime/dict/corrector.cc`，`src/rime/gear/script_translator.cc:81-128`（`enable_correction`）；`table_translator.cc` 無 `correct` 字樣 |
| libime / fcitx5 | `PinyinCorrectionProfile` 只給拼音；table 引擎（倉頡、五筆、速成表）沒有容錯。有一個**萬用鍵** `matchingKey`（設定名「Wildcard matching Key」），輸入碼中出現時改用萬用比對模式 | [原] libime `53df6a2` `tablebaseddictionary.cpp:1618-1630`；`fcitx5-chinese-addons` `im/table/ime.h:193-196` |
| 青簡 | 拼音糾錯一處編輯、替換限 QWERTY 鄰鍵；雙拼與形碼（五筆）不糾錯；沒有倉頡、速成 | repo `docs/research/qingjian.md` §6 |
| Gboard 速成／倉頡 | 容錯只靠點擊位置，原樣優先，漏碼與對調不救；字根鍵長按出英文字母。第三方站（hkcards）寫「長按按鍵或下拉即可輸入英文字母」。官方沒有倉頡／速成的滑行輸入文件 | repo `design-brief.md` §12.4（黑箱實測）；hkcards [文]；Gboard 說明「Slide your finger to type」只寫「Important: Not all languages work with this feature.」，沒列語言、沒提中文、沒提 flick [原，2026-11-12 讀] |
| macOS／iOS 內建倉頡、速成 | 倉頡有**萬用鍵**：「Use the Z key, the Cangjie function key, in place of any unknown keys. For example, if you aren't sure of the second key for 蘋, you can enter "廿重竹金" (tzhc)」，可在同一碼中多次用。速成一頁沒提萬用鍵。無容錯、無滑動 | [Apple 倉頡](https://support.apple.com/guide/chinese-input-method/cangjie-traditional-cimcjt11820/mac)、[Apple Cantonese 倉頡](https://support.apple.com/guide/chinese-input-method/cangjie-cantonese-cimd4bef33e6/mac) [原] |
| 微軟新倉頡／新速成 | 搜到的只有安裝與相容性設定，沒有容錯或萬用鍵的官方說明 | **未查到** |
| 搜狗五筆、百度五筆 | 只查到產品介紹（五筆拼音混輸、「智能纠错」是拼音的），沒有五筆本身容錯的說明 | **未查到** |
| 滑行輸入 × 形碼 | 在 Gboard、Rime 系（Trime、Hamster：樹中無 glide 相關檔名）、FlorisBoard、ASK、HeliBoard 都**未查到**任何形碼的滑行輸入。最接近的是青簡：滑行只給拼音 26 鍵與英文，且需要「按詞給的字母串詞表」：拼音是「主詞庫每個詞的音節連起來」。作者明確寫「双拼、五笔的字母不是这样拼词的」所以不給 | [原] qingjian `docs/design/voice-and-swipe.md` §滑行·範圍 |

補充：形碼的容錯空白不是因為沒人想過：速成每個兩碼組合幾乎都是合法碼（brief §4.8），單字層容錯無法分辨；唯一有公開實作的是拼音（有音節詞表可驗）與英文（有詞典可驗）。

---

## 5. 滑動輸入與按鍵自己的滑動動作（flick）並存

| 產品 | 怎麼分 | 來源 |
| --- | --- | --- |
| AOSP／HeliBoard | **按下時**就決定這根手指要不要偵測滑行：只在字母鍵盤、從非修飾鍵開始、且該手指不是 key swipe（空白鍵與刪除鍵若設了滑動動作）才偵測。偵測後還要「快速移動 + 離開起點夠遠 + 夠久」才算：`mDetectFastMoveSpeedThreshold = 1.5 鍵寬/秒`、動態距離門檻從 6.0 鍵寬降到 0.35 鍵寬、時間門檻從 300 ms 降到 20 ms、剛快打完 500 ms 內更嚴；起點在字母鍵才能啟動（`Character.isLetter(key.getCode())`）；彈出 popup 面板時不進手勢。門檻都以鍵寬為單位 | [原] `PointerTracker.java:687-689, 776-800`；`GestureStrokeRecognitionParams.java:42-54` |
| FlorisBoard v0.5.2 | 兩個偵測器並行。滑行要：**距離 > 一個鍵寬、速度 > 0.10 dp/ms、起點鍵不在 `SWIPE_GESTURE_KEYS`（空白、刪除等）**（`GlideTypingGesture.kt:91`）。鍵上的 swipe 動作（全域設定，不是每鍵）只在**滑行輸入關閉**時對字母鍵生效：`!isGlideEnabled && !pointer.hasTriggeredGestureMove`（`TextKeyboardLayout.kt:734`）。也就是滑行開著就沒有字母鍵上滑。長按已觸發的指頭不進滑行（`hasTriggeredLongPress`）。master 的 LDML 格式每鍵有 `flick`，但 glide 被整個註解掉，無法對照 | [原] FlorisBoard `v0.5.2` |
| 青簡（Android，自繪） | **上滑 flick 與滑行在 view 層用方向扇形分開**：起點在有 flick 的鍵上時，離開按下點 10 dp 後一律要落在**向上的扇形**內（`side ≤ up × 1.2`），往上超過 18 dp 就 armed、抬手出上滑字；中途偏出扇形（橫走、往下、拐彎）就 `flickBroken`，這一筆交給滑行或點按。滑行本身：「手指落在字母键上就开始记点，离起点挪开 0.6 个键宽才算滑行」，第二根手指在滑開前落下當作快打、不當滑行。刪除鍵上滑清空同理 | [原] `KeyboardView.kt:121-130, 360-395, 543-552`、`docs/design/voice-and-swipe.md` |
| Gboard | **未查到**官方或原始碼說明（Gboard 說明頁只寫 Glide typing、空白鍵滑動移游標、刪除鍵向左滑刪詞） | [Gboard 說明](https://support.google.com/gboard/answer/2811346)、Play 商店說明 |
| Hamster（iOS） | 每鍵 `swipe: [{direction: up\|down\|left\|right, action}]` 與全域 `distanceThreshold: 20`、`tangentThreshold: 0.577`（約 ±30° 扇形）；樹中沒有 glide 相關程式 → 沒有衝突要處理 | [原] repo `open-source-imes.md` §Hamster；樹檢索 |
| Trime（Android） | 每鍵 `swipe_*`；樹中查不到 glide 功能（官方 repo 只查到釋出紀錄提到 `swipe distance` 設定可設 0 關閉手勢） | [文] |

整理：三種做法 — **(a)** 全域二選一（FlorisBoard：開滑行就關字母鍵 flick）；**(b)** 起點與按鍵種類決定（AOSP：字母鍵才偵測、空白與刪除是 key swipe）；**(c)** 方向扇形加距離（青簡、Hamster：扇形角度 ±約 30–50°，距離 18–20 dp）。(c) 是唯一能讓「同一個鍵既有 flick 又能當滑行起點」並存的做法。

---

## 6. 英文詞頻資料：來源與授權（要含 z 開頭的詞）

背景：`docs/plan.md` 第 540 行現在的英文資料包 `scowl-en-us` 取 ESDB 的 hunspell `en_US` 展開，且「只留轉成小寫後全是 a–y 的詞（`z` 不在速成字母表）」，沒有詞頻、也沒有 z 開頭的詞；英文模式預測要詞頻、要含 z（plan 的原話：「英文模式的預測（M4）要詞頻，開工時另查」）。

下表的 z 檢查是**我當場抽查**，方法在表後。

| 來源 | 內容 | 授權 | 可商用 | 抽查（z 開頭） | 備註 |
| --- | --- | --- | --- | --- | --- |
| **ESDB**（前身 SCOWL）`en-wl/wordlist` `v2` | 英文拼字資料庫，以「size」分級（35–80，預設 60）而非詞頻，另有詞形、方言、詞性；SQLite 與文字檔 | 「Permission to use, copy, modify, distribute, and **sell** any part of the [ESDB], or word lists created from it, is hereby granted without fee」，須保留版權聲明；size > 80 要另附 UKACD 版權；含澳洲拼法要另附 AU 版權；COCA 的 3-gram 資料「is not freely available; however, the usage is within the rights given by the NDA」；WordNet 版權「MIGHT apply」 | 可（MIT-like，須保留聲明）；size ≤ 60 的現行用法已符合 | 有（詞表本身，不需抽查） | 沒有逐詞頻率，只有 size 層級 |
| **HeliBoard／OpenBoard 用的 AOSP 詞典** `Helium314/aosp-dictionaries`（Codeberg） | `main_en_us__experimental.dict`：279,960 詞、逐詞 `f`（0–255 的對數頻率）、含 bigram；由 `wordlist.py` 用 Leipzig Wortschatz 詞表建 | README：「source lists under CC BY 4.0」。`main_en_us.dict`（legacy，160,715 詞，2014）來自 OpenBoard v1.4.5 的 `en_wordlist.combined.gz`，README 只給來源連結、**未查到**授權文字 | experimental：CC BY 4.0，可，須標示。legacy：授權**未查到** | 425 個 z 開頭詞：`zero f=126`、`zoom f=92`、`zebra f=69`、`zucchini f=65`、`zygote f=31` | 原始檔 `wordlists/main_en_US__experimental.combined`，格式 `word=…,f=…` + `bigram=…,f=…`。README 警告生成腳本「still experimental, rather slow and may produce bad dictionaries in some languages」。Leipzig 本身授權未另查 |
| **wordfreq**（rspeer）`912caf6` | 40+ 語言的詞頻 Python 套件；英文資料來源含 Google Books Ngrams 2012、OpenSubtitles／SUBTLEX、Wikipedia、NewsCrawl、OSCAR、Reddit、Twitter | 程式 Apache；資料「may be redistributed under a Creative Commons Attribution-ShareAlike 4.0 license」；SUBTLEX 要署名並註明為免費資料；Twitter 資料不轉載內容 | 可（CC BY-SA 4.0 允許商用，條件是署名與相同方式分享；散布時 SA 範圍是否及於包含它的 App 需法律意見，**未查到**作者的說明） | 未下載，不抽查 | **已停更**：「The word frequencies are a snapshot of language usage through about 2021」「the data is unlikely to be updated again」 |
| **hermitdave/FrequencyWords** `525f9b5` | 由 OpenSubtitles 2016／2018 統計的各語言詞頻（`content/2018/en/en_50k.txt` 等） | README：「MIT License for code」「CC-by-sa-4.0 for content」（GitHub 偵測到的 repo 授權為 MIT） | 資料 CC-BY-SA 4.0：可商用，同上 SA 疑慮。底層字幕本身的著作權處理：**未查到**（OPUS 頁面 404、搜尋只找到要求「add a link to [OpenSubtitles] to your website and to your reports」） | `en_50k.txt` 有 196 個 z 開頭：`zero 19582`、`zone 17386`、`zoo 8483`、`zebra 1752`、`zucchini 658` | 口語、字幕體；詞形各自獨立計數（未做詞幹合併） |
| **Peter Norvig `count_1w.txt`** | Google Web Trillion Word Corpus 的 33 萬詞、原始計數 | 「Code copyright (c) 2008-2009 by Peter Norvig. You are free to use this code under the MIT license」；資料「derived from the Google Web Trillion Word Corpus [...] distributed by the Linguistic Data Consortium」，**資料本身授權未明寫** | **未查到**資料授權 → 不建議 | 2,276 個 z 開頭（含大量非詞）：`zip 70551270`、`zone 46897368`、`zero 30735412`、`zebra 2902469`、`zygote 129318` | 太髒（含網路雜訊與非詞） |
| **Google Books Ngram** | 1-gram 到 5-gram 年份計數，版本 2009／2012／2020（v3） | 頁面：「This compilation is licensed under a Creative Commons Attribution 3.0 Unported License.」wordfreq 引用的用語為「Ngram Viewer graphs and data may be freely used for any purpose」 | 可，須標示 | v2 有按首字母切檔（`googlebooks-eng-all-1gram-20120701-z.gz`） | 資料量大（要先自己彙整）；書本語體、含 OCR 雜訊；只有詞形計數，不分詞性 |
| **OpenSubtitles／OPUS** | 字幕平行語料，英文含數十億 token | 頁面無授權條款文字（OPUS 的 `/OpenSubtitles` 頁 404）；Hugging Face 資料卡 `license: ""`（空）；要求「add a link to [opensubtitles.org] to your website」 | **未查到** → 不要直接用，透過 FrequencyWords／wordfreq 用等於繼承同一疑慮 | — | 原始語料太大，不建議自己處理 |

z 開頭詞的抽查方法：Norvig 檔用 `grep -c "^z" count_1w.txt`；FrequencyWords 用 `grep -c "^z" en_50k.txt`；aosp-dictionaries 用 `grep -c "word=z"` 於 `main_en_US__experimental.combined`。三者都含常用 z 詞（`zero`、`zone`、`zoom`、`zoo`、`zebra`），所以「來源沒有 z 詞」不是風險；風險在 plan 現行管線自己把 z 濾掉了。

---

## 對 QIM 的建議

### 有來源支持

1. **tap 的空間模型用 2D Gaussian，σ 跟各鍵自己的大小成比例，全鍵共用比例係數、用基準調。** VelociTap 做的就是這樣（§2.3），Fowler 也說 AOSP 的核心是 2D Gaussian。σ 不要縮到零：FFitts 的 σa 是手指絕對精度的底。
2. **用實體尺寸（mm／dp）而非像素當單位**。Fowler 的 σ 與剪枝半徑都是毫米，AOSP 也是比例式的，`ProximityInfo` 裡寫死像素的只有鍵盤自己的格子。
3. **只對距離近的鍵算機率**（剪枝）。Fowler 剪枝半徑 7.99 mm，AOSP 是 1.2 個鍵寬（離邊緣）。每格最多 16 個鄰鍵是 AOSP 的記憶體上限。
4. **autocorrect 對「原樣讀得通」要保守**：Ouyang 對已是合法詞的 literal 加額外成本；Fowler 把 false correction 獨立量測，threshold 0.0 時 > 3.5%，0.7 時 < 1%；AOSP 的 `shouldBeAutoCorrected` 有一長串不自動修的條件（含數字、全大寫或大部分大寫、resumed、email／網址字元、沒有主字典）。這與 brief §4.8「預設相信使用者」一致。
5. **退格還原**照 AOSP：存原打字串、上屏字與分隔符；只有「真的被改過」才可還原；還原時不把修正字計入學習（unlearn）；原打字串回到組字或放回，被拒絕的字留在建議列；別的編輯或游標移動就失效。
6. **萬用鍵**：Apple 倉頡以 `Z` 代替任意未知碼、可重複使用；libime table 引擎有單一 `matchingKey` 設定。兩者都是「碼表的比對模式」，不是容錯。
7. **滑行不要假設全鍵盤皆字母鍵**：AOSP 在按下時就用「鍵種類 + 是否有 key swipe」決定要不要偵測；青簡用方向扇形把上滑 flick 與滑行分開。
8. **基準集合的錯誤模型**：用 Fowler 的方式在**佈局幾何上**抽 tap（鍵中心為平均、σ 同上），落到哪個鍵就是哪個碼；落在所有鍵外則重抽。RoboType 證明同法可用在 FITALY、OPTI-II 這類非 QWERTY 佈局。Fowler 還把「literal 本來就對卻被改錯」獨立統計，這是 QIM 的容錯要守的指標。

### 推論

**自訂佈局（畫布模型）**

- AOSP 的 `ProximityInfo` 不能直接搬：它假設矩形、一字一鍵、一個共用尺度、以「排」為單位的 sweet spot（§1.2）。QIM 的 `Key::Weighted` 反而更簡單：不要預建格子，每次點擊對**每個**鍵直接算「點到形狀的有號距離」（在形狀內為 0，旋轉與多邊形都只是距離函式不同），再套 σ_i。鍵數 < 40，Fowler 剪枝半徑內通常只有 3–6 個鍵，計算量可忽略（ranking-techniques §5.6 的結論同）。
- **σ_i 以該鍵的「有效大小」算**。矩形用 min(寬, 高) 或兩軸分別乘係數（Fowler 的 x、y σ 不同；VelociTap 也分 x、y）。旋轉的鍵把共變異數轉到鍵的區域座標（Goodman 觀察到分布有旋轉；他看到的是人手造成的旋轉，不是鍵旋轉造成的，所以這裡是我借用）。多邊形可用面積等效或最小外接橢圓。σ_i = √(σa² + (k·size_i)²)（套 FFitts 的兩項結構；這個式子不是來源給的，係數要用基準調）。
- **重疊的鍵**：使用者肉眼看到的是最上層的鍵，但落點機率不該被 z-order 吃掉。每個鍵獨立算高斯，最上層（點擊判定的贏家）給額外的先驗加成，被蓋住的鍵仍有機率。AOSP 的「後者蓋前者」（`mLowerCodePointToKeyMap`）不適用，因為 QIM 允許同一字多個鍵：同字多鍵時，該字的機率取各鍵的和或最大（需用基準選）。
- **偏移（Goodman 的向下、向中心偏）**：自訂佈局沒有可預訓練的逐鍵資料。Yin 2013 的 backoff 說明正確的順序：先用共用模型（與鍵大小成比例，不分鍵），累積足夠個人資料後再逐鍵調整，資料不足就退回。不要先做逐鍵參數。
- **bi-key**：只有 Ouyang 做過，沒有可抄的參數；先不做。
- **鄰鍵表（實體鍵盤）**不受這些影響：brief §4.8 的「按實體鍵位」就是用實體鍵盤的幾何，不涉及畫布。
- **滑動輸入的解碼**：FlorisBoard 的 σ 隨鍵短邊、路徑剪枝的距離門檻用「第一個鍵的寬度 / 4」（`StatisticalGlideTypingClassifier.kt:136`）：後者假設鍵大小一致。換成畫布要改成逐鍵（各鍵自己的 σ）。
- **flick 與滑行並存**（plan M5 的「開工時定」）：用青簡的方向扇形做法最貼 QIM：按下時看該鍵有沒有定義 flick；有則在起點的扇形內走夠遠才是 flick，離開扇形或換到別的鍵就改走滑行或點按。扇形角度、距離門檻用 dp，與鍵的大小無關（青簡 18 dp、Hamster 20 dp）。旋轉的鍵：flick 方向到底是螢幕方向還是鍵的區域座標，來源沒有答案；建議用螢幕方向（使用者的手勢是相對螢幕的），編輯器顯示箭頭時也用螢幕方向。滑行的門檻建議單位用鍵寬（AOSP）或 dp（FlorisBoard、青簡）都行，但畫布上鍵大小不一，用 dp 較穩。

**形碼（速成、倉頡、五筆）**

- 所有開源實作中，容錯與滑行都只做在**有詞表可驗的字母串**上（拼音、英文）；形碼在 Rime、libime、青簡、Gboard 都是「原樣 + 空間模型」。這支持 brief §4.8 的「單字不容錯、只在詞與句層」，也與 QIM 先前黑箱實測 Gboard 速成的結論一致。**沒有查到任何一個形碼的容錯做法可以抄**；詞與句層的容錯是 QIM 自己的設計，要靠基準驗證。
- 速成的特性放大了 Ouyang 與 Fowler 提的風險：兩碼幾乎都是合法碼，「原樣已是合法詞」的判斷幾乎恆真，所以 Ouyang 的「對已是合法詞的修正加成本」在單字層等於全面關掉容錯；只有詞、句層（多字，詞庫與前後文能分辨）才有判斷依據。AOSP 的「長度 > 1 且不在字典才可能被改」在形碼單字層沒有對應物。
- Fowler 的 autocorrect threshold 實驗給的是**操作方法**，不是數字：先把 false correction 的上限定下來（<1%），再往上調容錯強度。QIM 要做的是對應的指標：「原樣讀得通卻被容錯路徑蓋掉的比例」。
- 萬用鍵：倉頡的 `Z`（Apple）在速成上的語意不明：速成只取首尾碼，`Z` 在中間沒有意義；若放首或尾，等於「不知道頭／尾字根」，結果是大量重碼。此處需自行設計並用基準驗，來源沒有速成的萬用鍵先例（Apple 速成頁沒提）。
- 形碼滑行：**沒有先例可循**。可行性推論：滑行路徑含 2n 個鍵（n 個字、每字首尾碼），詞表可以是「詞 → 碼串」，結構與青簡的拼音滑行同；但每個 2 碼對應很多字，形狀只能區分鍵序列，字的選擇全落在詞庫與語言模型。M5「評估中文滑動」建議先用 qingjian 的作法做離線模擬（用 §3.3 的 Gaussian 抽樣生成假路徑，量詞首選率與 top-5），再決定要不要做 UI。

**英文資料**

- plan 現行的英文管線同時缺詞頻、又濾掉 z（因 z 不在速成字母表）。英文模式用的是一般英文鍵盤，不受速成字母表限制，所以 z 的濾除只該限於「速成資料包」，不該限於英文模式的資料。
- 以授權簡單與有逐詞頻率為準，首選 **aosp-dictionaries 的 experimental 詞典**（CC BY 4.0、有 `f` 與 bigram、已含 z 詞、可轉成自己的格式；要標示來源）。次選 wordfreq（CC BY-SA 4.0、已停更、需自行處理 SA 的範圍）。FrequencyWords 與 Norvig 因資料授權疑慮不建議。ESDB 可繼續當「是否為合法拼字」的白名單（size 60），詞頻由上述來源補。
- 拼字白名單與詞頻的交集有取捨：詞頻來源詞形多（專有名詞、縮寫、拼錯），要用 ESDB 過濾或設最低頻率。

### 還需要自己量的

- σ 的比例係數與其下限（σa），用 §3.3 的合成方式在 QIM 的實際預設佈局上對基準調；plan §7.5 的「含打錯碼集合用較粗的錯誤模型（鄰鍵等機率加漏碼、多碼、對調）」是使用者同意的做法，與本檔建議的 Gaussian 抽樣可並行：前者測詞與句層的編輯容錯（漏碼、多碼、對調），後者測空間模型本身，兩者量的東西不同。
- 形碼的詞、句層容錯懲罰與啟動條件：來源沒有數字。
- 畫布佈局下的 flick／滑行門檻（dp 與角度）：參考值 18–20 dp、±30° 到 ±50°，要實機驗。

---

## 參考

- Goodman, Venolia, Steury, Parker. *Language Modeling for Soft Keyboards.* IUI 2002. <https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/p194-goodman.pdf>
- Ouyang, Rybach, Beaufays, Riley. *Mobile Keyboard Input Decoding with Finite-State Transducers.* arXiv:1704.03987. <https://arxiv.org/abs/1704.03987>
- Fowler, Partridge, Chelba, Bi, Ouyang, Zhai. *Effects of Language Modeling and its Personalization on Touchscreen Typing Performance.* CHI 2015. <https://research.google.com/pubs/archive/43272.pdf>
- Vertanen, Memmi, Emge, Reyal, Kristensson. *VelociTap.* CHI 2015. <https://www.repository.cam.ac.uk/bitstreams/bd481ad9-db9d-4e03-a65e-295a79bb5664/download>
- Bi, Li, Zhai. *FFitts Law.* CHI 2013. <https://research.google.com/pubs/archive/41645.pdf>
- Yin, Ouyang, Partridge, Zhai. *Making Touchscreen Keyboards Adaptive to Keys, Hand Postures, and Individuals.* CHI 2013. <https://groups.csail.mit.edu/mug/pubs/Yin2013Making.pdf>
- Komninos. *RoboType.* MobileHCI 2023 Companion. <http://www.komninos.info/sites/default/files/sample-authordraft_0.pdf>
- HeliBoard `3e91b4f`：[`ProximityInfo.java`](https://github.com/Helium314/HeliBoard/blob/3e91b4f1bf711a8bd3eb2d049c8b285fcc9cfa10/app/src/main/java/com/android/inputmethod/keyboard/ProximityInfo.java)、[`proximity_info.cpp`](https://github.com/Helium314/HeliBoard/blob/3e91b4f1bf711a8bd3eb2d049c8b285fcc9cfa10/app/src/main/jni/src/suggest/core/layout/proximity_info.cpp)、[`proximity_info_state.cpp`](https://github.com/Helium314/HeliBoard/blob/3e91b4f1bf711a8bd3eb2d049c8b285fcc9cfa10/app/src/main/jni/src/suggest/core/layout/proximity_info_state.cpp)、[`touch_position_correction_utils.h`](https://github.com/Helium314/HeliBoard/blob/3e91b4f1bf711a8bd3eb2d049c8b285fcc9cfa10/app/src/main/jni/src/suggest/core/layout/touch_position_correction_utils.h)、[`typing_weighting.h`](https://github.com/Helium314/HeliBoard/blob/3e91b4f1bf711a8bd3eb2d049c8b285fcc9cfa10/app/src/main/jni/src/suggest/policyimpl/typing/typing_weighting.h)、[`scoring_params.cpp`](https://github.com/Helium314/HeliBoard/blob/3e91b4f1bf711a8bd3eb2d049c8b285fcc9cfa10/app/src/main/jni/src/suggest/policyimpl/typing/scoring_params.cpp)、[`Suggest.kt`](https://github.com/Helium314/HeliBoard/blob/3e91b4f1bf711a8bd3eb2d049c8b285fcc9cfa10/app/src/main/java/helium314/keyboard/latin/suggestions/Suggest.kt)、[`InputLogic.java`](https://github.com/Helium314/HeliBoard/blob/3e91b4f1bf711a8bd3eb2d049c8b285fcc9cfa10/app/src/main/java/helium314/keyboard/latin/inputlogic/InputLogic.java)、[`LastComposedWord.java`](https://github.com/Helium314/HeliBoard/blob/3e91b4f1bf711a8bd3eb2d049c8b285fcc9cfa10/app/src/main/java/helium314/keyboard/latin/LastComposedWord.java)、[`PointerTracker.java`](https://github.com/Helium314/HeliBoard/blob/3e91b4f1bf711a8bd3eb2d049c8b285fcc9cfa10/app/src/main/java/helium314/keyboard/keyboard/PointerTracker.java)、[`GestureStrokeRecognitionParams.java`](https://github.com/Helium314/HeliBoard/blob/3e91b4f1bf711a8bd3eb2d049c8b285fcc9cfa10/app/src/main/java/helium314/keyboard/keyboard/internal/GestureStrokeRecognitionParams.java)、[`touch-position-correction.xml`](https://github.com/Helium314/HeliBoard/blob/3e91b4f1bf711a8bd3eb2d049c8b285fcc9cfa10/app/src/main/res/values/touch-position-correction.xml)
- AOSP LatinIME [`config-auto-correction-thresholds.xml`](https://android.googlesource.com/platform/packages/inputmethods/LatinIME/+/127336e9f29d69607eab55982324b210279ae8c5/java/res/values/config-auto-correction-thresholds.xml)
- FlorisBoard `v0.5.2`：[`TextKeyboardLayout.kt`](https://github.com/florisboard/florisboard/blob/2e82060251897226c0739b9f52d1d051b02305fb/app/src/main/kotlin/dev/patrickgold/florisboard/ime/text/keyboard/TextKeyboardLayout.kt)、[`GlideTypingGesture.kt`](https://github.com/florisboard/florisboard/blob/2e82060251897226c0739b9f52d1d051b02305fb/app/src/main/kotlin/dev/patrickgold/florisboard/ime/text/gestures/GlideTypingGesture.kt)、[`StatisticalGlideTypingClassifier.kt`](https://github.com/florisboard/florisboard/blob/2e82060251897226c0739b9f52d1d051b02305fb/app/src/main/kotlin/dev/patrickgold/florisboard/ime/text/gestures/StatisticalGlideTypingClassifier.kt)
- 青簡 `4f5f7d2`：[`KeyboardView.kt`](https://github.com/qingjian-team/qingjian/blob/4f5f7d2069e5fcc07dc20636f56aa6397e4fabf2/apps/android/app/src/main/kotlin/app/qingjian/inputmethod/keyboard/KeyboardView.kt)、[`docs/design/voice-and-swipe.md`](https://github.com/qingjian-team/qingjian/blob/4f5f7d2069e5fcc07dc20636f56aa6397e4fabf2/docs/design/voice-and-swipe.md)（GPL-3.0，只讀）
- librime `bb56876`：[`corrector.cc`](https://github.com/rime/librime/blob/bb56876890ba60f446fa2640c244b1794b965eda/src/rime/dict/corrector.cc)、[`script_translator.cc`](https://github.com/rime/librime/blob/bb56876890ba60f446fa2640c244b1794b965eda/src/rime/gear/script_translator.cc)
- libime `53df6a2`：`src/libime/table/tablebaseddictionary.cpp`；fcitx5-chinese-addons `e8d930f`：`im/table/ime.h`
- Apple：[Cangjie - Traditional](https://support.apple.com/guide/chinese-input-method/cangjie-traditional-cimcjt11820/mac)
- 資料：[ESDB `Copyright` 與 README](https://github.com/en-wl/wordlist)（`1e5b7d3`）、[aosp-dictionaries](https://codeberg.org/Helium314/aosp-dictionaries)、[wordfreq](https://github.com/rspeer/wordfreq)、[FrequencyWords](https://github.com/hermitdave/FrequencyWords)、[Norvig ngrams](https://norvig.com/ngrams/)、[Google Books Ngram datasets](https://storage.googleapis.com/books/ngrams/books/datasetsv3.html)
