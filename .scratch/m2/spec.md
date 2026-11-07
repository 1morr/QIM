# M2 平台可行性實驗

由 plan §7.3 細化（2026-11-07）。拆票後以票為準；票與本檔衝突時改票或改本檔，並在 progress〈偏差與決定〉記一行。

## 目標

在平台殼開工前，先回答核心與資料包放不放得進 iOS 鍵盤擴充、端上模型與上下文能用到什麼程度。只量數字，不做產品。量到的數字決定 M3 模型階梯的大小上限（plan §7.8）與資料包格式要不要改（plan §8 的 mmap 那列）。

四個問題：

1. iOS 鍵盤擴充的記憶體：核心加 M1 的資料包常駐多少、系統在多少時終止擴充。
2. Foundation Models 能否在鍵盤擴充內使用。
3. `documentContextBeforeInput` 實際讀得到多少字。
4. 小型神經模型（隨機權重，Gboard 級約 1.4M 參數、青簡級約 23M 參數）在鍵盤擴充裡的載入時間、單次推論延遲、記憶體。

## 範圍（做 / 不做）

### 做

- FFI 前的清理（M1 review 決定，`.scratch/m1/issues/24-m1-close.md` Comments 的「M2」各條）：
  - `set_options` 後 `select`、`focus_segment`、`text` 用舊候選列的 bug，先寫會紅的測試再修。
  - 統一命名：`Pack` 改 `SchemePack`；分段只留一種型別；`UserModel` 的方法不再每個都帶 `scheme`。
  - 綁定層不放邏輯：`qim-wasm` 的 `perform`（動作轉 `Key`、切頁）、`WebLayout` 的版本與目前頁、`WebSession.composing` 收進 `Session` 與 `qim-layout`；按鍵收成一個入口；`JsError` 轉換收成一處。
- CI：GitHub Actions 跑 `cargo xtask check --fast` 與 `cargo test`；`bench --check` 要下載大來源，不進。
- Mac 上量各資料包載入後的常駐記憶體，有無 `jsonschema` 各一次。
- iOS 鍵盤擴充的 tracer：先不接核心，查證並實測免費佈建能否在使用者的 iPhone 上跑鍵盤擴充、App Group 能否使用（brief §11）。
- `crates/qim-ffi/` 最小版（uniffi）：只包到能載入資料包、處理按鍵；量綁定的開銷。
- 鍵盤擴充接上核心：在 iPhone 的真實宿主程式打字讀回；量常駐記憶體、每鍵延遲；加壓找終止上限。
- 同一個擴充裡量 Foundation Models 與 `documentContextBeforeInput`。
- 小型神經模型開銷：推論引擎先試青簡用的 candle。
- M1 留給 M2 的延遲缺口：瀏覽器（Chromium）的 WebAssembly 每鍵延遲、選開頭一段時從頭重建 lattice 的那一鍵。

### 不做

- 任何產品化的平台殼、主 App、設定頁、安裝流程：M6 起。實驗的 Xcode 專案放 `scripts/experiments/`，不進 `platforms/`。
- 資料包改 mmap 友善格式的實作：M2 只做決定、寫回 plan；要改時另開票或留給 M3。
- 訓練神經模型、量它們的正確率：M3 模型階梯（plan §7.8）。
- Android、macOS、Windows、Linux 的實機量測：各自的平台里程碑。
- `qim-layout` 的 `jsonschema` 做成 feature 關掉：M2 只量、只決定。

## 共同規則

- 沿用 M1 的：依賴用 `cargo add` / `npm install` 取當時 stable，與 plan §1.4 不同時同一 commit 更新該表，版本相關 API 先查 context7 或官方文件；命名照 plan §0 與 `GLOSSARY.md`；建置產物不進 git；新增或改動開發指令的票同輪更新 `docs/development.md`。
- 清理票不改行為：`cargo xtask bench --check` 綠燈且數字不變、不 `--update-baseline`；Playwright e2e 全過。修 bug 的票例外，前後對照寫在票的 `## Comments`。
- 實驗程式碼放 `scripts/experiments/<名>/`，各有 README：日期、裝置與系統版本、Xcode 與 Rust 版本、commit、量法、結果表。換版本時要能照 README 重量。實驗的 Rust crate 不加進 workspace，免得動到 plan §1.3 的依賴表；`qim-ffi` 例外，它是產品的一部分。
- 量到的數字先寫進 `docs/research/ios-feasibility.md`（各實驗一節，附原始輸出的摘要），結論摘進 brief〈查證過的事實〉§12.3 並連結；收尾票再統一寫 brief §9、§11。
- 實機只有使用者的 iPhone 18 Pro Max：每個數字註明機型與 iOS 版本，寫回 brief 時註明旗艦機的記憶體上限可能比低階機高。
- 平台框架的事實（佈建、App Group、記憶體指標、Foundation Models 的可用條件）只信 brief〈查證過的事實〉與現查的官方文件；查到的補進 brief 附來源。
- iPhone 上的輸入行為以實際打字並讀回畫面為準（`CLAUDE.md`〈驗收輸入行為〉）：真機用 iphone-use 操作備忘錄這類宿主程式；截圖存 `.scratch/m2/evidence/`。
- 要使用者動手或決定的事（Xcode 登入 Apple ID、在 iPhone 上信任開發者與加入鍵盤、打開 Apple Intelligence、免費佈建不能用時要不要申請付費帳號），到了就在該票停下問，不阻塞其他票。
- 推到遠端（CI 那張票）前先問使用者。

## 驗收

- 四個問題都有實測數字或明確的「不能」，寫進 `docs/research/ios-feasibility.md`，摘進 brief §9（記憶體目標從暫定改成量到的值）、§11（免費佈建那條結案）、§12.3（附來源），註明機型與 iOS 版本。
- 鍵盤擴充在 iPhone 的真實宿主程式打 `anr`，讀回首選「門口」、上屏「門口」；截圖。
- `qim-ffi` 存在，plan §1.2、§1.3 的表與 `xtask` 的 `RULES` 同步，`deps` 綠燈。
- 資料包要不要改 mmap 友善格式、鍵盤擴充要不要略過 Schema 驗證，兩個決定寫回 plan（§1.3、§1.4、§4、§8），M3 模型大小的上限寫進 plan §7.8。
- 清理票之後：`qim-wasm` 只做型別轉換，`set_options` 那條 bug 有測試守住；`bench --check` 數字與 M1 結束時相同。
- GitHub Actions 在遠端綠燈（使用者同意推送後）；不同意時註明，靠 pre-commit 把關。
- `cargo xtask check` 全套綠燈；progress.md 依〈寫法與封存〉封存 M2、M3 標為進行中之前，以上全部成立。

## 票序

| 票 | 被誰擋 | 交付 |
| --- | --- | --- |
| [01](issues/01-set-options-stale-candidates.md) | — | `set_options` 後 `select`、`focus_segment`、`text` 用新的候選列，先紅再修 |
| [02](issues/02-unify-naming.md) | 01 | `SchemePack`、一種分段型別、`UserModel` 不逐一帶方案 |
| [03](issues/03-bindings-without-logic.md) | 02 | `qim-wasm` 只做型別轉換；按鍵一個入口；`JsError` 收一處 |
| [04](issues/04-mac-pack-memory.md) | — | Mac 上各資料包的常駐記憶體，有無 `jsonschema` |
| [05](issues/05-github-actions-ci.md) | — | GitHub Actions 跑 `check --fast` 與 `cargo test`（推送前問使用者） |
| [06](issues/06-ios-keyboard-tracer.md) | — | 空鍵盤擴充用免費佈建裝上 iPhone、備忘錄打出「日」；App Group 實測 |
| [07](issues/07-qim-ffi-minimal.md) | 03 | `qim-ffi` + Swift 在 Mac 打出「門口」；原生、FFI、Chromium WebAssembly 的每鍵延遲 |
| [08](issues/08-ios-keyboard-with-core.md) | 04, 06, 07 | 擴充接上核心，iPhone 備忘錄打出「門口」；常駐記憶體與每鍵延遲 |
| [09](issues/09-ios-memory-limit.md) | 08 | 加壓找終止上限；資料包格式、Schema 驗證、M3 模型上限的決定 |
| [10](issues/10-foundation-models-and-context.md) | 06 | 擴充內的 Foundation Models；`documentContextBeforeInput` 的實際字數 |
| [11](issues/11-small-neural-model-cost.md) | 06 | candle 跑 1.4M、23M 參數隨機模型的載入、推論延遲、記憶體 |
| [12](issues/12-m2-close.md) | 01–11 | 里程碑 code-review、乾淨 checkout 重驗、結論寫回 brief 與 plan、封存 |
