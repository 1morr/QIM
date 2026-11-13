# 05c: 翻頁 e2e 在高負載下被停鍵重排改了候選順序

Status: done
Blocked by: None (can start immediately)
讀: 票 05、06 的 Comments（翻頁 e2e 的既有競態）；plan §2.7（停鍵層）；`web/e2e/keys.spec.ts`、`web/e2e/key-table.spec.ts`

## What to build

協調者插入的修復票（2026-11-12）。`web/e2e/keys.spec.ts`「- = 與 PageUp PageDown 翻頁」與 `web/e2e/key-table.spec.ts` 的翻頁測試，在機器高負載時紅：測試打完 `ar` 立刻擷取候選列當 `first`，停鍵層的重排晚到、改了候選順序（例如第 2、3 位對調），之後的 `toHaveText(first)` 逾時。票 05 的 session 三次全套紅兩次，票 06 的 session 又紅一次；單獨重跑都過。整批並行跑票時機器常在高負載，會反覆擋住每張票。

- 先分清是產品的問題還是測試的問題：停鍵後重排改變候選順序是設計（plan §2.7）；但要確認翻頁中途重排抵達時，產品的行為（目前頁、高亮、使用者看到的順序）符合 plan，不是只把測試改綠。若產品在翻頁後被重排打亂頁面，那是產品 bug，照 tdd 修。
- 先寫出會紅的 repro：用可控的方式讓重排晚到（例如測試替身延遲重排結果、或 Playwright 控制時鐘），在任何負載下都能穩定重現紅燈。
- 修法：測試要等到重排穩定（或關掉停鍵層、或用確定的重排結果）再擷取 `first`；只拉長逾時而說不出原因的修法不收。

## 驗收

- [x] 會紅的 repro（不靠機器負載）與紅的輸出貼在 Comments；寫明是產品還是測試的問題。
- [x] 修後 repro 綠；兩個翻頁測試在高負載下（另開佔滿 CPU 的程序）`--repeat-each=10` 全綠，輸出貼在 Comments。
- [x] 若是產品問題，補 plan §2.7 的行為描述與對應的單元或 e2e 測試。（不適用：是測試的問題，見 Comments 第一條）
- [x] `cargo xtask check` 綠燈。

## Comments

- **是測試的問題，不是產品的。** 停鍵重排在翻頁中途回來時，核心照 plan §2.1〈重排〉把結果記著、看得到的那頁不變，回到第一頁才套用（`crates/qim-core/tests/rerank.rs` 的 `a_result_arriving_on_a_later_page_waits_for_the_first_page` 守核心）；兩個翻頁測試卻在重排回來前讀第一頁當 `first`，回到第一頁時順序已被重排改了。機器快時整個測試在停鍵 80 ms 內跑完、重排在測試結束後才要求，所以平常綠、高負載時紅。plan 不用改（「若是產品問題」那條不適用）。
- 網頁層原本沒有 e2e 守「翻頁中途回來」：補 `web/e2e/rerank.spec.ts`「翻頁中途重排回來：那頁不變，回到第一頁才插入」（`raise-last` 加 `?rerank-delay=1000`，確定翻頁後才回來）。
- repro（暫時檔 `web/e2e/repro-05c.spec.ts`，跑完刪掉）：照原本兩個翻頁測試的步驟，每步之間停 1 秒（超過停鍵 80 ms 加模型時間），不靠負載。修前兩條都紅，和票上描述的一樣是第 2、3 位對調（神經模型把「昭」排到第 2）：

  ```
  1) e2e/repro-05c.spec.ts:21:1 › keys.spec 的翻頁，每步停 1 秒
    Error: expect(locator).toHaveText(expected) failed
    - Expected  - 2
    + Received  + 2
        "1問",
    -   "2🙏",
    -   "3昭",
    +   "2昭",
    +   "3🙏",
  2) e2e/repro-05c.spec.ts:38:1 › key-table.spec 的翻頁，重排在第 1 頁晚到
    （同上的 diff）
  2 failed
  ```

- 修法：兩個翻頁測試只驗翻頁鍵，開頭把「重排」選單設成關（票上列的選項之一）；重排和翻頁的互動由上面那條 e2e 與核心測試守。沒選「等重排穩定再讀」：打 `ar` 時 `a` 也可能先要求一次重排，「等到一次結果」分不出是哪串碼的，還得在產品加狀態給測試讀。repro 套上同樣的修法後 `2 passed`。repro 靠神經模型剛好把「昭」排到第 2，換了模型可能不紅，所以沒留下；留下的回歸守門是上面那條 rerank e2e（用確定的 `raise-last`），它守產品行為，翻頁測試本身已不受重排影響。
- 高負載（每核心一個 `yes > /dev/null`，load average 17–21）下 `npx playwright test e2e/keys.spec.ts e2e/key-table.spec.ts e2e/rerank.spec.ts -g "翻頁" --repeat-each=10`：兩個翻頁測試、新的 rerank 測試與其他三條名字有「翻頁」的各 10 次，`60 passed (18.5s)`。
- 驗證：`cargo xtask check` 通過（cargo test 1032 passed；e2e 238 passed、2 skipped）。
- code-review 未處理：新 e2e 拿 `data-rerank-unchanged` 當結果回來的訊號，網頁之後若另記「延後套用」要跟著改（已在測試註解寫明）；`rerank.spec` 收尾幾行和上一條 raise-last 測試相似，兩處不抽。
- 順帶看到、沒改（不影響本票）：翻頁中收到的結果在網頁記成 `unchanged`（核心回同一個 `generation`），debug 面板因此顯示「名次沒變」，其實是延後套用；回到第一頁套用後快照的 `generation` 變了，網頁會再要求一次重排，路徑相同、名次相同，只是多算一次。

Session 紀錄：翻頁 e2e 的競態是測試的問題（重排翻頁中回來照 plan 延到第一頁）：兩個翻頁測試關掉重排，補翻頁中途回來的 e2e
