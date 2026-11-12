# 22: 英文資料包換成 aosp-dictionaries

Status: ready-for-agent
Blocked by: 18, 18b
讀: brief §4.3、§4.7、§5、§11；plan §2.4（規則 4）、§4.4；`docs/research/spatial-correction-swipe.md` §6；spec〈共同規則〉（資料來源）

## What to build

英文資料包改用 aosp-dictionaries 的 experimental 詞典（CC BY 4.0，逐詞頻率與 bigram、含 `z` 開頭的詞），取代 `scowl-en-us`；同一份給英文模式與中文狀態的規則 4：

- 來源釘進 `data/sources.toml`，資料包的 `LICENSE`、`NOTICE` 跟著換。
- `z` 只在中文組字的比對時濾掉（plan §4.4）。
- 用 `bench` 確認中文狀態的第 2 位不會被多搶：規則 4 觸發的題數前後並列。

## 驗收

- [ ] `cargo xtask data --verify` 決定性；`LICENSE`、`NOTICE` 只寫用到的來源。
- [ ] `bench --check` 前後對照與規則 4 觸發題數附在 Comments，變多的說明原因。
- [ ] Playwright：中文狀態打 `receive` 第 2 位是 receive，打 `recieve` 不出英文候選。
- [ ] plan §4.4 與 brief 相關段落更新；`cargo xtask check` 綠燈。
