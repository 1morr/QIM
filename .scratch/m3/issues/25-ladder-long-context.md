# 25: 階梯補測：依句長分組與口語長前文

Status: ready-for-agent
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

- [ ] 依句長分組的表（口語、維基；各模型；首選、前 2 名、題數）在 `docs/research/model-ladder.md`。
- [ ] 口語長前文題的建法、前文長度分佈、1.4M 在 0／50／200 字的結果（dev 與 test 分開）；雲端有量或註明沒量的原因。
- [ ] 結論與票 11 的對照：哪些結論不變、哪些要使用者重新決定（附使用者的答覆）。
- [ ] `cargo xtask check` 綠燈；個人集 baseline 不動。

## Comments
