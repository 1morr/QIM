# QIM 設計綱要（Design Brief）

> 狀態：使用者已確認（2026-10-30）。本檔只寫現況：決定改了就直接改寫，不留「原寫」「取代原」「某日改」；舊版與理由記在 progress〈偏差與決定〉，影響架構的寫 ADR。標【決定】的段落是拍板過的，之後要推翻得在 `docs/progress.md`〈偏差與決定〉記一行。入門背景見 `docs/research/ime-primer.md`。

## 0. 一句話【決定】

給速成熟手的全平台輸入法：速成能連打、首選準、會學使用者的用字，各平台體驗一致。

## 1. 產品定位

### 1.1 本產品負責【決定】

- 速成連打：一串不分隔的速成碼切成字詞並排出首選（§4）。
- 候選排序與使用者學習，全部在裝置上。
- 輸入方案的抽象：核心不寫死速成，日後可加倉頡、粵拼、注音。
- 手機的觸控鍵盤，佈局用資料描述，提供畫布式的佈局編輯器（§4.6）。
- 功能分級見 §1.5。

### 1.2 本產品不負責【決定】

- 初學者教學：只提供字根提示，不做教學模式。
- 自架雲端服務：雲端 LLM 由使用者自帶 API key，QIM 不架代理伺服器、不做帳號系統（§8）。
- 多端檔案傳輸：待評估，即使做也放在主 App，不在輸入法引擎（`docs/research/ime-primer.md` §5）。
- 輸入碼裡的小工具（計算機、日期、大寫金額）與速成輔碼（碼後補倉頡中間碼篩字）：使用者用不到，重碼靠連打、學習與翻頁。

### 1.3 參考的成熟產品與採用的慣例

| 產品 | 採用 | 刻意不同 |
| --- | --- | --- |
| Gboard | 連打、容錯、長按出英文、預設佈局 | 候選依使用頻率前移 |
| iOS / macOS 內建速成 | 聯想、中文狀態下的英文候選、字根提示 | 支援連打 |
| Rime | 輸入方案與核心分離的做法 | 每個方案只索引自己的碼；長詞條與造句用同一把尺比分數（`docs/research/rime.md` §5） |
| libime | 碼表格式、使用者學習的做法 | — |
| afm-ime / 青簡 | 慢的模型只做非同步重排、不搶首選（ADR 0005）；青簡的容錯「預設相信使用者」（§4.8）、譯詞、主題格式（可繼承、按淺深色給變數） | 譯詞表與詞庫不拷青簡的加工檔（GPL，`docs/research/qingjian.md` §10.3） |
| Delta skin、Keyboard Designer | 佈局是畫布：設計尺寸加每鍵座標；按鍵可自由擺放 | 允許重疊、多邊形、旋轉；內建格線與吸附 |

### 1.4 名稱

QIM（暫名）。

### 1.5 功能分級【決定】

依據 `docs/research/ime-primer.md` §5。

- **MVP**：速成連打；候選依使用頻率學習；手動新增詞與觸發詞；上屏學詞（含分段選字後自動造詞）；使用者詞補上詞庫沒有的詞；粵語口語詞（使用者詞加一份授權乾淨的小詞表）；簡繁切換；中文狀態下的英文候選；組字中修改分段；畫布模型的佈局（預設 Gboard 佈局、多頁、長按、直向與橫向、淺色與深色）；按鍵放大預覽；佈局編輯器；手機的候選展開面板；字根提示；學習資料可看可刪；無痕模式。
- **之後**：聯想；雲端聯想與補全；資料包線上更新；可選的崩潰回報；自動記住 email 這類字串；輸入框上下文配合 n-gram 排序；英文滑動輸入；剪貼簿歷史與同步；曲線自由手繪的按鍵、按鍵貼圖、外觀主題（鍵盤與桌面候選窗共用一份主題格式）；個人 bigram；空間模型；英文模式的預測與自動修正；依欄位類型切換；智慧標點；單手佈局；震動與按鍵音；觸控平台的實體鍵盤；Android 懸浮鍵盤；無障礙；萬用鍵（`z`：速成只用 a–y；macOS 倉頡也用 Z，`docs/research/closed-source-imes.md` §1.2）；容錯（§4.8）；譯詞（打中文出英文，可選資料包，詞表來源與授權開工時查）；撤銷學習；空白鍵滑動移游標；常用語；emoji 候選；詞庫匯入；完整粵語詞庫（授權查清後）。
- **待評估**：小型神經模型重排；LLM 候選優化；中文滑動輸入；語音輸入；多端檔案傳輸；句中英文混輸（§4.7）；腳本外掛（網頁版驗收後再定，理由見 `docs/research/cyime.md` §6）。

## 2. 使用者與情境【決定】

- 主要使用者：會連打的速成熟手，港澳、寫繁體、會打粵語口語、中英混打，在手機和電腦之間切換。
- 次要（之後）：倉頡、粵拼、注音的使用者，透過新增輸入方案支援。
- 不是主要對象：初學者，只靠字根提示照顧。

## 3. 平台範圍與順序【決定】

網頁 prototype → 平台可行性實驗 → 智慧候選、輸入增強、佈局編輯器（網頁）→ 使用者驗收 → macOS → iOS → Android → Windows → Linux。

- 先在網頁做完所有核心功能：功能大多在共用核心裡，平台殼開工時核心介面已穩定，每個平台一次接上全部功能，不必做半成品再慢慢修改（plan §7）。
- 平台可行性實驗排在智慧候選之前：iOS 限制最多（記憶體上限不公開、沒有 marked text、密碼欄換系統鍵盤，§12.3），模型大小要先知道放不放得下。
- macOS 是第一個平台殼：開發機就是 Mac，能用 `orca computer` 實跑驗證，沒有記憶體上限，適合打通核心與平台殼的界線。
- 網頁驗不到的部分（真實 App 內的行為、各平台讀上下文的差異、手機真實延遲）仍可能在平台里程碑修改。
- Android、Windows、Linux 依序。

## 4. 輸入行為

### 4.1 字集與字形【決定】

- 輸出以繁體為主，字集涵蓋 Big5 加 HKSCS；簡體輸出靠 OpenCC 轉換。
- Unicode 擴展區的罕用字不出現在連打候選，只在單字查詢時出現。
- 字形跟系統字型。

### 4.2 取碼【決定】

- 速成碼取倉頡碼首尾兩碼；同時接受由倉頡三代與五代推導出的速成碼（Microsoft 預設三代、macOS 用五代，據 [維基百科](https://zh-yue.wikipedia.org/wiki/%E5%80%89%E9%A0%A1%E8%BC%B8%E5%85%A5%E6%B3%95)，二手來源）。
- 速成碼在建置資料包時推導，存成速成輸入方案自己的碼表；倉頡碼不是速成的合法輸入碼（ADR 0003）。
- 字根提示預設顯示五代，可改三代。

### 4.3 桌面鍵盤操作【決定】

參考 macOS 與 Windows 內建速成，加上連打。

| 鍵 | 行為 |
| --- | --- |
| 空白 | 上屏首選（翻頁後也一樣） |
| 1–9 | 選對應候選 |
| `-` / `=`、PageUp / PageDown | 翻頁 |
| Enter | 把輸入碼當英文原樣上屏 |
| Esc | 清除輸入碼 |
| ← / → | 組字中移到前一段或後一段，候選改列該段的字詞 |
| Shift（單按） | 切換中英文 |

- 標點預設全形。
- 英文候選：輸入碼剛好是完整英文單字時放第 2 位，不搶首選；不容錯、不補全，糾錯與預測只在英文模式（§4.7）。
- 組字中顯示已打的字根（如「日弓口」）；候選旁不顯示碼，用快捷鍵查字根提示。

### 4.4 手機觸控鍵盤【決定】

以 Gboard 佈局為底：

- 字母鍵主標籤顯示字根，角標顯示英文字母（AnySoftKeyboard、倉輸入法的倉頡鍵盤做法）。
- 長按字母鍵彈出英文字母；第一排另含數字。
- 數字列預設不顯示，設定可開。
- 打完符號留在符號頁，按「ABC」才回字母頁。
- 候選列單列、可左右滑，右側展開鍵一次看全部候選（候選展開面板因此列入 MVP）。
- 空白上屏首選；英文候選規則同 §4.3。
- 按鍵時顯示放大預覽；點組字裡的某一段可改該段。
- 候選列不因背景模型的結果改動首選（ADR 0005）。

### 4.5 學習與排位【決定】

- 候選依使用頻率逐步前移：同一個輸入碼選同一個候選兩次即成首選，兩次之間隔多久都算；舊習慣隨使用次數衰減。參考 librime 的衰減計數（§12.2），參數用首選字基準調。
- 學過的字詞在更長的連打裡也加分，不只在輸入碼完全相同時生效；個人分只加不減（libime 做法，§12.4）。
- 自動造詞：分段選字上屏後立即學成使用者詞，下次先排第 2，再選一次成首選。
- 手動新增的使用者詞在打它的觸發詞時一定是首選。
- 學習可在設定關閉或重置。
- 無痕模式：密碼欄與帶「不要個人化學習」旗標的欄位自動進入，也可從候選列按鈕手動切換。
- 學習資料在主 App 列出，可搜尋、刪除、全清。

### 4.6 佈局與編輯器【決定】

- 佈局是畫布：每個按鍵各自有位置、大小、旋轉、形狀、上下層順序、外觀與動作（ADR 0004）。預設是 Gboard 佈局，也是這種資料。
- 一份佈局可依裝置與方向有多個佈局版本（手機、平板；直向、橫向），每個按鍵有淺色與深色的顏色，並帶無障礙標籤。
- 允許搞怪：按鍵可以重疊、亂序、超出常規。重疊時上層優先；各平台用同一份點擊判定。
- 形狀：長方形、圓角、橢圓、旋轉、拖頂點改任意多邊形。外觀：每鍵顏色、透明度、文字大小與位置。曲線自由手繪、按鍵貼圖、外觀主題是之後的功能。
- 編輯器：拖動、縮放、旋轉、改形狀、調上下層、多選、復原；設定輸出、長按、滑動、切頁；邊編輯邊試打；匯入匯出。只寫一份網頁版，在各平台主 App 內嵌，桌面可用滑鼠編輯（ADR 0004）。
- 對齊：格線與吸附預設開，欄數列數可調並存進佈局檔；可一鍵切完全自由，也可按住修飾鍵（手機用雙指）暫時不吸附；拖動時顯示與其他按鍵對齊的參考線。
- 安全底線：只擋三種情況：檔案壞掉；頁面之間切不回有字母的頁面；iOS 需要切換鍵盤鍵時卻沒有。重疊、被蓋住、超出畫布、缺字母只警告。主 App 永遠有「恢復預設佈局」。
- 單手：內建左手、右手兩份預設佈局。懸浮鍵盤只在 Android 做；iOS 不能畫到鍵盤外（§12.4）。

### 4.7 英文模式與欄位類型【決定】

- 英文模式有單字預測、自動修正（退格可還原）、Shift 與句首大寫。自動修正只在英文模式作用。
- 中文狀態的英文候選只認拼對的完整單字，不容錯、不補全：速成連打兩個字就是 4 個字母，隨機 4 個字母約四成與英文詞只差一處（§12.1），容錯會常佔第 2 位、打亂熟手「按 2 選某字」的習慣；拼對的完整單字隨機撞到不到 1%。
- 句中英文混輸（一串輸入碼裡夾英文詞，如「今日有meeting」）待評估：拼音靠「拆不成合法音節」認出英文，速成沒有這個訊號，只能比語言模型分數，誤判風險高；M4 當實驗，同樣只認拼對的常見詞，純中文句子的首選與修正次數不退步才採用。
- 欄位類型：email、網址預設英文模式並關自動修正；數字欄開數字頁。
- 智慧標點（可關）：數字後的句點與逗號用半形；成對符號自動配對；英文模式連按兩下空白輸出句點。

### 4.8 容錯【決定】

依據 `docs/research/ranking-techniques.md` §5 與青簡（§12.4）。

- 預設相信使用者：原樣讀得通就照原樣排，修正過的讀法帶懲罰，通常只當備選。
- 速成每字只有兩碼，幾乎每個兩字母組合都是合法碼，打錯或漏碼多半變成另一個合法字（漏打 `anr` 的 `n` 就是「問」的 `ar`）。所以單字不容錯，只在詞與句層靠詞庫和上下文判斷「原樣讀不通、改一處就通」。
- 按鍵位置：觸控用點擊位置算各鍵的機率（空間模型）；實體鍵盤用實體鍵盤的幾何算鄰鍵（含上下排與斜角），按實體鍵位、不按字母，換配列照樣適用。
- 機制所有方案共用，允許哪些修正（鄰鍵替換、漏碼、多碼、兩碼對調）、懲罰與啟動條件由方案各自設定並用自己的基準調：倉頡長碼稀疏，單字層也可容錯；拼音、粵拼另有模糊音與簡拼。

## 5. 碼表與資料

### 5.1 發佈原則【決定】

碼表、詞庫、語言模型放在與程式碼分開的資料包，各自保留原授權與聲明；LGPL 資料這樣隨 App 發佈是否足以符合條款，由使用者判斷。

### 5.2 來源【決定】

| 資料 | 來源 | 授權 | 備註 |
| --- | --- | --- | --- |
| 倉頡碼表（三代、五代） | libcangjie `data/table.txt` | Public Domain（聲明的源頭未獨立查證，使用者接受） | 速成碼由此推導 |
| 字序、碼表交叉驗證 | Unihan `kGradeLevel`、`kCangjie` | Unicode License v3 | `kGradeLevel` 排單字字序（有字頻前用，見 plan §4.1），進速成資料包；`kCangjie` 只用來抓錯碼，不打包 |
| 詞頻 | rime-essay | LGPL-3.0 | 獨立資料包；建置時設最低詞頻門檻；之後可換成自建詞頻 |
| 粵語口語小詞表 | 自行整理 | CC0 | 不從 rime-cantonese、粵典複製 |
| 口語語料 | HKCanCor、Tatoeba 粵語句子；Common Voice 粵語句子待讀完條款 | CC BY、CC BY 2.0 FR；Common Voice 標 CC0-1.0（§12.1） | 維基偏書面語，要補口語，訓練 bigram 與基準都用；LIHKG、OpenSubtitles 權利來源不清，不用 |
| 字級 bigram | 中文維基、粵語維基訓練（plan §4.3） | 語料為 CC BY-SA 4.0 | 獨立資料包；模型是否須同授權散布見 §11 |
| 英文詞表 | SCOWL | MIT 類寬鬆授權 | — |
| 簡繁轉換 | OpenCC | Apache-2.0 | — |

不用 Rime 的倉頡 / 速成碼表（授權標示矛盾，§12.1）。

### 5.3 版本與更新【決定】

- 資料包隨 App 發佈一份，之後可以線上更新、不必等 App 發版；設定可關閉線上更新。
- 新輸入方案：打字必需且不大的資料內建，在設定裡啟用，不用連網；很大的模型等可選資料才在主 App 內下載；另可從檔案匯入資料包，與下載走同一條安裝流程。查碼方式等程式碼只隨 App 發版，下載的只有資料（§12.4 的 App Review 2.5.2）。
- 使用者詞庫檔案格式帶版本號，App 升級時先備份再自動轉換。

## 6. 核心引擎

- 【決定】自己寫核心，參考 libime 與 librime 的做法但不依賴它們（ADR 0001）。
- 【決定】核心與平台殼分離：核心是純邏輯，收輸入碼、回候選；平台殼負責註冊、事件、畫面、上下文。
- 【決定】核心用 Rust：編成 WebAssembly 給網頁 prototype，uniffi 給 Swift / Kotlin，C ABI 給 Windows TSF 與 Fcitx5。網頁 prototype 的引擎就是真實核心，只有網頁 UI 會丟（ADR 0002）。
- 【決定】輸入方案是宣告式的方案檔（參考 Rime `schema.yaml`）：鍵、字根標籤、碼表、每字最長碼數、詞庫。切分、排序、學習寫在核心，所有輸入方案共用，方案檔調參數。拼音、粵拼這類音碼方案另需讀音資料與一種新的查碼方式（分隔符、簡拼、懶音）。格式在 plan 設計。

## 7. 設定與同步

【決定】

- MVP 不同步；設定在主 App（桌面為設定視窗）修改，存在本機。
- 使用者詞庫可匯出、匯入成檔案，用來換機或備份。
- 跨裝置同步是「之後」的功能：合併規則與詞庫格式先在核心定稿；實際傳輸方式在第一個平台殼開工時選（iCloud、自架、同步資料夾），要能跨 Apple 與 Android。

## 8. 隱私與安全【決定】

網路是正式功能，不是例外；以下三條是硬限制：

1. 核心打字永不依賴網路。打字、排序、學習在裝置上完成；雲端結果一律非同步，斷網或慢時打字與本地候選不受影響。
2. 密碼欄與帶「不要個人化學習」旗標的欄位：不學習、不讀上下文、不記剪貼簿、不送雲端。
3. iOS 不要求完整取用權限：沒開時打字與學習照常（鍵盤可寫自己的容器）；開了才有雲端、同步、剪貼簿歷史（App Review 4.4.1，§12.3）。

其他：

- 雲端功能預設開啟，但只有使用者在設定裡接上雲端服務（填入 API key）後才實際連線；雲端給的候選標示來源。
- 雲端 LLM 由使用者自帶 API key，支援 OpenAI 相容介面與 Anthropic 介面；QIM 不架代理服務。
- API key 存系統的安全儲存（Keychain、Android Keystore 等），不存明文。
- 送出前遮蔽 email、電話、卡號這類字串；可設每日請求上限，超過就停止送出，打字不受影響；可排除指定 App，平台拿不到宿主識別時在設定頁註明。
- 送往雲端的內容：游標前最多 200 字、目前輸入碼、前 10 個候選；不送游標後的文字。QIM 不在任何伺服器保存；模型供應商的保存規則依其政策，在設定頁註明。200 字由模型階梯實驗調整（plan §7.8）。
- 不收集使用統計。崩潰回報可選、預設關閉，只送堆疊不送輸入內容。

## 9. 效能與品質指標

【決定】

- 延遲：在最低階目標裝置上，每按一鍵到候選更新，目標 ≤ 50 ms，上限 100 ms（研究推出的工程預算，`docs/research/ranking-techniques.md` §8.5）。
- 記憶體：iOS 鍵盤擴充常駐暫定 ≤ 30 MB，平台可行性實驗（plan §7.3）量到真實上限後修訂。
- 首選正確率：首選字基準。把真實繁體 / 粵語句子機械轉成速成碼，讓核心連打，量整句首選正確率與字正確率；`mir` → 一台、`anr` → 門口等案例為固定必過題。改核心或資料包時必跑，答錯數不得上升。
- 修正成本：長句首選常會錯（速成每字資訊少，§12.4 的拼音數字可參考），所以同時量打一句平均要選字和翻頁幾次、每字按鍵數，以及打字途中首選跳動多少；修正次數不得上升（plan §5.2）。
- 基準語料：中文維基與粵語維基抽樣（CC BY-SA 4.0，基準檔單獨標註授權，只放測試資料、不打包進 App）；口語語料（§5.2）；使用者提供的真實句子（去個資，M1 就要有）；使用者回報的案例為固定必過題，持續增加。品質判斷以口語與真實句子為主，維基只防退步。

## 10. 發佈與更新

- 【決定】開源，程式碼 Apache-2.0；免費上架 App Store / Play Store，桌面發安裝檔。資料授權見 §5.1。
- 【決定】使用者目前沒有付費 Apple Developer 帳號：macOS 里程碑先用本機簽章自用；iOS 上架前申請帳號，之後 macOS 改發簽章、公證過的 `.pkg`。自動更新與 Homebrew cask 之後再做。
- 【決定】iOS：App Store。Android：Play Store，GitHub Releases 同時發 APK；F-Droid 之後。Windows：簽章安裝檔；winget 之後。Linux：框架（Fcitx5 或 IBus）與打包到 Linux 里程碑再定。

## 11. 待決

- 語言模型升級、小型神經重排、LLM 候選優化各接入到什麼程度：模型階梯實驗後決定（plan §7.8）。
- Common Voice 粵語句子能否用於訓練與基準：標 CC0-1.0，但資料說明另寫不得轉發資料集，M1 票 03 讀完條款後定（§12.1）。
- LGPL 資料、以 CC BY-SA 語料訓練的 bigram 隨商店 App 發佈是否合規：使用者判斷（§5.1、§5.2）。
- 沒有付費帳號時，iOS 鍵盤擴充能否用免費佈建在實機上跑、App Group 能否使用：平台可行性實驗開工時查證（plan §7.3）。
- Linux 用 Fcitx5 還是 IBus：Linux 里程碑開始時定（§10）。

## 12. 查證過的事實

外部事實（平台 API 行為、碼表授權、外部引擎能力），每條附來源連結或 `docs/research/` 的檔案。沒有來源的不寫進來。查證日 2026-10-30；細節與未查證清單在各研究檔。

### 12.1 速成與碼表

- 速成碼取倉頡碼首尾兩碼，1–2 碼的倉頡碼不變。[Microsoft Traditional Chinese IME](https://github.com/MicrosoftDocs/globalization/blob/main/globalization/input/traditional-chinese-ime.md)、[Apple 速成說明](https://support.apple.com/en-gb/guide/chinese-input-method/cimsgt11821/mac)、`rime/rime-quick` `quick5.schema.yaml`。
- libcangjie 的 74,677 個同時有三代與五代碼的漢字中，3,628 字推導出的速成碼不同，其中 Big5 字 715 個。多數差異是多一個 `x` 開頭的難字碼，也有真正不同的（丟：三代 `hi`、五代 `mi`；乍：`hs` / `os`）。本機統計，libcangjie `data/table.txt` 第 12、13 欄，2026-10-30。
- 速成重碼重：fcitx 速成表全集 674 個碼、單碼最多 480 字，常用集單碼最多 93 字。`docs/research/data-sources.md` §3.2（本機統計）。
- libcangjie `data/table.txt`（77,575 列，倉頡三代與五代、HKSCS 旗標）聲明為 Public Domain。[libcangjie](https://gitlab.freedesktop.org/cangjie/libcangjie) `data/README.table.md`。
- Unihan `kCangjie` 29,189 字，對應倉頡三代，授權 Unicode License v3。[UAX #38](https://www.unicode.org/reports/tr38/)、[Unicode copyright](https://www.unicode.org/copyright.html)。
- Unihan `kGradeLevel`：香港小學第幾年級要認得這個字（1–6），出自《朗文初級中文詞典》2001，屬 Provisional。18.0.0 有 2,633 字，其中 2,625 字在連打字集；「日」1 年級、「曰」沒有年級。[UAX #38](https://www.unicode.org/reports/tr38/)；字數為本機統計，2026-11-03。
- Unicode License V3 的全文在 `https://www.unicode.org/license.txt`，沒有隨 UCD 版本化的網址，版權年份每年改（2026-01-05 版為 1991-2026），所以 QIM 把全文存在 `data/licenses/Unicode-3.0.txt`，不釘下載。[license.txt](https://www.unicode.org/license.txt)，2026-11-03 查。
- Unihan 18.0.0 `kCangjie` 與 libcangjie `346026b` 的三代碼：兩邊都有碼 29,121 字，2,744 字不同；其中 2,246 字的 Unihan 碼等於 libcangjie 的五代碼，其餘 498 字才是疑似錯碼。本機統計，`cargo xtask data --verify` 的報表，2026-11-03。
- rime-cangjie、rime-quick、rime-essay 為 LGPL-3.0；`cangjie5.base.dict.yaml` 檔頭卻寫 GPL，上游《五倉世紀》權利人未查明。各 repo LICENSE；`docs/research/rime.md` §3。
- 繁體詞頻只找到 LGPL 來源：rime-essay（442,688 筆）、libchewing-data（160,462 筆）。教育部國語辭典為 CC BY-ND 3.0 台灣、無頻次。`docs/research/data-sources.md` §5。
- rime-cantonese 為 CC BY 4.0（`maps` 為 ODbL），但詞條參考了非商業授權的粵典及商業輸入法詞庫，授權鏈未查明；粵典 `wordslist`（字 → 讀音）為 Public Domain。[rime-cantonese](https://github.com/rime/rime-cantonese)、[粵典授權](https://words.hk/base/hoifong/)。
- OpenCC（含字典）為 Apache-2.0；純 Rust 實作 ferrous-opencc 亦為 Apache-2.0。[OpenCC](https://github.com/BYVoid/OpenCC)。
- CLDR annotations（yue、zh_Hant、zh_Hant_HK）為 Unicode-3.0。[CLDR](https://github.com/unicode-org/cldr/tree/main/common/annotations)。
- HKSCS 資料集（含倉頡碼、粵拼）商用免費，須署名，有賠償條款。[data.gov.hk 條款](https://data.gov.hk/en/terms-and-conditions)。
- 隨機速成字母串（a–y）對 macOS `/usr/share/dict/words` 的 210,773 個小寫詞：4 個字母剛好是英文詞 0.9%、差一處（插入、刪除、替換、對調）38.5%；5 個字母 0.1%、8.7%；6 個字母 0%、1.1%。隨機字母不代表真實速成碼的分佈，詞表含冷僻詞，只看量級。本機實驗 `scripts/experiments/english-edit1/`，2026-11-03。
- HKCanCor：1997–1998 年的自發對話與電台節目轉寫，約 23 萬詞，CC BY（目錄標 4.0，未找到主要來源的授權聲明）。[Hugging Face](https://huggingface.co/datasets/hkcancor)、[PyCantonese](https://pypi.org/project/pycantonese)，2026-10-31 查。
- Common Voice Scripted Speech 25.0 粵語：標 CC0-1.0，文字語料 28,727 句；24.0 的資料說明另寫不得轉發資料集，兩者的關係未查清。[25.0](https://datacollective.mozillafoundation.org/datasets/cmn29rqn9016to107eniyak65)、[24.0](https://datacollective.mozillafoundation.org/datasets/cmj8u3q2b00v9nxxborfkm824)，2026-10-31 查。
- Tatoeba 句子預設 CC BY 2.0 FR，須署名，部分句子另有 CC0；粵語句子的數量未查。[Tatoeba 下載頁](https://tatoeba.org/ca/downloads)、[CC0 說明](https://en.wiki.tatoeba.org/articles/show/cc0-contributions)，2026-10-31 查。

### 12.2 Rime 行為（librime `56737cd2`）

- rime-quick 的 quick5 匯入倉頡全碼表 `cangjie5.base`，用 `derive/^([^z])\w+(\w)$/$1$2/` 另加速成碼；`derive` 保留原拼寫且不降權，所以倉頡全碼也是合法輸入、排序時與速成碼不分。`rime-quick` `5dcdb9e`；`src/rime/algo/calculus.h:72-73`。
- script_translator 只要有詞條精確覆蓋整段輸入就不造句。`src/rime/gear/script_translator.cc:502-505`。
- 同長度候選先比是否精確匹配、再比剩餘碼長、最後比詞頻。`src/rime/dict/dictionary.cc:73-84`。
- 不用語言模型造句時每多一個詞扣 log(1e-6)。`src/rime/gear/grammar.h:18-27`。
- 使用者詞熱度隨上屏次數指數衰減，時間常數 200 次。`src/rime/algo/dynamics.h`、`src/rime/dict/user_dictionary.cc:425-457`。
- rime-quick `45d0690`（2026-07-17）拿掉 `min_phrase_weight: 100`，八股文全部約 49.8 萬條多字詞進碼表，含權重 0 的詞。
- 三案例根因：`anr` 的「問」是倉頡全碼、與「門口」同長而比詞頻（50737 > 9435）；`mir` 的「一台」不在詞庫，且精確詞條（䂖、一店、歹口）擋掉造句；只保留速成碼後 `anr` 首選變「門口」。`docs/research/rime.md` §4，復現：`scripts/experiments/rime-cases/`。
- librime-octagram 外掛為 BSD-3-Clause，語法模型資料 rime-octagram-data 為 LGPL-3.0；繁體字級 / 詞級壓縮模型 10.3 MB / 41.4 MB（release 20260712）。[librime-octagram](https://github.com/lotem/librime-octagram)、[rime-octagram-data](https://github.com/lotem/rime-octagram-data)。

### 12.3 平台框架

- iOS：密碼欄與電話欄由系統強制換回系統鍵盤；App 可用 `shouldAllowExtensionPointIdentifier` 拒絕第三方鍵盤。[Configuring a custom keyboard interface](https://developer.apple.com/documentation/uikit/configuring-a-custom-keyboard-interface)、[App Extension Programming Guide](https://developer.apple.com/library/archive/documentation/General/Conceptual/ExtensibilityPG/CustomKeyboard.html)。
- iOS：沒有 Full Access 時無網路、App Group 共享容器唯讀；開了才能網路、寫共享容器、iCloud。[Configuring open access](https://developer.apple.com/documentation/uikit/configuring-open-access-for-a-custom-keyboard)。
- iOS：鍵盤擴充有記憶體上限，Apple 不公布數字、隨機型而異；社群實測 30–77 MB 互相矛盾。[Creating a custom keyboard](https://developer.apple.com/documentation/uikit/creating-a-custom-keyboard)；`docs/research/platform-frameworks.md` §3.6。
- iOS：App Review 4.4.1 要求鍵盤在沒有 Full Access 時仍可用。[App Review Guidelines](https://developer.apple.com/app-store/review/guidelines/)。
- iOS：只能經 `textDocumentProxy` 讀游標前後文，長度上限官方未寫；沒有 marked text。`docs/research/platform-frameworks.md` §3.2。
- Android：10+ 只有預設 IME 或有焦點的 App 能讀剪貼簿。[Android 10 privacy changes](https://developer.android.com/about/versions/10/privacy/changes)。
- Android：`getTextBeforeCursor` 是 IPC，可能回 null 或少於要求的字數。[InputConnection](https://developer.android.com/reference/android/view/inputmethod/InputConnection)。
- Android：密碼欄仍交給當前 IME，IME 須自己不顯示、不儲存。`docs/research/platform-frameworks.md` §4.3。
- Android：ML Kit GenAI（Gemini Nano）只允許 top foreground app 推論，背景回 `BACKGROUND_USE_BLOCKED`；IME 是否算前景未查證。[ML Kit GenAI](https://developers.google.com/ml-kit/genai)。
- Android 11+：IME 可顯示 autofill inline 建議，選取前看不到內容。[IME autofill](https://developer.android.com/identity/autofill/ime-autofill)。
- macOS：IMK 沒有「游標前 N 字」API，`selectedRange` 等可能回 nil 或 `NSNotFound`；沒有 email / URL 欄位提示，密碼欄只能靠 Secure Event Input 判斷。SDK `IMKInputSession.h`；`docs/research/platform-frameworks.md` §5。
- Windows：TSF DLL 載入每個宿主程序；密碼欄應停用 text services，不靠 `IS_PASSWORD`；app container 內網路不保證。[InputScope](https://learn.microsoft.com/en-us/windows/win32/api/inputscope/ne-inputscope-inputscope)、[IME requirements](https://learn.microsoft.com/en-us/windows/apps/develop/input/input-method-editor-requirements)。
- Linux：Wayland text-input-v3 的 surrounding text 上限 4000 bytes，`content_hint` 有 `hidden_text`、`sensitive_data`；GNOME 走 text-input-v3 加 IBus，不實作 input-method-v2。[text-input-v3](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/raw/main/unstable/text-input/text-input-unstable-v3.xml)、[mutter](https://gitlab.gnome.org/GNOME/mutter/-/raw/main/src/wayland/meta-wayland-text-input.c)。
- 共用核心先例：Rime 系列為 C++ 核心；青簡為 Rust 核心加 macOS / Windows / Android 的 Rust 殼，無 iOS。`docs/research/platform-frameworks.md` §1。

### 12.4 成熟產品的做法

- Gboard 解碼：lexicon FST、n-gram LM、空間模型組合後 beam search；按鍵約 20 ms 內回饋，鍵盤 LM 不超過 5–10 MB；個人化用動態模型加 on-the-fly lattice rescoring。[arXiv:1704.03987](https://arxiv.org/abs/1704.03987)。
- Gboard 下一詞預測：CIFG-LSTM 1.4M 參數、量化後 1.4 MB、詞表 1 萬；英文靜態 LM 為 Katz 5-gram 125 萬條。[arXiv:1811.03604](https://arxiv.org/abs/1811.03604)。
- Gboard 的神經 LM 皆以聯邦學習加差分隱私訓練；OOV 學詞取的是全體使用者的高頻詞。[arXiv:2305.18465](https://arxiv.org/abs/2305.18465)、[Google Research blog](https://research.google/blog/advances-in-private-training-for-production-on-device-language-models/)。
- macOS 倉頡 / 速成可輸入英文字後從候選窗選，並有「Adjust word frequency」設定。[Apple 倉頡](https://support.apple.com/guide/chinese-input-method/cangjie-traditional-cimcjt11820/mac)、[Apple 設定](https://support.apple.com/guide/chinese-input-method/change-chinese-and-cantonese-settings-cim21aa5fa50/mac)。
- DeepSeek API 相容 OpenAI 與 Anthropic 的介面格式，模型為 `deepseek-flash` 與 `deepseek-v4-pro`；思考模式預設開啟，可用 `thinking` 參數關閉；Responses API 為無狀態、不在伺服器保存對話。[DeepSeek API Docs](https://api-docs.deepseek.com)、[Responses API](https://api-docs.deepseek.com/api/create-response)，2026-10-30 查。
- Apple 端上基礎模型約 3B 參數；iPhone 15 Pro 每個 prompt token 約 0.6 ms、生成 30 token/s。[Apple ML Research](https://machinelearning.apple.com/research/introducing-apple-foundation-models)。
- llama.cpp 在 iPhone 15 Pro 的 Q4_0 實測：TinyLlama 1.1B prompt 646 token/s、生成 57 token/s。[llama.cpp #4508](https://github.com/ggml-org/llama.cpp/discussions/4508)。
- libime：KenLM trie，beam 20，每按一鍵增量更新 lattice；使用者歷史 bigram 以三個 LRU 池（128 / 8192 / 65536 句）固定權重混入。`fcitx/libime` `src/libime/core/decoder.h`、`historybigram.cpp`。
- afm-ime 用 Apple Foundation Models 做異步重排，結果只放第 2 位。`Bemly/afm-ime` `Sources/IMECore/FMReranker.swift`（`a8543a1`）。
- 可重用的滑動輸入實作：FlorisBoard v0.5.2 `StatisticalGlideTypingClassifier.kt`、AnySoftKeyboard `GestureTypingDetector.java`，皆 Apache-2.0；HeliBoard / OpenBoard 的滑動依賴閉源函式庫。`docs/research/open-source-imes.md` §1。
- Delta skin 的 `info.json`：每個方向一個 `mappingSize`（設計尺寸）與 `items`，每項有 `frame`（x、y、width、height）與 `extendedEdges`（擴大觸控範圍），依裝置與方向分版本。[Delta skin 文件](https://noah978.gitbook.io/delta-docs/skins)，2026-10-31 查。
- Keyboard Designer（Android）可自由擺放按鍵並設定尺寸、標籤、顏色、邊框；官網沒提重疊、格線與吸附。[Keyboard Designer](https://www.keyboarddesigner.com)，2026-10-31 查。
- iOS 鍵盤不能畫到鍵盤上緣之外；`needsInputModeSwitchKey` 為真時要自己提供切換鍵盤的按鈕。[App Extension Programming Guide](https://developer.apple.com/library/archive/documentation/General/Conceptual/ExtensibilityPG/CustomKeyboard.html)、[needsInputModeSwitchKey](https://developer.apple.com/documentation/uikit/uiinputviewcontroller/needsinputmodeswitchkey)。
- 開源佈局格式（FlorisBoard、Trime、Hamster、AnySoftKeyboard、HeliBoard）都只支援矩形按鍵。`docs/research/open-source-imes.md` §4。
- 青簡（0.3.2）的兩個可選本地模型都在停鍵後（macOS 80 ms、Windows 30 ms）非同步改首選：知意（mmBERT-small 12 層加選詞頭，權重 Apache-2.0，Windows int8 ONNX 70 MB）讀游標前文，在前 5 / 9 / 18 條中挑首選；通变（自研 P2C，43 MB）在乾淨拼音 6 音節以上時把生成句排第一。作者開發集首選：通变 59.1%、知意 75.0%（含 79 MB 三元模型）。只防「翻過頁就不換」與過期結果，按空白時整句還沒換上的問題仍在。評測資料不入庫。`docs/research/qingjian.md` §4、§5。
- 青簡沒有倉頡、速成；繁體是簡體核心在出口用 OpenCC 轉字形。拼寫糾錯只算一處編輯，替換限 QWERTY 鄰鍵，每段都讀得通時照原樣、糾正只當備選；雙拼與形碼（五筆）不糾錯。英文糾錯至少 4 個字母，對 9.4 萬詞的詞表查一處編輯。譯詞表（英、日、西、越、德，約 30 MB）由 LLM 與機器翻譯離線生成，修飾鍵加數字上屏。主題是一個目錄的 `theme.json` 加圖片，可繼承另一個主題、按淺深色給變數。宣傳影片示範句中英文混輸（`jin'tian'de'me'e'ting'qu'xiao'le` → 今天的Meeting取消了）時開著通变模型。`docs/research/qingjian.md` §1、§6、§7。
- CyIME 九宮格拼音（自測）：真實聊天 130 句整段首選，原算法 54、詞級 bigram 92、詞級 trigram 97；trigram 203 MB，剪枝版 105 MB、約低 3 個百分點，每鍵約 5–7 ms，可選下載。34 MB 的神經整句重排因提升小、拖慢打字、停頓後改動候選而停用。GPL-3.0。`docs/research/cyime.md` §3。
- CyIME 的外掛是 Lua 腳本，主要用於工具面板、線上語音、同步；模型和方案不走外掛。唯一進按鍵路徑的候選變換有 15 ms 硬超時、連續 3 次超時熔斷。它的審計記下新增一個能力要動 6 處。`docs/research/cyime.md` §2。
- App Review 2.5.2 禁止下載或執行會引入、改變功能的程式碼；4.7 允許 HTML5 與 JavaScript 的 mini app、聊天機器人與外掛等不內嵌於二進位的軟體，但開發者要對其負責，並提供索引、內容檢舉，不得未經許可開放原生 API，每次分享資料都要使用者同意。[App Review Guidelines](https://developer.apple.com/app-store/review/guidelines/)，2026-10-31 查。
- xsime（librime 五端前端，自有程式碼 Apache-2.0，可取用但須保留 NOTICE；詞庫各有授權，含 GPL-3.0）的平台殼做法：Windows 的 TSF DLL 不載引擎，引擎在程序外以 named pipe 通訊、失敗時放行按鍵，x86 與 x64 DLL 都要註冊；macOS 以 root 在 pkg postinstall 註冊輸入源，供沙盒 App 辨識，更新前先切走輸入源；iOS 只用 Full Access 閘控網路、剪貼簿與匯出，App Group 只由主 App 寫，使用者詞庫留在鍵盤私有目錄並排除備份，鍵盤 `phys_footprint` 峰值約 23.7 MB（含 librime，單一樣本）；Android 的 `TYPE_NULL` 編輯器仍要能打中文。`docs/research/xsime.md` §2。
- macOS ad-hoc 簽章的 designated requirement 是 cdhash，每次重建都變，輔助功能授權會失效（xsime 維護者觀察）；Apple Development 憑證的預設 requirement 指向憑證、不含 cdhash。免費帳號（Personal Team）是否同樣適用、7 天期限是否及於 macOS 未查證。[TN3127](https://developer.apple.com/documentation/technotes/tn3127-inside-code-signing-requirements)、[Compare memberships](https://developer.apple.com/support/compare-memberships/)，2026-11-03 查；`docs/research/xsime.md` §4。
- Google 條款禁止逆向其服務與機器學習模型以取得商業機密，除非法律允許。[Google Terms](https://policies.google.com/terms)，2026-10-31 查。
- Apple 的 macOS 授權限制反編譯與逆向，例外是法律禁止此限制或開源元件條款允許；核對的是 Sequoia 版摘錄，未核對最新版。[macOS Sequoia SLA](https://www.apple.com/legal/sla/docs/macOSSequoia.pdf)，2026-10-31 查。
