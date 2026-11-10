# 07: `qim-ffi` 最小版：在 Mac 用 Swift 打出「門口」

Status: done
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

- [x] Swift 程式打 `anr` 讀回首選「門口」，空白後上屏「門口」；輸出貼在 `## Comments`。
- [x] 三種路徑的每鍵 p50 / p95 都有數字，含選段那一鍵；WebAssembly 在 Chromium 仍 ≤ 50 ms。
- [x] plan §1.2、§1.3、§1.4 與 `RULES` 已更新，`deps` 綠燈。
- [x] `cargo xtask check` 綠燈。

## Comments

- 票 01 轉入：殼只呼叫 `set_options` 時 `Session` 有「待生效選項」狀態（重列前 `all_candidates` 空、不重排）。設計 FFI 介面時考慮讓 `set_options` 帶 `Engine`、`UserModel` 直接重列並回快照，去掉這個狀態（plan §2.1）。
- 2026-11-10 完成（Mac：Apple M6、macOS 27.0.1，Xcode 27.1、Rust 1.99.1、uniffi 0.32.2）。`scripts/experiments/ffi-swift/run.sh` 的 `demo`：

  ```text
  按 a：輸入碼 a，首選 日
  按 n：輸入碼 an，首選 門
  按 r：輸入碼 anr，首選 門口
  按空白：上屏 門口
  ```

- 每鍵 p50 / p95（µs，同一批 123,330 鍵）：原生 30.7 / 41.8、Swift 42.3 / 58.1、Chromium 的 WebAssembly 40 / 55；選開頭一段原生 222 / 737、Swift 235 / 760、Chromium 275 / 890。全表與結論在 `docs/research/ios-feasibility.md`〈綁定開銷〉。
- 決定：
  - 按鍵入口用 `press(kind, value)`（`Key::parse`），不直接收 `Key`：uniffi 沒有 `char`（官方 Built-in types 表），鏡像列舉會讓加一種鍵多改一處；plan §2.1 已改。
  - 介面只有建構、`press`、`select`（量選段那一鍵要它）；`qim-layout` 沒用到就沒加依賴。§1.3 的表與 `RULES` 本來就有 `qim-ffi`、內容符合，不必改；§1.2、§1.4 已更新。
  - 票 01 轉入的 `set_options`：FFI 這次不匯出換選項，Swift 碰不到待生效狀態；加的時候讓綁定的 `set_options` 同時 `refresh` 回快照，寫進 plan §2.1，留給需要它的票（票 08 起）。
  - 選開頭一段的量法：打完有兩段以上時按分段數次 ←（不計時）到第一段、計 `select(0)`；三條路徑同規則，量到的輸入數一致（6,411）。整串候選裡直接點開頭的候選沒另外量（research 已註明）。
  - `qim-bench` 常設「選開頭一段」一組（原生與 Node），WebAssembly 量的輸入和原生不同時基準失敗：票只要求量，這是票外擴張，補 M1 票 24 的缺口，記在 progress 偏差。
  - `Session` 的 `Mutex` 中毒（前一個呼叫在核心 panic）時照樣用裡面的狀態：鍵盤不該因一次 panic 之後每個鍵都失敗；組字可能停在一半，下一個 Esc 或上屏就清掉。沒有測試（要讓核心 panic 才造得出）。
  - `cargo xtask ffi` 只產生 macOS；iOS target 與 xcframework 的做法寫進 `docs/development.md`〈Swift 綁定〉，由票 08 做。
- code-review 沒處理的發現（判斷題）：
  - 「選段規則」與依魔數分辨資料包在 `latency.rs`、`latency.ts`、`compare.mjs`、`main.swift`、測試的假計時器各一份；`run.sh` 用 awk 重寫 qim-bench 取輸入的順序。實驗腳本是一次性的，原生與 Node 有交叉檢查、Swift 與 Chromium 靠輸入數與基準相同核對；不收。
  - `Timing` 的 `per_key_ms` 與 `select_ms` 是兩條靠順序對齊的平行陣列，可合成一個每輸入的型別；`measure` 已驗形狀，改動不值得，不收。
  - `xtask/src/ffi.rs` 沒有測試：它只轉呼叫 `cargo build` 與 uniffi 的 CLI，實跑由 `run.sh` 驗；同 `xtask/src/web.rs` 的 `wasm_bindings`。
- 已知差距（協調者轉入）：`Session` 建構子自己建 `Engine`，多個 `Session` 不共用，與 plan §2.1 不符；最小版不改介面，轉給票 08 的 Comments（擴充只建一個 `Session`，或把 `Engine` 拆成獨立 Object 共用）。
