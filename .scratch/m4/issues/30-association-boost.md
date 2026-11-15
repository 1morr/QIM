# 30: 聯想補強

Status: done
Blocked by: 18, 18b, 18c
讀: plan §2.1（聯想）、§2.4；`.scratch/m3/issues/13-association.md`；`docs/research/blackbox-baseline.md` 的 T8；spec〈共同規則〉

## What to build

M3 票 13 留下、黑箱 T8 的缺功能，先評估再做：

- emoji 聯想、空輸入的句首建議、數字與標點後的量詞聯想。
- 每項查成熟做法、資料來源與誤觸風險，決定做或不做。
- 做的照聯想現有的開關（brief §7，預設開）。

## 驗收

- [x] 每項做或不做與理由寫在 Comments 與 plan §2.1。
- [x] 做的各有核心測試與 Playwright 讀回、截圖；`bench --check` 不變；`cargo xtask check` 綠燈。

## Comments

- 評估（plan §2.1 聯想那條的最後兩點）：

  | 項目 | 成熟做法 | 資料 | 誤觸風險 | 結論 |
  | --- | --- | --- | --- | --- |
  | emoji 聯想 | macOS 速成有（黑箱 T8：日 → 本、期、🌞；日本 → 人、隊、🇯🇵、🗾），iOS、Gboard 沒看到 | 現成的 emoji 資料包（CLDR，M3 票 16），查剛上屏的字詞 | 聯想沒有預選、空白不選，只有按數字或點選才選到；代價是佔掉這頁最多 3 格，第 3 位起的字詞往後移（同 emoji 候選佔第一頁的代價） | **做** |
  | 空輸入的句首建議 | 只有 iOS（黑箱 T8、T18：我、你、咁、佢…，打「，」後重設成句首建議）；macOS 沒有，Gboard 接到句號就換回工具列 | 字級 bigram 的句首（`CLAUSE_START`）。dev 切分的句子第一字落在 train 句首最常見 9 字的比例：hkcancor 32.6%、common-voice-yue 26.6%、yue-wiki 26.3%、zh-wiki 14.0% | 桌面：每次聚焦欄位、打完標點都跳出候選窗，數字鍵要嘛選候選（打不出數字）、要嘛照打（只能用滑鼠選）。觸控：M5 已定沒有組字時候選列換成工具列，兩者搶同一個位置 | **不做**，M5 重做候選列時評估（roadmap 已加一條） |
  | 數字、標點後的量詞聯想 | 只有 iOS（黑箱 T18：日＋1 之後給 號、月、點、日、年、個，留在數字頁） | **沒有**：語料前處理把阿拉伯數字當字集外字元，含數字的句子全丟掉，四個語料的 train 裡「數字 → 字」是 0 次；要另整理量詞表或改前處理重訓 bigram | 數字鍵選聯想（plan〈介面〉照 macOS）時打第 2 個數字就選走了量詞；改成數字照打，桌面每打一個數字都跳候選窗。觸控同上，搶工具列的位置 | **不做**，同上 |

  量句首比例的腳本只跑一次、不進 repo（取 train 前 200 萬句的句首字頻、dev 前 20 萬句算比例）。

- emoji 聯想的做法（plan §2.1）：
  - 剛上屏的最後幾段（分段的邊界）接起來、有 emoji 的最長那個，最多 `EMOJI_LIMIT`（3）個，插在前兩個字詞聯想後面（`EMOJI_AFTER`，照 macOS 排第 3 位），佔掉排最後的字詞；字詞不到兩個時排最後，沒有字詞時只有 emoji。
  - 選了原樣上屏、不學（不記個人 bigram）、不接下一輪，同組字中選 emoji。
  - 要聯想與「emoji 候選」兩個開關都開；debug 面板標 emoji。
  - 真資料：上屏「笑」→ 話、容、😆、🤣、😂、了、着、著、笑；「腦」→ 子、袋、🧠…；「多謝」→ 晒、謝謝、🙏、🫂…。
- 測試：
  - 核心 `crates/qim-core/tests/association.rs` 加 7 個（位置、最長的那段、只有 emoji、選了不學不接、兩個開關、debug 面板的種類），實作前先跑過都紅。
  - Playwright `web/e2e/association.spec.ts` 加 3 個：桌面上屏「笑」後第 3 位是 😆、按 3 上屏、面板標 emoji；關「emoji 候選」沒有；觸控點「笑」再點 😆。截圖 `.scratch/m4/evidence/30-desktop-emoji-after-laugh.png`、`30-touch-emoji-after-laugh.png`。
  - 實際操作亂按一輪（Playwright 開開發伺服器手動按）：選 emoji 後翻頁鍵、Esc、退格、聯想中切 emoji 與簡體、快速連按 `an 123`（門 → 檻 → 車 → 🚃，數字照規則選聯想，選到 emoji 就停）、Shift、Enter，都照預期，沒找到問題。
- `cargo xtask bench --check`：每個集合的答錯與修正都和 baseline 相同、沒有翻盤、固定題 84/84。第一次跑時機器上有別的 session（load average 約 15），原生「空白、選開頭一段」p95 5,867 µs 超過 5 ms；照〈每鍵延遲〉在負載較低時（約 4–6）單獨重跑，`bench 通過`：空白全部 p95 231 µs、選開頭一段 4,652 µs，學過全部 417 µs、選開頭一段 6,703 µs，含萬用鍵的也都在上限內。聯想只在上屏後算，bench 計時的鍵碰不到它。沒動學習與重排的輸入，沒跑 `--learning`、`model-ladder`。
- code-review（`43800aa` 起，Standards／Spec 兩軸）：
  - 已處理：測試名避開名詞表〈emoji 候選〉列為 _Avoid_ 的「emoji 聯想」；fixture 註解寫錯；`emoji` 參數被同名變數遮蔽；組 emoji 候選的寫法兩處重複，收成 `CandidateSpan::emoji`；最後一段超過 8 字時連它自己的 emoji 都不查，改成最後一段一定查；plan 補 emoji 的誤觸代價。
  - 未處理：名詞表〈emoji 候選〉的定義只講組字中的 emoji、_Avoid_ 仍列「emoji 聯想」，要不要補一句聯想裡的也算，名詞表改動要使用者確認，留給合併的人問；brief §7 的「emoji 候選」開關現在也管聯想的 emoji，brief 沒改（同上，要使用者確認）；簡體模式下前兩個字詞轉成同一個簡體字時去重，emoji 會從第 3 位變第 2 位，影響小、不處理；「emoji 候選關掉就沒有」核心與 e2e 各測一次，e2e 守的是設定區接到開關，留著。

Session 紀錄：三項聯想補強評估完，做聯想的 emoji（照 macOS 排第 3 位、選了不學不接），句首建議與量詞聯想因桌面數字鍵衝突、和 M5 的工具列搶位置（量詞也沒有資料）不做，排進 M5 評估。
決定：做聯想的 emoji，句首建議與量詞聯想不在 M4 做、排進 roadmap M5 評估（Agent 定）
決定：聯想的 emoji 插在前兩個字詞後面，照 macOS 速成的第 3 位，不放在整頁最後（Agent 定）
決定：查剛上屏的最後幾段接起來有 emoji 的最長那個，只給那一個字詞的 emoji，最多 3 個同 emoji 候選（Agent 定）
決定：聯想的 emoji 要聯想與「emoji 候選」兩個開關都開，不另加開關（Agent 定）
決定：選了聯想的 emoji 和組字中選 emoji 一樣原樣上屏、不學、不接下一輪（Agent 定）
