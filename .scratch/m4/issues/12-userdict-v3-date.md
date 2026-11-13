# 12: 學習資料 v3 格式與日期

Status: done
Blocked by: None (can start immediately)
讀: plan §1.3、§2.4（tick、匯出格式）；spec〈細節〉（學習時限與上限、同步）；brief §4.5、§5.3、§7；`docs/research/learning-forgetting-sync.md`；`docs/research/userdict-text-formats.md`；spec〈共同規則〉的對外格式

## What to build

學習資料的時限、英文使用者詞與同步都要新的欄位與列種類，v2 已定稿，所以先升 v3，把整條路接通：

- 殼交給核心今天的日期（介面由票定，例如 `Session` 或 `UserModel` 的方法）；核心不讀系統時間（plan §1.3）。每個條目記最後使用的日期。
- 匯出升 `# qim-userdict v3`：每列有明確的種類欄（未知種類略過並警告，已知種類照舊嚴格檢查）與最後使用的日期；檔頭記裝置 id，由網頁產生並存起來。
- 匯入照收 v1、v2；沒有日期的列取匯入當天，或其他做法，由票定並寫明理由。匯入暫時照舊取代整份，改成合併在票 17。
- 先對照 librime userdb、LatinIME 的做法（研究筆記已有），採用誰的慣例寫在 Comments。

v3 在 M4 收尾前還沒發佈，之後的票可以往裡加列的種類與欄位，收尾票定稿。

## 驗收

- [x] 依賴規則守著核心不讀系統時間；測試直接給日期。
- [x] 匯出 v3 再匯入等價（性質測試）；v1、v2 的檔仍匯入得了；未知種類的列略過並回警告，其他列照收。
- [x] Playwright：匯出的檔有 v3 檔頭、裝置 id 與日期欄；換一天（測試掛鉤給日期）上屏後日期跟著變。
- [x] plan §2.4 的匯出格式改寫成 v3 草稿；`cargo xtask check` 綠燈。

## Comments

- 2026-11-12 實作（開工 `1cf2f1c`）：
  - 介面：殼用 `UserModel::set_today(Date)` 交今天（`qim_core::Date`，ISO 8601 `YYYY-MM-DD`，0001–9999 年，只到日、沒有預設值）、`set_device(&str)` 交裝置 id。`UserModel::import(text, today: Option<Date>) -> Imported { user, warnings }` 把檔讀成一份新的個人模型（今天是 `today`、裝置 id 是檔裡的）；殼取代用 `replace_learning`，今天與本機的裝置 id 留著。票 17 改成合併時加 `merge`，交換律、結合律、冪等放在它身上，`import` 仍只負責讀檔。
  - 採用的慣例：librime userdb 文字檔的 metadata 行 `#@/key<Tab>value`（裝置 id 對應它的 `/user_id`，網頁照它每次安裝一個 id，用 `crypto.randomUUID()` 存 `localStorage` 的 `qim.device`）；LatinIME 每個條目存最後使用的時間（300 天 GC、淘汰先刪最舊的，`docs/research/learning-forgetting-sync.md` §2.2–§2.3），這裡只記到日（一年時限只要日，也少洩漏打字的時刻）。兩者都沒有種類欄：librime 靠 parser 略過壞列相容，這裡改成未知種類略過並警告、已知種類照舊嚴格（spec 的決定）。
  - v3 的列：`entry`（11 欄，原本的 9 欄加種類與 `last_used`）、`bigram`（6 欄）、`string`（5 欄）；日期欄放在最後 tick 之後，時間相關的欄放一起。格式全文在 plan §2.4。
  - 沒有日期的列（v1、v2）取匯入那天：那是使用者正在搬過來的資料，給它一年，票 13 的時限才不會一升級就清掉；改用「很久以前」會丟資料，從 tick 推日期沒有根據（tick 是上屏次數）。殼還沒給日期時（`today` 是 `None`）留空。
  - 協調者要求（Heads up）：殼沒給日期時不寫假日期。`Date` 不給 `Default`，今天與每個 `last_used` 都是 `Option<Date>`，匯出時日期欄留空、匯入讀回 `None`，票 13 的時限跳過它們；plan §2.4 寫明殼的日期契約（第一次學習前、之後每次使用者操作前給）；`learning_before_the_shell_gives_a_date_writes_no_date` 守「沒設日期不會寫出 1970」。沒給日期時再用到一個有日期的條目，日期蓋成 `None`，沒有沿用舊的：沿用的話，殼之後給了真的日期，那個其實剛用過的條目可能被當成一年沒用而清掉；寧可讓它不過期。
  - 同一組（方案, 輸入碼, 詞）的計數與使用者詞是匯出檔的同一列，只有一個日期：學到或改到時兩者一起蓋（`SchemeModel::stamp`）。所以手動加一個已經學過的詞，計數的日期也跟著變成今天。
  - 網頁在 `window` 捕獲 `keydown`、`pointerdown`、`blur` 時交日期：每條學習路徑（實體鍵、點候選、觸控、失焦記住 email）都在其中之一之前；載入時、匯入存檔之前也給一次。`page.clock` 是 Playwright 換日的掛鉤，不另加測試專用的程式碼。
  - 依賴規則：`xtask` 的 `deps` 原本就禁 `std::time::SystemTime`（含變異測試），核心沒有其他讀牆鐘的路，這張票只改 plan §1.3 的說明字樣與它的變異測試。
- 驗證：`cargo xtask check` 全部步驟通過（fmt、clippy、test、deps、docs、wasm、tsc、biome、unit、e2e：Playwright 226 passed、2 skipped，skipped 是原本就有的）。中間一次全套有一例 `key-table.spec.ts` 在 `beforeEach` 等網頁載入逾時（當時另開著手動測試的 dev server），單獨重跑該檔 3 輪 57 例全過、全套重跑通過。新測試：`tests/date.rs`、`tests/user_dict.rs`（v3 格式、換日、未知種類、v1/v2、裝置 id、沒給日期、`replace_learning`）、`properties.rs` 的匯出再匯入等價加了中途換日與裝置 id、`web/test/today.test.ts`、`e2e/user-dict.spec.ts` 兩例、`e2e/userdict-date.spec.ts`（失焦、觸控）。失焦那例先紅（記成前一天）才修；拿掉 `pointerdown`、拿掉全部捕獲監聽時對應的 e2e 會紅（變異驗證過）。
- 亂按一輪：dev server 上亂按 300 鍵（組字中夾 Shift、Esc、Enter、方向鍵、Tab、標點，中途換日兩次、開關無痕），存檔有 v3 檔頭、裝置 id，日期跟著時鐘變；沒有新的 console 錯誤（只有本機沒有線上更新索引 `pack-updates/index.json` 的 404，原本就有）。截圖 `.scratch/m4/evidence/12-userdict-v3-random.png`。
- `bench --check`（`build/packs` 從同一個 base commit 的另一個 worktree 複製）：`bench 通過`，固定題 84/84，13 個集合都等於 baseline、沒有翻盤（有前文給／不給答錯 1524／1568、修正 5573／5651，同 M3 票 23）。這張票沒改打分，日期還沒有用在排序。`--learning` 學習開的修正 yue-wiki 3216、zh-wiki 3578、common-voice-yue 3108、hkcancor 1719，和 M3 票 23 記的逐項相同。每鍵延遲（原生 p95）全部 95.75 µs、第 20 碼起 104.21 µs、選開頭一段 1646.25 µs；票 05 的閘門還沒接上，只報。本機沒有個人集。
- code-review（Standards、Spec 兩軸）處理了：失焦沒有前導按鍵時日期記成前一天（加 `blur`，先寫紅測試）；匯入、清空時留住今天與裝置 id 的規則從 wasm 綁定搬進核心 `replace_learning`（平台殼漏做時票 17 會記錯格）；觸控換日補 e2e；`Kind::String` 改名 `Kind::Remembered`、`deviceId()` 改名 `ensureDeviceId()`、`for_scheme_mut` 收成私有；名詞表與 plan §0 補「最後使用的日期」「裝置 id」。沒處理、留給後面的票：
  - 種類名 `string`（記住的字串）：票 14 換成英文使用者詞時一起改名，收尾票定稿前再看名詞表要不要收「列的種類」。
  - `#@/device`、`#@/tick` 行夾在資料列中間也收：收尾票定稿 v3 時決定要不要限定在檔頭。
  - 三個 parser 拆日期欄的形狀重複、`(version, today)` 一起傳、裝置 id 用 `String` 而非 newtype、`UserModel` 同時帶殼給的今天：判斷題，票 13、17 再加欄位時如果再重複一次就收成 helper 或 newtype。

Session 紀錄：殼給日期（`set_today`，沒給不寫假日期）、匯出 v3（種類欄、`last_used`、裝置 id）、v1/v2 照收；plan §2.4 改成 v3 草稿
