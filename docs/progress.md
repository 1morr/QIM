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
| 2026-11-11 | M3 06 | 青簡神經重排與 CyIME 模型評估寫成 `docs/research/neural-rerank.md`，摘進 brief §12.4；見票 06 | 新 session 下 `/implement .scratch/m3/issues/01-web-modules.md` |
| 2026-11-11 | M3 02 | 資料包清單收成 `build_packs` 一處、`Pack` trait、xtask 比對個人集路徑；六份 sha256 與 bench 不變；見票 02 | 新 session 下 `/implement .scratch/m3/issues/01-web-modules.md` |
| 2026-11-11 | M3 01 | `main.ts` 拆成候選列、面板、設定區等模組，行為不變（亂按腳本前後逐步相同）；設定區移到頁面下方；見票 01 | 新 session 下 `/implement .scratch/m3/issues/03-ranking-rule-guards.md` |
| 2026-11-11 | M3 14 | 常用語：來源 `phrase` 的使用者詞，觸發詞或候選列清單上屏，設定區增刪改；匯出升 v2（詞欄跳脫）、收 v1；bench 不變；見票 14 | 新 session 下 `/implement .scratch/m3/issues/03-ranking-rule-guards.md` |
| 2026-11-11 | M3 19 | 繁體模式的簡體字（278 字，建置端判定，格式 v6）排在同碼繁體字之後，開關「繁體字優先」預設開；選兩次照常成首選；見票 19 | 新 session 下 `/implement .scratch/m3/issues/03-ranking-rule-guards.md` |
| 2026-11-11 | M3 04 | `set_context`、選開頭一段後接選定段、bigram 加句首、「有前文」集合（給前文合計答錯與修正都較少，dev 重比結論相同）；見票 04 | 新 session 下 `/implement .scratch/m3/issues/03-ranking-rule-guards.md` |
| 2026-11-11 | M3 16 | emoji 候選：CLDR 粵語、繁中標註建 `cldr-emoji-yue-hant`，整串字詞後接最多 3 個、預設開；首選不變、修正多 0.1–1.4% 更新 baseline；見票 16 | 新 session 下 `/implement .scratch/m3/issues/03-ranking-rule-guards.md` |
| 2026-11-11 | M3 03 | 規則 1 × 2 多移一次先紅再修；規則 3、4 與分段選出的整串補測試；`beam` 驗證搬進 `Ranking::check`；bench 前後相同；見票 03 | 新 session 下 `/implement .scratch/m3/issues/12-personal-bigram.md` |
| 2026-11-11 | M3 05 | 網頁給游標前文（含每鍵前重給）、數字後半形看前文；核心記游標前的文字，殼沒給時用自己上屏的字（照 librime）；bench 不變；見票 05 | 新 session 下 `/implement .scratch/m3/issues/12-personal-bigram.md` |
| 2026-11-11 | M3 17 | 詞表匯入：Rime `.dict.yaml`、Gboard `dictionary.txt`、一行一詞，碼由字推出、加進使用者詞，壞列報行號；Gboard 快捷鍵勾選才設觸發詞；見票 17 | 新 session 下 `/implement .scratch/m3/issues/12-personal-bigram.md` |
| 2026-11-11 | M3 18 | 粵語詞庫：rime-cantonese、粵典查不清不用（粵典待書面確認）；HKCanCor train 的詞補 383 個，口語集變好、個人集多錯 1 題更新 baseline；見票 18 | 新 session 下 `/implement .scratch/m3/issues/12-personal-bigram.md` |
| 2026-11-11 | M3 20 | 資料包 manifest（plan §4.6），核心載入時核對；網頁設定區列出、切換、刪除、從檔案匯入（IndexedDB）；bench 不變；見票 20 | 新 session 下 `/implement .scratch/m3/issues/12-personal-bigram.md` |
| 2026-11-11 | M3 12 | 個人 bigram（λ 0.8，dev 掃出；句中學到的不搶單打首選）、學習總開關（網頁加開關）、匯出 v2 加 bigram 列；合併 main 後 `--learning` 公開集合合計修正 −138；見票 12 | 新 session 下 `/implement .scratch/m3/issues/13-association.md` |
| 2026-11-11 | M3 21 | 線上更新：核心 `PackIndex`、`pack-release`（擋沒加版本）、網頁背景檢查與開關，本機伺服器驗；內建與下載版取較高的；未建 GitHub release；見票 21 | 新 session 下 `/implement .scratch/m3/issues/13-association.md` |
| 2026-11-11 | M3 13 | 聯想：上屏後給下一個字詞（補完詞庫的詞、bigram、個人 bigram，同一把尺），按鍵照 macOS、開關預設開；bench 前後相同；黑箱 T6、T8 重跑；見票 13 | 新 session 下 `/implement .scratch/m3/issues/07-ladder-baseline.md` |
| 2026-11-11 | M3 15 | 記住 email：經 QIM 打過一次就記，中文組字打開頭 3 個字母起排第 2 位起（規則 5，開關預設開），設定區可刪，匯出 v2 加列；bench 不變；見票 15 | 新 session 下 `/implement .scratch/m3/issues/07-ladder-baseline.md` |

## 偏差與決定

- 2026-11-11 M3 票 13：數字鍵照 macOS 選聯想（不學 Microsoft 的 Shift＋數字），上屏後緊接著打數字要先按 Esc；聯想只看剛上屏的分段、從段的邊界補詞，無痕不出；改 plan §2.1、§2.3、§2.4；見票 13。
- 2026-11-11 M3 票 17：Gboard 快捷鍵預設不設成觸發詞（原本是方案的鍵就設），確認框勾選才設、標出會搶首選的列：在別的鍵盤設的快捷鍵常撞速成的碼；改 plan §2.4；見票 17。
- 2026-11-10 M3（M2 票 12）：spec〈驗收〉的「`bench --check` 數字與 M1 結束時相同」改成和 M2 開工（`17727d9`）比：M1 封存後的補票改過詞庫，baseline M2 沒動；見票 12。
- 2026-11-11 M3 拆票：繁體模式的簡體字照 macOS 排在繁體後、不過濾，𨳊 照字頻（使用者決定，改 brief §1.3）；匯出升 v2、收 v1；更新來源 GitHub Releases；見 spec。
- 2026-11-11 M3 票 19：bench 修正次數 yue-wiki +6、common-voice-yue +2（答案含簡體字的題），使用者同意 `--update-baseline`（合併票 04 後 yue-wiki 前文兩集同因各 +6）；「恒」查證後仍算簡體字、「够」是香港標準字形，移出；叙携等與繁體同序號的算異體；見票 19。
- 2026-11-11 M3 票 16：驗收「`bench --check` 不變差」改成首選不變：emoji 佔第一頁把正解擠到下一頁，修正多 0.1–0.9%、個人集 1.4%；使用者選每列最多 3 個、更新 baseline；見票 16。
- 2026-11-11 M3 票 04：前文以標點結尾時接句首、不跨標點接上一句（跨標點較差），bigram 加句首一列，改 plan §2.3、§4.3；選開頭一段後接選定段，推翻 M1 票 21 的現狀；見票 04。
- 2026-11-11 M3 票 04：「有前文」驗收看四語料合計（yue-wiki 答錯多 2、zh-wiki 持平），個人集修正 +13 接受並更新本機 baseline（使用者決定）；見票 04。
- 2026-11-11 M3 票 01：開關收進頁面下方的設定區（`settings.ts` 一張表產生），重排選單改完也還焦點；plan §3.1 補一行；見票 01。
- 2026-11-11 M3 票 14：匯出 v2 由票 14 先升（詞欄跳脫給常用語的換行），票 12 只加 bigram 列；常用語在簡體模式照樣轉簡體，同 M1 票 23；plan §2.4；見票 14。
- 2026-11-11 M3 票 20：方案 `[packs]` 指名的改成預設，`Engine::new`、`RadicalHints::for_scheme` 只核對要不要、不核對 id（推翻 M1），建出的是否指名的那份改由 `--verify` 擋；plan §4.6；見票 20。
- 2026-11-11 M3 票 20：沒照研究把 manifest 塞進 `.pack` 檔頭（改格式、sha256 全換），匯入改成一起選 manifest 與 `.pack`；libcangjie 授權改寫 `LicenseRef-Public-Domain`；見票 20。
- 2026-11-11 M3 票 05：殼沒給前文時用這個輸入框上屏過的字（同 librime），退格、方向鍵、`clear_context` 讓它作廢；無痕時不記也不收（brief §8）；plan §2.1；見票 05。
- 2026-11-11 M3 票 03：觸發詞已在首選時，規則 2「選一次不成首選」不動（原本把不學習時的首選多移到第 2）；`Engine::new` 也驗打分參數；改 plan §2.3、§2.4；見票 03。
- 2026-11-11 M3 票 12：個人 bigram 不照 libime 的凸組合，改 `Pu + (1−Pu)·λ·Pb`（bigram 只往上加）；`--learning` 驗收改看合計，合併 main 後只有 yue-wiki·前文 +6，其餘減少（使用者決定）；學習關掉時手動加詞、常用語照收（使用者確認）；改 plan §2.3、§2.4；見票 12。
- 2026-11-11 M3 票 18：完整粵語詞庫只補 HKCanCor 的詞；粵典 public domain 詞表與詞典本體授權對不上算查不清，留在之後待書面確認（使用者選），改 brief §1.5、§12.1、plan §4.2；見票 18。
- 2026-11-11 M3 票 15：英文直通的鍵核心看得到（殼先交給 `handle`），記住 email 從送出去的字認；`clear_context` 改收 `UserModel`，記一次 email 讓共用 tick 加一；plan §2.1、§2.4；見票 15。
- 2026-11-11 M3 票 21：沒建 GitHub release（使用者不在、只授權推 main），只驗本機流程，建前先問；來源用 tag `packs` 不用 `latest`（App 的 release 會搶）；plan §4.7；見票 21。
