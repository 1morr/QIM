# wildcard：萬用鍵的語意與延遲（M4 票 11）

- 日期：2026-11-12；機器：Apple M6（Mac18,5，32 GB）、macOS 27.0.1；rustc 1.99.1。
- commit：在 M4 票 11 的分支上跑（開工 `8f1cde1`，合併票 01 後 `43fd777` 起的工作區）；之後重跑用 `git log -- scripts/experiments/wildcard` 對。
- 問題：速成的萬用鍵 `z` 收在哪些位置、整句與單字的首選命中、每鍵延遲。結果與結論在 `docs/research/wildcard.md`。

## 量法

`src/main.rs`：讀四個語料的 dev 切分（`build/corpus/<名>/dev.tsv`），每句由雜湊挑一個字，分三組把首碼、尾碼或一碼字的碼換成 `z`，另打原樣與「`z` 吃掉」（改前的行為）；單字取句子裡出現過的兩碼字。指標用 `qim_bench::score_set`（同首選字基準），延遲用 `qim_bench::native_timing_with`，空白與學過（`qim_bench::learned_model` 學完全部公開集合的句子）各一次。題目由程式產生、不存檔。

獨立的 crate（`Cargo.toml` 有空的 `[workspace]`），不在 repo 的 workspace 裡，`cargo xtask check` 不編它；它用路徑依賴 `qim-core`、`qim-bench`。改程式後自己跑 `cargo clippy --release -- -D warnings`、`cargo fmt --check`。

## 怎麼重跑

```sh
cargo xtask data                                 # repo 根目錄；速成資料包要有萬用鍵
cd scripts/experiments/wildcard
cargo run --release                              # 全部，約 6 分鐘（12 核）
cargo run --release -- --latency-only            # 只量每鍵延遲，約 2 分鐘；負載低時跑
```

## 結果

| 組 | 整句首選（四個語料） | 修正/句 |
| --- | --- | --- |
| 原樣 | 51.9% / 49.6% / 44.2% / 48.9% | 1.8–2.1 |
| 首碼換成 `z` | 36.7% / 36.8% / 30.5% / 34.4% | 3.9–5.1 |
| 尾碼換成 `z` | 36.4% / 36.5% / 27.6% / 32.7% | 3.8–4.8 |
| 一碼字換成 `z` | 38.4% / 37.7% / 29.7% / 35.9% | 6.5–10.6 |
| 單字（兩碼字一碼換成 `z`） | 0.9%–2.4% | 5.5–9.7 |

每鍵延遲（原生 release，負載中等時單獨跑）：空白全部 p95 786 µs、選開頭一段 p95 3.8 ms；學過 1,056 µs、6.4 ms；都在 M4 spec〈每鍵延遲上限〉內。完整的表、規則比較與單字的候選數在 `docs/research/wildcard.md`。
