# 決策紀錄

進行中里程碑做過的決定，一條一行，只追加。已驗收里程碑的在 `docs/archive/<里程碑>.md` 的〈決定〉。現在的做法寫在 brief、`ARCHITECTURE.md`、ADR 與 spec；這裡記的是「誰在什麼時候定了什麼」，並連到寫著細節的地方。

## 寫法

- 一條一行：`- <日期> <里程碑>（<決定者>）：<決定，一句>。<細節在哪：ADR、票、brief 的章節>`，整行不超過 200 字元。
- 決定者三選一：`（使用者定）` 使用者自己定的、`（使用者同意）` agent 提議而使用者同意的、`（Agent 定）` 使用者沒看過的。
- 要記的：訪談與規劃時定下的、推翻先前決定的、做票時在兩種做法之間選的。照票面與文件做、沒有選擇的不記。
- 做票的分支不改本檔：票的 Comments 留 `決定：<一句>（<決定者>）` 行，合併的人在主分支抄成這裡的一條（`AGENTS.md`〈每個 session 的流程〉）。其他 session（訪談、規劃、開工、驗收票）自己加。
- 需要寫理由、難回頭的架構決定另寫 ADR，在這裡佔一行並連過去。
- 推翻舊的一條時加新的一條，寫明推翻哪一條的日期；舊的那條留著。
- 里程碑驗收時，把標著它的那幾條整段搬到 `docs/archive/<里程碑>.md` 的 `## 決定`，逐條比對沒有遺失；還沒有里程碑時定的（標「框架」這類）跟第一個里程碑一起搬。本檔只留進行中的。
- `python3 scripts/check_docs.py` 強制〈紀錄〉每條的日期開頭、一個決定者、長度。

## 紀錄

- 2026-11-17 M5（使用者同意）：里程碑開工照範本加「對程式碼」一步，票上講到的現有程式碼逐條核對、對不上的給使用者接受或拒絕。`docs/agents/milestone.md`〈開工〉
- 2026-11-17 M5（使用者同意）：〈UI〉的規則搬到 `docs/agents/ui.md`，`AGENTS.md` 只留實跑驗證與一句指向。
- 2026-11-17 M5（使用者定）：用 `visual-explainer` 把架構草稿、開工計劃、對程式碼的問題、二選一做成一頁作答。`docs/agents/visual.md`
- 2026-11-17 M5（使用者同意）：換範本新版 `next_step.py`、pre-commit 開頭清 `GIT_*`；範本新版 `check_docs.py` 的新規則點名進 flow-sync 票 01，不另開票。
- 2026-11-17 M5（使用者同意）：先跑 gates、flow-sync、arch-diet 三批流程票，再拆 M5；gates 進行中，M5 排在 arch-diet 之後。M4 回顧，`docs/roadmap.md`
- 2026-11-17 M5（使用者同意）：M5 分兩期，第一期盤點、設計稿、候選列與工具列、設定頁；期界在盤點勾完後定。M4 回顧，roadmap〈M5 觸控鍵盤與設定〉
- 2026-11-17 M5 gates（Agent 定）：密鑰掃描用 gitleaks 8.30.1，不用後繼 betterleaks（v2 還在 rc）或會連網驗證的 trufflehog；betterleaks v2 穩定後再評估。事實 M54
- 2026-11-17 M5 gates（Agent 定）：依賴漏洞用 osv-scanner 一個工具掃 Cargo.lock 與 web/package-lock.json，不用 cargo-audit 加 npm audit：npm audit 不能忽略單一公告。事實 M55
- 2026-11-17 M5 gates（Agent 定）：依賴更新用 Dependabot 不用 Renovate；每週一次，minor 與 patch 併一個 PR，wasm-bindgen 一家另開。事實 M56
- 2026-11-17 M5 gates（Agent 定）：沒有修正版的已不維護公告寫進 `osv-scanner.toml` 忽略並附理由，不設到期日，`check` 維持綠燈。
- 2026-11-17 M5 gates（Agent 定）：`secrets` 步驟只看暫存區（pre-commit 用），CI 另全掃 git 歷史，擋用 `--no-verify` 跳過 hook 的 commit。
