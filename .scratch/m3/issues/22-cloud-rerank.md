# 22: 雲端重排與雲端安全

Status: wontfix
Blocked by: 05, 11
讀: `docs/research/model-ladder.md`〈雲端層〉；brief §1.5、§8；spec〈採用門檻〉雲端層

## What to build

不做（票 11 依模型階梯結論改寫，使用者 2026-11-11 同意）。DeepSeek flash 重排口語前 2 名 59.5%，比基線多 10.4 點、延遲與費用都過，但只比已採用的本地停鍵層（1.4M，57.5%）多 2.0 點，沒過「再多 3.0 點」；照 spec〈採用門檻〉，雲端的範圍是「不做」。Foundation Models 也沒過。M3 不接任何雲端 LLM，brief §8 的雲端規則留給重評或之後的雲端功能。

重評條件：票 11d 在詞級 trigram 的新基線上重量 1.4M 後不採用時，本地停鍵層就沒有已採用的模型，雲端改成和新基線比（+6.0 點）。那時把本票改回 `ready-for-agent`，先付費重量 flash（test 一次約 7 元，先問使用者），過了才照下面原本的範圍做；選項要改成票 11c 的路徑格式。

原本的範圍（保留給重評）：`qim-core` 的純函式組提示（照 `scripts/experiments/llm-rerank/README.md` 的 `hint-paths`、前文 50 字）、解析回覆、遮蔽 email／電話／卡號、每日上限、排除指定 App；網頁經 Vite 轉發、支援 OpenAI 相容與 Anthropic 兩種介面、標示來源。重排和排位規則 3、4 的先後已搬到票 11c。

## 驗收

- [x] 不做的理由與重評條件寫在本票、`docs/research/model-ladder.md` 與 brief §1.5、§8。

## Comments

- 2026-11-11（票 15 留）：重排要求的前 9 個候選可能含記住的字串（使用者打過的 email，plan §2.4 規則 5），組提示時候選也要遮蔽或排除，不只前文。
- 2026-11-11（票 11）：照模型階梯結論改成 wontfix；重排的選項改看路徑（票 11c）後，這條遮蔽的要求重評時一樣適用（路徑裡也可能有記住的字串）。
