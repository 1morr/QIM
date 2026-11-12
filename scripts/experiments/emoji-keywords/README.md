# CLDR 關鍵字的門檻

回答的問題（M3 票 16）：CLDR 的標註是給 emoji 選單搜尋用的，關鍵字收得很寬（粵語標註裡 😄 的關鍵字有「嘴」「眼」「打開」「臉」）。全收時打常用詞會在後面塞不相關的笑臉。要收到多泛？規則用資料定，不手寫排除清單：短名（`type="tts"`）一律收；關鍵字只在它對到的 emoji 數（兩份標註合計、不重複）不超過門檻時收。結論寫在 `docs/research/emoji-candidates.md`，摘進 plan §2.4、§4.4 與 brief §12.1，門檻是 `crates/qim-databuild/src/emoji.rs` 的 `MAX_KEYWORD_EMOJI`。

## 跑法

先跑過一次 `cargo xtask data`（只讀快取與 `build/packs/`，不下載）。

```sh
cargo run --release -q -p qim-databuild --example emoji-keywords           # 各門檻的探測詞第一頁
WRITE=6 cargo run --release -q -p qim-databuild --example emoji-keywords   # 另把門檻 6 的 emoji 資料包寫到 build/packs/，接著跑 cargo xtask bench
```

`WRITE` 會蓋掉 `cargo xtask data` 建的 emoji 資料包；量完重跑 `cargo xtask data`。換 CLDR 版本時重跑，看「笑」「貓」「多謝」還有沒有 emoji。

## 結果

2026-11-11，Apple M6、macOS 27.0.1、rustc 1.99.1（`rust-toolchain.toml`），commit `3543c83` 之上（票 16 未 commit 的工作區），CLDR `release-49`（`96e54b2`）、emoji-test 18.0。每鍵經 `Session` 打（預設選項、每列最多 3 個 emoji），字取它排得最前的碼。

| 門檻 | 太泛不收的關鍵字 | 收的字詞 | 笑 `hk` | 貓 `bw` | 多謝 `niyi` | 打開 `qnat` | 眼 `bv` | 臉 `bo` | 嘴 `rb` |
| --- | ---: | ---: | --- | --- | --- | --- | --- | --- | --- |
| 不限 | 0 | 6718 | 😆🤣😂 | 🐈😺 | 🙏🫂 | 😃😄😆 | 👁️😄😁 | 😀😃😄 | 👄😃😄 |
| 20 | 18 | 6703 | 😆🤣😂 | 🐈😺 | 🙏🫂 | 😃😄😆 | 👁️😄😁 | — | 👄😃😄 |
| 10 | 75 | 6663 | 😆🤣😂 | 🐈 | 🙏🫂 | — | 👁️ | — | 👄😃😄 |
| 6 | 148 | 6622 | 😆🤣😂 | 🐈 | 🙏🫂 | — | 👁️ | — | 👄 |
| 5 | 190 | 6592 | 😆🤣😂 | 🐈 | 🙏🫂 | — | 👁️ | — | 👄 |
| 4 | 256 | 6549 | — | 🐈 | 🙏🫂 | — | 👁️ | — | 👄 |

「笑」的關鍵字兩份合計剛好對到 5 個 emoji（😆🤣😂🥰😼），門檻 4 以下就沒有。

首選字基準（`cargo xtask bench`）：各門檻答錯數都和沒有 emoji 時相同（沒有翻盤），修正次數幾乎不隨門檻變：

| 門檻 | yue-wiki | zh-wiki | common-voice-yue | hkcancor |
| --- | ---: | ---: | ---: | ---: |
| 沒有 emoji（`3543c83` 的 baseline） | 4467 | 4505 | 4043 | 2588 |
| 不限 | 4480 | 4515 | 4055 | 2595 |
| 10 | 4480 | 4515 | 4054 | 2595 |
| 6 | 4480 | 4515 | 4054 | 2595 |
| 5 | 4479 | 4514 | 4054 | 2595 |

結論：門檻取 6。基準題很少碰到太泛的關鍵字，修正次數由「整串的字詞後面接 emoji 擠掉第 7–9 位」決定，不由門檻決定，所以門檻依抽查定：6 拿掉「打開」「眼」「臉」「嘴」後面的笑臉，留下「笑」「貓」「多謝」的 emoji，比 5 多一個餘裕，CLDR 改版給「笑」多加一個 emoji 也不會整個消失。
