# 25: 階梯補測：依句長分組與口語長前文

Status: done
Blocked by: 11b
讀: `docs/research/model-ladder.md`〈前文長度〉〈停鍵層〉〈雲端層〉〈停鍵層在新基線重量〉；`scripts/experiments/model-ladder/README.md`〈限制〉（長前文那條）；`scripts/experiments/neural-rerank/README.md`；`scripts/experiments/llm-rerank/README.md`；spec〈採用門檻〉〈共同規則〉；plan §5.2（有前文的題）；brief §8

## What to build

1.4M 未採用（票 11d：新基線上個人集退步超過門檻），長前文組照樣量 1.4M 與基線當實驗數字；1.4M 用票 11d 重訓的 `build/neural-rerank/lstm-c0.5/`（λ 2，`docs/research/model-ladder.md`〈停鍵層在新基線重量〉）；`build/` 不進 git，只在票 11d 的 worktree，檔不在就照 README 重訓。

票 07–11 的結論有兩個缺口，使用者要求補測（2026-11-11）：

1. **沒有依句長分組**：只有整體數字，不知道長句的進步是否比短句大。
2. **口語幾乎沒有長前文**：test 的長前文 p50 是 common-voice-yue 8 字、hkcancor 16 字，到 50 字的 0%、17%，到 200 字的 0%、1%。「前文 200 字比 50 字沒差」（brief §8 改成 50 字）幾乎全來自維基，口語的長上下文沒量到。

要做：

- **依句長分組**（不重訓、不花錢）：用現有的逐題結果，把基線（現在的詞級 trigram）、1.4M、23M、flash、Foundation Models 的口語與維基結果依句長分組（例如 1–4、5–8、9–14、15 字以上），每組列首選正確率、前 2 名命中率、題數。缺逐題結果的模型照 README 的重跑步驟補跑 dev 或用既有檔，不重跑 test 的付費請求。
- **口語長前文題**：HKCanCor 是對話錄音，同一段對話的前幾輪接起來當前文，做一組「口語長前文」的題（dev 與 test 都做，切分照語料管線，答案句不進訓練）。先報前文長度的分佈（p50、到 50 字與到 200 字的比例）。
- 在這組題上量 1.4M（票 11d 重訓的那個，沒採用、當實驗數字）送 0、50、200 字前文的前 2 名，dev 調參、test 只跑一次。
- 雲端（flash）要花錢：先估請求數與費用，用 AskUserQuestion 問使用者同意才跑；不同意就註明沒量。個人集不送。
- 結果寫進 `docs/research/model-ladder.md` 新的一節，與 README。

採用門檻不改（spec〈採用門檻〉）。若口語長前文組上 200 字比 50 字多 1.0 點以上（門檻同一條），停下用 AskUserQuestion 問使用者要不要把 brief §8 改回 200 字，不要自己改；同樣，若結果動搖票 11 的其他結論（例如長句上雲端比基線多 6.0 點以上：1.4M 沒採用後，雲端的門檻改和基線比，spec〈採用門檻〉），寫明並問使用者，不自己推翻。

## 驗收

- [x] 依句長分組的表（口語、維基；各模型；首選、前 2 名、題數）在 `docs/research/model-ladder.md`。
- [x] 口語長前文題的建法、前文長度分佈、1.4M 在 0／50／200 字的結果（dev 與 test 分開）；雲端有量或註明沒量的原因。
- [x] 結論與票 11 的對照：哪些結論不變、哪些要使用者重新決定（附使用者的答覆）。
- [x] `cargo xtask check` 綠燈；個人集 baseline 不動。

## Comments

- 2026-11-11 做完。數字在 `scripts/experiments/model-ladder/README.md`〈依句長分組與口語長前文〉與 `docs/research/model-ladder.md` 同名一節。
- 口語長前文題：語料管線加 `Builder::add_dialogue`（HKCanCor 一段錄音的話輪一起給），寫 `build/corpus/hkcancor/{test,dev}-dialogue.tsv`（988、968 題，前文 p50 200 字）；train／dev／test 切分與 bench 檔沒變（`cargo xtask data` 後 `git status` 乾淨）。plan §5.2、`docs/development.md` 已補。
- 舊的逐題結果都不在（worktree 已刪，也是對字級 bigram 量的），全部在 `7862312` 的新基線重跑。1.4M 用協調者複製來的 `lstm-c0.5`；23M 照 README 重訓（44 分鐘，dev 損失和票 09 幾乎相同）。字表用 `train.py prepare` 重建（train 語料沒變）。
- 跑 test 的紀錄（各一次）：基線 `eval --split test`；1.4M `neural --split test --lambdas 2`（λ 在 dev 掃 0.5／1／2，口語與口語長前文都是 2 最好）；23M 同（dev 口語 λ 2 最好）；flash 送 test 口語長前文 2,964 次與 test 口語組 2,638 次。FM 只跑 dev。
- 使用者的答覆：(1) 測試接口照兩個（語料 Builder、model-ladder 的分組與分佈）。(2) flash 量 (a) test 口語長前文與 (b) dev 口語加維基（約 4.3 元）。(3) 看到 dev 口語 +7.3 後，付費照門檻量 test 口語組。(4) test 口語組 +7.9 對基線；先選「維持雲端不做」，協調者轉達使用者看過 1.4M 加回後的重算（比 1.4M 只多 2.4 點、要 3.0）才選的。(5) 協調者轉達：模型階梯的個人集改成只報不擋、1.4M 採用並恢復 11c–11e。flash 合計 12,282 次、約 5.5 元。
- 結論對照票 11：不變的有同步層、23M 不做（改成只因成本：比 1.4M +2.1 點已過正確率那條）、FM 不做、前文 50 字（口語長前文 200 比 50：1.4M +0.0、23M +0.2、flash −0.4）。改的有個人集門檻與 1.4M 採用、雲端改和 1.4M 比（仍不做）。brief §8 不用改回 200 字，沒觸發停下的條件。
- 個人集 baseline（`bench/personal/baseline.json`）沒動；這張票沒改核心與資料包，`cargo xtask bench --check` 不受影響。

- code-review（`7862312` 起，Standards 與 Spec 兩軸，含 revert 與文件）：數字抽查都對得上。已修：`docs/development.md` 的共用路徑段、名詞表補「口語長前文」並統一註解用詞、`dialogue_bench` 改名 `dialogue_items`、`build_corpus` 不再重複判斷對話語料、「全部」欄改用 `LengthCell::sum`、`pct` 共用、量測的 commit 寫成基線 `7862312`／程式 `657a40d`、`deps.rs` 註明 `qim-neural` 還沒建、brief §8 與 §12.5 補票 25 的雲端數字、重評條件兩處統一、spec 的「結論以口語與個人集為主」與〈採用門檻〉開頭、〈票序〉票 25 的 Blocked by 改回 11b、票 11c 過時的開工條件、「5–14 字最穩定」註明只指口語。改完重跑 `cargo xtask data`，口語長前文題與各 train 的 sha 不變。沒處理（判斷題，實驗程式）：`conversational()` 與 `segmented()` 實作相同（概念不同，留著）、`context-stats` 用集合名字尾切名字、`by-length` 先挑出 `--split` 再交給 `flags()`、`(字數, 首選, 前 2 名)` 沒包成型別、`read_length_rows` 與 `read_rows` 骨架相似、`Corpus::dialogues` 的 zip 路徑沒有直接測試（同既有的 `archive`）。
