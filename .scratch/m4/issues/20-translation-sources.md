# 20: 譯詞的來源與授權

Status: ready-for-agent
Blocked by: 18
讀: brief §1.5（譯詞）、§5、§11；spec〈細節〉（譯詞）；skill `mattpocock-skills:research`

## What to build

找繁體與粵語對英文的詞表，能做成可選資料包散布：

- CC-CEDICT、CC-Canto 是不是 share-alike、能不能以資料包散布；粵典的英文釋義是非商業條款；其他候選來源。
- 查不到合用的，評估自己用 LLM 離線生成（青簡做法）：成本、生成結果的授權、品質抽查方法。要花錢的先問使用者。

輸出 `docs/research/translation-sources.md`，結論摘進 brief §12.1 與 §11。

## 驗收

- [ ] 每個來源附授權條款原文的連結與能不能用。
- [ ] 結論：採用哪個來源，或 LLM 生成的方案與估計費用；brief 更新；票 29 依結論更新。
