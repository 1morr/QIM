# 05: 網頁接上下文

Status: done
Blocked by: 01, 04
讀: plan §2.1（上下文、數字後半形）、§3.1；brief §8、§12.3（iOS 上下文長度）；spec〈範圍〉〈共同規則〉；`.scratch/m2/issues/12-m2-close.md` Comments 的「M3」各條（上下文那條）

## What to build

在網頁的輸入框打字時，候選照游標前的文字變。

- 網頁把輸入框游標前最多 200 字傳給 `set_context`（聚焦、移動游標、上屏之後）；無痕時不傳。
- 數字後半形改看前文（plan §2.1：「M3 有 `set_context` 後改看前文」），`Session` 記最近上屏字的那份狀態要不要留由票定，寫回 plan §2.1。
- 殼沒給前文時要不要用這個輸入框上屏過的字補（iOS 常只讀到一兩句）：先查成熟做法（例如 fcitx5 / libime 在宿主不給 surrounding text 時怎麼辦）再定，寫回 plan §2.1 並附來源。

## 驗收

- [x] Playwright：輸入框先有一段前文，打同一串碼讀回首選和沒有前文時不同；把游標移到別處後候選跟著變；截圖。
- [x] Playwright：前文以半形數字結尾時 `.` 上屏半形；無痕時前文不影響候選。
- [x] 「殼沒給前文」的決定與來源寫在 plan §2.1。
- [x] 實際操作亂按一輪；`cargo xtask check` 綠燈。

## Comments

### 2026-11-11 實作（開工 `936a7cf`）

**核心：一份「游標前的文字」取代 `after_digit`。** `Session` 記最多 `CONTEXT_LIMIT`（200，brief §8）字：`set_context` 取代它；上屏的字、交還宿主的 `Letter` / `Punct` / `Digit` / `Space` 接在後面；交還宿主的退格、Enter、方向鍵、`Other` 等讓它作廢（不知道前文，用 unigram）；`Modifier` 不動；打開無痕時丟掉殼給過的前文。bigram 的前一字與數字後半形都看它，`Session` 原本記「最近送出去的字是不是數字」的布林拿掉（票要定的「那份狀態要不要留」：不留，併進這份文字）。寫回 plan §2.1、§2.3。

**殼沒給前文時：用這個輸入框上屏過的字補（決定）。** 查了三家（子代理讀原始碼，摘進 brief §12.2、§12.4，plan §2.1 附連結）：
- librime（`c95c0d2`）根本不讀宿主文字：組字從頭開始時前文是 commit history 的最後一筆（`GetPrecedingText`），commit history 記上屏與沒處理的可見 ASCII 鍵、最多 20 筆、退格與 Enter 清空；數字後半形也只看它。
- fcitx5-chinese-addons（`e8d930f`）的標點模組平時靠自己記的狀態，宿主有 SurroundingText 時在聚焦後校正一次。
- AOSP LatinIME（`127336e`）宿主回 null 就當空前文，不用自己的歷史。
QIM 合起來：殼給的優先、每次給都取代，沒給時靠自己記的；和 librime 不同，方向鍵也作廢（游標已移走，維持 M1 票 26 的判斷）。不學 LatinIME：bigram 只看最後一字，自己上屏的字就是游標前的字。

**網頁（`web/src/context.ts`）**：聚焦、`input`、`selectionchange` 時給游標前最多 `contextLimit()` 字；無痕時不給；設定區改了開關後重給（關掉無痕要立刻有前文）。另外每個實體鍵送進核心前（沒有組字時）重給一次：交還瀏覽器的鍵讓核心作廢前文，游標沒動（結尾按 End）時沒有 `selectionchange` 補給，探索時就踩到（e2e「游標沒動的交還鍵」守著，拿掉這段會紅）。綁定加 `setContext`、`incognito` getter、`contextLimit()`。

**網頁行為的改變**：空的輸入框現在有前文（空字串，接句首），上屏後接上屏的字，舊 e2e 有五條照「不知道前文」寫的期望值：`composition-keys`（改讀當下首選）、`radical-hint`（第二次打前先打逗號回到句首）、`separator` 的「絃樂室」（句首時「將樂室」勝出，和分隔無關，改在無痕下打）、`edit-segments` 兩條（「目前分段的候選和單打相同」改成和**不知道前文時**單打相同，單打改在無痕下讀；plan §2.1 同步改字）。

**首選字基準**（`cargo xtask bench --check`，前 = `936a7cf` 的 `bench/baseline.json`）：全部集合的答錯與修正和改前相同，沒有翻盤，資料包 sha 同票 04（bigram `dfa21786b886`）。

| 集合 | 答錯 前 → 後 | 修正 前 → 後 |
| --- | ---: | ---: |
| yue-wiki | 1131 → 1131 | 4340 → 4340 |
| zh-wiki | 1126 → 1126 | 4380 → 4380 |
| common-voice-yue | 1032 → 1032 | 3946 → 3946 |
| hkcancor | 663 → 663 | 2573 → 2573 |
| 有前文 ×4（給 / 不給） | 576/574、559/559、200/205、454/465（不變） | 2288/2300、2166/2188、643/655、1779/1801（不變） |
| personal | 1081 → 1081 | 1446 → 1446 |

依長度分組也不變。不給前文的題在題中上屏後現在會接自己上屏的字，但模擬只在整串選完才上屏，題中上屏後重打只在「卡住後一個一個打」時發生，量不出差別。沒動學習，不附 `--learning`。個人集 baseline 檔是 1466（另一個 worktree 更新過），這次是 1446。

**驗證**：核心單元測試（`tests/context.rs` 五條、`tests/digit_separators.rs` 兩條、`session.rs` 的 `last_chars` 與上限）、網頁單元測試 `test/context.test.ts`、e2e `e2e/context.spec.ts` 七條（實體鍵盤：前文換首選、游標移開跟著變、上屏的字是前文、結尾按 End、數字後半形、無痕兩條；觸控：點鍵也照前文）。截圖 `.scratch/m3/evidence/05-context-after-mou.png`、`05-context-at-start.png`、`05-context-incognito.png`。亂按一輪（`05-chaos.png`）：組字中移游標、方向鍵、Tab、Shift 留碼、Ctrl+A / B / F、切無痕與簡體（含組字中）、快打，讀回都合理、沒有 console 錯誤；先前一次看到上屏「我」後 `hi` 首選是「我」，是 MCP 瀏覽器的 localStorage 學過「我」，清掉後是「的」，不是 bug。

**code-review（Standards、Spec 兩軸）處理**：brief §4.7 兩處、`Options::digit_separators` 與 `punctuation.rs` 的 doc 還寫「前一個上屏的字」，已改成看前文；`session.rs` 的 doc 統一用名詞表的「前文」（殼給的那段叫「殼給的前文」）；`passed_to_host` 的數字改 `d.to_string()`、`last_chars` 拿掉只有測試走到的 `limit == 0` 分支；plan §2.1 的「iOS 只給一兩句也用上屏的字」與實作不符（殼給了就整份取代），改寫成只在殼沒給時用，補長度留給雲端重排票 22 依階梯結論定；無痕的 e2e 加上非空前文「冇」（平時 `of` 首選「你」，無痕時「係」）。未處理（判斷項）：
- e2e 用無痕代表「不知道前文」（`edit-segments`、`separator`）：無痕就是網頁不給前文的唯一途徑，另開測試用入口反而多一條產品沒有的路徑；無痕的前文規則改了再一起改。
- 核心在交還宿主的鍵後作廢前文、網頁每鍵前重給，兩邊都知道這條規則：核心要照顧不給前文的殼（交還的鍵之後不能還當成數字後），網頁要照顧游標沒動的情形，plan §3.1 寫明。
- 「用上屏過的字當前文」沒加 `Options` 開關：它是排序本身用的前文（同票 04 的 `set_context`，也沒開關），brief §7 的開關清單沒有上下文；不想用前文的情形由無痕涵蓋。
