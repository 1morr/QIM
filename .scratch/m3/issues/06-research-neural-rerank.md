# 06: 讀青簡的神經重排與 CyIME 的模型評估

Status: done
Blocked by: None (can start immediately)
讀: spec〈模型階梯實驗〉；`docs/research/qingjian.md` §4、§5；`docs/research/cyime.md` §3

## What to build

階梯實驗開工前，把兩個專案怎麼訓練、評估小型模型的做法讀清楚，記下 QIM 可照做的部分（兩者都是 GPL，只取做法、不抄程式碼）。用 `mattpocock-skills:research`。

- 青簡：`qingjian-neural` 與 `docs/notes/neural-rescoring.md` 的原文：模型結構、訓練語料與步數、重排幾條路徑、前綴快取、分數怎麼和詞圖結合、作者記下的坑。
- CyIME：九宮格模型評估的文件：模型大小與正確率的關係、評估集怎麼造、指標。
- 對 QIM：列出階梯各票（基線、同步 n-gram、神經模型、LLM）可直接採用的做法與要避開的坑。

## 驗收

- [x] `docs/research/` 有一份筆記（或補進 `qingjian.md`、`cyime.md` 的新節），每個事實附原始檔路徑與 commit。
- [x] 最後一節「對階梯實驗」逐條指向 spec〈模型階梯實驗〉的哪一列。
- [x] `cargo xtask check` 綠燈。

## Comments

- 筆記寫成新檔 `docs/research/neural-rerank.md`（青簡 `729ecba`、CyIME `5962b8b`）；`qingjian.md` §4、`cyime.md` §3 各加一行指向它的〈對舊紀錄的更正〉，舊內容不改寫。結論摘進 brief §12.4。
- CyIME 的 `docs/development/` 在 `5754fb0a` 被 `.gitignore` 排除、HEAD 已無；評估文件改用上次的完整 SHA `31ff80e84405264115ba4443f3338c2dbb20b831` 從 GitHub 取原文，筆記記作 `C0`。
- 給後面各票的重點（細節在筆記 §5）：
  - 票 07：兩個專案都沒有「修正次數」的定義，要自己定；QIM 的 bench 集合沒有調參／終評切分，後面各票要掃 λ、權重、路徑數，票 07 要定怎麼切。
  - 票 09：青簡訓練程式碼不在 repo，步數、batch、硬體都查不到，要自己定；青簡 23M 與 36M 打平，1.4M 對 23M 沒有現成答案。
  - 票 10：兩個專案都沒有 LLM 重排或提示的做法；CyIME 記下 `deepseek-flash` 多次逾時，要報逾時率。
- code-review（從 `4bcee1c` 起，Standards 與 Spec 兩軸）已處理：出處補完整路徑與行號、補上缺的出處；混進的簡體字改掉；§5 每組小標標明對應 spec 受測模型表的哪一列或表外哪一項，補〈採用門檻〉與字級 trigram 兩組，給票 07 的兩條標成「建議」；Common Voice／KN 那條原本答非所問（對到洩漏），已改；青簡三元插值補進 §1.5；brief 新條刪掉和青簡那條重複的「5／9／18 條」。
- code-review 未處理（判斷題，保留）：§2.2 重抄 `cyime.md` §3 的表，因為多了「Rime 原生」一欄，放一起比較好讀；本節給票 07、09、10 的重點和筆記 §5 重複，這裡只留摘要、細節以筆記為準；「線程」「搜索」這類大陸用詞照 `qingjian.md` 原用法不改。
