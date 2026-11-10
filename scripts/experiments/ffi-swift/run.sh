#!/bin/sh
# 產生 Swift 綁定、編 Swift 程式、打 anr，再用 qim-bench 的同一批輸入量每鍵延遲。用法見 README.md。
set -eu
cd "$(dirname "$0")/../../.."

cargo xtask ffi
mkdir -p build/ffi/bin
swiftc -O -o build/ffi/bin/qim-swift scripts/experiments/ffi-swift/main.swift build/ffi/swift/qim.swift \
	-Xcc -fmodule-map-file=build/ffi/swift/module.modulemap -I build/ffi/swift -L target/release -lqim_ffi

# 基準載入的四份資料包（qim-bench 的 load_engine）。
set -- build/packs/quick/quick.pack build/packs/quick-lexicon/quick-lexicon.pack \
	build/packs/zh-hant-bigram/zh-hant-bigram.pack build/packs/scowl-en-us/scowl-en-us.pack
build/ffi/bin/qim-swift demo "$@"

# 與 qim-bench 同序：固定題、bench/corpus/*.tsv、bench/colloquial/*.tsv（各目錄依檔名），第一欄去空白。
{
	grep -v '^#' bench/fixed.tsv | awk -F '\t' 'NF { print $1 }'
	for f in $(ls bench/corpus/*.tsv | sort) $(ls bench/colloquial/*.tsv | sort); do
		awk -F '\t' 'NF { gsub(/ /, "", $1); print $1 }' "$f"
	done
} | build/ffi/bin/qim-swift latency "$@"
