# 07: 階梯實驗的框架、基線與採用門檻

Status: ready-for-agent
Blocked by: 04, 06
讀: spec〈模型階梯實驗〉〈共同規則〉；plan §2.7、§5.2；ADR 0005；票 06 的研究筆記；`docs/research/ios-feasibility.md`〈小型神經模型〉；`.scratch/m2/issues/12-m2-close.md` Comments 的「M3」各條

## What to build

建 `scripts/experiments/model-ladder/`：之後三張實驗票都在這個框架上換模型比數字。

- 從核心匯出每題的候選與前 N 條路徑（含前文長度 0、50、200 字三種），存成實驗用的中間檔；N 與路徑長度在這裡定（M2 票 11 的 6 條、8 字是近似），定了之後重算 M2 的神經模型延遲估計並註明。
- 指標照 spec：修正次數、首選正確率、「最高到第 2 位」規則下的前 2 名命中率；依集合（維基、口語、個人、有前文）分開。
- 量出基線（現在的字級 bigram 加 `set_context`）。
- 依基線**提出**採用門檻（每層要改善多少、延遲與大小與費用的上限），停下給使用者確認；確認後寫進 spec〈採用門檻〉，之後不改。

## 驗收

- [ ] README 有日期、機器、commit、怎麼重跑、基線結果表。
- [ ] 實驗 crate 不在 workspace 裡；個人集只報數字。
- [ ] spec〈採用門檻〉已填，Comments 記使用者確認的日期。
- [ ] `cargo xtask check` 綠燈。

## Comments
