# 進度

工作單位是票（`.scratch/<里程碑>/issues/NN-slug.md`），狀態在票的 `Status:` 行。這裡只記進行中里程碑的 session 紀錄；里程碑的順序、狀態與閱讀指引在 `docs/roadmap.md`，決定在 `docs/decisions.md`。

## 寫法與封存

- 做票的分支不改本檔：票的 Comments 留 `Session 紀錄：` 行，合併的人在主分支抄成這裡的一列（`AGENTS.md`〈每個 session 的流程〉）。
- 一列不超過 200 字元；細節在票的 `## Comments`，這裡只放指向它的一句話。
- 「票」欄標里程碑（`M0 01`），封存時靠它分。
- 里程碑驗收時（步驟在 `docs/agents/milestone.md`〈收尾票〉），把它的 session 紀錄移到 `docs/archive/<里程碑>.md` 的 `## Session 紀錄`、`docs/decisions.md` 裡它的決定移到同一個檔的 `## 決定`，逐列逐條比對沒有遺失，`docs/roadmap.md` 里程碑表的狀態換成那個檔的連結。同一個 commit 把下一個里程碑標成進行中，「下一步」寫 `/1morr:checkpoint <封存的里程碑>`。本檔只留進行中的里程碑。
- `python3 scripts/check_docs.py`（`cargo xtask check` 的 `docs` 步驟會跑它）強制的規則列在腳本開頭；本檔的是：兩個 `##` 段的順序、session 紀錄是一整張表、每列的長度、最後一列的下一步。

## Session 紀錄

| 日期 | 票 | 結果 | 下一步 |
| --- | --- | --- | --- |
| 2026-11-12 | M4 封存 M3 | 票 23：里程碑 code-review（修匯出 tick、補個人 bigram e2e 等）、乾淨 checkout 重驗 spec〈驗收〉、匯出 v2 定稿；M3 封存到 `docs/archive/M3.md`；見票 23 | 新 session 下 `/to-tickets docs/plan.md 拆 M4 的票` |
| 2026-11-12 | M4 檢視 | 回顧 M3、畫架構與流程圖；子代理審查與研究（兩份筆記）；使用者同意 D1–D8，roadmap 的 M4 一節重寫成兩期、加 M5 一節、brief §4.5 已改 | 新 session 下 `/to-tickets docs/plan.md 拆 M4 的票` |
| 2026-11-12 | M4 拆票 | roadmap 的 M4 一節細化成 `.scratch/m4/spec.md`（每鍵延遲上限、細節搬進 spec〈細節〉）；使用者同意切法，31 張票 | 新 session 下 `/implement .scratch/m4/issues/01-debug-panel.md` |
| 2026-11-12 | M4 12 | 殼給日期（`set_today`，沒給不寫假日期）、匯出 v3（種類欄、`last_used`、裝置 id）、v1/v2 照收；plan §2.4 改成 v3 草稿；見票 12 | 新 session 下 `/implement .scratch/m4/issues/13-learning-expiry.md` |
| 2026-11-12 | M4 05 | `bench --check` 擋原生每鍵 p95（空白、學過兩組），WebAssembly 只報；數字與翻頁 e2e 的既有競態見票 05 | 新 session 下 `/implement .scratch/m4/issues/01-debug-panel.md` |
| 2026-11-12 | M4 04 | 簡體模式的前文轉回繁體再接字級 bigram、詞級 trigram 與個人 bigram（剛上屏那段取原文）；bench 與 --learning 前後相同；見票 04 | 新 session 下 `/implement .scratch/m4/issues/01-debug-panel.md` |
| 2026-11-12 | M4 01 | debug 面板：核心 `Session::inspect`（不進 Snapshot）、網頁面板與重排模型總分；bench、--learning 前後逐行相同，延遲高負載下交替量在雜訊內；見票 01 | 新 session 下 `/implement .scratch/m4/issues/02-web-hardening.md` |
| 2026-11-12 | M4 13 | 一年時限（整串上屏另記日期、規則 2、3 只看一年內、一年沒用的停用不刪）、條目上限 2 萬列／3 萬組（LatinIME 1.2 倍）；v3 entry 12 欄；見票 13 | 新 session 下 `/implement .scratch/m4/issues/14-english-user-words.md` |
| 2026-11-12 | M4 06 | `Key::Weighted` 與 `qim-layout` 的空間模型（到形狀的距離加 2D Gaussian、點中的鍵一定最高）；網頁觸控送機率、面板顯示；bench 前後相同；見票 06 | 新 session 下 `/implement .scratch/m4/issues/07-physical-geometry.md` |
| 2026-11-12 | M4 11 | 萬用鍵 `z`：每個代一碼、任一位置，學習記實際的碼，面板標萬用鍵；整句首選約原樣七成、單字 1–2%（票 18 評估）；bench 前後相同；見票 11 | 新 session 下 `/implement .scratch/m4/issues/02-web-hardening.md` |
| 2026-11-12 | M4 05b | `qim_bench::run` 收 `Timers`，`run.rs` 改用數字固定的假原生計時器，load 61 下連跑三次都綠；補 `--check` 延遲接線與真實計時形狀的測試；見票 05b | 新 session 下 `/implement .scratch/m4/issues/05c-page-turn-e2e-race.md` |
| 2026-11-12 | M4 05c | 翻頁 e2e 的競態是測試的問題（重排翻頁中回來照 plan 延到第一頁）：兩個翻頁測試關掉重排，補翻頁中途回來的 e2e；見票 05c | 新 session 下 `/implement .scratch/m4/issues/02-web-hardening.md` |
| 2026-11-12 | M4 06b | 平手點都在鍵外（點擊判定照直線距離、Gaussian 各軸 σ 不同），照設計提到平手；測試格子補 Shift、刪除，種子進 repo 並加固定測試，產品沒改；見票 06b | 新 session 下 `/implement .scratch/m4/issues/07-physical-geometry.md` |
| 2026-11-12 | M4 14 | 英文使用者詞取代記住的字串：4 字元起都學、門檻 2／3 次、規則 4、5 改寫、v3 種類 `english`；bench 不變；見票 14 | 新 session 下 `/implement .scratch/m4/issues/15-backspace-undo.md` |
| 2026-11-12 | M4 02 | 換模型競態與模型載入搬進 `NeuralReranker`、`snapshot.free()` 收一處；IndexedDB 或 `localStorage` 打不開照樣能打字（只留記憶體、狀態列說明）；見票 02 | 新 session 下 `/implement .scratch/m4/issues/03-rerank-cache.md` |
| 2026-11-12 | M4 07 | 實體鍵盤送 `code`，`qim-layout` 的鄰鍵先驗轉成 `Weighted`；中文模式整個當美式鍵盤（使用者決定）；名詞改成按鍵機率；bench 前後相同；見票 07 | 新 session 下 `/implement .scratch/m4/issues/08-typo-bench.md` |
| 2026-11-12 | M4 15 | 退格撤銷：撤銷紀錄 `Undo` 以動作為單位，中英共用；照 librime 的 3 秒由殼計時（`expire_undo`）；Session 分開、無痕不撤銷；bench 不變；見票 15 | 新 session 下 `/implement .scratch/m4/issues/16-forget-candidate.md` |
| 2026-11-12 | M4 03 | 停鍵結果快取放核心：鍵是重排要求、上限 32；殼給 `set_reranker_id`，漏給只是沒快取；退格回算過的碼不等停鍵；見票 03 | 新 session 下 `/implement .scratch/m4/issues/16-forget-candidate.md` |
| 2026-11-12 | M4 16 | 撤銷學習：核心 `Session::forget`、網頁組字中 Shift+Delete／Ctrl+Delete 清目前頁第一個；個人 bigram 只清現在前文那組；第 2 位的自動造詞桌面清不到，留給 M5；見票 16 | 新 session 下 `/implement .scratch/m4/issues/17-sync-merge.md` |
| 2026-11-12 | M4 08 | 口語的「含打錯碼」「觸控模擬」集合（test、dev，決定性）、改錯比例（票 10 前是 0）、不容錯基線；〈容錯採用門檻〉使用者確認；見票 08 | 新 session 下 `/implement .scratch/m4/issues/09-tolerance-experiment.md` |
| 2026-11-13 | M4 17 | 跨裝置合併：各台次數相加、墓碑只存雜湊、版本取大、tick 對齊、清空也同步；匯入改合併，網頁說明合併幾筆；ADR 0007；收 7ad01da 草稿；見票 17 | 新 session 下 `/implement .scratch/m4/issues/09-tolerance-experiment.md` |
| 2026-11-13 | M4 09 | 容錯實驗：四種修正合起來沒過〈容錯採用門檻〉（修正次數、改錯、沒打錯退步、延遲），速成不做、票 10 wontfix；機制量完刪掉；見票 09 | 等使用者：期中檢查（票 18）試用第一期 |
| 2026-11-13 | M4 18 | 期中檢查：bench --check、--learning、check 全綠；使用者試用第一期可開第二期；插修復票 18b（萬用鍵延遲進閘門）、18c（聯想時退格就刪字）；見票 18 | 新 session 下 `/implement .scratch/m4/issues/18b-wildcard-latency.md` |
| 2026-11-13 | M4 18c | 有聯想時退格交還宿主刪字、聯想收起，照〈退格撤銷〉撤銷這次上屏；Esc 照舊；plan §2.1、§2.4、brief §4.5 與事實檔已改；bench 不變；見票 18c | 新 session 下 `/implement .scratch/m4/issues/19-gboard-english-blackbox.md` |
| 2026-11-13 | M4 18b | 含萬用鍵的輸入每鍵延遲：候選列重用開頭各段的 span、字頻排序只查一次，空白全部 p95 961→237 µs；`bench --check` 加這批輸入、同一組上限；見票 18b | 新 session 下 `/implement .scratch/m4/issues/19-gboard-english-blackbox.md` |
| 2026-11-13 | M4 流程 | 對齊 project-init：roadmap、事實編號、決策紀錄、票留 Session 紀錄由合併的人寫本檔、收尾票加使用者試；文件檢查換成 `scripts/check_docs.py` | 新 session 下 `/implement .scratch/m4/issues/19-gboard-english-blackbox.md` |
| 2026-11-13 | M4 流程 | 里程碑表的狀態改成固定詞開頭（已驗收），`arch-diet` 佔一列；`scripts/check_docs.py` 換成新版：有票的目錄要列在表裡、票要有 `## 驗收` | 新 session 下 `/implement .scratch/m4/issues/19-gboard-english-blackbox.md` |
| 2026-11-14 | M4 09b | Gboard 式鄰鍵讀法：觸控修正 −12.5%，點對鍵時 +5.6–8.4%，延遲在上限內；機制已刪；另修 hook 寫壞 index；使用者決定採用（票 09c）；見票 09b | 新 session 下 `/implement .scratch/m4/issues/09c-touch-neighbor-readings.md` |
