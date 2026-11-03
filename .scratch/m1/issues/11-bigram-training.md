# 11: bigram 訓練

Status: ready-for-agent
Blocked by: 03, 07
讀: plan §2.3、§2.5（`language_model`）、§4.3、§8（第一列）；brief §5.2、§9、§11、§12.2（`mir` 根因）；`docs/research/ranking-techniques.md`；spec〈共同規則〉

## What to build

用票 01、03 的 train 切分訓練字級 interpolated Kneser-Ney bigram，剪枝成 ≤ 8 MB 的資料包，接進 §2.3 打分的 `log P_bigram`。目標是 `mir` 的首選變成「一台」，`anr` 仍是「門口」。

- 訓練在 `qim-databuild`，決定性（同樣的語料建出同樣的資料包）；口語語料的權重先用 1，票 12 調。
- 資料包 `zh-hant-bigram`，`LICENSE` / `NOTICE` 寫語料授權（CC BY-SA 4.0 等）；是否須同授權散布仍是 brief §11 的待決項，`NOTICE` 不下結論。
- 核心讀 bigram 包，打分改成 plan §2.3 的完整式子（`personal` 留到票 13）；查詢延遲要在票 05 的量測下報出來。
- `mir` 首選若不是「一台」：照 plan §8 第一列，記偏差並停下問使用者要改詞級 bigram 還是其他資料，不硬調參數湊答案。
- 通過後 `mir	一台` 加進 `fixed.tsv`。
- `--verify` 加 bigram 包大小 ≤ 8 MB。

## 驗收

- [ ] `fixed.tsv` 含 `mir	一台`，`anr	門口` 仍過；`cargo xtask bench --check` 綠燈。
- [ ] bigram 資料包 ≤ 8 MB，`--verify` 檢查它；建兩次 sha256 相同。
- [ ] 核心測試：bigram 讓同碼的兩條路徑依前後字改變名次（測試資料包）。
- [ ] bench 前後對照（含延遲）貼在 `## Comments`。

## Comments
