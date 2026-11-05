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
| M2 | `.scratch/m2/issues/` | brief §9、§11、§12.3；plan §1.4、§3、§7.3；ADR 0002、0005；`.scratch/m1/issues/24-m1-close.md` 的 Comments（留給 M2 起各里程碑的發現） | 進行中，尚未拆票 |

## Session 紀錄

| 日期 | 票 | 結果 | 下一步 |
| --- | --- | --- | --- |
| 2026-11-04 | M2 封存 M1 | 票 24：里程碑 code-review、乾淨 checkout 重驗 spec〈驗收〉全過、黑箱在 QIM 重跑；M1 封存到 `docs/archive/M1.md`，M2 標進行中；細節見票 24 | 新 session 下 `/to-tickets docs/plan.md 拆 M2 的票` |
| 2026-11-05 | M2（補 M1 25） | 單按 Shift 照 Rime：左 Shift 進臨時英文、右 Shift 先上屏，`Options.shift_switch` 三選一、網頁有選單；細節見票 25 | M1 26、27 也做完後，新 session 下 `/to-tickets docs/plan.md 拆 M2 的票` |
| 2026-11-05 | M2（補 M1 27） | 票 27：觸控佈局照 Gboard 速成實測（Shift、符號三頁、配色、數字列開關、長按只出字母）；組字中的分隔鍵另開補票，見票 27 | 補票都做完後，新 session 下 `/to-tickets docs/plan.md 拆 M2 的票` |
| 2026-11-05 | M2（補 M1 25b） | 臨時英文改照舊版 librime（1.17.0 以前）：空白接進英文、Enter 上屏整段，再按留碼的 Shift 回中文組字；細節見票 25b | M1 17c、26、27 也做完後，新 session 下 `/to-tickets docs/plan.md 拆 M2 的票` |

## 偏差與決定

- 2026-11-05 M2（補 M1 25）：組字中單按 Shift 改照 Rime（左留碼接英文、右先上屏），設定可改；使用者試用後決定，推翻 M1 票 23 的左右都先上屏（brief §4.3）。
- 2026-11-05 M2（補 M1 27）：長按字母鍵不再出數字，改由設定的數字列（使用者決定，推翻 brief §4.4「第一排另含數字」）；佈局格式加 `"shift"` 動作，只加不改、不升版本（plan §2.6）。
- 2026-11-05 M2（補 M1 25b）：臨時英文的空白接進英文、再按留碼的 Shift 回中文組字，照使用者用慣的舊版 librime；使用者決定，推翻 M1 票 25 照新版的做法；偏離舊版的四處使用者接受（brief §4.3）。
