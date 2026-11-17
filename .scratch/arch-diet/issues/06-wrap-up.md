# 06: §8 歸位、改引用、刪掉 plan

Status: ready-for-agent
Blocked by: 01, 02, 03, 04, 05
讀: `.scratch/arch-diet/spec.md`；plan 剩下的全部；`AGENTS.md`；`docs/agents/`；roadmap；brief；M4 還沒做的票

## What to build

把 plan §8〈風險與回寫點〉歸位，確認 plan 每一節都只剩標題與指向，然後刪掉 `docs/plan.md`。改引用：`AGENTS.md`、`docs/agents/milestone.md`、`docs/agents/ticket-template.md`、`docs/agents/issue-tracker.md`、roadmap 的「拆票前讀」、brief、ADR、M4 還沒做的票與 spec、程式碼註解裡的 `plan〈…〉`，都改成 `architecture〈節名〉` 或實際的檔。票 02–05 留在 `.scratch/arch-diet/adr-drafts/` 的 ADR 草稿（不編號）和各票 Comments「給使用者決定」的各條（含票 01 的〈識別符〉表要不要搬進 `GLOSSARY.md`），併進使用者試的清單；使用者確認的草稿取下一個 ADR 編號寫進 `docs/adr/`，引用跟著改，再刪掉 `adr-drafts/`。最後把使用者試的清單給使用者，`docs/progress.md` 的下一步寫 `等使用者：讀 ARCHITECTURE.md`。

## 驗收

- [x] `docs/plan.md` 已刪；`ARCHITECTURE.md` 不超過 15,000 字（`python3 scripts/check_docs.py` 會擋）。
- [x] 除了已封存的里程碑，`grep -rn "plan §\|docs/plan.md\|plan〈"` 沒有結果。
- [ ] 使用者讀過 `ARCHITECTURE.md` 並回覆可以（回覆記在 Comments）。
- [x] Comments 有對照表：這張票負責的 plan 每一段 → 新位置或刪掉的理由。
- [x] `cargo xtask check --fast` 綠燈。

## Comments

### 對照表：plan 剩下的每一段 → 新位置

開工時 plan §0–§7 每一節都只剩標題與一行「已搬到 …」（票 01–05 搬的），本票負責的是檔頭與 §8。

| plan 的段 | 新位置或刪掉的理由 |
| --- | --- |
| 檔頭「狀態：M0–M4 完成…」 | 里程碑狀態在 roadmap〈里程碑〉；「只寫現況、決定改了就改寫」在 `ARCHITECTURE.md` 檔頭；「開工時細化成 spec、拆票後以票為準」在 roadmap 檔頭 |
| §0–§7 的「已搬到」行 | 只是指向，隨 plan 刪掉；指向的內容由票 01–05 的對照表交代 |
| §8「調 bigram 的參數後 `mir` 不再選『一台』」 | 刪：`bench/fixed.tsv` 擋（`cargo xtask bench --check` 要固定題全過） |
| §8「口語語料太小」 | roadmap〈不排里程碑〉 |
| §8「Wikimedia 的 XML dump 預告 2027-01 停用」 | roadmap〈不排里程碑〉 |
| §8「使用者的 Rime 學習資料遲遲沒到」 | 刪：已解決，個人集由 `crates/qim-databuild/src/pipeline.rs` 轉成本機的 `bench/personal/`（`docs/development.md`〈個人集〉） |
| §8「iPhone 版 Gboard 沒有速成」 | 刪：已解決，只測 Android 版，結果在 `docs/research/blackbox-baseline.md` |
| §8「空間模型讓正確輸入被錯字蓋過」 | 刪：鄰鍵讀法從第 2 格起插、不搶首選，由 `crates/qim-core/tests/neighbor_readings.rs` 說明；排位在 `ARCHITECTURE.md`〈候選的排位順序〉 |
| §8「小型神經模型放不進鍵盤擴充或太慢」 | 旗艦機的結論在 `docs/research/ios-feasibility.md`；低階機還沒量的部分進 `ARCHITECTURE.md`〈風險與回寫點〉 |
| §8「維基語料授權（CC BY-SA）對模型的要求」 | roadmap〈不排里程碑〉（brief §11 的待決項） |
| §8「iOS 鍵盤擴充放不下核心與資料包」 | `ARCHITECTURE.md`〈風險與回寫點〉（低階機） |
| §8「`jsonschema` 讓程式碼變大」 | 已在 roadmap〈平台里程碑（M7–M11）〉的 M8（鍵盤擴充略過 Schema 驗證） |
| §8「免費佈建不能跑鍵盤擴充」 | 刪：已解決，能跑，事實 P4 |
| §8「延遲超過預算」 | `ARCHITECTURE.md`〈風險與回寫點〉 |
| §8「LLM 重排提升太小或太慢」 | 刪：已決定不做，brief §1.5 與 `docs/research/model-ladder.md` |
| §8「iOS 鍵盤擴充不能用 Foundation Models」 | roadmap M8（端上 LLM 太慢時只用雲端）；量測在事實 P12 |
| §8「瀏覽器呼叫雲端 LLM 被 CORS 擋」 | 已在 roadmap〈不排里程碑〉雲端 LLM 那條（經 Vite 開發伺服器轉發，同 `web/vite.config.ts` 的 `/pack-updates/`） |
| §8「內嵌網頁編輯器在手機主 App 手感不佳」 | roadmap〈平台里程碑（M7–M11）〉每個平台都有的那串 |
| §8「iOS 鍵盤擴充收不到實體鍵事件」 | roadmap〈平台里程碑（M7–M11）〉觸控平台的實體鍵盤那條 |
| §8「網頁驗不到的平台差異」 | `ARCHITECTURE.md`〈風險與回寫點〉 |

### 引用

- 約 580 處 `plan` 引用照 plan 節 → 新位置的對照改寫（程式註解 330 處在 `crates/qim-core/`、230 處在其他 crate 與 `web/`、`data/`、`scripts/`，文件 90 處）；新位置不存在的（例如〈資料流〉，程式檔本身就是說明）拿掉引用。
- 沒改、仍會被 `grep "plan §\|docs/plan.md\|plan〈"` 找到的，都不是指向 plan 的引用：`.scratch/arch-diet/` 自己的票、證據表與 ADR 草稿的出處行（草稿使用者確認後重寫檔頭再搬走）；`scripts/check_docs.py` 與 `scripts/tests/test_check_docs.py` 裡規則的說明與測試資料（範本的通用規則，`plan §3.2` 是故意要擋的寫法）。
- 研究筆記裡別的專案的 `docs/plan/…` 路徑（青簡）改寫成「`docs/` 下的 `plan/…`」，指的檔不變。

### ADR 草稿的整理（協調者指示）

- `ranking-parameters`、`lexicon-and-model-tuning` 只記調參與量到的數字，改放 `docs/research/ranking-parameters.md`、`docs/research/lexicon-and-model-tuning.md`，不當 ADR。
- `pack-format-and-manifest` 與 `pack-online-update` 同是資料包，併成 `pack-format-manifest-and-update`。
- 剩 11 份，列在下面的清單，等使用者確認才編號；`adr-drafts/` 留著。

### 給使用者的清單

每題：背景、選項、建議。回「照建議」只算同意這裡列的題。

**一、讀 `ARCHITECTURE.md`**（約 13,800 字，上限 15,000）

從頭讀一遍，回覆看不懂的段落、覺得不該刪的內容。改完才算 arch-diet 驗收。

**二、11 份 ADR 草稿**（在 `.scratch/arch-diet/adr-drafts/`，內容是 plan 原有的理由，沒加新決定）

背景：架構決定的理由原本散在 plan 裡，搬出來寫成 ADR（架構決定紀錄，`docs/adr/` 已有 0001–0008）。選項：每份「編號寫進 `docs/adr/`」「改放研究筆記」「不要」。建議：全部依下面的順序編成 ADR 0009–0019；不決定的話 `adr-drafts/` 一直留在 `.scratch/`，程式註解只能指到 `ARCHITECTURE.md` 或程式檔。

1. `preceding-text-from-shell-then-own-commits`：前文以殼給的為準，沒給時用自己上屏的字。
2. `field-kinds`：欄位類型由殼聚焦時給；email、網址、密碼欄預設英文直通。
3. `key-probabilities`：按鍵機率分觸控與實體鍵盤兩種，實體鍵盤用鍵位先驗。
4. `association`：聯想由詞庫與 bigram 現算，數字鍵選、空白不選。
5. `rerank-by-paths-after-rules`：停鍵重排排 lattice 的路徑，排在排位規則排定的候選後面。
6. `userdict-text-format`：學習資料匯出成帶版本的 TSV。
7. `learning-expiry-deactivates`：學習一年沒用就停用、不刪。
8. `phrases-and-manual-words`：手動加詞與常用語分開。
9. `new-schemes-share-models`：加第二個輸入方案時模型共用、碼表各一份（現在只有速成，記的是打算）。
10. `pack-format-manifest-and-update`：資料包整份解開進記憶體、manifest 放檔外、經 GitHub Releases 線上更新。
11. `data-sources-and-licenses`：授權鏈不清的來源不用，非商業的只進可選資料包。

**三、〈識別符〉表要不要搬進 `GLOSSARY.md`**

背景：`ARCHITECTURE.md`〈模組地圖〉底下有一張名詞 ↔ 程式識別符的對照表，約 3,300 字，佔全檔四分之一。選項：(a) 搬進 `GLOSSARY.md` 各詞條，`ARCHITECTURE.md` 空出字數；(b) 留在原處。建議：(a)，名詞表本來就是查詞的地方，`ARCHITECTURE.md` 留給分工與契約。不決定的話 `ARCHITECTURE.md` 只剩約 1,200 字可加。

**四、照 repo 現況寫、沒照 plan 原句的地方**（`AGENTS.md`〈事實與查證〉：現況以 repo 為準）

背景：plan 和程式碼對不上時，`ARCHITECTURE.md` 與 `docs/development.md` 照程式碼寫，plan 原句在 `git show 4832a21:docs/plan.md`。選項：接受，或指出要以 plan 為準的那幾條（改程式另開票）。建議：全部接受。

- 模組地圖：沒有 `web/editor/` 與 `data/` 的佈局檔（預設佈局由 `qim-databuild` 算出），拿掉；補上 `qim-neural`。〈測試策略〉也不提不存在的 `data/layouts/`。ADR 0004 說 `web/editor/` 隨產品發佈：建議 M6 開工時照 ADR 做，不另寫。
- 刪掉的過時句：核心「一直送 0 個 `delete_before`」（現在會送）、資料流圖（缺詞級 trigram 與聯想、去重的位置畫錯）、方案檔範例（β、γ 寫 0.0，實際 1.0）、中英鍵轉成 `ToggleAscii`（實際是 `ToggleEnglish`）。
- `qim-ffi` 還沒有 `set_options`：〈核心對殼〉只寫核心的規則，綁定怎麼包留給加這個介面的票。
- 「門口」的檢查直接查詞庫、不經 `Engine`（`verify.rs`）；建置的決定性檢查有 11 份資料包（含停鍵層模型），不是 10 份。
- 首選字基準的「原樣讀法」是預設選項再關掉鄰鍵讀法（`docs/development.md` 已照程式改）。
- 小出入：佈局範例的畫布高 216（實際 224）、網頁另在 `blur` 給今天的日期、匯出檔的裝置欄是 `#@/user_id`。

**五、刪掉的理由指到文件、不是測試或程式檔的段落**

背景：spec 規定刪掉一段只能以「測試或程式檔已說明」為理由；有些段落其實已在研究筆記、事實、brief、ADR、`data/sources.toml` 或 `AGENTS.md`（例如「核心走 tdd」）裡，就刪了；有幾句歷史（「推翻 M1」「當時只授權推 `main`」）直接刪；§8 已解決的風險指到結果所在的檔。選項：接受，或指出要留的段落。建議：接受，這些內容在原處就有。

**六、搬到 roadmap 的打算**

背景：只和還沒開工的里程碑有關的段落搬進 roadmap（spec 的四個去處沒有 roadmap）：鍵盤擴充略過 Schema 驗證、iOS 預設佈局放不放地球鍵、Foundation Models 太慢的備案、實體鍵事件與內嵌編輯器的備案（M8 與平台通則）；口語語料、維基 dump、維基授權、簡繁逐段轉換的已知限制、`bench/personal/sentences.txt`（從沒做過）進〈不排里程碑〉；空輸入句首建議的數字併進 M5。選項：接受，或指定別的去處。建議：接受。

**七、沒放進 `ARCHITECTURE.md`、留在程式碼或 dev 的**

背景：分不清是契約還是實作細節、受預算所限沒放進去的。選項：維持，或指定要搬進 `ARCHITECTURE.md` 的（剩約 1,200 字）。建議：都維持。

- manifest 各欄位的格式（`crates/qim-core/src/manifest.rs`）、網頁存資料包的規則（`web/src/packs.ts`）。
- 首選字基準的細節：個人集的偏差、依長度分組與翻盤清單、回報的案例加進 `fixed.tsv`、錯誤模型故意和容錯模型不同（`docs/development.md`〈首選字基準〉）。
- 英文模式 Enter 自動修正時各平台怎麼先上屏再交還 Enter：各平台里程碑開工時寫進它的 spec。
- 停鍵多久才重排：網頁 80 ms（`web/src/rerank.ts`），各平台開工時照實測定，不寫進契約。
- 〈主要依賴〉的「理由」欄留在表裡，不拆成 ADR。
- 欄位類型的輸入框用 `textarea` 的理由、「新的排序原因要在 `Inspection` 與 debug 面板加一欄」這個慣例：建議分別補進 `web/src/field.ts` 的註解與 `docs/agents/` 的票慣例，另開小票。

**八、過時的程式註解與一處沒有測試的行為**

背景：spec 只准改指向文件的註解，下面幾處內容過時沒改：`Session::forget` 的註解讀起來像清掉所有前文後面的個人 bigram；`user/format.rs` 的 `HEADER` 還寫「M4 收尾前還沒定稿」；`word_trigram.rs` 的 `LARGE` 寫「常駐估計 35 MB」（iPhone 實測留下 +23.4 MB、峰值 +71.3 MB）；`verify.rs` 的 `CheckId::Decodes` 漏了詞級 trigram 的檢查；「舊核心讀到不認得的資料包種類拒絕載入」靠 serde、沒有測試。選項：開一張修復票，或放 roadmap〈不排里程碑〉。建議：開一張修復票，M5 開工前做。

**九、要不要重量 `bench --check` 的時間**

背景：plan 寫 `bench --check` 約 1 分鐘，`docs/development.md` 寫整個 `cargo xtask bench` 約 3 分鐘（加了學過的學習資料之後）；只留了 3 分鐘。選項：重量一次再寫，或維持。建議：維持，下次有人跑時順手核對。

code-review：Standards 指出的兩處斷掉的引用已修（`lexicon-and-model-tuning.md` 的 brief 節、`properties.rs` 三處）；其餘指錯節的判斷題（`session.rs` 聯想鍵、`english.rs` 等指〈資料包〉、brief〈來源〉表的 kGradeLevel 與 emoji 指向、`user.rs`「（初值）」、〈風險與回寫點〉與〈主要依賴〉的 1.4M 數字重複、〈平台殼〉沒寫上屏後給前文）未處理，留給下一個 session；Spec 軸審查在額度用完前沒回報。
Session 紀錄：2026-11-18 plan §8 歸位到 `ARCHITECTURE.md`〈風險與回寫點〉與 roadmap，約 580 處 plan 引用改指新位置，刪掉 `docs/plan.md`，ADR 草稿整理成 11 份待確認，使用者清單寫在本票
決定：§8 只留會改到分工或格式、還沒排除的四條在 `ARCHITECTURE.md`，平台相關的進 roadmap 各里程碑，已解決的指到結果所在的檔（Agent 定）
決定：〈核心對殼〉與〈平台殼〉重複的殼呼叫時機只留在〈平台殼〉，〈核心對殼〉只寫 API 的意思（Agent 定）
決定：〈學習資料的格式〉補一句清空與還原都留清空記號、合併時傳到別台（票 03 留給本票看預算）（Agent 定）
決定：`scripts/check_docs.py` 與它的測試裡的 `plan §` 是範本規則的測試資料，不改（Agent 定）
決定：調參的兩份草稿改放 `docs/research/`、資料包兩份草稿合併，照協調者指示（Agent 定）
決定：roadmap 已驗收里程碑那幾列「拆票前讀」裡的 plan 節號拿掉，不換成新位置（Agent 定）
變更：無
等使用者：讀 `ARCHITECTURE.md` 並回覆；確認上面清單的二到九題（11 份 ADR 草稿、識別符表、照現況寫的地方等）
