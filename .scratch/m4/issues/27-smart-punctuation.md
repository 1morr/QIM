# 27: 智慧標點：成對符號與兩下空白

Status: done
Blocked by: 24
讀: brief §4.7、§7；plan §2.1（數字後半形）；票 19、21；spec〈共同規則〉

## What to build

智慧標點的其餘兩項（可關；數字後半形已在 M1 票 26 做）：

- 成對符號自動配對（中文的「」『』（）等與英文的括號、引號）：插入一對、游標放中間，還是左右輪流，照票 19 的 Gboard 實測定。
- 英文模式連按兩下空白輸出句點加空格。
- 都用 `delete_before`；開關怎麼分（和數字後半形共用一個或各一個）由票定，預設值照 brief §7。

## 驗收

- [x] 核心測試涵蓋各情境（組字中、臨時英文中、關開關）。
- [x] Playwright：打成對符號讀回文字與游標位置；英文模式連按兩下空白讀回句點；關開關後不做；截圖。
- [x] brief §4.7、plan §2.1 更新；`cargo xtask check` 綠燈。

## Comments

- 成對符號的依據：票 19 實測 Gboard 不配對（M48），改查其他產品寫成事實 M53（librime 與 fcitx5 預設左右輪流、只有 fcitx5（預設關）、Trime（長按）、搜狗手機版有插入一對、沒有產品退格刪空的一對）；兩下空白另查 LatinIME 原始碼寫成 M52。研究底稿沒進 repo，結論都在 M52、M53。
- 做法：中文狀態 `"` `'` 左右輪流（`Options::paired_quotes`、`punctuation::Quotes`）；英文模式連按兩下空白（`Options::double_space_period`、`punctuation::SpaceTail`、`Session::expire_double_space`，網頁每個按鍵、點擊之前重算 1100 ms，點輸入框時丟掉）。沒有加 `Snapshot` 欄位，`delete_before` 只用在兩下空白（刪 1）。
- 被取代的舊測試同輪改寫：`english_mode.rs` 的點建議後空白（關掉兩下空白，只看補的空白）、`separator.rs` 與 `web/e2e/separator.spec.ts` 的「沒有組字時 `'` 照打」（改成關掉開關時照打、預設是 ‘）。
- 截圖：`.scratch/m4/evidence/27-double-space-period.png`、`27-double-space-off.png`、`27-paired-quotes.png`、`27-paired-quotes-off.png`。
- 亂按一輪（臨時 Playwright 腳本，沒進 repo）：自動修正開著打 `teh` 兩下空白得「the. 」、緊接退格得「the.」；點建議後兩下空白得「held. 」；英文模式空白後按地球鍵回中文再空白照打；聯想顯示中按 `"`、臨時英文裡的引號、關開關後組字中按 `"` 都對；沒有頁面錯誤。
- code-review（`7c9dab8` 起，Standards／Spec 兩軸）：已處理：點輸入框移游標後 1.1 秒內再按空白會在新位置換句點（先寫會紅的 e2e 再修，網頁點輸入框時呼叫 `expireDoubleSpace`，plan 補殼的契約）；單按修飾鍵也重算時限（改成不重算，共用 `keys.ts` 的 `MODIFIERS`）；`Action` 欄位與 `Session` 欄位同名、私有方法改名 `converts_double_space`、`handle_english` 註解補兩下空白；一個測試塞多個行為的拆開；補「換出句點後是句子開頭」「退格不還原」兩個測試；brief §7 句子的歧義（「學習」「無痕」看起來像智慧標點的一項）。未處理：計時器沿用 `undo-window.ts` 的 `undoWindow`（它本來就是通用的計時器，改名要動別的呼叫端）；1100 ms 在網頁常數與核心、wasm 註解各出現一次（核心註解寫的是殼的契約）；`SpaceTail` 兩個欄位可見度不同（只有一個要給 `Session` 讀）；`Quotes` 用 0、1 當索引；名詞表沒收「智慧標點」「連按兩下空白」「引號左右輪流」，要收時給使用者看草稿。
- 全部檢查：`cargo xtask check` 通過（e2e 323 passed、2 skipped；本 worktree 先 `npm ci`、`cargo xtask data`）。第一次跑時 `web/e2e/separator.spec.ts` 還斷言舊的「沒有組字時 `'` 照打」而紅，改寫後重跑通過。
- `cargo xtask bench --check`：通過，各集合與 baseline 比都沒有翻盤、答錯與修正數不變，固定題 84/84（本票只改沒有組字時的標點與英文模式的空白，基準逐鍵打輸入碼碰不到）。每鍵延遲 p95（原生、Apple M6）：空白學習資料全部 187.75 µs、學過的 533.50 µs，都在上限內（本票沒動 lattice 與排序，照附）。

Session 紀錄：中文狀態 `"` `'` 左右輪流、英文模式連按兩下空白輸出「. 」，各一個開關預設開；成對符號照使用者決定不插入一對、不加 `Snapshot` 欄位；事實 M52、M53。
決定：成對符號照小狼毫（librime `pair`）做中文狀態 `"` `'` 左右輪流，括號維持單個，不做插入一對、不加 `Snapshot` 欄位（使用者定）
決定：智慧標點三項各一個開關：數字後半形（已有）、兩下空白、引號輪流，預設都開（使用者定）
決定：brief〈英文模式與欄位類型〉〈桌面鍵盤操作〉〈設定與同步〉與 plan〈介面〉照草稿寫入（使用者同意）
決定：兩下空白的時限照 LatinIME 1100 ms，核心不讀時間、由殼計時呼叫 `Session::expire_double_space`（同 `expire_undo` 的契約），殼點輸入框時也呼叫（Agent 定）
決定：兩下空白後緊接著退格不還原，照 Gboard（M48），不照 LatinIME 換回一個空白（Agent 定）
決定：點建議補空白之後被吃掉的那一下空白算第一下，緊接著再按一下換句點，同 LatinIME 點建議後的兩下空白（Agent 定）
決定：email、網址、密碼欄切到英文模式也不做兩下空白，照 LatinIME 只在一般文字欄（Agent 定）
決定：第一個空白前要是字母、數字或 `' " ) ] } > + %`，照 LatinIME 的字元表，不收 Unicode「其他符號」（Rust 標準庫沒有這個分類）（Agent 定）
決定：引號的輪流在 `clear_context`（換欄位、失焦）時從左邊重來，照 fcitx5；librime 不重來（Agent 定）
決定：單按修飾鍵不打斷兩下空白、網頁也不重算時限，同退格撤銷對修飾鍵的處理（Agent 定）
