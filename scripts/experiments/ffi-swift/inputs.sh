#!/bin/sh
# 印出每鍵延遲的那批輸入，一行一個：與 qim-bench 同序，固定題、bench/corpus/*.tsv、bench/colloquial/*.tsv
# （各目錄依檔名），第一欄去空白。在 repo 根目錄跑。
set -eu
grep -v '^#' bench/fixed.tsv | awk -F '\t' 'NF { print $1 }'
for f in $(ls bench/corpus/*.tsv | sort) $(ls bench/colloquial/*.tsv | sort); do
	awk -F '\t' 'NF { gsub(/ /, "", $1); print $1 }' "$f"
done
