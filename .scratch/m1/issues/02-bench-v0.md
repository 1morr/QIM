# 02: 首選字基準 v0

Status: done
Blocked by: 01
讀: plan §1.2（`qim-bench`、`bench/`）、§1.3、§5.2；brief §9；spec〈共同規則〉

## What to build

`cargo xtask bench` 用核心實際連打 `bench/fixed.tsv` 與 `bench/corpus/` 的每一題，印出各集合的分數，並能擋退步。先在 M0 的整串查碼上跑出第一份 baseline，數字差沒關係；之後每張改核心或資料包的票都拿它做前後對照。

- 建 `crates/qim-bench`（plan §1.3 表已列），只透過 `qim-core` 的公開介面打字（`Session::handle`），不重寫排序。
- `bench/fixed.tsv`：`輸入碼<TAB>期望首選<TAB>來源`，起始只有 `a	日`（plan §5.2）。
- 本票的指標：固定題通過數、各集合整句首選正確率、字正確率、答錯數。修正次數、跳動、分組、翻盤清單、個人集在票 04，延遲在票 05；輸出格式留好位置。
- `bench/baseline.json` 記各集合答錯數（之後加修正次數）；`--check`：固定題全過且答錯數 ≤ baseline，否則失敗；`--update-baseline` 改寫 baseline。
- 雙向測試（spec〈共同規則〉）：在暫存副本造一個退步（例如換一個會讓某題答錯的資料包或 baseline 調低）會紅；只改輸出格式、欄位順序不紅。
- 指令寫進 `docs/development.md`；plan §5.2 寫「改核心或資料包時必跑」，本票不接進 `check`（`check --fast` 的速度要看本票量到的時間，在 `## Comments` 記一筆建議）。

## 驗收

- [x] `cargo xtask bench` 印出 fixed 與粵語維基 test 的固定題通過數、整句首選正確率、字正確率、答錯數（貼輸出）。
- [x] `bench/baseline.json` 進 git，是 M0 查碼上的第一份 baseline。
- [x] `--check` 在固定題沒過或答錯數高於 baseline 時紅，有雙向測試。
- [x] `--update-baseline` 可用；`docs/development.md` 已寫三個指令。
- [x] `deps` 步驟對 `qim-bench` 綠燈。

## Comments

- 2026-11-04 實作（M0 整串查碼上的第一份 baseline）：

  ```text
  $ cargo xtask bench --check
  首選字基準（資料包 build/packs/quick/quick.pack）

  | 集合 | 題數 | 整句首選正確 | 字正確率 | 答錯 | baseline 答錯 |
  | --- | ---: | ---: | ---: | ---: | ---: |
  | fixed | 1 | 1（100.00%） | 100.00% | 0 | — |
  | yue-wiki | 2000 | 0（0.00%） | 0.00% | 2000 | 2000 |

  固定題 1/1 通過
  bench 通過
  ```

  維基句全錯是因為 M0 只整串查碼，輸入超過 2 碼就沒有候選（首選記為空字串）。`--update-baseline` 寫出 `{"yue-wiki": {"wrong": 2000}}`。在暫存副本手動驗過：`fixed.tsv` 加 `anr 門口`、或 baseline 調成 1999，`--check` 都 exit 1。
- 取捨：
  - 首選 = 逐鍵打完輸入碼後的第一個候選，不按空白；每題新的 `Session` 與空白 `UserModel`。
  - 字正確率 = 1 − Σ 編輯距離 ÷ Σ 期望字數，每題編輯距離封頂在該題字數（不讓一題扣成負分）；首選比期望長時多出的字會少算，和一般 CER 不同。定義寫在 `docs/development.md`〈首選字基準〉。
  - `--check` 比票嚴格：集合沒有 baseline、baseline 的集合這次沒跑到也算違規，免得新增或刪掉語料時安靜通過。票 03 加語料時要同 commit 跑 `--update-baseline`。
  - `fixed.tsv` 允許 `#` 註解與空行，用來在檔頭寫欄位說明。
  - 輸出是一個集合一列的 Markdown 表，可以直接貼進票或 commit body；票 04、05 的指標加成新的欄。
- `check --fast` 的建議：runner 本身打 2001 題 15 ms（M0 查碼），改 `qim-core` 後 `cargo xtask bench --check` 約 1 秒，沒改動時 0.3 秒（Apple M6）。現在夠快，但它要先有 `cargo xtask data` 建的資料包，乾淨 clone 跑 pre-commit 會失敗；建議先接進完整 `check`（`e2e` 已經要資料包），票 07、11 的 lattice 與 bigram 進來後再量一次，決定要不要進 `--fast`。
- code-review 未處理的發現：
  - `strsim` 沒列進 plan §1.4：表名是「主要依賴」，`tempfile` 等輔助依賴本來就沒列，維持不列。
  - 錯誤與違規用 `String`、`Item.label` 混放「來源」與「第 N 行」、`run` 在 `UpdateBaseline` 時兼寫檔：判斷題，現在的用途夠用，等票 04 加翻盤清單、個人集時再看要不要給型別。
  - 「只改輸出格式不紅」沒有獨立測試：`--check` 只比結構化的結果，不解析印出的報告，沒有東西可測；JSON 排版、集合順序、`fixed.tsv` 的註解與來源欄都有不紅的測試。
- 2026-11-04 協調者回報（沒重建資料包時基準會量到舊資料包，前後對照會假裝「沒變」）：`docs/development.md`〈首選字基準〉寫明改了 `data/`、`qim-databuild` 或資料包格式後先跑 `cargo xtask data`；報告的表格上方印資料包 sha256 前 12 碼，可和 `cargo xtask data` 印的完整 sha256 對開頭。沒改成自動重建：`data` 要約 50 秒，只改核心的票每次都要等；也沒比 mtime：sha256 直接回答「是不是同一份資料包」，切分支、複製檔案都不會誤判。
