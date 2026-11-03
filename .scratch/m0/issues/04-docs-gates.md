# 04: progress 與 spec 文件閘門

Status: done
Blocked by: 01
讀: plan §6；`docs/progress.md`〈寫法與封存〉；`docs/agents/issue-tracker.md`；spec〈共同規則〉

## What to build

progress.md 或里程碑 spec 違反規則時（例如 union 合併安靜地抄出兩個〈Session 紀錄〉段、某列寫太長、忘了寫下一步的指令），`cargo xtask check` 會紅，並指出哪一行違反哪一條。

- `cargo xtask check` 加文件閘門步驟，守 plan §6 列出的規則：
  - progress 的 `##` 段固定是〈寫法與封存〉〈里程碑與閱讀指引〉〈Session 紀錄〉〈偏差與決定〉，各一次、依序。
  - session 紀錄是一整張表。
  - session 紀錄每列、偏差每條，整行不超過 200 個字元（以 Unicode 字元計）。
  - session 紀錄「票」欄開頭的里程碑、偏差每條日期後的里程碑，都是里程碑表中備註以「進行中」開頭的那一個。
  - session 紀錄最後一列的「下一步」含反引號包住、`/` 開頭的指令，或以 `等使用者：` 開頭。
  - `.scratch/` 下有 `issues/` 的目錄都有 `spec.md`，含〈目標〉〈範圍〉〈共同規則〉〈驗收〉〈票序〉五個 `##` 段。
  - `.scratch/<里程碑>/` 只能是 progress 里程碑表中備註不是「未開始」的里程碑。
- progress〈寫法與封存〉最後一條的「M0 票 01 的閘門」改成「M0 票 04 的閘門」；plan §6 開頭改成依票分列的交付（01–04）。

## 驗收

- [x] 目前的 repo 上文件閘門綠燈。
- [x] 測試在暫存副本造出每一條違規，各自會紅：重複的〈Session 紀錄〉段、段落順序錯、表格中間斷開、201 字元的一列、標 M1 的條目（M1 未開始）、最後一列的下一步沒有指令、`.scratch/m1/issues/` 存在、spec 缺〈票序〉。
- [x] 測試在暫存副本做無關改動不紅：改某列措辭但不超長、加一列合規的紀錄、下一步改成 `等使用者：…`、spec 內文改寫但段落齊全。

## Comments

- 2026-11-03 最後一條（「M0 票 01 的閘門」改成「M0 票 04」、plan §6 依票分列 01–04）開工時已成立；這輪把〈寫法與封存〉最後一條改成逐項列出閘門守什麼、什麼靠自查。
- 2026-11-03 比票嚴的地方（都寫回 plan §6）：
  - 里程碑表要剛好一個「進行中」。為此〈寫法與封存〉規定封存與下一個里程碑開工在同一個 commit，見 progress〈偏差與決定〉。
  - 缺段、多出段都紅；session 紀錄的表外不放文字。
  - 偏差每條一行、`- <日期> <里程碑>` 開頭：否則分兩行或用 `*` 就能繞過長度與里程碑檢查。
  - 里程碑目錄是 `m` 加數字，不在里程碑表上也紅（`.scratch/m7/`）；其他名字是不屬於里程碑的工作，只查 spec。
- 2026-11-03 寬容的地方：spec 標題可加全形括號說明（`docs/agents/issue-tracker.md` 寫「範圍（做 / 不做）」）；表格的 `\|` 不分欄；` ``` ` 與 `~~~` 程式碼區塊內的 `##` 不算段。
- 2026-11-03 變異測試用 `docs.rs` 內建的小型 progress 與 spec 範本，不複製 repo 現況：第一版複製現況，code-review 實測模擬 M0 封存後 23 支壞 8 支，會擋住封存那個 session。現況由 `current_repo_passes` 守。
- 2026-11-03 驗收輸出（工作樹上暫時抄出重複的〈Session 紀錄〉段、加一條標 M1 的偏差、建 `.scratch/m1/issues/`，跑 `cargo xtask check --fast`，之後還原）：
  - `docs/progress.md:47: 〈Session 紀錄〉段重複：…`
  - `docs/progress.md:97: 標 M1，但進行中的里程碑是 M0：…`
  - `.scratch/m1/: 里程碑 M1 在 docs/progress.md 的里程碑表是「未開始」：…`
  - `.scratch/m1/spec.md: 有 issues/ 就要有 spec.md`
  - `check 失敗：docs`
- 2026-11-03 code-review 沒處理的發現：
  - wayfinder 版面（`docs/agents/issue-tracker.md`〈Wayfinding operations〉）是 `.scratch/<effort>/map.md` 加 `issues/`、沒有 spec.md，會被這個閘門擋。照票字面做；目前沒有這種目錄，真要用 `/wayfinder` 時再決定豁免 `map.md` 目錄或改 issue-tracker。
  - `docs::report` 與 `deps::report`、兩邊測試的 `repo_root` / `assert_red` 幾乎相同；等第三個閘門再抽共用。
