#!/bin/sh
# 在 dev 切分的「有前文」題上比三種前文規則（M3 票 04）：不給前文、跨過標點接上一句的
# 最後一字、接句首（現行的 `Engine::context_char`）。
#
# 用法（repo 根目錄，先跑過 `cargo xtask data`）：
#   scripts/experiments/context-rule/run.sh
#
# 「接句首」與「不給前文」直接用 `qim-bench --dev` 的 `<名>·前文`、`<名>·前文·不給`。
# 「跨過標點」不改核心：把每題前文結尾的標點、空白、符號（不是字母也不是數字的字元）拿掉，
# 前文就以上一句的最後一字結尾，現行規則照樣接它的 bigram，和票 04 先試過的做法相同；
# 前文的最後一字沒有統計時兩者都用 unigram。改寫的是 `build/corpus/<名>/dev-context.tsv`，
# 跑完（含中斷）放回原檔。
# 輸出：每個集合一列「規則、集合、題數、答錯、修正」，TSV。
set -eu
bin=target/release/qim-bench
cargo build -q --release -p qim-bench

table() {
  # $1 是規則名；從 `--dev` 報告的「有前文的題」表取給前文與不給前文兩欄。
  "$bin" --dev | awk -F'|' -v rule="$1" '
    /^有前文的題/ { on = 1; next }
    on && /^\| [^-]/ && $2 !~ /題目/ {
      name = $2; gsub(/ /, "", name)
      if (rule == "none") print "不給前文\t" name "\t" $3+0 "\t" $5+0 "\t" $7+0
      else print rule "\t" name "\t" $3+0 "\t" $4+0 "\t" $6+0
    }
    on && /^$/ && seen { on = 0 } on { seen = 1 }'
}

printf '規則\t集合\t題數\t答錯\t修正\n'
table none
table 接句首

files=$(ls build/corpus/*/dev-context.tsv)
for f in $files; do cp "$f" "$f.orig"; done
restore() { for f in $files; do [ -f "$f.orig" ] && mv "$f.orig" "$f"; done; }
trap restore EXIT INT TERM
for f in $files; do
  # Python 的 isalnum 和 Rust 的 char::is_alphanumeric 在組合記號上略有不同，前文的結尾是漢字
  # 與常見標點時相同。
  python3 -I -c '
import sys
out = []
for line in open(sys.argv[1], encoding="utf-8"):
    before, rest = line.split("\t", 1)
    while before and not before[-1].isalnum():
        before = before[:-1]
    out.append(before + "\t" + rest)
open(sys.argv[1], "w", encoding="utf-8").write("".join(out))
' "$f"
done
table 跨過標點
