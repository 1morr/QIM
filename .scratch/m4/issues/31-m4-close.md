# 31: M4 收尾

Status: ready-for-agent
Blocked by: 01, 02, 03, 04, 05, 06, 07, 08, 09, 09b, 09c, 11, 12, 13, 14, 15, 16, 17, 18, 18b, 18c, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 30（之後補的票也擋；10 依票 09 的結論 wontfix；29 因譯詞移出 M4 wontfix）
讀: spec 全部；`docs/progress.md`〈寫法與封存〉；各票的 Comments

## What to build

驗收 M4 並封存：

- 從 M4 開工的 commit（spec 與票的那個 commit，見 progress 的 session 紀錄）起跑里程碑 code-review（`mattpocock-skills:code-review`）。
- 在乾淨 checkout 重驗 spec〈驗收〉的每一條。
- 學習資料 v3 定稿，寫進 plan §2.4。
- 使用者在手機瀏覽器試用第二期的觸控功能並回覆。
- 各票 Comments 沒處理的發現逐條註明去處（M5、平台里程碑、等使用者、不修）。
- 期中檢查（票 18）點名留給本票的：v3 定稿時決定 `#@/device`、`#@/tick` 行是否限定在檔頭、名詞表收不收「列的種類」（票 12）；`pack-update.spec.ts`「sha256 不符時拒絕」偶發紅（票 13）；debug 面板開著打過字後第一次點「無痕」沒反應（票 16）；組字中途開無痕，面板寫「不讀前文」但開頭接「句首」（票 01）；`learning.spec.ts` 每次 e2e 改寫已封存的 `.scratch/m3/evidence/23-personal-bigram.png`（票 01）；progress.md union 合併後的重複段落。
- 照 progress〈寫法與封存〉封存 M4，M5 標成進行中。

## 驗收

- [ ] 里程碑 code-review 的發現已處理或註明去處。
- [ ] 乾淨 checkout 重驗 spec〈驗收〉每一條，證據附在 Comments。
- [ ] v3 定稿寫進 plan §2.4。
- [ ] 使用者試用第二期並回覆，progress〈偏差與決定〉記一條。
- [ ] 各票 Comments 沒處理的發現逐條註明去處。
- [ ] M4 封存到 `docs/archive/M4.md`，M5 進行中；`cargo xtask check` 綠燈。

## Comments

- 2026-11-13 流程對齊（使用者同意）：已做完（roadmap、事實編號、決策紀錄、票留 Session 紀錄、收尾加使用者試、文件檢查換成 `scripts/check_docs.py`）。剩 plan 縮成 `ARCHITECTURE.md`，票在 `.scratch/arch-diet/`，和 M4 的票互不相擋。
- 2026-11-17 里程碑 code-review（`1cf2f1c...8e46661`，195 個 commit，兩軸各一個子代理）：Spec 軸：〈驗收〉每一條都有對應的測試（下面重驗那條），v3 與 plan §2.4 一致，`Options` 預設值與 brief §7 一致，網頁設定頁每項都有一列。處置：
  - 已修：`Options::lists_like` 漏了 `neighbor_readings`，殼只呼叫 `set_options` 不 `refresh` 時選字照舊用插了鄰鍵讀法的那列（Standards）：`neighbor_readings.rs` 加「關掉後沒 refresh 就選，選的是原樣讀法的那列」，修前紅、修後綠；網頁每個 setter 都呼叫 `refresh`，使用者碰不到。程式碼引 plan 章節號 9 處（Standards，含 M4 前的 `docs/plan.md` §1.3 四處與 `edit1.py` 的 brief §12.1），改成節名；`check_docs.py` 的 `SECTION_REF` 只認 `plan` 緊接 `§`，放寬成中間可隔反引號或節名、不跨過別的 `.md` 或 ADR、40 字內，測試加三個會紅的寫法與兩個不該紅的（研究筆記、ADR 的節號），先紅後綠。`neighbor-readings/README.md` 的分支名換成合併的 commit `60d5bfb`（〈公開 repo〉）。`Snapshot.ascii` 的 doc 與 `main.ts` 註解補上英文模式（改名是對外介面，延後，見下）。自動修正後接著兩下空白再退格只刪空白（Spec：只用臨時腳本驗過）：`smart_punctuation.rs` 加一例守順序。spec〈範圍〉欄位類型與智慧標點兩行補註使用者在票 26、27 的改動（Spec）。
  - 延後：核心給殼的介面整理（`Snapshot.ascii` 改名、模式改列舉、`set_options` 自己判斷重列、拆 `session.rs`、三份 tally／墓碑、四種小寫鍵、兩份 `TouchSurface`、`afterHostKey`）→ roadmap〈平台里程碑〉M7 開工時；網頁的 `free()` 寫法、`undoWindow` 名字、e2e 共用 helper → roadmap〈M5〉。
  - 不做：文件檢查改寫成 `scripts/check_docs.py` 不屬於任何票（Spec），是 progress 記過的流程對齊；`Snapshot.auto_shift` 沒寫進 ADR 0008（ADR 留給票 24 定，plan〈介面〉有寫）。
- 2026-11-17 乾淨 checkout 重驗：`git worktree add --detach` 到 session 暫存目錄（`8e46661`），`build`、`target`、`web/node_modules`、`bench/personal` 都不存在；`npm ci --prefix web`、`npx playwright install chromium`、wasm-bindgen 0.2.130、`cargo xtask data`（exit 0）、`cargo xtask check`（e2e `323 passed (1.4m)`、`2 skipped`、`check 通過`）、`bench --check`（`bench 通過`）、`bench --learning`；`bench/personal` 用 `ln -s` 接主 checkout。spec〈驗收〉逐條：
  - 網頁功能各有 Playwright 實打讀回，各票截圖在 `.scratch/m4/evidence/`；這次挑 10 張存成 `31-clean-*.png`。觸控在手機瀏覽器的試用見下一條（使用者試）；期中檢查試過第一期（票 18）。
  - 首選字基準：「含打錯碼」「觸控模擬」四個集合都在 baseline；沒打錯的 12 個集合答錯與修正和 M4 開工時（`1cf2f1c` 的 baseline）逐一相同（例 yue-wiki 941／3239、hkcancor 572／2148），17 個集合沒翻盤、固定題 84/84、個人集沒翻盤；改錯 0/475、0/298、0/241、0/147；觸控鄰鍵讀法開／關修正 5312／6116、3346／3776。每鍵原生 p95：空白全部 189 µs、第 20 碼起 201 µs、選開頭一段 4.03 ms；學過 326 µs、479 µs、5.73 ms；含萬用鍵空白 393 µs、344 µs、4.09 ms，學過 605 µs、495 µs、5.91 ms，都在上限內；載入 149 ms。`--learning` 公開集合學習開 11621 對關 12129，個人集 1389 對 1462。
  - Playwright 讀回（都在上面 323 passed 裡）：`autocorrect.spec.ts`「預設不修正 teh…」「打開自動修正後英文模式打 teh 修正成 the，退格還原…」（`31-clean-autocorrect-*.png`）；`english.spec.ts`「打 receive 第 2 位是 receive…打 recieve 沒有英文候選」（`31-clean-english-receive.png`）；`field-types.spec.ts`「email 欄位預設英文直通、打 teh 不修正、沒有建議…」；`english-mode.spec.ts`「地球鍵切到英文頁打 rrrrrr 一次後…打第二次後有」（`31-clean-rrrrrr-twice.png`）、「v2 匯入的 email 打開頭的字母，英文建議列就有它」；`backspace-undo.spec.ts`「聯想關掉：選第 2 個候選兩次，第二次上屏後緊接退格，再打同一串碼首選不變」（`31-clean-backspace-undo.png`）；`debug-panel.spec.ts`「前文的三種狀態」「停鍵後讀得到重排的要求與結果」（`31-clean-debug-*.png`）。
  - 同步：`sync.spec.ts`「兩個瀏覽器設定檔各選一次同一個候選，互相匯出匯入後它成首選」「一個新使用者詞與一筆刪除同步後兩邊讀回一致」（`31-clean-synced-*.png`）。
  - 格式：`properties.rs` 的 `exporting_then_importing_gives_an_equal_user_model`、`merging_is_commutative_associative_and_idempotent`；`user_dict.rs` 的 `v1_and_v2_files_import_with_the_import_day_as_last_used_and_export_as_v3`（`cargo test` 綠燈）。
  - 結論寫回：句中英文混輸不做（brief〈英文模式與欄位類型〉，票 28）；聯想補強做 emoji、另兩項排 M5（plan〈介面〉，票 30）。
- 2026-11-17 v3 定稿：plan §2.4 寫明 M4 票 31 定稿，四種列（`entry` 16 欄、`bigram` 10、`english` 9、`deleted` 7）、metadata 行與照收的 v3 草稿（7ad01da 的 12／6／6 欄，依欄數認；票 12 的 11 欄不收）就是 v3，之後升 v4。使用者同意 metadata 行只能寫在檔頭：`user_dict.rs` 加「v3 的 metadata 寫在第一筆資料列之後整份不收、v1、v2 照舊」，先紅後綠，`format.rs` 讀到 v3 的資料列後再遇到 `#@/` 就錯在那一行。網頁存的檔本來就是檔頭在前。
- 2026-11-17 各票 Comments 沒處理的發現逐條歸位（子代理整理 28 條，加協調者列的）：
  - 已修：`learning.spec.ts` 改寫 M3 證據（e5e4686 已修）；`field-types.spec.ts` 每次 e2e 改寫 `.scratch/m4/evidence/26-field-*.png`（協調者，本票改寫到 Playwright 輸出目錄）；progress.md union 合併後的重複段落（b3a1a7b、b4e34c9 已修）；名詞表「列的種類」使用者同意不收；範例 email `ivan.test@example.com` 換成 `jean.test@example.com`（使用者同意；核心、e2e、ADR 0008、v3 草稿固定檔，`iva` 本身是英文詞、`jea` 不是，兩個 e2e 的名次跟著改；已封存的 M3 票、實驗腳本 `remembered-prefix`、`autocorrect-threshold` 的輸入與舊截圖不動，改了會讓記錄的數字重現不了）；名詞表加「欄位類型」（協調者）「智慧標點」、改「emoji 候選」（票 27、30，使用者同意），brief〈設定與同步〉的 emoji 候選開關補上管聯想裡的 emoji。
  - 延後到 M5：e2e 共用 helper（票 04 等）、debug 面板開著打過字後第一次點「無痕」沒反應（票 16）、`🌐` 標籤與 `touch.ts` 的中英狀態（票 23）、觸控刪除鍵的退格撤銷（票 18c）、觸控按鍵機率接進英文修正與 4 倍門檻重量（票 25）；本來就在 M5 的：`z` 鍵標示（票 11）、撤銷學習手勢與簡體去重（票 16）、句首建議與量詞聯想（票 30）、英文下一個詞（票 21）。
  - 延後到平台里程碑：組字中換資料包不重建 lattice 的欄（票 18b，M7 背景更新資料包時）；email、網址欄自己切英文模式的邊角（票 26）；iOS 切換鍵與系統鍵盤清單、事實 P25 實機量（票 21、23，M8、M9）；本來就在的：實體鍵盤 ISO、dead key、真實配列（票 07，M7）。
  - 延後到〈不排里程碑〉：容錯重評要補的（票 06–10、09c 的重排下觸控修正次數）；e2e 高負載偶發紅三例（票 13 的 `pack-update.spec.ts`、協調者列的票 21 長按切頁、票 26 單按 Shift 計時），這次乾淨 checkout 與本 session 都沒重現。
  - 不做：組字中途開無痕，面板寫「不讀前文」但開頭接「句首」（票 01）：lattice 是開無痕前建的，無痕在下一次組字才完全生效；學習在上屏時看無痕、不受影響，平台殼的無痕多由聚焦密碼欄觸發，那時沒有組字；時鐘錯時的未來日期、條目計數 58 µs、墓碑只在合併時回收（票 12、13、17，條件式，票 18 已判不修）。
  - 等使用者：規則 4 要不要只出頻率 ≥ 90 的英文詞（票 22）→ 使用者同意維持全收。
- 2026-11-17 `scripts/check_docs.py` 放寬 plan 章節號規則的修改要回饋到 project-init 範本（待使用者）：QIM 這份是從範本換來的，下次同步可能被舊版蓋回，用同一份範本的專案也還沒有這個修正。
