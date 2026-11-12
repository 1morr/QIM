# 進度

工作單位是票（`.scratch/<里程碑>/issues/NN-slug.md`），狀態在票的 `Status:` 行。這裡只記各里程碑的閱讀指引、進行中里程碑的 session 紀錄、偏差與決定。

## 寫法與封存

- session 紀錄一列、偏差一條，整行各不超過 200 字元；細節在票的 `## Comments`、brief、plan 或 ADR，這裡只放指向它們的一句話。
- session 紀錄的「票」欄與偏差每條的開頭都標里程碑（`M0 01`、`2026-10-30 M0 票 01：…`），封存時靠它分。
- 里程碑驗收時，把它的 session 紀錄與偏差整段移到 `docs/archive/<里程碑>.md`，里程碑表的備註換成那個檔的連結。同一個 commit 把下一個里程碑標成進行中，封存這個 session 的紀錄標下一個里程碑。本檔只留進行中的里程碑。
- 里程碑的閱讀範圍、票序等細節寫在 `.scratch/<里程碑>/spec.md`，里程碑表的備註只寫一句狀態。spec 與票留在原處、不搬：它們是歷史快照，別處用路徑引用。
- `cargo xtask check` 的 `docs` 步驟強制 plan §6 列的規則：`##` 段、session 紀錄一整張表、長度、里程碑標記、最後一列的下一步、`.scratch/` 的 spec。其餘靠收尾自查：細節不寫進本檔、何時封存、備註只寫一句、spec 與票不搬。

## 里程碑與閱讀指引

| 里程碑 | 票目錄 | 拆票前讀 | 備註 |
| --- | --- | --- | --- |
| M0 | `.scratch/m0/issues/` | brief §3、§5、§6；plan §0、§1、§4.1、§4.5、§6；ADR 0001–0003；`.scratch/m0/spec.md` | 完成，[已封存](archive/M0.md) |
| M1 | `.scratch/m1/issues/` | brief §1.5、§4、§5、§9；plan §2–§5、§7.2；ADR 0004–0006；`.scratch/m0/issues/09-m0-close.md` 的 Comments（留給 M1 各票的發現） | 完成，[已封存](archive/M1.md) |
| M2 | `.scratch/m2/issues/` | brief §9、§11、§12.3；plan §1.4、§3、§7.3；ADR 0002、0005；`.scratch/m1/issues/24-m1-close.md` 的 Comments（留給 M2 起各里程碑的發現）；`.scratch/m2/spec.md` | 完成，[已封存](archive/M2.md) |
| M3 | `.scratch/m3/issues/` | brief §1.5、§7、§8、§12.3；plan §2.3、§2.7、§7.4、§7.8；ADR 0005；`docs/research/ios-feasibility.md`；`.scratch/m1/issues/24-m1-close.md` 與 `.scratch/m2/issues/12-m2-close.md` 的 Comments（標 M3 的各條） | 完成，[已封存](archive/M3.md) |
| M4 | `.scratch/m4/issues/` | brief §4.7、§4.8、§7、§12.4；plan §2.1、§2.3、§2.4、§2.6、§3.1、§4.4、§7.5、§7.9；`docs/research/ranking-techniques.md` §5；`.scratch/m3/issues/23-m3-close.md` 的 Comments（標 M4 的各條；M1、M2 收尾票標 M4 的也讀） | 進行中，已拆 31 張票（第一期 01–18、第二期 19–30、收尾 31） |

## Session 紀錄

| 日期 | 票 | 結果 | 下一步 |
| --- | --- | --- | --- |
| 2026-11-12 | M4 封存 M3 | 票 23：里程碑 code-review（修匯出 tick、補個人 bigram e2e 等）、乾淨 checkout 重驗 spec〈驗收〉、匯出 v2 定稿；M3 封存到 `docs/archive/M3.md`；見票 23 | 新 session 下 `/to-tickets docs/plan.md 拆 M4 的票` |
| 2026-11-12 | M4 檢視 | 回顧 M3、畫架構與流程圖；子代理審查與研究（兩份筆記）；使用者同意 D1–D8，plan §7.5 重寫成兩期、§7.9、brief §4.5 已改 | 新 session 下 `/to-tickets docs/plan.md 拆 M4 的票` |
| 2026-11-12 | M4 拆票 | plan §7.5 細化成 `.scratch/m4/spec.md`（每鍵延遲上限、§7.5 細節搬進 spec〈細節〉）；使用者同意切法，31 張票 | 新 session 下 `/implement .scratch/m4/issues/01-debug-panel.md` |
| 2026-11-12 | M4 12 | 殼給日期（`set_today`，沒給不寫假日期）、匯出 v3（種類欄、`last_used`、裝置 id）、v1/v2 照收；plan §2.4 改成 v3 草稿；見票 12 | 新 session 下 `/implement .scratch/m4/issues/13-learning-expiry.md` |
| 2026-11-12 | M4 05 | `bench --check` 擋原生每鍵 p95（空白、學過兩組），WebAssembly 只報；數字與翻頁 e2e 的既有競態見票 05 | 新 session 下 `/implement .scratch/m4/issues/01-debug-panel.md` |
| 2026-11-12 | M4 04 | 簡體模式的前文轉回繁體再接字級 bigram、詞級 trigram 與個人 bigram（剛上屏那段取原文）；bench 與 --learning 前後相同；見票 04 | 新 session 下 `/implement .scratch/m4/issues/01-debug-panel.md` |
| 2026-11-12 | M4 01 | debug 面板：核心 `Session::inspect`（不進 Snapshot）、網頁面板與重排模型總分；bench、--learning 前後逐行相同，延遲高負載下交替量在雜訊內；見票 01 | 新 session 下 `/implement .scratch/m4/issues/02-web-hardening.md` |
| 2026-11-12 | M4 13 | 一年時限（整串上屏另記日期、規則 2、3 只看一年內、一年沒用的停用不刪）、條目上限 2 萬列／3 萬組（LatinIME 1.2 倍）；v3 entry 12 欄；見票 13 | 新 session 下 `/implement .scratch/m4/issues/14-english-user-words.md` |
| 2026-11-12 | M4 06 | `Key::Weighted` 與 `qim-layout` 的空間模型（到形狀的距離加 2D Gaussian、點中的鍵一定最高）；網頁觸控送機率、面板顯示；bench 前後相同；見票 06 | 新 session 下 `/implement .scratch/m4/issues/07-physical-geometry.md` |
| 2026-11-12 | M4 05b | `qim_bench::run` 收 `Timers`，`run.rs` 改用數字固定的假原生計時器，load 61 下連跑三次都綠；補 `--check` 延遲接線與真實計時形狀的測試；見票 05b | 新 session 下 `/implement .scratch/m4/issues/05c-page-turn-e2e-race.md` |
| 2026-11-12 | M4 06b | 平手點都在鍵外（點擊判定照直線距離、Gaussian 各軸 σ 不同），照設計提到平手；測試格子補 Shift、刪除，種子進 repo 並加固定測試，產品沒改；見票 06b | 新 session 下 `/implement .scratch/m4/issues/07-physical-geometry.md` |

## 偏差與決定

- 2026-11-12 M4 票 06（協調者要求）：空間模型保證點中的鍵的字母機率最高（不是就提到和最高的平手、排第一）；plan §2.1 的 `Weighted(Vec)` 改成 `Weights` 型別，§2.6 加〈空間模型〉。
- 2026-11-12 M4 票 13（使用者決定）：一年沒用的條目停用、不刪（時鐘錯了改回來要恢復、舊備份照收），實體刪除只在到上限時淘汰；改 plan §2.4、brief §4.5、spec〈細節〉。
- 2026-11-12 M4 票 13（協調者決定）：中文條目沒有出現門檻，上限淘汰「先刪沒到門檻的」由票 14 驗；v3 的 entry 加 `last_commit_date`（12 欄）；改 plan §2.4、brief §4.5。
- 2026-11-12 M4 票 12（協調者要求）：殼沒給日期時條目沒有日期（`Option`，匯出留空），不寫 1970-01-01，票 13 的時限跳過它們；plan §2.4 加殼的日期契約。
- 2026-11-12 M4（拆票，使用者決定）：每鍵延遲改成會擋，原生 p95 空白 ≤ 1 ms／選開頭一段 ≤ 5 ms、學過 ≤ 3／10 ms（spec〈每鍵延遲上限〉）；第二期的研究票也等期中檢查。
- 2026-11-12 M4（檢視，使用者決定）：照子代理審查改 D1–D8：學習時限改一年日曆、退格撤銷、v3 各台計數相加、改刪字的 Snapshot、M4 分兩期；改 brief §4.5、plan §7.5、§7.9。
- 2026-11-12 M4（檢視，使用者決定）：學習次數只算 1 萬次上屏內（推翻 brief §4.5 間隔多久都算）、條目設上限；英文使用者詞 4 字元起、2 次才出；改 brief §4.5、plan §7.5。
- 2026-11-12 M4（檢視，使用者決定）：記住的字串併入英文使用者詞（照 LatinIME 上屏都學），中文組字完整串排第 2、前綴補完限最常用 20 條；數字與卡號照學、改用出現門檻；plan §7.5。
- 2026-11-12 M4（檢視）：滑動輸入、空白鍵移游標、中文滑動評估移到 M5；匯出升 v3 含墓碑；英文模式另建資料包並寫 ADR；改 plan §7、§7.5、§7.9。
- 2026-11-12 M4（M3 票 23）：無痕時滑鼠移游標後數字後半形的布林不更新，照 brief §8 無痕不讀上下文記為代價、不改；改 plan §2.1；見票 23。
- 2026-11-12 M4：使用者試網頁後要求 debug 面板（M4 第一張票）、觸控中英鍵與英文鍵盤、英文模式學打過的字串（不限 email）、記住的 email 出在英文建議列；寫進 plan §7.5，brief §4.4 的觸控限制註明排在 M4。
- 2026-11-12 M4（使用者決定插入 M5 觸控鍵盤與設定）：網頁觸控鍵盤做到平台樣板（Gboard 盤點、候選列與工具列接在鍵盤頂端、鍵盤內設定頁、各面板），原 M5–M10 順延成 M6–M11；plan §3.1、§7、§7.9 與現行文件、程式註解已改，M0–M3 的舊票與封存保留舊編號（plan §7 有對照）。
