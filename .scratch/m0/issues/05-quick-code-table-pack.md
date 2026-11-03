# 05: 速成碼表資料包

Status: ready-for-agent
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

- [ ] `cargo xtask data --verify` 綠燈（貼輸出）。
- [ ] 報告列出連打字集字數與擴展區字數。
- [ ] 抽查：`anr` 查不到「問」；「問」的速成碼含 `ar`；「丟」同時有 `hi` 與 `mi`。
- [ ] 鍵長檢查：測試在碼表裡造一個 3 碼的鍵會紅（ADR 0003 的守門）。
- [ ] 同樣的來源建兩次，資料包 sha256 相同。
- [ ] 竄改快取裡的 `table.txt`，建置因 sha256 不符失敗。
- [ ] `qim-core` 的測試：資料包往返編解碼一致；格式版本不符時回錯誤。
