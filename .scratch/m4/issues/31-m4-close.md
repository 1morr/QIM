# 31: M4 收尾

Status: ready-for-agent
Blocked by: 01, 02, 03, 04, 05, 06, 07, 08, 09, 11, 12, 13, 14, 15, 16, 17, 18, 18b, 18c, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30（之後補的票也擋；10 依票 09 的結論 wontfix）
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

- 2026-11-13 流程對齊（使用者同意）：M4 封存後、拆 M5 的票之前，在新 session 下 `/1morr:project-init`，它會走同步模式。要對齊的有：路線圖搬到 `docs/roadmap.md`、brief〈查證過的事實〉搬進 `docs/research/` 並編號、plan 只留現況與跨模組契約、程式碼不引用章節號（現有約 700 處）、票留 `Session 紀錄：` 由合併的人寫 progress.md、驗收加使用者試。M4 進行中不動這些，免得和還沒做的票撞。
