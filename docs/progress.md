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
| M3 | `.scratch/m3/issues/` | brief §1.5、§7、§8、§12.3；plan §2.3、§2.7、§7.4、§7.8；ADR 0005；`docs/research/ios-feasibility.md`；`.scratch/m1/issues/24-m1-close.md` 與 `.scratch/m2/issues/12-m2-close.md` 的 Comments（標 M3 的各條） | 進行中，已拆 23 張票（`.scratch/m3/spec.md`） |

## Session 紀錄

| 日期 | 票 | 結果 | 下一步 |
| --- | --- | --- | --- |
| 2026-11-10 | M3 封存 M2 | 票 12：里程碑 code-review、乾淨 checkout 重驗 spec〈驗收〉、iPhone 重打「門口」；brief §9、§11 寫回；M2 封存到 `docs/archive/M2.md`；見票 12 | 新 session 下 `/to-tickets docs/plan.md 拆 M3 的票` |
| 2026-11-11 | M3 拆票 | 寫 `.scratch/m3/spec.md`（含 plan §7.8 搬來的階梯實驗），拆 23 張票；plan §7.4、§7.8 縮成連結 | 新 session 下 `/implement .scratch/m3/issues/01-web-modules.md` |
| 2026-11-11 | M3 04 | `set_context`、選開頭一段後接選定段、bigram 加句首、「有前文」集合（給前文合計答錯與修正都較少，dev 重比結論相同）；見票 04 | 新 session 下 `/implement .scratch/m3/issues/03-ranking-rule-guards.md` |
| 2026-11-11 | M3 01 | `main.ts` 拆成候選列、面板、設定區等模組，行為不變（亂按腳本前後逐步相同）；設定區移到頁面下方；見票 01 | 新 session 下 `/implement .scratch/m3/issues/02-pack-list-cleanup.md` |
| 2026-11-11 | M3 06 | 青簡神經重排與 CyIME 模型評估寫成 `docs/research/neural-rerank.md`，摘進 brief §12.4；見票 06 | 新 session 下 `/implement .scratch/m3/issues/01-web-modules.md` |
| 2026-11-11 | M3 02 | 資料包清單收成 `build_packs` 一處、`Pack` trait、xtask 比對個人集路徑；六份 sha256 與 bench 不變；見票 02 | 新 session 下 `/implement .scratch/m3/issues/01-web-modules.md` |
| 2026-11-11 | M3 14 | 常用語：來源 `phrase` 的使用者詞，觸發詞或候選列清單上屏，設定區增刪改；匯出升 v2（詞欄跳脫）、收 v1；bench 不變；見票 14 | 新 session 下 `/implement .scratch/m3/issues/03-ranking-rule-guards.md` |

## 偏差與決定

- 2026-11-10 M3（M2 票 12）：spec〈驗收〉的「`bench --check` 數字與 M1 結束時相同」改成和 M2 開工（`17727d9`）比：M1 封存後的補票改過詞庫，baseline M2 沒動；見票 12。
- 2026-11-11 M3 拆票：繁體模式的簡體字照 macOS 排在繁體後、不過濾，𨳊 照字頻（使用者決定，改 brief §1.3）；匯出升 v2、收 v1；更新來源 GitHub Releases；見 spec。
- 2026-11-11 M3 票 04：前文以標點結尾時接句首、不跨標點接上一句（跨標點較差），bigram 加句首一列，改 plan §2.3、§4.3；選開頭一段後接選定段，推翻 M1 票 21 的現狀；見票 04。
- 2026-11-11 M3 票 04：「有前文」驗收看四語料合計（yue-wiki 答錯多 2、zh-wiki 持平），個人集修正 +13 接受並更新本機 baseline（使用者決定）；見票 04。
- 2026-11-11 M3 票 01：開關收進頁面下方的設定區（`settings.ts` 一張表產生），重排選單改完也還焦點；plan §3.1 補一行；見票 01。
- 2026-11-11 M3 票 14：匯出 v2 由票 14 先升（詞欄跳脫給常用語的換行），票 12 只加 bigram 列；常用語在簡體模式照樣轉簡體，同 M1 票 23；plan §2.4；見票 14。
