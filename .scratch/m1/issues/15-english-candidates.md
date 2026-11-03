# 15: 英文候選

Status: ready-for-agent
Blocked by: 13
讀: plan §2.4（規則 4）、§4.4；brief §4.3、§4.7、§5.2、§12.1；`scripts/experiments/english-edit1/`；spec〈共同規則〉

## What to build

中文狀態下，輸入碼剛好是完整英文單字時，第 2 位出現那個英文字；不容錯、不補全，不搶首選。

- SCOWL / ESDB（`en-wl/wordlist`）size ≤ 60 的詞釘進 `sources.toml`，建成英文資料包（自己的 `LICENSE`、`NOTICE`）；只收能用 a–y 打出的字（`z` 不在速成字母表）。大小寫怎麼處理在本票定，參考黑箱 T7（已做完的話）。
- 排位規則 4：放第 2 位；和規則 1–3 的先後照 plan §2.4。
- `Options.english_candidates` 開關（預設開），網頁加開關。
- bench 前後對照：英文候選只佔第 2 位，首選正確率不應改變；有改變就查明。

## 驗收

- [ ] 核心測試：`the` 第 2 位是 `the`；非完整單字（如拼錯一碼）不出英文候選；關掉選項不出。
- [ ] `--verify` 報英文詞數；建兩次 sha256 相同。
- [ ] Playwright：中文狀態打 `the`，讀回第 2 位是 `the`，按 `2` 上屏 `the`。
- [ ] bench 前後對照貼在 `## Comments`。

## Comments
