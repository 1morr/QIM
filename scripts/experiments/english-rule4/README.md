# 規則 4 的英文候選出現幾題

回答的問題（M4 票 22）：英文資料包從 ESDB 的 `scowl-en-us`（85,959 詞）換成 AOSP LatinIME 的 `aosp-en-us`，詞多了，中文狀態第 2 位被英文候選（plan §2.4 規則 4）搶走的題會不會變多？`cargo xtask bench --check` 只看首選與修正次數，看不到這個數。

## 跑法

先跑過 `cargo xtask data`（讀 `build/packs/` 與 `bench/`）。

```sh
cargo run --release -q -p qim-bench --example english-rule4              # 用 build/packs 的英文資料包
cargo run --release -q -p qim-bench --example english-rule4 words.txt    # 換成一行一詞的詞表
LIST=1 cargo run --release -q -p qim-bench --example english-rule4       # 另在 stderr 列每題的英文詞與頻率
```

`LIST=1` 的頻率一律查 `build/packs` 的英文資料包；個人集只列長度與頻率，不列詞。

每題用新的 `Session` 與空白的 `UserModel`、預設選項，照首選字基準的鍵打完整題（打錯碼與觸控的題按實際按的鍵），讀 `Session::inspect` 的候選種類，數第一頁有英文候選的題、英文候選在第 2 位的題。集合是維基、口語、打錯碼、觸控與個人集（有的話，只報數字）；「有前文」的題和口語是同一批語料，不另算。只數整題打完時，不數修正途中剩下的輸入碼。

核心只讀現在的資料包格式。換資料包前的 `scowl-en-us` 是格式 8，用 `dump_pack_v8.py` 轉成一行一詞再給第二種跑法：

```sh
python3 -I scripts/experiments/english-rule4/dump_pack_v8.py <舊的 scowl-en-us.pack> > words.txt
```

## 結果

2026-11-15，Apple M6、macOS 27.0.1、rustc 1.99.1，`594aa9b` 之上（票 22 未 commit 的工作區）。前：`594aa9b` 建的 `scowl-en-us`（85,959 詞）轉成詞表；後：LatinIME `127336e` 建的 `aosp-en-us`（156,636 詞）。其餘資料包相同。

| 集合 | 題數 | 前：有英文候選 | 前：第 2 位 | 後：有英文候選 | 後：第 2 位 |
| --- | ---: | ---: | ---: | ---: | ---: |
| yue-wiki | 2000 | 0 | 0 | 0 | 0 |
| zh-wiki | 2000 | 2 | 2 | 2 | 2 |
| common-voice-yue | 1649 | 1 | 1 | 2 | 2 |
| hkcancor | 989 | 1 | 1 | 1 | 1 |
| common-voice-yue·打錯 | 1649 | 1 | 1 | 2 | 2 |
| hkcancor·打錯 | 989 | 1 | 1 | 1 | 1 |
| common-voice-yue·觸控 | 1649 | 0 | 0 | 1 | 0 |
| hkcancor·觸控 | 989 | 1 | 0 | 1 | 0 |
| personal | 4517 | 47 | 47 | 77 | 77 |
| 合計 | 16431 | 54 | 53 | 87 | 85 |

- 公開集合只多一題：common-voice-yue 第 1108 行打出 `METV`（全大寫的縮寫，AOSP 的 `f=30`），同一題在打錯與觸控的集合也出現。原有的 `able`、`heed`、`KGB`、`goat` 前後都在。
- 個人集多 30 題（3 個字母 21 題、4 個字母 9 題），前後的（長度, 頻率）清單相減，多出來的英文詞頻率在 18–89 之間（AOSP 的 `f`，常用詞如 `the` 是 222、`receive` 137），沒有一題 ≥ 90：ESDB size 60 不收、LatinIME 收的冷僻詞與縮寫。原因是 LatinIME 的詞表大（依小寫歸併後 156,636 詞，ESDB 85,959 詞），速成 3–4 個字母的碼撞到它的機會多。
- 英文候選不搶首選，`cargo xtask bench --check` 前後各集合的答錯數與修正次數相同（個人集修正次數少 1 次）。被擠的是第 2 位：上面「第 2 位」那欄的題，原本第 2 位的中文候選往後一格。
- 若規則 4 只收頻率夠高的詞，後的 87 題裡頻率 ≥ 60 的 66 題、≥ 90 的 27 題（前是 54 題）。這會改 brief〈英文模式與欄位類型〉「拼對的完整單字就出」的規則，本票不改，留給使用者決定。
