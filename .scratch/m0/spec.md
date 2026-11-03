# M0 骨架

由 plan §7.1 細化（2026-11-03）。拆票後以票為準；票與本檔衝突時改票或改本檔，並在 progress〈偏差與決定〉記一行。

## 目標

repo 骨架與閘門就位；資料管線產出速成碼表資料包；在網頁按 `a` 看到「日」，按空白上屏（tracer bullet）。

## 範圍

### 做

- 骨架：Cargo workspace（edition 2024）、`rust-toolchain.toml`、`xtask`、`web/` 的 Vite + TypeScript + Biome 空殼、`docs/development.md`、`CLAUDE.md` 的頂層目錄地圖。crate 隨用到它的票建立，不預先建空殼（M0 只會有 `qim-core`、`qim-databuild`、`qim-wasm`、`xtask`）。
- 閘門（plan §6）：`cargo xtask check` 各步驟、`deps`（plan §1.3）、`.githooks/` 的 pre-commit 與 commit-msg、progress 規則、里程碑 spec 規則；每個閘門都附雙向變異測試。
- 資料管線 v0（plan §4、§4.1 步驟 1–4、§4.5）：
  - `data/sources.toml` 釘 libcangjie `346026b` 與 Unihan（版本與 sha256 在票內釘）。
  - `qim-databuild` 下載到 `.cache/sources/` 並核對 sha256。
  - 推導速成碼表：三代與五代的聯集，連打字集與擴展區分開標記，字序依 libcangjie 的 Big5 排序值（還沒有字頻）。
  - `data/schemes/quick.toml` 由 `qim-databuild` 編進資料包，`qim-core` 不讀 TOML（plan §2.5）。
  - 資料包用 `postcard`，帶格式版本號，旁邊放 `LICENSE` 與 `NOTICE`。
  - `cargo xtask data` 與 `--verify` 跑 §4.5 全部檢查；Unihan 交叉比對只出報表、不擋建置。
- 核心 tracer bullet：
  - `Engine` 從資料包的位元組建立，由呼叫端讀檔後傳入（`qim-core` 不碰 `std::fs`，plan §1.3）。
  - `Session` 處理 `Letter`、`Space`、`Backspace`、`Escape`、`Digit`、`PageUp` / `PageDown`。
  - 只做單字查詢：整串輸入碼查碼表，沒有切分。
  - `qim-wasm` 綁定；網頁有輸入框、組字顯示（輸入碼）、候選列；Playwright 實打驗證。

### 不做

- 連打、lattice、bigram、詞庫、字頻：M1。
- 學習與使用者詞：`UserModel` 只留 plan §2.1 的型別讓簽章成立，內容 M1 做。
- 英文候選、簡繁、`Options` 的實際作用、組字中顯示字根、字根提示資料包（§4.1 步驟 5）：M1。
- 佈局與 `qim-layout`、觸控模式、`qim-bench`、首選字基準：M1。
- CI 與遠端：建遠端要先問使用者（plan §6）。

## 共同規則

- 依賴版本：開工時用 `cargo add` / `npm install` 取當時 stable；與 plan §1.4 不同時，同一 commit 更新該表。版本相關的 API 先查 context7 或官方文件，不憑記憶寫。
- 命名照 plan §0 的對照表與 `GLOSSARY.md`。
- 建置產物與下載的來源不進 git：`target/`、`.cache/sources/`、`build/packs/`、`web/public/packs/`（plan §1.2）；第一張用到的票加進 `.gitignore`。
- 新增或改動開發指令的票，同輪更新 `docs/development.md`；之後每張票的驗證就是它的〈全部檢查〉加 test。
- 閘門測試做雙向變異：在暫存副本造出違規會紅，改無關的格式或命名不紅；只斷言原始碼含某字串的測試不算。
- 候選分頁：每頁 9 個，對應數字鍵 1–9；`-` / `=` 由網頁轉成 `PageUp` / `PageDown`（brief §4.3）。
- 擴展區的字排在連打字集的字之後（plan §2.3、brief §4.1）。

## 驗收

- `cargo xtask check` 綠燈；pre-commit 與 commit-msg hook 依 `docs/development.md` 啟用後，違規的 commit message 被擋。
- `deps`、progress、spec 閘門各自在變異副本上會紅、在無關改動上不紅。
- `cargo xtask data --verify` 綠燈；報告列出連打字集字數、擴展區字數、Unihan 差異數；同樣的來源建兩次，資料包 sha256 相同。
- Playwright 在網頁打 `a` 讀回候選含「日」，按空白讀回輸入框為首選「曰」（Big5 序排在「日」前），按 `a` `2` 讀回「日」；打 `ar` 讀回候選含「問」（必要時翻頁）；截圖存證。
- progress.md 依〈寫法與封存〉封存 M0、M1 標為進行中之前，以上全部成立。

## 票序

| 票 | 被誰擋 | 交付 |
| --- | --- | --- |
| [01](issues/01-skeleton-and-check.md) | — | workspace、`cargo xtask check`、`web/` 空殼、pre-commit、`docs/development.md` |
| [02](issues/02-deps-gate.md) | 01 | `deps` 閘門（plan §1.3） |
| [03](issues/03-commit-msg-gate.md) | 01 | commit-msg 閘門 |
| [04](issues/04-docs-gates.md) | 01 | progress 與 spec 文件閘門 |
| [05](issues/05-quick-code-table-pack.md) | 01 | 速成碼表資料包與 `--verify` |
| [06](issues/06-unihan-cross-check.md) | 05 | Unihan 交叉比對報表 |
| [07](issues/07-tracer-bullet-type-a.md) | 05 | 網頁打 `a` 列出「日」、空白上屏首選 |
| [08](issues/08-m0-keys-and-paging.md) | 07 | Backspace、Esc、數字選字、翻頁；`ar` → 問 |
| [08b](issues/08b-m0-order-and-space.md) | 08 | 「日」當 `a` 的首選（新字序規則）、空白一律上屏首選 |
| [09](issues/09-m0-close.md) | 08b | M0 收尾：里程碑 code-review、重驗驗收、封存 M0、M1 標進行中 |

02–05 只被 01 擋，可並行；一個一個做時照編號。
