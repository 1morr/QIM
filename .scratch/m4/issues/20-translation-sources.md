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

- 等使用者：譯詞來源與生成模型（見 `docs/research/translation-sources.md`〈結論〉），協調者已轉使用者。兩題：(1) 來源用 LLM 離線生成（建議）、CC-CEDICT＋CC-Canto（BY-SA 4.0，免費，詞前 2 萬約 48% 有合用譯詞）或兩者混用；(2) 生成用哪個模型、怎麼付費：Claude Sonnet 5.5 Batch（建議，25 萬詞約 $17）、Haiku 5.5 Batch（約 $1）或 Claude Code 子代理（耗訂閱額度）。
- 做到：研究文件、覆蓋率腳本 `scripts/experiments/gloss-coverage/`、事實 T34–T38 已 commit；各來源的授權原文連結與能不能用在研究文件 §1。
- 剩下：照使用者答案改 brief（§1.5 譯詞那條、§5.2 加來源列、§11、〈事實索引〉T 的條數 33→38）；票 29 依結論更新（來源、授權、生成步驟與產物放哪）；勾驗收、Status 改 done、補 Session 紀錄與決定行。
