# 17: 粵語詞表

Status: ready-for-agent
Blocked by: 12
讀: plan §4.2（粵語口語詞表）；brief §1.5、§5.2（自行整理、不從 rime-cantonese 與粵典複製）；spec〈共同規則〉

## What to build

`data/lexicon/yue-colloquial.tsv` 初版，使用者審過後進詞庫資料包；固定題加 3–5 句粵語口語，全部通過。

- 自行整理常用粵語口語詞（CC0），不從 rime-cantonese、粵典或其他有授權的詞表複製（brief §5.2）；格式與詞頻欄怎麼給在本票定。
- 寫好初版後**停下，請使用者審**；使用者改過或同意後才接進資料包。沒審完不往下做。
- 口語詞併進詞庫資料包，`NOTICE` 列出這份詞表與 CC0。
- 選 3–5 句粵語口語句加進 `fixed.tsv`（不用個人集的內容）；不過的話先看是詞表、權重還是排序的問題，修到過，修不了的停下問。
- bench 前後對照。

## 驗收

- [ ] 使用者審過的詞表進資料包（`## Comments` 記使用者回覆的日期與改動摘要）。
- [ ] `fixed.tsv` 新增的 3–5 句全過，`cargo xtask bench --check` 綠燈。
- [ ] bench 前後對照貼在 `## Comments`。

## Comments
