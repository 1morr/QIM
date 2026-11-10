# 08: 鍵盤擴充接上核心：在 iPhone 打出「門口」

Status: done
Blocked by: 04, 06, 07
讀: brief §9、§12.3；plan §2.1、§3.3、§7.3；spec〈共同規則〉；`docs/research/ios-feasibility.md`（票 04、06、07 的節）

## What to build

把 `qim-ffi` 接進票 06 的鍵盤擴充，連同 M1 的資料包放進擴充，在 iPhone 的真實宿主程式打出「門口」，並量常駐記憶體與每鍵延遲。只要能打字讀回，不做佈局與產品 UI：幾個固定鍵（`a`、`n`、`r`、空白）加一行候選就夠。

- 交叉編譯 `qim-ffi` 到 `aarch64-apple-ios`，接進 Xcode 專案；資料包放進擴充的 bundle（或 App Group，看票 06 的結果），選哪些資料包依票 04 的數字。
- 記憶體：載入每份資料包前後、打字中的常駐記憶體，扣掉票 06 空擴充的基線。
- 延遲：在擴充內量每鍵到快照回來的時間 p50 / p95，含選開頭一段的那一鍵；只量核心呼叫，另記一次含畫面更新的大概值。
- 結果寫進 `docs/research/ios-feasibility.md` 的〈鍵盤擴充接上核心〉一節，附機型與 iOS 版本。

## 驗收

- [x] 備忘錄裡打 `anr` 讀回首選「門口」、空白後讀回上屏「門口」；截圖 `.scratch/m2/evidence/08-*.png`。
- [x] 常駐記憶體：每份資料包的增量與總量都有數字。
- [x] 每鍵 p50 / p95 都有數字，和 brief §9 的 50 ms 目標比較。
- [x] README 的重量步驟更新；`cargo xtask check` 綠燈。

## Comments

- 票 07 轉入：`qim-ffi` 的 `Session` 建構子自己載入資料包、各建一份 `Engine`（`crates/qim-ffi/src/lib.rs` 的 `#[uniffi::constructor]`），不符 plan §2.1「`Engine` 不可變，多個 `Session` 共用」。資料包載入後約 96 MB（票 04，Mac），擴充上限推得約 177 MB（票 06）：換輸入框或重載畫面時另建 `Session`、舊的沒放掉，記憶體就翻倍、擴充可能被終止，量到的數字也失真。本票擴充全程只建一個 `Session`，或把 `Engine` 拆成獨立的 uniffi Object 給各 `Session` 共用（改介面就同步 plan §2.1）；量記憶體前確認沒有重複的 `Session`。
- 2026-11-10 票 08 實作：票 07 轉入的那條照第二種做：`Engine` 拆成獨立的 uniffi 物件，各 `Session(engine:)` 共用（plan §2.1 已同步）。資料包先各自解開成 `SchemePack` 等物件再移交給 `Engine`，讓殼能逐份量 footprint；每份只能給一個 `Engine`，`Engine::new` 中途失敗時已移交的那幾份也跟著丟掉，殼要重讀檔。擴充裡另把 `Engine`、`Session` 放在靜態變數，畫面重建不會多建一份。
- 2026-11-10 票 08 實作：`UserModel` 暫時留在各 `Session` 裡，換輸入框就是新的學習資料；學習資料存檔與共用由需要的票拆（plan §2.1）。
- 2026-11-10 票 08 實作：xcframework 用一般的 `module` 放在 `Headers/qimFFI/`，不用 uniffi `--xcframework` 產生的 `framework module`：後者在靜態函式庫的 xcframework 裡 Xcode 找不到（實測，`docs/development.md`〈Swift 綁定〉）。
- 2026-11-10 票 08 結果：iPhone 18 Pro Max（iOS 27.2）上五份資料包加 `Engine` footprint +51.4 MB、載入後 64.4 MB；每鍵 p95 64 µs、選開頭一段 p95 0.79 ms；見 ios-feasibility〈鍵盤擴充接上核心〉。字根提示與佈局沒載入，留給票 09 加壓時算進去。
- 2026-11-10 票 08 code-review 未處理的發現：
  - ios-keyboard README〈上次結果〉的 commit 寫「`232d528` 加上票 08 的改動」，照票 07 的寫法，沒回填本票 commit 的 hash（README 不能寫進含自己的 commit）。
  - 五份資料包的清單在 `CoreProbe.swift`、`core.sh`、`project.yml`、README 各寫一次；實驗程式，接受。
  - `Latency.swift` 的頂層函式（`press`、`read`、`ms`、`percentile`）名字泛，現在也編進 iOS target；實驗程式，接受。
  - `qim-ffi` 的 `Engine::new` 與 `qim-wasm` 各寫一份「`Engine::new` 加 `with_simplifier`、`with_english`」的組裝；加第六種資料包時考慮在核心提供收 `Option` 的建構式（留給 M6 `qim-ffi` 完整版）。

