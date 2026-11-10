# 18: 完整粵語詞庫：查授權，查清才建

Status: ready-for-agent
Blocked by: 02
讀: brief §1.5、§5.1、§5.2、§11、§12.1；plan §4.2；`docs/research/data-sources.md`；spec〈範圍〉〈共同規則〉

## What to build

現在的粵語口語詞只有自己整理的 196 個。查有沒有授權乾淨、能隨商店 App 發佈的完整粵語詞庫，有就建進資料，沒有就寫明理由留到之後。

- 先查證（`mattpocock-skills:research`，結果放 `docs/research/`）：候選來源例如 rime-cantonese、粵典（words.hk）、HKCanCor 的詞表、維基詞典；每個的授權、能否商用與再散布、署名要求、格式與詞數。結論摘進 brief §12.1 並附來源。
- 查清有可用的：釘進 `sources.toml`，照 plan §4.2 的規則併進詞庫資料包（或另一份資料包，票內定），寫回 plan §4.2。
- 都不行或不確定：停下問使用者；決定不做時寫回 brief §1.5（留在「之後」），本票照樣結案。

## 驗收

- [ ] `docs/research/` 有查證紀錄，brief §12.1 有結論與來源。
- [ ] 做了的話：`cargo xtask data --verify` 綠燈、`LICENSE`／`NOTICE` 完整；口語集 `bench --check` 不變差，貼對照。
- [ ] 沒做的話：brief §1.5 寫明原因。
- [ ] `cargo xtask check` 綠燈。

## Comments
