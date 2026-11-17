# QIM 路線圖

> 里程碑的順序、狀態、開工前要讀什麼。還沒開工的里程碑在下面各有一小節；開工 session（`/to-tickets docs/roadmap.md 拆 <里程碑> 的票`）把那一節細化成 `.scratch/<里程碑>/spec.md`，這裡縮成一行連到它。不是里程碑的工作（文件瘦身、流程對齊）也在表裡佔一列，「里程碑」欄以它的目錄名開頭（`arch-diet 架構文檔瘦身` 對 `.scratch/arch-diet/`）。同時有兩批票可以做時，只有一列是「進行中」，另一列的狀態寫明排在誰之後（`未開工，排在 M4 之後`）；要先插做另一批，就把兩列的狀態對調。`python3 scripts/next_step.py` 照這些狀態算出下一步。

## 里程碑

| 里程碑 | 目標 | 拆票前讀 | 狀態 |
| --- | --- | --- | --- |
| M0 骨架 | 閘門、資料管線 v0、核心 tracer bullet：在網頁打出第一個字 | brief §3、§5、§6；plan §0、§1、§4.1、§4.5、§6；ADR 0001–0003；`.scratch/m0/spec.md` | 已驗收，[已封存](archive/M0.md) |
| M1 網頁 prototype | 競品黑箱基線；速成連打、學習、英文候選、佈局；三個 Rime 案例首選正確 | brief §1.5、§4、§5、§9；plan §2–§5、〈M1 網頁 prototype〉；ADR 0004–0006；`.scratch/m0/issues/09-m0-close.md` 的 Comments（留給 M1 各票的發現） | 已驗收，[已封存](archive/M1.md) |
| M2 平台可行性實驗 | 只量數字不做產品：iOS 記憶體上限、Foundation Models、上下文長度、小型神經模型開銷 | brief §9、§11、`docs/research/facts-platforms.md`；plan §1.4、§3、〈M2 平台可行性實驗〉；ADR 0002、0005；`.scratch/m1/issues/24-m1-close.md` 的 Comments（留給 M2 起各里程碑的發現）；`.scratch/m2/spec.md` | 已驗收，[已封存](archive/M2.md) |
| M3 智慧候選 | 模型階梯實驗（雲端 LLM 量過不做）；上下文、詞級 trigram、停鍵層神經重排、個人 bigram、聯想、記住常打字串、詞庫擴充、資料包 manifest 與線上更新 | brief §1.5、§7、§8、`docs/research/facts-platforms.md`；plan §2.3、§2.7、〈M3 智慧候選〉、〈M3 的模型階梯實驗〉；ADR 0005；`docs/research/ios-feasibility.md`；`.scratch/m1/issues/24-m1-close.md` 與 `.scratch/m2/issues/12-m2-close.md` 的 Comments（標 M3 的各條） | 已驗收，[已封存](archive/M3.md) |
| M4 輸入增強 | 空間模型與容錯、英文預測與自動修正、句中英文混輸實驗、欄位類型、智慧標點、同步的合併規則 | brief §4.7、§4.8、§7、`docs/research/facts-products.md`；plan §2.1、§2.3、§2.4、§2.6、§3.1、§4.4、〈M4 輸入增強〉、〈M5 觸控鍵盤與設定〉；`docs/research/ranking-techniques.md` §5；`.scratch/m3/issues/23-m3-close.md` 的 Comments（標 M4 的各條；M1、M2 收尾票標 M4 的也讀） | 已驗收，[已封存](archive/M4.md) |
| M5 觸控鍵盤與設定 | 網頁觸控鍵盤做到平台樣板：Gboard 觸控功能盤點、候選列與工具列接在鍵盤頂端、鍵盤內設定頁、emoji 與文字編輯等面板、滑動輸入與空白鍵移游標（〈M5 觸控鍵盤與設定〉） | 開工時定 | 未開工，排在 arch-diet 之後（M4 回顧 2026-11-17 定）；拆票前讀 `.scratch/m4/issues/31-m4-close.md` 的 Comments |
| M6 佈局編輯器 | 畫布式視覺編輯器、單手佈局；使用者驗收關卡 | 開工時定 | 未開工 |
| M7 macOS | 第一個平台殼，一次接上全部已完成的功能 | 開工時定 | 未開工 |
| M8 iOS | 鍵盤擴充與主 App；多端檔案傳輸評估 | 開工時定 | 未開工 |
| M9 Android | InputMethodService | 開工時定 | 未開工 |
| M10 Windows | TSF | 開工時定 | 未開工 |
| M11 Linux | Fcitx5 或 IBus | 開工時定 | 未開工 |
| arch-diet 架構文檔瘦身 | `docs/plan.md` 縮成根目錄的 `ARCHITECTURE.md`（上限 15,000 字元） | `.scratch/arch-diet/spec.md` | 進行中（2026-11-18 起），已拆 6 張票；之後拆 M5 |
| flow-sync 做票紀錄對齊 | 做票的紀錄只留在票上、不再抄進 progress.md 與 decisions.md；加 `CHANGELOG.md`；收尾票改叫驗收票 | `.scratch/flow-sync/spec.md` | 已驗收（2026-11-18），票 01、02 與修復票 02b、02c 做完 |
| gates 補齊閘門 | 補密鑰掃描、依賴漏洞、依賴更新三道閘門，接進 `cargo xtask check` 與 CI | `.scratch/gates/spec.md` | 已驗收（2026-11-17），票 01 做完 |

狀態只寫一句，進行中的那一個以「進行中」開頭，驗收後換成 `docs/archive/<里程碑>.md` 的連結。

M1–M4 留給之後各里程碑的發現（每條註明去處）在 `.scratch/m1/issues/24-m1-close.md`、`.scratch/m2/issues/12-m2-close.md`、`.scratch/m3/issues/23-m3-close.md` 與 `.scratch/m4/issues/31-m4-close.md` 的 Comments（M4 的已寫進下面接手的各節），里程碑開工細化 spec 時讀屬於它的那幾條。

編號（2026-11-12）：插入 M5 觸控鍵盤與設定，原本的 M5–M10 各往後順延一號（佈局編輯器 M6、macOS M7、iOS M8、Android M9、Windows M10、Linux M11）。M0–M3 的票、spec、封存與上面三張收尾票的 Comments 寫的是舊編號（舊 M5 = 新 M6，依此類推），沒有改。

排法：功能大多在共用核心裡，先在網頁把它們做完並確認可用（M1、M3、M4、M5、M6），平台殼開工時核心介面已穩定，每個平台一次接上全部功能（M7–M11），不必跟著核心反覆改。平台限制會影響核心設計的部分先量（M2），例如智慧候選要用多大的模型。網頁驗不到的部分（真實 App 內的行為、各平台讀上下文的差異、手機真實延遲）仍可能在平台里程碑修改。brief §1.5「之後」與「待評估」的每一項都落在 M3–M9 其中一個；例外是雲端聯想與補全，跟著雲端重評（M3 票 22 的重評條件），沒有排進里程碑。

## M0 骨架

已完成（2026-11-04）：範圍與驗收見 `.scratch/m0/spec.md`，session 紀錄與偏差封存在 `docs/archive/M0.md`。

## M1 網頁 prototype

已完成（2026-11-04）：目標、範圍、共同規則、驗收與票序見 `.scratch/m1/spec.md`，session 紀錄與偏差封存在 `docs/archive/M1.md`；留給之後里程碑的發現在 `.scratch/m1/issues/24-m1-close.md` 的 Comments。

## M2 平台可行性實驗

已完成（2026-11-10）：目標、範圍、共同規則、驗收與票序見 `.scratch/m2/spec.md`，session 紀錄與偏差封存在 `docs/archive/M2.md`；量到的數字在 `docs/research/ios-feasibility.md`，留給之後里程碑的發現在 `.scratch/m2/issues/12-m2-close.md` 的 Comments。

## M3 智慧候選

已完成（2026-11-12）：目標、範圍、共同規則、驗收與票序見 `.scratch/m3/spec.md`，session 紀錄與偏差封存在 `docs/archive/M3.md`；模型階梯的結論在 `docs/research/model-ladder.md`，重驗的證據與留給之後里程碑的發現在 `.scratch/m3/issues/23-m3-close.md` 的 Comments。

## M4 輸入增強

已完成（2026-11-17）：目標、範圍、共同規則、驗收與票序見 `.scratch/m4/spec.md`，session 紀錄與決定封存在 `docs/archive/M4.md`；容錯沒採用的量法在 `docs/research/tolerance.md`，重驗的證據與各票發現的去處在 `.scratch/m4/issues/31-m4-close.md` 的 Comments。

## M6 佈局編輯器

畫布式視覺編輯器（ADR 0004、brief §4.6）。放在 `web/editor/`，用 `qim-layout` 的 WebAssembly 版做驗證與點擊判定；平台里程碑在主 App 內嵌同一份。

- 編輯：拖動、縮放、旋轉、拖頂點改多邊形、橢圓、調上下層順序；多選、複製、刪除、新增按鍵；復原與重做；多頁面。
- 按鍵設定：輸出、長按、滑動、切頁；顏色、透明度、文字大小與位置。
- 對齊：格線與吸附預設開，欄數列數可調並存進佈局檔；一鍵切完全自由；按住修飾鍵（手機用雙指）暫時不吸附；拖動時顯示與其他按鍵對齊的參考線。
- 佈局版本：新增直向、橫向、平板版本，可從另一個版本複製；淺色與深色分開預覽。
- 邊編輯邊試打；匯入、匯出佈局檔；從預設佈局開新佈局；「恢復預設佈局」。
- 單手佈局：內建左手、右手兩份預設佈局，用同一套格式，切換鍵放在佈局裡。
- 錯誤級問題擋存檔並指出位置，警告級只提示（§2.6）。
- 數字列（brief §4.4）：M1 只對內建佈局有效（另一份佈局檔 `gboard-quick-number-row.json`，§2.6），使用者匯入或自訂的佈局不受開關影響（使用者 2026-11-05 決定，M1 票 27）。M6 要定自訂佈局怎麼支援：例如編輯器從數字列開與關兩份預設佈局各開新佈局，或格式加「數字列」的頁面或版本條件。
- 不做（之後）：曲線自由手繪、按鍵貼圖；整套外觀主題在 M7 定格式（〈平台里程碑（M7–M11）〉）。
- 完成標準：
  - Playwright 在桌面與行動裝置 viewport 拖動、縮放、旋轉按鍵並讀回座標；開關吸附後座標符合格線或不符合。
  - 匯出再匯入後，佈局完全相同。
  - 做一份「搞怪佈局」（字母亂序、按鍵重疊、多邊形、旋轉），在觸控模式實際打出「門口」。
  - 使用者在電腦和手機瀏覽器試過編輯手感。
- **使用者驗收關卡**：M7 開工前要同時滿足三件事，不滿意的項目先修再驗：
  1. 首選字基準沒有退步。
  2. M1、M3、M4、M5、M6 的功能都有 Playwright 實打紀錄。
  3. 使用者在電腦和手機瀏覽器試用後回覆同意（progress〈偏差與決定〉記一條）。

## 平台里程碑（M7–M11）

每個平台一次接上當時已完成的全部功能。開工時補 §3 對應小節，完成標準都是：在真的宿主程式裡實打 M1、M3、M4 的驗收項目全部通過，鍵盤的外觀與互動照 M5 的網頁樣板，觸控平台的鍵盤原生繪製所有佈局功能（形狀、旋轉、透明度、重疊、深色、橫向），M6 的搞怪佈局也能打字（`AGENTS.md`〈驗收輸入行為〉）。

每個平台另有：
- API key 存系統的安全儲存（Keychain、Android Keystore、Windows Credential Manager、Secret Service）。
- 排除指定 App：拿得到宿主識別的平台才接，拿不到的在設定頁註明。
- 無障礙：按鍵與候選讓 VoiceOver、TalkBack、Narrator 讀得到，實際開啟讀回。
- 觸控平台：震動與按鍵音（可關）。開工先查證系統限制（iOS 是否需要完整取用權限）。
- 觸控平台的實體鍵盤：偵測到實體鍵盤時收起螢幕鍵盤，只留跟隨游標的候選列，按鍵照 brief §4.3。開工先查證平台是否把實體鍵事件交給輸入法。
- 核心給殼的介面先整理（M4 票 31 的 code-review，M7 開工時做，之後各平台照同一份接）：`Snapshot.ascii` 也含英文模式，改名或改成狀態列舉（對外介面，先問使用者）；`Session` 的 `ascii`、`english`、`mode_before_field` 用幾個 bool 表示互斥的模式，改成一個列舉；`set_options` 自己判斷要不要重列，綁定層不再每個 setter 各寫一次 `refresh`；`session.rs`（約 2,600 行）把英文模式的狀態拆到 `session/`；各台次數加墓碑表的 `tally_mut`、`delete`、`digest` 在 `Bigrams`、`SchemeModel`、`EnglishUserWords` 各一份；英文的小寫鍵有四種寫法；`TouchSurface` 在 `qim-databuild`、`qim-bench` 各一份；網頁 `afterHostKey` 由按鍵種類猜核心有沒有學到，改由 Snapshot 回報。M4 回顧補的：鄰鍵讀法的碼與懲罰散在 `ColumnEdge`、`Node`、`NeighborSpan`，`tapped` 與 `key_weights` 成對、wasm 用三條平行陣列（M4 票 09c）；`Output::ToggleAscii` 實際送 `ToggleEnglish`（票 24）；`Session::private()` 與 `TypedRun.private` 同名不同義、`mode_before_field` 是 `(bool, bool)`（票 26）。
- 組字中換資料包（背景線上更新）：`Session::refresh` 只重列候選、不重建 lattice 的欄，多段路徑仍是舊資料包算的；組字中殼改了學習資料又沒呼叫 `refresh` 時開頭各段照舊（M4 票 18b）。平台殼背景更新資料包時先處理。
- 欄位類型的邊角（M4 票 26，網頁沒查證）：使用者在 email、網址欄自己切到英文模式時，點建議仍補空白、建議列仍列拼錯的修正（LatinIME `shouldInsertSpacesAutomatically`）；`inputmode=text` 會不會蓋掉 `type=email` 沒查證（M4 回顧補）；各平台照自己的欄位屬性對應 `FieldKind` 時一起定。
- 桌面平台接實體鍵盤（M4 票 07 留下，網頁驗不到）：ISO 鍵盤的 `Backquote` 與 `IntlBackslash` 在 macOS 可能對調、dead key 換了字元攔下後系統的 dead key 狀態有沒有取消，用系統的實體鍵位實機驗；Dvorak、AZERTY 等真實配列也在這時實打。

- **M7 macOS**：`qim-ffi` 完整版、IMK 殼、使用者資料存檔、設定視窗（含學習資料管理、API key、內嵌佈局編輯器）。定外觀主題格式，鍵盤與桌面候選窗共用一份（參考青簡 `theme.json`：一個主題一個目錄、可繼承、按淺深色給變數、可帶圖片，`docs/research/qingjian.md` §7），接上候選窗；之後各平台殼接上同一份。依 〈M3 的模型階梯實驗〉 的結果接端上 LLM。定同步的傳輸方式（要能跨 Apple 與 Android，內容含自訂佈局；WebDAV 列入候選，CyIME 用它同步剪貼簿與備份）；接剪貼簿歷史與可選的崩潰回報（預設關，brief §8）。
- **M8 iOS**：鍵盤擴充與主 App（設定、學習資料管理、使用者詞匯入匯出、內嵌佈局編輯器，開工時實測內嵌的手感）。開工先查證鍵盤擴充拿不拿得到宿主 App 的識別。鍵盤擴充讀佈局時略過 Schema 驗證（M2 票 09 決定）：佈局在主 App 與編輯器存檔時驗過，擴充只讀，驗一次 footprint 多約 3 MB、執行檔多 3.1 MB（Mac）；殼要讀佈局、`qim-ffi` 加佈局介面時，`qim-layout` 的 `jsonschema` 改成預設開的 feature，關掉時只留型別與引用檢查的讀法，`qim-ffi` 給鍵盤擴充的建置關掉它；`ARCHITECTURE.md`〈依賴規則〉的表不用改（關掉時 `jsonschema` 仍是可選依賴）。另外的完成標準：沒開完整取用權限時打字與學習照常；與 macOS 互相同步一個新使用者詞與一筆剪貼簿並讀回。做完後評估多端檔案傳輸，結論寫回 brief。
- **M9 Android**：InputMethodService 殼與設定 App（內嵌佈局編輯器）。開工先查證 Gemini Nano 能否在輸入法內使用，並讀 CyIME 的實體鍵盤與懸浮鍵盤做法（`docs/research/cyime.md` §4，GPL，只取做法）。做懸浮鍵盤（拖曳、調大小、半透明；iOS 不能畫到鍵盤外，不做）。評估語音輸入，結論寫回 brief（iOS 鍵盤擴充沒有麥克風）。另外的完成標準：與 macOS、iOS 互相同步並讀回。
- **M10 Windows**、**M11 Linux**：平台殼。M10 開工先比較核心直接載入 TSF DLL 與「薄 DLL 加程序外引擎」（`docs/research/xsime.md` §2.1）：前者引擎崩潰會拖垮宿主、x86 宿主要另建，後者多一層 IPC 與冷啟動等待。

M7–M9 開工時另讀 `docs/research/xsime.md` 對應小節，列入任務：
- M7：開工先問使用者粵典詞庫在平台 App 要內建還是可選下載（brief §11），照決定接資料包管理。
- M7：用 `codesign -d -r-` 讀回 Personal Team 簽章的 requirement，重建兩次確認輔助功能授權仍在（§4）；以 root 註冊輸入源、更新前切走輸入源、只換執行檔的開發迴圈（§2.2）。
- M8：需要鍵盤自己放切換鍵的機型，地球鍵長按的切換鍵盤選單是否照 Apple 改成換系統鍵盤，選單裡的系統鍵盤現在只是占位，iOS、Android 殼接系統的清單（M4 票 23，使用者同意）；事實 P25 `deleteBackward()` 刪的單位實機量（M4 票 21，核心只刪單一 scalar 的字，不依賴它）。
- M8：Full Access 只閘控網路與剪貼簿、App Group 只由主 App 寫、Keychain group 寫成固定字串（未簽章的 CI archive 前綴為空）、冷啟動高度、用 `phys_footprint` 量記憶體（§2.3）。
- M9：測試宿主 App 涵蓋一般、密碼、WebView、`TYPE_NULL` 欄位（§2.4）。

## M3 的模型階梯實驗

已細化進 `.scratch/m3/spec.md`〈模型階梯實驗〉（2026-11-11），別處引用「plan 〈M3 的模型階梯實驗〉」的都指那一段。

## M5 觸控鍵盤與設定

使用者 2026-11-12 決定：平台殼開工前，先在網頁把觸控鍵盤做到幾乎完整，使用者在手機瀏覽器確認後平台照做（§3.1）。排在 M4 之後，因為英文鍵盤與英文建議列在 M4；排在佈局編輯器（M6）之前，編輯器建在定稿的鍵盤上。

- 分兩期（使用者 2026-11-17 同意，M4 回顧）：第一期盤點、設計稿、候選列與工具列、設定頁、鍵盤高度、從 M4 來的網頁整理，做完在手機試用；第二期面板、滑動、撤銷學習手勢、句首建議與量詞聯想、英文下一個詞。期界在盤點勾完後定。
- 開工先盤點：Android 模擬器上的 Gboard 實測觸控鍵盤的全部功能（工具列、各面板、長按、手勢、設定頁的分類與項目），另對照 iOS 內建鍵盤，寫到 `docs/research/`；列成清單請使用者逐項勾要不要，勾完才拆票。
- 候選列接在觸控鍵盤頂端（現在在輸入框下方，和鍵盤隔著設定區）；沒有組字時換成工具列（Gboard 做法），放設定、emoji、剪貼簿、文字編輯、常用語等入口。實體鍵盤模式的候選列另外定。
- 鍵盤內設定頁：分類與項目照之後平台 App 的設定，取代頁面下方的勾選框；數字列、深色、字根提示這類開關也搬進去。
- 面板：emoji 面板（分類、最近用過、搜尋）、文字編輯面板（方向鍵、選取、全選、複製、貼上）、剪貼簿（瀏覽器讀不到系統剪貼簿歷史，開工時查限制，做得到的範圍先做）。
- 調整鍵盤高度。
- 從 M4 移來（使用者 2026-11-12 同意，觸控手勢放在重做鍵盤的這個里程碑）：空白鍵滑動移游標、英文滑動輸入（開工讀 FlorisBoard v0.5.2 與 AnySoftKeyboard，`docs/research/facts-products.md`）、評估中文滑動輸入（結論寫回 brief）。滑動的解碼用 `qim-layout` 的按鍵幾何，不寫死 QWERTY；和佈局裡按鍵自己的滑動動作（flick）怎麼分，開工時定，研究建議照青簡的方向扇形（`docs/research/spatial-correction-swipe.md` §5）；形碼的滑行沒有先例，先用模擬軌跡量首選率再決定做不做畫面。
- 從 M4 來的觸控入口（使用者 2026-11-12 同意）：撤銷學習的手勢跟重做的候選列一起定；長按候選目前是字根提示（brief §4.4）。撤銷學習的手勢要能指定任一個候選（`Session::forget` 收任何位置）：桌面只清得到首選，釘在第 2 位的自動造詞要靠它（M4 票 16，§2.4）。簡體模式兩個原文轉成同一個簡體字時候選去重只留第一個，撤銷學習只清得到它（M4 票 16），重做候選列時一起定。觸控鍵盤的 `z` 鍵照 Gboard 標「重」、點了是萬用鍵，盤點 Gboard 時實測它在速成的行為，定要不要改標「＊」（M4 票 11）。
- 評估空輸入的句首建議與數字、標點後的量詞聯想（iOS 速成有，黑箱 T8、T18）：和沒有組字時的工具列搶同一個位置，跟重做的候選列一起定；量詞要另整理資料（M4 票 30）：句首建議可用字級 bigram 的句首（`CLAUSE_START`），dev 切分的句子第一字落在 train 句首最常見 9 字的比例口語 27–33%、維基 14–26%；量詞沒有資料，語料前處理丟掉了含阿拉伯數字的句子，bigram 裡沒有「數字 → 字」，要另整理量詞表或重訓。桌面上兩者都要在沒有組字時開候選窗，數字鍵這時要嘛選候選、要嘛照打，每次聚焦、打數字都會跳出候選窗。
- 評估英文建議列的下一個詞（空白之後就出建議，Gboard 與 LatinIME 都有）：要英文 bigram 資料包、英文的個人 bigram 與空白後的建議列，跟重做的候選列一起定（協調者 2026-11-15 定不在 M4 做，ADR 0008）。
- 盤點時使用者勾的其他項目。平台才有的（按鍵音、震動、語音、懸浮鍵盤）只在設定頁留位置與說明。
- 從 M4 收尾來的網頁整理（M4 票 31）：e2e 各 spec 各抄一份的 helper（`ready`、`ui`、`shown`、`place`、讀面板機率）收成共用模組；用完就 `free()` 的寫法收成一種（`withSnapshot`、`using`、手寫 try/finally 各有）；`undoWindow` 也當兩下空白的計時器，改成通用的名字；debug 面板開著又打過字後，第一次點「無痕」勾選框沒反應（M4 票 16，先寫成會紅的 Playwright 測試）；觸控鍵盤靠標籤 `🌐` 認要畫成圖示的鍵、`touch.ts` 另存一份中英狀態（M4 票 23）；觸控刪除鍵的退格撤銷只在核心驗過（M4 票 18c）；觸控點擊的按鍵機率接進英文修正（現在英文容錯不看鍵位鄰近），接上後重量自動修正的 4 倍門檻（M4 票 25）。M4 回顧補的：無痕時殼不給前文，剛聚焦時不知道游標前有沒有字、換行後不句首自動大寫（M4 票 24）；簡體去重時聯想的 emoji 從第 3 位變第 2 位，併進簡體去重那條（M4 票 30）；地球鍵選單的「點外面收起」只在鍵盤範圍內（M4 票 23）。
- 完成標準：
  - Playwright 用手機 viewport 讀回候選列在鍵盤頂端、工具列與各面板開得了、設定頁改了會生效，截圖存證，平台里程碑拿它當對照。
  - 使用者在手機瀏覽器試用後回覆同意（progress〈偏差與決定〉記一條）。

## 不排里程碑

驗收與回顧時留下、沒有排進任何里程碑的項目，每條一行、附出處。有 repro 或使用者要求再開票。

- 譯詞（打中文時候選旁顯示英文、可上屏，例「發票」旁的 invoice）：使用者暫不需要，2026-11-15 移出 M4。來源、授權與 LLM 生成費用的研究在 `docs/research/translation-sources.md`；要做時重開資料包的票（原 M4 票 29）。出處：M4 票 20。
- plan 縮成根目錄的 `ARCHITECTURE.md`（上限 15,000 字）：已拆成 `.scratch/arch-diet/` 的 6 張票，隨時可以用 `/1morr:ticket-coordinator .scratch/arch-diet/issues` 跑。出處：2026-11-13 流程對齊。
- 容錯重評時要補的：四種修正沒過門檻（M4 票 09），重評時照票 10 列的四項與 `docs/research/tolerance.md` 補；空間模型的「上層加成 2」、σ、鄰鍵先驗 0.9／σ 0.5 沒有來源也沒調；重排開著時觸控集合的修正次數量不了（model-ladder 沒有觸控的路，M4 票 09c）。出處：M4 票 06–10、09c、18。
- e2e 在高負載下偶發紅、單獨重跑會過：`pack-update.spec.ts`「sha256 不符時拒絕，保留舊版」（M4 票 13）、`touch-features` 長按切頁（M4 票 21）、單按 Shift 的計時（M4 票 26）。再紅一次就用 `mattpocock-skills:diagnosing-bugs` 開修復票。出處：M4 票 13、21、26 與票 31。
- e2e 開跑前不檢查資料包建好了沒：網頁要載的資料包少一個（例如主 checkout 沒重跑 `cargo xtask data`），整批測試在等就緒時逾時，看不出原因；要做時讓 e2e 步驟先比對網頁要載的資料包，少了就印出要跑 `cargo xtask data`。再踩到或做 gates 時順手加再開票。出處：M4 回顧（2026-11-17）。
- 簡體模式逐個分段轉換，跨兩段的詞可能轉錯（「乾」「隆」分在兩段時成了「干隆」）；常見詞在詞庫裡本身就是一段，所以罕見。等使用者回報再改：整串一起轉換，再依字數切回各段。出處：M1 票 16。
