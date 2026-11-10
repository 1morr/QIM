# 06: 讀青簡的神經重排與 CyIME 的模型評估

Status: ready-for-agent
Blocked by: None (can start immediately)
讀: spec〈模型階梯實驗〉；`docs/research/qingjian.md` §4、§5；`docs/research/cyime.md` §3

## What to build

階梯實驗開工前，把兩個專案怎麼訓練、評估小型模型的做法讀清楚，記下 QIM 可照做的部分（兩者都是 GPL，只取做法、不抄程式碼）。用 `mattpocock-skills:research`。

- 青簡：`qingjian-neural` 與 `docs/notes/neural-rescoring.md` 的原文：模型結構、訓練語料與步數、重排幾條路徑、前綴快取、分數怎麼和詞圖結合、作者記下的坑。
- CyIME：九宮格模型評估的文件：模型大小與正確率的關係、評估集怎麼造、指標。
- 對 QIM：列出階梯各票（基線、同步 n-gram、神經模型、LLM）可直接採用的做法與要避開的坑。

## 驗收

- [ ] `docs/research/` 有一份筆記（或補進 `qingjian.md`、`cyime.md` 的新節），每個事實附原始檔路徑與 commit。
- [ ] 最後一節「對階梯實驗」逐條指向 spec〈模型階梯實驗〉的哪一列。
- [ ] `cargo xtask check` 綠燈。

## Comments
