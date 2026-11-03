# 03: commit-msg 閘門

Status: ready-for-agent
Blocked by: 01
讀: plan §6；spec〈共同規則〉

## What to build

開發者寫了不符 Conventional Commits 的 commit message，`git commit` 被 hook 擋下，並說明哪裡不對。

- `cargo xtask commit-msg <訊息檔>` 檢查 subject：
  - 格式 `type(scope)!: description`，scope 與 `!` 選填。
  - type 限 `feat`、`fix`、`docs`、`refactor`、`test`、`chore`、`build`、`ci`、`perf`、`style`、`revert`。
  - description 小寫開頭、結尾沒有句號，整行 ≤ 72 字元。
- git 自動產生的 `Merge …`、`Revert "…"`、`fixup!` / `squash!` 開頭的訊息放行；`#` 開頭的註解行不算訊息內容。
- `.githooks/commit-msg` 呼叫它；`docs/development.md` 補上說明。

## 驗收

- [ ] 測試：未知 type、缺冒號、description 大寫開頭、結尾句號、subject 73 字元，各自會紅。
- [ ] 測試：有 scope、有 `!`、有 body、剛好 72 字元、Merge 與 fixup 訊息、帶 `#` 註解行，都不紅。
- [ ] 在暫存 repo 啟用 hook 後，實際 `git commit -m "Add stuff."` 被擋，`git commit -m "chore: add stuff"` 通過（貼輸出；不在本 repo 留測試 commit）。
