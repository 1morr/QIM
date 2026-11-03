# 開發

開發指令的單一來源。新增或改動開發指令的票，同輪更新本檔。

## 環境需求

| 工具 | 版本 | 說明 |
| --- | --- | --- |
| rustup | 任一近期版本 | Rust 版本由 `rust-toolchain.toml` 釘住，第一次在 repo 內跑 `cargo` 時 rustup 自動安裝，含 `rustfmt`、`clippy` |
| Node.js | 26 LTS（開工時 26.12.0） | 附帶的 `npm` 管理 `web/` 的依賴 |
| git | 任一近期版本 | hook 用 `core.hooksPath`，Windows 上由 Git for Windows 的 `sh` 執行 |

## 安裝

在 repo 根目錄：

```sh
npm ci --prefix web                    # 依 web/package-lock.json 裝網頁依賴
git config core.hooksPath .githooks    # 啟用 pre-commit hook
cargo xtask check                      # 確認環境就緒
```

## 建置

| 指令 | 作用 |
| --- | --- |
| `cargo build` | 建置全部 Rust crate |
| `npm run --prefix web dev` | 網頁開發伺服器 |
| `npm run --prefix web build` | 型別檢查後打包網頁到 `web/dist/` |

## 全部檢查

每張票收尾時照跑，並貼輸出：

```sh
cargo xtask check
```

依序跑下列步驟，第一個失敗就停，最後一行印 `check 失敗：<步驟>`，exit code 非零；全部通過印 `check 通過`。

| 步驟 | 指令 | `--fast` |
| --- | --- | --- |
| `fmt` | `cargo fmt --all --check` | 跑 |
| `clippy` | `cargo clippy --workspace --all-targets -- -D warnings` | 跑 |
| `test` | `cargo test --workspace` | 跳過 |
| `tsc` | `web/` 的 `tsc --noEmit`（`npm run typecheck`） | 跑 |
| `biome` | `web/` 的 `biome check`（`npm run lint`） | 跑 |

`cargo xtask check --fast` 是 pre-commit hook 跑的快速版，只跳過 `test`：測試會隨碼表與性質測試變慢，其他步驟只做靜態檢查，增量執行幾秒內完成。`clippy --all-targets` 仍會編譯測試碼，所以測試編不過也會被擋。

格式自動修正：`cargo fmt --all`、`npm run --prefix web lint -- --write`。

步驟定義在 `xtask/src/check.rs`；加步驟時同輪更新上表。
