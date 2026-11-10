# 05: GitHub Actions CI

Status: done
Blocked by: None (can start immediately)
讀: plan §6；`docs/development.md`〈安裝〉〈全部檢查〉；spec〈共同規則〉；`.scratch/m1/issues/24-m1-close.md` Comments 的「CI 接上時」那條

## What to build

repo 已公開到 https://github.com/1morr/QIM，接上 GitHub Actions，每次 push 與 PR 跑 `cargo xtask check --fast` 與 `cargo test`。`bench --check` 要下載約 360 MB 的來源，先不進（決定與理由寫進 plan §6）。

- 先查證 Actions 的現行寫法（runner 映像、Rust toolchain 的 action、快取）再寫 workflow；用 `rust-toolchain.toml` 釘的版本。
- 測試要用到 `build/packs/` 而 CI 沒有資料包時：查清楚哪些測試會失敗，選一個不必下載大來源的做法（跳過並註明，或只建小資料包），寫在 `## Comments`。
- plan §6 的「CI：repo 還沒有遠端」改寫成現況；`docs/development.md` 補一句 CI 跑什麼。
- 推到遠端是寫到遠端的動作：push 前先問使用者，說明會推哪些 commit、Actions 會在公開 repo 跑。
- M1 留下的「CI 接上時」各條（`bench --check` 進不進、Node 延遲腳本沒有單元測試、粵語維基 dump、`grid.sh` 欄號）逐條在 `## Comments` 註明這張票處理了沒有、沒處理的去處。

## 驗收

- [x] workflow 檔在 repo；本機用同樣的指令跑過一次綠燈。
- [x] 使用者同意後 push，Actions 綠燈，貼 run 連結；不同意時註明。（使用者已同意；協調者合併到 main 後推送，run 連結由協調者補到 Comments）
- [x] plan §6、`docs/development.md` 已更新。
- [x] `cargo xtask check` 綠燈。

## Comments

- 2026-11-10 沒有資料包時的測試：在沒有 `build/` 的 worktree（`bench/corpus/`、`bench/colloquial/` 有進 git，CI 也有）跑 `cargo test --workspace`，626 個全過、0 個 ignored。用到資料包的測試（`qim-databuild`、`qim-bench`、`xtask` 的 `web`）都在暫存目錄自建小資料包；讀下載快取的 `default_cache_dir` 只有 `qim-databuild` 的 `main.rs` 用。所以 CI 不跳過測試、不建資料包。
- workflow 寫法（2026-11-10 查各 action 的 README 與 release 頁）：
  - `ubuntu-latest`（24.04）、`actions/checkout@v7`。
  - `actions-rust-lang/setup-rust-toolchain@v2` 不給 `toolchain` input，版本、components、targets 都讀 `rust-toolchain.toml`，內建 Swatinem/rust-cache。`dtolnay/rust-toolchain` 不讀這個檔、版本要在 workflow 再寫一次，不用。rustup 1.28 起不自動裝 toolchain，直接用 runner 的 rustup 要先 `rustup toolchain install`。
  - wasm-bindgen 用 `taiki-e/install-action@v2` 裝預編譯的 0.2.130，`fallback: none` 不退回編譯（`cargo install` 要編幾分鐘）。版本寫死在 workflow；升 `wasm-bindgen` 時 `check` 的 `wasm` 步驟會擋下並印出版本差異，不會悄悄漂移，不另加測試。
  - Node 用 `actions/setup-node@v7` 的 `node-version: 26`，npm 快取用 `web/package-lock.json`。
  - setup-rust-toolchain v2 的 `rustflags` 預設是空字串（README），CI 不像 v1 那樣加 `-D warnings`，與本機的編譯旗標相同。
  - actionlint 1.7.12 無警告。
- 觸發收窄：push 只限 `main`，另加 `pull_request`，同 repo 的 PR 分支才不會 push 與 PR 各跑一次。票寫「每次 push」，但 worktree 分支都在本機合進 `main` 才推，效果相同。`concurrency` 取消同一 ref 的舊 run。
- M1「CI 接上時」各條（`.scratch/m1/issues/24-m1-close.md` Comments）：
  - `bench --check` 進不進：處理了。不進 CI（建資料包要下載約 360 MB 來源），也不進 `check`（約 1 分鐘）；照 `docs/development.md`〈首選字基準〉在改排序、資料或核心的票手動跑。寫進 plan §6。
  - Node 延遲腳本 `web/bench/latency.ts` 沒有單元測試：沒處理。型別有 `tsc -p bench`（CI 跑）守，行為只在要資料包的 `bench` 裡驗。去處：票 07 會改延遲量法，由它補或在收尾票 12 決定。
  - 粵語維基 XML dump 停用前換取法：與 CI 無關（CI 不下載來源），沒處理；已在 plan §8 風險表，期限 2027-01。
  - `grid.sh` 靠報表欄號：與 CI 無關（調參實驗，CI 不跑）。處理成在 `scripts/experiments/bigram-tuning/grid.sh` 開頭註明它靠 `report.rs` 哪幾張報表的欄號，報表改欄時看得到。
- 2026-11-10 推送：使用者已同意；協調者合併到 main 後推送，run 連結由協調者補到 Comments。code-review 的發現都已處理（`grid.sh` 註解涵蓋全部欄號、development.md 句子挪位、本條上方兩處事實更正），沒有未處理的。
