# 07: 階梯實驗的框架、基線與採用門檻

Status: in-progress
Blocked by: 04, 06
讀: spec〈模型階梯實驗〉〈共同規則〉；plan §2.7、§5.2；ADR 0005；票 06 的研究筆記；`docs/research/ios-feasibility.md`〈小型神經模型〉；`.scratch/m2/issues/12-m2-close.md` Comments 的「M3」各條

## What to build

建 `scripts/experiments/model-ladder/`：之後三張實驗票都在這個框架上換模型比數字。

- 從核心匯出每題的候選與前 N 條路徑（含前文長度 0、50、200 字三種），存成實驗用的中間檔；N 與路徑長度在這裡定（M2 票 11 的 6 條、8 字是近似），定了之後重算 M2 的神經模型延遲估計並註明。
- 指標照 spec：修正次數、首選正確率、「最高到第 2 位」規則下的前 2 名命中率；依集合（維基、口語、個人、有前文）分開。
- 量出基線（現在的字級 bigram 加 `set_context`）。
- 依基線**提出**採用門檻（每層要改善多少、延遲與大小與費用的上限），停下給使用者確認；確認後寫進 spec〈採用門檻〉，之後不改。

## 驗收

- [ ] README 有日期、機器、commit、怎麼重跑、基線結果表。
- [ ] 實驗 crate 不在 workspace 裡；個人集只報數字。
- [ ] spec〈採用門檻〉已填，Comments 記使用者確認的日期。
- [ ] `cargo xtask check` 綠燈。

## Comments

- 2026-11-11 開工：`936a7cf`。`bench/personal` 用軟連結接主 checkout 的個人集。
- 2026-11-11 框架：`scripts/experiments/model-ladder/`（自成 workspace，路徑依賴 `qim-core`、`qim-bench`）。`eval` 依集合報首選正確率、前 2 名命中率、修正次數、召回；`export` 匯出不含答案的重排要求（JSONL），外部模型回 `{"set","line","ranked"}` 再 `eval --orders`；`compare` 報翻好／翻壞與 McNemar 精確檢定。首選與修正次數直接用 `qim_bench::score_set`（新開放的 `pub`，和 `cargo xtask bench` 同一份程式），修正次數的定義不在實驗裡另抄一份。
- 2026-11-11 本票內的取捨：
  - 「前 N 條路徑」從 lattice 讀，不是候選列：候選列只放 3 條多段路徑（`ranking.sentences`），口語 test 答案在候選列第一頁 54.9%、在 lattice 前 9 條 67.6%。核心沒有讀 lattice 的公開介面，加了唯讀的 `Session::paths(limit)`（路徑與分數，plan §2.1；神經分的接法要路徑分）。產品要不要用、`apply_rerank` 要不要能插入候選列以外的路徑，留給票 11。
  - 重排模型回排序的文字，不回位置；前 2 名是首選加上它排的第一個不是首選的文字。因為可以挑候選列以外的路徑，框架不經 `apply_rerank`。
  - 重排改變不了首選與修正次數（首選不動，模擬選第一頁任何一個都算一次修正），所以停鍵層與雲端層只比前 2 名命中率；寫進 spec〈模型階梯實驗〉。
  - 前文 50、200 字要有夠長的前文：`qim-databuild` 另寫 `build/corpus/<名>/{test,dev}-context-long.tsv`（「有前文」同一批題，前文換成同一段裡前面最多 200 字），基準檔不變（`cargo xtask data` 重建後 `bench/` 沒有差異）。口語的長前文很短（common-voice-yue p50 8 字、0% 到 50 字；hkcancor 17% 到 50 字），200 字那檔只在維基量得到；要量口語的長前文得把 HKCanCor 同一段對話接起來，留給票 09、10 決定。
  - 調參與報數：各票調參只看 dev（`dev.tsv` 與 dev 的長前文題），test 與個人集只在定案後跑一次；個人集不切（理由見 README〈調參與報數〉）。本票的 test 只跑了一次（基線沒有可調的參數）。
  - N = 9、路徑長度口語 p50 7 字、p95 18 字（README〈N 與路徑長度〉，用 dev 定）。
- 2026-11-11 重算 M2 的神經模型延遲：`neural-cost` 加 `NC_PATHS`、`NC_PATH_LEN`，Mac 上交替量 6×8、9×7、9×18 各 3 個程序。機器很忙（load average 15–24），只看倍數：23M q8 上下文 200 字 9×18 是 6×8 的 2.0 倍、峰值多約 15 MB，推估 iPhone p95 約 105–110 ms、峰值約 +62 MB，貼近 60 MB 上限；1.4M q8 約 18 ms。寫進 ios-feasibility、brief §12.3、neural-cost README；票 09 要在 iPhone 用 9×18 重量。第一次跑時 zsh 沒有切開 `set -- $cfg`，三次都跑成預設的 6×8，丟掉重跑。
- 2026-11-11 給票 10：DeepSeek 官方價格頁（2026-11-11 查）`deepseek-flash` 高峰時段每百萬 tokens 輸入（未命中快取）2 元、輸出 8 元，空閒時段半價；`deepseek-v4-pro` 自 2026-09-14 起轉給 V4.1-Flash、按 Flash 計費，spec 表上「v4-pro 當對照」已不成立，票 10 開工時要另選對照。
- 2026-11-11 `cargo xtask bench --check`（改了 `qim-core` 的公開介面與 `qim-databuild` 的建置輸出）：`bench 通過`，13 個集合都沒有翻盤，資料包 sha256 不變；每鍵延遲原生 p50 34.71 / p95 59.13 µs、選開頭一段 259.50 / 930.50 µs，WebAssembly（Node）42.63 / 69.67 µs、306.17 / 1055.83 µs（機器忙）。沒動學習，不附 `--learning`。

