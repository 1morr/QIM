# 20: 譯詞的來源與授權

Status: ready-for-agent
Blocked by: 18
讀: brief §1.5（譯詞）、§5、§11；spec〈細節〉（譯詞）；skill `mattpocock-skills:research`

## What to build

找繁體與粵語對英文的詞表，能做成可選資料包散布：

- CC-CEDICT、CC-Canto 是不是 share-alike、能不能以資料包散布；粵典的英文釋義是非商業條款；其他候選來源。
- 查不到合用的，評估自己用 LLM 離線生成（青簡做法）：成本、生成結果的授權、品質抽查方法。要花錢的先問使用者。

輸出 `docs/research/translation-sources.md`，結論摘進 `docs/research/facts-tables.md` 與 §11。

## 驗收

- [ ] 每個來源附授權條款原文的連結與能不能用。
- [ ] 結論：採用哪個來源，或 LLM 生成的方案與估計費用；brief 更新；票 29 依結論更新。

## Comments

- 等使用者：打中文時候選旁要顯示英文譯詞（例「發票」旁的 invoice），譯詞從哪來？(1) 用 LLM 離線生成整份詞庫（約 25 萬詞），Claude Sonnet 5.5 Batch 約 $17、Haiku 5.5 約 $1，要你提供 API 金鑰並盲評 100 詞，輸出可標 CC0；(2) 免費的 CC-CEDICT 加 CC-Canto（CC BY-SA 4.0），但前 2 萬個常用詞只有約 48% 拿得到能直接上屏的譯詞；(3) CC-CEDICT 為主、LLM 補缺，整包仍是 BY-SA。票 20 的建議是 (1) 用 Sonnet。不決定的話票 20 收不了尾（brief 要寫進採用的來源），票 29 譯詞資料包不能開工；協調者的隊列停在票 20。研究在票 20 的分支 `1morr/t20` 的 `docs/research/translation-sources.md`。
