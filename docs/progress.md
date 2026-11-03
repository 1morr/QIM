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
| M1 | `.scratch/m1/issues/` | brief §1.5、§4、§5、§9；plan §2–§5、§7.2；ADR 0004–0006；`.scratch/m0/issues/09-m0-close.md` 的 Comments（留給 M1 各票的發現） | 進行中，尚未拆票 |

## Session 紀錄

| 日期 | 票 | 結果 | 下一步 |
| --- | --- | --- | --- |
| 2026-11-04 | M1 封存 M0 | 票 09：里程碑 code-review、乾淨 checkout 重驗 spec〈驗收〉全過；M0 封存到 `docs/archive/M0.md`，M1 標進行中；細節見票 09 | 新 session 下 `/to-tickets docs/plan.md 拆 M1 的票` |
| 2026-11-04 | M1 plan | M0 review 後依使用者同意改 plan §5.2、§7.2：語料管線與基準先行、拆 bigram 與佈局、學習模式、個人集改用 Rime 學習資料；brief §9、§12.2 同步 | 新 session 下 `/to-tickets docs/plan.md 拆 M1 的票` |
| 2026-11-04 | M1 plan | 收到使用者小狼毫的 `quick5.userdb`，轉成 `bench/personal/rime-userdb.txt`（本機）；查出 Rime 記的是倉頡全碼而非按鍵，plan §5.2 改由自己轉碼 | 新 session 下 `/to-tickets docs/plan.md 拆 M1 的票` |

## 偏差與決定

- 2026-11-04 M1 plan：M1 改成語料管線 → 首選字基準 → 詞庫 → 連打…（plan §7.2）：原票 1 的字頻要用票 3 的語料，順序矛盾；先建基準，每張票才有前後數字可比。
- 2026-11-04 M1 plan：bigram 拆成訓練、調參兩項，佈局拆成格式、觸控鍵盤兩項；M1 各項改用名稱稱呼，票號拆票時定。
- 2026-11-04 M1 plan：組字中按鍵殘留（`z`、Enter、標點時 `handled` 為假但組字還在）由票 09 的去處 M1 票 8、9 提前到〈連打〉，因為連打後組字變長、更常碰到。
- 2026-11-04 M1 plan：個人集由「≥ 100 句真實句子」改為使用者 Windows 小狼毫的 Rime 學習資料（使用者提供），句子改選填；內容只在本機、不送雲端（plan §5.2）。
- 2026-11-04 M1 plan：基準加依長度分組、翻盤清單、`--learning` 學習模式（plan §5.2）；`fixed.tsv` 起始只放已過的 `a 日`，`anr`、`mir` 在通過時加入，`--check` 才不會一開始就紅。
- 2026-11-04 M1 plan：個人集的輸入碼改由 `qim-databuild` 轉碼，不用 Rime 記下的碼：實測那是詞典的倉頡全碼（99.5%），不是使用者按的鍵，看不出打不打全碼（brief §12.2）。
