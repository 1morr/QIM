# 首選字基準的合成「打錯」模型：實體鍵盤錯誤率與種類、觸控偏移與 σ

日期：2026-11-12

標記：[原] 讀到原文（PDF 全文或逐字摘錄）；[文] 轉引（他人論文引述，沒讀到原文）；[推] 我由上述數字推算或推論。查不到的寫「未查到」。

相關既有筆記：`docs/research/spatial-correction-swipe.md` §2（Goodman、Ouyang、Fowler、σ 與鍵大小）、§3.3（合成錯誤模型的做法）。本篇不重複，只補數字。

## 問題清單

A. 實體鍵盤（十指）「含打錯碼」集合
1. 每鍵（每字元）錯誤率：未修正與「打的當下出錯（含之後修正掉的）」。
2. 四種錯誤（替換、漏打、多打、對調）的比例；替換裡鄰鍵占多少。

B. 觸控鍵盤「觸控模擬」集合
3. 點擊平均位置相對鍵中心的偏移（向下、向鍵盤中心），是否隨離中心的距離變大。
4. σx、σy 與共變異數或旋轉角；Azenkot & Zhai 2012 依姿勢的數字表。
5. 觸控每鍵錯誤率的參考值。

## 結論一覽

| 問題 | 結論 |
| --- | --- |
| 1 | 未修正 1.17%（Dhakal）。「打的當下出錯」沒有直接報告；Dhakal 的 TextTest 分類合計 3.12%，我推論它含已修正的（§A1）。快打者約 1.8%、慢打者約 5.8% |
| 2 | 四類比例沒有一份資料同時給齊。現代大樣本（Dhakal）替換 53%、漏打 26%、多打 21%，沒有對調類；Grudin 1983 的專業打字員（打字機）多打最多，且對調約 7%。替換中鄰鍵：新手 75%，專家約 47% |
| 3 | 向下：0.59 mm（Azenkot & Zhai 食指，[文]）。向中心：鍵盤上沒有數字；Henze 2011 在非鍵盤目標上有「螢幕角落往中心偏約 0.2–0.55 σ」的比值。Goodman 2002 本身沒有數字 |
| 4 | Fowler 的 1.97 / 1.88 mm 已證實 [原]，且自稱取自 Azenkot & Zhai；Azenkot & Zhai 的依姿勢表**未查到**（ACM 擋、無開放版本）。可得的只有食指 σx 1.81、σy 1.48 mm [文]。共變異數或旋轉角：**未查到**任何數字 |
| 5 | Fowler 12.8%（模擬）、Ouyang 8–9% [原，既有筆記]、Palin 2019 最終文字未修正 2.34% [原] |

---

## A1. 實體鍵盤的每鍵錯誤率

### 數字

來源：Dhakal, Feit, Kristensson, Oulasvirta, *Observations on Typing from 136 Million Keystrokes*, CHI 2018。[PDF](https://userinterfaces.aalto.fi/136Mkeystrokes/resources/chi-18-analysis.pdf)，DOI 10.1145/3173574.3174220。下表取自其 Table 2 與內文 [原]。

| 指標 | 全體 | 快打者（前 10%，約 > 78 WPM） | 慢打者（後 10%，約 < 26 WPM） | 備註 |
| --- | --- | --- | --- | --- |
| WPM | 51.56 | 89.56 | 20.91 | |
| 未修正錯誤率 Unc. Error (%) | 1.17 | 0.71 | 1.78 | Levenshtein 距離 ÷ 兩字串較長者的字元數；全體 SD 1.43 |
| 錯誤修正 Error Correct. (%) | 6.31 | 3.40 | 9.05 | 退格或 Delete 占所有按鍵的百分比；每句平均 2.29 次 |
| KSPC | 1.17 | 1.10 | 1.24 | 全部按鍵數 ÷ 最終字串字元數 |
| 替換 Substitutions (%) | 1.65 | 0.84 | 3.72 | TextTest；僅 783 人的分層樣本 |
| 漏打 Omissions (%) | 0.80 | 0.49 | 1.29 | 同上 |
| 多打 Insertions (%) | 0.67 | 0.50 | 0.80 | 同上 |

補充 [原]：90% 的參與者最終文字的錯誤低於 2.66%；受過打字訓練者未修正 1.02%，未受訓者 1.23%。參與者約 168,000 人，每人打 15 句。

### 「打的當下出錯」怎麼取

論文沒有直接報告「打的當下出錯率」。三條線索：

1. TextTest 的替換、漏打、多打三項合計。論文寫「We used Wobbrock's TextTest software to make a detailed analysis of the corrected and uncorrected errors. Given the input stream, ...」[原]，而合計 1.65 + 0.80 + 0.67 = **3.12%**，明顯大於未修正的 1.17%，所以 [推] 這三項統計的是輸入流裡的錯誤，含已修正的。論文沒有明講分母是字元還是按鍵，也沒有明講這三項含已修正，這是推論。快打者合計 1.83%，慢打者合計 5.81% [推]。
2. 退格上界：6.31% 的按鍵是退格或 Delete，KSPC 1.17，所以每字元約 0.074 次退格 [推]。一個錯誤常需多次退格（發現得晚、整詞刪掉），所以每字元出錯率會低於 7.4%，是上界。
3. 下界：未修正的 1.17%。

[推] 建議區間：下界 1.2%（只算沒修正的）、中心 3%（TextTest 合計）、上界約 7%（退格上界）。

### 行動裝置對照

Palin, Feit, Kim, Kristensson, *How do People Type on Mobile Devices? Observations from a Study with 37,000 Volunteers*, MobileHCI 2019。[PDF](https://userinterfaces.aalto.fi/typing37k/resources/Mobile_typing_study.pdf) [原]：未修正 2.34%（SD 2.08），KSPC 1.18，每句 1.89 次退格；未修正錯誤的組成為替換 55.6%、漏打 33.3%、多打 11.1%。這是觸控加自動修正環境，不是實體鍵盤，只當對照。作者說「Substitution was the most salient error type, which is in line with a study of text entry on physical keyboards [Dhakal]」。

### 其他

- Grudin 1983 的專業打字員錯誤率 0.4–1.9%，新手 3.2%，都是「不修正」的轉錄打字 [原，Table 6.1]。
- Dhakal 內文轉述，前人（Grudin 1983、Salthouse）在要求不修正的轉錄打字中，錯誤率約 1.0–3.2% [文]。

---

## A2. 錯誤種類比例與替換的鄰鍵比例

### Grudin 1983（打字機轉錄，不修正）[原]

Jonathan T. Grudin, *Error patterns in novice and skilled transcription typing*, in Cooper (ed.), *Cognitive Aspects of Skilled Typewriting*, Springer 1983, pp. 121–143。全文影本：[gwern.net](https://gwern.net/doc/design/typography/1983-grudin.pdf)。

Table 6.1：各打字員的錯誤種類占「全部錯誤」的百分比（Ins 多打、Omi 漏打、Sub 替換、Tra 對調、Oth 其他）[原]。

| 打字員 | WPM | 錯誤率 % | Ins | Omi | Sub | Tra | Oth |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 專家 1 | 90.4 | 1.1 | 28 | 21 | 27 | 10 | 13 |
| 專家 2 | 65.6 | 0.5 | 17 | 11 | 69 | 3 | 0 |
| 專家 3 | 76.1 | 1.9 | 75 | 5 | 6 | 4 | 10 |
| 專家 4 | 74.9 | 1.0 | 47 | 15 | 4 | 4 | 29 |
| 專家 5 | 61.3 | 0.4 | 53 | 19 | 9 | 11 | 9 |
| 專家 6 | 81.9 | 0.8 | 35 | 12 | 21 | 11 | 20 |
| 新手（20 人合計） | 約 20 | 3.2 | 9 | 4 | 75 | 4 | 11 |
| 專家平均（未加權）[推] | | | 42.5 | 13.8 | 22.7 | 7.2 | 13.5 |
| 專家平均（以錯誤率加權）[推] | | | 48.8 | 12.3 | 17.5 | 6.5 | 14.4 |

[原] 內文：「Substitution errors completely dominate the errors of students. These are substantially reduced in experts, where they are the second most common error. Insertion errors, the most frequent for experts, are not necessarily the most interesting: The overwhelming majority of a randomly selected subset examined on videotape are misstrokes, two keys struck by one finger.」意思是專家的多打大多是「一指同時碰到兩個鍵」，即多打的字元是鄰鍵。「其他」類中，新手有不少其實是一個字裡兩個替換。只有 6 位專家，個體差異極大（替換 4%–69%），平均沒有統計力。

替換的目標 [原]：

| 指標 | 新手 | 專家 | 備註 |
| --- | --- | --- | --- |
| 同一列緊鄰的鍵 | 59% | 31% | Lessenberry 的資料 43% |
| 同一手指同一欄（換列） | 8% | 16% | Lessenberry 15% |
| 兩者合計 [推] | 67% | 47% | |
| 「緊鄰鍵」（內文總結句） | 75% | 未給 | 「The majority are substitution errors, of which 75% are substitutions of immediately adjacent keys」，其餘的 51% 是左右手鏡像（homologous）。專家「fewer of them are adjacent keys」 |
| 鏡像鍵（homologous） | 16% | 4% | 隨機機率約 3% |
| 同一隻手內 | 72% | 78% | |

本分析用 3,300（新手）與 500（專家）個替換。

### Dhakal 2018（網路、168,000 人）[原]

TextTest 三類，沒有對調類。依 §A1 的表，三類合計 3.12%：

| 種類 | 百分比（占字元） | 占三類合計 [推] |
| --- | --- | --- |
| 替換 | 1.65 | 52.9% |
| 漏打 | 0.80 | 25.6% |
| 多打 | 0.67 | 21.5% |

論文把這與前人對照：「This result stands in contrast to prior studies in which insertion errors were commonly found to be the most frequent error type [12 (Grudin), 16, 33 (Salthouse)]」，並推測替換多是因為參與者打字不穩、手指位置的心像差 [原]。

### 對調

- Grudin 專家 3–11%（平均 6.5–7.2% [推]），新手 4% [原]。
- Dhakal 與 Palin 用的 TextTest 沒有對調類，對調會被算成兩個替換或一漏一多 [推]。
- Wobbrock 的 TrueKeys（IUI 2008）有對調：動作受限者 4.1%、非受限者 6.6%（詞層級，樣本小，且是 MSD 糾錯的情境）[原，摘錄自搜尋結果]。[PDF](https://faculty.washington.edu/wobbrock/pubs/iui-08.01.pdf)
- Salthouse 1986（*Psychological Bulletin* 99(3)）的錯誤種類比例：**未查到**（無開放全文）。Dhakal 把「多打與漏打比替換多」歸給 Grudin 與 Salthouse（其 [33] 是 Salthouse 1984 *JEP: General*）[文]，沒有數字。

---

## B3. 觸控偏移

### Goodman et al. 2002 [原]

[PDF](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/p194-goodman.pdf)。我下載並抽出全文，只有兩頁：「the average position for each key was not the keys center, but was slightly shifted down and towards the center of the keyboard. Second, variances in the x and y position are typically different. Third, some distributions are rotated」。**沒有任何數字**（沒有 mm、像素、比例、共變異數）。

### Azenkot & Zhai 2012（MobileHCI '12, pp. 251–260）

- [原，摘要] 32 人、三種姿勢（雙拇指、食指、單拇指）；「participants' touch precision was relatively high given typical key dimensions, but there were pronounced and consistent touch offsets」。
- 全文：**未讀到**（ACM 403，開放版本不存在，OpenAlex 標為 closed）。
- [文] 經 Yi et al. 2017（*Int. J. Human-Computer Studies* 106:44–62，[PDF](https://pi.cs.tsinghua.edu.cn/lab/papers/IJHCS_Xin%20YI.pdf)）引述：「when typing with an index finger on a 60 × 37.5 mm keyboard, the collective touch points tended to land 0.59 mm below the target key center, and the standard deviations were 1.81 mm and 1.48 mm in the x and y dimensions」。Yi 把它設為 `Offsetx = 0, Offsety = 0.59 mm, σx = 1.81 mm, σy = 1.48 mm`。
- [文] Yi 另引：個人的散布在寬度方向大於高度方向（與 Azenkot & Zhai 一致），合併資料則高度大於寬度；個人散布的寬、高分別是合併散布的 77%、63%，意思是各人精準但系統性偏移（尤其垂直）差異大。空白鍵的 x 偏移約為字母鍵的 3 倍，朝右。
- [文] Yin et al. 2013（CHI，[PDF](https://groups.csail.mit.edu/mug/pubs/Yin2013Making.pdf)）：不同姿勢在水平偏移上的差異，「最明顯的是鍵盤左半邊的鍵」。
- [文] Goel et al. ContextType（UW）：單拇指時「the user tended to bias towards the side of the thumb because of its limited reach」。

### Henze, Rukzio, Boll 2011（*100,000,000 Taps*, MobileHCI '11）[原，經 tavily 抽出的段落]

[PDF](http://nhenze.net/uploads/100000000-Taps-Analysis-and-Improvement-of-Touch-Performance-in-the-Large.pdf)。這是遊戲裡點圓形目標，**不是鍵盤**，單位是像素（機型 HTC Wildfire，像素大小論文段落沒給）。

| 螢幕區域 | x 偏移 | y 偏移 | SD(x) | SD(y) |
| --- | --- | --- | --- | --- |
| 左上 | +4.90 px | +6.64 px | 11.91 | 12.01 |
| 右下 | −2.55 px | −2.24 px | 11.63 | 12.57 |

正 x 為偏右、正 y 為偏下。左上區的點擊往右下偏、右下區往左上偏，也就是**朝螢幕中心偏**，且偏移隨螢幕位置而變（他們用 10×10 區塊畫整個螢幕的偏移向量圖）[原]。偏移相對 SD 的比值（不需要換算 mm）：左上 4.90/11.91 = 0.41、6.64/12.01 = 0.55；右下 2.55/11.63 = 0.22、2.24/12.57 = 0.18 [推]。

### 其他 [文]

- Yi et al. 2017 自己的資料是 2–4 cm 的**智慧手錶**鍵盤（20 人，右手），字母鍵 Offsetx +0.39 到 +0.64 mm（向右），Offsety −0.06 到 −0.16 mm（略偏**上**），並指出邊緣鍵的偏移會被「內側鍵是干擾項」推向外側（P、L、M 的 x 偏移較大）。這和 Goodman 的「向下、向中心」方向不同，說明方向因裝置、鍵大小與姿勢而異，不能當手機鍵盤的數字。
- Holz & Baudisch 2011（*Understanding Touch*, CHI 2011，[PDF](https://static.siplab.org/papers/chi2011-understanding_touch.pdf)）[原，摘錄]：接觸面積模型有約 4 mm 的系統性偏移，用「projected center」模型降到 1.6 mm。這是單點目標的指向任務，**不是鍵盤**，數字不能直接搬。
- 學生報告 arXiv:1504.02134（8 人，比較觸控點與觸控面積的偏移）有逐鍵偏移圖，沒抓到可引的數字，不採用。

### 偏移是否隨離中心距離變大

- 鍵盤：**未查到**逐鍵偏移量的數字表（Azenkot & Zhai 的圖表在擋住的全文裡）。[文] Yin 與 ContextType 都說逐鍵偏移不同、姿勢相關。
- 螢幕層級：Henze 2011 有（偏移依螢幕位置不同，角落朝中心偏）[原]。

---

## B4. σx、σy、共變異數與旋轉

### Fowler et al. 2015（CHI 2015，[PDF](https://research.google.com/pubs/archive/43272.pdf)）[原]

逐字：「The mean of the distribution for each tap was the center of the intended key, and the standard deviations were set to be 1.97 mm in the x direction and 1.88 mm in the y direction ... These values were based on [Azenkot & Zhai], which determined the 2D Gaussian distribution to be representative of actual human tapping, and described typical variances. Taps landing outside of the border of the intended key and inside another key were given the identity of the new key in the literal string output (a typing error). Taps outside the border of every key were discarded and resampled. We found that these settings result in a per-tap error rate of 12.8%, which translates to 38.4% of words in the test set containing at least one key error.」鍵大小為 Nexus 5：寬 6.16 mm、高 9.42 mm。已確認 σx = 1.97 mm、σy = 1.88 mm，均值在鍵中心（無偏移）、無共變異數。

注意：Fowler 的 1.97 / 1.88 與 Yi 轉引的 Azenkot & Zhai 食指值（1.81 / 1.48）不同。可能是別的姿勢（雙拇指？單拇指？）或全體平均，無法確認，因為 Azenkot & Zhai 的表**未查到**。

### Azenkot & Zhai 依姿勢的數字表

**未查到。** 只有上面的食指一組 [文]。若需要，要靠 ACM 全文（`10.1145/2371574.2371612`）人工取得。

### 共變異數、旋轉角

**未查到。** Goodman 2002 只說「noticeable covariance」；Azenkot & Zhai 轉引裡沒有；Yi 的橢圓圖是畫出來的，沒給角度。

### 其他可作 σ 底限的數字

- Bi, Li, Zhai（FFitts law, CHI 2013）：2.4 mm 圓目標上重複點擊的 2D SD 為 1.5 mm（σa）[文，經 Yi 與既有筆記 §2.3 引用]。
- Yi et al. 2017：個人散布約為合併散布的 77%（寬）與 63%（高）[文]，所以「合併多人」的 σ 比「單人」大。

---

## B5. 觸控的每鍵錯誤率

| 來源 | 數字 | 條件 | 標記 |
| --- | --- | --- | --- |
| Fowler et al. 2015 | 12.8% per-tap；38.4% 的詞至少一個錯鍵 | 模擬，σ 1.97 / 1.88 mm，鍵 6.16 × 9.42 mm | [原] |
| Ouyang et al. 2017 | 「per-letter error rates around 8-9%」 | Gboard，手機觸控打字 | [原，既有筆記 §2.2] |
| Palin et al. 2019 | 最終文字未修正 2.34%（美國子樣本 2.25%）；KSPC 1.18；每句 1.89 次退格 | 37,000 人、自己的手機和鍵盤（含自動修正、詞預測） | [原] |
| Yi et al. 2017 | 空白鍵錯誤率 30.5% → 3.8%（鍵盤 2.0 → 4.0 cm） | 智慧手錶尺寸，不是手機 | [原]，不採用 |
| Azenkot & Zhai 2012 | 各姿勢錯誤率：未查到數字（[文] 單拇指的錯誤率最低，來自 Lehmann & Kipp 2018 的敘述） | | |

---

## 對 QIM 參數的建議

這些是建議值，標明來源或「我們自己定」。

### A. 實體鍵盤「含打錯碼」集合

| 參數 | 建議 | 依據 |
| --- | --- | --- |
| 每鍵出錯機率 p | **0.03**（主設定）；另跑 0.018 與 0.058 當敏感度 | Dhakal TextTest 三類合計 3.12%（全體）、1.83%（快）、5.81%（慢）[原數字，合計與「含已修正」是推論]。未修正的 1.17% 當下界不用，因為我們關心打當下出錯。上界約 7% 是退格推算，不用 |
| 種類比例 替換 : 漏打 : 多打 : 對調 | **0.50 : 0.20 : 0.20 : 0.10** | Dhakal 的三類去掉對調後是 53 : 26 : 21 [原]；對調取 Grudin 專家約 7%、四捨五入到 10%，從三類按比例扣 [推]。這是我們自己定的折衷，沒有一份資料給齊四類 |
| 變體（可選，驗證結論不受比例左右） | **0.25 : 0.15 : 0.50 : 0.10**（多打為主） | Grudin 專業打字員（加權平均 替換 17.5 / 漏打 12.3 / 多打 48.8 / 對調 6.5，其他 14.4 另計）[推] |
| 替換落在鄰鍵的比例 | **0.65** 落在緊鄰鍵（QWERTY 幾何上與目標鍵相鄰，含左右與上下斜向）；其餘 0.35 隨機挑，且優先同一隻手 | Grudin：新手 75%、專家約 47%（31% 同列緊鄰 + 16% 同指換列）[原]；0.65 落在兩者之間 [推]。同手比例 72–78% [原]，其餘隨機的細節是我們自己定 |
| 多打的字元 | 目標鍵的鄰鍵，插在目標鍵前或後 | Grudin：專家多打的絕大多數是一指碰到兩鍵（錄影子樣本）[原]。同鍵重複打（連按兩次）的比例**未查到**，自己定 |
| 對調 | 相鄰兩個擊鍵對調 | 定義如此；Grudin 對調的細節（哪些雙字母易對調）未查到 |
| 速成的碼 | 每個「碼」（每次按鍵）獨立以 p 抽樣，錯誤落在該次擊鍵上 | Dhakal 的 p 是每字元，KSPC 只有 1.17，與每鍵差不多 |

### B. 觸控「觸控模擬」集合

與 Fowler 的差別是：Fowler 用 σ 1.97 / 1.88、中心無偏移、無旋轉；這裡加偏移、σx ≠ σy、旋轉，符合 Goodman 的三個觀察。

| 參數 | 建議 | 依據 |
| --- | --- | --- |
| 向下偏移 | **+0.6 mm**（y 方向向下），約 0.06 倍鍵高（以 Fowler 9.42 mm 鍵高算） | Azenkot & Zhai 食指 0.59 mm [文，Yi 轉引]。Goodman 只說 slightly shifted down。雙拇指與單拇指的值未查到 |
| 向鍵盤中心的水平偏移 | **自己定**：偏移量 = k × (鍵中心 x − 鍵盤中心 x) 取反號，k 取使最外側的鍵偏移約 0.3 σx（約 0.5–0.6 mm）、中心鍵 0 | 鍵盤上**未查到**數字。Henze 2011 在螢幕角落，偏移 / SD 的比值 0.18–0.55（取 0.3 為中間偏低）[原數字，搬到鍵盤是推論]。要讓偏移隨離中心距離線性增加是我們的假設，只有 Henze 的「依螢幕位置變」支持 |
| σx | **1.8 mm** | Azenkot & Zhai 食指 1.81 [文]；Fowler 1.97 [原] 當上界 |
| σy | **1.5 mm** | Azenkot & Zhai 食指 1.48 [文]；Fowler 1.88 [原] 當上界。σy < σx，符合 Goodman 的「variances differ」 |
| 旋轉 / 相關係數 | **自己定**：橢圓長軸旋轉 15°，右半邊的鍵順時針、左半邊逆時針（拇指弧形），或相關係數 ρ = ±0.25 | **未查到**任何數字。Goodman 只說有旋轉。此處只要「有旋轉」就達到目的（讓雜訊與空間模型的軸對齊假設不同），數值任意，要在文件裡標明是自訂 |
| 落在所有鍵外 | 丟棄重抽 | 同 Fowler [原] |
| 檢核 | 產生後算 per-tap 錯誤率，應落在約 8–13% | Ouyang 8–9% [原]、Fowler 12.8% [原]。若落在區間外，只調 σ，不調偏移 |
| σ 的單位 | 用毫米固定值，不隨鍵寬縮放 | FFitts law 說有手指的絕對不精確底限 [文]，既有筆記 §2.3；QIM 的鍵若比 6.16 mm 寬很多，σ 會顯得太小，要重新檢核錯誤率 |

### 需要我們自己定、查不到的

1. 實體鍵盤四類錯誤的最終比例（只有分開的資料；對調在大樣本裡沒有統計）。
2. 同鍵連按（雙擊）當多打的比例。
3. 觸控向鍵盤中心的水平偏移量與其隨距離的變化。
4. 旋轉角或共變異數。
5. Azenkot & Zhai 依姿勢的 σ 與偏移表（要取得 ACM 全文）。
6. Salthouse 1986 的錯誤種類比例。

## 參考文獻

1. Dhakal, V., Feit, A. M., Kristensson, P. O., Oulasvirta, A. *Observations on Typing from 136 Million Keystrokes*. CHI 2018. DOI 10.1145/3173574.3174220. https://userinterfaces.aalto.fi/136Mkeystrokes/resources/chi-18-analysis.pdf [原]
2. Grudin, J. T. *Error patterns in novice and skilled transcription typing*. In Cooper, W. E. (ed.), *Cognitive Aspects of Skilled Typewriting*, Springer, 1983, pp. 121–143. https://gwern.net/doc/design/typography/1983-grudin.pdf [原]
3. Salthouse, T. A. *Perceptual, cognitive, and motoric aspects of transcription typing*. Psychological Bulletin 99(3), 1986, 303–319. 未取得全文 [文，經 Dhakal]
4. Palin, K., Feit, A. M., Kim, S., Kristensson, P. O., Oulasvirta, A. *How do People Type on Mobile Devices? Observations from a Study with 37,000 Volunteers*. MobileHCI 2019. https://userinterfaces.aalto.fi/typing37k/resources/Mobile_typing_study.pdf [原]
5. Goodman, J., Venolia, G., Steury, K., Parker, C. *Language Modeling for Soft Keyboards*. IUI 2002, pp. 194–195. https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/p194-goodman.pdf [原，無數字]
6. Fowler, A., Partridge, K., Chelba, C., Bi, X., Ouyang, T., Zhai, S. *Effects of Language Modeling and its Personalization on Touchscreen Typing Performance*. CHI 2015. https://research.google.com/pubs/archive/43272.pdf [原]
7. Azenkot, S., Zhai, S. *Touch behavior with different postures on soft smartphone keyboards*. MobileHCI 2012, pp. 251–260. DOI 10.1145/2371574.2371612. 只讀到摘要 [原]，數字為 [文]（經 Yi et al. 2017）
8. Yi, X., Yu, C., Shi, W., Bi, X., Shi, Y. *Is it too small? Investigating the performances and preferences of users when typing on tiny QWERTY keyboards*. Int. J. Human-Computer Studies 106, 2017, 44–62. https://pi.cs.tsinghua.edu.cn/lab/papers/IJHCS_Xin%20YI.pdf [原，智慧手錶；引述 Azenkot & Zhai 為 [文]]
9. Henze, N., Rukzio, E., Boll, S. *100,000,000 Taps: Analysis and Improvement of Touch Performance in the Large*. MobileHCI 2011, pp. 133–142. http://nhenze.net/uploads/100000000-Taps-Analysis-and-Improvement-of-Touch-Performance-in-the-Large.pdf [原，非鍵盤]
10. Holz, C., Baudisch, P. *Understanding Touch*. CHI 2011. https://static.siplab.org/papers/chi2011-understanding_touch.pdf [原，非鍵盤]
11. Yin, Y., Ouyang, T. Y., Partridge, K., Zhai, S. *Making Touchscreen Keyboards Adaptive to Keys, Hand Postures, and Individuals: A Hierarchical Spatial Backoff Model Approach*. CHI 2013. https://groups.csail.mit.edu/mug/pubs/Yin2013Making.pdf [文]
12. Ouyang, T., Rybach, D., Beaufays, F., Riley, M. *Mobile Keyboard Input Decoding with Finite-State Transducers*. arXiv:1704.03987. 見 `spatial-correction-swipe.md` §2.2 [原]
13. Kane, S. K., Wobbrock, J. O., Harniss, M., Johnson, K. L. *TrueKeys: Identifying and Correcting Typing Errors for People with Motor Impairments*. IUI 2008. https://faculty.washington.edu/wobbrock/pubs/iui-08.01.pdf [原，摘錄]
14. Bi, X., Li, Y., Zhai, S. *FFitts law: modeling finger touch with Fitts' law*. CHI 2013. https://research.google.com/pubs/archive/41645.pdf [原，摘錄；SDxy 1.5 mm]
