#!/bin/sh
# 在 dev 切分上跑 β（word_bonus）× γ（segment_penalty）的格點，每格一列 TSV（M1 票 12）。
#
# 用法（repo 根目錄，先跑過 `cargo xtask data` 與一次 `cargo xtask bench --dev`）：
#   scripts/experiments/bigram-tuning/grid.sh "<β 值…>" "<γ 值…>" [其他覆寫，如 beam=40] > out.tsv
#
# 輸出欄：β、γ、固定題通過數，再依集合順序每個集合「答錯、修正」兩欄，最後是
# 口語兩集（common-voice-yue、hkcancor）與維基兩集各自的答錯、修正總和，與兩組的短輸入
# （dev 句子的單字、句子裡出現的詞庫的詞，依次數加權）首選正確率。
# 每格是一個 `qim-bench --dev` 行程，同時跑 `JOBS` 個（預設 CPU 數）。
# 下面的 awk 靠報表的欄號（`$6` 答錯、`$9` 修正，`crates/qim-bench/src/report.rs` 的 `METRICS_HEADER`）解析，報表欄位改時一起改。
set -eu
bin=target/release/qim-bench

if [ "${1:-}" = "--cell" ]; then
  b=$2 g=$3
  set -- --dev --set "word_bonus=$b" --set "segment_penalty=$g"
  for kv in ${GRID_SET:-}; do set -- "$@" --set "$kv"; done
  "$bin" "$@" | awk -F'|' -v b="$b" -v g="$g" '
    /^\| fixed \|/ { split($4, f, "（"); gsub(/ /, "", f[1]); fixed = f[1] }
    NF > 12 && /^\| (yue-wiki|zh-wiki|common-voice-yue|hkcancor) \|/ {
      name = $2; w = $6 + 0; c = $9 + 0
      row = row "\t" w "\t" c
      if (name ~ /common-voice|hkcancor/) { cw += w; cc += c } else { ww += w; wc += c }
    }
    /·(單字|詞)（加權）/ {
      split($3, n, " "); hit = $4 + 0; total = n[1] + 0; k = ($2 ~ /單字/) ? 1 : 2
      g2 = ($2 ~ /common-voice|hkcancor/) ? "c" : "w"
      sum[g2 k] += hit * total; cnt[g2 k] += total
    }
    END {
      printf "%s\t%s\t%s%s\t%d\t%d\t%d\t%d", b, g, fixed, row, cw, cc, ww, wc
      printf "\t%.2f\t%.2f\t%.2f\t%.2f\n", sum["c1"] / cnt["c1"], sum["w1"] / cnt["w1"], sum["c2"] / cnt["c2"], sum["w2"] / cnt["w2"]
    }'
  exit
fi

betas=$1
gammas=$2
shift 2
GRID_SET="$*"
export GRID_SET
[ -x "$bin" ] || { echo "先跑一次 cargo xtask bench --dev 編出 $bin" >&2; exit 1; }
jobs=${JOBS:-$(sysctl -n hw.ncpu 2>/dev/null || nproc)}
printf 'beta\tgamma\tfixed\tyue_w\tyue_c\tzh_w\tzh_c\tcv_w\tcv_c\thk_w\thk_c\tcolloq_w\tcolloq_c\twiki_w\twiki_c\tcolloq_1\twiki_1\tcolloq_2\twiki_2\n'
for b in $betas; do for g in $gammas; do echo "$b $g"; done; done |
  xargs -P "$jobs" -L 1 "$0" --cell |
  sort -k1,1n -k2,2n
