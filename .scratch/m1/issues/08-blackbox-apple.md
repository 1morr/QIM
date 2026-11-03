# 08: 黑箱基線：Apple 內建速成

Status: ready-for-agent
Blocked by: None (can start immediately)
讀: ADR 0006；`docs/research/closed-source-imes.md` §1.2、§4；brief §1.3；spec〈共同規則〉

## What to build

照 `docs/research/closed-source-imes.md` §4 的 T1–T18，實測 macOS 內建速成與 iOS 內建速成，記下競品實際行為，給 QIM 的取捨（票 10 的標點與大寫、票 12 的試打、票 24 的並列）當依據。

- 只打字、讀回畫面，不開啟產品的檔案、不逆向（ADR 0006）。
- macOS 用 `orca computer`，在「文字編輯」裡打；iPhone 用 `iphone-use`。iPhone 要使用者接上：沒接上就停下問，不跳過硬做。
- T5 只用測試字串 `bc12345@example.edu`，不用使用者的真實 email、不登入個人帳號。清除學習資料前先確認不會刪掉使用者的資料，會的話先問。
- 測試表標明不適用的項目（如 T5 只對 Gboard）照表跳過；做不到的寫原因。
- 結果寫進 `docs/research/blackbox-baseline.md`（本票建檔）：版本、裝置、系統、每項結果、截圖路徑（截圖放 `.scratch/m1/evidence/`）。
- QIM 要達到或刻意不同的項目寫回 brief §1.3；可機械驗證的案例（輸入碼 → 期望首選）且 QIM 已通過的進 `fixed.tsv`，還沒通過的列在 `## Comments` 等通過的票加。

## 驗收

- [ ] `docs/research/blackbox-baseline.md` 有 macOS 與 iOS 內建速成的 T1–T18，每項有結果或不能做的原因。
- [ ] brief §1.3 已更新（或註明沒有要寫回的）。
- [ ] 可機械驗證的案例已分流（進 `fixed.tsv` 或列在 Comments）。
- [ ] 沒有用真實 email、沒有登入個人帳號、沒有刪掉使用者的學習資料。

## Comments
