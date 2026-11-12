# 04: 簡體模式的前文轉回繁體

Status: ready-for-agent
Blocked by: None (can start immediately)
讀: plan §2.3、§2.7（簡體模式那條）、§4.4；spec〈共同規則〉

## What to build

簡體模式下，殼給的前文是簡體字，但同步層的統計（字級 bigram、詞級 trigram）都是繁體：簡體字沒有統計，前文等於沒用，退回 unigram。停鍵層已經把前文逐字轉回繁體（`Simplifier::traditional_chars`，M3 票 11e），同步層照做。個人 bigram 的前文要不要也轉，由票定（它記的是上屏的原文還是轉換後的字，先查清楚）。

## 驗收

- [ ] 先寫會紅的核心測試：簡體模式的前文接一串碼，首選和繁體模式下對應繁體前文的首選相同；修後綠。
- [ ] 個人 bigram 的前文照決定處理，有測試。
- [ ] Playwright：簡體模式下前文改變候選；截圖。
- [ ] `cargo xtask bench --check` 前後對照（基準是繁體，預期不變）；plan §2.7 刪掉「同步層 bigram 的前文還沒轉」；`cargo xtask check` 綠燈。
