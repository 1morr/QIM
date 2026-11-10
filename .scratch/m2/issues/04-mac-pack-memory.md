# 04: 在 Mac 量資料包載入後的常駐記憶體

Status: done
Blocked by: None (can start immediately)
讀: plan §1.4、§4、§7.8、§8（mmap 與 `jsonschema` 兩列）；brief §9；spec〈共同規則〉；`docs/development.md`〈資料建置〉

## What to build

上 iOS 前先知道每份資料包在記憶體裡有多大：postcard 反序列化後的結構可能比檔案大好幾倍。結果用來決定票 08 把哪些資料包放進鍵盤擴充，也是票 09「要不要改 mmap 友善格式」的依據。

- `scripts/experiments/pack-memory/`：一個不加進 workspace 的小程式，依序載入速成、詞庫、bigram、字根提示、簡繁、英文資料包與一個 `Engine`，每步記常駐記憶體（RSS 與 macOS 的 footprint，指標的意義查證後寫進 README），並列出檔案大小與倍數。
- `jsonschema`：同一個程式連同 `qim-layout` 載入預設佈局並跑驗證，有無 Schema 驗證各量一次常駐記憶體與執行檔大小。
- 結果寫進 `docs/research/ios-feasibility.md` 的〈Mac 上的資料包記憶體〉一節，並建議：鍵盤擴充要不要略過 Schema 驗證、哪些資料包最可能需要改格式。

## 驗收

- [x] README 有日期、機器、Rust 版本、commit、量法與結果表；照 README 能重量。
- [x] 每份資料包都有「檔案大小 / 載入後增加的常駐記憶體 / 倍數」；有無 `jsonschema` 各一列。
- [x] `docs/research/ios-feasibility.md` 有這一節與建議。
- [x] `cargo xtask check` 綠燈。

## Comments

- 2026-11-10 結果：全部載入 footprint 約 96 MB（還在用的 heap 68 MB，詞庫佔 41 MB）；整條 `validate` 讓執行檔多 3.1 MB。建議與數字見 `docs/research/ios-feasibility.md`〈Mac 上的資料包記憶體〉。
- 票外發現：macOS 27 上 malloc 釋放的頁留在 footprint、只被之後的配置重用（13 MB 對照），所以讀檔緩衝區與 `Engine::new` 的暫存 `HashSet`（單獨 +17.3 MB）會墊高 footprint；省法列在研究文件〈建議〉，交給票 09 決定。
- 量法的取捨：「不驗證」那版只用 `serde_json` 讀型別，Schema 那版是整條 `validate`；「只留型別與引用檢查」沒量，`qim-layout` 沒有公開那條路，等票 09 決定加 feature 後再量。
- code-review 未處理：`by_frequency` 複製碼表的峰值沒有單獨量（併在 `Engine` 那步）；程式輸出與 panic 訊息用中文，跟 repo 其他實驗與 xtask 一致，不照全域「log 用英文」。其餘發現（heap 註解、推論標註、HashSet 數字可重量、pressure relief 再呼叫一次、`run.sh` 相對路徑、重複的取樣碼）已修。
