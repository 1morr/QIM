# 08: 階梯：同步層的 trigram 與詞級 n-gram

Status: done
Blocked by: 07
讀: spec〈模型階梯實驗〉〈採用門檻〉〈共同規則〉；plan §2.3、§4.3；brief §12.2（librime-octagram）；`docs/research/cyime.md` §3；`.scratch/m1/issues/24-m1-close.md` Comments 的「M3」各條（Common Voice、KN 那條）

## What to build

量同步層換語言模型值不值得。

- 字級 trigram、詞級 n-gram：同一份預算（≤ 8 MB）比較，再看放寬到 M2 上限（峰值增量 ≤ 60 MB）的增益；另量一份不設預算、可選下載的詞級 trigram。
- 同時看 M1 留下的兩件事：KN 在乘權重的次數上算、口語幾乎不平滑（比照標準 KN 重訓比一次）；Common Voice 因「說」「記」同碼 `yu` 多錯的題換模型後有沒有改善。
- 每個模型記大小、載入時間、Mac 上每鍵延遲 p50 / p95，與 spec〈採用門檻〉比。

## 驗收

- [x] README 的結果表每列一個模型：各集合的三個指標、大小、延遲；標出過不過門檻。
- [x] 訓練用的語料與授權註明（brief §11）。
- [x] `cargo xtask check` 綠燈。

## Comments

- 2026-11-11 開工：`fa07bd3`。`bench/personal` 用軟連結接主 checkout 的個人集；`cargo xtask data` 前先拿掉軟連結（同票 07），重建後接回。協調者補充：個人集 baseline 是共用的本機檔，這張票不跑 `--update-baseline`，量到翻盤就記下回報；本票沒有跑。
- 2026-11-11 怎麼量：核心加實驗用的 `Engine::with_segment_model`（`SegmentModel`、`SegmentQuery`，plan §2.1），lattice 的邊與整串字詞的靜態機率改問外部模型，首選、修正次數與延遲都由核心的 Viterbi 量，不另寫一份解碼。沒掛上時打分不變（`tests/segment_model.rs` 三個測試，兩個做過變異確認會紅）。`qim-bench` 公開 `native_timing` 與 `latency_inputs`，延遲和 `cargo xtask bench` 同一份計時。模型與命令列 `sync` 在 `scripts/experiments/model-ladder/src/sync.rs`、`sync_cli.rs`，結果在該 README〈同步層〉。
- 2026-11-11 本票內的取捨：
  - 高一階的模型都「退回現行打分」：沒有統計的地方和基線一樣，增益只來自留下的 n-gram，比較的是同一把尺上多加一階。
  - 詞級的段用現行打分切訓練句（文字已知的 Viterbi），和解碼的切法一致；不用 HKCanCor 的人工分詞（只有口語一小部分）。
  - 預算比的是「新模型加字級 bigram 的 3.02 MB」≤ 8 MB；放寬版的常駐留 15 MB 以上餘裕給估計誤差，所以選 `word3:2:4`（35 MB），不選 `word3:2:2`（58.8 MB）。
  - 標準 KN 先試「分開訓練、線性內插」（`mix`），任何 λ 不是口語差就是維基差；改成次數相加、各語料各算 D（`kn2`），`kn2:2:w` 重現了現行的字級 bigram（dev 口語 38.6% / 2.279 對 2.280，資料包 3,018,901 對 3,018,934 位元組），算法可信。
- 2026-11-11 dev 上的 bug：第一次掃 `char3` 時給前文比不給差、首字變罕用字；罕用字當前文時重算的 α 爆到 e^13 以上（低一階的質量幾乎全給了留下的後字，分母近 0、又有量化誤差）。先寫會紅的測試（`backoff_never_raises_an_unseen_event_above_the_lower_order`，第一版沒紅，改成各語料折扣的資料後紅），α 夾在 1 以下後修好。修正前的數字不用。
- 2026-11-11 調參只看 dev（README〈同步層〉的 dev 表，24 個設定）；test 與個人集在定案後跑一次（2026-11-11 11:57–12:09，7 個設定一個程序跑完）。之後的改動（code-review 的改名、去重、`Lookup::or_lower`、clippy 的型別別名）不改行為：dev 重跑 `char3:10`、`word3:8:8`、`kn2:2` 和定案前的數字相同。
- 2026-11-11 結果（test，對 spec〈採用門檻〉；細節與 McNemar 在 README）：
  - 預算版：字級 trigram `char3:10`（7.57 MB）口語首選 36.3% → 42.3%、修正 2.421 → 2.162；詞級 trigram `word3:8:8`（7.84 MB）44.5%、2.046，比 `char3:10` 也再過門檻（+2.2 點、−5.4%、p = 0.0004）。兩者成本都過（每鍵 p95 0.1 ms 級、選開頭一段 1.9 ms、解開 ≤ 14 ms）。
  - 放寬版：`char3:2` 比 `char3:10` +2.1、`word3:2:4` 比 `word3:8:8` +2.2，都剛好過；採用 `word3:8:8` 時 `char3:2` 不過。`word3:2:4` 常駐 35 MB 是估計，要在 iPhone 量。
  - 不設預算 `word3:1:1`（149 MB）：比基線 +12.0 點過「≥ 4.0」那條；比 `word3:2:4` 只 +1.6，照「較貴的一級再過門檻」不過。兩條的先後交票 11。
  - KN：`kn2:2` 口語首選 −0.1、修正 +2.0%（p = 0.85），字級 bigram 照舊；高一階要用各語料各算的折扣（現行做法 dev 口語首選低 1.6–3.5 點）。plan §4.3 改寫。
  - 說／記：test 基線只剩 3 處（期望「記」首選「說」），各模型 2–4 處，n-gram 沒有改善，也已不是主要的錯。
  - 個人集（只報數字）：首選翻好／翻壞 基線 → `char3:10` 18/3、`char3:2` 24/8、`word3:8:8` 26/27、`word3:2:4` 36/27、`word3:1:1` 38/26、`kn2:2` 4/9；`char3:10` → `word3:8:8` 16/32（p = 0.029），個人首選都在不退步的 0.5 點內。
- 2026-11-11 `cargo xtask bench --check`（改了 `qim-core` 與 `qim-bench`）：`bench 通過`，固定題全過，13 個集合（含個人集）都沒有翻盤、答錯與修正次數和 baseline 相同，資料包 sha256 不變（沒掛模型時打分不變）。每鍵延遲原生 p50 39.0 / p95 60.1 µs、選開頭一段 263 / 889 µs，WebAssembly 45.8 / 65.3 µs、302 / 1013 µs（機器忙，load average 9–15）。沒動學習，不附 `--learning`。
- 2026-11-11 code-review（`fa07bd3` 起的未提交改動；Standards 讀 `CLAUDE.md`、全域偏好、plan §0–§2.1、§5.2、`GLOSSARY.md`、`docs/development.md`、spec〈共同規則〉，Spec 讀本票與 spec〈模型階梯實驗〉〈採用門檻〉〈共同規則〉）。Spec 軸逐列比對了 README 與原始輸出、自己重算了 McNemar，數字都對。已處理：
  - README：test 表補維基修正/句、另加有前文表（前文 0、50）；dev 表補解開時間；`word3:1:1` 明寫「較貴一級那條不過」、`char3:2` 寫明採用 `word3:8:8` 時不過；「載入」註明是解開、不含讀檔，基線的 2.7 ms 引〈基線結果〉，`word3:1:1` dev 306 ms 也列；個人集 `char3:10` → `word3:8:8` 的 16/32 寫出；除錯用的 `SHOW`、`EXPLAIN` 環境變數寫進〈怎麼重跑〉。
  - 四個模型的 `log_prob` 補測試（`tests/sync.rs`，檔頭改成和內容相符）；`CharTrigram::explain` 的註解改指 `EXPLAIN`。
  - 改名：`SegmentQuery::bigram` → `current_log_prob`（和 `edge_score` 的個人 bigram 參數撞名）、`Static` → `CurrentScoring`；`native` 直接改名 `native_timing`，不在 re-export 改名。
  - 去重：`count` 改呼叫 `merge`；`Lookup::or_lower` 收四處退回的 match；字級鍵的 21 位元收成 `CHAR_BITS`；`qim_bench::run` 不再讀兩次固定題與語料（`inputs_of`）。刪掉沒人呼叫的 `shared`、`count_words` 只跑一次的迴圈；`MixedBigram` 的常駐改成「沒估」，不印 0；`main.rs` 的 `mod` 位置。
  - plan §4.3 的 KN 那句改成「採用的話用標準 KN」，不替票 11 下結論。
- 2026-11-11 沒處理的發現：
  - 核心加的公開介面（`with_segment_model`）是範圍外：為了量真的 Viterbi，plan §2.1 寫明票 11 定去留，不留就連同 `tests/segment_model.rs` 刪。
  - 結論沒寫 `docs/research/`、沒摘進 brief〈查證過的事實〉：spec〈模型階梯實驗〉的產出規定由結論票（票 11）寫回 brief §12、§1.5、§11；本票的結果在 README。brief §11 只補一句詞級模型另受 LGPL 影響。
  - `sync_command` 每種模型「訓練、印統計、量成本、評估」四段相似、`base_model` 與 `vocabulary` 重讀資料包：實驗的命令列，收成表驅動不會更好讀。
  - `Engine::span` 自己算 `before.segment()`，不用 `Lattice` 存的那份：`span` 收的是 `Preceding`，不經 lattice。
  - 延遲在忙碌的機器上量（同一次的 `bigram` p95 130.9 µs，空著時 44 µs）；離門檻一個數量級，沒重量。

