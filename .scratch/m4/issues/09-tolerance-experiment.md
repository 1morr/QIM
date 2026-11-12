# 09: 容錯實驗

Status: ready-for-agent
Blocked by: 05, 08
讀: brief §4.8；spec〈細節〉〈容錯〉；`docs/research/spatial-correction-swipe.md` §4、`docs/research/ranking-techniques.md` §5；spec〈容錯採用門檻〉〈每鍵延遲上限〉〈共同規則〉

## What to build

在 `scripts/experiments/tolerance/` 量容錯值不值得進核心（形碼的容錯沒有任何先例）：

- 修正只落在詞庫詞或使用者詞的邊上，每條邊最多改一處，單字邊永遠不修（brief §4.8）。
- 四種修正（漏碼、多碼、兩碼對調、鄰鍵替換）各自量，也量合起來；鄰鍵替換的懲罰用 `Weighted` 的機率。「只在無候選時啟動」對速成幾乎不觸發，不用。
- 懲罰只在 dev 調；test 與個人集只在定案後跑一次。
- 量正確率、「原樣正確卻被改錯」的比例、每鍵延遲（lattice 放大多少）。改錯比例用票 08 的量法（`qim_bench::score_set` 的 `SetResult::typo_stats`：沒打錯、容錯關時首選正確的題裡，容錯開時首選不正確的比例），各修正分開報；題目用 `bench/colloquial/typo/`、`touch/` 與 dev 的 `dev-typo.tsv`、`dev-touch.tsv`（`qim_bench::parse_typo`、`parse_touch`）。

結論：哪幾種修正過門檻、懲罰值、成本。沒過就不進核心，票 10 改成 wontfix 並寫明重評條件。

## 驗收

- [ ] README（日期、機器與系統、commit、量法、結果表）。
- [ ] 每種修正對照〈容錯採用門檻〉列出過或沒過，各附票 08 量法的「原樣正確卻被改錯」比例；延遲對照〈每鍵延遲上限〉。
- [ ] 結論寫進 `docs/research/`，摘進 brief〈查證過的事實〉；票 10 依結論更新。
