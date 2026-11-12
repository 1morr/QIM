# 05c: 翻頁 e2e 在高負載下被停鍵重排改了候選順序

Status: ready-for-agent
Blocked by: None (can start immediately)
讀: 票 05、06 的 Comments（翻頁 e2e 的既有競態）；plan §2.7（停鍵層）；`web/e2e/keys.spec.ts`、`web/e2e/key-table.spec.ts`

## What to build

協調者插入的修復票（2026-11-12）。`web/e2e/keys.spec.ts`「- = 與 PageUp PageDown 翻頁」與 `web/e2e/key-table.spec.ts` 的翻頁測試，在機器高負載時紅：測試打完 `ar` 立刻擷取候選列當 `first`，停鍵層的重排晚到、改了候選順序（例如第 2、3 位對調），之後的 `toHaveText(first)` 逾時。票 05 的 session 三次全套紅兩次，票 06 的 session 又紅一次；單獨重跑都過。整批並行跑票時機器常在高負載，會反覆擋住每張票。

- 先分清是產品的問題還是測試的問題：停鍵後重排改變候選順序是設計（plan §2.7）；但要確認翻頁中途重排抵達時，產品的行為（目前頁、高亮、使用者看到的順序）符合 plan，不是只把測試改綠。若產品在翻頁後被重排打亂頁面，那是產品 bug，照 tdd 修。
- 先寫出會紅的 repro：用可控的方式讓重排晚到（例如測試替身延遲重排結果、或 Playwright 控制時鐘），在任何負載下都能穩定重現紅燈。
- 修法：測試要等到重排穩定（或關掉停鍵層、或用確定的重排結果）再擷取 `first`；只拉長逾時而說不出原因的修法不收。

## 驗收

- [ ] 會紅的 repro（不靠機器負載）與紅的輸出貼在 Comments；寫明是產品還是測試的問題。
- [ ] 修後 repro 綠；兩個翻頁測試在高負載下（另開佔滿 CPU 的程序）`--repeat-each=10` 全綠，輸出貼在 Comments。
- [ ] 若是產品問題，補 plan §2.7 的行為描述與對應的單元或 e2e 測試。
- [ ] `cargo xtask check` 綠燈。
