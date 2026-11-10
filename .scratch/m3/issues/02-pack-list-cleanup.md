# 02: 資料包清單整理

Status: ready-for-agent
Blocked by: None (can start immediately)
讀: plan §1.2、§4、§4.5、§5.2（個人集）；spec〈共同規則〉；`.scratch/m1/issues/24-m1-close.md` Comments 的「M3」各條（資料包清單、個人集路徑兩條）

## What to build

M3 要加 emoji、粵語詞庫等資料包，還要給每份資料包加 manifest。先把散開的資料包清單收攏，加一種資料包只改一處。不改任何資料包的內容。

- `qim-databuild` 的資料包清單在 `Outcome`、`verify::Report`、`PackBytes`、`main.rs` 平行散開，`pipeline::run` 的建包形狀重複六次：收成一個描述（例如 `BuiltPack`）加 `write_pack`。
- 六種資料包的 `to_bytes` / `from_bytes` 同形：用一個 trait 收。
- 個人集表頭、語料目錄、`dev.tsv` 路徑在 `qim-databuild` 與 `qim-bench` 各寫一份：`xtask` 加測試比對兩邊（`xtask` 不受依賴規則限制）。

## 驗收

- [ ] 加一種資料包時，`qim-databuild` 只需在一處登記（Comments 寫明是哪裡）。
- [ ] `cargo xtask data --verify` 綠燈，六份資料包的 sha256 和改之前相同（貼前後對照）。
- [ ] `xtask` 的路徑比對測試做雙向變異：改掉一邊會紅、改無關命名不紅。
- [ ] `cargo xtask bench --check` 數字不變；`cargo xtask check` 綠燈。

## Comments
