# 21: 英文模式 ADR 與 `delete_before`

Status: done
Blocked by: 19, 18b
讀: brief §4.7、`docs/research/facts-platforms.md`；plan §1.3、§2.1（`Snapshot`、英文直通、臨時英文）、§4.4；spec〈細節〉〈英文模式〉；票 19 的結論；spec〈共同規則〉

## What to build

核心的英文直通把鍵交還宿主；英文模式要核心自己組字、出建議，是新的組字狀態。寫一份 ADR，一起定：

- 英文模式的組字狀態，和中文組字、英文直通、臨時英文的關係；桌面的英文仍是直通（spec〈細節〉）。
- `Snapshot` 加「先刪游標前 N 字再上屏」（例如 `delete_before`），給自動修正還原、連按兩下空白、成對符號用；殼照做，不自己判斷（plan §1.3）。先查證 iOS `deleteBackward()`、Android `deleteSurroundingText`、macOS IMK `replacementRange` 的行為，寫進 `docs/research/facts-platforms.md`。
- 英文的切詞：`@`、`.` 這類字元算不算詞內（email 要整串是一個詞，LatinIME 以 `.` 分隔）。
- 「預測」包不包括下一個詞。
- 自動修正、句首大寫、英文容錯在 `Options` 的欄位與預設（brief §7）。

ADR 之外，`Snapshot` 加上這個欄位、網頁照做（核心先只在測試裡送），之後的票直接用。

## 驗收

- [x] ADR 寫進 `docs/adr/`，各項決定附成熟產品的做法（LatinIME、Gboard 實測、librime）。
- [x] `docs/research/facts-platforms.md` 加三個平台刪除游標前文字的查證與來源。
- [x] `Snapshot` 有 `delete_before`，網頁照做（測試掛鉤或單元測試驗）；plan §2.1 更新；`cargo xtask check` 綠燈。

## Comments

- 做到：ADR 0008（英文模式逐字上屏、進出照切換鍵的來源、`delete_before`、切詞、預測不含下一個詞、`Options` 三個欄位、無痕照樣有建議）；事實 P25–P28（iOS、Android、macOS 與 Fcitx5／Wayland 等刪游標前文字的 API 與單位），P11 補註 iOS 13 起有 `setMarkedText`；`Snapshot.delete_before`（核心、wasm、ffi，核心一直送 0）；網頁 `host.ts` 的 `insert` 先刪再上屏、實體與觸控退格共用 `deleteStart`，單元測試 `web/test/host.test.ts`；plan §2.1、brief §4.7 與 §7、roadmap〈M5 觸控鍵盤與設定〉跟著改。
- 成熟產品的做法：LatinIME（分隔字元與 word connectors、`revertCommit` 以 Java char 刪、「Show correction suggestions」）、Gboard 實測（M47、M48：Chrome 裡逐字上屏、修正時刪詞、`.` 是詞界、密碼欄沒有建議、不配對成對符號）、librime（`ascii_composer` 的 `switch_key` 依切換鍵綁行為，R14；只有 `ascii_mode`、沒有英文組字）、Fcitx5（刪除以 code point 計）；採用誰寫在 ADR 各條。
- 全部檢查：`cargo xtask check` 通過（e2e 286 passed）。第一次跑時 e2e 的 `touch-features` 長按切頁一題紅，當時 commit hook 的 clippy 同時在編譯；單獨重跑該檔 32 題全過，第二次完整跑全綠，判定是負載造成。worktree 先 `npm ci` 與 `cargo xtask data`。
- 亂按一輪：實體鍵盤組字、標點、數字、退格、Esc，觸控鍵盤字母、空白、句號、刪除，上屏與退格讀回正確；截圖 `.scratch/m4/evidence/21-delete-before-smoke.png`。核心一直送 0，畫面看不出 `delete_before`，行為由單元測試驗。
- code-review（`62b1552` 起，Standards／Spec 兩軸）：已處理：只刪不上屏時 `insert` 連選取一起刪（改成有上屏時一次取代「游標前 N 字 + 選取」、只刪時 `preserve` 留住選取），補 `insert` 的單元測試（照 HTML `setRangeText` 的假輸入框）；ADR 補 librime 先例；自動修正補「全大寫不修」；改正對 M47「`.` 是詞界」的誤引；成對符號用不上 `delete_before`，ADR 寫明偏離票的原意、核心註解拿掉；plan 的 `Snapshot.ascii` 註解與 `ToggleAscii` 段跟上英文模式；核心註解拿掉會過期的「哪幾張票才送」。未處理：「英文模式」「正在打的串」還沒進名詞表，照 spec〈共同規則〉由用到的票 24 補；第一個 commit 的註解先引用了下一個 commit 才加的 ADR 0008（只差一個 commit，不改寫歷史）；`insert` 分開收 `commit` 與 `deleteBefore` 而不收整個 Snapshot（`hostDefault` 也呼叫它，那裡沒有 Snapshot 的上屏）。

Session 紀錄：寫 ADR 0008 定英文模式與 `delete_before`、查證三個平台刪游標前文字的 API（P25–P28），`Snapshot` 加 `delete_before`、網頁照做並有單元測試。
決定：英文模式逐字上屏、沒有組字，改字用 `delete_before`，照 Gboard 在 Chrome 的做法（Agent 定）
決定：觸控的中英鍵進英文模式、實體單按 Shift 照舊進英文直通，照切換鍵的來源決定、不加選項（Agent 定）
決定：M4 的預測不含下一個詞，留到 M5 重做候選列時評估，記在 roadmap（Agent 定）
決定：`Options` 另開 `autocorrect`（關）、`auto_capitalize`（關）、`english_corrections`（開），英文容錯不和中文的鄰鍵讀法共用開關（Agent 定）
決定：英文模式的詞沿用英文使用者詞的串字元，email 整串一個詞；自動修正只修去掉頭尾 `.` `'` 後全是字母、不是全大寫的詞，在串打完時修（Agent 定）
決定：`delete_before` 以 Unicode scalar 計，核心只刪自己送出去、沒作廢的字，不刪殼給的前文（Agent 定）
決定：本票核心只加欄位、一直送 0，前文與正在打的串跟著刪的帳留給第一張送出它的票經公開介面測；網頁以 node:test 單元測試驗（Agent 定）
決定：無痕與學習關掉時英文模式照樣在記憶體追蹤正在打的串、出建議與自動修正，只是不學；密碼欄照 brief §8 不學、不讀上下文，要不要另關建議由票 26 定（Agent 定）
決定：成對符號用不上 `delete_before`（插入一對要放游標到中間、刪空的一對要刪游標後的字），要不要另加欄位由票 27 定（Agent 定）
決定：事實編號用 P25–P28，iOS `deleteBackward()` 的單位未實機量、標未查證，因核心只刪單一 scalar 的字而不依賴它（Agent 定）
決定：P11 的「沒有 marked text」補註 iOS 13 起有 `setMarkedText`、宿主怎麼顯示未測（Agent 定）
決定：「英文模式」「正在打的串」的名詞表條目留給用到它們的票 24（Agent 定）
