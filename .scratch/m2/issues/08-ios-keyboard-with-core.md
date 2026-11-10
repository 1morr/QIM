# 08: 鍵盤擴充接上核心：在 iPhone 打出「門口」

Status: ready-for-agent
Blocked by: 04, 06, 07
讀: brief §9、§12.3；plan §2.1、§3.3、§7.3；spec〈共同規則〉；`docs/research/ios-feasibility.md`（票 04、06、07 的節）

## What to build

把 `qim-ffi` 接進票 06 的鍵盤擴充，連同 M1 的資料包放進擴充，在 iPhone 的真實宿主程式打出「門口」，並量常駐記憶體與每鍵延遲。只要能打字讀回，不做佈局與產品 UI：幾個固定鍵（`a`、`n`、`r`、空白）加一行候選就夠。

- 交叉編譯 `qim-ffi` 到 `aarch64-apple-ios`，接進 Xcode 專案；資料包放進擴充的 bundle（或 App Group，看票 06 的結果），選哪些資料包依票 04 的數字。
- 記憶體：載入每份資料包前後、打字中的常駐記憶體，扣掉票 06 空擴充的基線。
- 延遲：在擴充內量每鍵到快照回來的時間 p50 / p95，含選開頭一段的那一鍵；只量核心呼叫，另記一次含畫面更新的大概值。
- 結果寫進 `docs/research/ios-feasibility.md` 的〈鍵盤擴充接上核心〉一節，附機型與 iOS 版本。

## 驗收

- [ ] 備忘錄裡打 `anr` 讀回首選「門口」、空白後讀回上屏「門口」；截圖 `.scratch/m2/evidence/08-*.png`。
- [ ] 常駐記憶體：每份資料包的增量與總量都有數字。
- [ ] 每鍵 p50 / p95 都有數字，和 brief §9 的 50 ms 目標比較。
- [ ] README 的重量步驟更新；`cargo xtask check` 綠燈。

## Comments

- 票 07 轉入：`qim-ffi` 的 `Session` 建構子自己載入資料包、各建一份 `Engine`（`crates/qim-ffi/src/lib.rs` 的 `#[uniffi::constructor]`），不符 plan §2.1「`Engine` 不可變，多個 `Session` 共用」。資料包載入後約 96 MB（票 04，Mac），擴充上限推得約 177 MB（票 06）：換輸入框或重載畫面時另建 `Session`、舊的沒放掉，記憶體就翻倍、擴充可能被終止，量到的數字也失真。本票擴充全程只建一個 `Session`，或把 `Engine` 拆成獨立的 uniffi Object 給各 `Session` 共用（改介面就同步 plan §2.1）；量記憶體前確認沒有重複的 `Session`。
