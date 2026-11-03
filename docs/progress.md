# 進度

工作單位是票（`.scratch/<里程碑>/issues/NN-slug.md`），狀態在票的 `Status:` 行。這裡只記各里程碑的閱讀指引、進行中里程碑的 session 紀錄、偏差與決定。

## 寫法與封存

- session 紀錄一列、偏差一條，各不超過 200 字；細節在票的 `## Comments`、brief、plan 或 ADR，這裡只放指向它們的一句話。
- session 紀錄的「票」欄與偏差每條的開頭都標里程碑（`M0 01`、`2026-10-30 M0 票 01：…`），封存時靠它分。
- 里程碑驗收時，把它的 session 紀錄與偏差整段移到 `docs/archive/<里程碑>.md`，里程碑表的備註換成那個檔的連結。本檔只留進行中的里程碑。
- 里程碑的閱讀範圍、票序等細節寫在 `.scratch/<里程碑>/spec.md`，里程碑表的備註只寫一句狀態。spec 與票留在原處、不搬：它們是歷史快照，別處用路徑引用。
- 以上由 M0 票 01 的閘門強制（plan §6）；閘門接上之前靠收尾時自查。

## 里程碑與閱讀指引

| 里程碑 | 票目錄 | 拆票前讀 | 備註 |
| --- | --- | --- | --- |
| M0 | `.scratch/m0/issues/` | brief §3、§5、§6；plan §0、§1、§4.1、§4.5、§6、§7.1；ADR 0001–0003 | 進行中。未拆票，下一步在新 session 下 `/to-tickets docs/plan.md 拆 M0 的票` |
| M1 | `.scratch/m1/issues/` | brief §1.5、§4、§5、§9；plan §2–§5、§7.2；ADR 0004–0006 | 未開始 |

## Session 紀錄

| 日期 | 票 | 結果 | 下一步 |
| --- | --- | --- | --- |
| 2026-10-30 | M0 框架 | 參照 Berth 建立文件骨架：`CLAUDE.md`、brief 與 plan 骨架、本檔、`docs/agents/` | 新對話跑 `/grill-with-docs` 填 brief，再寫 plan |
| 2026-10-30 | M0 框架 | 加入 progress 的寫法與封存規則（Berth 的 progress.md 長到 884 KB，session 讀不完）；brief、plan 只寫現況 | 同上 |
| 2026-10-30 | M0 研究 | 階段 1–2：六份研究進 `docs/research/`，事實摘進 brief §12；三個 Rime 案例找到根因並可用 `scripts/experiments/rime-cases/` 重跑；寫 `ime-primer.md` | 使用者看過 primer 後跑階段 3，從 primer §6 的六題問起 |
| 2026-10-30 | M0 brief | 階段 3 訪談三輪：brief §0–§11 填完、`GLOSSARY.md`、ADR 0001–0003（自寫 Rust 核心、prototype 共用核心、速成碼建置時推導） | 使用者確認 brief 後寫 plan（階段 4） |
| 2026-10-30 | M0 plan | 寫 `docs/plan.md`：架構、目錄、依賴規則、測試策略（首選字基準）、M0–M1 任務與驗收；研究檔 email 脫敏並改寫本機歷史 | `/to-tickets docs/plan.md` 拆 M0 票 |
| 2026-10-31 | M0 plan | 依使用者要求重排路線圖：網頁做完核心功能並經使用者驗收，才開工平台殼（plan §7、brief §3） | 新 session 下 `/to-tickets docs/plan.md 拆 M0 的票` |
| 2026-10-31 | M0 plan | 加入畫布式佈局與佈局編輯器（brief §4.6、plan §2.6、§7.6、ADR 0004） | 開新 session 開工 M0：細化 spec 再拆票 |
| 2026-10-31 | M0 框架 | 開工改成一個指令做完；所有 session 結尾都給下一步的完整指令（`CLAUDE.md`〈每個 session 的流程〉），閘門列入 plan §6 | 新 session 下 `/to-tickets docs/plan.md 拆 M0 的票` |
| 2026-10-31 | M0 plan | 審 plan 缺口（未寫回）：學習未進 lattice、選兩次成首選的門檻算不到、佈局缺方向與深色、無空間模型與小神經重排、語料偏書面、黑箱測試未排 | 等使用者：決定這些缺口是否照建議寫進 plan |
| 2026-10-31 | M0 plan | 上述缺口照建議寫回 plan、brief、`GLOSSARY.md`、ADR 0004–0006：個人模型進 lattice、佈局版本與深色、模型階梯實驗、空間模型、口語語料、修正次數、黑箱基線 | 新 session 下 `/to-tickets docs/plan.md 拆 M0 的票` |
| 2026-10-31 | M0 研究 | 查 CyIME 的外掛與模型評估，寫 `docs/research/cyime.md`；plan 加觸控平台實體鍵盤、Android 懸浮鍵盤、可選下載的詞級 trigram 對照 | 等使用者：決定擴充架構（資料擴充包、服務接縫、暫不做腳本外掛） |
| 2026-10-31 | M0 plan | 擴充架構維持原案，腳本外掛列待評估；M3 線上更新時才加 manifest，另補撤銷學習語意與基準防洩漏 | 新 session 下 `/to-tickets docs/plan.md 拆 M0 的票` |
| 2026-10-31 | M0 plan | 審新增輸入方案的路徑：學習鍵與匯出加輸入方案、佈局標籤可取自方案、詞庫碼索引分方案，音碼方案的工作寫進 plan §2.5 | 新 session 下 `/to-tickets docs/plan.md 拆 M0 的票` |
| 2026-10-31 | M0 plan | 定新方案的發佈方式：必需且不大的資料內建，大模型主 App 下載，資料包可從檔案匯入，程式碼只隨 App 發版（brief §5.3、plan §2.5） | 新 session 下 `/to-tickets docs/plan.md 拆 M0 的票` |
| 2026-11-03 | M0 研究 | 讀 xsime（librime 五端前端）寫 `docs/research/xsime.md`；平台殼做法與簽章查證摘進 brief §12.4，plan §7.7 的 M6–M9 加開工時要做的檢查 | 新 session 下 `/to-tickets docs/plan.md 拆 M0 的票` |

## 偏差與決定

- 2026-10-30 M0 框架：名詞表用 `GLOSSARY.md`（mattpocock skills 現行慣例），不沿用 Berth 的 `CONTEXT.md`。
- 2026-10-30 M0 框架：技術棧未定，lint、CI、commit-msg hook 等閘門延到 M0 票 01（plan §6）。
- 2026-10-30 M0 框架：progress 封存到 `docs/archive/` 而不刪除。plan、測試、ADR 可能引用某條偏差當理由，刪掉就斷了；封存檔不在每個 session 的閱讀範圍內，不佔上下文。
- 2026-10-30 M0 plan：字級 bigram 改用中文 / 粵語維基自行訓練（plan §4.3），不用 LGPL 的 rime-octagram-data：可同時產生首選字基準的 test 切分，格式自己掌握；CC BY-SA 的影響列入 brief §11。
- 2026-10-30 M0 plan：brief §8 由「預設不連網」改為網路是正式功能（使用者聯網接受度高），保留三條硬限制；連帶改 §1.2、§5.3（資料包可線上更新）。雲端 LLM 用使用者自帶 key；崩潰回報可選、預設關。
- 2026-10-30 M0 plan：plan 加入功能里程碑 M4 智慧候選、M6 輸入增強、M7 同步與資料，平台順延為 M5 Android、M8 Windows、M9 Linux；M2 加 LLM 評估實驗（plan §7.4）。
- 2026-10-30 M0 框架：里程碑規格改放 `.scratch/<里程碑>/spec.md`（學 Berth docs-diet），plan §7 只留路線圖。Berth 的票序寫在里程碑表備註，一格長成一段。
- 2026-10-31 M0 框架：spec 由使用者在新 session 下 `/to-tickets docs/plan.md` 時先寫、接著拆票，一個指令做完；之後的里程碑留在 plan §7。預先寫好的 spec 會因重排里程碑要改目錄名，內容也會在開工前過時。
- 2026-10-31 M0 plan：路線圖改為 M2 平台可行性實驗、M3 智慧候選、M4 輸入增強（含使用者驗收關卡）、M5 macOS、M6 iOS、M7 Android、M8 Windows、M9 Linux。同步與資料拆入 M3、M4 與各平台。理由：核心穩定後平台一次接上，避免半成品反覆改。
- 2026-10-31 M0 plan：佈局從按排的矩形改為畫布模型，允許重疊與亂序（ADR 0004）；新增 M5 佈局編輯器，使用者驗收關卡移到 M5 結尾，平台順延為 M6 macOS、M7 iOS、M8 Android、M9 Windows、M10 Linux。
- 2026-10-31 M0 plan：學習門檻改看不衰減的上屏次數，衰減值只決定個人分：衰減後選兩次約 1.995，到不了舊規則的 2，brief 的「選兩次成首選」不會觸發。學過的段進 lattice 加分（plan §2.3、§2.4）。
- 2026-10-31 M0 plan：LLM 評估實驗擴大成模型階梯實驗，同步層、小型神經重排都要量（plan §2.7、§7.8，ADR 0005）。品質判斷改以口語與個人集為主，維基只防退步（brief §5.2、§9）。
- 2026-10-31 M0 plan：不逆向閉源輸入法，改排黑箱基線 M1 票 00；開源專案隨功能在里程碑開工時深讀（ADR 0006）。懸浮鍵盤由 M5 改到 M8 評估：要平台支援，M5 只做單手佈局。
- 2026-10-31 M0 研究：懸浮鍵盤由 M8 評估改為 M8 要做（只限 Android）：使用者想要，CyIME 已在 Android 做到（`docs/research/cyime.md` §4）。
- 2026-10-31 M0 plan：brief §6「方案檔只調參數」改為音碼方案另需讀音資料與新的查碼方式；形碼才是只調參數。學習資料與佈局標籤現在就分方案，因為這兩個格式在 M1 定稿，之後改要轉換使用者資料。
