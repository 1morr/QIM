# s2hk 換掉了哪些字

回答的問題（M1 票 17b）：語料前處理的 OpenCC `s2hk` 在四份語料裡把哪些字換成別的字，各幾次。結論寫在票 17b 的 Comments 與 plan §4.3。

## 跑法

先跑過一次 `cargo xtask data`（只讀快取，不下載）。

```sh
cargo run --release -q -p qim-databuild --example s2hk-diff            # s2hk，各語料列前 60 種
CONFIG=t2hk cargo run --release -q -p qim-databuild --example s2hk-diff # 換成 t2hk
TOP=100000 SAMPLES=2 cargo run --release -q -p qim-databuild --example s2hk-diff  # 全列，每種附 2 個例子
ONLY='晒搵吓' SAMPLES=5 cargo run --release -q -p qim-databuild --example s2hk-diff # 只看這幾個原字
```

每份語料一段：原字、轉成的字、次數、原字在不在連打字集、轉成的字在原文出現幾次、前後各 6 字的例子。讀的是 `qim_databuild::corpora` 的 `raw_texts`（來源抽出、還沒轉換的文字），比的是 OpenCC 本身，不含前處理轉完後的字形對照表（`common_forms`，換回常見寫法）。換 OpenCC 或語料版本時要重跑。

## 上次結果（2026-11-04，ferrous-opencc 0.4.0）

| 語料 | 字數 | `s2hk` 換掉 | `t2hk` 換掉 |
| --- | --- | --- | --- |
| yue-wiki | 34,115,412 | 132,587（1,071 種） | 64,651（50 種） |
| zh-wiki | 78,999,379 | 9,489,378（2,586 種） | — |
| hkcancor | 210,892 | 596（43 種） | 81（11 種） |
| common-voice-yue | 303,395 | 2,143（120 種） | 885（26 種） |

四份語料經兩種設定都逐字對應（字數不變）。
