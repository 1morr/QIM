# 07: 連打

Status: ready-for-agent
Blocked by: 06
讀: plan §2.1、§2.2、§2.3、§2.5（`[ranking]`）；brief §4.1、§4.8（為什麼單字不容錯）、§12.2（三案例根因）；`docs/research/rime.md` §4–§5；`docs/research/open-source-imes.md` §5；`docs/research/ranking-techniques.md` §4；spec〈共同規則〉；`.scratch/m0/issues/09-m0-close.md` 的 Comments（「M1 票 2」那條）

## What to build

速成連打：輸入碼不再整串查表，而是切成字與詞的 lattice，用 plan §2.3 的同一把尺打分取首選。網頁打 `anr` 首選「門口」、打一長串碼得到一句話；選了開頭的一段（例如只選第一個字）後，那段固定，其餘輸入碼繼續組字。

- 開工先讀 libime 的 lattice 與增量更新（GPL 只取做法，ADR 0006），摘要記到 `docs/research/`（新檔或補進 `open-source-imes.md`）。
- lattice：位置是輸入碼的字母邊界；邊來自碼表（1–2 碼）與詞庫；擴展區的字不進 lattice，只在整串碼剛好是一個字的碼時排在一般字之後（plan §2.3）。
- Viterbi + beam（寬 20，讀方案檔 `[ranking].beam`）取前 k 條路徑；每按一鍵只補算新增的位置。
- 打分先只有 unigram（字頻、詞頻）加 β × 詞段數 − γ × 段數；β、γ 初值寫進方案檔，用 bench 粗調，細調在票 12。
- 候選列：整串的前幾條路徑，加上覆蓋開頭一段的字詞（plan §5.2 的修正次數模擬要選「接得上的最長候選」）；選一個 prefix 候選後，該段固定、其餘重算，組字直到全部選定或空白上屏。
- 碼表與詞庫的查詢搬進 `Engine`（見〈M0 留下的〉）。
- `anr` 首選「門口」，若票 06 還沒加，加進 `fixed.tsv`。
- bench 前後對照；延遲（票 05 若已完成）一併貼。

## M0 留下的

票 09 Comments「M1 票 2」：碼用 `String`，輸入碼切分時收成 `QuickCode` / `CangjieCode`；`Engine` 與 `Pack` 欄位相同、`Session` 直接伸進 `engine.scheme` 與 `code_table` 查候選，改寫查詢時把查詢搬到 `Engine`。

## 驗收

- [ ] `docs/research/` 有 libime lattice 與增量更新的摘要。
- [ ] `fixed.tsv` 含 `anr	門口`，`cargo xtask bench --check` 綠燈（baseline 更新寫理由）。
- [ ] 核心測試：連打多字的首選；選 prefix 候選後該段固定、其餘重算；擴展區字不出現在多字路徑；增量計算與從頭算結果相同（性質測試）。
- [ ] Playwright：打 `anr` 讀回首選「門口」；打一串連打、按數字選第一段、再按空白，讀回輸入框為兩段合起來的文字。
- [ ] bench 前後對照貼在 `## Comments`。
- [ ] 〈M0 留下的〉每條有結果。

## Comments
