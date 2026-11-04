# 01: 語料管線：粵語維基 tracer

Status: done
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

- [x] `data/sources.toml` 釘了粵語維基 dump（日期、sha256、授權、取用的檔案）。
- [x] `bench/corpus/` 有粵語維基 test 切分與 `ATTRIBUTION.md`；`.gitignore` 排除 train / dev 產物。
- [x] `cargo xtask data --verify` 綠燈，報告列出句數、轉碼失敗數、洩漏句數 0（貼輸出）。
- [x] 同樣的來源建兩次，`bench/corpus/` 與訓練切分的 sha256 相同。
- [x] 轉碼有單元測試：單碼字、多碼字取五代第一碼、字集外字算失敗。
- [x] `docs/development.md` 若有新指令或耗時步驟已更新。

## Comments

- 來源：開工查證 CirrusSearch dump 已搬到 `other/cirrus_search_index/`、改成分片 `.json.bz2`，只列最近 5 週；釘了一個月內就下載不到，乾淨建置會壞。照 plan §8 退回 pages-articles（20261101，sha1 與 Wikimedia 公告相符），自己抽內文（`wikitext.rs`），查證寫進 brief §12.1，plan §4.3 改寫、§8 那列刪掉。
- 含字集外字的句子：句子是標點、空白、符號之間的文字；含連打字集以外字元（英文字母、數字、擴展區的字）就整句丟掉，不拿掉個別字。拿掉字會拼出原文沒有的相鄰字，污染 bigram，也不是使用者會一口氣連打的句子。模板拿掉時留一個空白當分隔，理由相同。粵語維基約 29% 的句子因此丟掉，多半含數字或英文。
- 切分：依句子 sha256 分 train 90%、dev 5%、test 5%，同一句永遠同一切分。test 取雜湊最小的 2,000 個不重複句進 `bench/corpus/yue-wiki.tsv`（約 120 KB）；夠量出變化，又不讓 repo 變大。train / dev 在 `build/corpus/`（`/build/` 已在 `.gitignore`）。
- 格式：`bench/corpus/<名>.tsv` 一行「逐字速成碼（空格分隔）<TAB>句子」，分開寫是讓票 04 的修正次數知道每個字幾個鍵；寫在 plan §5.2 與 `docs/development.md`〈語料〉。
- 「轉碼失敗數」是留下的句子轉碼失敗的數量，依設計為 0，守的是之後改管線；因字集外字丟掉的句數另列在摘要行。
- 依賴：`qim-databuild` 加 `bzip2` 0.6.1（預設純 Rust 後端）、`quick-xml` 0.42.0、`ferrous-opencc` 0.4.0（用它內建的 `s2hk` 字典），plan §1.4 已更新。
- 觀察：train 最常見的句子是機器人小作品的固定句型（「係日本足球員」5,843 次、「位置喺埃納」793 次）；基準取不重複句，不受影響，bigram 會偏向這些句型。
- 時間（Apple M6）：`cargo xtask data` 約 27 秒，`--verify` 約 1 分鐘（語料再建一次比 sha256）。

`cargo xtask data --verify`（2026-11-04）：

```text
資料包：build/packs/quick/quick.pack
連打字集：17611 字
擴展區：57052 字
速成碼：649 個
sha256：31974c80fe810e4d0a24537acb6767346e34f40bac338d8361b44f1963e2125f
Unihan kCangjie：比對 29121 字，差異 2744 字（其中 2246 字等於五代碼），報表 build/reports/unihan-kcangjie.tsv（不影響 verify 結果）
語料 yue-wiki：train 2150857 句、dev 128122 句、test 90352 句（不重複），其中 2000 句進 bench/corpus/yue-wiki.tsv；含字集外字元丟掉 883432 句
[通過] qim-core 解得開資料包：格式版本 1
[通過] 不掉字：連打字集 17611 / 來源 17611，擴展區 57052 / 來源 57052
[通過] 鍵長只有 1 或 2：沒有長度 3 以上的鍵
[通過] 抽查：anr 查不到「問」：符合
[通過] 抽查：「問」的速成碼含 ar：符合
[通過] 抽查：「丟」同時有 hi 與 mi：符合
[通過] 抽查：a 的首選是「日」：符合
[通過] 決定性：建兩次 sha256 相同：第二次 31974c80fe810e4d0a24537acb6767346e34f40bac338d8361b44f1963e2125f
[通過] 語料轉碼：yue-wiki 失敗 0 句
[通過] 語料不洩漏：yue-wiki test 句原樣出現在 train 0 句
[通過] 語料決定性：yue-wiki 建兩次 sha256 相同
verify 通過
```

另外分兩次執行 `cargo xtask data`，`bench/corpus/` 與 `build/corpus/yue-wiki/` 四個檔的 sha256 相同（`yue-wiki.tsv` 88013bba…、`train.txt` e2172862…、`dev.txt` 1dad5f44…、`ATTRIBUTION.md` 5e09699f…）。

code-review（基準 `1cb1e17`）已修：沒收尾的 `[[` 讓後面的連結都不處理、`<br>` 判斷每個標籤複製剩餘全文（O(n²)）、多個 revision 被接在一起、寫檔錯誤丟了路徑、sha256 靠位置對應、hex 格式化重複、`Wiki.source` 改名 `source_id`、`sources.toml` 沒來源的「2017 年起都在」、票 03 指向已刪的 plan §8 列。未處理：

- 防洩漏只比整句原樣，依切分方式必為 0；子字串重疊（2,000 句中 222 句，5 字以上 55 句）記到票 11 的 Comments，沒有加檢查：plan §5.2 定義的是「原樣出現」，維基集只防退步。
- `Transcoder::new` 與 `quick::code_table` 各自合併同一個字的多列（連打字集旗標）；兩邊要的結果不同，有第三個使用者時再抽出。
- 識別符用 `bench`（`BENCH_SENTENCES`、`CorpusSummary::bench`）而非 `TopChoiceBench`：指的是 `bench/` 目錄與檔案，`TopChoiceBench` 留給票 02 的 runner。
- 中文維基要串流下載與解壓（目前整份讀進記憶體、每來源一個檔），記到票 03 的 Comments。
