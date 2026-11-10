#!/bin/sh
# iOS：寫出模型檔（還沒有時）並編 aarch64-apple-ios 的靜態庫，給 ../ios-keyboard 連結。
set -eu
cd "$(dirname "$0")"
rustup target add aarch64-apple-ios >/dev/null
cargo build --release -q
[ -f models/gpt-23m-q8.gguf ] || ./target/release/neural-cost gen models
cargo build --release -q --target aarch64-apple-ios --lib
ls -l target/aarch64-apple-ios/release/libneural_cost.a models
