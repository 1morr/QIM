# 11b: 同步層：詞級 trigram 進核心

Status: ready-for-agent
Blocked by: 11
讀: `docs/research/model-ladder.md`〈同步層〉〈實驗用的核心介面〉；`scripts/experiments/model-ladder/README.md`〈同步層〉與 `src/sync.rs`；plan §2.1（實驗的語言模型那條）、§2.3、§4.3、§4.5、§4.6；brief §9、§11；`docs/research/ios-feasibility.md`〈鍵盤擴充接上核心〉〈記憶體上限〉；spec〈共同規則〉

## What to build

把票 08 量過的詞級 trigram 做成正式的資料包與核心打分，取代實驗用的 `with_segment_model`。

- `qim-databuild` 建詞級 trigram 資料包，照 `word3:8:8` 的做法：段由現行打分切訓練句、各語料各算折扣（plan §4.3 的標準 KN）、α 夾在 1 以下；進 `build_packs`、manifest（plan §4.6）、`cargo xtask data --verify` 的決定性檢查；`LICENSE`、`NOTICE` 寫四份語料與 rime-essay（LGPL-3.0）。方案檔的 `[packs]` 指名它。
- `qim-core` 載入它，lattice 的邊與整串字詞的靜態機率改用它打分，沒有統計的地方退回字級 bigram（同實驗的接法）；做成核心內建的打分，不經 `dyn` 介面。做完刪 `Engine::with_segment_model`、`SegmentModel`、`SegmentQuery` 與 `tests/segment_model.rs`，`model-ladder` README 註明 `sync` 只在刪之前的 commit 建得起來。
- 放寬版：剪枝參數照 `word3:2:4` 另建一份，用 `ios-keyboard` 實驗的量法在 iPhone 量模型的 footprint 峰值增量：核心加現有資料包（79 MB，含字級 bigram）載入後，再載入放寬版多出的峰值。這個增量連 1.4M 的 +2 MB ≤ 60 MB（spec〈採用門檻〉的模型上限）就改成預設用放寬版，超過就留預算版（預算版也照樣量一次當對照）；結果寫進 `docs/research/ios-feasibility.md` 與 brief §12.3。要接 iPhone 時停下問使用者。
- 網頁載入新的資料包；`cargo xtask bench` 的每鍵延遲（原生與 WebAssembly）和載入時間對 spec〈採用門檻〉同步層的成本那條。
- 寫回 plan §2.1、§2.3、§4.3、§4.6、brief §9（核心加資料包的記憶體）。

## 驗收

- [ ] 詞級 trigram 資料包由 `cargo xtask data` 建出，`--verify` 通過決定性檢查，有 manifest、`LICENSE`、`NOTICE`。
- [ ] `cargo xtask bench --check` 的前後對照附在 Comments：口語首選與修正次數比開工時的基線仍過 spec〈採用門檻〉同步層那條，個人集與維基不退步；`--update-baseline` 的理由寫在同一個 commit（個人集 baseline 是共用的本機檔，更新前問使用者）。
- [ ] `with_segment_model` 一系列與它的測試已刪；plan §2.1 同步改寫。
- [ ] 放寬版的 iPhone 峰值與預設用哪一版寫進 plan §4.3、`docs/research/ios-feasibility.md`、brief §12.3。
- [ ] 每鍵延遲原生 p95 ≤ 1 ms、選開頭一段 p95 ≤ 5 ms、載入 ≤ 300 ms（這台 Mac）。
- [ ] Playwright 在網頁打一句詞級 trigram 才選對的句子、讀回首選；`cargo xtask check` 綠燈。

## Comments
