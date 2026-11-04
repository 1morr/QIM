# 03: 語料管線：中文維基與口語語料

Status: done
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

- [x] Common Voice 的結論寫進 brief §11、§12.1。
- [x] `bench/corpus/` 含中文維基 test 句、`bench/colloquial/` 含口語 test 句，各有 `ATTRIBUTION.md`。
- [x] `cargo xtask data --verify` 綠燈，每個集合列句數、轉碼失敗數、洩漏句數 0（貼輸出）。
- [x] 字頻產物存在，建兩次 sha256 相同。
- [x] `cargo xtask bench` 列出維基（粵、中）與口語集；`baseline.json` 依新集合更新並寫理由。

## Comments

- 票 01 留下的（2026-11-04）：管線用 pages-articles 自己抽內文（CirrusSearch dump 只留約 5 週，見 progress 偏差）。`pipeline.rs` 的 `WIKIS` 要求每個來源剛好一個 dump 檔，`sources::load` 把整份讀進記憶體、`--verify` 再讀一次；粵語維基 130 MB 沒問題，中文維基數 GB，要改成串流下載與解壓、只取需要的篇數。`split_of` 是全域的句子雜湊，各語料沿用它，同一句跨語料也不會一邊 train 一邊 test。

- 2026-11-04 實作：
  - 來源（查證寫進 brief §5.2、§11、§12.1，`sources.toml` 註解寫取法與抽法）：
    - 中文維基：dumps.wikimedia.org 對大站只留約 7 個月（zhwiki 20240101 已 404），鏡像站更短，XML dump 預告 2027-01 停用；能長期下載同一個檔的只有 archive.org 的 `zhwiki-20220501`。抽樣取它的第一分片 `pages-articles1`（page id 1–187712，204 MB），分片內每篇都用、依 dump 順序，其餘分片不下載。
    - Common Voice：Mozilla Data Collective 的資料集要 API key、條款禁止 re-host，不用；改用 GitHub `common-voice` 的 `server/data/{yue,zh-HK}/sentence-collector.txt`（README 與 `server/data/LICENSE` 聲明 CC0），兩檔都是粵語口語、只重疊 69 句，合成一個集合 `common-voice-yue`。
    - HKCanCor：用原始 repo `fcbond/hkcancor` 的 `data/hkcancor-utf8.zip`（CC BY 4.0，署名照 README 引用 Luke & Wong 2015），不用 PyCantonese 改過的版本；`hkcancor.rs` 只取 `<sent_tag>` 的詞接成話輪。
    - Tatoeba 不加：下載檔每週覆蓋、沒有版本化網址也沒有 Wayback 快照，釘 sha256 一週後乾淨建置就壞；CC BY 2.0 FR 要逐句列作者（433 人）。口語先靠 HKCanCor 與 Common Voice（約 5.6 萬句 train）；不夠時在票 12 請使用者決定（例如同意自存一份），plan §8 那列已改。
  - 協調者的兩項：
    - 長期重現：中文維基如上。粵語維基仍釘 dumps.wikimedia.org（小站舊 dump 目前全留），但 2027-01 XML dump 停用後可能下載不到，archive.org 也沒有 2026 的存檔；`sources.toml` 註解與 plan §8 新增一列，停用前要換取法。沒有替它建 archive.org 存檔：那是對外寫入，要先問使用者。
    - 共用快取：下載快取改到 repo 外，`QIM_CACHE_DIR` 優先，否則 `dirs::cache_dir()/qim`（macOS `~/Library/Caches/qim`），照 pip、uv 的慣例（平台快取目錄 + 環境變數覆寫）。路徑本來就含 sha256，共用不會互蓋；暫存檔名含行程 id，兩個 worktree 同時下載同一檔也不衝突。`docs/development.md`、plan §1.2、§1.4 同步。`.gitignore` 的 `/.cache/` 留著，舊 checkout 裡的殘檔才不會被誤 commit。
  - 串流：`Fetch` 改成邊收邊寫進暫存檔並算 sha256，`sources::cached` 回傳驗過的快取路徑；dump 由 `File` 邊讀邊解壓。`run` 多一個 `cache` 參數。
  - 語料清單在 `corpora.rs`（名字、來源、格式、基準目錄、署名）；每份語料讀它的來源在 `sources.toml` 列的所有檔，換 dump 日期只改 `sources.toml`。維基進 `bench/corpus/`、口語進 `bench/colloquial/`，各一份 `ATTRIBUTION.md`；`qim-bench` 依序跑兩個目錄，集合名重複就失敗。
  - 字頻：每份語料的 train 一份 `build/corpus/<名>/chars.tsv`（「字<TAB>次數」，次數多的在前、同次數依碼位），格式在 `docs/development.md`〈語料〉；怎麼合併、口語權重留給票 06、12。
  - 防洩漏改成每個基準集對所有語料的 train：bigram 用全部 train 訓練，只比自己的 train 會漏掉跨語料的洩漏。
  - `wikitext`：中文維基常見行內繁簡規則 `-{zh-hans:…;zh-hant:…}-`，原本整段丟掉會把兩邊的字黏成原文沒有的相鄰字；改成取 `zh-hk` > `zh-hant` > `zh-tw` > `zh-mo` > `zh-hans` > `zh-cn` > `zh-sg`，`H`、`T`、`-` 旗標與挑不到時留一個空白。粵語維基的基準句沒變。
  - 依賴：`qim-databuild` 加 `dirs` 7.0.0，plan §1.4 已更新。
  - 觀察：HKCanCor 在逗號處切句後多是短句（平均約 7 字），口語集的整句正確率會比維基好量；含 `𡃉` 等 HKSCS 字都在連打字集。
  - 時間（Apple M6）：`cargo xtask data` 約 35 秒，`--verify` 約 75 秒；中文維基佔一半。各語料可以分執行緒建，現在不需要，沒做。

  `cargo xtask data --verify`（2026-11-04，code-review 修正後）：

  ```text
  資料包：build/packs/quick/quick.pack
  連打字集：17611 字
  擴展區：57052 字
  速成碼：649 個
  sha256：31974c80fe810e4d0a24537acb6767346e34f40bac338d8361b44f1963e2125f
  Unihan kCangjie：比對 29121 字，差異 2744 字（其中 2246 字等於五代碼），報表 build/reports/unihan-kcangjie.tsv（不影響 verify 結果）
  語料 yue-wiki：train 2150857 句、dev 128122 句、test 90352 句（不重複），其中 2000 句進 bench/corpus/yue-wiki.tsv；含字集外字元丟掉 883432 句
  語料 zh-wiki：train 4921902 句、dev 269080 句、test 215987 句（不重複），其中 2000 句進 bench/corpus/zh-wiki.tsv；含字集外字元丟掉 1851844 句
  語料 hkcancor：train 24442 句、dev 1486 句、test 992 句（不重複），其中 992 句進 bench/colloquial/hkcancor.tsv；含字集外字元丟掉 2138 句
  語料 common-voice-yue：train 31128 句、dev 1774 句、test 1638 句（不重複），其中 1638 句進 bench/colloquial/common-voice-yue.tsv；含字集外字元丟掉 510 句
  [通過] qim-core 解得開資料包：格式版本 1
  [通過] 不掉字：連打字集 17611 / 來源 17611，擴展區 57052 / 來源 57052
  [通過] 鍵長只有 1 或 2：沒有長度 3 以上的鍵
  [通過] 抽查：anr 查不到「問」：符合
  [通過] 抽查：「問」的速成碼含 ar：符合
  [通過] 抽查：「丟」同時有 hi 與 mi：符合
  [通過] 抽查：a 的首選是「日」：符合
  [通過] 決定性：建兩次 sha256 相同：第二次 31974c80fe810e4d0a24537acb6767346e34f40bac338d8361b44f1963e2125f
  [通過] 語料轉碼：yue-wiki 失敗 0 句
  [通過] 語料決定性：yue-wiki 建兩次 sha256 相同
  [通過] 語料轉碼：zh-wiki 失敗 0 句
  [通過] 語料決定性：zh-wiki 建兩次 sha256 相同
  [通過] 語料轉碼：hkcancor 失敗 0 句
  [通過] 語料決定性：hkcancor 建兩次 sha256 相同
  [通過] 語料轉碼：common-voice-yue 失敗 0 句
  [通過] 語料決定性：common-voice-yue 建兩次 sha256 相同
  [通過] 語料不洩漏：yue-wiki test 句原樣出現在 train 0 句
  [通過] 語料不洩漏：zh-wiki test 句原樣出現在 train 0 句
  [通過] 語料不洩漏：hkcancor test 句原樣出現在 train 0 句
  [通過] 語料不洩漏：common-voice-yue test 句原樣出現在 train 0 句
  verify 通過
  ```

  另外分兩次執行 `cargo xtask data`，`bench/corpus/`、`bench/colloquial/` 與 `build/corpus/*/` 共 18 個檔的 sha256 相同；字頻 `chars.tsv`：yue-wiki `6270d94a2a5c…`、zh-wiki `065ee8bbf0a8…`、hkcancor `9cbdbaaf9f85…`、common-voice-yue `8ac57a6f66af…`。

  bench 前後對照（資料包 sha256 `31974c80fe81` 沒變，只多了集合）：

  ```text
  前（37bafdf）
  | fixed | 1 | 1（100.00%） | 100.00% | 0 | — |
  | yue-wiki | 2000 | 0（0.00%） | 0.00% | 2000 | 2000 |

  後
  | 集合 | 題數 | 整句首選正確 | 字正確率 | 答錯 | baseline 答錯 |
  | --- | ---: | ---: | ---: | ---: | ---: |
  | fixed | 1 | 1（100.00%） | 100.00% | 0 | — |
  | yue-wiki | 2000 | 0（0.00%） | 0.00% | 2000 | 2000 |
  | zh-wiki | 2000 | 0（0.00%） | 0.00% | 2000 | 2000 |
  | common-voice-yue | 1638 | 3（0.18%） | 0.02% | 1635 | 1635 |
  | hkcancor | 992 | 2（0.20%） | 0.03% | 990 | 990 |
  ```

  `--update-baseline` 加了三個新集合的答錯數（M0 整串查碼，超過 2 碼就沒候選），理由寫在 commit `f8e4c1c` 的 body；yue-wiki 不變。

- code-review（基準 `37bafdf`）已修：`sources.rs` 模組註解還寫 `.cache/sources/`、`HashingWriter` 與 `pipeline` 的 `Hashed` 重複（收成一個）、`-{zhang}-` 這種剛好含 zh 的純文字被當規則丟掉、`corpora.rs` 重複寫 `sources.toml` 已釘的檔名、development.md 的下載量 360 / 500 MB 不一致、粵語維基 2027-01 的期限沒寫在 `sources.toml`。未處理：
  - 中斷的下載留下 `.part<行程 id>`，共用快取裡不會自動清：要清就得判斷別的行程是不是還在下載；development.md 寫明可以直接刪。
  - `Corpus` 的署名五個欄位、`pipeline` 裡的 `(名字, 基準)`、`(Group, 段落)` tuple：判斷題，只在一處用，等票 06 第二個資料包來源進來時一起看。
  - 一個 commit 同時改快取、串流與語料：三者是接中文維基的同一件事（數 GB 的來源才需要串流與共用快取），沒拆。
