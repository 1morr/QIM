# 02: 首選字基準 v0

Status: ready-for-agent
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

- [ ] `cargo xtask bench` 印出 fixed 與粵語維基 test 的固定題通過數、整句首選正確率、字正確率、答錯數（貼輸出）。
- [ ] `bench/baseline.json` 進 git，是 M0 查碼上的第一份 baseline。
- [ ] `--check` 在固定題沒過或答錯數高於 baseline 時紅，有雙向測試。
- [ ] `--update-baseline` 可用；`docs/development.md` 已寫三個指令。
- [ ] `deps` 步驟對 `qim-bench` 綠燈。

## Comments
