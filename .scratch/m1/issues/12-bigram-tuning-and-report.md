# 12: bigram 調參與「回報」

Status: ready-for-agent
Blocked by: 04, 05, 08, 09, 11
讀: plan §2.3、§2.5（`[ranking]`）、§4.3（口語權重）、§5.2；brief §9；`docs/research/blackbox-baseline.md`；spec〈共同規則〉

## What to build

用 dev 切分調出 β、γ 與口語語料權重，寫進方案檔，定下 M1 的正式 baseline；網頁加「回報」，使用者試打遇到不滿意的首選能一鍵存成固定題。

- 調參只看 dev 切分（維基 dev、口語 dev），不看 test 與個人集；搜尋方式（格點或其他）與結果寫進 `## Comments`。
- 口語權重調完後，比較「加口語前 / 後」：口語集修正次數要下降；維基集首選正確率若退步，退多少寫進 `## Comments`。
- 延遲：原生 p95 ≤ 10 ms、WebAssembly p95 ≤ 50 ms（含 20 碼以上）；不達標時照 plan §8「延遲超過預算」調 beam 或剪枝。
- 網頁「回報」：把剛才那次組字的輸入碼和使用者要的字匯出成 `fixed.tsv` 的一行（使用者要的字怎麼輸入在本票定）。
- 黑箱基線裡可機械驗證、現在已通過的案例加進 `fixed.tsv`。
- 做完請使用者在網頁試打（不擋後續的票），回報的案例在之後的票加進 `fixed.tsv`；在 progress 的下一步註明。

## 驗收

- [ ] β、γ、口語權重寫進 `data/schemes/quick.toml`；`baseline.json` 更新並寫理由。
- [ ] bench 分開列出維基、口語、個人三個集合的指標；加口語前後的比較在 `## Comments`。
- [ ] 每鍵 p95：原生 ≤ 10 ms、WebAssembly ≤ 50 ms，含 20 碼以上（貼輸出）。
- [ ] Playwright：打一段、按「回報」，讀回匯出的那一行格式正確。
- [ ] `cargo xtask bench --check` 綠燈。

## Comments
