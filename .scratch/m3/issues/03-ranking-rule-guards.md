# 03: 排位規則補測試再修

Status: done
Blocked by: None (can start immediately)
讀: plan §2.3、§2.4；spec〈共同規則〉；`.scratch/m1/issues/24-m1-close.md` Comments 的「M3」各條（排位規則、`beam=0` 兩條）

## What to build

個人 bigram 與聯想都要動排位，先把 M1 留下沒測到的規則守住。

- 規則 1 把觸發詞釘在首選時，規則 2「選一次不成首選」可能會多移一次（M1 讀程式推論、未實測）：先寫出會紅的測試，紅了再修；寫不出就在 Comments 說明並留測試守門。
- 規則 3、4 與「分段選出、詞庫沒有的整串」補測試（M1 票 13、14 沒有）。
- `--set beam=0` 或 `NaN` 繞過方案檔的驗證：驗證搬到 `qim_core::Ranking`，方案檔與 `bench --set` 共用同一處。

## 驗收

- [x] 規則 1–4 各有測試；規則 1 與 2 的交互有測試，修了的話先紅後綠（Comments 記紅的輸出）。
- [x] `bench --dev --set beam=0`、`--set beam=NaN` 被拒，錯誤訊息指出是哪個值。
- [x] `cargo xtask bench --check` 與 `--learning` 前後對照寫在 Comments；`cargo xtask check` 綠燈。

## Comments

- 2026-11-11 規則 1 × 2 的「多移一次」實測成立：觸發詞釘在首選後，規則 2 的「選一次不成首選」看的 `free.first()` 其實是第 2 位，把不學習時的首選多移到第 2。紅的輸出（`learning.rs` 的 `a_trigger_on_top_leaves_the_once_committed_candidate_where_it_was`）：`left: ["某乙", "乙丙", "丙丙"]`、`right: ["某乙", "丙丙", "乙丙"]`。修法：首選已排定時這一支不動（`lattice.rs` 的 `once_chosen_keeps_the_unlearned_top`），plan §2.4 補一句。「選兩次」那一支照 plan 的通則退到第 2，另有 `a_trigger_on_top_puts_the_twice_chosen_candidate_second` 守。
- 2026-11-11 新測試：規則 3 取最近整串上屏的、已是首選的釘在原位（`import` 造 10,000 次以前的計數，避開規則 2）；規則 4 排在規則 1、3 之後（`english.rs`）；分段選出、學之前不在候選列的 `乙丁` 先第 2 再成首選（M1 票 13 的 `abcd → XY`）。變異驗證都紅：拿掉規則 3 的釘在原位、改取最舊、拿掉規則 2 的修正、拿掉規則 2 的釘位。
- 2026-11-11 驗證搬進 `qim_core::Ranking::check`（`RankingError` 指名欄位與值）；方案檔不再有 `RankingTable`，直接讀成 `Ranking`（加 `deny_unknown_fields`）。CLI：`--set beam=0` →「--set 之後的打分參數不合用：ranking.beam 至少是 1，給的是 0」；`beam=NaN` →「--set beam 的值不對：NaN」（`u16` 解析就擋下）；`word_bonus=NaN` 指名 word_bonus 與 NaN；都 exit 1。
- 2026-11-11 票外的取捨：`Engine::new` 也呼叫 `check`，不合時回 `PackError::Malformed`：票 20 起殼可以從檔案匯入資料包，不一定經 `qim-databuild`。`--set` 的解析錯誤加 `--set ` 前綴；只有給了 `--set` 才用「--set 之後…」的訊息，沒給時由 `Engine::new` 報資料包。
- 2026-11-11 bench 前後對照（前 = 同一份資料包拿掉規則 2 的修正）：`--check` 的各集合表、依長度分組與翻盤清單逐字相同，`bench 通過`；`--learning` 輸出逐字相同。基準不加觸發詞，規則 1 不會出現，數字不變是預期的。主要數字（前 = 後，答錯／修正）：yue-wiki 1131／4340、zh-wiki 1126／4380、common-voice-yue 1032／3946、hkcancor 663／2573、personal 1081／1446；學習開修正 yue-wiki 4267、zh-wiki 4396、common-voice-yue 3855、hkcancor 2098、personal 1386。
- 2026-11-11 驗證：`cargo xtask check` 全綠（`e2e` 155 passed、`check 通過`），`cargo test --workspace` 707 passed。新 worktree 要先 `npm ci`（`tsc` 步驟會提示）。
- 2026-11-11 code-review（`936a7cf` 起）：已處理：規則 1 × 2 的「選兩次」補測試；`--set` 訊息只在有覆寫時用；測試抽 `candidates_of`、`learn_nian_zhu_shui` helper；`RankingTable` 去掉。沒處理：`RankingError::NotFinite` 的欄位用 `&'static str`（只有兩個值，訊息直接用方案檔欄位名）；`pinned.contains_key(&0)` 不另取名（規則 2 之前只有規則 1 釘首選，註解已寫）。
- 2026-11-11 合併 main（票 16、19）後沒有衝突；session 紀錄照合併順序把 03 移到最後。`data --verify` 通過（37 項）、`cargo xtask check` 全綠（e2e 164 passed）。`bench --check` 公開集合都等於 baseline（答錯／修正：yue-wiki 1131／4352、zh-wiki 1126／4385、common-voice-yue 1032／3966、hkcancor 663／2596），翻盤清單都是「沒有翻盤」；只有本機個人集修正 1468、高於 baseline 1466。合併後的樹拿掉本票規則 2 的修正重跑，輸出逐字相同，所以 +2 來自 main 的票 16 與 19 合在一起（票 16 單獨是 1446 → 1466），不是本票；本機 baseline 沒有更新，等使用者決定。`--learning` 學習開：yue-wiki 4277、zh-wiki 4400、common-voice-yue 3871、hkcancor 2105、personal 1407。
- 2026-11-11 個人集 1466→1468 來自票 19 合併，由協調者同意更新：worktree 的 `bench/personal` 改用軟連結接主 checkout（`docs/development.md`〈個人集〉，先前是複製），`cargo xtask bench --update-baseline` 只改到主 checkout 的本機 baseline（答錯 1081 與行號不變），`bench/baseline.json` 沒有差異。
