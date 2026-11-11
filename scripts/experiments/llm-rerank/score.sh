#!/bin/sh
# 一種寫法的回覆打分（README〈怎麼重跑〉）：回覆換成結果、`model-ladder eval`、只比提示檔裡的題。
#   score.sh <dev|test> <提示.jsonl> <回覆.jsonl> <名> [逾時 ms]
# 在 repo 根目錄執行；逐題結果寫到 build/model-ladder/<切分>/<名>/。
set -eu
split=$1 prompts=$2 replies=$3 name=$4 deadline=${5:-}
here=$(dirname "$0")
orders=build/llm-rerank/orders/$split-$name.jsonl
mkdir -p build/llm-rerank/orders
python3 "$here/llm.py" orders "$prompts" "$replies" -o "$orders" ${deadline:+--deadline-ms "$deadline"}
(cd "$here/../model-ladder" && cargo run --release -q -- eval --split "$split" \
  --orders "../../../$orders" --name "$name") > "build/llm-rerank/orders/$split-$name.eval.md"
grep -E '以外的文字|沒有外部' "build/llm-rerank/orders/$split-$name.eval.md" || true
python3 "$here/llm.py" subset "$prompts" "build/model-ladder/$split/baseline" "build/model-ladder/$split/$name"
