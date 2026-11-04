# 09: 黑箱基線：Gboard

Status: done
Blocked by: None (can start immediately)
讀: ADR 0006；`docs/research/closed-source-imes.md` §1.1、§3、§4；brief §1.3；spec〈共同規則〉；票 08（若已完成，沿用 `docs/research/blackbox-baseline.md` 的格式）

## What to build

照 T1–T18 實測 Gboard 速成：Android 模擬器，加上 iPhone 版。開工先確認 iPhone 版 Gboard 有沒有速成（目前只有第三方整理的說法）；沒有就只測 Android，註明（plan §8）。

- Android 用 `orca emulator-android`，iPhone 用 `iphone-use`；iPhone 沒接上就停下問。
- 規則同票 08：只讀畫面、不逆向；T5 只用 `bc12345@example.edu`，不登入個人 Google 帳號（模擬器需要帳號才能裝 Gboard 時，停下問使用者怎麼處理）。
- T5 系列（email 的來源假設 H1–H4）照 `closed-source-imes.md` §3 的推論記錄。
- 結果寫進 `docs/research/blackbox-baseline.md`（票 08 沒做就本票建檔）；截圖放 `.scratch/m1/evidence/`。
- brief §1.3、`fixed.tsv` 的分流同票 08。

## 驗收

- [x] iPhone 版 Gboard 有沒有速成已確認並記下。（記為未確認：使用者決定不在 iPhone 裝 Gboard，比照 plan §8 的備案只測 Android，見 Comments）
- [x] `docs/research/blackbox-baseline.md` 有 Gboard 的 T1–T18，每項有結果或不能做的原因。
- [x] brief §1.3 已更新（或註明沒有要寫回的）；可機械驗證的案例已分流。
- [x] 沒有用真實 email、沒有登入個人帳號。

## Comments

- 2026-11-04：結果在 `docs/research/blackbox-baseline.md`〈環境：Gboard〉〈結果：Gboard〉，截圖 `.scratch/m1/evidence/09-android-*.png`（每張是同一項的幾個畫面拼成，左邊紅字標註）；brief §1.3 的 Gboard 列、§12.4 加一條。
- iPhone 版：iPhone 上沒裝 Gboard，問過使用者，決定不安裝。App Store 說明與 Gboard 說明的 iOS 語言清單只列繁體中文（台灣、香港），沒提速成，所以仍未確認；比照 plan §8「iPhone 版沒有速成」的備案只測 Android（備案原本的條件是確認沒有，這次是沒測）。
- 加進 `bench/fixed.tsv`：`r` 口、`dp` 想、`ov` 食（iOS 與 Gboard 一致，QIM 現在就過）。
- 還沒通過、Gboard 與 iOS 一致的案例（括號裡是 QIM 現在的首選），和票 08 Comments 那幾條同一批，預計票 11、12 處理：
  - `rn` 嘢（別）
  - `ri` 啲（戰）
  - `hidpovrn` 我想食嘢（我想食別）
- 只有 Gboard 這樣、不當目標的首選（括號裡是 QIM）：`hi` 的（的，iOS 與 macOS 是我）、`os` 作（作，iOS 是佢）、`oe` 假（雙，iOS 是隻）、`hh` 身（物）、`yp` 意（此）、`hv` 很（委）。
- 做不到或沒做完的：T5c（學習觸發不穩，重開就會清掉，量不到刪除本身的效果）、T9（模擬器量不出手機的延遲）、T3b（T3 沒有前移）。T5a 第三輪學不起來的原因沒查出，不再深挖。
- 對使用者環境的影響：新建 AVD `qim_t09_gboard`（約 2.5 GB，已關機，留著給之後重測）；不要了可用 `~/Library/Android/sdk/cmdline-tools/latest/bin/avdmanager delete avd -n qim_t09_gboard` 刪。使用者原有的 AVD `Medium_Phone_API_37.0` 沒開也沒改。iPhone 沒動。AVD 內的系統 autofill 服務測完已改回。
- `oe` 不收進目標：iOS 的「隻」是在 T6 打過前文後讀到的，不是乾淨的首選，「隻」也不是粵語特有字；三者首選各不同（iOS 隻、Gboard 假、QIM 雙）。
- code-review 未處理的發現：
  - 驗收第 1 條打了勾，但結果是「未確認」；依使用者決定比照 plan §8 的備案，已在 progress〈偏差與決定〉記一行。
  - T4 沒單獨看第 5 次；T7 沒測大小寫（速成佈局沒有 Shift）；T5c 錯過了第二輪學到時做的時機。要補得重開模擬器重來，不值得。
  - 本票 Comments 的「只有 Gboard 這樣」清單附了 QIM 現在的首選，和研究文件〈首選對照〉部分重複；留著，因為研究文件不記 QIM 的首選。
  - 「T17 罕用字」照票 08 沿用測試表的項目名，沒改成名詞表的詞。
