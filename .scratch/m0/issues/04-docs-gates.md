# 04: progress 與 spec 文件閘門

Status: ready-for-agent
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

- [ ] 目前的 repo 上文件閘門綠燈。
- [ ] 測試在暫存副本造出每一條違規，各自會紅：重複的〈Session 紀錄〉段、段落順序錯、表格中間斷開、201 字元的一列、標 M1 的條目（M1 未開始）、最後一列的下一步沒有指令、`.scratch/m1/issues/` 存在、spec 缺〈票序〉。
- [ ] 測試在暫存副本做無關改動不紅：改某列措辭但不超長、加一列合規的紀錄、下一步改成 `等使用者：…`、spec 內文改寫但段落齊全。
