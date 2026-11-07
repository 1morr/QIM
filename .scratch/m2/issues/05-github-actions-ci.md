# 05: GitHub Actions CI

Status: ready-for-agent
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

- [ ] workflow 檔在 repo；本機用同樣的指令跑過一次綠燈。
- [ ] 使用者同意後 push，Actions 綠燈，貼 run 連結；不同意時註明。
- [ ] plan §6、`docs/development.md` 已更新。
- [ ] `cargo xtask check` 綠燈。
