# 04: 在 Mac 量資料包載入後的常駐記憶體

Status: ready-for-agent
Blocked by: None (can start immediately)
讀: plan §1.4、§4、§7.8、§8（mmap 與 `jsonschema` 兩列）；brief §9；spec〈共同規則〉；`docs/development.md`〈資料建置〉

## What to build

上 iOS 前先知道每份資料包在記憶體裡有多大：postcard 反序列化後的結構可能比檔案大好幾倍。結果用來決定票 08 把哪些資料包放進鍵盤擴充，也是票 09「要不要改 mmap 友善格式」的依據。

- `scripts/experiments/pack-memory/`：一個不加進 workspace 的小程式，依序載入速成、詞庫、bigram、字根提示、簡繁、英文資料包與一個 `Engine`，每步記常駐記憶體（RSS 與 macOS 的 footprint，指標的意義查證後寫進 README），並列出檔案大小與倍數。
- `jsonschema`：同一個程式連同 `qim-layout` 載入預設佈局並跑驗證，有無 Schema 驗證各量一次常駐記憶體與執行檔大小。
- 結果寫進 `docs/research/ios-feasibility.md` 的〈Mac 上的資料包記憶體〉一節，並建議：鍵盤擴充要不要略過 Schema 驗證、哪些資料包最可能需要改格式。

## 驗收

- [ ] README 有日期、機器、Rust 版本、commit、量法與結果表；照 README 能重量。
- [ ] 每份資料包都有「檔案大小 / 載入後增加的常駐記憶體 / 倍數」；有無 `jsonschema` 各一列。
- [ ] `docs/research/ios-feasibility.md` 有這一節與建議。
- [ ] `cargo xtask check` 綠燈。
