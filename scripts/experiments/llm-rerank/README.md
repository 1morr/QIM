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
