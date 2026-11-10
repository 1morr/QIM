# 02: 資料包清單整理

Status: done
Blocked by: None (can start immediately)
讀: plan §1.2、§4、§4.5、§5.2（個人集）；spec〈共同規則〉；`.scratch/m1/issues/24-m1-close.md` Comments 的「M3」各條（資料包清單、個人集路徑兩條）

## What to build

M3 要加 emoji、粵語詞庫等資料包，還要給每份資料包加 manifest。先把散開的資料包清單收攏，加一種資料包只改一處。不改任何資料包的內容。

- `qim-databuild` 的資料包清單在 `Outcome`、`verify::Report`、`PackBytes`、`main.rs` 平行散開，`pipeline::run` 的建包形狀重複六次：收成一個描述（例如 `BuiltPack`）加 `write_pack`。
- 六種資料包的 `to_bytes` / `from_bytes` 同形：用一個 trait 收。
- 個人集表頭、語料目錄、`dev.tsv` 路徑在 `qim-databuild` 與 `qim-bench` 各寫一份：`xtask` 加測試比對兩邊（`xtask` 不受依賴規則限制）。

## 驗收

- [x] 加一種資料包時，`qim-databuild` 只需在一處登記（Comments 寫明是哪裡）。
- [x] `cargo xtask data --verify` 綠燈，六份資料包的 sha256 和改之前相同（貼前後對照）。
- [x] `xtask` 的路徑比對測試做雙向變異：改掉一邊會紅、改無關命名不紅。
- [x] `cargo xtask bench --check` 數字不變；`cargo xtask check` 綠燈。

## Comments

- 2026-11-11 做法：
  - `qim-core` 加 `Pack` trait（`MAGIC`、`id()`，`to_bytes` / `from_bytes` 是預設方法），六種資料包各只寫魔數與 id；格式與 `FORMAT_VERSION` 不變。呼叫端要 `use qim_core::Pack`（約 15 個檔）。
  - `qim-databuild` 新模組 `built`：`BuiltPack { name, id, magic, bytes }`，`built::find` / `decode` 依魔數取出某種資料包。`Outcome.packs: Vec<WrittenPack>` 取代六組 `*_path` / `*_sha256`；`verify::run` 收兩份 `&[BuiltPack]`，`PackBytes` 與 `Report` 的六個 sha256 欄位刪掉；`main.rs` 逐一印清單。`packing` 的上游來源與 repo 資料併成 `PackSource`，詞庫的口語詞表收進 `lexicon_sources()`。
- 一處登記：`crates/qim-databuild/src/pipeline.rs` 的 `build_packs` 那個 `vec![LicensedPack::new(…)]`。寫檔、印路徑與 sha256、`--verify` 的決定性比對都跟著它。新資料包要的定義放在旁邊：來源 id 與 NOTICE 說明在 `pipeline.rs`、授權來源常數在 `packing.rs`；解得開與內容抽查另加在 `verify.rs` 的 `decode` / 檢查（那是驗證，不是登記）。
- `cargo xtask data --verify` 前後對照（改前在 `4bcee1c` 跑）：兩次都 `verify 通過`，六份 sha256 相同，連同各資料包的 `LICENSE`、`NOTICE`、`build/layouts/`、`bench/corpus/`、`bench/colloquial/` 共 26 個檔 `shasum` 逐一相同：
  - 速成 `ae3bc207…d4f9`、詞庫 `08bc8adb…f5b0`、語言模型（原「bigram」）`9e393a0a…c173`、字根提示 `9e755488…11ce`、簡繁 `1c5ab2aa…c26f`、英文 `1afd4794…5a17`。
- `cargo xtask bench --check`：`bench 通過`，固定題 84/84；答錯 / 修正與 baseline 相同、沒有翻盤：yue-wiki 1131 / 4467、zh-wiki 1126 / 4505、common-voice-yue 1032 / 4043、hkcancor 663 / 2588、個人集 1081 / 1433（加權首選 87.16%）。
- 路徑比對：`xtask/tests/shared_paths.rs`（`xtask` 加 `qim-databuild`、`qim-bench` 為 dev 依賴）比個人集題目、來源、檔頭、兩個基準目錄、每個語料的 `dev.tsv`。`corpora::build_dir` / `dev_items_path` 改收語料名。雙向變異：
  - 檔內：任一邊任一項改掉只報那一項、多一項也報；語料改名或加語料不報。
  - 原始碼（手動，跑完還原）：`qim-bench` 的 `PERSONAL_SOURCE_HEADER` 改成 `"# rime-userdb sha256 "` → 紅；`corpora.rs` 的 `"dev.tsv"` 改成 `"dev-items.tsv"` → 紅；`CORPORA` 的 `hkcancor` 改名 → 綠。
- 輸出的改動（不是資料格式）：`cargo xtask data` 每次都印各資料包的路徑與 sha256，`--verify` 的報告不再重印；「資料包：」改「速成資料包：」、「bigram」改「語言模型」（名詞表的詞）。`docs/development.md` 已同步。
- code-review 未處理的發現：
  - 授權的讀取從建置前移到寫檔時：缺授權的錯誤要等語料建完（約 45 秒）才報。M3 的 manifest 票（20）若要提早失敗，可在建語料前先 `resolve` 一次清單的來源。
  - 有建置統計的資料包（像英文的 `english::Counts`）仍要在 `PackSet`、`Outcome`、`Report`、`main.rs` 各加欄位；emoji 票（16）若要報統計，可改成 `BuiltPack` 帶一段顯示文字。
  - `built::find` 假設每種資料包一份（已寫進 doc）；粵語詞庫票（18）若另建一份 `LexiconPack`，要改用 id 找。
  - `BuiltPack.name` 同時是顯示名與測試、決定性比對的鍵；`built` / `verify::decode` 回 `String` 錯誤而非 `PackError`。都不影響行為，留著。

