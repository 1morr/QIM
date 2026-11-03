# 03: commit-msg 閘門

Status: done
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

- [x] 測試：未知 type、缺冒號、description 大寫開頭、結尾句號、subject 73 字元，各自會紅。
- [x] 測試：有 scope、有 `!`、有 body、剛好 72 字元、Merge 與 fixup 訊息、帶 `#` 註解行，都不紅。
- [x] 在暫存 repo 啟用 hook 後，實際 `git commit -m "Add stuff."` 被擋，`git commit -m "chore: add stuff"` 通過（貼輸出；不在本 repo 留測試 commit）。

## Comments

- 2026-11-03 正在 `git merge`（`MERGE_HEAD` 存在）時一律放行，經使用者同意：ticket-coordinator 用 `git merge --no-ff -m "merge: ticket NN slug"`，實測 git merge 會跑 commit-msg hook，`merge` 不在 type 清單會被擋。`git commit -m "merge: …"` 仍擋。
- 2026-11-03 放行清單多了 `amend!`（`--fixup=amend:`）與 `Reapply "`（git 2.43 起 revert 一個 revert 的預設訊息），都是 git 產生的。
- 2026-11-03 「小寫開頭」實作成第一個字不是大寫字母：數字、反引號、中文開頭都放行。另外比票嚴：scope 不可空白、冒號後只接一個空格、全形「。」也算句號；`git commit -v` 的剪刀線之後不算訊息。
- 2026-11-03 development.md 列的 type 由 `development_md_lists_the_coded_types` 比對 `TYPES`，附雙向變異。
- 2026-11-03 驗收輸出（暫存 repo 複製本工作樹、`core.hooksPath .githooks`）：
  - `git commit -m "Add stuff."` → `commit message 不符規則（docs/development.md〈Commit message〉）：` / `  Add stuff.` / `- 缺冒號：subject 要是 \`type(scope)!: description\``，沒有產生 commit。
  - `git commit -m "chore: add stuff"` → `[main aa2d64d] chore: add stuff`。
  - `git commit --allow-empty -m "merge: ticket 03 x"` → `- type \`merge\` 不在清單：…`；`git merge --no-ff -m "merge: ticket 03 x" t` → `Merge made by the 'ort' strategy.`。
  - 結果的 `git log`：`ab8291c merge: ticket 03 x`、`aa2d64d chore: add stuff`、`f022384 feat: add t`、`e6f7e50 chore: import snapshot`。
- 2026-11-03 code-review 沒處理的發現：
  - `git commit -m "#12 fix"` 的 subject 被當成註解，回報「訊息是空的」而不是格式錯；反正會擋。`core.commentChar` 不是 `#` 時不支援。
  - `merge_in_progress` 的兩個測試跑 git 時繼承環境變數；若在 hook 內跑 `cargo test`，git 匯出的 `GIT_DIR` 會讓它們指到外層 repo。pre-commit 的 `--fast` 不跑 test，目前不會發生。
  - 合併放行不看訊息，`git merge -m "Totally Bad."` 也過；這是選定的語義。
