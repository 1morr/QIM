#!/bin/sh
# 編有無 Schema 驗證兩個版本，各跑 RUNS 次（預設 3）；先在 repo 根目錄跑過 cargo xtask data。
set -eu
cd "$(dirname "$0")"
runs=${RUNS:-3}
for variant in plain schema; do
  if [ "$variant" = schema ]; then features="--features schema"; else features=""; fi
  # shellcheck disable=SC2086 # features 要拆成兩個參數
  cargo build --release -q $features
  cp target/release/pack-memory "target/pack-memory-$variant"
done
for variant in plain schema; do
  i=1
  while [ "$i" -le "$runs" ]; do
    echo "== $variant 第 $i 次"
    "./target/pack-memory-$variant" "$@"
    i=$((i + 1))
  done
done
