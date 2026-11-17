# 英文自動修正的門檻與每鍵延遲

回答的問題（M4 票 25）：英文模式的自動修正（`crates/qim-core/src/english_mode.rs` 的 `autocorrection`）要比次一個修正高多少才修？英文容錯與自動修正讓英文模式每個鍵慢多少？

- 日期：2026-11-17
- 機器：Apple M6，macOS 27.0.1，跑的時候機器有別的負載（load average 約 3–6）
- 版本：commit `0144e75` 之後的工作樹（門檻改成 4 倍之後），詞表是 `data/sources.toml` 的 LatinIME `127336e` `en_US_wordlist.combined.gz`

## 門檻：`threshold.py`

```sh
python3 -I scripts/experiments/autocorrect-threshold/threshold.py <快取>/sources/latinime/<sha256 前 16 碼>/dictionaries/en_US_wordlist.combined.gz
```

照核心的規則重寫一份（只用標準函式庫）：一處拼錯（換、對調、漏打、多打）的詞典詞當修正，漏撇號與重複字母不扣分、換、對調、漏打一個字母扣刻度一倍、多打一個非重複字母不自動修；至少 3 個字元；有漏撇號的修正而最高的不是它時不修。核心另有的條件（大寫、數字、email 字元、學過的串）這裡的題目都碰不到，不寫。

三組題目：

- Gboard 22 個：`docs/research/gboard-english-2026-11-15.md`〈自動修正〉裡 Gboard 18.5.10 修了的錯字。
- AOSP 47 條：詞表裡 `not_a_word` 的錯拼與它的 whitelist 修正（多半是漏撇號的縮寫，如 `youd` → you'd，另有 `alot` → a lot 這類一處拼錯到不了的）。LatinIME 靠 whitelist 修它們，QIM 沒收 whitelist，這組看的是「修錯」有多少。
- 隨機錯字 3,000 個：詞表裡 f ≥ 120、全是字母、3 個字母以上的前 3,000 個詞，隨機挑一個位置做一處換（均勻亂選字母）、漏打、多打（均勻亂選字母）或對調，結果不是詞典的詞才收（種子 25）。均勻亂選字母比真實的鄰鍵錯難，修對的比例偏低。

| 門檻（倍） | Gboard 22 個修對 | AOSP 47 條修對 | AOSP 47 條修錯 | 隨機錯字修對 | 隨機錯字修錯 | 隨機錯字不修 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 2 | 18 | 8 | 6 | 61.2% | 9.1% | 29.7% |
| 4 | 17 | 8 | 4 | 58.9% | 6.7% | 34.4% |
| 8 | 16 | 8 | 1 | 56.5% | 5.4% | 38.1% |

採 4 倍。2 倍時 `helo` 修成 held（held 扣分後 149，help 140，差 9 剛過 2 倍的 8.6），沒有空間模型時 o 換成遠處的 d 和換成鄰鍵 p 扣一樣的分，修錯會比表上多；4 倍時這類差不多常見的都留給使用者選。8 倍再少掉 Gboard 的一個與 2.4% 的修對，換 1.3% 的修錯。

4 倍時 Gboard 22 個沒修的 5 個：`wierd`（wired 99、weird 96）、`peice`（piece、peace、price）、`gogle`（google 72、Goole 65）差不多常見；`truely` 要去掉一個字母；`its` 本身是詞。AOSP 47 條修錯的 4 個：`inteh` → inter、`itsa` → IMSA、`mote` → more、`wel` → well。

## 每鍵延遲：`latency/`

```sh
cd scripts/experiments/autocorrect-threshold/latency
cargo run --release -q -- ../../../../build/packs
```

先跑過 `cargo xtask data`。速成、詞庫、字級 bigram 與英文資料包建一個 `Engine`，空白的學習資料，英文模式逐字打一段 269 個字元、含 24 個錯字與一個 email 的英文，重複 20 次，量每個 `Session::handle`。

| 設定 | 鍵數 | p50（µs） | p95（µs） | 最大（µs） |
| --- | ---: | ---: | ---: | ---: |
| 只補完（英文容錯關，等於票 25 之前） | 5380 | 2.4 | 224.3 | 516.1 |
| 英文容錯（預設） | 5380 | 116.9 | 321.0 | 546.8 |
| 英文容錯＋自動修正 | 5380 | 186.0 | 404.1 | 762.5 |

都在〈每鍵延遲上限〉的空白學習資料 p95 1000 µs 內（`.scratch/m4/spec.md`）。英文容錯每鍵查一處拼錯的全部字串（約 60 × 詞長個，各做一次詞典的二分搜尋）；自動修正開著時建議列與打完時各算一次。
