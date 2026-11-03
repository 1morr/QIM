# 01: 語料管線：粵語維基 tracer

Status: ready-for-agent
Blocked by: None (can start immediately)
讀: plan §4 開頭、§4.3、§4.5、§5.2（`bench/corpus/` 那列與「防洩漏」）；brief §5.2、§9；`docs/research/data-sources.md`；spec〈範圍〉〈共同規則〉

## What to build

`cargo xtask data` 從粵語維基一路產出第一個基準集合：取得 → 前處理 → 切分 → 轉碼 → `bench/corpus/`，讓下一張票的首選字基準有句子可跑。只做粵語維基（量小、建得快），中文維基與口語語料在票 03 接上同一條管線。

- 開工先查證 Wikimedia CirrusSearch 內容 dump 現在的網址、格式與粵語維基（`zh_yuewiki`）有沒有；查不到或格式改了，照 plan §8 退回 pages-articles 加文字抽取，記偏差。dump 日期與 sha256 釘進 `data/sources.toml`。
- 前處理照 plan §4.3：OpenCC `s2hk` 轉繁體（`qim-databuild` 可以用 `ferrous-opencc`，原生編譯即可）→ 切句 → 只留連打字集內的字（含字集外字的句子怎麼處理在本票定，寫進 `## Comments`）。
- 切 train / dev / test，切法決定性（同樣的來源切出同樣的結果）；test 進 `bench/corpus/`，句數上限在本票定（進 git，不要太大），附 `ATTRIBUTION.md`（CC BY-SA 4.0、來源、dump 日期）。train / dev 是建置產物，不進 git。
- 轉碼：每個字取速成碼，多碼字取五代第一碼（plan §5.2）；轉碼函式之後個人集（票 04）也要用，放在 `qim-databuild` 的公開介面。
- `cargo xtask data --verify` 報：各切分句數、轉碼失敗數、test 句原樣出現在 train 的句數（要為 0）。

## 驗收

- [ ] `data/sources.toml` 釘了粵語維基 dump（日期、sha256、授權、取用的檔案）。
- [ ] `bench/corpus/` 有粵語維基 test 切分與 `ATTRIBUTION.md`；`.gitignore` 排除 train / dev 產物。
- [ ] `cargo xtask data --verify` 綠燈，報告列出句數、轉碼失敗數、洩漏句數 0（貼輸出）。
- [ ] 同樣的來源建兩次，`bench/corpus/` 與訓練切分的 sha256 相同。
- [ ] 轉碼有單元測試：單碼字、多碼字取五代第一碼、字集外字算失敗。
- [ ] `docs/development.md` 若有新指令或耗時步驟已更新。

## Comments
