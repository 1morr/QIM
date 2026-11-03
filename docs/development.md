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
git config core.hooksPath .githooks    # 啟用 pre-commit 與 commit-msg hook
cargo xtask check                      # 確認環境就緒
```

macOS 上 `npm ci` 會提示 `fsevents` 的 install script 未核准：它附預編譯檔，不必核准。

## 建置

| 指令 | 作用 |
| --- | --- |
| `cargo build` | 建置全部 Rust crate |
| `npm run --prefix web dev` | 網頁開發伺服器 |
| `npm run --prefix web build` | 型別檢查後打包網頁到 `web/dist/` |
| `cargo xtask data [--verify]` | 建資料包到 `build/packs/`，見〈資料建置〉 |

## 資料建置

```sh
cargo xtask data            # 建資料包
cargo xtask data --verify   # 建資料包並跑 plan §4.5 的檢查，有一項失敗就 exit 非零
```

`cargo xtask data` 用 release 編 `qim-databuild` 並在 repo 根目錄執行，步驟見 plan §4：

- 來源列在 `data/sources.toml`：每個來源釘 commit 與每個檔的 sha256。下載到 `.cache/sources/<來源>/<sha256 前 16 碼>/<檔案路徑>`；已有快取就不連網。快取或下載的檔 sha256 不符時建置失敗、不重新下載：確認來源後刪掉那個檔重跑，或更新 `sources.toml`。換來源版本時改 commit、url 與 sha256，快取路徑跟著換。
- `sources.toml` 的 `license_statement` 照抄上游的授權聲明，建置時核對它確實在 `license_file` 裡（空白與換行不計），再寫進資料包的 `LICENSE`。
- 方案檔在 `data/schemes/`，由 `qim-databuild` 編進資料包；`qim-core` 不讀 TOML。
- 產物：`build/packs/<方案 id>/` 下的 `<方案 id>.pack`、`LICENSE`、`NOTICE`。資料包格式定義在 `crates/qim-core/src/pack.rs`，改格式時加 `FORMAT_VERSION`。

`--verify` 的報告列出連打字集字數、擴展區字數、速成碼個數、資料包 sha256，再逐項列 `[通過]` 或 `[失敗]`，最後一行 `verify 通過` 或 `verify 失敗`。檢查定義在 `crates/qim-databuild/src/verify.rs`。

`.cache/` 與 `build/` 不進 git。

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
| `deps` | 依賴規則（plan §1.3）：`cargo metadata` 比對白名單，並掃 `qim-core`、`qim-layout` 的禁用 I/O | 跑 |
| `docs` | 文件規則（plan §6）：`docs/progress.md` 的段落、表格、長度、里程碑標記、下一步指令，與 `.scratch/` 的 spec | 跑 |
| `tsc` | `web/` 的 `tsc --noEmit`（`npm run typecheck`） | 跑 |
| `biome` | `web/` 的 `biome check`（`npm run lint`） | 跑 |

`cargo xtask check --fast` 是 pre-commit hook 跑的快速版，只跳過 `test`：測試會隨碼表與性質測試變慢，其他步驟只做靜態檢查，增量執行幾秒內完成。`clippy --all-targets` 仍會編譯測試碼，所以測試編不過也會被擋。

`deps` 失敗時每條違規印一行，說明是哪個 crate、違反 plan §1.3 的哪一條；禁用路徑附 `檔案:行號`。

`docs` 失敗時每條違規印一行 `檔案:行號: 說明`，規則在 `xtask/src/docs.rs`。最常見的是平行的票合併後 `docs/progress.md` 被 union 合併抄出第二個〈Session 紀錄〉段：把第二段新的列移回第一段的表尾、刪掉重複段。

格式自動修正：`cargo fmt --all`、`npm run --prefix web lint -- --write`。

步驟定義在 `xtask/src/check.rs`；加步驟時同輪更新上表，`cargo test` 會比對兩邊的步驟名與 `--fast` 欄。

## Commit message

commit-msg hook 跑 `cargo xtask commit-msg <訊息檔>`，subject（第一個非空白、非 `#` 註解的行）不符下列規則就擋下並逐條說明：

- 格式 `type(scope)!: description`，`(scope)` 與 `!` 選填，冒號後接一個空格。
- type 只能是 `xtask/src/commit_msg.rs` 的 `TYPES`：feat、fix、docs、refactor、test、chore、build、ci、perf、style、revert。
- description 不以大寫字母開頭，結尾不加句號；subject 整行 ≤ 72 字元。

不檢查的情況：

- git 自動產生的 `Merge …`、`Revert "…"`、`Reapply "…"`、`fixup!`、`squash!`、`amend!` 開頭的訊息。
- 正在 `git merge` 時（`MERGE_HEAD` 存在）一律放行：ticket-coordinator 用 `merge: ticket NN slug` 合併票。
- body 與 `git commit -v` 剪刀線之後的 diff。
