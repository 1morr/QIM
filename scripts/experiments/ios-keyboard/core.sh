#!/bin/sh
# 票 08：備好鍵盤擴充接上核心要的檔，再跑 xcodegen。用法見 README.md〈接上核心（票 08）〉。
set -eu
cd "$(dirname "$0")/../../.."

cargo xtask ffi --ios
scripts/experiments/ffi-swift/inputs.sh > build/ffi/latency-inputs.txt
for id in quick quick-lexicon zh-hant-bigram zh-hant-word-trigram zh-hant-word-trigram-large opencc-hk2s scowl-en-us cangjie-hint; do
	test -f "build/packs/$id/$id.pack" || { echo "缺 build/packs/$id/$id.pack，先跑 cargo xtask data" >&2; exit 1; }
done
cd scripts/experiments/ios-keyboard
xcodegen generate
