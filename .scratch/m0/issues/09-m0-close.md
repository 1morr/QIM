# 09: M0 收尾

Status: done
Blocked by: 08b
讀: spec〈驗收〉；`docs/progress.md`〈寫法與封存〉；plan §6、§7；`docs/development.md`〈全部檢查〉；票 01–08b 的 `## Comments`

## What to build

M0 的驗收全部重驗一次並留下證據；M0 的 session 紀錄與偏差封存到 `docs/archive/M0.md`，M1 標成進行中，下一個 session 可以直接 `/to-tickets docs/plan.md 拆 M1 的票`。

- **里程碑 code-review**：`/code-review a2a4d5b`（M0 開工時的 commit），Spec 軸讀 spec，Standards 軸讀 `CLAUDE.md`。小問題修掉；不修的與票 01–08b Comments 裡沒處理的發現，整理成一份清單寫在本票 `## Comments`，每條註明留到哪個里程碑或不修的理由。
- **重驗 spec〈驗收〉**，在乾淨的 checkout（`git worktree add --detach` 到暫存目錄，照 `docs/development.md` 從零安裝）跑：
  - `cargo xtask check` 全套（含 `e2e`）。
  - 暫存 repo 啟用 hook，違規的 commit message 被擋、合規的通過（不在本 repo 留測試 commit）。
  - `deps`、`docs` 閘門的雙向變異測試在 `cargo test` 裡（點名測試名稱即可）。
  - `cargo xtask data --verify`：報告的連打字集字數、擴展區字數、Unihan 差異數；建兩次 sha256 相同。
  - Playwright：照 spec〈驗收〉（08b 改過後是 `a` 空白上屏「日」、`ar` 翻頁找到「問」）；截圖路徑。
- **封存**（〈寫法與封存〉，同一個 commit）：
  - M0 的 session 紀錄與偏差整段移到 `docs/archive/M0.md`。
  - 里程碑表 M0 的備註換成 `docs/archive/M0.md` 的連結；M1 的備註改成進行中、尚未拆票。
  - plan §7 表 M0 狀態改完成、M1 改進行中；§7.1 改成指向 spec 與封存檔。
  - progress 這個 session 的紀錄標 `M1`（例如 `M1 封存 M0`），「下一步」寫「新 session 下 `/to-tickets docs/plan.md 拆 M1 的票`」。
  - spec 與票留在原處不搬。

## 驗收

- [x] 票 08b 已 done；spec〈驗收〉的 Playwright 項照 08b 改過的版本驗。
- [x] 里程碑 code-review 跑完；沒處理的發現清單（含票 01–08b 留下的）寫在 `## Comments`，每條有去處或理由。
- [x] 乾淨 checkout 上 `cargo xtask check` 全套綠燈（貼輸出最後幾行）。
- [x] 暫存 repo 的 hook 實測：違規 message 被擋、合規通過（貼輸出）。
- [x] `cargo xtask data --verify` 綠燈，貼出三個字數與兩次建置的 sha256。
- [x] spec〈驗收〉的 Playwright 項讀回成立，截圖路徑寫在 `## Comments`。
- [x] `docs/archive/M0.md` 有 M0 全部的 session 紀錄與偏差；progress 只留 M1；里程碑表與 plan §7 已更新；同一個 commit。
- [x] 封存後 `cargo xtask check` 的 `docs` 步驟綠燈。

## Comments

- 2026-11-03 使用者決定（開票後回覆）：`a` 首選要「日」、空白一律上屏第一頁首選，兩項由 08b 做；暫時不用 `/wayfinder`，文件閘門不放寬（票 04 Comments 那條結案）。
- 2026-11-04 里程碑 code-review（`a2a4d5b...HEAD`，Spec 軸讀 spec 與票 01–08b、Standards 軸讀 `CLAUDE.md`）：Spec 軸沒有高或中的發現；Standards 軸一條硬違規（plan §1.2 說 `data/` 不放第三方原始檔，與 08b 的 `data/licenses/` 矛盾）。已處理（`c5603ad`）：
  - plan §1.2 的 `data/` 列補 `licenses/`。
  - `NOTICE` 的字序說明補「連打字集在擴展區前」。
  - `web/index.html` 的 `<link>` 與 `main.ts` 的 `import` 重複載入樣式表，拿掉 `<link>`。
  - xtask 找 cargo 的寫法三處不一致（兩處讀 `CARGO`、一處 `env!`），收成 `main.rs` 的 `cargo()`。
  - `check`、`deps` 測試各自切 Markdown 表格欄、不處理 `\|`，改用 `docs::cells`。
  - `unihan::parse` 改名 `cangjie_codes`，與旁邊的 `grade_levels` 對稱。
- 2026-11-04 沒處理的發現（含票 01–08b 留下的），每條註明去處：
  - M1 票 1（詞庫與字頻資料包）：`PACKED` 是四個 `&str` 的元組；`license_file` / `license_statement` / `license_text` 三個 `Option` 靠建置時擋組合；`verify.rs` 解碼失敗時另組一份 `Report`；年級表的 `BTreeMap<char, u8>` 與之後的字頻收成一個排序依據。第二個進資料包的來源加進來時一起收（票 06、08b）。
  - M1 票 2（連打 v1）：碼用 `String`，輸入碼切分時收成 `QuickCode` / `CangjieCode`（票 05、06）；`Engine` 與 `Pack` 欄位相同、`Session` 直接伸進 `engine.scheme` 與 `code_table` 查候選，改寫查詢時把查詢搬到 `Engine`。
  - M1 票 4（學習）：翻頁中的重排結果要不要丟、`generation` 怎麼算（票 08）。
  - M1 票 5（英文與簡繁）：`Options` 沒有簽章用到它，plan §2.1 的 `set_options` 跟著第一個真的選項一起加。
  - M1 票 7（佈局）：`qim-layout` 進來時 plan §1.3 表要改，順便補「不可以」欄與 `FORBIDDEN_PATHS` 的同步測試（票 02）。
  - M1 票 8、9：組字中 `z`、Enter、←、→ 交還宿主的行為（票 07、08）；`toKey` 的分支在鍵位變多時加單元測試工具再測（票 07）。
  - M9 Windows：`check::exec` 的 `npm.cmd` 分支沒有自動測試（票 01）。
  - CI 接上時（建遠端要先問使用者）：pre-commit 檢查工作目錄而非暫存區（票 01），由 CI 在乾淨 checkout 補。
  - 下次加閘門時：`docs::report` / `deps::report` 與測試的 `assert_red` 抽共用（票 04）。
  - 不修，選定的語義或代價：合併中一律放行、`#12 fix` 回報成空訊息（票 03）；Unihan 下載失敗或格式錯仍讓 `--verify` 失敗（票 06）；改名 `std` 繞過 `deps` 掃描，plan §1.3 已註明（票 02）；加一種鍵要改四處，薄綁定的代價（票 07）；`check` 的 `e2e` 步驟用現有的 `build/packs/`，舊包只會紅、不會假綠，`docs/development.md` 已寫要先跑 `cargo xtask data`。
  - 不修，只轉交或重複很小：`main.rs` 參數解析與 `qim-databuild` 轉交沒測試（票 01、05）；外部工具步驟只手動驗過，步驟表由 `development_md_lists_every_step_as_coded` 守（票 01）；`CheckId` 用拼音字名（票 05）；測試資料包的 `Scheme` 建構三份、Unihan zip 組法兩份、測試的 `Ime` 與 `WebSession` 同形（票 06、07、08）；`Key::Digit(u8)` 收得到 0–255（票 08）；`merge_in_progress` 測試繼承 `GIT_DIR`，hook 不跑 test 時不會發生（票 03）；`CrossCheck` 摘要由呼叫端組、各模組都有 `run`；xtask 的依賴不列 plan §1.4（票 02）。
  - 五代帶 `x` 前綴的碼照樣推導速成碼：真資料（libcangjie `346026b`）只有 `○`、`〇` 兩個符號，不影響漢字，不修。
  - 票 04 的 wayfinder 那條已結案（見上一條使用者決定）。
- 2026-11-04 重驗在乾淨 checkout（`git worktree add --detach` 到 session 暫存目錄，`.cache`、`build`、`target`、`node_modules` 都不存在，照 `docs/development.md`〈安裝〉從 `npm ci` 做到 `cargo xtask data`）：
  - `cargo xtask check`（`c5603ad`）：`==> deps` `==> docs` `==> wasm` `==> tsc` `==> biome` `==> e2e`、`6 passed (1.1s)`、`check 通過`，exit 0。
  - hook：暫存 repo（`git archive` 出的快照、`git init`、`core.hooksPath .githooks`）。`git commit -m "Add stuff."` → `- 缺冒號：subject 要是 \`type(scope)!: description\``，exit 1；`git commit -m "feat: Add stuff."` → `- description 要小寫開頭（現在是 \`A\`）`、`- description 結尾不要句號`，exit 1；`git commit -m "chore: add hook probe"` → `1 file changed`，exit 0；`git log` 只多了這一個 commit。本 repo 沒留測試 commit。
  - 雙向變異測試（`cargo test` 內）：`deps::tests::formatting_naming_and_allowed_paths_are_green` 與 `core_dependency_off_the_table_is_red`、`use_std_fs_in_core_is_red` 等，`deps::tests` 共 20 支；`docs::tests::rewording_and_reformatting_rows_is_green`、`rewriting_spec_body_is_green` 與 `duplicated_session_section_from_union_merge_is_red`、`not_exactly_one_milestone_in_progress_is_red`、`issues_without_spec_are_red` 等，`docs::tests` 共 31 支；`archiving_m0_while_opening_m1_is_green` 守封存那一步。
  - `cargo xtask data --verify`（`c5603ad`，`rm -rf build` 後跑兩次）：連打字集 17611 字、擴展區 57052 字、Unihan 差異 2744 字（比對 29121 字，其中 2246 字等於五代碼），八項 `[通過]`、`verify 通過`；兩次 sha256 都是 `31974c80fe810e4d0a24537acb6767346e34f40bac338d8361b44f1963e2125f`。
  - Playwright（08b 的版本）：讀回 `a` 候選首位「日」、空白後輸入框「日」；`ar` 第一頁首位「問」、數字鍵上屏「問」；翻頁後空白上屏全部候選的第一個「問」。截圖 `.scratch/m0/evidence/09-type-a.png`、`09-type-a-space.png`、`09-ar-found.png`、`09-ar-committed.png`、`09-page-1.png`、`09-page-2.png`、`09-space-after-paging.png`、`09-space-after-paging-committed.png`、`09-backspace.png`。
- 2026-11-04 封存：M0 的 31 列 session 紀錄與 29 條偏差移到 `docs/archive/M0.md`；里程碑表 M0 連到封存檔、M1 進行中；plan §7 表 M0 完成、M1 進行中，§7.1 指向 spec 與封存檔；`CLAUDE.md` 目錄地圖的 `docs/` 列加 `archive/`。封存後 `cargo xtask check` 的 `docs` 步驟綠燈。
- 2026-11-04 本 session 的 code-review（`d7ce6dd` 起）已處理：plan 開頭的狀態行改成 M0 完成、M1 進行中；progress 里程碑表 M1 的「拆票前讀」加上本票 Comments，拆 M1 時才看得到上面留給 M1 的清單。沒處理：`unihan::cangjie_codes` 不照其他模組叫 `parse`，因為 `unihan` 有兩個取欄函式，`parse` 分不出是哪一欄；`main.rs` 的 `data()` 改成執行期讀 `CARGO`，只轉交，沒補測試。
