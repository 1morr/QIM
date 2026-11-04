# 08: 黑箱基線：Apple 內建速成

Status: done
Blocked by: None (can start immediately)
讀: ADR 0006；`docs/research/closed-source-imes.md` §1.2、§4；brief §1.3；spec〈共同規則〉

## What to build

照 `docs/research/closed-source-imes.md` §4 的 T1–T18，實測 macOS 內建速成與 iOS 內建速成，記下競品實際行為，給 QIM 的取捨（票 10 的標點與大寫、票 12 的試打、票 24 的並列）當依據。

- 只打字、讀回畫面，不開啟產品的檔案、不逆向（ADR 0006）。
- macOS 用 `orca computer`，在「文字編輯」裡打；iPhone 用 `iphone-use`。iPhone 要使用者接上：沒接上就停下問，不跳過硬做。
- T5 只用測試字串 `bc12345@example.edu`，不用使用者的真實 email、不登入個人帳號。清除學習資料前先確認不會刪掉使用者的資料，會的話先問。
- 測試表標明不適用的項目（如 T5 只對 Gboard）照表跳過；做不到的寫原因。
- 結果寫進 `docs/research/blackbox-baseline.md`（本票建檔）：版本、裝置、系統、每項結果、截圖路徑（截圖放 `.scratch/m1/evidence/`）。
- QIM 要達到或刻意不同的項目寫回 brief §1.3；可機械驗證的案例（輸入碼 → 期望首選）且 QIM 已通過的進 `fixed.tsv`，還沒通過的列在 `## Comments

- 2026-11-04：結果在 `docs/research/blackbox-baseline.md`，截圖 `.scratch/m1/evidence/08-*.png`；加進 `fixed.tsv` 的題與取捨見該文件〈首選對照〉〈給 QIM 的取捨〉。
- 還沒通過的可機械驗證案例（iOS 內建速成的首選；括號裡是 QIM 現在的首選），等通過的票加進 `bench/fixed.tsv`，預計是票 11、12（bigram 與口語權重）：
  - `hi` 我（的）
  - `rn` 嘢（別）
  - `ri` 啲（戰）
  - `os` 佢（作）
  - `hidpovrn` 我想食嘢（我想食別）
- 對使用者環境的影響：macOS 暫時啟用速成，測完停用、切回 ABC；兩邊速成的學習資料多了「`bc` → 貝、冥在前」等最近使用紀錄，沒有刪任何資料。iOS 臨時備忘錄已刪（在「最近刪除」）；飛航模式已關回。
- T9 macOS 延遲的量測工具在 `scripts/experiments/blackbox-latency-macos/`，換 macOS 主版本時重量。
- code-review 未處理的發現：
  - 首選對照的 iOS 欄除 T1 外沒有截圖，只記讀到的候選。補截圖要重新接手機，不值得。
  - 「T17 罕用字」沿用測試表的項目名，沒改成名詞表的「擴展區」。
