# 05: 速成碼表資料包

Status: done
Blocked by: 01
讀: plan §1.3、§2.5、§4 開頭、§4.1 步驟 1–4、§4.5；brief §4.1、§4.2、§5.1、§12.1；ADR 0003；`docs/research/data-sources.md`（libcangjie 一節，`table.txt` 的欄位定義）；spec〈範圍〉〈共同規則〉

## What to build

開發者跑 `cargo xtask data`，從釘住版本的 libcangjie 建出速成輸入方案的資料包；跑 `cargo xtask data --verify` 拿到驗證報告，知道有沒有掉字、有沒有倉頡碼滲進速成碼表。

- `data/sources.toml`：libcangjie（freedesktop GitLab，commit `346026b`）的 URL、`data/table.txt` 的 sha256、授權、取用的檔案。
- `qim-databuild` 下載來源到 `.cache/sources/`，sha256 不符就失敗；已有快取時不連網。
- 推導速成碼表（plan §4.1 步驟 1–4）：
  - 漢字中 Big5 或 HKSCS 旗標為 1 的是連打字集，其餘漢字標成擴展區。
  - 由倉頡三代（可能多碼，逗號分隔）與五代各自推導速成碼，取聯集。
  - 字序用 libcangjie 的 Big5 排序值；擴展區的字排在連打字集之後。
- `data/schemes/quick.toml`：照 plan §2.5 的欄位，字根 a–y 填速成的字根；由 `qim-databuild` 編進資料包。
- 資料包格式定義在 `qim-core`（`qim-databuild` 依賴它），`postcard` 序列化，帶格式版本號；`qim-core` 從位元組解碼，遇到不認得的版本回錯誤，不 panic。
- 產物放 `build/packs/`，旁邊放 `LICENSE` 與 `NOTICE`（libcangjie 資料的 Public Domain 聲明與出處）。
- `--verify` 跑 plan §4.5 的檢查，Unihan 交叉比對除外（票 06）：不掉字、鍵長只有 1 或 2、抽查、決定性。
- `.gitignore` 加 `.cache/`、`build/`；`docs/development.md` 加資料建置指令。

## 驗收

- [x] `cargo xtask data --verify` 綠燈（貼輸出）。
- [x] 報告列出連打字集字數與擴展區字數。
- [x] 抽查：`anr` 查不到「問」；「問」的速成碼含 `ar`；「丟」同時有 `hi` 與 `mi`。
- [x] 鍵長檢查：測試在碼表裡造一個 3 碼的鍵會紅（ADR 0003 的守門）。
- [x] 同樣的來源建兩次，資料包 sha256 相同。
- [x] 竄改快取裡的 `table.txt`，建置因 sha256 不符失敗。
- [x] `qim-core` 的測試：資料包往返編解碼一致；格式版本不符時回錯誤。

## Comments

- 2026-11-03 資料包一個方案一份：`build/packs/<方案 id>/<方案 id>.pack` 裝方案與碼表，旁邊放 `LICENSE`、`NOTICE`。碼表每方案一份（ADR 0003），跟方案一起發佈；之後的詞庫、bigram 授權不同，各自一個目錄才不混授權檔。plan §2.5 的 `[packs]` 拿掉 `code_table`、§4 開頭同步改。
- 2026-11-03 `quick.toml` 只放現在用得到的欄位（id、name、version、alphabet、max_code_length、radicals）；`[packs]`、`[ranking]` 在 M1 用到時加，`deny_unknown_fields` 會逼加欄位時同步改 `qim_core::Scheme`。
- 2026-11-03 位元組格式：魔數 `QIMP`、postcard varint 的 `FORMAT_VERSION`（1）、postcard 本體；結尾有多餘位元組也算損壞。碼表是 `BTreeMap<String, Vec<CodeEntry>>`，`CodeEntry { text: char, extension: bool }`，字序在建置時排好。
- 2026-11-03 libcangjie 同一個字可能分在幾列（`乍`、`嘅`），推導與驗證都以字為單位合併：旗標取 OR、排序值取非 0 的最小值。字序是（連打字集在前、排序值非 0 在前、排序值、碼位）；HKSCS 非 Big5 的字排序值都是 0，排在 Big5 字之後、依碼位。
- 2026-11-03 快取路徑 `.cache/sources/<來源>/<sha256 前 16 碼>/<檔案路徑>`，換釘版本自動換目錄；快取不符直接失敗、不重新下載。另取 `data/README.table.md`，建置時核對 `sources.toml` 照抄的 Public Domain 聲明確實在上游檔內，再寫進 `LICENSE`。
- 2026-11-03 `xtask/src/deps.rs` 的變異測試原本假設 `qim-core` 是空殼（斷言固定行號、往 `Cargo.toml` 追加 `serde`）。改成 `append` 回傳追加起點的行號，格式測試改寫既有的 `serde` 行。
- 2026-11-03 驗收輸出：
  - `cargo xtask data --verify`：連打字集 17611 字、擴展區 57052 字、速成碼 649 個，七項 `[通過]`，`verify 通過`，sha256 `42669866fd6fc4c9028a9da649505eb2a5bcda6a6bb8db32844e770451b8d00f`。
  - `rm -rf build` 後重建兩次，`shasum -a 256` 都是上面的值。
  - 把快取裡 `table.txt` 的「問」改成 `abr`：`建置失敗：…table.txt：sha256 不符，應為 86118f2a…，實為 e30985ca…`，exit 1；還原後 sha256 回到釘住的值。
  - 3 碼鍵、掉字、`anr` 滲入、抽查缺碼、決定性不符、解不開的資料包各有一支會紅的測試（`crates/qim-databuild/tests/verify.rs`）；版本不符、非資料包、截斷與多餘位元組在 `crates/qim-core/tests/pack.rs`。
- 2026-11-03 code-review 已處理：驗證與推導對連打字集的口徑統一成以字為單位（先寫了會紅的測試）；`--verify` 的第二次建置改為重讀快取與方案檔；名詞表加「連打字集」「擴展區」；`sources.toml` 路徑與 `malformed` 去重；plan §4.5 標題改成「每次改資料管線或來源都跑」。
- 2026-11-03 code-review 沒處理的發現：
  - `CheckId::NoCangjieCodeForWen` 等用拼音字名；它們就是 plan §4.5 的抽查，doc comment 寫了原文，保留。
  - 碼都用 `String`，plan §0 有 `QuickCode` / `CangjieCode`；等 M1 有詞庫碼與輸入碼切分時再收成型別。
  - `qim-databuild` 的 `main.rs` 參數解析與 `cargo xtask data` 的轉交沒有測試，兩者只轉交；行為由 `run` 的測試守。
  - `sources::Source` 的 `commit`、`license_file`、`license_statement` 必填、`deny_unknown_fields`；Unihan 沒有 git commit，票 06 要調 schema（已在票 06 的 Comments 提醒）。
