# Swift 經 `qim-ffi` 用核心：打 `anr`、每鍵延遲

回答的問題（M2 票 07）：uniffi 產生的 Swift 綁定接得上核心嗎？經綁定的每鍵延遲比原生多多少？結果摘進 `docs/research/ios-feasibility.md`〈綁定開銷〉。

## 跑法

先跑過 `cargo xtask data`（資料包），在 repo 根目錄：

```sh
scripts/experiments/ffi-swift/run.sh
```

`run.sh` 依序：`cargo xtask ffi`（靜態函式庫與 Swift 綁定，見 `docs/development.md`〈Swift 綁定〉）、`swiftc -O` 把 `main.swift` 與產生的 `qim.swift` 編成 `build/ffi/bin/qim-swift`、`qim-swift demo` 打 `anr` 再按空白（首選或上屏不是「門口」時 exit 非零）、`qim-swift latency` 量每鍵延遲。

量法同 `cargo xtask bench` 的每鍵延遲（`docs/development.md`〈每鍵延遲〉）：同一批輸入（固定題、`bench/corpus/`、`bench/colloquial/`，依序）、同四份資料包（速成、詞庫、bigram、英文）、同三組（全部、第 20 碼起、選開頭一段）、nearest-rank。每鍵計 `Session.press(kind:value:)` 加讀出畫組字與候選要的欄位，計時用 `clock_gettime_nsec_np(CLOCK_UPTIME_RAW)`。和原生的差別就是綁定的開銷：按鍵種類的字串、`Mutex`、快照序列化成 uniffi 的 `RustBuffer` 再在 Swift 解開。

換 uniffi、Swift 或 Rust 的版本、或核心變重時重量。

## 上次結果（2026-11-10）

環境：Apple M6、32 GB、macOS 27.0.1，Xcode 27.1（Swift 6.4），Rust 1.99.1，uniffi 0.32.2，commit `2b49606` 加上本票的改動；`qim-ffi` release、`swiftc -O`。同時跑 `cargo xtask bench` 取原生的數字，各跑兩次，數字幾乎不變（差在 0.5 µs 內），列第二次。

`demo`：

```text
按 a：輸入碼 a，首選 日
按 n：輸入碼 an，首選 門
按 r：輸入碼 anr，首選 門口
按空白：上屏 門口
```

`latency`（µs）：

| 環境 | 輸入 | 輸入數 | 鍵數 | p50 | p95 |
| --- | --- | ---: | ---: | ---: | ---: |
| 原生（release，`cargo xtask bench`） | 全部 | 6722 | 123330 | 30.71 | 41.75 |
| 原生 | 第 20 碼起 | 2587 | 27142 | 34.75 | 44.25 |
| 原生 | 選開頭一段 | 6411 | 6411 | 222.25 | 737.12 |
| Swift（uniffi） | 全部 | 6722 | 123330 | 42.25 | 58.08 |
| Swift（uniffi） | 第 20 碼起 | 2587 | 27142 | 51.62 | 64.38 |
| Swift（uniffi） | 選開頭一段 | 6411 | 6411 | 235.08 | 759.71 |

`build/ffi/bin/qim-swift`（含核心）163 KB；`libqim_ffi.a` 20.6 MB 是沒經連結器刪減的大小。

## 結論

- 綁定接得上：Swift 打 `anr` 首選「門口」、空白上屏「門口」，與網頁、原生相同。
- 綁定每次呼叫多約 10–20 µs，和組字長短、選段與否關係不大，看來是固定開銷（快照序列化與解開，沒有再拆開量）；p95 58 µs、最重的選段 760 µs，都比每鍵 50 ms 的預算小兩個數量級以上。
- iPhone 上的數字由接上鍵盤擴充的票（M2 票 08）實測；Mac 的 M6 比手機快，這裡的數字只當下限。
