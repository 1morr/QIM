# 排序的參數：詞內用詞頻、個人分照 libime 內插、β γ beam 照 dev 切分

> 排序參數的取捨與調參。公式本身寫在 `crates/qim-core/src/engine.rs`（`word_edge`、`edge_score_parts`）與 `crates/qim-core/src/user.rs`（`personal_bonus`），`crates/qim-core/tests/viterbi.rs` 照同一個式子窮舉對照；這裡記的是為什麼選這些、沒選別的；每個參數試過的另一個值寫在該條裡，量測紀錄在 `.scratch/m1/issues/` 的 11–13 與 `.scratch/m3/issues/` 的 07、12 這幾張票的 Comments。

同一把尺（ADR 0001）的分數由四部分組成：段間的字級 bigram、段內的詞頻、β（詞庫的詞加分）與 γ（每段扣分），再加個人分。各部分的取捨：

- **詞內用詞頻**：`P(詞 | 前一字) ≈ P(第一字 | 前一字) × P_詞頻(詞) ÷ P(第一字)`。詞內也用字級 bigram 時，同碼的詞分不出常用度，個人集答錯多 27%；在詞頻與字級 bigram 之間內插（權重 0.4–0.6）口語略好、維基答錯少約 3–5%，但個人集答錯多 5–16%，不採用。
- **β、γ 由 dev 切分調**（`cargo xtask bench --dev`），寫進方案檔；速成是 β = 1、γ = 1。dev 偏好更小的 β，但個人集（多是兩字詞）答錯變多；個人集只當守門、不拿來調。
- **beam 20**（libime 預設，`docs/research/ranking-techniques.md` §4）：每個起點每個碼只取字頻最高的 beam 個字進 lattice，另加使用者學過的字；全收或 beam 40 在 dev 只差 1 題，不放寬。代價：前文再支持第 21 名以後沒學過的同碼字也進不來。
- **個人分照 libime 內插、只加不減**：`max(0, ln((1−w)·P + w·P_personal) − ln P)`，`w` = 0.05（libime 用 0.2，`docs/research/libime-user-model.md` §1）。w 0.2 在換上 bigram 後讓 zh-wiki 學習開變差，換上口語權重與新 β、γ 後連 common-voice-yue 也變差；在 dev 切分的學習模式掃 0.01–0.2，0.05 對口語與維基都最好，test 上口語兩集、個人集與 yue-wiki 修正較少，zh-wiki 持平。
- **個人機率的分母取常數** `D_max = 1 / (1 − e^(−1/200))`（每次上屏都是它時衰減值的上限），照 libime 除以滿載歷史的總量：學過一次的段 `P_personal ≈ 0.005`，不會因為才學幾次就壓過所有詞。分母改用實際上屏的段數衰減總和時，`bench --learning` 四個公開集合的修正次數都較多（當時是 unigram）。
- **個人 bigram 只往上加**：`P_personal = P_unigram + (1 − P_unigram)·λ·P_bigram`，λ = 0.8。libime 是 `(1 − λ)·P_unigram + λ·P_bigram`，沒有前一個字詞時學過的段只剩兩成的個人分（`a_learned_word_also_scores_in_longer_continuous_input` 會紅）。λ 在 dev 切分的學習模式掃 0–1，0.8 八個集合合計修正最少（`docs/research/libime-user-model.md` §2）。
- **候選列放 3 條多段路徑**（方案檔 `ranking.sentences`）：放多時維基集修正次數降、口語集升。繁體模式的簡體字照樣進 lattice、在多段路徑裡照分數比，不另扣分：它們在語料裡只有零星殘留，字頻極低。

## Consequences

- β、γ、beam、多段路徑數寫在方案檔 `[ranking]`；方案檔、`bench --set` 與載入資料包都經 `Ranking::check` 驗（beam 至少 1，β、γ 是有限的數）。
