# 06: Unihan 交叉比對報表

Status: ready-for-agent
Blocked by: 05
讀: plan §4 開頭、§4.5；brief §5.2、§12.1；`docs/research/data-sources.md`（Unihan 與 libcangjie 的比較）；spec〈共同規則〉

## What to build

開發者跑 `cargo xtask data --verify` 時，多拿到一份 libcangjie 與 Unihan `kCangjie` 的差異報表，用來抓錯碼；有差異也不擋建置。

- `data/sources.toml` 加 Unihan：開工時最新的 Unicode 版本，釘 URL、sha256、授權（Unicode License v3）。只用來驗證，不進資料包。
- 讀 `kCangjie`，與 libcangjie 的倉頡三代碼比對（研究顯示 `kCangjie` 對應三代）；兩邊都有的字才比。
- 差異清單寫成報表檔放在 `build/` 下，格式由本票決定；`--verify` 的輸出列出比對字數、差異數與報表路徑。
- 報表格式寫進 `docs/development.md` 的資料建置一節。

## 驗收

- [ ] `cargo xtask data --verify` 綠燈，輸出含 Unihan 比對字數與差異數（貼輸出）。
- [ ] 差異數大於 0 時建置仍成功。
- [ ] 測試：用小型假資料，一個字兩邊碼相同不進報表、碼不同進報表、只有一邊有的字不進報表。

## Comments

- 2026-11-03 票 05 留下的介面：`qim_databuild::sources::Source` 的 `commit`、`license_file`、`license_statement` 是必填、`deny_unknown_fields`，是照 libcangjie 寫的；Unihan 沒有 git commit，開工時調 schema（例如 `commit` 改成 `version`）。快取路徑以 sha256 為鍵，不受影響。
