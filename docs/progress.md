# 進度

工作單位是票（`.scratch/<里程碑>/issues/NN-slug.md`），狀態在票的 `Status:` 行。這裡只記各里程碑的閱讀指引、進行中里程碑的 session 紀錄、偏差與決定。

## 寫法與封存

- session 紀錄一列、偏差一條，整行各不超過 200 字元；細節在票的 `## Comments`、brief、plan 或 ADR，這裡只放指向它們的一句話。
- session 紀錄的「票」欄與偏差每條的開頭都標里程碑（`M0 01`、`2026-10-30 M0 票 01：…`），封存時靠它分。
- 里程碑驗收時，把它的 session 紀錄與偏差整段移到 `docs/archive/<里程碑>.md`，里程碑表的備註換成那個檔的連結。同一個 commit 把下一個里程碑標成進行中，封存這個 session 的紀錄標下一個里程碑，「下一步」寫 `/1morr:checkpoint <封存的里程碑>`（它結束時再寫下一個里程碑的開工指令）。本檔只留進行中的里程碑。
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

## 偏差與決定

- 2026-11-13 M4 票 18（使用者決定）：期中檢查試用第一期通過；容錯暫時放棄、萬用鍵單字不補強、條目上限不變；含萬用鍵的輸入先優化再進閘門（18b）、聯想時退格就刪字（18c）；見票 18。
- 2026-11-13 M4 票 09：容錯沒過〈容錯採用門檻〉（修正次數、改錯、沒打錯不退步、延遲），速成不做、票 10 wontfix；改 brief §1.5、§4.8、§7、§12.4，plan §2.1、§2.5、§2.6、§5.2，spec；見票 09。
- 2026-11-13 M4 票 09（協調者決定，偏離 spec「過門檻才進核心」的字面）：容錯機制先進 qim-core 預設關量（`117252a`），沒過門檻，同票刪掉、核心回到開工時。
- 2026-11-13 M4 票 17（協調者決定）：加「從備份還原」（新的清空記號再收備份），和合併的匯入分開；清空時別台未同步的新學習被清掉是接受的代價；改 plan §2.4、§3.1、ADR 0007。
- 2026-11-13 M4 票 17（使用者決定）：清空學習資料同步到別台：留清空記號（代數），合併時代數大的整份算數、清空前學的不回來；推翻第一版「清空只清這台」；改 plan §2.4、ADR 0007。
- 2026-11-13 M4 票 17（協調者替使用者決定）：墓碑只存鍵的 SHA-256（前 16 位元組），維持 M3 票 14「刪掉的常用語不留在匯出檔」；改 plan §2.4，ADR 0007。
- 2026-11-13 M4 票 17（協調者決定）：照收 main 7ad01da 的 v3 草稿（網頁存檔），次數記在檔頭的裝置；更早的草稿不收；改 plan §2.4。
- 2026-11-13 M4 票 17：匯入改成合併（網頁讀回自己的存檔仍是取代 `loadUserDict`）；v3 加各台次數、墓碑、版本與 `deleted` 種類；改 plan §2.4、§3.1，brief §7。
- 2026-11-12 M4 票 08（使用者確認）：〈容錯採用門檻〉照提案寫進 spec：打錯的題首選 ≥ 10%／觸控 ≥ 6%、修正少 10%，沒打錯的不退步，改錯 ≤ 1.0%。
- 2026-11-12 M4 票 08：含打錯碼的替換 65% 鄰鍵、35% 非鄰鍵（票寫鄰鍵等機率，照 Grudin 1983）；基準照殼送實體鍵的按鍵機率，`qim-bench` 加依賴 `qim-layout`；改 plan §1.3、§5.2。
- 2026-11-12 M4 票 16（協調者決定）：撤銷學習刪資料從嚴，個人 bigram 只清現在前文那組；桌面按鍵照 librime （Ctrl+Delete，另收 Shift+Delete）清目前頁第一個；改 plan §2.1、§2.4、§3.1、§7.9，brief §4.3、§4.5、§12.2、§12.4。
- 2026-11-12 M4 票 03（協調者要求）：快取放核心，殼經 `set_reranker_id` 給重排識別、漏給只是沒快取；`rerank_request` 改 `&mut`（記下給出的要求）；改 plan §0、§2.1、§2.7、§3.1。
- 2026-11-12 M4 票 15（協調者決定）：退格撤銷照 librime 保留 3 秒時限，核心不讀時間、由殼計時（`Session::expire_undo`，網頁 `undo-window.ts`）；改 plan §2.4、brief §12.2。
- 2026-11-12 M4 票 07：名詞「觸控機率」改成按鍵機率（含實體鍵盤），新增鄰鍵先驗；`touch_weights` → `key_weights`、面板 `debug-weights`；改名詞表、plan §0、§2.1、§3.1。
- 2026-11-12 M4 票 07（使用者決定）：中文模式整個當美式鍵盤，字母與標點都照實體鍵位（`code`），英文直通照配列（臨時英文同樣照配列，使用者同日確認）；改 brief §4.8、plan §2.1。
- 2026-11-12 M4 票 02（協調者決定）：票誤寫學習資料在 IndexedDB（實際在 `localStorage`）；本票改成兩種儲存打不開都照樣能打字、存不了的只留記憶體；改 plan §3.1、§4.6。
- 2026-11-12 M4 票 14（協調者決定）：中文組字的前綴補完只取英文詞典沒有的英文使用者詞（過 3 次、最常用 20 條），詞典詞走規則 4 與第二期的英文建議列，免得 that、have 把 email 擠出去；改 plan §2.4 規則 5、brief §4.7。
- 2026-11-12 M4 票 14：記住的字串併入英文使用者詞（plan §2.1、§2.4、brief、名詞表）；v3 草稿 `string` 換成 `english`、舊列略過；v2 的記住的字串匯入成次數 3；與中文的列共用條目上限。
- 2026-11-12 M4 票 11：沒有組字時 `z` 改為開始組字（brief §4.3 原本交還宿主），不設開關；單獨的 `z` 代一碼字、每段最多兩個；協調者補 `Blocked by: 01`；見票 11。
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
