# 09: 黑箱基線：Gboard

Status: ready-for-agent
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

- [ ] iPhone 版 Gboard 有沒有速成已確認並記下。
- [ ] `docs/research/blackbox-baseline.md` 有 Gboard 的 T1–T18，每項有結果或不能做的原因。
- [ ] brief §1.3 已更新（或註明沒有要寫回的）；可機械驗證的案例已分流。
- [ ] 沒有用真實 email、沒有登入個人帳號。

## Comments
