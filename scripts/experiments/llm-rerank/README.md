# llm-rerank：Foundation Models 與雲端 LLM 的重排（M3 票 10）

- 日期：2026-11-11；機器：Apple M6（Mac18,5，32 GB）、macOS 27.0.1（26A434）；Swift 6.4.2；Python 3（只用標準函式庫）。
- commit：開工 `fa07bd3`。README 不寫含它自己的 commit，之後重跑用 `git log -- scripts/experiments/llm-rerank` 對。
- 問題（`.scratch/m3/spec.md`〈模型階梯實驗〉）：停鍵層的 Apple Foundation Models、雲端層的 DeepSeek 重排，前 2 名命中率能補多少，延遲與費用是否過〈採用門檻〉。框架、基線與指標定義在 `../model-ladder/README.md`。

## 內容

| 路徑 | 內容 |
| --- | --- |
| `llm.py` | 組提示、送 DeepSeek、回覆換回文字、延遲與費用統計、費用估計 |
| `test_llm.py` | `llm.py` 的測試：`python3 -m unittest test_llm` |
| `fm/fm-rerank.swift` | Foundation Models 的命令列：逐題送出（`run`）、停頓後第一個請求的延遲（`tail`） |

## 怎麼重跑

在 repo 根目錄：

```sh
cargo xtask data                                       # 題目與長前文題
(cd scripts/experiments/model-ladder && cargo run --release -- eval && cargo run --release -- export)
swiftc -O -swift-version 6 scripts/experiments/llm-rerank/fm/fm-rerank.swift -o build/llm-rerank/fm-rerank
L=scripts/experiments/llm-rerank/llm.py; R=build/model-ladder/dev/requests; T=build/llm-rerank/tune

# dev 調參的題：口語 dev 不給前文 300 題 + 口語前文 200 字 200 題（種子 1），每種寫法一份
python3 $L prompts --style one --choices merge --sample 300 $R/common-voice-yue.jsonl $R/hkcancor.jsonl -o $T/plain-one-merge.jsonl
python3 $L prompts --style one --choices merge --sample 200 "$R/common-voice-yue·前文200.jsonl" "$R/hkcancor·前文200.jsonl" -o $T/ctx-one-merge.jsonl
cat $T/plain-one-merge.jsonl $T/ctx-one-merge.jsonl > $T/one-merge.jsonl

# Foundation Models
build/llm-rerank/fm-rerank info
build/llm-rerank/fm-rerank run $T/one-merge.jsonl build/llm-rerank/fm/one-merge.replies.jsonl
build/llm-rerank/fm-rerank tail $T/one-merge.jsonl build/llm-rerank/fm/tail.jsonl --gaps 2,5,10,30,60,120,300 --repeat 3

# DeepSeek（key 在 repo 外的檔，只進這個指令的環境變數）
(set -a; . ~/.config/qim/deepseek.env; python3 $L cloud --model deepseek-flash $T/one-merge.jsonl build/llm-rerank/flash/one-merge.replies.jsonl)

# 回覆換成結果、交給框架
python3 $L orders $T/one-merge.jsonl <回覆.jsonl> -o <結果.jsonl> [--deadline-ms 3000]
(cd scripts/experiments/model-ladder && cargo run --release -- eval --orders ../../../<結果.jsonl> --name <名>)
python3 $L stats <回覆.jsonl> [--model deepseek-flash]
python3 $L tail-stats build/llm-rerank/fm/tail.jsonl
```

- `cloud` 拒收個人集的提示，除非加 `--allow-personal`（每次先問使用者，plan §5.2）；回覆檔已有的題不重送。
- `eval --orders` 只算結果檔裡有的題；調參的子集合用 `eval` 的各集合列不準（沒回的題照原順序），所以 dev 的子集合直接用 `model-ladder` 的逐題結果算（下面〈dev 調參〉）。

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

設定在 dev 定案後各跑一次：DeepSeek flash `hint-paths` 在 2026-11-11 11:52–12:24（北京時間，平日高峰時段）送 test 的 15,713 題；Foundation Models `one-paths` 跑 test 的 20,230 題（含個人集，只在本機）。基線在同一個 commit 重量，和 `../model-ladder/README.md` 的數字相同。

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

延遲與費用（每千次照高峰價；實際送出時正好是高峰時段）：

| 前文 | 請求 | p50 ms | p95 ms | 最慢 ms | 每千次（元） |
| --- | ---: | ---: | ---: | ---: | ---: |
| 不給前文（維基、口語集） | 6638 | 734 | 1062 | 1695 | 0.414 |
| 前文 0 | 3025 | 730 | 1087 | 2055 | 0.381 |
| 前文 50 | 3025 | 750 | 1095 | 2397 | 0.438 |
| 前文 200 | 3025 | 764 | 1094 | 2069 | 0.509 |
| 合計 | 15713 | 742 | 1083 | 2397 | 0.430 |

沒有出錯、沒有沒序號的回覆，超過 3 s 的 0 題（超過 2 s 的 3 題）。快取命中只有 2.5%（84,509 / 3,400,529 個輸入 token）：每題的 system 訊息相同，但 DeepSeek 的快取要偵測到共同前綴才建，命中是盡力而為。test 實花 6.76 元，加上調參約 1.1 元，合計約 7.9 元。
