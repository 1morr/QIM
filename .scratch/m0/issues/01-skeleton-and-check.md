# 01: 骨架與 `cargo xtask check`

Status: done
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

- [x] 乾淨的 checkout 照 `docs/development.md` 操作後，`cargo xtask check` 綠燈（貼輸出）。
- [x] 分別造出 fmt 錯、clippy 警告、TS 型別錯、Biome 錯，各自讓 `cargo xtask check` 紅並指出是哪一步（手動驗一次，貼輸出後還原）。
- [x] 啟用 hook 後，`check --fast` 失敗時 `git commit` 被擋。
- [x] 依賴版本與 plan §1.4 不同的，已同一 commit 更新該表。
- [x] `CLAUDE.md` 有頂層目錄地圖。

## Comments

- 2026-11-03 `--fast` 只跳過 `test`：`test` 會隨碼表測試與 proptest 變慢；其餘步驟是靜態檢查，增量執行幾秒內完成。`clippy --all-targets` 仍編譯測試碼，測試編不過照樣擋 commit。之後的 `deps` 與文件閘門也是靜態檢查，應放進 `--fast`。
- 2026-11-03 驗收輸出：乾淨 clone 照 `docs/development.md` 做完 `cargo xtask check` 印 `check 通過`、exit 0；fmt、clippy、TS 型別、Biome 錯各自印 `check 失敗：fmt` / `clippy` / `tsc` / `biome`、exit 1；hook 啟用後造 fmt 錯，`git commit` 印 `check 失敗：fmt`、exit 1、沒有產生 commit。
- 2026-11-03 依賴版本（Rust 1.99.1、Vite 8.3.5、TypeScript 7.0.2、Biome 2.6.4）與 plan §1.4 相同，表不用改。Biome 釘精確版本，這是官方建議，因為格式化規則會隨版本變；`biome.json` 設 `vcs.root: ".."` 讀根目錄的 `.gitignore`。
- 2026-11-03 code-review 沒處理的發現：
  - `check::exec` 的 `node_modules` 缺失分支、Windows `npm.cmd`、`main` 的參數解析（錯參數 exit 2）沒有自動測試，只手動跑過正常路徑。
  - spec 寫「每個閘門都附雙向變異測試」，外部工具步驟（fmt、clippy、tsc、biome）照本票只手動驗過一次；自動的雙向變異測試留給票 02–04 自己寫的閘門。
  - pre-commit 檢查的是工作目錄，不是暫存區：沒有 stage 的修正也會讓 hook 通過。之後要改的話，可以在 hook 裡用 `git stash push --keep-index`，但它和其他 worktree 共用 stash 堆疊，所以這次沒做。
