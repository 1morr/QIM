# llm-rerank：Foundation Models 與雲端 LLM 的重排（M3 票 10）

- 日期：2026-11-11；機器：Apple M6（Mac18,5，32 GB）、macOS 27.0.1（26A434）；Swift 6.4.2；Python 3（只用標準函式庫）。
- commit：開工 `fa07bd3`。README 不寫含它自己的 commit，之後重跑用 `git log -- scripts/experiments/llm-rerank` 對。
- 問題（`.scratch/m3/spec.md`〈模型階梯實驗〉）：停鍵層的 Apple Foundation Models、雲端層的 DeepSeek 重排，前 2 名命中率能補多少，延遲與費用是否過〈採用門檻〉。框架、基線與指標定義在 `../model-ladder/README.md`。

## 內容

| 路徑 | 內容 |
| --- | --- |
| `llm.py` | 組提示（`prompts`）、送 DeepSeek（`cloud`）、回覆換回文字（`orders`）、延遲與費用（`stats`、`tail-stats`）、費用估計（`estimate`）、只比調參那批題（`subset`） |
| `test_llm.py` | `llm.py` 的測試：在本目錄 `python3 -m unittest test_llm` |
| `fm/fm-rerank.swift` | Foundation Models 的命令列：逐題送出（`run`）、停頓後第一個請求的延遲（`tail`） |
| `score.sh` | 一種寫法的回覆打分：`orders` → `model-ladder eval` → `subset` |

## 怎麼重跑

在 repo 根目錄。輸出都在 `build/llm-rerank/`，不進 git。

```sh
cargo xtask data                                       # 題目與長前文題
ln -s <主 checkout>/bench/personal bench/personal      # worktree 裡要個人集時
(cd scripts/experiments/model-ladder && cargo run --release -- eval && cargo run --release -- export)
swiftc -O -swift-version 6 scripts/experiments/llm-rerank/fm/fm-rerank.swift -o build/llm-rerank/fm-rerank
build/llm-rerank/fm-rerank info                        # 要是 available
L=scripts/experiments/llm-rerank/llm.py; S=scripts/experiments/llm-rerank/score.sh
R=build/model-ladder/dev/requests; T=build/llm-rerank/tune; mkdir -p $T build/llm-rerank/fm build/llm-rerank/flash

# dev 調參：每種寫法（<st> 是 one、two、hint、hint2，<ch> 是 cand、merge、paths）同一批 500 題
#   口語 dev 不給前文 300 題 + 口語前文 200 字 200 題，種子 1
python3 $L prompts --style <st> --choices <ch> --sample 300 $R/common-voice-yue.jsonl $R/hkcancor.jsonl -o $T/plain-<st>-<ch>.jsonl
python3 $L prompts --style <st> --choices <ch> --sample 200 "$R/common-voice-yue·前文200.jsonl" "$R/hkcancor·前文200.jsonl" -o $T/ctx-<st>-<ch>.jsonl
cat $T/plain-<st>-<ch>.jsonl $T/ctx-<st>-<ch>.jsonl > $T/<st>-<ch>.jsonl
build/llm-rerank/fm-rerank run $T/<st>-<ch>.jsonl build/llm-rerank/fm/tune-<st>-<ch>.jsonl
(set -a; . ~/.config/qim/deepseek.env; python3 $L cloud --workers 6 --model deepseek-flash $T/<st>-<ch>.jsonl build/llm-rerank/flash/tune-<st>-<ch>.jsonl)
$S dev $T/<st>-<ch>.jsonl build/llm-rerank/fm/tune-<st>-<ch>.jsonl fm-tune-<st>-<ch>        # 印出和基線比的表
python3 $L stats build/llm-rerank/fm/tune-<st>-<ch>.jsonl

# test：定案的寫法各跑一次（調參只看 dev，README〈調參與報數〉）
(cd scripts/experiments/model-ladder && cargo run --release -- eval --split test && cargo run --release -- export --split test --include-personal)
P=build/model-ladder/test/requests; O=build/llm-rerank/test; mkdir -p $O
ls $P | grep -v personal | sed "s|^|$P/|" | tr '\n' '\0' | xargs -0 python3 $L prompts --style hint --choices paths -o $O/cloud-hint-paths.jsonl
ls $P | sed "s|^|$P/|" | tr '\n' '\0' | xargs -0 python3 $L prompts --style one --choices paths -o $O/fm-one-paths.jsonl
(set -a; . ~/.config/qim/deepseek.env; python3 $L cloud --workers 6 --model deepseek-flash $O/cloud-hint-paths.jsonl build/llm-rerank/flash/test-hint-paths.jsonl)
build/llm-rerank/fm-rerank run $O/fm-one-paths.jsonl build/llm-rerank/fm/test-one-paths.jsonl
$S test $O/cloud-hint-paths.jsonl build/llm-rerank/flash/test-hint-paths.jsonl flash-hint-paths 3000
$S test $O/fm-one-paths.jsonl build/llm-rerank/fm/test-one-paths.jsonl fm-one-paths
(cd scripts/experiments/model-ladder && cargo run --release -- compare ../../../build/model-ladder/test/baseline ../../../build/model-ladder/test/flash-hint-paths)
python3 $L stats --by context --model deepseek-flash build/llm-rerank/flash/test-hint-paths.jsonl
python3 $L stats --by context build/llm-rerank/fm/test-one-paths.jsonl

# Foundation Models 停頓後第一個請求：機器上沒有別的 fm-rerank 在跑時量
build/llm-rerank/fm-rerank tail $O/fm-one-paths.jsonl build/llm-rerank/fm/tail.jsonl --gaps 5,10,20,30,60,120 --repeat 8
for i in 1 2 3 4 5 6; do sleep 60; build/llm-rerank/fm-rerank tail $O/fm-one-paths.jsonl build/llm-rerank/fm/tail-cold.jsonl --gaps 0 --repeat 1; done
python3 $L tail-stats build/llm-rerank/fm/tail.jsonl
python3 $L tail-stats build/llm-rerank/fm/tail-cold.jsonl
```

- `cloud` 沒有 `DEEPSEEK_API_KEY` 時印一行就結束（跳過雲端）；拒收個人集的提示，除非加 `--allow-personal`（每次先問使用者，plan §5.2）。續跑時已有回答的題不重送，出錯的列丟掉重送。`fm-rerank run` 的出錯（安全過濾）每次都一樣，算做過、不重送。
- 調參那批題只是 dev 的一部分，`eval` 的各集合列會把沒送的題照原順序算進去，所以 `score.sh` 最後用 `subset` 只比提示檔裡的題（下面〈dev 調參〉）。
- 付費：每次真的呼叫前把估計的請求數與費用告訴使用者（spec〈共同規則〉）；`estimate` 用字數保守估，第一批送完改用 `stats --model` 的實測每千次費用。

## 模型與環境

| 模型 | 版本 | 怎麼呼叫 |
| --- | --- | --- |
| Apple Foundation Models | AFM 3 Core Advanced（`SystemLanguageModel.default`，`contextSize` 8192），macOS 27.0.1 | `fm-rerank`：每題新開 `LanguageModelSession(instructions:)`、`respond(to:options:)`，`samplingMode: .greedy`、`maximumResponseTokens: 8`，逐題依序送 |
| DeepSeek `deepseek-flash` | DeepSeek-V4.1-Flash-1017（brief §12.4，2026-11-11 查） | `llm.py cloud`：`POST https://api.deepseek.com/chat/completions`，`"thinking": {"type": "disabled"}`、`temperature: 0`、`max_tokens: 8`，6 個並行，逾時 30 s；從這台 Mac 送（時區 UTC+8） |

- 開工時 Foundation Models 回 `modelNotReady`：系統設定寫「Apple Intelligence 需要 Mac 和 Siri 設為相同的語言」（Mac 中文（台灣）、Siri 中文（香港））。使用者把兩者改成一致後變成 `available`。
- `deepseek-v4-pro` 沒量：使用者只同意 flash 的費用（約 11 元）。票 07 記的「`deepseek-v4-pro` 已轉給 V4.1-Flash」與 2026-11-11 的官方文件不符（轉走的是舊名 `deepseek-v4-flash`，V4 Pro 9 月 14 日後繼續提供），要量 pro 得另外問費用（官方價約 flash 的 4.5 倍）。
- 量時機器很忙（別的 session 在編譯，load average 8–10），Foundation Models 的延遲偏高的可能性要記著；雲端的延遲主要是網路與伺服器。

## 提示

spec 的寫法：上下文 + 輸入碼 + 編號的選項，只回序號（afm-ime）。user 訊息固定這個格式，沒有前文時寫「（無）」：

```text
上下文：我想，討飯一樣的人，
輸入碼：pdmujshiii
候選：1.也配考特麼 2.也配窄的魔 3.也配勃特麼 4.也配窄的底 5.也配麪特麼 6.也配窄的冷 7.也配協的魔 8.也配考的魔 9.也配麪我麼
序號：
```

指示（system / `instructions`）四種，全文在 `llm.py` 的 `STYLES`：

| 名 | 全文 |
| --- | --- |
| `one` | 你是中文輸入法的候選排序器。根據上下文，從候選中選出最可能接在上下文後面的一個，只回答它的序號（阿拉伯數字），不要其他文字。 |
| `two` | 你是中文輸入法的候選排序器。根據上下文，從候選中選出最可能接在上下文後面的兩個，最可能的在前，只回答兩個序號（阿拉伯數字，以逗號分隔），不要其他文字。 |
| `hint` | 你是速成輸入法的候選排序器。使用者用速成碼打了一句話，候選是同一串輸入碼的不同轉換，內容可能是粵語口語或書面中文，通常是香港繁體字。根據上下文與語感，選出使用者最可能想打的一個，只回答它的序號（阿拉伯數字），不要其他文字。 |
| `hint2` | 同 `hint`，最後一句改成「選出使用者最可能想打的兩個，最可能的在前，只回答兩個序號（阿拉伯數字，以逗號分隔），不要其他文字。」 |

`one` 是 M2 票 10 在 iPhone 上用的那段。選項三種（`llm.py` 的 `choices`）：

- `cand`：spec 原寫法，候選列第一頁（9 個，依核心的順序）。
- `merge`：前 9 條 lattice 路徑在前，再補沒出現過的候選，取前 10 個（spec 的「前 10 個」）。
- `paths`：只給前 9 條路徑（N = 9，票 07）。

第一頁只放 3 條整句，其餘 6 個是開頭幾個字的短詞，整句的答案不會是它們。給模型看的選項裡有沒有答案（上限），dev：

| 題 | 題數 | 核心前 2 個候選（基線的前 2 名） | `cand` | `merge` | 聯集不截 | `paths` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 口語 dev 全部（不給前文） | 2680 | 51.3% | 56.7% | 67.9% | 67.9% | 67.9% |
| 口語 dev 前文 200 全部 | 988 | 50.3% | 56.4% | 68.2% | 68.2% | 68.2% |
| 調參的 500 題 | 500 | 51.4% | 57.4% | 67.6% | 67.6% | 67.6% |

只給第一頁時，模型就算每題都挑對，前 2 名最多也只多約 6 點，雲端門檻要 +6.0 點；給路徑的三種上限相同（候選與路徑的聯集不截到 10 個，多出的都是短詞）。

## dev 調參

題：口語 dev 不給前文的 300 題（`common-voice-yue`、`hkcancor` 合起來抽）加口語 dev 前文 200 字的 200 題，種子 1，每種寫法同一批 500 題。前 2 名照框架的定義（首選不動，第 2 名是模型排的第一個不是首選的文字）；「好/壞」是對基線翻好、翻壞的題數，p 是 McNemar 精確檢定；「可動首選」是模型自己的第一名答對的比例（ADR 0005 不讓重排動首選，只當參考）。出錯（Foundation Models 的安全過濾）與沒有序號的題照原順序。

先用 `one` 比三種選項，再拿最好的選項比其他指示寫法。DeepSeek 只跑這兩段（3,000 次）；Foundation Models 免費，`cand`、`merge` 配 `two`、`hint`、`hint2` 的 6 種沒跑完就停了（三種選項已用 `one` 比過，`paths` 明顯較好）。

| 模型 | 寫法 | 前 2 名（基線 51.4%） | 好/壞 | p | 可動首選（基線 37.6%） | 出錯 | p50 ms | p95 ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| DeepSeek flash | `one-cand` | 47.4% | 11/31 | 0.003 | 27.2% | 0 | 777 | 1087 |
| DeepSeek flash | `one-merge` | 49.4% | 28/38 | 0.268 | 29.6% | 0 | 739 | 1049 |
| DeepSeek flash | `one-paths` | 58.6% | 43/7 | 0.000 | 52.4% | 0 | 752 | 1093 |
| DeepSeek flash | `two-paths` | 58.2% | 43/9 | 0.000 | 49.0% | 0 | 758 | 1081 |
| DeepSeek flash | `hint-paths` | **60.0%** | 48/5 | 0.000 | 55.8% | 0 | 743 | 1173 |
| DeepSeek flash | `hint2-paths` | 60.0% | 50/7 | 0.000 | 53.8% | 0 | 758 | 1095 |
| Foundation Models | `one-cand` | 48.4% | 7/22 | 0.008 | 19.0% | 11 | 369 | 854 |
| Foundation Models | `one-merge` | 52.4% | 19/14 | 0.487 | 23.8% | 11 | 424 | 835 |
| Foundation Models | `one-paths` | **53.0%** | 22/14 | 0.243 | 26.4% | 12 | 445 | 1046 |
| Foundation Models | `two-paths` | 51.4% | 8/8 | 1.000 | 38.0% | 13 | 479 | 1136 |
| Foundation Models | `hint-paths` | 52.0% | 15/12 | 0.701 | 27.4% | 41 | 439 | 1032 |
| Foundation Models | `hint2-paths` | 52.8% | 12/5 | 0.143 | 37.4% | 37 | 465 | 1083 |

- 只給第一頁（`cand`）兩個模型都比基線差：模型常挑開頭幾個字的短詞（flash 試送的 20 題有 11 題回最後一個選項），那個短詞頂替了核心原本的第 2 名，而它不可能是整句的答案。`merge` 也混了短詞，同樣沒有好處。
- 定案：flash 用 `hint-paths`（和 `hint2-paths` 同分，翻壞較少、只要一個序號）；Foundation Models 用 `one-paths`（前 2 名最高、出錯最少）。test 各只跑定案的那一種。
- Foundation Models 的出錯都是安全過濾（`May contain unsafe content` 一類），`hint` 類寫法被擋得多；它自己的第一名比核心的首選還差（26.4% 對 37.6%）。
- DeepSeek 調參花的錢：3,020 次（含試送 20 次），實測每千次 0.32–0.42 元。

## test 結果（採用門檻看這份）

設定在 dev 定案後各跑一次：DeepSeek flash `hint-paths` 在 2026-11-11 11:52–12:24（北京時間；高峰時段到 12 時，之後是離峰）送 test 的 15,713 題；Foundation Models `one-paths` 跑 test 的 20,230 題（含個人集，只在本機）。基線在同一個 commit 重量，和 `../model-ladder/README.md` 的數字相同。

### DeepSeek flash（`hint-paths`）

| 組 | 題數 | 前 2 名：基線 | 前 2 名：flash | 差 | 可動首選：基線 → flash |
| --- | ---: | ---: | ---: | ---: | ---: |
| 口語 | 2638 | 49.1% | **59.5%** | +10.4 | 36.3% → 54.6% |
| 維基 | 4000 | 55.1% | 65.6% | +10.5 | 43.8% → 62.0% |
| 個人 | 4517 | 85.5% | 沒量 | | |
| 有前文·口語·前文0 | 1025 | 48.8% | 56.7% | +7.9 | 35.3% → 50.3% |
| 有前文·口語·前文50 | 1025 | 50.1% | 60.4% | +10.3 | 36.8% → 53.8% |
| 有前文·口語·前文200 | 1025 | 50.1% | 60.5% | +10.4 | 36.8% → 54.5% |
| 有前文·維基·前文0 | 2000 | 55.2% | 65.8% | +10.6 | 43.5% → 62.0% |
| 有前文·維基·前文50 | 2000 | 55.7% | 70.1% | +14.4 | 43.4% → 68.7% |
| 有前文·維基·前文200 | 2000 | 55.7% | 70.4% | +14.7 | 43.4% → 69.3% |
| 有前文·前文50 | 3025 | 53.8% | 66.8% | +13.0 | 41.2% → 63.6% |
| 有前文·前文200 | 3025 | 53.8% | 67.0% | +13.2 | 41.2% → 64.3% |

首選與修正次數不變（重排不動首選，見 `../model-ladder/README.md`〈指標〉）。前 2 名的翻好／翻壞（`model-ladder compare`）：`common-voice-yue` 213/8、`hkcancor` 88/17（口語合計 301/25，p < 0.001）、`yue-wiki` 225/9、`zh-wiki` 219/14。

個人集沒送：使用者 2026-11-11 決定不把個人集送 DeepSeek（plan §5.2）。

延遲與費用（`stats --by context --model deepseek-flash`；每千次照高峰價算，spec〈採用門檻〉照高峰價）：

| 前文 | 請求 | p50 ms | p95 ms | 最慢 ms | 每千次（元） |
| --- | ---: | ---: | ---: | ---: | ---: |
| 不給前文（維基、口語集） | 6638 | 734 | 1062 | 1695 | 0.414 |
| 前文 0 | 3025 | 730 | 1087 | 2055 | 0.381 |
| 前文 50 | 3025 | 750 | 1095 | 2397 | 0.438 |
| 前文 200 | 3025 | 764 | 1094 | 2069 | 0.509 |
| 合計 | 15713 | 742 | 1083 | 2397 | 0.430 |

沒有出錯、沒有不含數字的回覆，超過 3 s 的 0 題（超過 2 s 的 3 題）。快取命中只有 2.5%（84,509 / 3,400,529 個輸入 token）：每題的 system 訊息相同，但 DeepSeek 的快取要偵測到共同前綴才建，命中是盡力而為。照高峰價算 test 是 6.76 元，加上調參約 1.1 元，合計約 7.9 元；12 時後送出的部分實付半價，實際花費較少。

### Foundation Models（`one-paths`）

2026-11-11 11:59–14:18 跑（當時機器 load average 約 10）。

| 組 | 題數 | 前 2 名：基線 | 前 2 名：FM | 差 | 可動首選：基線 → FM |
| --- | ---: | ---: | ---: | ---: | ---: |
| 口語 | 2638 | 49.1% | **50.4%** | +1.3 | 36.3% → 22.3% |
| 維基 | 4000 | 55.1% | 59.1% | +4.0 | 43.8% → 26.5% |
| 個人 | 4517 | 85.5% | 86.1% | +0.6 | 76.0% → 24.2% |
| 有前文·口語·前文0 | 1025 | 48.8% | 49.7% | +0.9 | 35.3% → 21.2% |
| 有前文·口語·前文50 | 1025 | 50.1% | 52.0% | +1.9 | 36.8% → 26.6% |
| 有前文·口語·前文200 | 1025 | 50.1% | 52.4% | +2.3 | 36.8% → 26.7% |
| 有前文·維基·前文0 | 2000 | 55.2% | 59.1% | +3.9 | 43.5% → 26.2% |
| 有前文·維基·前文50 | 2000 | 55.7% | 64.3% | +8.6 | 43.4% → 43.6% |
| 有前文·維基·前文200 | 2000 | 55.7% | 64.7% | +9.0 | 43.4% → 44.1% |
| 有前文·前文50 | 3025 | 53.8% | 60.2% | +6.4 | 41.2% → 37.9% |
| 有前文·前文200 | 3025 | 53.8% | 60.5% | +6.7 | 41.2% → 38.2% |

前 2 名的翻好／翻壞：`common-voice-yue` 47/18、`hkcancor` 29/22（口語合計 76/40，p = 0.001）、`yue-wiki` 82/17、`zh-wiki` 110/14、個人 83/58（p = 0.043）。292 題（1.4%）被安全過濾擋下、照原順序。它自己的第一名遠比核心的首選差（口語 22.3% 對 36.3%），只當第 2 名才有一點用。

延遲（`stats --by context`，每題新開 session、依序送，連續請求；停頓後的第一個請求見下節）：

| 前文 | 請求 | 出錯 | p50 ms | p95 ms | 最慢 ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| 不給前文（維基、口語、個人集） | 11155 | 86 | 316 | 749 | 2042 |
| 前文 0 | 3025 | 34 | 343 | 817 | 1537 |
| 前文 50 | 3025 | 75 | 352 | 847 | 2183 |
| 前文 200 | 3025 | 97 | 362 | 820 | 1680 |
| 合計 | 20230 | 292 | 333 | 788 | 2183 |

M2 票 10 在 iPhone 18 Pro Max 上用短提示量的連續請求是 185–194 ms；這裡的提示長（9 條整句），機器又忙，p50 比它慢。

### Foundation Models 停頓後第一個請求

2026-11-11 14:20–15:02（load average 約 5）。同一個程序裡依序停 5、10、20、30、60、120 秒，各送 3 個請求（停頓後第 1、2、3 個），整輪 8 次；提示取 test 的 `fm-one-paths` 輪流用。另起 6 個新程序（每個前面停 60 秒），各送 3 個。計時從 `respond` 開始，不含建 session（同 M2 票 10）。

| 停頓 | 停頓後第 1 個：p50 / 最慢 ms | 超過 2 s | 第 2 個：p50 / 最慢 ms | 第 3 個：p50 / 最慢 ms |
| ---: | ---: | ---: | ---: | ---: |
| 5 s | 967 / 2074 | 1 / 8 | 395 / 1329 | 372 / 799 |
| 10 s | 1327 / 1513 | 0 / 8 | 399 / 1050 | 337 / 895 |
| 20 s | 1284 / 2126 | 1 / 8 | 361 / 623 | 346 / 623 |
| 30 s | 1294 / 2114 | 1 / 8 | 350 / 631 | 350 / 839 |
| 60 s | 1541 / 2481 | 2 / 8 | 348 / 708 | 380 / 808 |
| 120 s | 1235 / 2772 | 2 / 8 | 344 / 518 | 396 / 686 |
| 新程序 | 768 / 774 | 0 / 6 | 289 / 295 | 261 / 334 |

- 停頓後第一個請求 48 次裡 7 次超過 2 s（**14.6%**），p50 約 1.3 s；停 5 秒就會變慢，停得更久沒有明顯更慢。最慢 2.8 s，沒有出現 M2 在 iPhone 上 12 次裡 4 次的 6–15 s。
- 新程序的第一個請求反而穩定在約 770 ms：慢的不是載入模型，而是同一個程序閒置後再送。沒有查原因（`prewarm()` 在 iPhone 上也擋不住，M2 票 10）。
- 停頓後的第 2、3 個請求回到連續請求的水準（p50 約 350 ms）。

## 與門檻比較

照 spec〈採用門檻〉，test 切分（基線：口語前 2 名 49.1%、個人 85.5%、維基 55.1%）。採用與否由階梯結論票（票 11）連同票 08、09 一起定，這裡只對門檻。

| 門檻 | DeepSeek flash | Foundation Models |
| --- | --- | --- |
| 口語前 2 名（雲端 ≥ +6.0 點、停鍵 ≥ +4.0 點） | **過**：59.5%（+10.4），p < 0.001 | **沒過**：50.4%（+1.3），p = 0.001 |
| 比已採用的本地停鍵層多（雲端 +3.0 點；FM 比神經模型 +2.0 點） | 待票 09：Foundation Models 沒過，本地停鍵層看神經模型 | 待票 09 |
| 個人集不退步（前 2 名 ≥ 85.0%；首選、修正次數重排改不了） | **沒量**：使用者決定不送雲端 | 過：86.1% |
| 維基不退步（前 2 名 ≥ 54.1%） | 過：65.6% | 過：59.1% |
| 延遲（雲端 p95 ≤ 1.5 s、逾時 3 s ≤ 2%；FM p50 ≤ 300 ms、p95 ≤ 800 ms） | 過：p95 1,083 ms，逾時 0% | p50 333 ms **沒過**、p95 788 ms 過（機器忙） |
| FM 停頓後第一個請求超過 2 s ≤ 5% | | **沒過**：14.6% |
| 費用（每千次 ≤ 3 元，高峰價） | 過：0.43 元 | 免費 |

- Foundation Models：正確率與停頓後的延遲都沒過，照規則不採用。
- DeepSeek flash：正確率、延遲、費用都過；缺「個人集不退步」（沒量）與「比本地停鍵層多 3.0 點」（等票 09）兩項，由票 11 定。
- 前文長度（brief §8 的 200 字）：有前文組的前 2 名，flash 送 200 字比 50 字多 0.2 點（67.0% 對 66.8%），Foundation Models 多 0.3 點（60.5% 對 60.2%），都不到 1.0 點，照規則改成 50 字。口語的長前文本來就短（`../model-ladder/README.md`〈限制〉），差別幾乎都來自維基。

給票 11：

- 定案的選項是 lattice 路徑（`paths`），產品要讓 `apply_rerank` 能把候選列以外的路徑插進第 2 位（`../model-ladder/README.md`〈重排模型看什麼〉）；只給候選列第一頁時兩個模型都比基線差（〈dev 調參〉）。雲端重排票（票 22）的提示照搬 `hint-paths`：`STYLES["hint"]` 的指示、上面〈提示〉的 user 訊息、選項是前 9 條路徑。
- 若 ADR 0005 讓雲端結果也能動首選，flash 的口語首選會從 36.3% 變成 54.6%（可動首選欄）；Foundation Models 動首選會變差。
- 每日上限的預設值（票 22）：每千次 0.43 元（高峰價），每天 1,000 次約 0.4 元。

## 限制

- 只在打完整串輸入碼時重排一次，和框架相同（`../model-ladder/README.md`〈限制〉）；雲端 p50 約 0.74 s，使用者在結果回來前就選字的比例沒量。
- 延遲是這台 Mac 在 load average 5–10 時量的，Foundation Models 的連續請求在空著的機器可能更快；iPhone 上的停頓尾巴要在 M7 接上時重量。雲端延遲取決於網路，只量了這台機器、這一天。
- DeepSeek 的回覆用 `temperature: 0`，同一題重送不保證相同；調參的 500 題只送一次。
- 調參的子集合 500 題，寫法之間差 1–2 點時分不出高下（`hint-paths` 與 `hint2-paths` 同分）。
- `deepseek-v4-pro` 沒量（上面〈模型與環境〉）。
