# 16: 簡繁

Status: ready-for-agent
Blocked by: 13
讀: plan §1.4（`ferrous-opencc`）、§2.2（後處理）、§4.4、§8（`ferrous-opencc` 那列）；brief §4.1、§5.2；spec〈共同規則〉

## What to build

開簡體模式後，候選與上屏都是簡體：打 `anr` 首選「门口」。

- 開工先驗證 `ferrous-opencc` 能不能編進 `qim-wasm`（WebAssembly），結果寫進 `## Comments`；不能就照 plan §8 退回自讀 OpenCC 字典檔，記偏差。轉換放在哪一層（`qim-core` 後處理或另一個 crate）要守 plan §1.3，加依賴先改表。
- OpenCC `fae2a0a` 的字典釘進 `sources.toml`，簡繁資料包有自己的 `LICENSE`（Apache-2.0）、`NOTICE`。
- `Options.simplified`；轉換在排序之後（plan §2.2 的後處理），轉換後重複的候選去重。學習仍記繁體原文。
- 網頁加簡繁開關；量 WebAssembly 包大小與每鍵延遲的變化。

## 驗收

- [ ] `ferrous-opencc` 能否編 WebAssembly 的結論在 `## Comments`（不能時偏差已記）。
- [ ] 核心測試：簡體模式 `anr` 首選「门口」；轉換後重複候選只留一個；簡體模式下學習記的是繁體。
- [ ] Playwright：開簡體後打 `anr`，讀回首選「门口」，空白上屏「门口」。
- [ ] WebAssembly 包大小與延遲的前後數字在 `## Comments`。

## Comments
