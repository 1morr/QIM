# 閉源輸入法的候選與排序：Gboard、Apple、Microsoft、搜狗

調查日期：2026-10-30。目的：了解成熟閉源輸入法怎麼做「輸入碼 → 分詞 → 候選生成 → 排序 → 上屏」，並設計黑箱測試，之後與 QIM 比較。

查證規則：每條宣稱附來源編號（見〈來源〉）。查不到的寫「未查證」。標「推論」的是我從事實推出的，不是來源原文。

## 摘要

1. **Gboard 的公開資料集中在「英文等拼音文字的觸控鍵盤」**，沒有找到 Gboard 中文 / 粵語（速成、倉頡）引擎的公開論文或技術部落格。粵語、速成的內部做法：未查證。可以拿來類比的是 Ouyang et al. 2017 的 FST decoder 框架：lexicon FST、n-gram LM FST、spatial model 三者組合，再用 beam search 解碼 [S1][S2]。
2. **端上、小模型、嚴格延遲**是 Gboard 的設計前提：按鍵要在約 20 ms 內有視覺回應，LM 不超過 5–10 MB（約數十萬詞以內）[S1]。神經 NWP 模型只有 1.4M 參數、量化後 1.4 MB [S3]；2023 年的 LM 為 2.4M / 4.4M / 6.4M 參數（詞彙 10K / 20K / 30K）[S6]。
3. **個人化的公開機制是「動態模型 + on-the-fly lattice rescoring」**：把使用者打過的 n-gram、聯絡人等放進動態模型，解碼時再重打分，不需要改主 decoder 圖 [S1]。這是目前最接近「記住 email」的有據機制。
4. **聯邦學習（FL）學的是全體使用者的新詞（OOV）與 LM，不是你個人的 email**。OOV 論文用 character-level LSTM 取樣出高頻 OOV 詞，目的是擴充全域詞彙 [S4]。所有 Gboard NWP 神經 LM 已用 FL + DP 訓練，ε 範圍 0.994–13.69 [S7][S8]。
5. **`bc` → `bc12345@example.edu`**：有三個可能機制，都沒有官方文件直接證明是它。個人 n-gram / 使用者詞（有據於 [S1]）、Android 11+ 的 IME inline autofill（官方文件 [S12]，但要欄位被標成可 autofill）、聯絡人 / 帳號資料（未查證）。見第 3 節的黑箱測法。
6. **macOS 速成 / 倉頡「打英文單字直接出英文候選」是官方文件記載的功能**（輸入英文字、在候選窗選那個字，如 `ipad`）[S13]，不是巧合。macOS 還有官方的「Adjust word frequency」設定，把常用候選排前，選過的字下次同碼時上移 [S14]。
7. **Apple 在 iOS 17 公開說「Autocorrect 用 transformer language model，端上」** [S9]，但沒有論文、模型大小、延遲。iOS 速成的內部：未查證。
8. **REA 只讀看了 macOS 的 TCIM.app 與相關 framework 的檔案結構**：`TCIM.app` 只是 740 KB 的殼，引擎在 `CoreChineseEngine.framework`，詞庫 / LM 更新資產以 `Mecabra` 命名並按需下載（見第 5 節）。
9. 值得 QIM 考慮的功能（第 6 節）：使用者詞 + 個人 n-gram、候選頻率自適應（可關閉、可重置）、association（後續建議）、中文模式下的英文候選、容錯、保守的 post-correction、可見可刪的學習資料。

## 1. 產品分節

### 1.1 Google Gboard

#### 架構（輸入碼 → 解碼 → 候選）

| 項目 | 事實 | 來源 |
|---|---|---|
| 解碼框架 | 以 FST 表示。拿語音辨識的技術做 decoder，支援 literal decoding、autocorrection、word completion、next-word prediction | [S1] |
| 輸入 | 觸控點時間序列 x（座標），手勢輸入約每 100 ms 取樣一點 | [S1] |
| 組成 | key-to-word lexicon FST（含可選空格、可選撇號）+ n-gram LM FST（「低階 n-gram、詞彙如 64K 詞」）+ spatial model | [S1] |
| spatial model（2017 論文） | 對每個 tap 給出各鍵的機率分佈；處理 substitution（鄰鍵誤觸、拼錯）、deletion（漏打，解碼時可略過一個 arc，帶權重）、insertion（多餘的 tap，消耗輸入但不前進狀態） | [S1] |
| spatial model（2017 部落格） | 以神經網路（單一 LSTM、CTC 準則）取代早期的高斯模型與規則模型 | [S2] |
| 搜尋 | beam search，spatial 分數與 LM 分數合併 | [S2] |
| next word | 從當前最佳部分句子找最高階 n-gram state，取 n-best 輸出標籤（含 backoff 路徑）。LM FST 的 arc 依 label 排序，所以必須走完 state 的所有 arc，高出度狀態（如 unigram state）要快取 | [S1] |
| post-correction | 可以事後修正已輸入的字，風險是誤改；建議保守：只在最近一個詞的時間窗內、decoder 信心很高時才改 | [S1] |

- **模型大小與延遲**：按鍵要在約 20 ms 內有視覺回饋；全部在端上處理；RAM / CPU 受嚴格控制以免被系統回收、省電；LM 不超過 5–10 MB [S1]。NWP 神經模型在 [S3] 寫「語言模型通常限制在數十 MB、詞彙數十萬」。
- **2017 部落格的成果**：模型比初版「快 6 倍、小 10 倍」，解碼延遲降 50%，離線資料集上 bad autocorrect 少約 15%、錯誤解碼的手勢少 10%，使用者手動修正的詞比例降 10% 以上 [S2]。
- **訓練資料**：神經空間模型的標籤是半監督的使用者訊號：被撤銷的 autocorrect 當負例、點選建議當正例 [S2]。
- **中文 / 粵語**：Gboard 的設定頁列有粵語、倉頡、速成、筆劃（第三方整理 [S18]，非 Google 官方）。內部解碼、LM、詞庫來源：**未查證**。Google 的 Deep Internationalization 報告談 900+ 語言變體與 70+ 書寫系統、鍵盤佈局設計 [S11]，我取得的內容沒有提到粵語或中文輸入法引擎，所以不能拿它說明速成。

#### 排序與學習

| 層 | 做法 | 來源 |
|---|---|---|
| 字頻 / n-gram | 低階 n-gram，64K 詞等級 | [S1] |
| 神經 LM（NWP） | CIFG（coupled input-forget gate）LSTM，1 層 670 單元，embedding 維度 96，詞彙 10,000，1.4M 參數（超過三分之二在 embedding），量化後 1.4 MB，input / output embedding 共用 | [S3] |
| 神經 LM（2023） | 同架構；詞彙 10K / 20K / 30K 時為 2.4M / 4.4M / 6.4M 參數。NWP 詞彙約 10–20K；OTF（on-the-fly rescoring）用約 30K 詞；Smart Compose（SC）重用 NWP LM、不另訓練 | [S6] |
| Smart Compose | 比 NWP 更長的 inline 建議，信心夠高時可以逐字元觸發 | [S6] |
| 個人化 | 動態模型累積使用者打過的 n-gram、聯絡人等，以 on-the-fly lattice rescoring 併入解碼，不更新主圖；論文說個人化可補償端上模型太小：「一個人很少需要超過 100K 詞，但需要的是對的 100K 詞」 | [S1] |
| 使用者詞 | 使用者可以對紅底線詞「Add to dictionary」；長按建議可拖到垃圾桶移除；個人詞典可匯入 / 匯出（舊版有 sync） | [S15][S16] |
| 學習資料位置 | 「Personalize for you」會把音訊與逐字稿存在裝置上，可在 Privacy 中「Delete learned words & data」清除 | [S17] |
| 聯邦學習 | 只在充電、連 Wi-Fi、不使用時進行；送出的是學到的更新，不是輸入文字；預設開啟 | [S17] |
| OOV 學詞 | character-level CIFG LSTM（含 peephole 與 projection layer）經 FL 訓練，對模型做 Monte Carlo 取樣得到高頻 OOV 詞。Reddit 模擬：top 10^5 unique 詞 precision 90.56%、recall 81.22%；真機 FL：字元層級 top-3 準確率 55.8%、cross entropy 2.35；取樣結果能反映當時新聞熱詞 | [S4] |
| DP | 自 2022 年西班牙語 LM 起為第一個直接以使用者資料訓練並宣告 DP 的 production 神經網路；現在所有 NWP 神經 LM 都 FL + DP，30+ 個模型、7+ 語言、15+ 國家，δ = 10^-10、ε 在 0.994 到 13.69 | [S7][S8] |
| DP 訓練細節 | DP-FTRL、SecAgg、預訓練用 multilingual C4；強隱私模型每輪 12,000+ 裝置（一般 6,500+）；每個 client 一定時間內最多參與兩次 | [S7][S8] |

- **查詢建議 FL**：Gboard 搜尋建議模型也用 FL 全球規模訓練，報告線上品質提升 [S5]（摘要層級，沒有模型細節）。

#### 上下文

- NWP 是「前一個詞確認後才觸發」[S6]，即用已提交文字當上下文。Gboard 是否讀取輸入框內既有文字（IME 能用 `InputConnection` 讀取游標前文字）：公開論文沒有逐項說明，**未查證**；用黑箱測試 T6 驗證。

#### 隱私

- FL 不送文字，只送更新 [S17]；Gboard Help 於 iOS 版寫「不會送出你輸入的內容（搜尋除外），已儲存的字不會送出」[S19]。

#### 已知限制

- 官方承認「端上輸入法遠未解決，仍會給出不直覺或低用處的建議」[S2]。
- 候選是否隨使用頻率前移、中文引擎是否有頻率自適應：未查證（和使用者觀察一致，見第 2 節）。

### 1.2 Apple（iOS 與 macOS）

#### iOS

| 項目 | 事實 | 來源 |
|---|---|---|
| Autocorrect | iOS 17 起「用新的 transformer language model，端上 ML 語言模型，用於字詞預測」；「iPhone 12 以後」只見於 Apple 社群頁引用的功能描述（二手，未在 Apple 官方頁核對） | [S9][S20] |
| 預測 | 輸入時顯示字、emoji、資訊建議，例如打「I'm at」會給目前位置 | [S10] |
| 學習 | 拒絕同一建議幾次後會停止建議 | [S20]（Apple 社群頁引用支援文字，二手；支援頁原文未逐字核對） |
| 模型大小、延遲、訓練資料 | **未查證**。Apple Machine Learning Research 上找到的是 Apple Intelligence 基礎模型（約 3B 參數端上模型）[S21]，與鍵盤 autocorrect 的 transformer 不是同一個，不可混用 |
| 中文速成 | 後續建議詞、連選多字、不能連打、數字在二級選單、符號後跳回字母頁：這些是使用者觀察，我沒有找到 Apple 官方技術文件。內部：**未查證**。2026-11-04 黑箱實測（iOS 27.0.1）證實不做整串解碼（固定 2 碼一段）與打標點後跳回字根頁，見 `blackbox-baseline.md` |

#### macOS（官方使用者文件）

| 項目 | 事實 | 來源 |
|---|---|---|
| 輸入源 | Traditional：Pinyin、Shuangpin、Stroke、Zhuyin、Zhuyin Eten、Cangjie、Sucheng；Cantonese：Cangjie、Sucheng、Stroke、Phonetic | [S13][S14] |
| Sucheng | 是 Cangjie 的簡化，取 Cangjie 碼的第一碼與最後一碼；碼更短但候選更多 | [S22] |
| 倉頡萬用鍵 | Z 鍵代替不確定的碼，可重複使用 | [S13] |
| 英文候選 | 「Enter an English word, then select the word in the candidate window.」例：`ipad`；專有名詞自動大寫 | [S13] |
| Show predictive completions | 補全以第二或第三個候選出現 | [S14] |
| Show candidates while typing | 速成 / 倉頡可設定自動顯示候選窗或按空白才開 | [S14] |
| Adjust word frequency | 「把最常用的候選排在前面」：選過的候選下次輸入同碼時前移；適用於 Sucheng 等多數輸入源 | [S14] |
| 其他 | Undo the last substitution（Shift-Delete）、Show structural candidates（Shift-Space）、Find an input code、Facemarks & Punctuation、Specialized vocabularies、使用者詞典（文件另有一頁） | [S14][S23] |
| 候選窗 | 顯示可能的字，某些語言含 emoji | [S14] |

macOS 上我沒有找到連打（整句）的官方描述；使用者觀察「不能連打」與速成的設計一致（Sucheng 一碼對應一字，沒有分詞需求）[S22]。2026-11-04 黑箱實測（macOS 27.0.1）：第 3 碼起整串改當英文字母，見 `blackbox-baseline.md`。

### 1.3 Microsoft（Windows）

- **Microsoft Pinyin**：設定中有「Lexicon and self-learning」；Windows 11 的「Try text suggestions from Bing」（Windows 10 稱 Cloud service）會在候選窗以雲圖示顯示由 Bing 取得的建議；有 Fuzzy Pinyin [S24]。自學習、詞庫細節：官方頁未描述，**未查證**。
- **Microsoft Traditional Chinese IME**（Bopomofo、Changjie、Quick）：有 Associate phrase window，顯示與已打內容相關的「關聯候選」，以 Shift + 數字選取 [S25]。候選排序、學習、雲：官方頁未描述，**未查證**。
- 內部架構、模型：未查證。

### 1.4 搜狗輸入法

- 找到的第一手資料很少：Microsoft Store 頁列出功能含 AI 搜索、AI 幫寫、AI 快查、語音、手寫、五筆、細胞詞庫 [S26]；騰訊宣布搜狗輸入法 20.0 版「全面 AI 化」，語音、打字、翻譯三大場景做了模型級升級（二手新聞轉述）[S27]。
- 搜狗的候選生成、排序、雲端 / 端上分工、個人化：**未查證**。Wikipedia 提到 2009 年有雲端輸入的 bookmarklet 版本 [S28]，只能當線索。
- 學界相關：Chen et al. PACLIC 2015 討論拼音輸入法的標準管線「拼音切分 → 候選詞擷取 → 候選句生成」，並把 NN LM 轉成 back-off n-gram LM 以供 decoder 使用 [S29]。這不是搜狗的論文，只能當通用架構參考。

## 2. 解釋使用者觀察

| 觀察 | 解釋（依據） | 把握 |
|---|---|---|
| Gboard 連打準確、分詞合理 | FST decoder 把 lexicon（含可選空格）與 n-gram LM 合成，同時對整串輸入做搜尋；分詞是搜尋結果的一部分，不是先切再查 [S1][S2]。中文版本是否同構：未查證 | 架構有據，中文未查證 |
| 打錯一兩碼首選仍對 | spatial / 編輯模型明確建模 substitution、deletion、insertion，各有機率權重 [S1]；LM 提供上下文把正確詞推上來 | 有據（英文）；速成未查證 |
| 支援粵語口語 | 與 LM / 詞庫語料有關。Gboard 粵語的語料與詞表來源：未查證。推論：口語詞要靠粵語專屬語料，不是書面中文詞庫 | 未查證 |
| 記住常打內容（`bc` → email） | 見第 3 節 | 見第 3 節 |
| 長按出英文 | 鍵盤佈局設定 / 長按子鍵屬於 layout 設計（報告談到動態佈局與每使用者佈局個人化 [S11]）；速成鍵上長按顯示字母是 layout 的一部分。具體實作：未查證 | 未查證 |
| 候選不隨使用頻率前移 | 與 macOS 對比：macOS 有明確的「Adjust word frequency」開關 [S14]；Gboard 官方文件只講「學詞」與個人化 [S17]。推論：Gboard 中文引擎可能把個人頻率放在 rescoring 的弱權重或根本沒做，或者候選順序由全域 LM 與上下文主導。需 T3 驗證 | 推論 |
| iOS 速成後續建議好、能連選多字 | 「association」功能；Microsoft 有同類 Associate phrase window [S25]，Apple 的詞頻 / 語料細節未查證。使用者觀察到的是：選完字後給下一字 / 詞建議 | 現象明確，機制未查證 |
| iOS 速成不能連打 | 速成一碼（首碼 + 末碼）對一個字 [S22]，沒有「連續輸入多字的碼串」要分詞；要連選是靠 association 而不是整句解碼 | 推論 |
| iOS 數字在二級選單、符號後跳回字母頁 | UI 佈局與狀態機設計，沒有技術文件。屬於 UX 決定，不是排序問題 | 未查證 |
| macOS 中文狀態打英文單字直接出英文候選 | 官方文件記載：倉頡 / 速成輸入英文字後，在候選窗選取 [S13] | 有據 |

## 3. 「記住 email」的已知機制與假設

### 已知（有出處）

1. **個人動態模型**：Gboard 的 FST decoder 設計明確支援「累積使用者打過的 n-gram、聯絡人等」的動態模型，用 on-the-fly lattice rescoring 併入，並用在 next word 預測與 word completion 的動態詞彙 [S1]。這是 2017 年論文層級的機制，不保證現行版本完全一樣。
2. **使用者詞典與已學詞**：Gboard 有「Add to dictionary」、「Delete learned words & data」，說明端上存有已學詞 [S15][S17]。Google 社群回覆指出學到的詞和個人詞典是分開的（個人詞典可為空但仍有學詞）[S30]，二手，僅作線索。
3. **Android IME inline autofill**（Android 11+）：IME 可在建議列顯示 autofill service 提供的建議，IME 在使用者選取前看不到內容；提供者視欄位與設定可為平台或 autofill service（如密碼管理員）[S12]。若 `bc12345@example.edu` 是由 Google Autofill 在 email 類欄位提供的，它會以 autofill 晶片出現，而不是 Gboard LM 的候選。
4. **FL OOV 學詞不是個人化機制**：它從許多裝置取樣高頻 OOV 詞，目標是擴充全域詞彙，同時有 DP 以避免記住單一使用者的獨特資料 [S4][S7]。所以「記住你的 email」不能歸因於它。

### 未查證，需黑箱測試

H1 個人 n-gram / 學詞：打過幾次就會學。
H2 Autofill：只在欄位有 autofill hint（email、username）時出現。
H3 聯絡人 / 帳號資料：來自 Google 帳號或聯絡人（`@example.edu` 可能是使用者的帳號 email）。
H4 剪貼簿：最近複製過這個 email（Gboard 有剪貼簿建議，具體條件未查證）。

分辨方法見測試表 T5a–T5f。

## 4. 黑箱行為測試表

通用準備：每項測試前記錄 App、裝置、系統與輸入法版本、是否開飛航模式；用同一個純文字 App（筆記）與一個有 email 欄位的 App；每項至少重複 3 次；螢幕錄影（240 fps 更佳）以量測延遲；先清除學習資料（Gboard：Privacy → Delete learned words & data；macOS：重設詞頻前先匯出使用者詞典備份）。

| ID | 輸入什麼 | 看什麼 | 能推論什麼 | 對象 |
|---|---|---|---|---|
| T1 連打 | 輸入常見多字詞的速成碼串，不按空白（如連續 8–10 碼的常用句），再加上空格 / 標點 | 候選是整句、分段、還是只顯示最後一字；首選是否為正確句子；是否可回頭修改前面分詞 | 是否做整串解碼（FST / lattice）而非逐字 | Gboard、iOS、macOS、QIM |
| T2a 容錯：替換 | 把正確碼中某一碼換成鍵盤上相鄰鍵 | 首選是否仍為原字 / 詞；錯在第 1 碼、中間、末碼分別測 | 有沒有編輯距離或鄰鍵模型；位置是否影響 | Gboard、iOS、QIM |
| T2b 容錯：漏碼 / 多碼 / 對調 | 漏打一碼、多打一碼、兩碼對調 | 同上；候選出現速度 | deletion / insertion / transposition 的處理 [S1] | 同上 |
| T3 頻率自適應 | 選一個原本排第 5 的候選 10 次，每次換行再打同碼 | 該候選的位置是否前移；前移是立刻、還是要多次；關閉預測後是否仍前移 | 有沒有使用頻率排序（macOS 有開關 [S14]）；學習速度與門檻 | Gboard、iOS、macOS、QIM |
| T3b 衰退 | T3 之後，改選另一個候選 10 次，再回頭看 | 原先前移的候選是否掉回去 | 是否有時間衰減或最近性權重 | 同上 |
| T4 學新詞 | 用候選以外的方式打一個罕見字串（自創詞）3、5、10 次，之後打其首碼 | 是否出現為候選；需要幾次 | 學詞門檻；是否學進使用者詞庫 | Gboard、iOS、macOS |
| T5a email：基本 | 在筆記 App 打 3 次完整 email，之後只打 `bc` | 是否出現完整 email；排第幾 | H1：個人學詞 | Gboard |
| T5b email：欄位類型 | 在 email 欄位、密碼欄位、一般文字欄位分別打 `bc` | email 候選是否只在某類欄位出現；外觀是否為 autofill 晶片 | H2：autofill 與 LM 候選的區別 | Gboard |
| T5c email：清除學習資料 | 在 T5a 成立後，Delete learned words & data，再打 `bc` | 是否消失 | 若消失，來源是端上學習資料，不是 autofill / 帳號 | Gboard |
| T5d email：關掉 Autofill | 系統設定關閉 Autofill service 後重做 T5b | 是否仍出現 | 排除 H2 | Gboard |
| T5e email：飛航模式 | 飛航模式下重做 T5a | 是否仍出現 | 排除雲端 | Gboard、iOS |
| T5f email：剪貼簿 / 換 App | 先複製另一個陌生 email，再打 `bc`；換 App 重打 | 是否出現剛複製的；跨 App 是否一致 | H4 剪貼簿；學習是否跨 App | Gboard |
| T6 上下文 | 先輸入兩種不同前文（如「我想食」與「我想睇」），再打同一個速成碼 | 首選是否隨前文改變；刪掉前文後是否復原；從別處貼上的前文是否也有效 | 是否讀取輸入框前文；使用 n-gram 還是神經 LM | Gboard、iOS、QIM |
| T7 中文模式英文 | 打 `ipad`、`iphone`、`github`、`bc` | 是否直接給英文候選；是否與中文碼衝突時優先；大小寫處理 | 英文詞典來源；與中文候選混排方式 [S13] | Gboard、iOS、macOS、QIM |
| T8 association | 選一個字後不輸入，看建議；連選 3–4 次 | 建議是詞還是字；是否因前一字改變；是否可無限接續 | 後續建議機制（bigram / 詞表 / NN）[S25] | iOS、Gboard、QIM |
| T9 延遲 | 錄影下連打 20 碼，量按鍵到候選更新的幀數 | 中位數與 P95 延遲；長碼串時是否變慢 | 是否達約 20 ms 等級 [S1]；是否有雲端往返 | Gboard、iOS、macOS、QIM |
| T10 離線 | 全部測試在飛航模式重跑一輪 | 結果是否與有網路時不同（特別是英文、粵語口語） | 雲端候選的存在 | Gboard、iOS、macOS、Windows |
| T11 undo | 觸發 autocorrect 後按退格 | 是否還原為原輸入；同一詞被拒絕幾次後不再建議 [S20] | post-correction 與負反饋機制 [S2] | Gboard、iOS |
| T12 post-correction | 打完一詞、再打下一詞，看前一詞是否被改 | 是否發生，信心條件 | 是否有 [S1] 所述的事後修正 | Gboard、iOS |
| T13 長按 | 長按各鍵 | 子鍵內容（英文、數字、符號） | layout 設計 [S11] | Gboard、iOS |
| T14 持久化 | 重啟 App、重啟裝置、換 App 後重做 T3 / T4 | 學習結果是否保留；是否跨 App | 持久化範圍 | 全部 |
| T15 隱私欄位 | 在密碼欄位反覆輸入唯一字串，離開後在一般欄位打其首碼 | 是否被學進去 | 是否排除敏感欄位 | Gboard、iOS、QIM |
| T16 平手穩定性 | 同一碼、無學習資料下重複輸入 10 次 | 順序是否固定 | 排序是否確定性 | 全部 |
| T17 罕用字 / 簡繁 | 打已知罕用字的碼 | 是否出現、排序位置 | 字集覆蓋與預設排序（字頻 vs 碼表順序） | 全部 |
| T18 標點與數字 | 候選中途輸入數字、符號 | 是否上屏候選、是否跳頁（iOS 觀察：符號後回字母頁） | UX 狀態機 | iOS、QIM |

QIM 的比較方式：每個測試在 QIM 上重跑並用同一格式記錄，差異分三類：缺功能、行為不同但合理、行為更差。

## 5. REA / 只讀結構調查

範圍：只用 `ls` / `find` / `du` / `otool -L` 看檔名、大小、連結的 library，**沒有開啟或複製任何碼表、詞庫、模型內容**，也沒有把任何資源內容寫進 repo。macOS 版本資訊：Darwin 27.0.0。

| 路徑 | 看到什麼 | 推論 |
|---|---|---|
| `/System/Library/Input Methods/TCIM.app`（總 740 KB） | `Contents/MacOS/TCIM` 只連結 Foundation、AppKit、libobjc、libSystem；有一個 `PlugIns/TCIM_Extension.appex`；資源以各語言 `.lproj` 與 `cangjie.tiff` 為主 | 它是輸入法殼（UI、IMK 進入點），沒有引擎和詞庫；引擎在別處 |
| `/System/Library/PrivateFrameworks/CoreChineseEngine.framework`（資源約 1.2 MB） | 資源只有小型 plist 與 sqlite：`PinyinSyllables.plist`、`PinyinToZhuyin.plist`、`CangjieToASCII.plist`、標點對照 plist、`ContextualVariantMapping.plist`、`CharacterAccessibilityData.sqlite`，以及每個輸入法一個 preferences 模組（`TCIMSuchengPreferencesModule`、`TCIMCangjiePreferencesModule`、`CIMExpertDictionariesPreferencesModule` 等）；`SharedSupport` 只有 `CIMFindInputCodeTool.app` | 速成 / 倉頡的字碼表本身不在這個 framework 的資源裡（或被編譯進二進位）；碼表格式與位置：未驗證。preferences 模組列表對應第 1.2 節文件列出的設定項 |
| `/System/Library/PrivateFrameworks/Lexicon.framework`（72 KB） | `al-lexicon.idx` / `al-lexicon.dat`、`confusables.dat`、`collator.dat`、`vulgar_word_usage_database_schema.sql` | 通用詞彙 / 排序 / 易混淆字處理；`vulgar_word_usage_database_schema.sql` 的檔名暗示有針對不當詞的使用記錄資料庫 schema，內容未讀，僅為推論 |
| `/System/Library/PrivateFrameworks/` 下的相關 framework | `TextInput`、`TextInputCJK`、`TextInputCore`、`TextInputSwift`、`LanguageModeling`、`InputContext`、`ProactiveInputPredictions`（20 KB）、`KeyboardServices`、`InputAnalytics` | 輸入、LM、上下文、預測是分層的獨立 framework；`LanguageModeling` 與 `InputContext` 的存在與「讀上下文、用 LM 排序」的架構一致（推論，未看內容） |
| `/System/Library/AssetsV2/com_apple_MobileAsset_MecabraDictionaryRapidUpdates/*.asset/AssetData` | `asset_zh-Hant_sys.dic`（169 KB）、`asset_zh-Hant_mix.dic`（8.8 KB）、`asset_zh-Hant_ucm.dat`（518 B）、`Chinese.lm/` 目錄含 `params.dat`（722 B，Apple binary plist）、`cdistr.dat`（1 KB）、`blocklist.dat`（296 KB）、`Info.plist` | 詞庫以「RapidUpdates」資產按需下載更新，所以 Apple 能不升級系統就更新詞表（新詞、熱詞）；檔名 `Chinese.lm`、`cdistr`（疑為分佈 / 類別分佈）、`blocklist`（封鎖不當候選）暗示有獨立的 LM 與封鎖表。**只憑檔名與大小，內容與格式未驗證**；`Mecabra` 是 Apple 內部引擎名（檔名可見） |
| `/System/Library/AssetsV2/com_apple_MobileAsset_UAF_LinguisticData` | 9 個 asset 目錄，只看了目錄結構 | 其他語言資料的按需資產；未進一步查看 |

對 QIM 的意義：（1）Apple 把「引擎 / 殼 / 詞庫更新 / LM」拆開，且詞庫可線上更新，是 QIM 設計資料更新通道時可參考的做法；（2）有獨立的封鎖表；（3）看不出速成碼表格式，這是正常的，也不該去看。

## 6. 成熟輸入法值得加入 QIM 的功能

「支持程度」：有據 = 來源文字直接描述；推論 = 由來源或觀察推出。

| 功能 | 理由 | 出處 / 支持程度 |
|---|---|---|
| 使用者詞 + 個人 n-gram（動態模型，解碼時 rescoring，不改主詞庫） | 小模型靠個人化補足；也是「記住常打內容」最有據的機制 | [S1]，有據 |
| 候選頻率自適應，可開關、可重置 | macOS 有官方開關；Gboard 沒有但使用者會期待；QIM 若做要有衰減策略 | [S14]，有據；Gboard 缺失是觀察 |
| 學習資料可見、可刪、可全清、不外送 | Gboard 有「Delete learned words & data」，macOS 有使用者詞典頁；信任與除錯都需要 | [S17][S23]，有據 |
| 後續建議（association） | iOS 使用者觀察好用；Microsoft 有 Associate phrase window | [S25]，有據（Microsoft）；iOS 為觀察 |
| 中文模式直接出英文候選 | macOS 官方功能，使用者觀察到 | [S13]，有據 |
| 預測補全（輸入一半的補全碼） | macOS 補全放在第二或第三候選，避免搶首選 | [S14]，有據；排位策略可借鑑 |
| 容錯（鄰鍵替換、漏碼、多碼） | Gboard 的 spatial model 明確建模三類錯誤 | [S1]，有據（英文）；速成需自行設計 |
| 保守的 post-correction | 事後修正有誤改風險，論文建議只在最近一詞、高信心時 | [S1]，有據 |
| undo 上一次替換（快捷鍵） | macOS 有 Shift-Delete；Gboard 退格即還原 | [S14][S15]，有據 |
| 倉頡 Z 萬用鍵 / 速成結構候選 | macOS 已支援，使用者遷移成本低 | [S13][S14]，有據 |
| emoji 候選 | macOS 候選窗含 emoji | [S14]，有據 |
| 雲端建議（完全 opt-in，明確標示） | Microsoft 用 Bing 並以雲圖示標示 | [S24]，有據；對 QIM 的隱私取捨需要另外決定，預設不開 |
| Android：整合 inline autofill | 官方 API 允許 IME 顯示 autofill 建議；Gboard 可能已如此顯示 email | [S12]，有據（API）；Gboard 行為為假設 |
| 細胞詞庫（可安裝的領域詞庫） | 搜狗的功能；macOS 有 Specialized vocabularies 設定 | [S26][S14]，有據 |
| 詞庫可獨立更新 | Apple 的 Mecabra RapidUpdates 結構 | 第 5 節，僅憑檔名，推論 |
| 隱私預設：端上學習；若要做統計，用 FL + DP 的路線 | Gboard 所有神經 LM 皆 DP，ε 範圍已公開 | [S7][S8]，有據；QIM 是否需要是產品決定 |
| 以神經 LM 重排序（非必要） | 1.4 MB / 1.4M 參數級別的 CIFG LSTM 已可上手機 | [S3]，有據；對速成的收益未查證，建議當作之後的實驗 |

## 來源

Google / Gboard
- [S1] Ouyang, Rybach, Beaufays, Riley. Mobile Keyboard Input Decoding with Finite-State Transducers. arXiv:1704.03987. https://arxiv.org/abs/1704.03987（PDF 全文經 tavily-extract 取得：延遲約 20 ms、LM 5–10 MB、64K 詞 n-gram、動態模型與 on-the-fly rescoring、編輯類型、post-correction）
- [S2] The Machine Intelligence Behind Gboard. Google Research Blog, 2017. https://research.google/blog/the-machine-intelligence-behind-gboard/
- [S3] Hard et al. Federated Learning for Mobile Keyboard Prediction. arXiv:1811.03604. https://arxiv.org/abs/1811.03604（架構與大小來自 PDF 全文）
- [S4] Chen, Mathews, Ouyang, Beaufays. Federated Learning Of Out-Of-Vocabulary Words. arXiv:1903.10635. https://arxiv.org/abs/1903.10635
- [S5] Yang et al. Applied Federated Learning: Improving Google Keyboard Query Suggestions. arXiv:1812.02903. https://arxiv.org/abs/1812.02903
- [S6] Xu et al. Federated Learning of Gboard Language Models with Differential Privacy. arXiv:2305.18465（ACL industry track 2023）. https://arxiv.org/abs/2305.18465
- [S7] Advances in private training for production on-device language models. Google Research Blog. https://research.google/blog/advances-in-private-training-for-production-on-device-language-models/
- [S8] Federated Learning of Gboard Language Models with Differential Privacy（research.google 頁面）. https://research.google/pubs/federated-learning-of-gboard-language-models-with-differential-privacy
- [S11] van Esch et al. Deep Internationalization for Gboard, the Google Keyboard. arXiv:1912.01218. https://arxiv.org/abs/1912.01218
- [S15] Gboard Help：Get word suggestions & fix mistakes（Android）. https://support.google.com/gboard/answer/7068415?hl=en&co=GENIE.Platform%3DAndroid
- [S16] Google Pixel 社群：舊版 Gboard 有 Sync personal dictionary，新版改為匯入 / 匯出（二手，Google 社群）. https://support.google.com/android/thread/98818352/
- [S17] Gboard Help：Learn how Gboard gets better. https://support.google.com/gboard/answer/12373137?hl=en
- [S18] Cantonese on Android（第三方整理，僅作線索）. https://www.cantonese.tools/android.html
- [S19] Gboard Help：Fix problems with Gboard（iPhone & iPad）. https://support.google.com/gboard/answer/9058584?hl=en&co=GENIE.Platform%3DiOS
- [S30] Google 社群：Gboard 學到的詞與個人詞典是分開的（二手，僅作線索）. https://support.google.com/chrome/thread/330850994/

Android
- [S12] Android Developers：Autofill — IME integration（inline suggestions，Android 11+）. https://developer.android.com/identity/autofill/ime-autofill

Apple
- [S9] Apple Newsroom：iOS 17 is available today. https://www.apple.com/uk/newsroom/2023/09/ios-17-is-available-today
- [S10] Apple Support：Use predictive text on iPhone. https://support.apple.com/guide/iphone/use-predictive-text-iphd4ea90231/ios
- [S13] Apple Support：Type Chinese using Cangjie - Traditional on Mac. https://support.apple.com/guide/chinese-input-method/cangjie-traditional-cimcjt11820/mac
- [S14] Apple Support：Change Chinese and Cantonese input source settings on Mac. https://support.apple.com/guide/chinese-input-method/change-chinese-and-cantonese-settings-cim21aa5fa50/mac
- [S20] Apple 社群頁（引用支援文字：拒絕同一建議幾次後停止建議；二手）. https://discussions.apple.com/thread/254733887
- [S21] Apple Machine Learning Research：Apple Intelligence Foundation Language Models（約 3B 端上模型；與鍵盤 autocorrect 無關，僅用於排除混淆）. https://machinelearning.apple.com/research/apple-intelligence-foundation-language-models
- [S22] Apple Support：Type Chinese using Sucheng - Traditional on Mac. https://support.apple.com/en-gb/guide/chinese-input-method/cimsgt11821/mac
- [S23] Apple Support：Keyboard shortcuts for the Traditional Chinese input method on Mac. https://support.apple.com/guide/chinese-input-method/keyboard-shortcuts-for-traditional-chinese-cimkeyt36646/mac
- 本機只讀結構調查：第 5 節，路徑見表。

Microsoft
- [S24] Microsoft Simplified Chinese IME. https://support.microsoft.com/en-us/windows/hardware/input-devices/microsoft-simplified-chinese-ime
- [S25] Microsoft Traditional Chinese IME. https://support.microsoft.com/en-us/windows/hardware/input-devices/microsoft-traditional-chinese-ime

搜狗與其他
- [S26] 搜狗輸入法 - Microsoft Store. https://apps.microsoft.com/detail/xp8ljrrt557rdl?hl=zh-CN&gl=CN
- [S27] 騰訊搜狗輸入法宣布全面 AI 化（infocast，二手新聞）. https://hk.finance.yahoo.com/news/%E9%A8%B0%E8%A8%8A%E6%90%9C%E7%8B%97%E8%BC%B8%E5%85%A5%E6%B3%95%E5%AE%A3%E5%B8%83%E5%85%A8%E9%9D%A2ai%E5%8C%96-025601569.html
- [S28] Wikipedia：Sogou Pinyin（僅作線索）. https://en.wikipedia.org/wiki/Sogou_Pinyin
- [S29] Chen, Zhao, Wang. Neural Network Language Model for Chinese Pinyin Input Method Engine. PACLIC 2015. https://aclanthology.org/Y15-1052

## 未查證清單

- Gboard 中文 / 粵語 / 速成引擎的內部架構、詞庫與語料來源。
- Apple iOS 鍵盤 transformer 的模型大小、延遲、訓練資料、速成引擎內部；macOS Mecabra 內部。
- Microsoft Pinyin / Quick 的排序與自學習機制；搜狗的全部內部資料。
- Gboard 是否讀取輸入框前文、`bc` → email 的真正來源。
- 本調查沒有實際在裝置上跑第 4 節的測試。
