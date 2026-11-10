#!/bin/sh
# Mac：寫出四個隨機權重的模型檔到 models/，每個模型各開 RUNS 個新程序量（預設 3），每次 ITER 次重排（預設 30）、THREADS 條執行緒（預設 2）。
# PREFIX=unshared 改量不共用上下文的對照（README〈前綴快取〉）。
set -eu
cd "$(dirname "$0")"
runs=${RUNS:-3}
iter=${ITER:-30}
threads=${THREADS:-2}
prefix=${PREFIX:-}
cargo build --release -q
[ -f models/gpt-23m-q8.gguf ] || ./target/release/neural-cost gen models
for model in lstm-1.4m-f32.safetensors lstm-1.4m-q8.gguf gpt-23m-f32.safetensors gpt-23m-q8.gguf; do
  i=1
  while [ "$i" -le "$runs" ]; do
    echo "== $model run $i"
    # shellcheck disable=SC2086 # prefix 空時不傳
    ./target/release/neural-cost run "models/$model" "$iter" "$threads" $prefix
    i=$((i + 1))
  done
done
