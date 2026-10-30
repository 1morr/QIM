# 進度

工作單位是票（`.scratch/<里程碑>/issues/NN-slug.md`），狀態在票的 `Status:` 行。這裡只記各里程碑的閱讀指引、進行中里程碑的 session 紀錄、偏差與決定。

## 寫法與封存

- session 紀錄一列、偏差一條，各不超過 200 字；細節在票的 `## Comments`、brief、plan 或 ADR，這裡只放指向它們的一句話。
- session 紀錄的「票」欄與偏差每條的開頭都標里程碑（`M0 01`、`2026-10-30 M0 票 01：…`），封存時靠它分。
- 里程碑驗收時，把它的 session 紀錄、偏差，連同 plan〈里程碑任務〉裡它的那一節，整段移到 `docs/archive/<里程碑>.md`，里程碑表的備註換成那個檔的連結。本檔與 plan 只留進行中和之後的里程碑。
- 以上由 M0 票 01 的閘門強制（plan §6）；閘門接上之前靠收尾時自查。

## 里程碑與閱讀指引

| 里程碑 | 票目錄 | 拆票前讀 | 備註 |
| --- | --- | --- | --- |
| M0 | `.scratch/m0/issues/` | brief §3、§5、§6；plan §0、§1、§4.1、§4.5、§6、§7.1；ADR 0001–0003 | 進行中。未拆票，下一步 `/to-tickets docs/plan.md` |
| M1 | `.scratch/m1/issues/` | brief §1.5、§4、§5、§9；plan §2–§5、§7.2 | 未開始 |

## Session 紀錄

| 日期 | 票 | 結果 | 下一步 |
| --- | --- | --- | --- |
| 2026-10-30 | M0 框架 | 參照 Berth 建立文件骨架：`CLAUDE.md`、brief 與 plan 骨架、本檔、`docs/agents/` | 新對話跑 `/grill-with-docs` 填 brief，再寫 plan |
| 2026-10-30 | M0 框架 | 加入 progress 的寫法與封存規則（Berth 的 progress.md 長到 884 KB，session 讀不完）；brief、plan 只寫現況 | 同上 |
| 2026-10-30 | M0 研究 | 階段 1–2：六份研究進 `docs/research/`，事實摘進 brief §12；三個 Rime 案例找到根因並可用 `scripts/experiments/rime-cases/` 重跑；寫 `ime-primer.md` | 使用者看過 primer 後跑階段 3 `/grill-with-docs`，從 primer §6 的六題問起 |
| 2026-10-30 | M0 brief | 階段 3 訪談三輪：brief §0–§11 填完、`GLOSSARY.md`、ADR 0001–0003（自寫 Rust 核心、prototype 共用核心、速成碼建置時推導） | 使用者確認 brief 後寫 plan（階段 4） |
| 2026-10-30 | M0 plan | 寫 `docs/plan.md`：架構、目錄、依賴規則、測試策略（首選字基準）、M0–M1 任務與驗收；研究檔 email 脫敏並改寫本機歷史 | `/to-tickets docs/plan.md` 拆 M0 票 |

## 偏差與決定

- 2026-10-30 M0 框架：名詞表用 `GLOSSARY.md`（mattpocock skills 現行慣例），不沿用 Berth 的 `CONTEXT.md`。
- 2026-10-30 M0 框架：技術棧未定，lint、CI、commit-msg hook 等閘門延到 M0 票 01（plan §6）。
- 2026-10-30 M0 框架：progress 封存到 `docs/archive/` 而不刪除。plan、測試、ADR 可能引用某條偏差當理由，刪掉就斷了；封存檔不在每個 session 的閱讀範圍內，不佔上下文。
- 2026-10-30 M0 plan：字級 bigram 改用中文 / 粵語維基自行訓練（plan §4.3），不用 LGPL 的 rime-octagram-data：可同時產生首選字基準的 test 切分，格式自己掌握；CC BY-SA 的影響列入 brief §11。
- 2026-10-30 M0 plan：brief §8 由「預設不連網」改為網路是正式功能（使用者聯網接受度高），保留三條硬限制；連帶改 §1.2、§5.3（資料包可線上更新）。雲端 LLM 用使用者自帶 key；崩潰回報可選、預設關。
- 2026-10-30 M0 plan：plan 加入功能里程碑 M4 智慧候選、M6 輸入增強、M7 同步與資料，平台順延為 M5 Android、M8 Windows、M9 Linux；M2 加 LLM 評估實驗（plan §7.4）。
