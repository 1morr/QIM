# 03: 語料管線：中文維基與口語語料

Status: ready-for-agent
Blocked by: 01, 02
讀: plan §4.2（字頻）、§4.3、§5.2（`bench/colloquial/`）、§8（口語語料那列）；`docs/development.md`〈語料〉；brief §5.2、§11、§12.1；spec〈共同規則〉

## What to build

把中文維基與口語語料接上票 01 的管線，產出 `bench/colloquial/`、中文維基的 test 句與訓練用切分，並由 train 統計字頻（票 06 要用）。基準多了口語集，品質判斷才有主要依據（brief §9：以口語與個人集為主，維基只防退步）。

- 開工先讀完 Common Voice 粵語句子的條款，結論寫回 brief §11、§12.1（附來源）；可用才加，不可用註明。
- 中文維基 dump 很大：抽樣方式（篇數或句數、決定性的抽法）寫在 `sources.toml` 的註解，下載與解壓只取需要的部分。
- HKCanCor、Tatoeba 粵語句子：查授權與取得方式，釘進 `sources.toml`；HKCanCor 若要另外下載工具或語料庫格式特殊，抽出純句子的方式寫進註解。
- 口語集 test 進 `bench/colloquial/`（附 `ATTRIBUTION.md`，依各來源授權）；dev 切分只給票 12 調參。
- 字頻：由各 train 切分統計，輸出成票 06 能讀的建置產物；口語語料的權重這張票不定（票 12 調）。
- `--verify` 照票 01 報每個集合的句數、轉碼失敗數、洩漏句數（都要為 0）。
- `bench` 自動跑新的集合；前後對照與 `--update-baseline` 照 spec〈共同規則〉。

## 驗收

- [ ] Common Voice 的結論寫進 brief §11、§12.1。
- [ ] `bench/corpus/` 含中文維基 test 句、`bench/colloquial/` 含口語 test 句，各有 `ATTRIBUTION.md`。
- [ ] `cargo xtask data --verify` 綠燈，每個集合列句數、轉碼失敗數、洩漏句數 0（貼輸出）。
- [ ] 字頻產物存在，建兩次 sha256 相同。
- [ ] `cargo xtask bench` 列出維基（粵、中）與口語集；`baseline.json` 依新集合更新並寫理由。

## Comments

- 票 01 留下的（2026-11-04）：管線用 pages-articles 自己抽內文（CirrusSearch dump 只留約 5 週，見 progress 偏差）。`pipeline.rs` 的 `WIKIS` 要求每個來源剛好一個 dump 檔，`sources::load` 把整份讀進記憶體、`--verify` 再讀一次；粵語維基 130 MB 沒問題，中文維基數 GB，要改成串流下載與解壓、只取需要的篇數。`split_of` 是全域的句子雜湊，各語料沿用它，同一句跨語料也不會一邊 train 一邊 test。
