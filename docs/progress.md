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
| M2 | `.scratch/m2/issues/` | brief §9、§11、§12.3；plan §1.4、§3、§7.3；ADR 0002、0005；`.scratch/m1/issues/24-m1-close.md` 的 Comments（留給 M2 起各里程碑的發現）；`.scratch/m2/spec.md` | 進行中，已拆票 |

## Session 紀錄

| 日期 | 票 | 結果 | 下一步 |
| --- | --- | --- | --- |
| 2026-11-04 | M2 封存 M1 | 票 24：里程碑 code-review、乾淨 checkout 重驗 spec〈驗收〉全過、黑箱在 QIM 重跑；M1 封存到 `docs/archive/M1.md`，M2 標進行中；細節見票 24 | 新 session 下 `/to-tickets docs/plan.md 拆 M2 的票` |
| 2026-11-05 | M2（補 M1 25） | 單按 Shift 照 Rime：左 Shift 進臨時英文、右 Shift 先上屏，`Options.shift_switch` 三選一、網頁有選單；細節見票 25 | M1 26、27 也做完後，新 session 下 `/to-tickets docs/plan.md 拆 M2 的票` |
| 2026-11-05 | M2（補 M1 27） | 票 27：觸控佈局照 Gboard 速成實測（Shift、符號三頁、配色、數字列開關、長按只出字母）；組字中的分隔鍵另開補票，見票 27 | 補票都做完後，新 session 下 `/to-tickets docs/plan.md 拆 M2 的票` |
| 2026-11-05 | M2（補 M1 26） | 數字後的 `.` `:` 在前一字是半形數字時直接上屏半形、逗號照舊全形（使用者決定）；加開關，細節見票 26 | 新 session 下 `/to-tickets docs/plan.md 拆 M2 的票`（票 27 也做完後） |
| 2026-11-05 | M2（補 M1 25b） | 臨時英文改照目前發佈版 librime（≤ 1.17.0）：空白接進英文、Enter 上屏整段，再按留碼的 Shift 回中文組字；細節見票 25b | M1 17c、26、27 也做完後，新 session 下 `/to-tickets docs/plan.md 拆 M2 的票` |
| 2026-11-05 | M2（補 M1 17c） | 詞庫照語料走 `t2hk` 加常見寫法（`sqvm` 群組）；並存寫法 11 組雙向、「着→著」單向，「顯着」併進「顯著」；細節見票 17c | M1 28 也做完後，新 session 下 `/to-tickets docs/plan.md 拆 M2 的票` |
| 2026-11-05 | M2（補 M1 28） | 手動分隔鍵：組字中 `'`（觸控組字中 ⇧ 變「'詞」）插入分隔記號，連打的段不跨過它，學習的碼不含它；細節見票 28 | 新 session 下 `/to-tickets docs/plan.md 拆 M2 的票` |
| 2026-11-05 | M2（補 M1 28b） | 詞庫的詞可跨過落在字界上的分隔記號（照 Rime 速成），單字照舊不跨；語料分隔後變錯 81 → 0；細節見票 28b | 補票 17d、26b、27b 都做完後，新 session 下 `/to-tickets docs/plan.md 拆 M2 的票` |
| 2026-11-05 | M2（補 M1 17d） | 詞庫補漏：鈎鉤、濕溼等 6 組補成雙向；臺→台、菸→煙單向，語料也照原文；同頻重複只算一次、合着不併合著；建置接線補測試；細節見票 17d | 新 session 下 `/to-tickets docs/plan.md 拆 M2 的票`（補票 26b、27b 也做完後） |
| 2026-11-05 | M2（補 M1 26b） | 單按修飾鍵、觸控切頁與 ⇧ 不結束數字（`Key::Modifier`）；單按 Shift 判定在放開別的鍵時解除；librime 用詞改成發佈版／master；細節見票 26b | 新 session 下 `/implement .scratch/m1/issues/27b-touch-shift-and-labels.md` |
| 2026-11-05 | M2（補 M1 27b） | 觸控標點照鍵上的字（使用者決定）；開一次的 Shift 照 Gboard 實測被任何鍵解除；讀屏、數字列狀態、角標與大寫字級照 Gboard；細節見票 27b | 新 session 下 `/to-tickets docs/plan.md 拆 M2 的票` |
| 2026-11-06 | M2（公開 repo） | 補 Apache-2.0 `LICENSE` 與 `README.md`，repo 公開到 https://github.com/1morr/QIM | 新 session 下 `/to-tickets docs/plan.md 拆 M2 的票` |
| 2026-11-07 | M2（M1 review） | 重跑 check 與 bench 全過；決定 M2 先清理 FFI 前的介面與 `set_options` bug、另開 CI 小票、只用 iPhone 18 Pro Max 量；見 plan §7.3 | 新 session 下 `/to-tickets docs/plan.md 拆 M2 的票` |
| 2026-11-07 | M2 拆票 | plan §7.3 細化成 `.scratch/m2/spec.md`，拆 12 張票（票序見 spec）；01、04、05、06 可平行 | 新 session 下 `/implement .scratch/m2/issues/01-set-options-stale-candidates.md` |
| 2026-11-10 | M2 01 | `set_options` 後 `select`、`focus_segment`、`text` 先依新選項重列；重列前展開面板空、不重排；bench 不變；細節見票 01 | 新 session 下 `/implement .scratch/m2/issues/02-unify-naming.md` |
| 2026-11-10 | M2 05 | GitHub Actions 跑 `check --fast` 與 `cargo test`；`bench --check` 不進 CI 與 `check`（plan §6）；推送由協調者合併後做，見票 05 | 新 session 下 `/implement .scratch/m2/issues/02-unify-naming.md` |
| 2026-11-10 | M2 06 | 免費佈建能跑鍵盤擴充，備忘錄打出「日」；App Group 擴充可讀不可寫；空擴充約 10 MB、上限推得 177 MB；見 ios-feasibility | 新 session 下 `/implement .scratch/m2/issues/10-foundation-models-and-context.md` |
| 2026-11-10 | M2 10 | 擴充內 Foundation Models 可用：連續約 190 ms、停頓後首個偶爾 6–15 s；上下文按句切，備忘錄至多 300 字、Safari 100；見 ios-feasibility | 新 session 下 `/implement .scratch/m2/issues/11-small-neural-model-cost.md` |

## 偏差與決定

- 2026-11-05 M2（補 M1 25）：組字中單按 Shift 改照 Rime（左留碼接英文、右先上屏），設定可改；使用者試用後決定，推翻 M1 票 23 的左右都先上屏（brief §4.3）。
- 2026-11-05 M2（補 M1 27）：長按字母鍵不再出數字，改由設定的數字列（使用者決定，推翻 brief §4.4「第一排另含數字」）；佈局格式加 `"shift"` 動作，只加不改、不升版本（plan §2.6）。
- 2026-11-05 M2（補 M1 26，使用者決定）：數字後半形從 M4 提前到 M1，預設開、可關；前一字是半形數字時 `.` `:` 直接上屏半形，不學 Rime 的待定與再按改全形，要全形就關開關；逗號照舊全形，同 Rime 2025-11 拿掉逗號後的預設（brief §4.7）。
- 2026-11-05 M2（補 M1 25b）：臨時英文的空白接進英文、再按留碼的 Shift 回中文組字，照使用者用的目前發佈版 librime；使用者決定，推翻 M1 票 25 照未發佈 master 的做法；偏離發佈版的四處使用者接受（brief §4.3）。
- 2026-11-05 M2（補 M1 17c，使用者決定）：不同碼、兩種寫法都常見的 11 組兩種都收、共用字頻詞頻 bigram，「着→著」只單向；推翻 M1 票 17b「不同碼不換」，語料也把另一種寫法換成代表寫法（brief §4.1、plan §4.3）。
- 2026-11-05 M2（補 M1 17c）：口語詞表拿掉「車仔麵」：essay 的「車仔麪」換成常見寫法後已收，詞表現在 196 詞（plan §4.2）。
- 2026-11-05 M2（補 M1 17c，使用者決定）：並存寫法的詞兩種寫法共用詞頻，接受「到底幾時」打成「到底綫時」這類另一種寫法的詞搶同碼的已知限制，不照字頻打折（plan §4.2）。
- 2026-11-05 M2（補 M1 28，使用者決定）：組字中 `'` 與觸控組字中的 ⇧（「'詞」）是手動分隔，推翻票 27 的組字中照樣是 Shift；段不跨分隔記號，比 Rime（詞可跨）嚴（brief §4.3、§4.4；票 28b 推翻）。
- 2026-11-05 M2（補 M1 28）：學習與自動造詞的碼去掉分隔記號，`mi'r` 選兩次的字詞不分隔打 `mir` 也成首選，同 Rime 使用者詞典（實作決定，plan §2.4）。
- 2026-11-05 M2（補 M1 28）：佈局格式加 `composing.a11yLabel`（組字中換了標籤時讀屏念它），只加不改、不升版本（plan §2.6）。
- 2026-11-05 M2（補 M1 28）：票的「在目前位置插入」實作成接在輸入碼最後：QIM 沒有輸入碼內的游標，有目前分段時也同打字母回到整串（brief §4.3）。
- 2026-11-05 M2（補 M1 28b，使用者決定）：詞庫的詞與使用者詞可跨過分隔記號，條件是它落在詞內兩字的碼之間，照 Rime 速成；推翻票 28 的段不跨（brief §4.3、§12.2，plan §2.3）。
- 2026-11-05 M2（補 M1 17d，使用者決定）：照審查建議補漏；「臺」「菸」改成單向並存寫法且語料照原文（協調者更正票原文的只改詞庫），「囪」雙向，簡體「随著」記成已知限制（brief §4.1、plan §4.2、§4.3）。
- 2026-11-05 M2（補 M1 26b）：核心加 `Key::Modifier`，單按修飾鍵、觸控切頁與 ⇧ 組字中吃下、沒有組字時交還宿主但不結束數字，同 Rime 不記沒有字元的鍵（plan §2.1、brief §4.7）。
- 2026-11-05 M2（補 M1 26b）：「舊版／新版 librime」改稱「目前發佈版（≤ 1.17.0）／未發佈的 master」；1.18 發佈後使用者的小狼毫會變成 master 的行為，到時再決定跟不跟（brief §4.3、§12.2）。
- 2026-11-05 M2（補 M1 27b，使用者決定）：觸控標點照鍵上的字輸出（`text`），數字後半形只留給實體鍵盤；推翻票 27 的觸控標點送 `Key::Punct`（brief §4.4、§4.7，plan §2.1、§2.6）。
- 2026-11-05 M2（補 M1 27b，使用者決定）：符號頁 `：；！？（）［］` 照 Gboard 實測標全形、打全形，推翻票面的「ASCII 標點半形」；英文直通時觸控打不出半形是已知限制（brief §4.4）。
- 2026-11-05 M2（補 M1 27b）：臨時英文中觸控的 `text`（標點、數字、表情、常用語）接在英文後，推翻票 25b 的先上屏英文（實作決定，brief §4.3、plan §2.1）。
- 2026-11-05 M2（補 M1 27b）：開一次的觸控 Shift 照 Gboard 實測被任何別的鍵解除，推翻票 27 的只有字母鍵解除（brief §4.4、plan §2.6、GLOSSARY）。
- 2026-11-07 M2（M1 review，使用者決定）：M2 加一張 FFI 前的清理票與一張 CI 小票（plan §7.3）；記憶體上限只在 iPhone 18 Pro Max 量並註明。
- 2026-11-10 M2 票 01：換選項還沒重列時 `all_candidates` 回空、`rerank_request` 與 `apply_rerank` 回 `None`，票外決定（同類錯）；`set_options` 帶 `Engine` 直接重列留給票 07（plan §2.1）。
