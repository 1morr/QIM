# 01: 骨架與 `cargo xtask check`

Status: ready-for-agent
Blocked by: None (can start immediately)
讀: plan §0、§1.2、§1.4、§6；spec〈範圍〉〈共同規則〉

## What to build

開發者 clone repo 後，照 `docs/development.md` 裝好環境、啟用 hook，跑一個指令 `cargo xtask check` 就能知道 Rust 與網頁兩邊是否都乾淨；commit 時 pre-commit 自動跑快速版。

- Cargo workspace（edition 2024）、`rust-toolchain.toml` 釘開工時的 stable。
- 成員只有空的 `qim-core` 函式庫與 `xtask`；其他 crate 由用到它的票建立。
- `cargo xtask check` 依序跑 `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`、網頁的 `tsc --noEmit` 與 `biome check`，任何一步失敗就整體失敗並指出是哪一步。步驟要容易加：票 02 加 `deps`、票 04 加文件閘門。
- `--fast` 跳過哪些步驟由本票決定，寫進 `docs/development.md` 並在 `## Comments` 記理由。
- `web/`：Vite + TypeScript + Biome 的空殼，`npm` 管理。
- `.githooks/pre-commit` 跑 `cargo xtask check --fast`；啟用方式 `git config core.hooksPath .githooks` 寫進 `docs/development.md`。
- `docs/development.md`：環境需求、安裝、建置、〈全部檢查〉。〈全部檢查〉是之後每張票驗證時照跑的清單。
- `CLAUDE.md` 加頂層目錄地圖（≤ 10 列、只列一級子目錄）。
- `.gitignore` 加 `target/`、`web/node_modules/` 等本票產生的產物。

## 驗收

- [ ] 乾淨的 checkout 照 `docs/development.md` 操作後，`cargo xtask check` 綠燈（貼輸出）。
- [ ] 分別造出 fmt 錯、clippy 警告、TS 型別錯、Biome 錯，各自讓 `cargo xtask check` 紅並指出是哪一步（手動驗一次，貼輸出後還原）。
- [ ] 啟用 hook 後，`check --fast` 失敗時 `git commit` 被擋。
- [ ] 依賴版本與 plan §1.4 不同的，已同一 commit 更新該表。
- [ ] `CLAUDE.md` 有頂層目錄地圖。
