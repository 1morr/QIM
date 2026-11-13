# arch-diet：plan 縮成架構總覽

不是里程碑，規格就是本檔（使用者 2026-11-13 同意）。不改產品行為；只動 `docs/`、`ARCHITECTURE.md`、`CLAUDE.md`、`.scratch/arch-diet/`，以及程式碼裡指向文件的註解。

## 目標

`docs/plan.md`（約 104,000 字）換成根目錄的 `ARCHITECTURE.md`（上限 15,000 字）：只留打開程式碼看不出來的東西，使用者讀得完，agent 不必再挑章節讀。

## 範圍

### 做

- plan 每一節的每一段歸到四處之一：留在 `ARCHITECTURE.md`（模組分工、依賴規則、跨模組的資料與契約、測試策略）、搬到 `docs/development.md`（操作步驟）、寫成 ADR（選型與取捨的理由）、刪掉（程式碼與測試已經說明的行為）。
- `ARCHITECTURE.md` 的節用標題、不編號；引用 plan 的地方（`CLAUDE.md`、roadmap、brief、ADR、M4 還沒做的票、程式碼註解的 `plan〈…〉`）跟著改。
- 做完刪掉 `docs/plan.md`，`scripts/check_docs.py` 的 15,000 字上限開始生效。

### 不做

- 不改 brief 的決定、不改產品程式與測試的行為。
- 已封存的里程碑（`.scratch/m0`–`m3`、`docs/archive/`、做完的票）裡的 `plan §x` 是歷史，不改。

## 共同規則

1. **一段都不能憑空消失**：每張票的 Comments 放對照表「plan 舊的每一段（節號加開頭幾個字）→ 新位置，或刪掉的理由」。刪掉的理由只能是「這個行為由 `<測試或程式檔>` 說明」，要寫出是哪個檔。
2. **整段搬移要證明沒丟**：搬到 `docs/development.md` 的內容逐行比對，來源刪掉的每一行都在目的地。
3. **只寫現況**：票號、日期、「某日改」不進 `ARCHITECTURE.md`；決定的來龍去脈在 `docs/decisions.md` 與 ADR。
4. **拿不準就留、並列出來**：分不清一段是契約還是實作細節時，留在 `ARCHITECTURE.md`，在 Comments 列給使用者決定。
5. **矛盾不自己挑**：plan 和程式碼或 brief 對不上時，列在 Comments 問使用者。
6. plan 的節搬完後，在 plan 留標題與一行「已搬到 …」，編號不變；最後一張票才整份刪掉。
7. 一個 session 一張票；每張票收尾跑 `cargo xtask check --fast`。

## 驗收

- `ARCHITECTURE.md` 不超過 15,000 字，`docs/plan.md` 已刪，`cargo xtask check` 全套綠燈。
- 每張票的對照表涵蓋 plan 那一節的每一段。
- repo 裡除了已封存的里程碑，沒有指向 `docs/plan.md` 或 `plan §` 的引用。

### 使用者試

- 使用者把 `ARCHITECTURE.md` 從頭讀一遍，回覆看不懂的段落與覺得不該刪的內容；改完才算驗收。

## 票序

01 → 02、03、04、05（互不相擋，可以同時跑）→ 06。
