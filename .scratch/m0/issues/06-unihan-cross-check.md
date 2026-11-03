# 06: Unihan 交叉比對報表

Status: done
Blocked by: 05
讀: plan §4 開頭、§4.5；brief §5.2、§12.1；`docs/research/data-sources.md`（Unihan 與 libcangjie 的比較）；spec〈共同規則〉

## What to build

開發者跑 `cargo xtask data --verify` 時，多拿到一份 libcangjie 與 Unihan `kCangjie` 的差異報表，用來抓錯碼；有差異也不擋建置。

- `data/sources.toml` 加 Unihan：開工時最新的 Unicode 版本，釘 URL、sha256、授權（Unicode License v3）。只用來驗證，不進資料包。
- 讀 `kCangjie`，與 libcangjie 的倉頡三代碼比對（研究顯示 `kCangjie` 對應三代）；兩邊都有的字才比。
- 差異清單寫成報表檔放在 `build/` 下，格式由本票決定；`--verify` 的輸出列出比對字數、差異數與報表路徑。
- 報表格式寫進 `docs/development.md` 的資料建置一節。

## 驗收

- [x] `cargo xtask data --verify` 綠燈，輸出含 Unihan 比對字數與差異數（貼輸出）。
- [x] 差異數大於 0 時建置仍成功。
- [x] 測試：用小型假資料，一個字兩邊碼相同不進報表、碼不同進報表、只有一邊有的字不進報表。

## Comments

- 2026-11-03 票 05 留下的介面：`qim_databuild::sources::Source` 的 `commit`、`license_file`、`license_statement` 是必填、`deny_unknown_fields`，是照 libcangjie 寫的；Unihan 沒有 git commit，開工時調 schema（例如 `commit` 改成 `version`）。快取路徑以 sha256 為鍵，不受影響。
- 2026-11-03 Unihan 釘 18.0.0 的 `Unihan.zip`（sha256 `4c93ea9c…`），`kCangjie` 在 zip 裡的 `Unihan_DictionaryLikeData.txt`。Unicode 只發 zip，加 `zip` 9.3.0（只開 deflate 解壓）。只在 `--verify` 時下載；不帶 `--verify` 的建置不碰 Unihan。
- 2026-11-03 schema：`commit` 改成 `version`（git 來源填 commit）；`license_file`、`license_statement` 改成選填，進資料包的來源缺了就建置失敗（有測試）。Unihan 不進資料包，不寫這兩欄。
- 2026-11-03 比對規則：兩邊都有碼才比，libcangjie 一個字所有列的三代碼合併，Unihan 的碼等於其中之一就算相同。報表 `build/reports/unihan-kcangjie.tsv`，格式見 `docs/development.md`〈Unihan 交叉比對〉。
- 2026-11-03 報表多一欄 libcangjie 五代碼、摘要多報「其中幾字等於五代碼」：真資料 2,744 個差異中 2,246 個等於五代碼，多半是版本差異；剩 498 個才是疑似錯碼。brief §12.1 記了這組數字。速成碼由三代與五代取聯集推導，不受影響。
- 2026-11-03 驗收輸出：
  - `cargo xtask data --verify`：`Unihan kCangjie：比對 29121 字，差異 2744 字（其中 2246 字等於五代碼），報表 build/reports/unihan-kcangjie.tsv（不影響 verify 結果）`，七項 `[通過]`，`verify 通過`，資料包 sha256 仍是 `42669866…`。
  - 差異數 > 0 時建置仍成功：上面的真資料，與 `tests/run.rs` 的 `verify_writes_the_unihan_report_and_still_passes_with_differences`。
  - 假資料的三種情況在 `crates/qim-databuild/tests/unihan.rs`。
- 2026-11-03 code-review 已處理：plan §4 開頭的「commit」改成「版本」；`pipeline.rs` 的格式佔位名 `commit` 改 `version`。
- 2026-11-03 code-review 沒處理的發現：
  - 「不擋建置」只涵蓋有差異：Unihan 下載失敗、sha256 不符、`kCangjie` 格式不合仍讓 `--verify` 失敗。來源已釘死，這些是環境或來源壞了，該停下來看。
  - `license_file` / `license_statement` 兩個 `Option` 允許只填一個，靠建置時擋；包成一個子表要改 `sources.toml` 結構，且 serde 的 `flatten` 不能配 `deny_unknown_fields`，等第二個進資料包的來源（M1 詞庫）時再看。
  - 測試裡組 Unihan zip 的程式碼在 `tests/run.rs` 與 `tests/unihan.rs` 各一份，各十行內，保留。
  - 倉頡碼仍用 `String`，同票 05 的決定，等 M1 再收成 `CangjieCode`。
