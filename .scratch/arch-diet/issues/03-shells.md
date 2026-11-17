# 03: plan §3〈平台殼〉歸位

Status: done
Blocked by: 01
讀: `.scratch/arch-diet/spec.md`；plan §3 全文；roadmap〈平台里程碑（M7–M11）〉

## What to build

§3 寫的是網頁殼的現況與各平台殼要守的事。殼與核心的分工、各平台都要守的契約留在 `ARCHITECTURE.md`；只和某個還沒開工的平台有關的打算，搬到 roadmap 那個里程碑的小節。

## 驗收

- [x] §3 的內容已歸位，plan 這一節只剩標題與一行指向。
- [x] Comments 有對照表：這張票負責的 plan 每一段 → 新位置或刪掉的理由。
- [x] `cargo xtask check --fast` 綠燈。

## Comments

### 對照表（plan §3 → 新位置或刪掉的理由）

| plan 的段落 | 去處 |
| --- | --- |
| §3.1「目的：手感驗證核心與佈局…網頁 UI 不隨產品發佈…觸控鍵盤是平台殼照做的樣板」 | 「不發佈」「平台殼照做的樣板」：`ARCHITECTURE.md`〈平台殼〉第一句；決定的來由已在 roadmap〈M5 觸控鍵盤與設定〉第一段；「只有 `web/editor/` 隨產品發佈」：不搬，ADR 0004 寫著（給使用者第 1 條） |
| §3.1「限制：只是測試頁…區網開在手機瀏覽器試…」 | `docs/development.md`〈網頁〉新加的「在手機上試手感」一段（補了指令與 `web/e2e/insecure-context.spec.ts`） |
| §3.1「兩種模式：實體鍵盤…觸控模擬…按鍵幾何由 `qim-layout` 算，網頁只畫」 | 兩種模式：刪，`web/e2e/desktop-mode.spec.ts`、`web/e2e/touch.spec.ts` 說明；幾何與點擊判定：`ARCHITECTURE.md`〈依賴規則〉`web/` 列已寫「不可重寫…點擊判定」，`web/src/touch.ts` 的 `Layout.hit` 註解說明（註解改指 `architecture〈依賴規則〉`） |
| §3.1「候選列、展開面板、無痕開關、簡繁開關；按鍵放大預覽、深色、直向與橫向」 | 刪：`web/src/candidate-bar.ts`、`web/src/panel.ts`、`web/e2e/touch-features.spec.ts`（放大預覽、深色、橫向）、`web/e2e/simplified.spec.ts`、`web/e2e/context.spec.ts`（無痕）說明 |
| §3.1「設定區…由 `settings.ts` 的一張表產生…畫面與觸控鍵盤的開關留在候選列下方」 | 刪：`web/src/settings.ts` 開頭註解（`ROWS` 加一列、之後的設定各加一段）、`web/index.html`（`#settings` 與工具列的深色、數字列）、`web/e2e/settings.spec.ts` 說明 |
| §3.1「資料包：設定區的『資料包』一段…」 | 刪：`web/src/packs.ts`、`web/src/pack-update.ts`、`web/src/pack-settings.ts` 的開頭註解與 `web/e2e/packs.spec.ts`、`web/e2e/pack-update.spec.ts` 說明 |
| §3.1「展開面板：…↓ 打開、↑ 收起；Microsoft 速成…選了字就收起」 | 刪：`web/src/panel.ts`、`web/e2e/candidate-panel.spec.ts`（↓↑、組字結束收起）、`web/e2e/radical-hint.spec.ts`（Tab）、`web/e2e/edit-segments.spec.ts`（→）說明；Microsoft 的對照是事實 M18（`docs/research/facts-products.md`），`web/src/main.ts` 的註解改指它 |
| §3.1「匯入詞庫…」 | 刪：`web/src/word-list.ts`、`web/e2e/word-list.spec.ts` 說明 |
| §3.1「常用語…照 Gboard 工具列…微信輸入法的常用語面板」 | 刪：`web/src/phrases.ts` 開頭註解（含兩個參照）、`web/e2e/phrases.spec.ts` 說明 |
| §3.1「英文使用者詞…交還瀏覽器的空白、標點、Enter、Tab 與失焦之後也存」 | 存檔時機：`ARCHITECTURE.md`〈平台殼〉「存學習資料」；畫面：刪，`web/src/english-user-words.ts`、`web/e2e/english-user-words.spec.ts` 說明 |
| §3.1「欄位類型…換選項等於聚焦新的欄位…輸入框仍是同一個 `textarea`…」 | 換欄位的順序：`ARCHITECTURE.md`〈平台殼〉「換欄位」；選項與對應：刪，`web/src/field.ts`、`web/e2e/field-types.spec.ts` 說明；「`type=email`、`type=number` 的 `input` 沒有游標 API，所以用 `textarea`」：程式裡沒寫，給使用者第 2 條 |
| §3.1「上下文…聚焦、`input`、`selectionchange`…每個實體鍵送進核心前…無痕時不給，改完設定後重給」 | `ARCHITECTURE.md`〈平台殼〉「前文」；細節（`contextLimit()`、改完設定重給）刪，`web/src/context.ts`、`web/src/main.ts`（送鍵前與 `settings.connect` 的註解）、`web/e2e/context.spec.ts` 說明 |
| §3.1「候選列左邊顯示中英狀態；『重排』選單…兩個假實作…『組字中單按 Shift』選單」 | 刪：`web/src/candidate-bar.ts`、`web/src/rerank.ts` 的 `RerankMode` 註解、`web/src/keys.ts` 的 `shiftSwitch`、`web/e2e/rerank.spec.ts` 說明 |
| §3.1「停鍵層：…80 ms…同時只有一個在算…Web Worker…主執行緒先建一次 `Reranker` 核對…A 晚失敗不報…`setRerankerId`…candle 多 1.84 MB…p95 27 ms」 | 各殼要守的（80 ms、背景執行緒、同時一個、`apply_rerank`、先核對、沒模型不重排、`set_reranker_id`）：`ARCHITECTURE.md`〈平台殼〉「停鍵層」；網頁的做法刪，`web/src/rerank.ts`（`STOP_MS`、`rerankScheduler`）、`web/src/neural-rerank.ts`、`web/src/rerank-worker.ts`、`web/test/neural-rerank.test.ts`（A→B 晚失敗）、`web/e2e/neural-rerank.spec.ts` 說明；數字刪，事實 L8（`docs/research/facts-model-ladder.md`）與量法 `web/e2e/rerank-latency.spec.ts` 說明 |
| §3.1「debug 面板…不進 `Snapshot`，`qim-ffi` 沒有…之後的票加了新的排序原因，在 `Inspection` 與面板加一欄」 | 「只給網頁」：`ARCHITECTURE.md`〈平台殼〉最後一條；面板內容刪，`web/src/debug-panel.ts`、`crates/qim-core/src/session/inspect.rs`、`web/e2e/debug-panel.spec.ts`（`data-inspect-count`）說明；「新原因加一欄」的慣例：程式裡沒寫，給使用者第 3 條 |
| §3.1「撤銷學習：Shift+Delete 或 Ctrl+Delete…」 | 刪：`web/e2e/forget.spec.ts` 開頭註解與測試說明 |
| §3.1「使用者資料存 `localStorage`…存不下時只留在記憶體…匯入是合併…不用原生對話框…從備份還原…清空也同步」 | 各殼要守的（`replace_learning`、匯入先預覽再 `merge`、存不了照樣打字並告知）：`ARCHITECTURE.md`〈平台殼〉「存學習資料」；網頁的做法刪，`web/src/storage.ts`、`web/src/user-dict.ts`（不用原生對話框、從備份還原留清空記號）、`web/e2e/storage.spec.ts`、`web/e2e/user-dict.spec.ts`、`web/e2e/sync.spec.ts` 說明；「從備份還原」「清空」在殼上怎麼接沒進〈平台殼〉（預算），給使用者第 4 條 |
| §3.1「LLM：M1 只有介面加假實作；M3 沒讓雲端 LLM 過門檻…端上 LLM 到平台殼才接…雲端重評後 API key 只放開發機環境變數」 | 假實作刪，`web/src/rerank.ts`、`web/e2e/rerank.spec.ts` 說明；雲端不做的結論已在 `docs/research/facts-model-ladder.md`；端上 LLM 已在 roadmap M7「依〈M3 的模型階梯實驗〉的結果接端上 LLM」；重評後網頁的 key 做法：roadmap〈不排里程碑〉新加一條 |
| §3.1「驗證：Playwright…`page.keyboard.type`…行動裝置 viewport 加 `tap`…截圖」 | 刪：`web/playwright.config.ts` 與 `web/e2e/*.spec.ts`（例如 `association.spec.ts`、`touch-features.spec.ts`）說明；plan §5.4 的同一句由票 05 搬 |
| §3.2「InputMethodKit，Swift，經 `qim-ffi`…Application Support…`orca computer` 在『文字編輯』與一個瀏覽器…本機簽章」 | roadmap〈平台里程碑（M7–M11）〉的 M7，併進原本的「IMK 殼、使用者資料存檔」與句尾 |
| §3.3「iOS…Linux 各在里程碑開頭補一節：框架、安裝與註冊、怎麼接核心、怎麼實跑驗證」 | roadmap〈平台里程碑（M7–M11）〉開頭：框架與怎麼接核心補在 `ARCHITECTURE.md`〈平台殼〉，安裝、註冊與實跑驗證寫進 `docs/development.md` |
| §3.3「iOS 的硬限制見 `docs/research/facts-platforms.md`」 | roadmap M8 |

引用跟著改：roadmap 兩處指 §3、§3.1 的、`docs/development.md` 三處 `plan §3.1`、brief 一處、`GLOSSARY.md` 一處（不在 spec 列的檔，只換指向，見決定）、程式註解 25 處 `plan〈網頁 prototype〉`（只拿掉這一段，同一處的 `plan〈模型分層〉` 等留給票 02、04）。plan 其他節裡指向 §3.1 的（§2.1、§2.7、§4.6、§4.7）留給搬那些節的票或票 06。

### 給使用者決定（spec 共同規則 4；協調者給的 `ARCHITECTURE.md` 淨增預算 450 字，這張票用了 373 字）

1. ADR 0004 寫「`web/editor/` 隨產品發佈」，但 repo 沒有 `web/editor/`（票 01 第 3 條）。§3.1 這句沒搬，以 ADR 為準。建議：M6 開工時照 ADR 做，不另寫。
2. 欄位類型的輸入框為什麼用 `textarea` 不用 `input`（`type=email`、`type=number` 的 `input` 沒有 `selectionStart`、`setRangeText`，上屏的路徑用不了）沒有寫在程式裡。建議：補進 `web/src/field.ts` 的開頭註解（spec 只准改指向文件的註解，這行是票 02 也在改的，所以沒動）。不決定的話，這個理由只留在本票。
3. 「之後的票加了新的排序原因，在 `Inspection` 與 debug 面板加一欄，測試用面板讀回原因」是做票的慣例，不是架構。建議：寫進 `docs/agents/` 的票慣例或交給票 05 的〈測試策略〉。不決定的話只留在本票。
4. 殼上「從備份還原」用 `UserModel::restore`（取代並留清空記號）、「清空學習資料」也同步到別台，沒寫進〈平台殼〉（預算）。現在由 `web/src/user-dict.ts` 與 `UserModel` 的文件註解說明。建議：票 06 看剩下的字數再決定要不要加一句。

### Code review

- Session 紀錄：2026-11-18 plan §3 歸位：各殼要守的契約進 `ARCHITECTURE.md`〈平台殼〉，macOS 與之後平台的打算進 roadmap，手機試用步驟進 `docs/development.md`，其餘以 `web/` 的檔說明刪掉
- 決定：`ARCHITECTURE.md` 新開〈平台殼〉一節放在〈跨模組的資料與契約〉之後，只寫各殼都要守的五條（Agent 定）
- 決定：網頁接雲端 LLM 時的 key 做法是假設性的打算，放 roadmap〈不排里程碑〉，不進 `ARCHITECTURE.md`（Agent 定）
- 決定：`GLOSSARY.md` 裡指向 plan §3.1 的一處換成 `web/src/debug-panel.ts`，只換指向不改詞條（Agent 定）
- 決定：程式註解的 `plan〈網頁 prototype〉`，契約相關的改指 `architecture〈平台殼〉`，其餘直接拿掉（檔本身就是說明）（Agent 定）
- 變更：無
