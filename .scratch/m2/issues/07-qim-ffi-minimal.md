# 07: `qim-ffi` 最小版：在 Mac 用 Swift 打出「門口」

Status: ready-for-agent
Blocked by: 03
讀: plan §1.1–§1.4、§2.1、§5.2（每鍵延遲）；ADR 0002；spec〈共同規則〉；`.scratch/m1/issues/24-m1-close.md` Comments 的「M2」各條（延遲那條）；`docs/development.md`〈首選字基準〉

## What to build

建 `crates/qim-ffi`，用 uniffi 產生 Swift 綁定，只包到能載入資料包、處理按鍵、讀快照的介面（票 03 的單一入口）。在 Mac 上用一個 Swift 程式打 `anr` 讀回首選「門口」，並量真實綁定的開銷。

- 開工先查 uniffi 現行版本的文件（context7）：Swift 綁定的產生方式、proc-macro 還是 UDL、產生 xcframework 的做法；plan §1.4 的版本照 `cargo add` 更新。
- plan §1.2、§1.3 的表與 `xtask` 的 `RULES` 同步，`deps` 對 `qim-ffi` 綠燈；介面只轉型別，不放邏輯。
- 產生 Swift 綁定與 macOS（之後加 iOS 的 `aarch64-apple-ios`）可用的產物；指令寫進 `docs/development.md`，產物不進 git。
- 延遲：同一批輸入在 Mac 上量原生（`qim-bench` 已有）、經 Swift 綁定、Chromium 裡的 WebAssembly（`scripts/experiments/wasm-latency-browser/` 有的話沿用）三者的每鍵 p50 / p95，含 20 碼以上的長輸入與「選開頭一段後重建 lattice」那一鍵。
- 結果寫進 `docs/research/ios-feasibility.md` 的〈綁定開銷〉一節。

## 驗收

- [ ] Swift 程式打 `anr` 讀回首選「門口」，空白後上屏「門口」；輸出貼在 `## Comments`。
- [ ] 三種路徑的每鍵 p50 / p95 都有數字，含選段那一鍵；WebAssembly 在 Chromium 仍 ≤ 50 ms。
- [ ] plan §1.2、§1.3、§1.4 與 `RULES` 已更新，`deps` 綠燈。
- [ ] `cargo xtask check` 綠燈。

## Comments

- 票 01 轉入：殼只呼叫 `set_options` 時 `Session` 有「待生效選項」狀態（重列前 `all_candidates` 空、不重排）。設計 FFI 介面時考慮讓 `set_options` 帶 `Engine`、`UserModel` 直接重列並回快照，去掉這個狀態（plan §2.1）。
